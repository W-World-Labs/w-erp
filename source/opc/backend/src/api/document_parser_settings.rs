use crate::api::error::{ApiError, api_err};
use crate::app::settings_store::{
    self, DocumentParserConfig, KEY_DP_API_KEY, KEY_DP_BACKEND, KEY_DP_BASE_URL, KEY_DP_MODEL,
    KEY_DP_OLLAMA_MODEL, KEY_DP_OLLAMA_URL,
};
use crate::auth::AuthUser;
use crate::runtime_config::{OLLAMA_VISION_BACKEND, OPENAI_VISION_BACKEND};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post, put},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use std::time::Instant;

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route(
            "/settings/document-parser",
            get(get_document_parser).put(put_document_parser),
        )
        .route("/settings/document-parser/test", post(test_document_parser))
        .with_state(pool)
}

async fn require_admin(pool: &SqlitePool, user: &AuthUser) -> Result<(), ApiError> {
    let role: Option<(String,)> =
        sqlx::query_as("SELECT role FROM company_members WHERE user_id = ? AND company_id = ?")
            .bind(&user.user_id)
            .bind(&user.company_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    match role.as_ref().map(|r| r.0.as_str()) {
        Some("owner") | Some("admin") => Ok(()),
        _ => Err(api_err(
            StatusCode::FORBIDDEN,
            "admin or owner role required",
        )),
    }
}

fn config_response(cfg: &DocumentParserConfig) -> Value {
    json!({
        "backend": cfg.backend,
        "base_url": cfg.base_url,
        "api_key_masked": settings_store::mask_api_key(&cfg.api_key),
        "api_key_set": !cfg.api_key.trim().is_empty(),
        "model": cfg.model,
        "ollama_url": cfg.ollama_url,
        "ollama_model": cfg.ollama_model,
        "enabled": settings_store::document_parser_enabled(cfg),
        "backends": [
            {"id": OLLAMA_VISION_BACKEND, "label": "本地 Ollama", "description": "本机 Ollama /api/chat 视觉模型"},
            {"id": OPENAI_VISION_BACKEND, "label": "OpenAI 兼容端点 (vLLM / w-maas 等)", "description": "标准 chat/completions 多模态"}
        ],
        "fields_help": {
            "base_url": "OpenAI 兼容 base 或完整 …/chat/completions",
            "api_key": "Bearer Token；裸 vLLM 可留空",
            "model": "视觉模型名",
            "ollama_url": "Ollama 地址",
            "ollama_model": "Ollama 模型名"
        }
    })
}

async fn get_document_parser(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<Value>, ApiError> {
    require_admin(&pool, &user).await?;
    Ok(Json(config_response(
        &settings_store::resolve_document_parser_config(&pool).await,
    )))
}

#[derive(Deserialize)]
struct PutDocumentParser {
    backend: String,
    #[serde(default)]
    base_url: Option<String>,
    #[serde(default)]
    api_key: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    ollama_url: Option<String>,
    #[serde(default)]
    ollama_model: Option<String>,
    #[serde(default)]
    keep_api_key: Option<bool>,
}

async fn put_document_parser(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Json(body): Json<PutDocumentParser>,
) -> Result<Json<Value>, ApiError> {
    require_admin(&pool, &user).await?;
    let backend = body.backend.trim();
    if backend != OLLAMA_VISION_BACKEND && backend != OPENAI_VISION_BACKEND {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "backend must be ollama-vision or openai-vision",
        ));
    }
    if backend == OPENAI_VISION_BACKEND && body.base_url.as_deref().unwrap_or("").trim().is_empty()
    {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "base_url is required for openai-vision",
        ));
    }
    settings_store::set(&pool, KEY_DP_BACKEND, backend)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "failed to save settings"))?;
    if let Some(v) = &body.base_url {
        settings_store::set(&pool, KEY_DP_BASE_URL, v.trim())
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "failed to save settings"))?;
    }
    if let Some(v) = &body.model {
        settings_store::set(&pool, KEY_DP_MODEL, v.trim())
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "failed to save settings"))?;
    }
    if let Some(v) = &body.ollama_url {
        settings_store::set(&pool, KEY_DP_OLLAMA_URL, v.trim())
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "failed to save settings"))?;
    }
    if let Some(v) = &body.ollama_model {
        settings_store::set(&pool, KEY_DP_OLLAMA_MODEL, v.trim())
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "failed to save settings"))?;
    }
    if !body.keep_api_key.unwrap_or(false) {
        if let Some(v) = &body.api_key {
            if !v.trim().starts_with("****") {
                settings_store::set(&pool, KEY_DP_API_KEY, v.trim())
                    .await
                    .map_err(|_| {
                        api_err(StatusCode::INTERNAL_SERVER_ERROR, "failed to save settings")
                    })?;
            }
        }
    }
    Ok(Json(config_response(
        &settings_store::resolve_document_parser_config(&pool).await,
    )))
}

const TEST_IMAGE_PNG: &[u8] = include_bytes!("../../assets/ocr_test.png");

async fn test_document_parser(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<Value>, ApiError> {
    require_admin(&pool, &user).await?;
    let cfg = settings_store::resolve_document_parser_config(&pool).await;
    if !settings_store::document_parser_enabled(&cfg) {
        return Ok(Json(
            json!({"ok": false, "latency_ms": 0, "model": null, "error": "文档识别未配置完整"}),
        ));
    }
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(TEST_IMAGE_PNG);
    let data_uri = format!("data:image/png;base64,{b64}");
    let started = Instant::now();
    let result = crate::api::ai::run_document_parse_for_test(&pool, &data_uri, "invoice").await;
    let latency_ms = started.elapsed().as_millis() as u64;
    let model = if cfg.backend == OLLAMA_VISION_BACKEND {
        cfg.ollama_model
    } else {
        cfg.model
    };
    match result {
        Ok(_) => Ok(Json(
            json!({"ok": true, "latency_ms": latency_ms, "model": model, "error": null}),
        )),
        Err(status) => Ok(Json(
            json!({"ok": false, "latency_ms": latency_ms, "model": model, "error": format!("识别失败: HTTP {}", status.as_u16())}),
        )),
    }
}
