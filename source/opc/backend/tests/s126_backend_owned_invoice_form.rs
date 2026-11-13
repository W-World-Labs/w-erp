use crate::common;

use serde_json::{Value, json};

fn assert_f64(actual: &Value, expected: f64) {
    let actual = actual.as_f64().expect("number");
    assert!(
        (actual - expected).abs() < 0.001,
        "expected {expected}, got {actual}"
    );
}

#[tokio::test]
async fn s126_invoice_create_accepts_form_strings_and_normalizes_on_backend() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-BACKEND-FORM-001",
            "invoice_type": "output",
            "supplier": "Backend Supplier",
            "amount": "¥3,210.50",
            "tax_amount": "¥321.05",
            "invoice_date": "2026-05-09T00:00:00.000Z",
            "due_date": "2026-06-08T00:00:00.000Z",
            "description": "backend-owned invoice form"
        }))
        .await;
    res.assert_status_ok();
    let invoice = res.json::<Value>();

    assert_f64(&invoice["amount"], 3210.50);
    assert_f64(&invoice["tax_amount"], 321.05);
    assert_eq!(invoice["invoice_date"], "2026-05-09");
    assert_eq!(invoice["due_date"], "2026-06-08");
    assert_eq!(invoice["invoice_type"], "receivable");
}

#[tokio::test]
async fn s126_invoice_form_options_includes_client_dropdown() {
    let app = common::TestApp::new().await;

    sqlx::query(
        "INSERT INTO clients (id, company_id, name) VALUES (?, ?, ?)",
    )
    .bind("client-form-opt-1")
    .bind(&app.company_id)
    .bind("Form Options Client")
    .execute(&app.pool)
    .await
    .expect("insert client");

    let res = app.get("/api/invoices/form_options").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["default_invoice_type"], "receivable");
    let types = body["invoice_type_options"]
        .as_array()
        .expect("invoice_type_options");
    let values: Vec<&str> = types
        .iter()
        .filter_map(|item| item["value"].as_str())
        .collect();
    assert!(values.contains(&"receivable"));
    assert!(values.contains(&"payable"));

    let clients = body["client_options"]
        .as_array()
        .expect("client_options");
    let found = clients.iter().any(|c| {
        c["value"] == "client-form-opt-1" && c["label"] == "Form Options Client"
    });
    assert!(found, "client_options should include seeded client");
}

#[tokio::test]
async fn s126_invoice_create_with_client_id_fills_supplier_and_overview() {
    let app = common::TestApp::new().await;

    sqlx::query(
        "INSERT INTO clients (id, company_id, name) VALUES (?, ?, ?)",
    )
    .bind("client-inv-link-1")
    .bind(&app.company_id)
    .bind("Linked Receivable Client")
    .execute(&app.pool)
    .await
    .expect("insert client");

    let res = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-CLIENT-LINK-001",
            "invoice_type": "receivable",
            "client_id": "client-inv-link-1",
            "amount": "1500",
            "invoice_date": "2026-05-10"
        }))
        .await;
    res.assert_status_ok();
    let invoice = res.json::<Value>();
    assert_eq!(invoice["client_id"], "client-inv-link-1");
    assert_eq!(invoice["supplier"], "Linked Receivable Client");
    assert_eq!(invoice["invoice_type"], "receivable");
    assert_eq!(invoice["currency"], "CNY");

    let overview = app.get("/api/invoices/overview").await;
    overview.assert_status_ok();
    let body = overview.json::<Value>();
    let row = body["output_invoices"]
        .as_array()
        .expect("output_invoices")
        .iter()
        .find(|item| item["invoice_number"] == "INV-CLIENT-LINK-001")
        .expect("created receivable in output list");
    assert_eq!(row["client_id"], "client-inv-link-1");
    assert_eq!(row["currency"], "CNY");
}

#[tokio::test]
async fn s126_invoice_create_links_backend_file_id_and_overview_returns_file_display_metadata() {
    let app = common::TestApp::new().await;

    sqlx::query(
        "INSERT INTO files (id, company_id, filename, original_name, content_type, size, uploaded_by) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("file-invoice-001")
    .bind(&app.company_id)
    .bind("stored-invoice-hash")
    .bind("backend-invoice.pdf")
    .bind("application/pdf")
    .bind(2048_i64)
    .bind(&app.user_id)
    .execute(&app.pool)
    .await
    .expect("insert file");

    let res = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-BACKEND-FILE-001",
            "invoice_type": "output",
            "supplier": "Backend File Supplier",
            "amount": "1000",
            "invoice_date": "2026-05-09",
            "file_path": "file-invoice-001"
        }))
        .await;
    res.assert_status_ok();
    let invoice = res.json::<Value>();
    assert_eq!(invoice["file_path"], "file-invoice-001");

    let linked: (Option<String>, Option<String>) =
        sqlx::query_as("SELECT entity_type, entity_id FROM files WHERE id = ? AND company_id = ?")
            .bind("file-invoice-001")
            .bind(&app.company_id)
            .fetch_one(&app.pool)
            .await
            .expect("linked file");
    assert_eq!(linked.0.as_deref(), Some("invoices"));
    assert_eq!(linked.1.as_deref(), invoice["id"].as_str());

    let overview = app.get("/api/invoices/overview").await;
    overview.assert_status_ok();
    let body = overview.json::<Value>();
    let row = body["invoices"]
        .as_array()
        .expect("invoices")
        .iter()
        .find(|item| item["invoice_number"] == "INV-BACKEND-FILE-001")
        .expect("created invoice row");

    assert_eq!(row["file_path"], "file-invoice-001");
    assert_eq!(row["file_name"], "backend-invoice.pdf");
    assert_eq!(row["file_label"], "backend-invoice.pdf");
    assert_eq!(row["has_file"], true);
}
