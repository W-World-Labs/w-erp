//! Middleware: persist HTTP 5xx responses into `error_logs` for the audit center.
//!
//! Failures while writing logs are swallowed — logging must never break a request.
//! Same (source, message) keys are throttled to one insert per 60 seconds.

use axum::{
    body::Body,
    extract::{OriginalUri, State},
    http::Request,
    middleware::Next,
    response::Response,
};
use sqlx::SqlitePool;
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

use crate::auth::middleware::AuthUser;

const THROTTLE_WINDOW: Duration = Duration::from_secs(60);
const MESSAGE_MAX_CHARS: usize = 500;

fn throttle_map() -> &'static Mutex<HashMap<String, Instant>> {
    static MAP: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();
    MAP.get_or_init(|| Mutex::new(HashMap::new()))
}

fn should_record(source: &str, message: &str) -> bool {
    let key = format!("{source}|{message}");
    let mut map = match throttle_map().lock() {
        Ok(g) => g,
        Err(_) => return true,
    };
    let now = Instant::now();
    // Opportunistic prune of stale entries to bound memory.
    if map.len() > 512 {
        map.retain(|_, t| now.duration_since(*t) < THROTTLE_WINDOW);
    }
    if let Some(prev) = map.get(&key)
        && now.duration_since(*prev) < THROTTLE_WINDOW
    {
        return false;
    }
    map.insert(key, now);
    true
}

/// Truncate a body preview for storage.
fn truncate_message(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.chars().count() <= MESSAGE_MAX_CHARS {
        return trimmed.to_string();
    }
    trimmed.chars().take(MESSAGE_MAX_CHARS).collect()
}

/// Insert one server-error row. Safe to call from middleware; errors are ignored.
pub async fn record_server_error(
    pool: &SqlitePool,
    company_id: Option<&str>,
    user_id: Option<&str>,
    method: &str,
    path: &str,
    status: u16,
    body_preview: &str,
) {
    let source = format!("{method} {path}");
    let message = if body_preview.trim().is_empty() {
        format!("HTTP {status}")
    } else {
        truncate_message(body_preview)
    };

    if !should_record(&source, &message) {
        return;
    }

    let id = uuid::Uuid::new_v4().to_string();
    // company_id is NOT NULL in schema; use empty string when unknown.
    let cid = company_id.unwrap_or("").to_string();
    let metadata = serde_json::json!({
        "endpoint": path,
        "method": method,
        "status": status,
        "source": "server_middleware",
    })
    .to_string();

    let _ = sqlx::query(
        r#"
        INSERT INTO error_logs (
            id, company_id, created_by, error_type, error_message,
            page_url, metadata
        ) VALUES (?, ?, ?, 'server_error', ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&cid)
    .bind(user_id)
    .bind(&message)
    .bind(path)
    .bind(&metadata)
    .execute(pool)
    .await;
}

/// Axum middleware: after the handler, persist 5xx responses into error_logs.
pub async fn log_server_errors(
    State(pool): State<SqlitePool>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().as_str().to_string();
    // Prefer OriginalUri so nested routers still log the full public path
    // (e.g. /api/v1/contracts rather than /contracts).
    let path = request
        .extensions()
        .get::<OriginalUri>()
        .map(|uri| uri.path().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());
    let auth = request.extensions().get::<AuthUser>().cloned();

    let response = next.run(request).await;
    let status = response.status();
    if !status.is_server_error() {
        return response;
    }

    let (parts, body) = response.into_parts();
    let bytes = axum::body::to_bytes(body, 64 * 1024)
        .await
        .unwrap_or_default();
    let preview = String::from_utf8_lossy(&bytes).into_owned();

    let company_id = auth.as_ref().map(|a| a.company_id.as_str());
    let user_id = auth.as_ref().map(|a| a.user_id.as_str());
    let status_code = status.as_u16();

    // Fire-and-forget on the current task (insert is cheap; must not fail the response).
    record_server_error(
        &pool,
        company_id,
        user_id,
        &method,
        &path,
        status_code,
        &preview,
    )
    .await;

    Response::from_parts(parts, Body::from(bytes))
}

/// Delete error_logs older than `days` days. Returns rows deleted (best-effort).
pub async fn purge_old_error_logs(pool: &SqlitePool, days: i64) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM error_logs WHERE created_at < datetime('now', ?)")
        .bind(format!("-{days} days"))
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_respects_char_budget() {
        let s: String = "中".repeat(600);
        let t = truncate_message(&s);
        assert_eq!(t.chars().count(), MESSAGE_MAX_CHARS);
    }
}
