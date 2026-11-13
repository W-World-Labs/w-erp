use crate::common;

use serde_json::Value;

async fn seed_client(
    app: &common::TestApp,
    id: &str,
    name: &str,
    email: &str,
    client_type: &str,
    annual_revenue: f64,
) {
    sqlx::query(
        r#"
        INSERT INTO clients (id, company_id, name, email, client_type, annual_revenue)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
    .bind(email)
    .bind(client_type)
    .bind(annual_revenue)
    .execute(&app.pool)
    .await
    .expect("seed client");
}

async fn seed_contact(app: &common::TestApp, id: &str, client_id: &str, name: &str) {
    sqlx::query(
        r#"
        INSERT INTO contacts (id, company_id, client_id, name, email)
        VALUES (?, ?, ?, ?, 'contact@example.com')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(client_id)
    .bind(name)
    .execute(&app.pool)
    .await
    .expect("seed contact");
}

async fn seed_interaction(
    app: &common::TestApp,
    id: &str,
    client_id: &str,
    interaction_type: &str,
    subject: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO interactions (
            id, company_id, client_id, interaction_type, subject, interaction_date
        )
        VALUES (?, ?, ?, ?, ?, '2026-01-01')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(client_id)
    .bind(interaction_type)
    .bind(subject)
    .execute(&app.pool)
    .await
    .expect("seed interaction");
}

#[tokio::test]
async fn s97_clients_overview_filters_labels_and_detail_on_backend() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_client(
        &app,
        "client-needle",
        "Needle Labs",
        "needle@example.com",
        "active",
        125_000.0,
    )
    .await;
    seed_client(
        &app,
        "client-prospect",
        "Prospect Co",
        "prospect@example.com",
        "prospect",
        80_000.0,
    )
    .await;
    seed_client(
        &other,
        "client-other",
        "Needle Other",
        "needle.other@example.com",
        "active",
        900_000.0,
    )
    .await;
    seed_contact(&app, "contact-needle", "client-needle", "Needle Contact").await;
    seed_contact(
        &app,
        "contact-prospect",
        "client-prospect",
        "Prospect Contact",
    )
    .await;
    seed_interaction(
        &app,
        "interaction-needle",
        "client-needle",
        "meeting",
        "Backend meeting",
    )
    .await;
    seed_interaction(
        &app,
        "interaction-prospect",
        "client-prospect",
        "call",
        "Prospect call",
    )
    .await;

    let res = app
        .get("/api/v1/clients/overview?search=needle&selected_client_id=client-needle")
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total"].as_i64().unwrap(), 2);
    assert_eq!(body["stats"]["active"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["prospect"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["total_revenue"].as_f64().unwrap(), 205_000.0);
    assert_eq!(body["stats"]["total_revenue_label"], "¥20.5万");

    let clients = body["clients"].as_array().unwrap();
    assert_eq!(clients.len(), 1);
    assert_eq!(clients[0]["id"], "client-needle");
    assert_eq!(clients[0]["type_badge"]["label"], "活跃客户");
    assert_eq!(clients[0]["annual_revenue_label"], "¥125,000");

    assert_eq!(body["detail_client"]["id"], "client-needle");
    assert_eq!(body["detail_contacts"].as_array().unwrap().len(), 1);
    assert_eq!(body["detail_contacts"][0]["id"], "contact-needle");
    assert_eq!(body["detail_interactions"].as_array().unwrap().len(), 1);
    assert_eq!(
        body["detail_interactions"][0]["interaction_badge"]["icon"],
        "Users"
    );
    assert!(
        body["type_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| { option["value"] == "churned" && option["label"] == "流失客户" })
    );
    assert_eq!(body["source_options"][0]["value"], "referral");
    assert_eq!(body["source_options"][0]["label"], "转介绍");
    assert!(clients.iter().all(|client| client["id"] != "client-other"));
}
