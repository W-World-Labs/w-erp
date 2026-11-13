// #13 document parser: openai-vision backend, settings API, capability switch.
use crate::common;
use axum::http::StatusCode;
use serde_json::json;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

struct EnvSnapshot { keys: Vec<&'static str>, values: Vec<Option<String>> }
impl EnvSnapshot {
    fn capture(keys: &[&'static str]) -> Self {
        Self { keys: keys.to_vec(), values: keys.iter().map(|k| std::env::var(k).ok()).collect() }
    }
}
impl Drop for EnvSnapshot {
    fn drop(&mut self) {
        for (key, value) in self.keys.iter().zip(self.values.iter()) {
            unsafe {
                match value { Some(v) => std::env::set_var(key, v), None => std::env::remove_var(key) }
            }
        }
    }
}

fn spawn_mock(status_line: &str, response_body: &str) -> (String, Arc<Mutex<Vec<u8>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let recorded = Arc::new(Mutex::new(Vec::new()));
    let rec = recorded.clone();
    let status = status_line.to_string();
    let body = response_body.to_string();
    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 65536];
            let n = stream.read(&mut buf).unwrap_or(0);
            rec.lock().unwrap().extend_from_slice(&buf[..n]);
            let resp = format!("{status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
            let _ = stream.write_all(resp.as_bytes());
        }
    });
    (format!("http://{addr}"), recorded)
}

#[tokio::test]
async fn openai_vision_sends_multimodal_body_and_succeeds() {
    let _env = common::env_guard();
    let _snap = EnvSnapshot::capture(&["DOCUMENT_PARSER_BACKEND","DOCUMENT_PARSER_BASE_URL","DOCUMENT_PARSER_API_KEY","DOCUMENT_PARSER_MODEL","OLLAMA_URL","OLLAMA_MODEL"]);
    let vision_resp = json!({"choices":[{"message":{"content":"{\"invoice_number\":\"INV-1\",\"amount\":\"100\"}"}}]}).to_string();
    let (base, recorded) = spawn_mock("HTTP/1.1 200 OK", &vision_resp);
    unsafe {
        std::env::set_var("DOCUMENT_PARSER_BACKEND", "openai-vision");
        std::env::set_var("DOCUMENT_PARSER_BASE_URL", format!("{base}/v1"));
        std::env::set_var("DOCUMENT_PARSER_API_KEY", "sk-test-key");
        std::env::set_var("DOCUMENT_PARSER_MODEL", "mimo-v2-omni");
        std::env::remove_var("OLLAMA_URL");
        std::env::remove_var("OLLAMA_MODEL");
    }
    let app = common::TestApp::new().await;
    let res = app.post("/api/ai/parse-document").json(&json!({"file_base64":"data:image/png;base64,iVBORw0KGgo=","document_type":"invoice"})).await;
    assert_eq!(res.status_code(), StatusCode::OK);
    let body: serde_json::Value = res.json();
    assert_eq!(body["success"], true);
    assert_eq!(body["fields"]["invoice_number"], "INV-1");
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let req = String::from_utf8_lossy(&recorded.lock().unwrap()).to_ascii_lowercase();
    assert!(req.contains("authorization: bearer sk-test-key"), "{req}");
    assert!(req.contains("image_url"), "{req}");
    assert!(req.contains("chat/completions"), "{req}");
}

#[tokio::test]
async fn openai_vision_upstream_error_maps_to_502() {
    let _env = common::env_guard();
    let _snap = EnvSnapshot::capture(&["DOCUMENT_PARSER_BACKEND","DOCUMENT_PARSER_BASE_URL","DOCUMENT_PARSER_API_KEY","DOCUMENT_PARSER_MODEL"]);
    let (base, _) = spawn_mock("HTTP/1.1 500 Internal Server Error", "{\"error\":\"boom\"}");
    unsafe {
        std::env::set_var("DOCUMENT_PARSER_BACKEND", "openai-vision");
        std::env::set_var("DOCUMENT_PARSER_BASE_URL", format!("{base}/v1"));
        std::env::set_var("DOCUMENT_PARSER_API_KEY", "sk-x");
        std::env::set_var("DOCUMENT_PARSER_MODEL", "m");
    }
    let app = common::TestApp::new().await;
    let res = app.post("/api/ai/parse-document").json(&json!({"file_base64":"iVBORw0KGgo=","document_type":"invoice"})).await;
    assert_eq!(res.status_code(), StatusCode::BAD_GATEWAY);
}

#[tokio::test]
async fn openai_vision_empty_key_omits_authorization() {
    let _env = common::env_guard();
    let _snap = EnvSnapshot::capture(&["DOCUMENT_PARSER_BACKEND","DOCUMENT_PARSER_BASE_URL","DOCUMENT_PARSER_API_KEY","DOCUMENT_PARSER_MODEL"]);
    let vision_resp = json!({"choices":[{"message":{"content":"{}"}}]}).to_string();
    let (base, recorded) = spawn_mock("HTTP/1.1 200 OK", &vision_resp);
    unsafe {
        std::env::set_var("DOCUMENT_PARSER_BACKEND", "openai-vision");
        std::env::set_var("DOCUMENT_PARSER_BASE_URL", format!("{base}/v1"));
        std::env::remove_var("DOCUMENT_PARSER_API_KEY");
        std::env::set_var("DOCUMENT_PARSER_MODEL", "m");
    }
    let app = common::TestApp::new().await;
    let _ = app.post("/api/ai/parse-document").json(&json!({"file_base64":"iVBORw0KGgo=","document_type":"invoice"})).await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let req = String::from_utf8_lossy(&recorded.lock().unwrap()).to_ascii_lowercase();
    assert!(!req.contains("authorization"), "{req}");
}

#[tokio::test]
async fn unconfigured_openai_returns_501_and_capability_false() {
    let _env = common::env_guard();
    let _snap = EnvSnapshot::capture(&[
        "DOCUMENT_PARSER_BACKEND",
        "DOCUMENT_PARSER_BASE_URL",
        "DOCUMENT_PARSER_API_KEY",
        "DOCUMENT_PARSER_MODEL",
    ]);
    unsafe {
        // Explicit empty strings beat inherited env / defaults for base_url.
        std::env::set_var("DOCUMENT_PARSER_BACKEND", "openai-vision");
        std::env::set_var("DOCUMENT_PARSER_BASE_URL", "");
        std::env::set_var("DOCUMENT_PARSER_API_KEY", "");
        std::env::set_var("DOCUMENT_PARSER_MODEL", "");
    }
    let app = common::TestApp::new().await;
    // Clear any DB-persisted parser settings from other suites sharing fixtures.
    sqlx::query("DELETE FROM runtime_settings WHERE key LIKE 'document_parser.%'")
        .execute(&app.pool)
        .await
        .ok();
    let caps = app.get("/api/ai/capabilities").await;
    assert_eq!(caps.status_code(), StatusCode::OK);
    let caps_body: serde_json::Value = caps.json();
    let enabled = caps_body
        .get("document_parser_enabled")
        .or_else(|| caps_body.pointer("/data/document_parser_enabled"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    assert_eq!(enabled, false, "caps={caps_body}");
    let res = app
        .post("/api/ai/parse-document")
        .json(&json!({"file_base64":"iVBORw0KGgo=","document_type":"invoice"}))
        .await;
    assert_eq!(res.status_code(), StatusCode::NOT_IMPLEMENTED);
}

#[tokio::test]
async fn document_parser_settings_put_get_masks_key() {
    let app = common::TestApp::new().await;
    let put = app.put("/api/v1/settings/document-parser").json(&json!({
        "backend": "openai-vision",
        "base_url": "http://127.0.0.1:9/v1",
        "api_key": "sk-secret-key-1234",
        "model": "mimo-v2-omni"
    })).await;
    assert_eq!(put.status_code(), StatusCode::OK);
    let body: serde_json::Value = put.json();
    assert_eq!(body["api_key_set"], true);
    assert!(!body.to_string().contains("sk-secret-key-1234"));
    let get = app.get("/api/v1/settings/document-parser").await;
    assert!(!get.json::<serde_json::Value>().to_string().contains("sk-secret-key-1234"));
}

#[tokio::test]
async fn document_parser_settings_rejects_invalid_backend() {
    let app = common::TestApp::new().await;
    let put = app.put("/api/v1/settings/document-parser").json(&json!({"backend":"liteparse","base_url":"http://x"})).await;
    assert_eq!(put.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn vision_retries_unusable_output_once_and_never_reports_false_success() {
    use axum::{Router, routing::post, response::Json};
    use std::collections::VecDeque;
    let _env = common::env_guard();
    let _snap = EnvSnapshot::capture(&["DOCUMENT_PARSER_BACKEND", "DOCUMENT_PARSER_BASE_URL", "DOCUMENT_PARSER_API_KEY", "DOCUMENT_PARSER_MODEL"]);
    for (first, second, succeeds) in [
        ("{}", "{}", false), ("[]", "[]", false),
        ("{\"warnings\":[]}", "{\"field_confidence\":{}}", false),
        ("{\"invoice_number\":null,\"amount\":null}", "{}", false),
        ("unreadable image", "still unreadable", false),
        ("{}", "{\"invoice_number\":\"RETRY-OK\",\"total_amount\":\"113\"}", true),
    ] {
        let outputs = Arc::new(Mutex::new(VecDeque::from([first.to_string(), second.to_string()])));
        let queue = outputs.clone();
        let router = Router::new().route("/v1/chat/completions", post(move || {
            let queue = queue.clone();
            async move { Json(json!({"choices":[{"message":{"content":queue.lock().unwrap().pop_front().unwrap_or_default()}}]})) }
        }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap(); });
        unsafe {
            std::env::set_var("DOCUMENT_PARSER_BACKEND", "openai-vision");
            std::env::set_var("DOCUMENT_PARSER_BASE_URL", format!("http://{addr}/v1"));
            std::env::set_var("DOCUMENT_PARSER_API_KEY", "synthetic-key");
            std::env::set_var("DOCUMENT_PARSER_MODEL", "synthetic-vision");
        }
        let app = common::TestApp::new().await;
        let res = app.post("/api/ai/parse-document").json(&json!({"file_base64":"data:image/png;base64,iVBORw0KGgo=", "document_type":"invoice"})).await;
        assert_eq!(res.status_code(), if succeeds { StatusCode::OK } else { StatusCode::BAD_GATEWAY }, "{first} -> {second}");
        if succeeds { assert_eq!(res.json::<serde_json::Value>()["fields"]["total_amount"], "113"); }
        assert!(outputs.lock().unwrap().is_empty(), "must retry exactly once");
        server.abort();
    }
}
