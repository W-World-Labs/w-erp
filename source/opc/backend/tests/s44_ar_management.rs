/// Scenario 44: 📊 AR Management — Aging Analysis & Summary
/// Tests accounts receivable (AR) endpoints for aging buckets and financial summaries.
use crate::common;
use serde_json::json;

// ==================== AR Aging — Empty (no invoices) ====================

#[tokio::test]
async fn s44_ar_aging_empty() {
    let app = common::TestApp::new().await;
    let res = app.get("/api/v1/ar/aging").await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    assert_eq!(body["total_invoices"], 0);
    assert_eq!(body["total_receivable"], 0.0);
}

// ==================== AR Aging — With receivable invoices ====================

#[tokio::test]
async fn s44_ar_aging_with_invoices() {
    let app = common::TestApp::new().await;

    // Create a receivable invoice with future due_date (should be "current")
    app.post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-AR-001",
            "amount": 5000.0,
            "invoice_type": "receivable",
            "status": "pending",
            "invoice_date": "2026-03-01",
            "due_date": "2099-12-31"
        }))
        .await
        .assert_status_ok();

    // Create an overdue invoice (due_date in the past)
    app.post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-AR-002",
            "amount": 3000.0,
            "invoice_type": "receivable",
            "status": "pending",
            "invoice_date": "2025-01-01",
            "due_date": "2025-01-15"
        }))
        .await
        .assert_status_ok();

    let res = app.get("/api/v1/ar/aging").await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();

    // Should have 2 invoices total
    assert_eq!(body["total_invoices"], 2);
    assert!((body["total_receivable"].as_f64().unwrap() - 8000.0).abs() < 0.01);

    // Future invoice should be in "current" bucket
    assert!(body["buckets"]["current"]["count"].as_u64().unwrap() >= 1);

    // Overdue invoice should be in 180+ bucket (2025-01 is way past)
    assert!(body["total_overdue"].as_f64().unwrap() >= 3000.0);
}

#[tokio::test]
async fn s44_ar_aging_returns_backend_owned_display_contract() {
    let app = common::TestApp::new().await;
    let today = chrono::Utc::now().naive_utc().date();
    let current_due = (today + chrono::Duration::days(7)).format("%Y-%m-%d").to_string();
    let yellow_due = (today - chrono::Duration::days(10)).format("%Y-%m-%d").to_string();
    let orange_due = (today - chrono::Duration::days(45)).format("%Y-%m-%d").to_string();
    let red_due = (today - chrono::Duration::days(100)).format("%Y-%m-%d").to_string();

    for (number, amount, due_date) in [
        ("INV-AR-DISPLAY-CURRENT", 1000.0, current_due),
        ("INV-AR-DISPLAY-YELLOW", 2000.0, yellow_due),
        ("INV-AR-DISPLAY-ORANGE", 3000.0, orange_due),
        ("INV-AR-DISPLAY-RED", 4000.0, red_due),
    ] {
        app.post("/api/invoices")
            .json(&json!({
                "invoice_number": number,
                "amount": amount,
                "invoice_type": "receivable",
                "status": "pending",
                "supplier": "Display Client",
                "due_date": due_date
            }))
            .await
            .assert_status_ok();
    }

    let res = app.get("/api/v1/ar/aging").await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();

    assert_eq!(body["total_receivable_label"], "¥10,000");
    assert_eq!(body["total_overdue_label"], "¥9,000");
    assert_eq!(body["overdue_rate_label"], "75.0%");

    let sections = body["bucket_sections"].as_array().expect("bucket sections");
    assert_eq!(sections.len(), 6);
    assert_eq!(sections[0]["key"], "current");
    assert_eq!(sections[0]["label"], "未到期");
    assert_eq!(sections[0]["risk_tone"], "green");
    assert_eq!(sections[1]["key"], "1_30_days");
    assert_eq!(sections[1]["risk_tone"], "yellow");
    assert_eq!(sections[2]["key"], "31_60_days");
    assert_eq!(sections[2]["risk_tone"], "orange");
    assert_eq!(sections[3]["key"], "61_90_days");
    assert_eq!(sections[3]["risk_tone"], "red");

    let yellow_invoice = sections[1]["invoices"].as_array().unwrap()[0].clone();
    assert_eq!(yellow_invoice["invoice_number"], "INV-AR-DISPLAY-YELLOW");
    assert_eq!(yellow_invoice["amount_label"], "¥2,000");
    assert_eq!(yellow_invoice["days_overdue"], 10);
    assert_eq!(yellow_invoice["days_overdue_label"], "已逾期10天");
    assert_eq!(yellow_invoice["risk_tone"], "yellow");
    assert_eq!(
        yellow_invoice["collection_action"]["method"].as_str(),
        Some("POST")
    );
    assert_eq!(
        yellow_invoice["collection_action"]["endpoint"].as_str(),
        yellow_invoice["id"]
            .as_str()
            .map(|id| format!("/api/v1/ar/invoices/{id}/collection-reminder"))
            .as_deref()
    );
    assert_eq!(
        yellow_invoice["collection_action"]["label"].as_str(),
        Some("生成催收提醒")
    );

    let current_invoice = sections[0]["invoices"].as_array().unwrap()[0].clone();
    assert!(
        current_invoice["collection_action"].is_null(),
        "current invoices must not expose collection reminder actions"
    );
}

// ==================== AR Aging — Excludes paid and non-receivable ====================

#[tokio::test]
async fn s44_ar_aging_excludes_paid() {
    let app = common::TestApp::new().await;

    // Create a payable (not receivable) — should NOT appear in aging
    app.post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-EXP-ONLY",
            "amount": 2000.0,
            "invoice_type": "payable",
            "status": "pending"
        }))
        .await
        .assert_status_ok();

    let res = app.get("/api/v1/ar/aging").await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    // Payable invoices should not appear in receivable aging
    let all_json = serde_json::to_string(&body["buckets"]).unwrap();
    assert!(
        !all_json.contains("INV-EXP-ONLY"),
        "Payable invoice should not appear in AR aging"
    );
}

// ==================== Issue #1: invoice data axis unification ====================

#[tokio::test]
async fn s44_manual_output_alias_lands_in_ar_and_currency_panel() {
    let app = common::TestApp::new().await;

    // Legacy UI alias "output" must normalize to receivable and appear in AR/currency panels.
    // Note: create_invoice ignores body status and stores pending by default.
    let created = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-MANUAL-OUT-001",
            "amount": 4500.0,
            "invoice_type": "output",
            "due_date": "2099-12-31",
            "supplier": "Manual Client Co"
        }))
        .await;
    created.assert_status_ok();
    let inv = created.json::<serde_json::Value>();
    assert_eq!(inv["invoice_type"], "receivable");
    assert_eq!(inv["currency"], "CNY");

    let aging = app.get("/api/v1/ar/aging").await;
    aging.assert_status_ok();
    let aging_body = aging.json::<serde_json::Value>();
    assert!(
        (aging_body["total_receivable"].as_f64().unwrap() - 4500.0).abs() < 0.01,
        "manual output invoice must count toward AR aging"
    );
    let aging_json = serde_json::to_string(&aging_body).unwrap();
    assert!(
        aging_json.contains("INV-MANUAL-OUT-001"),
        "manual output invoice number must appear in AR aging payload"
    );

    let summary = app.get("/api/v1/ar/summary").await;
    summary.assert_status_ok();
    let summary_body = summary.json::<serde_json::Value>();
    assert_eq!(summary_body["outstanding"]["count"], 1);
    assert!(
        (summary_body["outstanding"]["amount"].as_f64().unwrap() - 4500.0).abs() < 0.01
    );

    let balances = app.get("/api/v1/currency/balances").await;
    balances.assert_status_ok();
    let bal = balances.json::<serde_json::Value>();
    let currencies = bal["currencies"].as_array().expect("currencies array");
    let cny = currencies
        .iter()
        .find(|row| row["currency"] == "CNY")
        .expect("CNY row");
    assert!(
        (cny["receivables"].as_f64().unwrap() - 4500.0).abs() < 0.01,
        "currency balances must include receivable total after currency column exists"
    );

    // Payable-only invoice must not inflate AR.
    app.post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-MANUAL-IN-001",
            "amount": 900.0,
            "invoice_type": "input"
        }))
        .await
        .assert_status_ok();
    let aging2 = app.get("/api/v1/ar/aging").await;
    aging2.assert_status_ok();
    let aging2_body = aging2.json::<serde_json::Value>();
    assert!(
        (aging2_body["total_receivable"].as_f64().unwrap() - 4500.0).abs() < 0.01,
        "payable/input invoices must not appear in AR aging totals"
    );
}

#[tokio::test]
async fn s44_client_with_linked_invoice_cannot_be_deleted() {
    let app = common::TestApp::new().await;

    let client = app
        .post("/api/v1/clients")
        .json(&json!({
            "name": "Linked AR Client",
            "email": "ar@example.com"
        }))
        .await;
    client.assert_status_ok();
    let client_body = client.json::<serde_json::Value>();
    let client_id = client_body["id"].as_str().expect("client id").to_string();

    app.post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-LINK-001",
            "amount": 1200.0,
            "invoice_type": "receivable",
            "client_id": client_id
        }))
        .await
        .assert_status_ok();

    let del = app.delete(&format!("/api/v1/clients/{client_id}")).await;
    assert_eq!(
        del.status_code().as_u16(),
        409,
        "deleting a client with invoices must conflict: {}",
        del.text()
    );
}

// ==================== AR Summary — Empty ====================

#[tokio::test]
async fn s44_ar_summary_empty() {
    let app = common::TestApp::new().await;
    let res = app.get("/api/v1/ar/summary").await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    assert_eq!(body["outstanding"]["count"], 0);
    assert_eq!(body["overdue"]["count"], 0);
    assert_eq!(body["paid"]["count"], 0);
    assert_eq!(body["collection_rate"], 0.0);
}

// ==================== AR Summary — With data ====================

#[tokio::test]
async fn s44_ar_summary_with_data() {
    let app = common::TestApp::new().await;

    // Create a receivable invoice
    app.post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-SUM-RECV",
            "amount": 5000.0,
            "invoice_type": "receivable",
            "status": "pending",
            "due_date": "2099-12-31"
        }))
        .await
        .assert_status_ok();

    let res = app.get("/api/v1/ar/summary").await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();

    // Verify structure
    assert!(body["outstanding"]["count"].is_number());
    assert!(body["outstanding"]["amount"].is_number());
    assert!(body["overdue"]["count"].is_number());
    assert!(body["overdue"]["rate"].is_number());
    assert!(body["paid"]["count"].is_number());
    assert!(body["collection_rate"].is_number());

    // At least our 1 invoice should be outstanding
    assert!(body["outstanding"]["count"].as_i64().unwrap() >= 1);
    assert!(body["outstanding"]["amount"].as_f64().unwrap() >= 5000.0);
}

// ==================== AR Summary — Overdue rate calculation ====================

#[tokio::test]
async fn s44_ar_overdue_rate() {
    let app = common::TestApp::new().await;

    // 3 overdue invoices
    for i in 1..=3 {
        app.post("/api/invoices")
            .json(&json!({
                "invoice_number": format!("INV-OR-{}", i),
                "amount": 1000.0,
                "invoice_type": "receivable",
                "status": "pending",
                "due_date": "2025-01-01"
            }))
            .await
            .assert_status_ok();
    }

    // 1 current invoice
    app.post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-OR-CURR",
            "amount": 1000.0,
            "invoice_type": "receivable",
            "status": "pending",
            "due_date": "2099-12-31"
        }))
        .await
        .assert_status_ok();

    let res = app.get("/api/v1/ar/summary").await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();

    // 3/4 = 75% overdue rate
    let rate = body["overdue"]["rate"].as_f64().unwrap();
    assert!(rate > 70.0 && rate < 80.0, "Overdue rate: {}", rate);
}

#[tokio::test]
async fn s44_ar_collection_reminder_creates_backend_owned_draft() {
    let app = common::TestApp::new().await;
    let today = chrono::Utc::now().naive_utc().date();
    let due_date = (today - chrono::Duration::days(12))
        .format("%Y-%m-%d")
        .to_string();

    sqlx::query(
        r#"
        INSERT INTO clients (id, company_id, name, email, client_type)
        VALUES ('client-ar-collection', ?, 'Acme Customer', 'ap@acme.example', 'active')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed client");

    sqlx::query(
        r#"
        INSERT INTO invoices (
            id, company_id, invoice_number, amount, invoice_type, status,
            invoice_date, due_date, supplier, created_by
        )
        VALUES (
            'invoice-ar-collection', ?, 'INV-AR-COLLECT-001', 3200.0,
            'receivable', 'pending', '2026-01-01', ?, 'Acme Customer', ?
        )
        "#,
    )
    .bind(&app.company_id)
    .bind(&due_date)
    .bind(&app.user_id)
    .execute(&app.pool)
    .await
    .expect("seed invoice");

    let res = app
        .post("/api/v1/ar/invoices/invoice-ar-collection/collection-reminder")
        .json(&json!({}))
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();

    assert_eq!(body["success"], true);
    assert_eq!(body["invoice_id"], "invoice-ar-collection");
    assert_eq!(body["invoice_number"], "INV-AR-COLLECT-001");
    assert_eq!(body["recipient_email"], "ap@acme.example");
    assert_eq!(body["reminder_type"], "overdue_7");
    assert!(
        body["subject"]
            .as_str()
            .unwrap()
            .contains("INV-AR-COLLECT-001")
    );
    assert!(body["message"].as_str().unwrap().contains("已逾期12天"));
    assert!(body["reminder_id"].as_str().is_some());
    assert!(body["email_log_id"].as_str().is_some());

    let reminder_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM invoice_reminders WHERE company_id = ? AND invoice_id = ? AND reminder_type = 'overdue_7' AND status = 'pending' AND channel = 'email'",
    )
    .bind(&app.company_id)
    .bind("invoice-ar-collection")
    .fetch_one(&app.pool)
    .await
    .expect("count reminders");
    assert_eq!(reminder_count.0, 1);

    let log: (String, String, String, String) = sqlx::query_as(
        "SELECT to_address, subject, template, status FROM email_logs WHERE company_id = ? AND entity_id = ?",
    )
    .bind(&app.company_id)
    .bind("invoice-ar-collection")
    .fetch_one(&app.pool)
    .await
    .expect("email log");
    assert_eq!(log.0, "ap@acme.example");
    assert!(log.1.contains("INV-AR-COLLECT-001"));
    assert_eq!(log.2, "payment_reminder");
    assert_eq!(log.3, "pending");
}
