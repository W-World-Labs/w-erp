/// Scenario 69: backend-owned bulk imports.
/// Frontend may upload a file and display results, but parsing, validation,
/// table selection, row mapping, and writes must run in the backend.
use crate::common;

use axum::http::StatusCode;
use base64::{Engine as _, engine::general_purpose};
use serde_json::{Value, json};

#[tokio::test]
async fn s69_bulk_import_requires_auth() {
    let app = common::TestApp::new().await;

    let res = app
        .server
        .post("/api/v1/imports/bulk/invoices")
        .json(&json!({
            "filename": "invoices.csv",
            "file_base64": encode_csv("invoice_number,amount\nINV-1,100\n"),
        }))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn s69_bulk_import_csv_invoices_in_backend() {
    let app = common::TestApp::new().await;

    let csv = "\
invoice_number,invoice_type,amount,tax_amount,status,invoice_date,due_date,supplier,description
BULK-001,expense,100.50,13.00,pending,2026-05-09,2026-05-20,Vendor A,Backend import one
BULK-002,income,200.00,26.00,paid,2026-05-10,2026-05-30,Client B,Backend import two
";

    let res = app
        .post("/api/v1/imports/bulk/invoices")
        .json(&json!({
            "filename": "invoices.csv",
            "file_base64": encode_csv(csv),
        }))
        .await;
    res.assert_status_ok();

    let body: Value = res.json();
    assert_eq!(body["total"].as_i64().unwrap(), 2);
    assert_eq!(body["success"].as_i64().unwrap(), 2);
    assert_eq!(body["failed"].as_i64().unwrap(), 0);
    assert_eq!(body["table"].as_str().unwrap(), "invoices");

    let list = app.get("/api/v1/invoices").await;
    list.assert_status_ok();
    let list_body: Value = list.json();
    let imported = list_body["data"].as_array().unwrap();
    assert_eq!(imported.len(), 2);
    assert!(
        imported
            .iter()
            .any(|row| row["invoice_number"] == "BULK-001")
    );
    assert!(
        imported
            .iter()
            .any(|row| row["invoice_number"] == "BULK-002")
    );
}

#[tokio::test]
async fn s69_bulk_import_counts_failed_rows_without_partial_crash() {
    let app = common::TestApp::new().await;

    let csv = "\
invoice_number,amount,description
GOOD-001,88,valid row
,99,missing invoice number
GOOD-002,120,another valid row
";

    let res = app
        .post("/api/v1/imports/bulk/invoices")
        .json(&json!({
            "filename": "invoices.csv",
            "file_base64": encode_csv(csv),
        }))
        .await;
    res.assert_status_ok();

    let body: Value = res.json();
    assert_eq!(body["total"].as_i64().unwrap(), 3);
    assert_eq!(body["success"].as_i64().unwrap(), 2);
    assert_eq!(body["failed"].as_i64().unwrap(), 1);
    assert_eq!(body["errors"].as_array().unwrap().len(), 1);
    assert_eq!(body["errors"][0]["row"].as_i64().unwrap(), 3);

    let list = app.get("/api/v1/invoices").await;
    let list_body: Value = list.json();
    assert_eq!(list_body["count"].as_i64().unwrap(), 2);
}

#[tokio::test]
async fn s69_bulk_import_rejects_unknown_table() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/imports/bulk/sqlite_master")
        .json(&json!({
            "filename": "bad.csv",
            "file_base64": encode_csv("name\nattack\n"),
        }))
        .await;

    res.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn s69_bulk_import_is_company_scoped() {
    let app = common::TestApp::new().await;
    let rival = app.create_second_tenant().await;

    let res = app
        .post("/api/v1/imports/bulk/invoices")
        .json(&json!({
            "filename": "invoices.csv",
            "file_base64": encode_csv("invoice_number,amount\nTENANT-A-1,100\n"),
        }))
        .await;
    res.assert_status_ok();

    let a_list = app.get("/api/v1/invoices").await;
    let b_list = rival.get("/api/v1/invoices").await;
    a_list.assert_status_ok();
    b_list.assert_status_ok();

    assert_eq!(a_list.json::<Value>()["count"].as_i64().unwrap(), 1);
    assert_eq!(b_list.json::<Value>()["count"].as_i64().unwrap(), 0);
}

fn encode_csv(csv: &str) -> String {
    general_purpose::STANDARD.encode(csv.as_bytes())
}
