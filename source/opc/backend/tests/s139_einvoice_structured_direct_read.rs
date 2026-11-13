// #14 OCR quality: e-invoice structured direct-read (XML/OFD) before vision OCR.
use crate::common;
use axum::http::StatusCode;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::json;

fn fixture_b64(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/einvoice/{}",
        env!("CARGO_MANIFEST_DIR"),
        name
    );
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    STANDARD.encode(bytes)
}

#[tokio::test]
async fn parse_document_xml_structured_without_vision_backend() {
    // Serialize with other env-mutating suites (process-global env).
    let _env = common::env_guard();
    // Empty vision config must still succeed for structured XML direct-read.
    let _snap = EnvSnapshot::capture(&[
        "DOCUMENT_PARSER_BACKEND",
        "DOCUMENT_PARSER_BASE_URL",
        "DOCUMENT_PARSER_API_KEY",
        "DOCUMENT_PARSER_MODEL",
        "OLLAMA_URL",
        "OLLAMA_MODEL",
    ]);
    unsafe {
        std::env::set_var("DOCUMENT_PARSER_BACKEND", "openai-vision");
        std::env::set_var("DOCUMENT_PARSER_BASE_URL", "");
        std::env::set_var("DOCUMENT_PARSER_API_KEY", "");
        std::env::set_var("DOCUMENT_PARSER_MODEL", "");
        std::env::remove_var("OLLAMA_URL");
        std::env::remove_var("OLLAMA_MODEL");
    }

    let app = common::TestApp::new().await;
    let b64 = fixture_b64("sample_utf8.xml");
    let res = app
        .post("/api/ai/parse-document")
        .json(&json!({
            "file_base64": format!("data:application/xml;base64,{b64}"),
            "document_type": "invoice",
            "filename": "sample_utf8.xml",
        }))
        .await;
    assert_eq!(res.status_code(), StatusCode::OK, "body={}", res.text());
    let body: serde_json::Value = res.json();
    assert_eq!(body["success"], true);
    assert_eq!(body["parse_source"], "xml");
    assert_eq!(body["fields"]["invoice_number"], "25332000000012345678");
    assert_eq!(body["fields"]["supplier"], "测试销方科技有限公司");
    assert_eq!(body["fields"]["buyer"], "测试购方有限公司");
}

#[tokio::test]
async fn parse_document_ofd_structured_direct_read() {
    let app = common::TestApp::new().await;
    let b64 = fixture_b64("sample_ofd_with_attach.ofd");
    let res = app
        .post("/api/ai/parse-document")
        .json(&json!({
            "file_base64": b64,
            "document_type": "invoice",
            "filename": "sample.ofd",
        }))
        .await;
    assert_eq!(res.status_code(), StatusCode::OK, "body={}", res.text());
    let body: serde_json::Value = res.json();
    assert_eq!(body["success"], true);
    let src = body["parse_source"].as_str().unwrap_or("");
    assert!(
        src == "ofd_attachment" || src == "ofd_page_text" || src == "xml",
        "unexpected source {src}"
    );
    assert!(
        body["fields"]["invoice_number"]
            .as_str()
            .is_some_and(|s| !s.is_empty()),
        "fields={:?}",
        body["fields"]
    );
}

// Local env snapshot helper (same pattern as s130).
struct EnvSnapshot {
    keys: Vec<&'static str>,
    values: Vec<Option<String>>,
}
impl EnvSnapshot {
    fn capture(keys: &[&'static str]) -> Self {
        Self {
            keys: keys.to_vec(),
            values: keys.iter().map(|k| std::env::var(k).ok()).collect(),
        }
    }
}
impl Drop for EnvSnapshot {
    fn drop(&mut self) {
        for (key, value) in self.keys.iter().zip(self.values.iter()) {
            unsafe {
                match value {
                    Some(v) => std::env::set_var(key, v),
                    None => std::env::remove_var(key),
                }
            }
        }
    }
}
