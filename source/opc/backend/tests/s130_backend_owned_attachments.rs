/// Scenario 130: generic attachments use backend-owned file display metadata.
use crate::common;

use serde_json::Value;

#[tokio::test]
async fn s130_entity_file_list_returns_attachment_display_aliases() {
    let app = common::TestApp::new().await;

    sqlx::query(
        "INSERT INTO files (
            id, company_id, filename, original_name, content_type, size, entity_type, entity_id, created_at
        ) VALUES (
            'attachment-display', ?, 'stored-hash', 'receipt.pdf', 'application/pdf', 2048, 'payments', 'payment-1', '2026-03-04T05:06:07Z'
        )",
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();

    let res = app.get("/api/v1/files/list/payment/payment-1").await;
    res.assert_status_ok();
    let body: Value = res.json();
    let item = &body["data"][0];

    assert_eq!(item["file_name"], "receipt.pdf");
    assert_eq!(item["file_path"], "attachment-display");
    assert_eq!(item["file_type"], "application/pdf");
    assert_eq!(item["upload_date_label"], "2026-03-04");
    assert_eq!(item["file_icon"], "FileText");
    assert!(
        item["file_icon_class_name"]
            .as_str()
            .unwrap()
            .contains("red")
    );
    assert_eq!(
        item["delete_action"]["endpoint"],
        "/api/files/attachment-display"
    );
    assert_eq!(
        item["delete_action"]["confirm_message"],
        "确定要删除附件「receipt.pdf」吗？此操作无法撤销。"
    );
    assert_eq!(item["delete_action"]["success_message"], "附件已删除");
}
