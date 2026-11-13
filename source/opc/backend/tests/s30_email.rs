/// Scenario 30: 📧 Email System (邮件发送系统)
/// Tests email sending, config dependency, error handling, email logs
use crate::common;
use serde_json::json;

#[tokio::test]
async fn s30_email_send_without_config() {
    let app = common::TestApp::new().await;

    // Try to send email without any email config → should fail gracefully
    let res = app
        .post("/api/v1/email/send")
        .json(&json!({
            "to": "test@example.com",
            "subject": "Test Email",
            "body": "<h1>Hello</h1>",
            "template": "custom"
        }))
        .await;
    // Should return 502 (BAD_GATEWAY) — no SMTP config
    assert_eq!(res.status_code(), 502);
}

#[tokio::test]
async fn s30_email_send_missing_fields() {
    let app = common::TestApp::new().await;

    // Missing 'to'
    let res1 = app
        .post("/api/v1/email/send")
        .json(&json!({ "subject": "Test" }))
        .await;
    assert_eq!(res1.status_code(), 400);

    // Missing 'subject'
    let res2 = app
        .post("/api/v1/email/send")
        .json(&json!({ "to": "test@example.com" }))
        .await;
    assert_eq!(res2.status_code(), 400);
}

#[tokio::test]
async fn s30_email_with_config_no_credentials() {
    let app = common::TestApp::new().await;

    // Create email config with host but no credentials
    // Schema: smtp_host, smtp_port, smtp_user, smtp_password, sender_name, sender_email, enabled
    app.post("/api/v1/email_configurations")
        .json(&json!({
            "smtp_host": "smtp.example.com",
            "smtp_port": 587,
            "sender_email": "noreply@example.com",
            "enabled": 1
        }))
        .await
        .assert_status_ok();

    // Try send → should fail (no credentials)
    let res = app
        .post("/api/v1/email/send")
        .json(&json!({
            "to": "test@example.com",
            "subject": "Test With Config",
            "body": "Hello"
        }))
        .await;
    assert_eq!(res.status_code(), 502);
}

#[tokio::test]
async fn s30_email_log_recorded() {
    let app = common::TestApp::new().await;

    // Attempt to send (will fail but should still log)
    let _ = app
        .post("/api/v1/email/send")
        .json(&json!({
            "to": "log-test@example.com",
            "subject": "Log Test Email",
            "template": "invoice_send",
            "entity_type": "invoices",
            "entity_id": "test-inv-123"
        }))
        .await;

    // Check email logs
    let logs = app.get("/api/v1/email/logs").await;
    logs.assert_status_ok();
    let log_data = logs.json::<serde_json::Value>();

    if let Some(items) = log_data["data"].as_array() {
        let matching: Vec<_> = items
            .iter()
            .filter(|l| l["to_address"].as_str() == Some("log-test@example.com"))
            .collect();
        assert!(
            !matching.is_empty(),
            "Email attempt should be logged even on failure"
        );

        let log = &matching[0];
        assert_eq!(log["subject"].as_str().unwrap(), "Log Test Email");
        assert_eq!(log["template"].as_str().unwrap(), "invoice_send");
        assert_eq!(log["entity_type"].as_str().unwrap(), "invoices");
        assert_eq!(log["status"].as_str().unwrap(), "failed");
    }
}

#[tokio::test]
async fn s30_email_with_full_config() {
    let app = common::TestApp::new().await;

    // Create complete email config using actual schema column names
    app.post("/api/v1/email_configurations")
        .json(&json!({
            "smtp_host": "smtp.gmail.com",
            "smtp_port": 587,
            "smtp_user": "user@gmail-com.example.invalid",
            "smtp_password": "app-password",
            "sender_email": "user@gmail-com.example.invalid",
            "sender_name": "OPC",
            "enabled": 1
        }))
        .await
        .assert_status_ok();

    // Send email → should succeed (simulated in dev mode)
    let res = app
        .post("/api/v1/email/send")
        .json(&json!({
            "to": "client@example.com",
            "subject": "发票通知: INV-2024-001",
            "body": "<h1>尊敬的客户</h1><p>您的发票已生成</p>",
            "template": "invoice_send",
            "entity_type": "invoices",
            "entity_id": "test-invoice-id"
        }))
        .await;
    res.assert_status(axum::http::StatusCode::SERVICE_UNAVAILABLE);
    let body = res.json::<serde_json::Value>();
    assert_eq!(body["error"], "smtp_transport_unsupported");
    assert!(body["log_id"].as_str().is_some());
}
