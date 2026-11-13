/// Scenario 53: 📁 File API — 文件管理/安全
/// Tests the file management endpoints
use crate::common;

#[tokio::test]
async fn s53_file_list_empty() {
    let app = common::TestApp::new().await;

    // List files for a non-existent entity should return empty
    let res = app.get("/api/files/list/invoice/fake-invoice-id").await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn s53_file_download_not_found() {
    let app = common::TestApp::new().await;

    // Download non-existent file should 404
    let res = app.get("/api/files/non-existent-id").await;
    assert_eq!(res.status_code(), 404);
}

#[tokio::test]
async fn s53_file_static_path_traversal() {
    let app = common::TestApp::new().await;

    // Path traversal attempts must never leak file contents. Axum normalises
    // the URL so the request either 4xx/5xx or falls through to the SPA.
    let res = app
        .server
        .get("/api/files/static/../../../etc/passwd")
        .await;
    let body = res.text();
    assert!(!body.contains("root:"), "path traversal leaked /etc/passwd");
}

#[tokio::test]
async fn s53_file_static_dotdot_variant() {
    let app = common::TestApp::new().await;

    let res = app.server.get("/api/files/static/./../../secret").await;
    let body = res.text();
    assert!(!body.contains("root:"), "path traversal leaked sensitive data");
}

#[tokio::test]
async fn s53_file_static_not_found() {
    let app = common::TestApp::new().await;

    // Authenticated request for a non-existent file in the caller's own
    // company directory → 404 from the disk lookup
    let res = app
        .get(&format!(
            "/api/files/static/{}/no-such-file.png",
            app.company_id
        ))
        .await;
    assert!(res.status_code().is_client_error());
}

#[tokio::test]
async fn s53_file_static_requires_auth() {
    let app = common::TestApp::new().await;

    let res = app
        .server
        .get("/api/files/static/company123/some-file.png")
        .await;
    assert_eq!(res.status_code(), 401);
}

#[tokio::test]
async fn s53_file_static_cross_company_blocked() {
    let app = common::TestApp::new().await;

    // Authenticated caller requesting another company's path prefix must be
    // rejected by the tenant ownership check (404, no disk access)
    let res = app
        .get("/api/files/static/other-company-id/known-content-hash.png")
        .await;
    assert_eq!(res.status_code(), 404);
}

#[tokio::test]
async fn s53_file_delete_not_found() {
    let app = common::TestApp::new().await;

    // Delete non-existent file should align with download semantics and return 404
    let res = app.delete("/api/files/non-existent-id").await;
    assert_eq!(res.status_code(), 404);
}

#[tokio::test]
async fn s53_file_upload_record_via_db() {
    let app = common::TestApp::new().await;

    // Manually insert a file record and verify singular/plural entity aliases resolve the same record
    sqlx::query(
        "INSERT INTO files (id, company_id, filename, original_name, content_type, size, entity_type, entity_id, uploaded_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind("test-file-001")
    .bind(&app.company_id)
    .bind("abc123hash")
    .bind("invoice_scan.pdf")
    .bind("application/pdf")
    .bind(1024i64)
    .bind("invoices")
    .bind("inv-001")
    .bind(&app.user_id)
    .execute(&app.pool)
    .await
    .unwrap();

    // List files using singular alias; API should normalize to the stored canonical type
    let res = app.get("/api/files/list/invoice/inv-001").await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    let files = body["data"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(
        files[0]["original_name"].as_str().unwrap(),
        "invoice_scan.pdf"
    );
    assert_eq!(
        files[0]["content_type"].as_str().unwrap(),
        "application/pdf"
    );
    assert_eq!(files[0]["size_label"], "1 KB");
    assert_eq!(files[0]["created_at_label"].as_str().unwrap().len(), 10);
    assert_eq!(files[0]["metadata_label"], "application/pdf · 1 KB");
}

#[tokio::test]
async fn s53_file_tenant_isolation() {
    let app = common::TestApp::new().await;
    let app_b = app.create_second_tenant().await;

    // A inserts a file record
    sqlx::query(
        "INSERT INTO files (id, company_id, filename, original_name, content_type, size, uploaded_by) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind("tenant-file-001")
    .bind(&app.company_id)
    .bind("hash1")
    .bind("secret.pdf")
    .bind("application/pdf")
    .bind(2048i64)
    .bind(&app.user_id)
    .execute(&app.pool)
    .await
    .unwrap();

    // B should not be able to download A's file
    let res = app_b.get("/api/files/tenant-file-001").await;
    // Should fail (either 404 or 403 — file is company-scoped)
    assert!(res.status_code().is_client_error());
}
