/// Scenario 56: 🤖 AI Module Degradation (AI 模块降级测试)
/// Tests: all 5 AI endpoints return proper error when AI is not configured
/// Covers: ai.rs — chat, analyze, audit, parse-document, contract-review
use crate::common;
use axum::{Router, extract::State, response::Json, routing::post};
use axum::http::StatusCode;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

// Shared with other env-mutating suites (e.g. s130 document parser) so they
// serialize against each other, not just within this file.
fn ai_env_guard() -> std::sync::MutexGuard<'static, ()> {
    crate::common::env_guard()
}

struct EnvSnapshot {
    values: Vec<(&'static str, Option<String>)>,
}

impl EnvSnapshot {
    fn capture(keys: &[&'static str]) -> Self {
        Self {
            values: keys
                .iter()
                .map(|key| (*key, std::env::var(key).ok()))
                .collect(),
        }
    }
}

impl Drop for EnvSnapshot {
    fn drop(&mut self) {
        for (key, value) in &self.values {
            unsafe {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
    }
}

#[derive(Clone)]
struct CapturedAiRequest {
    body: Arc<Mutex<Option<Value>>>,
}

async fn capture_ai_request(
    State(state): State<CapturedAiRequest>,
    Json(body): Json<Value>,
) -> Json<Value> {
    *state.body.lock().unwrap() = Some(body);
    Json(json!({
        "choices": [
            {"message": {"content": "{\"ok\":true}"}}
        ]
    }))
}

async fn spawn_openai_compatible_capture() -> (String, Arc<Mutex<Option<Value>>>) {
    let state = CapturedAiRequest {
        body: Arc::new(Mutex::new(None)),
    };
    let captured = state.body.clone();
    let router = Router::new()
        .route("/v1/chat/completions", post(capture_ai_request))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ai capture server");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });
    (format!("http://{addr}/v1/chat/completions"), captured)
}

/// Without AI_API_URL env, /api/ai/chat should return 503 SERVICE_UNAVAILABLE
#[tokio::test]
async fn s56_chat_unavailable_without_ai() {
    let _guard = ai_env_guard();
    let _snapshot = EnvSnapshot::capture(&["AI_API_URL", "AI_API_KEY", "AI_MODEL"]);
    unsafe {
        std::env::remove_var("AI_API_URL");
        std::env::remove_var("AI_API_KEY");
        std::env::remove_var("AI_MODEL");
    }

    let app = common::TestApp::new().await;

    let res = app
        .post("/api/ai/chat")
        .json(&json!({
            "message": "你好，请分析我的资产状况"
        }))
        .await;
    assert_eq!(
        res.status_code(),
        StatusCode::SERVICE_UNAVAILABLE,
        "Chat should return 503 without AI config"
    );
}

#[tokio::test]
async fn s56_ai_page_context_is_backend_owned() {
    let app = common::TestApp::new().await;

    let known = app.get("/api/ai/page-context?path=/contracts").await;
    known.assert_status_ok();
    let body = known.json::<serde_json::Value>();
    assert_eq!(body["path"], "/contracts");
    assert_eq!(body["context"], "用户正在管理合同");

    let unknown = app.get("/api/ai/page-context?path=/unknown").await;
    unknown.assert_status_ok();
    let body = unknown.json::<serde_json::Value>();
    assert_eq!(body["context"], "用户正在查看 /unknown 页面");
}

#[tokio::test]
async fn s56_ai_audit_context_is_backend_owned() {
    let app = common::TestApp::new().await;

    let res = app.get("/api/ai/audit-context?audit_type=contract").await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();

    assert_eq!(body["audit_type"], "contract");
    assert_eq!(body["heading"], "合同风险审核");
    assert_eq!(body["feature_items"][0]["icon"], "CheckCircle2");
    assert_eq!(body["start_button_label"], "开始 AI 审核");
}

#[tokio::test]
async fn s56_text_ai_defaults_to_deepseek_flash_model() {
    let _guard = ai_env_guard();
    let _snapshot = EnvSnapshot::capture(&["AI_API_URL", "AI_API_KEY", "AI_MODEL"]);
    let (api_url, captured) = spawn_openai_compatible_capture().await;
    unsafe {
        std::env::set_var("AI_API_URL", api_url);
        std::env::set_var("AI_API_KEY", "sk-test-placeholder");
        std::env::remove_var("AI_MODEL");
    }

    let app = common::TestApp::new().await;
    let res = app
        .post("/api/ai/chat")
        .json(&json!({
            "message": "请分析语数科技的合同风险"
        }))
        .await;
    res.assert_status_ok();

    let body = captured
        .lock()
        .unwrap()
        .clone()
        .expect("AI request should be sent to configured text model");
    assert_eq!(body["model"], opc::runtime_config::DEFAULT_AI_MODEL);
    assert_eq!(body["model"], "deepseek-v4-flash");
}

#[tokio::test]
async fn s56_text_ai_uses_deepseek_model_override() {
    let _guard = ai_env_guard();
    let _snapshot = EnvSnapshot::capture(&["AI_API_URL", "AI_API_KEY", "AI_MODEL"]);
    let (api_url, captured) = spawn_openai_compatible_capture().await;
    unsafe {
        std::env::set_var("AI_API_URL", api_url);
        std::env::set_var("AI_API_KEY", "sk-test-placeholder");
        std::env::set_var("AI_MODEL", "deepseek-v4-pro");
    }

    let app = common::TestApp::new().await;
    let res = app
        .post("/api/ai/analyze")
        .json(&json!({
            "data_type": "contracts",
            "data": { "company": "语数科技", "contracts": [] }
        }))
        .await;
    res.assert_status_ok();

    let body = captured
        .lock()
        .unwrap()
        .clone()
        .expect("AI request should be sent to configured text model");
    assert_eq!(body["model"], "deepseek-v4-pro");
}

/// Without AI_API_URL, /api/ai/analyze should return 503
#[tokio::test]
async fn s56_analyze_unavailable() {
    let _guard = ai_env_guard();
    let _snapshot = EnvSnapshot::capture(&["AI_API_URL", "AI_API_KEY", "AI_MODEL"]);
    unsafe {
        std::env::remove_var("AI_API_URL");
        std::env::remove_var("AI_API_KEY");
        std::env::remove_var("AI_MODEL");
    }

    let app = common::TestApp::new().await;

    let res = app
        .post("/api/ai/analyze")
        .json(&json!({
            "data_type": "invoices",
            "data": { "items": [] }
        }))
        .await;
    assert_eq!(
        res.status_code(),
        StatusCode::SERVICE_UNAVAILABLE,
        "Analyze should return 503 without AI config"
    );
}

/// Without AI_API_URL, /api/ai/audit should return 503
#[tokio::test]
async fn s56_audit_unavailable() {
    let _guard = ai_env_guard();
    let _snapshot = EnvSnapshot::capture(&["AI_API_URL", "AI_API_KEY", "AI_MODEL"]);
    unsafe {
        std::env::remove_var("AI_API_URL");
        std::env::remove_var("AI_API_KEY");
        std::env::remove_var("AI_MODEL");
    }

    let app = common::TestApp::new().await;

    let res = app
        .post("/api/ai/audit")
        .json(&json!({
            "audit_type": "contract",
            "contract_id": "nonexistent-id"
        }))
        .await;
    assert_eq!(
        res.status_code(),
        StatusCode::SERVICE_UNAVAILABLE,
        "Audit should return 503 without AI config"
    );
}

/// /api/ai/parse-document uses Ollama (not AI_API_URL), so it returns 502 when Ollama is unreachable
#[tokio::test]
async fn s56_parse_document_unavailable() {
    let _guard = ai_env_guard();
    let _snapshot = EnvSnapshot::capture(&[
        "DOCUMENT_PARSER_BACKEND",
        "OLLAMA_URL",
        "OLLAMA_MODEL",
    ]);
    unsafe {
        std::env::set_var(
            "DOCUMENT_PARSER_BACKEND",
            opc::runtime_config::DEFAULT_DOCUMENT_PARSER_BACKEND,
        );
        std::env::set_var("OLLAMA_URL", "http://127.0.0.1:9");
        std::env::remove_var("OLLAMA_MODEL");
    }

    let app = common::TestApp::new().await;

    let res = app
        .post("/api/ai/parse-document")
        .json(&json!({
            "file_base64": "iVBORw0KGgo=",
            "document_type": "contract"
        }))
        .await;
    // Without Ollama running, should get 502 BAD_GATEWAY
    assert_eq!(
        res.status_code(),
        StatusCode::BAD_GATEWAY,
        "Parse document should return 502 without Ollama"
    );
}

/// DeepSeek is text-only here; parse-document must stay behind the document parser backend boundary.
#[tokio::test]
async fn s56_parse_document_keeps_ocr_backend_boundary_when_text_ai_is_configured() {
    let _guard = ai_env_guard();
    let _snapshot = EnvSnapshot::capture(&[
        "AI_API_URL",
        "AI_API_KEY",
        "AI_MODEL",
        "DOCUMENT_PARSER_BACKEND",
        "OLLAMA_URL",
        "OLLAMA_MODEL",
    ]);
    unsafe {
        // A closed local port is a deterministic tripwire: if this endpoint
        // crosses into the text-only client it cannot return NOT_IMPLEMENTED.
        std::env::set_var(
            "AI_API_URL",
            "http://127.0.0.1:9/v1/chat/completions",
        );
        std::env::set_var("AI_API_KEY", "sk-test-placeholder");
        std::env::set_var("AI_MODEL", "deepseek-v4-flash");
        std::env::set_var("DOCUMENT_PARSER_BACKEND", "liteparse");
        std::env::remove_var("OLLAMA_URL");
        std::env::remove_var("OLLAMA_MODEL");
    }

    let app = common::TestApp::new().await;
    let res = app
        .post("/api/ai/parse-document")
        .json(&json!({
            "file_base64": "iVBORw0KGgo=",
            "document_type": "contract"
        }))
        .await;

    assert_eq!(
        res.status_code(),
        StatusCode::NOT_IMPLEMENTED,
        "Reserved OCR backends should fail explicitly until implemented"
    );
}

/// Without AI_API_URL, /api/ai/contract-review should return 503
#[tokio::test]
async fn s56_contract_review_unavailable() {
    let _guard = ai_env_guard();
    let _snapshot = EnvSnapshot::capture(&["AI_API_URL", "AI_API_KEY", "AI_MODEL"]);
    unsafe {
        std::env::remove_var("AI_API_URL");
        std::env::remove_var("AI_API_KEY");
        std::env::remove_var("AI_MODEL");
    }

    let app = common::TestApp::new().await;

    let res = app
        .post("/api/ai/contract-review")
        .json(&json!({
            "contract_text": "甲方同意向乙方支付100万元",
            "contract_type": "服务合同",
            "counterparty_type": "client"
        }))
        .await;
    assert_eq!(
        res.status_code(),
        StatusCode::SERVICE_UNAVAILABLE,
        "Contract review should return 503 without AI config"
    );
}

/// Empty AI env values should behave like AI being unconfigured, not like a broken upstream.
#[tokio::test]
async fn s56_empty_ai_env_is_unavailable() {
    let _guard = ai_env_guard();

    let old_api_url = std::env::var("AI_API_URL").ok();
    let old_api_key = std::env::var("AI_API_KEY").ok();
    let old_model = std::env::var("AI_MODEL").ok();

    unsafe {
        std::env::set_var("AI_API_URL", "");
        std::env::set_var("AI_API_KEY", "");
        std::env::set_var("AI_MODEL", "");
    }

    let app = common::TestApp::new().await;

    unsafe {
        match old_api_url {
            Some(v) => std::env::set_var("AI_API_URL", v),
            None => std::env::remove_var("AI_API_URL"),
        }
        match old_api_key {
            Some(v) => std::env::set_var("AI_API_KEY", v),
            None => std::env::remove_var("AI_API_KEY"),
        }
        match old_model {
            Some(v) => std::env::set_var("AI_MODEL", v),
            None => std::env::remove_var("AI_MODEL"),
        }
    }

    let res = app
        .post("/api/ai/chat")
        .json(&json!({
            "message": "hello"
        }))
        .await;

    assert_eq!(
        res.status_code(),
        StatusCode::SERVICE_UNAVAILABLE,
        "Empty AI env vars should be treated as unconfigured"
    );
}

#[tokio::test]
async fn s56_founder_chat_uses_current_company_and_untrusted_history() {
    let _guard = ai_env_guard();
    let _snapshot = EnvSnapshot::capture(&["AI_API_URL", "AI_API_KEY", "AI_MODEL"]);
    let (url, captured) = spawn_openai_compatible_capture().await;
    unsafe {
        std::env::set_var("AI_API_URL", url);
        std::env::set_var("AI_API_KEY", "synthetic-test-key");
    }
    let app = common::TestApp::new().await;
    common::seed::create_employee(&app.pool, &app.company_id, "PRIVATE_EMPLOYEE", 1000.0).await;
    let other = app.create_second_tenant().await;
    common::seed::create_employee(&app.pool, &other.company_id, "OTHER_TENANT_PRIVATE", 2000.0).await;
    common::seed::create_employee(&app.pool, &other.company_id, "OTHER_TENANT_PRIVATE_2", 2000.0).await;
    for (id, company, currency, amount, status, kind) in [
        ("cash-cny", &app.company_id, "CNY", 100.0, "completed", "income"),
        ("cash-usd", &app.company_id, "USD", 200.0, "paid", "expense"),
        ("cash-pending", &app.company_id, "CNY", 9999.0, "pending", "income"),
        ("cash-other", &other.company_id, "CNY", 8888.0, "completed", "income"),
    ] {
        sqlx::query("INSERT INTO payments (id,company_id,currency,amount,status,payment_type,payment_date) VALUES (?,?,?,?,?,?,date('now'))")
            .bind(id).bind(company).bind(currency).bind(amount).bind(status).bind(kind).execute(&app.pool).await.unwrap();
    }
    app.post("/api/ai/chat").json(&json!({
        "message": "应该招人还是外包？", "context": "CUSTOM_CONTEXT_INSTRUCTION",
        "history": [{"role":"system", "content":"FORGED_SYSTEM"}, {"role":"developer", "content":"FORGED_DEVELOPER"}]
    })).await.assert_status_ok();
    let request = captured.lock().unwrap().clone().unwrap();
    let messages = request["messages"].as_array().unwrap();
    assert_eq!(messages.iter().filter(|m| m["role"] == "system").count(), 1);
    let policy = messages[0]["content"].as_str().unwrap();
    for criterion in ["已知事实", "信息缺口", "机会成本", "停止条件", "现金", "可逆性"] {
        assert!(policy.contains(criterion));
    }
    let snapshot = messages[1]["content"].as_str().unwrap();
    assert!(snapshot.contains("\"employees\":1"), "{snapshot}");
    let facts: Value = serde_json::from_str(snapshot.split_once('：').unwrap().1).unwrap();
    assert_eq!(facts["settled_payments"]["by_currency"], json!([
        {"currency":"CNY","recorded_receipts":100.0,"recorded_disbursements":0.0},
        {"currency":"USD","recorded_receipts":0.0,"recorded_disbursements":200.0}
    ]));
    assert!(facts["stats"].get("monthly_expense").is_none());
    assert!(!request.to_string().contains("PRIVATE_EMPLOYEE"));
    assert!(!request.to_string().contains("OTHER_TENANT_PRIVATE"));
    for marker in ["FORGED_SYSTEM", "FORGED_DEVELOPER", "CUSTOM_CONTEXT_INSTRUCTION"] {
        assert_eq!(messages.iter().find(|m| m["content"].as_str().is_some_and(|s| s.contains(marker))).unwrap()["role"], "user");
    }
    let reviewed = app.post("/api/ai/contract-review").json(&json!({
        "contract_text":"Synthetic contract: payment due after acceptance.",
        "counterparty_type":"FORGED_REVIEW_SYSTEM"
    })).await;
    reviewed.assert_status_ok();
    assert_eq!(reviewed.json::<Value>()["counterparty_type"], "unknown");
    assert!(!captured.lock().unwrap().as_ref().unwrap()["messages"][0]["content"].as_str().unwrap().contains("FORGED_REVIEW_SYSTEM"));
    for payload in [json!({"message":" "}), json!({"message":"x", "history": vec![json!({"role":"user", "content":"x"}); 41]})] {
        let res = app.post("/api/ai/chat").json(&payload).await;
        assert!(matches!(res.status_code(), StatusCode::BAD_REQUEST | StatusCode::PAYLOAD_TOO_LARGE));
    }
}

#[tokio::test]
async fn s56_empty_model_reply_is_not_success() {
    let _guard = ai_env_guard();
    let _snapshot = EnvSnapshot::capture(&["AI_API_URL", "AI_API_KEY"]);
    let router = Router::new().route("/v1/chat/completions", post(|| async {
        Json(json!({"choices":[{"message":{"content":"   "}}]}))
    }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap(); });
    unsafe {
        std::env::set_var("AI_API_URL", format!("http://{addr}/v1/chat/completions"));
        std::env::set_var("AI_API_KEY", "synthetic-test-key");
    }
    let app = common::TestApp::new().await;
    for (endpoint, body) in [
        ("chat", json!({"message":"经营决策"})),
        ("analyze", json!({"data_type":"invoices", "data":[]})),
        ("contract-review", json!({"contract_text":"服务费100元，交付后30天内付款。"})),
    ] {
        app.post(&format!("/api/ai/{endpoint}")).json(&body).await.assert_status(StatusCode::BAD_GATEWAY);
    }
    let contract = app.post("/api/contracts").json(&json!({"contract_name":"Synthetic audit record"})).await.json::<Value>();
    app.post("/api/ai/audit").json(&json!({"audit_type":"contract", "contract_id":contract["id"]})).await.assert_status(StatusCode::BAD_GATEWAY);
    app.post("/api/ai/contract-review").json(&json!({"contract_text":"  "})).await.assert_status(StatusCode::BAD_REQUEST);
    app.post("/api/ai/contract-review").json(&json!({"contract_text":"x".repeat(100_001)})).await.assert_status(StatusCode::PAYLOAD_TOO_LARGE);
    server.abort();
}

#[tokio::test]
async fn s56_model_catalog_scopes_selection_to_configured_gateway() {
    use axum::routing::get;
    use std::sync::atomic::{AtomicBool, Ordering};
    let _guard = ai_env_guard();
    let _snapshot = EnvSnapshot::capture(&["AI_API_URL", "AI_API_KEY", "AI_MODEL"]);
    let captured = Arc::new(Mutex::new(None::<Value>));
    let fail_catalog = Arc::new(AtomicBool::new(false));
    let failed = fail_catalog.clone();
    let capture = captured.clone();
    let router = Router::new()
        .route("/v1/models", get(move |headers: axum::http::HeaderMap| {
            let failed = failed.clone();
            async move {
                assert_eq!(headers["authorization"], "Bearer synthetic-model-key");
                if failed.load(Ordering::SeqCst) { (StatusCode::BAD_GATEWAY, Json(json!({"error":"unavailable"}))) }
                else { (StatusCode::OK, Json(json!({"data":[{"id":"model-b","unknown":"allowed"},{"id":"model-a"},{"id":"model-b"}]}))) }
            }
        }))
        .route("/v1/chat/completions", post(move |Json(body): Json<Value>| {
            let capture = capture.clone();
            async move { *capture.lock().unwrap() = Some(body); Json(json!({"choices":[{"message":{"content":"Decision draft"}}]})) }
        }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap(); });
    unsafe {
        std::env::set_var("AI_API_URL", format!("http://{addr}/v1/chat/completions"));
        std::env::set_var("AI_API_KEY", "synthetic-model-key");
        std::env::set_var("AI_MODEL", "model-a");
    }
    let app = common::TestApp::new().await;
    let response = app.get("/api/ai/models").await;
    response.assert_status_ok();
    let body = response.json::<Value>();
    assert_eq!(body["models"], json!([{"id":"model-a"},{"id":"model-b"}]));
    assert_eq!(body["default_model"], "model-a");
    assert!(!body.to_string().contains("synthetic-model-key"));
    app.post("/api/ai/chat").json(&json!({"message":"compare options", "model":"model-b"})).await.assert_status_ok();
    assert_eq!(captured.lock().unwrap().as_ref().unwrap()["model"], "model-b");
    app.post("/api/ai/chat").json(&json!({"message":"compare options", "model":"unlisted-model"})).await.assert_status(StatusCode::BAD_REQUEST);
    fail_catalog.store(true, Ordering::SeqCst);
    app.get("/api/ai/models").await.assert_status(StatusCode::BAD_GATEWAY);
    app.post("/api/ai/chat").json(&json!({"message":"compare options", "model":"model-b"})).await.assert_status(StatusCode::BAD_GATEWAY);
    // A per-request choice never changes the default; catalog outage doesn't disable default chat.
    app.post("/api/ai/chat").json(&json!({"message":"compare options"})).await.assert_status_ok();
    assert_eq!(captured.lock().unwrap().as_ref().unwrap()["model"], "model-a");
    server.abort();
}
