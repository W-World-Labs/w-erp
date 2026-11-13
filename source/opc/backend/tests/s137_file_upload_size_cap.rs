/// Scenario 137: multipart upload early size cap (10 MiB).
/// Oversized bodies must return 413 without relying on post-buffer checks.
use crate::common::TestApp;
use axum_test::multipart::{MultipartForm, Part};

#[tokio::test]
async fn s137_upload_rejects_body_over_10mib_with_413() {
    let app = TestApp::new().await;

    // Just over the production cap (10 MiB + 1). Early abort means the handler
    // never accepts/stores this payload even though the global body limit is higher.
    let oversized = vec![0u8; 10 * 1024 * 1024 + 1];
    let form = MultipartForm::new().add_part(
        "file",
        Part::bytes(oversized)
            .file_name("huge.bin")
            .mime_type("application/octet-stream"),
    );

    let resp = app.post("/api/files/upload").multipart(form).await;

    assert_eq!(
        resp.status_code(),
        axum::http::StatusCode::PAYLOAD_TOO_LARGE,
        "oversize upload must be 413; body={}",
        resp.text()
    );
}

#[tokio::test]
async fn s137_upload_accepts_small_file_under_cap() {
    let app = TestApp::new().await;
    let form = MultipartForm::new().add_part(
        "file",
        Part::bytes(b"small-ok".to_vec())
            .file_name("ok.txt")
            .mime_type("text/plain"),
    );

    let resp = app.post("/api/files/upload").multipart(form).await;

    assert_eq!(
        resp.status_code(),
        axum::http::StatusCode::OK,
        "{}",
        resp.text()
    );
    let body: serde_json::Value = resp.json();
    assert_eq!(body["original_name"], "ok.txt");
    assert_eq!(body["size"], 8);
}
