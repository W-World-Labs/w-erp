//! Global runtime settings KV store. Priority: database > env > default.

use sqlx::SqlitePool;

pub async fn get(pool: &SqlitePool, key: &str) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM runtime_settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0))
}

pub async fn set(pool: &SqlitePool, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO runtime_settings (key, value, updated_at) VALUES (?, ?, datetime('now')) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_with_env_fallback(
    pool: &SqlitePool,
    key: &str,
    env_key: &str,
    default: &str,
) -> String {
    if let Ok(Some(v)) = get(pool, key).await {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    if let Ok(v) = std::env::var(env_key) {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    default.to_string()
}

pub const KEY_DP_BACKEND: &str = "document_parser.backend";
pub const KEY_DP_BASE_URL: &str = "document_parser.base_url";
pub const KEY_DP_API_KEY: &str = "document_parser.api_key";
pub const KEY_DP_MODEL: &str = "document_parser.model";
pub const KEY_DP_OLLAMA_URL: &str = "document_parser.ollama_url";
pub const KEY_DP_OLLAMA_MODEL: &str = "document_parser.ollama_model";

pub const ENV_DP_BACKEND: &str = "DOCUMENT_PARSER_BACKEND";
pub const ENV_DP_BASE_URL: &str = "DOCUMENT_PARSER_BASE_URL";
pub const ENV_DP_API_KEY: &str = "DOCUMENT_PARSER_API_KEY";
pub const ENV_DP_MODEL: &str = "DOCUMENT_PARSER_MODEL";
pub const ENV_OLLAMA_URL: &str = "OLLAMA_URL";
pub const ENV_OLLAMA_MODEL: &str = "OLLAMA_MODEL";

pub const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";
pub const DEFAULT_OLLAMA_MODEL: &str = "paddleocr-vl:1.5";
pub const DEFAULT_OPENAI_VISION_MODEL: &str = "mimo-v2-omni";

#[derive(Debug, Clone)]
pub struct DocumentParserConfig {
    pub backend: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub ollama_url: String,
    pub ollama_model: String,
}

pub async fn resolve_document_parser_config(pool: &SqlitePool) -> DocumentParserConfig {
    DocumentParserConfig {
        backend: get_with_env_fallback(
            pool,
            KEY_DP_BACKEND,
            ENV_DP_BACKEND,
            crate::runtime_config::DEFAULT_DOCUMENT_PARSER_BACKEND,
        )
        .await,
        base_url: get_with_env_fallback(pool, KEY_DP_BASE_URL, ENV_DP_BASE_URL, "").await,
        api_key: get_with_env_fallback(pool, KEY_DP_API_KEY, ENV_DP_API_KEY, "").await,
        model: get_with_env_fallback(
            pool,
            KEY_DP_MODEL,
            ENV_DP_MODEL,
            DEFAULT_OPENAI_VISION_MODEL,
        )
        .await,
        ollama_url: get_with_env_fallback(
            pool,
            KEY_DP_OLLAMA_URL,
            ENV_OLLAMA_URL,
            DEFAULT_OLLAMA_URL,
        )
        .await,
        ollama_model: get_with_env_fallback(
            pool,
            KEY_DP_OLLAMA_MODEL,
            ENV_OLLAMA_MODEL,
            DEFAULT_OLLAMA_MODEL,
        )
        .await,
    }
}

pub fn document_parser_enabled(cfg: &DocumentParserConfig) -> bool {
    match cfg.backend.as_str() {
        crate::runtime_config::OLLAMA_VISION_BACKEND => {
            !cfg.ollama_url.trim().is_empty() && !cfg.ollama_model.trim().is_empty()
        }
        crate::runtime_config::OPENAI_VISION_BACKEND => {
            !cfg.base_url.trim().is_empty() && !cfg.model.trim().is_empty()
        }
        _ => false,
    }
}

pub fn mask_api_key(key: &str) -> String {
    let key = key.trim();
    if key.is_empty() {
        return String::new();
    }
    if key.chars().count() <= 4 {
        return "****".to_string();
    }
    let tail: String = key
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("****{tail}")
}

pub fn openai_chat_completions_url(base_or_full: &str) -> String {
    let base = base_or_full.trim().trim_end_matches('/');
    if base.is_empty() {
        return String::new();
    }
    if base.ends_with("/chat/completions") {
        base.to_string()
    } else {
        format!("{base}/chat/completions")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.expect("db");
        db::run_migrations(&pool).await.expect("mig");
        pool
    }

    #[tokio::test]
    async fn priority_db_over_env_over_default() {
        let pool = test_pool().await;
        let key = "test.priority";
        let env_key = "TEST_PRIORITY_SETTING_G5_ISO";
        assert_eq!(
            get_with_env_fallback(&pool, key, env_key, "default-val").await,
            "default-val"
        );
        unsafe {
            std::env::set_var(env_key, "env-val");
        }
        assert_eq!(
            get_with_env_fallback(&pool, key, env_key, "default-val").await,
            "env-val"
        );
        set(&pool, key, "db-val").await.unwrap();
        assert_eq!(
            get_with_env_fallback(&pool, key, env_key, "default-val").await,
            "db-val"
        );
        unsafe {
            std::env::remove_var(env_key);
        }
    }

    #[test]
    fn mask_and_url_and_enabled() {
        assert_eq!(mask_api_key("sk-abcdefgh"), "****efgh");
        assert_eq!(
            openai_chat_completions_url("http://host:8000/v1"),
            "http://host:8000/v1/chat/completions"
        );
        let ok = DocumentParserConfig {
            backend: crate::runtime_config::OPENAI_VISION_BACKEND.into(),
            base_url: "http://x/v1".into(),
            api_key: String::new(),
            model: "m".into(),
            ollama_url: String::new(),
            ollama_model: String::new(),
        };
        assert!(document_parser_enabled(&ok));
        let bad = DocumentParserConfig {
            base_url: String::new(),
            ..ok.clone()
        };
        assert!(!document_parser_enabled(&bad));
    }
}
