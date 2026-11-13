use crate::common;

use chrono::{Datelike, Utc};
use serde_json::Value;

async fn seed_client(app: &common::TestApp, id: &str, name: &str) {
    sqlx::query(
        r#"
        INSERT INTO clients (id, company_id, name, client_type)
        VALUES (?, ?, ?, 'active')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
    .execute(&app.pool)
    .await
    .expect("seed client");
}

async fn seed_quote(
    app: &common::TestApp,
    id: &str,
    client_id: &str,
    quote_number: &str,
    status: &str,
    total: f64,
    invoice_id: Option<&str>,
) {
    sqlx::query(
        r#"
        INSERT INTO quotes (
            id, company_id, client_id, quote_number, title, status, total, invoice_id
        )
        VALUES (?, ?, ?, ?, '报价', ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(client_id)
    .bind(quote_number)
    .bind(status)
    .bind(total)
    .bind(invoice_id)
    .execute(&app.pool)
    .await
    .expect("seed quote");
}

fn assert_f64(actual: &Value, expected: f64) {
    let actual = actual.as_f64().expect("number");
    assert!(
        (actual - expected).abs() < 0.001,
        "expected {expected}, got {actual}"
    );
}

#[tokio::test]
async fn s89_quotes_overview_calculates_stats_next_number_and_row_labels_on_backend() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;
    let year = Utc::now().year();

    seed_client(&app, "client-a", "Alpha 客户").await;
    seed_client(&app, "client-b", "Beta 客户").await;
    seed_client(&other, "client-other", "Other 客户").await;

    seed_quote(
        &app,
        "quote-draft",
        "client-a",
        &format!("QT-{year}-001"),
        "draft",
        100_000.0,
        None,
    )
    .await;
    seed_quote(
        &app,
        "quote-sent",
        "client-b",
        &format!("QT-{year}-002"),
        "sent",
        200_000.0,
        None,
    )
    .await;
    seed_quote(
        &app,
        "quote-accepted",
        "client-a",
        &format!("QT-{year}-003"),
        "accepted",
        300_000.0,
        None,
    )
    .await;
    seed_quote(
        &other,
        "quote-other",
        "client-other",
        &format!("QT-{year}-099"),
        "draft",
        999_000.0,
        None,
    )
    .await;

    let res = app.get("/api/v1/quotes/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total"].as_i64().unwrap(), 3);
    assert_eq!(body["stats"]["draft"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["sent"].as_i64().unwrap(), 1);
    assert_f64(&body["stats"]["total_amount"], 600_000.0);
    assert_eq!(
        body["next_quote_number"].as_str().unwrap(),
        format!("QUO-{year}-0001")
    );

    let accepted = body["quotes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|quote| quote["id"] == "quote-accepted")
        .expect("accepted quote");
    assert_eq!(accepted["client_name"], "Alpha 客户");
    assert_eq!(accepted["status_label"], "已接受");
    assert_eq!(accepted["can_convert_to_invoice"], true);

    let clients = body["clients"].as_array().unwrap();
    assert_eq!(clients.len(), 2);
    assert!(clients.iter().all(|client| client["id"] != "client-other"));
    assert!(
        body["status_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|status| status["value"] == "accepted")
    );
}
