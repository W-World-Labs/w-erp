/// Scenario 43: ⏱️→📄 Time Entry → Invoice Conversion
/// Tests the workflow of converting billable time entries into invoices.
use crate::common;
use serde_json::json;

// ==================== Happy Path: Convert time entries to invoice ====================

#[tokio::test]
async fn s43_time_to_invoice_basic() {
    let app = common::TestApp::new().await;

    // 1. Create a billable time entry
    let te = app
        .post("/api/v1/time_entries")
        .json(&json!({
            "description": "Frontend development",
            "date": "2026-03-01",
            "duration_minutes": 120,
            "hourly_rate": 150.0,
            "amount": 300.0,
            "is_billable": 1,
            "status": "logged"
        }))
        .await;
    te.assert_status_ok();

    // 2. Convert to invoice
    let conv = app
        .post("/api/v1/time_entries/invoice")
        .json(&json!({}))
        .await;
    conv.assert_status_ok();

    let body = conv.json::<serde_json::Value>();
    assert_eq!(body["success"], true);
    assert!(body["invoice_id"].is_string());
    assert!(
        body["invoice_number"]
            .as_str()
            .unwrap()
            .starts_with("INV-"),
        "expected sequential INV-YYYY-NNNN, got {:?}",
        body["invoice_number"]
    );
    assert_eq!(body["entries_count"], 1);
    assert!(body["amount"].as_f64().unwrap() >= 300.0);
}

// ==================== Multiple entries summed ====================

#[tokio::test]
async fn s43_time_to_invoice_multiple_entries() {
    let app = common::TestApp::new().await;

    // Create 3 time entries
    for i in 1..=3 {
        app.post("/api/v1/time_entries")
            .json(&json!({
                "description": format!("Task {}", i),
                "date": format!("2026-03-0{}", i),
                "duration_minutes": 60,
                "hourly_rate": 100.0,
                "amount": 100.0,
                "is_billable": 1,
                "status": "logged"
            }))
            .await
            .assert_status_ok();
    }

    let conv = app
        .post("/api/v1/time_entries/invoice")
        .json(&json!({}))
        .await;
    conv.assert_status_ok();

    let body = conv.json::<serde_json::Value>();
    assert_eq!(body["entries_count"], 3);
    assert!((body["amount"].as_f64().unwrap() - 300.0).abs() < 0.01);
    assert!((body["total_hours"].as_f64().unwrap() - 3.0).abs() < 0.01);
}

// ==================== No unbilled entries → 400 ====================

#[tokio::test]
async fn s43_time_to_invoice_no_entries() {
    let app = common::TestApp::new().await;

    // No time entries exist
    let conv = app
        .post("/api/v1/time_entries/invoice")
        .json(&json!({}))
        .await;
    assert_eq!(conv.status_code(), 400);
}

// ==================== Non-billable entries excluded ====================

#[tokio::test]
async fn s43_time_to_invoice_excludes_non_billable() {
    let app = common::TestApp::new().await;

    // Create a non-billable entry
    app.post("/api/v1/time_entries")
        .json(&json!({
            "description": "Internal meeting",
            "date": "2026-03-01",
            "duration_minutes": 30,
            "is_billable": 0,
            "status": "logged"
        }))
        .await
        .assert_status_ok();

    // No billable entries → should fail
    let conv = app
        .post("/api/v1/time_entries/invoice")
        .json(&json!({}))
        .await;
    assert_eq!(conv.status_code(), 400);
}

// ==================== Already invoiced entries excluded ====================

#[tokio::test]
async fn s43_time_to_invoice_idempotent() {
    let app = common::TestApp::new().await;

    // Create and convert
    app.post("/api/v1/time_entries")
        .json(&json!({
            "description": "Billable work",
            "date": "2026-03-01",
            "duration_minutes": 60,
            "hourly_rate": 200.0,
            "amount": 200.0,
            "is_billable": 1,
            "status": "logged"
        }))
        .await
        .assert_status_ok();

    let first = app
        .post("/api/v1/time_entries/invoice")
        .json(&json!({}))
        .await;
    first.assert_status_ok();

    // Second conversion should fail (entries now invoiced)
    let second = app
        .post("/api/v1/time_entries/invoice")
        .json(&json!({}))
        .await;
    assert_eq!(second.status_code(), 400);
}

// ==================== Filter by project_id ====================

#[tokio::test]
async fn s43_time_to_invoice_filter_project() {
    let app = common::TestApp::new().await;

    // Create a project
    let proj = app
        .post("/api/projects")
        .json(&json!({ "name": "Project Alpha", "status": "active" }))
        .await;
    proj.assert_status_ok();
    let proj_id = proj.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create time entries for this project
    app.post("/api/v1/time_entries")
        .json(&json!({
            "description": "Alpha task",
            "date": "2026-03-01",
            "duration_minutes": 90,
            "hourly_rate": 100.0,
            "amount": 150.0,
            "is_billable": 1,
            "project_id": &proj_id,
            "status": "logged"
        }))
        .await
        .assert_status_ok();

    // Create time entries for NO project
    app.post("/api/v1/time_entries")
        .json(&json!({
            "description": "General task",
            "date": "2026-03-02",
            "duration_minutes": 60,
            "hourly_rate": 100.0,
            "amount": 100.0,
            "is_billable": 1,
            "status": "logged"
        }))
        .await
        .assert_status_ok();

    // Convert only project Alpha entries
    let conv = app
        .post("/api/v1/time_entries/invoice")
        .json(&json!({ "project_id": &proj_id }))
        .await;
    conv.assert_status_ok();
    let body = conv.json::<serde_json::Value>();
    assert_eq!(body["entries_count"], 1);
    assert!((body["amount"].as_f64().unwrap() - 150.0).abs() < 0.01);
}

// ==================== Filter by client_id ====================

#[tokio::test]
async fn s43_time_to_invoice_filter_client() {
    let app = common::TestApp::new().await;

    // Create a client
    let cl = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "客户甲", "client_type": "corporate" }))
        .await;
    cl.assert_status_ok();
    let cl_id = cl.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Time entry for this client
    app.post("/api/v1/time_entries")
        .json(&json!({
            "description": "Client work",
            "date": "2026-03-01",
            "duration_minutes": 120,
            "hourly_rate": 200.0,
            "amount": 400.0,
            "is_billable": 1,
            "client_id": &cl_id,
            "status": "logged"
        }))
        .await
        .assert_status_ok();

    let conv = app
        .post("/api/v1/time_entries/invoice")
        .json(&json!({ "client_id": &cl_id }))
        .await;
    conv.assert_status_ok();
    let body = conv.json::<serde_json::Value>();
    assert_eq!(body["entries_count"], 1);
    assert!((body["amount"].as_f64().unwrap() - 400.0).abs() < 0.01);
}

// ==================== Invoice appears in invoice list ====================

#[tokio::test]
async fn s43_invoice_appears_in_list() {
    let app = common::TestApp::new().await;

    app.post("/api/v1/time_entries")
        .json(&json!({
            "description": "Visible invoice",
            "date": "2026-03-01",
            "duration_minutes": 60,
            "hourly_rate": 500.0,
            "amount": 500.0,
            "is_billable": 1,
            "status": "logged"
        }))
        .await
        .assert_status_ok();

    let conv = app
        .post("/api/v1/time_entries/invoice")
        .json(&json!({}))
        .await;
    conv.assert_status_ok();
    let invoice_id = conv.json::<serde_json::Value>()["invoice_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Verify invoice exists
    let inv = app.get(&format!("/api/invoices/{}", invoice_id)).await;
    inv.assert_status_ok();
    let inv_body = inv.json::<serde_json::Value>();
    assert_eq!(inv_body["invoice_type"], "receivable");
    assert!(
        inv_body["description"]
            .as_str()
            .unwrap()
            .contains("Time entries")
    );
}
