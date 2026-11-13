use crate::common;
use axum::http::StatusCode;
use base64::{Engine as _, engine::general_purpose};
use serde_json::{Value, json};

#[tokio::test]
async fn s139_mailbox_configuration_is_tenant_scoped_and_password_is_redacted() {
    let app = common::TestApp::new().await;
    let created = app
        .post("/api/v1/invoice-mailboxes")
        .json(&json!({
            "name": "电子票邮箱",
            "imap_host": "imap.example.test",
            "imap_port": 993,
            "imap_username": "invoice@example.test",
            "imap_password": "application-secret",
            "folder": "INBOX",
            "enabled": false
        }))
        .await;
    created.assert_status(StatusCode::CREATED);
    let id = created.json::<Value>()["id"].as_str().unwrap().to_string();

    let list = app.get("/api/v1/invoice-mailboxes").await;
    list.assert_status_ok();
    let body = list.json::<Value>();
    assert_eq!(body["data"][0]["id"], id);
    assert_eq!(body["data"][0]["has_password"], true);
    assert!(body["data"][0].get("imap_password").is_none());
    assert!(!body.to_string().contains("application-secret"));

    app.put(&format!("/api/v1/invoice-mailboxes/{id}"))
        .json(&json!({
            "name": "电子票邮箱（更新）",
            "imap_host": "imap.example.test",
            "imap_port": 993,
            "imap_username": "invoice@example.test",
            "imap_password": null,
            "folder": "INBOX",
            "enabled": false
        }))
        .await
        .assert_status_ok();
    let stored_password: String =
        sqlx::query_scalar("SELECT imap_password FROM invoice_mailboxes WHERE id = ?")
            .bind(&id)
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!(stored_password, "application-secret");

    let other = app.create_second_tenant().await;
    let other_list = other.get("/api/v1/invoice-mailboxes").await;
    other_list.assert_status_ok();
    assert!(
        other_list.json::<Value>()["data"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn s139_pending_xml_import_requires_confirmation_before_invoice_creation() {
    let app = common::TestApp::new().await;
    sqlx::query(
        "INSERT INTO invoice_mailboxes (id, company_id, name, imap_host, imap_username, imap_password, enabled) VALUES ('mail-1', ?, '票箱', 'imap.example.test', 'u', 'p', 0)",
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();

    let xml = r#"<Invoice><Fpdm>044001</Fpdm><Fphm>9001</Fphm><Kprq>2026-08-12</Kprq><Xfmc>XML供应商</Xfmc><Hjje>300.25</Hjje><Hjse>39.03</Hjse><Spxx>服务费</Spxx></Invoice>"#;
    let preview = app
        .post("/api/v1/imports/invoices/preview")
        .json(&json!({
            "filename": "invoice.xml",
            "file_base64": general_purpose::STANDARD.encode(xml.as_bytes()),
            "import_format": "xml"
        }))
        .await;
    preview.assert_status_ok();
    let invoice = preview.json::<Value>()["rows"][0].clone();
    assert_eq!(invoice["status"], "valid");

    sqlx::query(
        r#"
        INSERT INTO invoice_mail_imports (
            id, company_id, mailbox_id, message_uid, attachment_index,
            attachment_name, content_sha256, xml_content, invoice_number,
            parsed_json, status
        ) VALUES ('import-1', ?, 'mail-1', 10, 1, 'invoice.xml', 'hash-1', ?, ?, ?, 'pending_confirmation')
        "#,
    )
    .bind(&app.company_id)
    .bind(xml.as_bytes())
    .bind(invoice["invoice_number"].as_str().unwrap())
    .bind(serde_json::to_string(&invoice).unwrap())
    .execute(&app.pool)
    .await
    .unwrap();

    let before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM invoices WHERE company_id = ?")
        .bind(&app.company_id)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(before, 0);

    let queue = app
        .get("/api/v1/invoice-mail-imports?status=pending_confirmation")
        .await;
    queue.assert_status_ok();
    let queue_body = queue.json::<Value>();
    assert_eq!(queue_body["stats"]["pending_confirmation"], 1);
    assert_eq!(queue_body["data"][0]["status_label"], "待确认");
    assert!(queue_body["data"][0]["confirm_action"].is_object());
    assert!(queue_body.to_string().contains("XML供应商"));
    assert!(!queue_body.to_string().contains("<Invoice>"));

    let confirmed = app
        .post("/api/v1/invoice-mail-imports/import-1/confirm")
        .json(&json!({}))
        .await;
    confirmed.assert_status_ok();
    let confirmed_body = confirmed.json::<Value>();
    assert_eq!(confirmed_body["status"], "imported");

    let stored = sqlx::query(
        "SELECT invoice_number, status, supplier, amount FROM invoices WHERE id = ? AND company_id = ?",
    )
    .bind(confirmed_body["invoice_id"].as_str().unwrap())
    .bind(&app.company_id)
    .fetch_one(&app.pool)
    .await
    .unwrap();
    use sqlx::Row;
    assert_eq!(stored.get::<String, _>("invoice_number"), "044001-9001");
    assert_eq!(stored.get::<String, _>("status"), "pending");
    assert_eq!(stored.get::<String, _>("supplier"), "XML供应商");
    assert_eq!(stored.get::<f64, _>("amount"), 300.25);

    app.post("/api/v1/invoice-mail-imports/import-1/confirm")
        .json(&json!({}))
        .await
        .assert_status_conflict();
}

#[tokio::test]
async fn s139_disabled_mailbox_cannot_sync_and_exposes_failure_without_network() {
    let app = common::TestApp::new().await;
    sqlx::query(
        "INSERT INTO invoice_mailboxes (id, company_id, name, imap_host, imap_username, imap_password, enabled) VALUES ('disabled-mail', ?, '停用票箱', 'imap.example.test', 'u', 'p', 0)",
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();
    app.post("/api/v1/invoice-mailboxes/disabled-mail/sync")
        .json(&json!({}))
        .await
        .assert_status(StatusCode::CONFLICT);
}
