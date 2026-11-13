use crate::common;

use serde_json::Value;

async fn seed_client(
    app: &common::TestApp,
    id: &str,
    name: &str,
    client_type: &str,
    annual_revenue: f64,
) {
    sqlx::query(
        r#"
        INSERT INTO clients (id, company_id, name, client_type, annual_revenue)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
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

async fn seed_interaction(app: &common::TestApp, id: &str, client_id: &str, subject: &str) {
    sqlx::query(
        r#"
        INSERT INTO interactions (
            id, company_id, client_id, interaction_type, subject, interaction_date
        )
        VALUES (?, ?, ?, 'call', ?, '2026-01-01')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(client_id)
    .bind(subject)
    .execute(&app.pool)
    .await
    .expect("seed interaction");
}

#[tokio::test]
async fn s81_clients_overview_calculates_stats_on_backend() {
    let app = common::TestApp::new().await;

    seed_client(&app, "active", "活跃客户", "active", 1000.0).await;
    seed_client(&app, "prospect", "潜在客户", "prospect", 2000.0).await;
    seed_client(&app, "inactive", "休眠客户", "inactive", 3000.0).await;
    seed_contact(&app, "contact-a", "active", "联系人 A").await;
    seed_interaction(&app, "interaction-a", "active", "电话沟通").await;

    let res = app.get("/api/v1/clients/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total"].as_i64().unwrap(), 3);
    assert_eq!(body["stats"]["active"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["prospect"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["total_revenue"].as_f64().unwrap(), 6000.0);
    assert_eq!(body["clients"].as_array().unwrap().len(), 3);
    assert_eq!(body["contacts"].as_array().unwrap().len(), 1);
    assert_eq!(body["interactions"].as_array().unwrap().len(), 1);
    assert_eq!(body["create_action"]["method"], "POST");
    assert_eq!(body["create_action"]["endpoint"], "/api/v1/clients");
    assert_eq!(body["create_action"]["label"], "新增客户");
    assert_eq!(body["create_action"]["success_title"], "创建成功");
    assert_eq!(body["create_action"]["success_message"], "客户创建成功");

    let clients = body["clients"].as_array().unwrap();
    let active = clients
        .iter()
        .find(|client| client["id"] == "active")
        .expect("active client row");
    assert_eq!(active["update_action"]["method"], "PUT");
    assert_eq!(
        active["update_action"]["endpoint"],
        "/api/v1/clients/active"
    );
    assert_eq!(active["update_action"]["label"], "保存修改");
    assert_eq!(active["update_action"]["success_title"], "更新成功");
    assert_eq!(active["update_action"]["success_message"], "客户信息已更新");
    assert_eq!(active["can_delete"].as_bool().unwrap(), true);
    assert_eq!(active["delete_action"]["method"], "DELETE");
    assert_eq!(
        active["delete_action"]["endpoint"],
        "/api/v1/clients/active"
    );
    assert_eq!(
        active["delete_action"]["confirm_message"],
        "确定要删除客户「活跃客户」吗？"
    );
    assert_eq!(active["delete_action"]["success_title"], "删除成功");
    assert_eq!(active["delete_action"]["success_message"], "客户已删除");

    let detail = app
        .get("/api/v1/clients/overview?selected_client_id=active")
        .await;
    detail.assert_status_ok();
    let detail = detail.json::<Value>();
    let detail_client = &detail["detail_client"];
    assert_eq!(detail_client["update_action"]["method"], "PUT");
    assert_eq!(
        detail_client["update_action"]["endpoint"],
        "/api/v1/clients/active"
    );
    assert_eq!(detail_client["interaction_create_action"]["method"], "POST");
    assert_eq!(
        detail_client["interaction_create_action"]["endpoint"],
        "/api/v1/interactions"
    );
    assert_eq!(
        detail_client["interaction_create_action"]["label"],
        "添加互动记录"
    );
    assert_eq!(
        detail_client["interaction_create_action"]["success_message"],
        "互动记录已添加"
    );
    assert_eq!(
        detail_client["interaction_create_action"]["default_values"]["client_id"],
        "active"
    );
    assert_eq!(detail_client["contact_create_action"]["method"], "POST");
    assert_eq!(
        detail_client["contact_create_action"]["endpoint"],
        "/api/v1/contacts"
    );
    assert_eq!(
        detail_client["contact_create_action"]["label"],
        "添加联系人"
    );
    assert_eq!(
        detail_client["contact_create_action"]["success_message"],
        "联系人已添加"
    );
    assert_eq!(
        detail_client["contact_create_action"]["default_values"]["client_id"],
        "active"
    );
}

#[tokio::test]
async fn s81_clients_overview_is_company_scoped() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_client(&app, "owned", "本公司客户", "active", 1000.0).await;
    seed_client(&other, "other", "其他公司客户", "active", 9000.0).await;

    let res = app.get("/api/v1/clients/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["total_revenue"].as_f64().unwrap(), 1000.0);
    assert_eq!(body["clients"].as_array().unwrap().len(), 1);
    assert_eq!(body["clients"][0]["id"], "owned");
}
