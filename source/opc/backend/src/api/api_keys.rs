use crate::auth::middleware::{AuthUser, generate_api_key, hash_api_key};
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{delete, post},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

// ─── Request / Response Types ───

#[derive(Deserialize)]
pub struct CreateKeyRequest {
    pub name: String,
    #[serde(default = "default_scopes")]
    pub scopes: String,
    pub expires_in_days: Option<i64>,
}

fn default_scopes() -> String {
    "*".to_string()
}

#[derive(Serialize)]
pub struct CreateKeyResponse {
    pub id: String,
    pub name: String,
    pub key: String, // Only returned once at creation!
    pub key_prefix: String,
    pub scopes: String,
    pub created_at: String,
    pub expires_at: Option<String>,
}

#[derive(Serialize)]
pub struct ApiKeyInfo {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub scopes: String,
    pub rate_limit: i64,
    pub last_used_at: Option<String>,
    pub expires_at: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct KeyListResponse {
    pub keys: Vec<ApiKeyInfo>,
    pub total: usize,
}

// ─── Handlers ───

async fn create_key(
    auth: AuthUser,
    State(pool): State<SqlitePool>,
    Json(req): Json<CreateKeyRequest>,
) -> Result<(StatusCode, Json<CreateKeyResponse>), (StatusCode, String)> {
    if auth.api_key_id.is_some() {
        return Err((
            StatusCode::FORBIDDEN,
            "Creating API keys requires a user session".to_owned(),
        ));
    }
    let id = uuid::Uuid::new_v4().to_string();
    let raw_key = generate_api_key();
    let key_hash = hash_api_key(&raw_key);
    let key_prefix = raw_key[..11].to_string(); // "om_" + first 8 hex chars

    let expires_at = req
        .expires_in_days
        .map(|days| format!("datetime('now', '+{} days')", days));

    let expires_sql = if let Some(ref expr) = expires_at {
        expr.as_str()
    } else {
        "NULL"
    };

    sqlx::query(&format!(
        "INSERT INTO api_keys (id, user_id, company_id, name, key_hash, key_prefix, scopes, expires_at) VALUES (?, ?, ?, ?, ?, ?, ?, {})",
        expires_sql
    ))
    .bind(&id)
    .bind(&auth.user_id)
    .bind(&auth.company_id)
    .bind(&req.name)
    .bind(&key_hash)
    .bind(&key_prefix)
    .bind(&req.scopes)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

    // Fetch the created record to get the actual expires_at value
    let created: (String,) = sqlx::query_as("SELECT created_at FROM api_keys WHERE id = ?")
        .bind(&id)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("DB error: {}", e),
            )
        })?;

    let expires: Option<(String,)> =
        sqlx::query_as("SELECT expires_at FROM api_keys WHERE id = ? AND expires_at IS NOT NULL")
            .bind(&id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("DB error: {}", e),
                )
            })?;

    Ok((
        StatusCode::CREATED,
        Json(CreateKeyResponse {
            id,
            name: req.name,
            key: raw_key, // Only shown once!
            key_prefix,
            scopes: req.scopes,
            created_at: created.0,
            expires_at: expires.map(|e| e.0),
        }),
    ))
}

async fn list_keys(
    auth: AuthUser,
    State(pool): State<SqlitePool>,
) -> Result<Json<KeyListResponse>, (StatusCode, String)> {
    let rows: Vec<(String, String, String, String, i64, Option<String>, Option<String>, i64, String)> = sqlx::query_as(
        "SELECT id, name, key_prefix, scopes, rate_limit, last_used_at, expires_at, is_active, created_at FROM api_keys WHERE user_id = ? AND company_id = ? ORDER BY created_at DESC"
    )
    .bind(&auth.user_id)
    .bind(&auth.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

    let total = rows.len();
    let keys: Vec<ApiKeyInfo> = rows
        .into_iter()
        .map(
            |(id, name, prefix, scopes, rate, last_used, expires, active, created)| ApiKeyInfo {
                id,
                name,
                key_prefix: prefix,
                scopes,
                rate_limit: rate,
                last_used_at: last_used,
                expires_at: expires,
                is_active: active == 1,
                created_at: created,
            },
        )
        .collect();

    Ok(Json(KeyListResponse { keys, total }))
}

async fn revoke_key(
    auth: AuthUser,
    State(pool): State<SqlitePool>,
    axum::extract::Path(key_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let result = sqlx::query(
        "UPDATE api_keys SET is_active = 0 WHERE id = ? AND user_id = ? AND company_id = ?",
    )
    .bind(&key_id)
    .bind(&auth.user_id)
    .bind(&auth.company_id)
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("DB error: {}", e),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "API key not found".to_string()));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "API key revoked"
    })))
}

async fn delete_key(
    auth: AuthUser,
    State(pool): State<SqlitePool>,
    axum::extract::Path(key_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let result =
        sqlx::query("DELETE FROM api_keys WHERE id = ? AND user_id = ? AND company_id = ?")
            .bind(&key_id)
            .bind(&auth.user_id)
            .bind(&auth.company_id)
            .execute(&pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("DB error: {}", e),
                )
            })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "API key not found".to_string()));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "API key deleted permanently"
    })))
}

// ─── Router ───

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/", post(create_key).get(list_keys))
        .route("/{id}/revoke", post(revoke_key))
        .route("/{id}", delete(delete_key))
        .with_state(pool)
}
