/// Scenario 66: 🔁 Workflow Use Cases
/// High-signal black-box integration tests for core workflow state propagation.
use crate::common;

use axum::http::StatusCode;
use serde_json::{Value, json};

fn id_of(body: &Value) -> String {
    body["id"].as_str().expect("missing id").to_string()
}

#[tokio::test]
async fn s66_payment_update_propagates_to_invoice_and_salary() {
    let app = common::TestApp::new().await;

    // Invoice flow: payment starts partial, then update should mark invoice paid.
    let invoice = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-S66-001",
            "amount": 1000.0,
            "invoice_type": "receivable",
            "status": "pending",
            "supplier": "S66 Invoice Customer"
        }))
        .await;
    invoice.assert_status_ok();
    let invoice_id = id_of(&invoice.json::<Value>());

    let payment = app
        .post("/api/v1/payments")
        .json(&json!({
            "invoice_id": &invoice_id,
            "amount": 400.0,
            "payment_type": "receipt",
            "payment_date": "2026-04-10",
            "payment_method": "bank_transfer",
            "status": "pending"
        }))
        .await;
    payment.assert_status_ok();
    let payment_id = id_of(&payment.json::<Value>());

    let invoice_after_create = app
        .get(&format!("/api/invoices/{}", invoice_id))
        .await
        .json::<Value>();
    // A pending receipt does not settle any of the invoice yet.
    assert_eq!(invoice_after_create["status"], "pending");

    let payment_update = app
        .put(&format!("/api/v1/payments/{}", payment_id))
        .json(&json!({
            "invoice_id": &invoice_id,
            "amount": 1000.0,
            "payment_date": "2026-04-16",
            "status": "completed"
        }))
        .await;
    payment_update.assert_status_ok();

    let invoice_after_update = app
        .get(&format!("/api/invoices/{}", invoice_id))
        .await
        .json::<Value>();
    assert_eq!(invoice_after_update["status"], "paid");
    assert_eq!(invoice_after_update["amount"], 1000.0);

    let payment_after_update = app
        .get(&format!("/api/v1/payments/{}", payment_id))
        .await
        .json::<Value>();
    assert_eq!(payment_after_update["amount"], 1000.0);
    assert_eq!(payment_after_update["payment_date"], "2026-04-16");

    // Salary flow: payment update should also propagate to salary record state.
    let employee = app
        .post("/api/v1/employees")
        .json(&json!({
            "name": "S66 Payroll Employee",
            "position": "Engineer",
            "base_salary": 12000.0
        }))
        .await;
    employee.assert_status_ok();
    let employee_id = id_of(&employee.json::<Value>());

    let salary = app
        .post("/api/v1/salary")
        .json(&json!({
            "employee_id": &employee_id,
            "period": "2026-04",
            "base_salary": 9000.0,
            "bonus": 1000.0,
            "deductions": 0.0,
            "net_salary": 10000.0,
            "status": "unpaid"
        }))
        .await;
    salary.assert_status_ok();
    let salary_id = id_of(&salary.json::<Value>());

    let salary_payment = app
        .post("/api/v1/payments")
        .json(&json!({
            "salary_record_id": &salary_id,
            "amount": 5000.0,
            "payment_type": "salary",
            "payment_date": "2026-04-12",
            "payment_method": "bank_transfer",
            "status": "pending"
        }))
        .await;
    salary_payment.assert_status_ok();
    let salary_payment_id = id_of(&salary_payment.json::<Value>());

    let salary_before_update = app
        .get(&format!("/api/v1/salary/{}", salary_id))
        .await
        .json::<Value>();
    assert_eq!(salary_before_update["status"], "unpaid");

    let salary_payment_update = app
        .put(&format!("/api/v1/payments/{}", salary_payment_id))
        .json(&json!({
            "salary_record_id": &salary_id,
            "amount": 10000.0,
            "payment_date": "2026-04-16",
            "status": "completed"
        }))
        .await;
    salary_payment_update.assert_status_ok();

    let salary_after_update = app
        .get(&format!("/api/v1/salary/{}", salary_id))
        .await
        .json::<Value>();
    assert_eq!(salary_after_update["status"], "paid");
    assert_eq!(salary_after_update["payment_id"], salary_payment_id);
    assert_eq!(salary_after_update["payment_date"], "2026-04-16");
}

#[tokio::test]
async fn s66_time_entries_convert_to_invoice_with_coherent_statuses() {
    let app = common::TestApp::new().await;

    app.post("/api/v1/time_entries")
        .json(&json!({
            "description": "Discovery workshop",
            "date": "2026-04-02",
            "duration_minutes": 90,
            "hourly_rate": 200.0,
            "amount": 300.0,
            "is_billable": 1,
            "status": "logged"
        }))
        .await
        .assert_status_ok();

    app.post("/api/v1/time_entries")
        .json(&json!({
            "description": "Implementation sprint",
            "date": "2026-04-03",
            "duration_minutes": 150,
            "hourly_rate": 200.0,
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

    let body = conv.json::<Value>();
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
    assert_eq!(body["entries_count"], 2);
    assert_eq!(body["amount"], 800.0);
    assert_eq!(body["total_hours"], 4.0);

    let invoice_id = body["invoice_id"].as_str().unwrap().to_string();
    let invoice = app
        .get(&format!("/api/invoices/{}", invoice_id))
        .await
        .json::<Value>();
    assert_eq!(invoice["invoice_type"], "receivable");
    assert_eq!(invoice["status"], "pending");
    assert_eq!(invoice["amount"], 800.0);
    assert!(
        invoice["description"]
            .as_str()
            .unwrap()
            .starts_with("Time entries: 2 entries, 4.0h")
    );
    assert!(
        invoice["description"]
            .as_str()
            .unwrap()
            .contains("Discovery workshop")
    );

    let converted_time_entries = app.get("/api/v1/time_entries").await.json::<Value>();
    let rows = converted_time_entries["data"]
        .as_array()
        .expect("time entries data");
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|row| row["status"] == "invoiced"));
    assert!(rows.iter().all(|row| row["invoice_id"] == invoice_id));
}

#[tokio::test]
async fn s66_quote_conversion_blocks_duplicate_invoice_creation() {
    let app = common::TestApp::new().await;

    let client = app
        .post("/api/v1/clients")
        .json(&json!({
            "name": "S66 Quote Customer",
            "client_type": "corporate"
        }))
        .await;
    client.assert_status_ok();
    let client_id = id_of(&client.json::<Value>());

    let quote = app
        .post("/api/v1/quotes")
        .json(&json!({
            "client_id": &client_id,
            "quote_number": "QT-S66-001",
            "title": "S66 Conversion Test",
            "status": "accepted",
            "subtotal": 2000.0,
            "tax_amount": 200.0,
            "total": 2200.0
        }))
        .await;
    quote.assert_status_ok();
    let quote_id = id_of(&quote.json::<Value>());

    let first = app
        .post(&format!("/api/v1/quotes/{}/convert-to-invoice", quote_id))
        .json(&json!({}))
        .await;
    first.assert_status_ok();
    let first_body = first.json::<Value>();
    let invoice_id = first_body["invoice_id"].as_str().unwrap().to_string();

    let quote_after_first = app
        .get(&format!("/api/v1/quotes/{}", quote_id))
        .await
        .json::<Value>();
    assert_eq!(quote_after_first["status"], "invoiced");
    assert_eq!(quote_after_first["invoice_id"], invoice_id);

    let invoice = app
        .get(&format!("/api/invoices/{}", invoice_id))
        .await
        .json::<Value>();
    assert_eq!(invoice["invoice_type"], "receivable");
    assert_eq!(invoice["status"], "pending");
    assert_eq!(invoice["amount"], 2200.0);
    assert!(
        invoice["description"]
            .as_str()
            .unwrap()
            .contains("QT-S66-001")
    );

    let second = app
        .post(&format!("/api/v1/quotes/{}/convert-to-invoice", quote_id))
        .json(&json!({}))
        .await;
    assert_eq!(second.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn s66_payment_relink_recalculates_both_invoices_and_null_unlinks() {
    let app = common::TestApp::new().await;
    let mut ids = Vec::new();
    for number in ["RELINK-A", "RELINK-B"] {
        let response = app.post("/api/invoices").json(&json!({"invoice_number":number,"amount":100.0,"invoice_type":"receivable","status":"pending"})).await;
        response.assert_status_ok();
        ids.push(id_of(&response.json::<Value>()));
    }
    let payment = app.post("/api/v1/payments").json(&json!({"invoice_id":ids[0],"amount":100.0,"payment_type":"receipt","status":"completed"})).await;
    payment.assert_status_ok();
    let payment_id = id_of(&payment.json::<Value>());
    app.put(&format!("/api/v1/payments/{payment_id}")).json(&json!({"invoice_id":ids[1]})).await.assert_status_ok();
    let old = app.get(&format!("/api/invoices/{}", ids[0])).await.json::<Value>();
    let new = app.get(&format!("/api/invoices/{}", ids[1])).await.json::<Value>();
    assert_eq!(old["status"], "pending");
    assert_eq!(new["status"], "paid");
    app.put(&format!("/api/v1/payments/{payment_id}")).json(&json!({"invoice_id":null})).await.assert_status_ok();
    let cleared: Option<String> = sqlx::query_scalar("SELECT invoice_id FROM payments WHERE id=?").bind(&payment_id).fetch_one(&app.pool).await.unwrap();
    assert!(cleared.is_none());
    let new = app.get(&format!("/api/invoices/{}", ids[1])).await.json::<Value>();
    assert_eq!(new["status"], "pending");
}

#[tokio::test]
async fn s66_salary_payment_deletion_recomputes_status_and_live_reference() {
    let app = common::TestApp::new().await;
    let employee = app.post("/api/v1/employees").json(&json!({"name":"Payroll regression"})).await;
    employee.assert_status_ok();
    let employee_id = id_of(&employee.json::<Value>());
    let salary = app.post("/api/v1/salary").json(&json!({"employee_id":employee_id,"period":"2026-06","net_salary":100.0,"status":"pending"})).await;
    salary.assert_status_ok();
    let salary_id = id_of(&salary.json::<Value>());
    let mut ids = Vec::new();
    for day in ["2026-06-01", "2026-06-02"] {
        let payment = app.post("/api/v1/payments").json(&json!({"salary_record_id":salary_id,"amount":100.0,"payment_type":"salary","status":"completed","payment_date":day})).await;
        payment.assert_status_ok();
        ids.push(id_of(&payment.json::<Value>()));
    }
    app.delete(&format!("/api/v1/payments/{}", ids[1])).await.assert_status(StatusCode::NO_CONTENT);
    let paid = app.get(&format!("/api/v1/salary/{salary_id}")).await.json::<Value>();
    assert_eq!(paid["status"], "paid");
    assert_eq!(paid["payment_id"], ids[0]);
    assert_eq!(paid["payment_date"], "2026-06-01");
    app.delete(&format!("/api/v1/payments/{}", ids[0])).await.assert_status(StatusCode::NO_CONTENT);
    let unpaid = app.get(&format!("/api/v1/salary/{salary_id}")).await.json::<Value>();
    assert_eq!(unpaid["status"], "pending");
    assert!(unpaid["payment_id"].as_str().unwrap_or_default().is_empty());
    assert!(unpaid["payment_date"].as_str().unwrap_or_default().is_empty());
    let links: (Option<String>, Option<String>) = sqlx::query_as("SELECT payment_id,payment_date FROM salary_records WHERE id=?")
        .bind(&salary_id).fetch_one(&app.pool).await.unwrap();
    assert_eq!(links, (None, None));
}

#[tokio::test]
async fn s66_only_completed_receipts_settle_invoice_and_cancellation_reverses() {
    let app = common::TestApp::new().await;
    let invoice = app.post("/api/invoices").json(&json!({"invoice_number":"SETTLED-ONLY","amount":100.0,"invoice_type":"receivable","status":"pending"})).await;
    invoice.assert_status_ok();
    let invoice_id = id_of(&invoice.json::<Value>());
    let payment = app.post("/api/v1/payments").json(&json!({"invoice_id":invoice_id,"amount":100.0,"payment_type":"income","status":"pending"})).await;
    payment.assert_status_ok();
    let payment_id = id_of(&payment.json::<Value>());
    for (status, expected) in [("pending","pending"),("completed","paid"),("cancelled","pending"),("paid","paid")] {
        app.put(&format!("/api/v1/payments/{payment_id}")).json(&json!({"status":status})).await.assert_status_ok();
        let row = app.get(&format!("/api/invoices/{invoice_id}")).await.json::<Value>();
        assert_eq!(row["status"], expected, "payment status {status}");
    }
    app.put(&format!("/api/v1/payments/{payment_id}")).json(&json!({"status":"pending"})).await.assert_status_ok();
    app.post("/api/v1/agent/execute").json(&json!({"tool_name":"payments.approve","agent_name":"settlement-test","arguments":{"id":payment_id}})).await.assert_status_ok();
    let approved = app.get(&format!("/api/invoices/{invoice_id}")).await.json::<Value>();
    assert_eq!(approved["status"], "paid");

}
