use crate::common;
use axum::http::StatusCode;
use serde_json::{Value, json};

#[tokio::test]
async fn s140_empty_month_returns_placeholder_report() {
    let app = common::TestApp::new().await;
    let created = app
        .post("/api/v1/reports/monthly-close?period=2026-01")
        .json(&json!({}))
        .await;
    created.assert_status_ok();
    let body = created.json::<Value>();
    assert_eq!(body["period"], "2026-01");
    assert_eq!(body["empty"], true);
    assert_eq!(body["income"], 0.0);
    assert_eq!(body["expense"], 0.0);
    let file_id = body["file"]["id"].as_str().unwrap();
    let download = app.get(&format!("/api/v1/files/{file_id}")).await;
    download.assert_status_ok();
    let bytes = download.as_bytes();
    assert!(bytes.starts_with(b"%PDF"), "monthly close must be a PDF");
}

#[tokio::test]
async fn s140_month_with_cash_and_receivables_matches_source_rows() {
    let app = common::TestApp::new().await;
    app.post("/api/v1/payments")
        .json(&json!({ "status": "completed",
            "description": "Client fee",
            "amount": 8000.0,
            "payment_type": "income",
            "payment_date": "2026-03-08"
        }))
        .await
        .assert_status_ok();
    app.post("/api/v1/payments")
        .json(&json!({ "status": "completed",
            "description": "Office",
            "amount": 2000.0,
            "payment_type": "expense",
            "payment_date": "2026-03-09"
        }))
        .await
        .assert_status_ok();
    app.post("/api/v1/invoices")
        .json(&json!({
            "invoice_number": "AR-CLOSE-1",
            "invoice_type": "receivable",
            "amount": 1500.0,
            "status": "sent",
            "invoice_date": "2026-03-02",
            "due_date": "2026-02-01",
            "supplier": "Acme"
        }))
        .await
        .assert_status_ok();

    let body = app
        .post("/api/v1/reports/monthly-close?period=2026-03")
        .json(&json!({}))
        .await
        .json::<Value>();
    assert_eq!(body["empty"], false);
    assert_eq!(body["income"], 8000.0);
    assert_eq!(body["expense"], 2000.0);
    assert_eq!(body["net"], 6000.0);
    assert_eq!(body["billed"], 1500.0);
    assert!(body["outstanding"].as_f64().unwrap() >= 1500.0);
    assert!(body["overdue"].as_f64().unwrap() >= 1500.0);

    let list = app.get("/api/v1/reports/monthly-close").await;
    list.assert_status_ok();
    let listed = list.json::<Value>();
    assert_eq!(listed["data"][0]["period"], "2026-03");
}

#[tokio::test]
async fn s140_rejects_invalid_period_and_isolates_tenants() {
    let app = common::TestApp::new().await;
    app.post("/api/v1/reports/monthly-close?period=2026-99")
        .json(&json!({}))
        .await
        .assert_status(StatusCode::BAD_REQUEST);

    app.post("/api/v1/reports/monthly-close?period=2026-04")
        .json(&json!({}))
        .await
        .assert_status_ok();

    let other = app.create_second_tenant().await;
    let other_list = other.get("/api/v1/reports/monthly-close").await;
    other_list.assert_status_ok();
    assert!(
        other_list.json::<Value>()["data"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}
