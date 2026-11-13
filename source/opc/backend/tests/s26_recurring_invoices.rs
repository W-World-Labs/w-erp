/// Scenario 26: 🔄 Recurring Invoices & Payment Reminders
/// Tests recurring invoice templates, invoice reminders, FK validation, lifecycle management
use crate::common;
use serde_json::json;

// ==================== Recurring Invoices ====================

#[tokio::test]
async fn s26_recurring_invoice_crud() {
    let app = common::TestApp::new().await;

    // Create
    let res = app
        .post("/api/v1/recurring_invoices")
        .json(&json!({
            "title": "月度SaaS订阅费",
            "amount": 5000,
            "frequency": "monthly",
            "start_date": "2024-04-01",
            "next_invoice_date": "2024-04-01",
            "payment_terms_days": 15,
            "day_of_month": 1,
            "auto_send": 1,
            "status": "active"
        }))
        .await;
    res.assert_status_ok();
    let ri_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Get
    let get_res = app
        .get(&format!("/api/v1/recurring_invoices/{}", ri_id))
        .await;
    let ri = get_res.json::<serde_json::Value>();
    assert_eq!(ri["title"].as_str().unwrap(), "月度SaaS订阅费");
    assert_eq!(ri["frequency"].as_str().unwrap(), "monthly");
    assert_eq!(ri["status"].as_str().unwrap(), "active");

    // Update — pause
    app.put(&format!("/api/v1/recurring_invoices/{}", ri_id))
        .json(&json!({ "status": "paused" }))
        .await
        .assert_status_ok();

    let paused = app
        .get(&format!("/api/v1/recurring_invoices/{}", ri_id))
        .await;
    assert_eq!(
        paused.json::<serde_json::Value>()["status"]
            .as_str()
            .unwrap(),
        "paused"
    );

    // List
    let list = app.get("/api/v1/recurring_invoices").await;
    list.assert_status_ok();
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 1);

    // Delete
    let del_res = app
        .delete(&format!("/api/v1/recurring_invoices/{}", ri_id))
        .await;
    del_res.assert_status(axum::http::StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn s26_recurring_invoice_with_client_fk() {
    let app = common::TestApp::new().await;

    // Create client
    let client_res = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "测试公司A" }))
        .await;
    client_res.assert_status_ok();
    let client_id = client_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create recurring invoice linked to client
    let res = app
        .post("/api/v1/recurring_invoices")
        .json(&json!({
            "title": "季度咨询服务费",
            "amount": 30000,
            "frequency": "quarterly",
            "start_date": "2024-01-01",
            "next_invoice_date": "2024-04-01",
            "client_id": client_id
        }))
        .await;
    res.assert_status_ok();
    let ri_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Verify FK
    let get_res = app
        .get(&format!("/api/v1/recurring_invoices/{}", ri_id))
        .await;
    assert_eq!(
        get_res.json::<serde_json::Value>()["client_id"]
            .as_str()
            .unwrap(),
        client_id
    );
}

#[tokio::test]
async fn s26_recurring_invoice_frequencies() {
    let app = common::TestApp::new().await;

    // Test all frequency types
    for freq in &["weekly", "monthly", "quarterly", "yearly"] {
        let res = app
            .post("/api/v1/recurring_invoices")
            .json(&json!({
                "title": format!("{}计费", freq),
                "amount": 1000,
                "frequency": freq,
                "start_date": "2024-01-01",
                "next_invoice_date": "2024-01-01"
            }))
            .await;
        res.assert_status_ok();
    }

    let list = app.get("/api/v1/recurring_invoices").await;
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 4);
}

// ==================== Invoice Reminders ====================

#[tokio::test]
async fn s26_invoice_reminder_crud() {
    let app = common::TestApp::new().await;

    // Create invoice first (using generic invoices endpoint)
    let inv_res = app
        .post("/api/v1/invoices")
        .json(&json!({
            "invoice_number": "INV-2024-TEST-001",
            "amount": 10000,
            "status": "sent",
            "due_date": "2024-04-15"
        }))
        .await;
    inv_res.assert_status_ok();
    let invoice_id = inv_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create reminder
    let res = app
        .post("/api/v1/invoice_reminders")
        .json(&json!({
            "invoice_id": invoice_id,
            "reminder_type": "before_due",
            "scheduled_date": "2024-04-12",
            "subject": "付款提醒：即将到期",
            "message": "尊敬的客户，您的发票将在3天后到期。",
            "channel": "email"
        }))
        .await;
    res.assert_status_ok();
    let rem_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Get
    let get_res = app
        .get(&format!("/api/v1/invoice_reminders/{}", rem_id))
        .await;
    let rem = get_res.json::<serde_json::Value>();
    assert_eq!(rem["reminder_type"].as_str().unwrap(), "before_due");
    assert_eq!(rem["channel"].as_str().unwrap(), "email");

    // Update — mark as sent
    app.put(&format!("/api/v1/invoice_reminders/{}", rem_id))
        .json(&json!({ "status": "sent", "sent_at": "2024-04-12T10:00:00Z" }))
        .await
        .assert_status_ok();

    let sent = app
        .get(&format!("/api/v1/invoice_reminders/{}", rem_id))
        .await;
    assert_eq!(
        sent.json::<serde_json::Value>()["status"].as_str().unwrap(),
        "sent"
    );

    // List
    let list = app.get("/api/v1/invoice_reminders").await;
    list.assert_status_ok();
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn s26_multi_level_reminders() {
    let app = common::TestApp::new().await;

    // Create invoice
    let inv_res = app
        .post("/api/v1/invoices")
        .json(&json!({
            "invoice_number": "INV-2024-TEST-002",
            "amount": 20000,
            "status": "overdue",
            "due_date": "2024-03-01"
        }))
        .await;
    inv_res.assert_status_ok();
    let invoice_id = inv_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create escalating reminders
    let types = vec![
        "before_due",
        "on_due",
        "overdue_1",
        "overdue_7",
        "overdue_30",
    ];
    let dates = vec![
        "2024-02-28",
        "2024-03-01",
        "2024-03-02",
        "2024-03-08",
        "2024-03-31",
    ];

    for (rt, date) in types.iter().zip(dates.iter()) {
        app.post("/api/v1/invoice_reminders")
            .json(&json!({
                "invoice_id": invoice_id,
                "reminder_type": rt,
                "scheduled_date": date,
                "subject": format!("提醒: {}", rt)
            }))
            .await
            .assert_status_ok();
    }

    let list = app.get("/api/v1/invoice_reminders").await;
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 5);
}

#[tokio::test]
async fn s26_tenant_isolation() {
    let app = common::TestApp::new().await;

    // Create recurring invoice
    app.post("/api/v1/recurring_invoices")
        .json(&json!({
            "title": "租户隔离测试",
            "amount": 100,
            "frequency": "monthly",
            "start_date": "2024-01-01",
            "next_invoice_date": "2024-02-01"
        }))
        .await
        .assert_status_ok();

    // Verify company_id is set correctly
    let list = app.get("/api/v1/recurring_invoices").await;
    let data = list.json::<serde_json::Value>();
    if let Some(items) = data["data"].as_array() {
        for item in items {
            assert!(item["company_id"].as_str().is_some());
        }
    }
    assert!(data["count"].as_i64().unwrap() >= 1);
}
