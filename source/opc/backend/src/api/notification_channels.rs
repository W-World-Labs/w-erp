use crate::api::error::{ApiError, api_err};
use crate::auth::AuthUser;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::SqlitePool;

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route(
            "/notification_channels",
            get(list_channels).post(create_channel),
        )
        .route(
            "/notification_channels/{id}",
            get(get_channel).put(update_channel).delete(delete_channel),
        )
        .route("/settings/notifications/{id}/test", post(test_channel))
        .with_state(pool)
}

async fn list_channels(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<Value>, ApiError> {
    let rows: Vec<(String, Option<String>, String, String, i64, String)> = sqlx::query_as(
        "SELECT id, name, channel_url, mode, enabled, created_at FROM notification_channels WHERE company_id = ? ORDER BY created_at DESC",
    ).bind(&user.company_id).fetch_all(&pool).await.map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    let items: Vec<Value> = rows.into_iter().map(|(id, name, url, mode, enabled, created_at)| json!({
        "id": id, "name": name.unwrap_or_default(), "channel_url": url, "mode": mode,
        "enabled": enabled != 0, "created_at": created_at,
        "test_action": {"method": "POST", "endpoint": format!("/api/v1/settings/notifications/{id}/test"), "label": "发送测试消息"}
    })).collect();
    Ok(Json(json!({
        "data": items,
        "help": "支持飞书/企微/钉钉/Telegram/Slack/通用 webhook：粘贴机器人 Webhook URL。mode=instant 每条即发，digest 每日 09:00 摘要。"
    })))
}

async fn get_channel(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let row: Option<(String, Option<String>, String, String, i64, String)> = sqlx::query_as(
        "SELECT id, name, channel_url, mode, enabled, created_at FROM notification_channels WHERE id = ? AND company_id = ?",
    ).bind(&id).bind(&user.company_id).fetch_optional(&pool).await.map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    let (id, name, url, mode, enabled, created_at) =
        row.ok_or_else(|| api_err(StatusCode::NOT_FOUND, "channel not found"))?;
    Ok(Json(
        json!({"id": id, "name": name.unwrap_or_default(), "channel_url": url, "mode": mode, "enabled": enabled != 0, "created_at": created_at}),
    ))
}

#[derive(Deserialize)]
struct ChannelBody {
    name: Option<String>,
    channel_url: String,
    #[serde(default = "default_mode")]
    mode: String,
    #[serde(default = "default_enabled")]
    enabled: Option<bool>,
}
fn default_mode() -> String {
    "digest".into()
}
fn default_enabled() -> Option<bool> {
    Some(true)
}

fn validate_mode(mode: &str) -> Result<&str, ApiError> {
    match mode {
        "instant" | "digest" => Ok(mode),
        _ => Err(api_err(
            StatusCode::BAD_REQUEST,
            "mode must be 'instant' or 'digest'",
        )),
    }
}

async fn create_channel(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Json(body): Json<ChannelBody>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let url = body.channel_url.trim();
    if url.is_empty() {
        return Err(api_err(StatusCode::BAD_REQUEST, "channel_url is required"));
    }
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "channel_url must be http(s) URL",
        ));
    }
    let mode = validate_mode(body.mode.trim())?;
    let enabled = if body.enabled.unwrap_or(true) { 1 } else { 0 };
    let id = uuid::Uuid::new_v4().to_string();
    let name = body.name.unwrap_or_default();
    sqlx::query("INSERT INTO notification_channels (id, company_id, name, channel_url, mode, enabled) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&id).bind(&user.company_id).bind(&name).bind(url).bind(mode).bind(enabled)
        .execute(&pool).await.map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    Ok((
        StatusCode::CREATED,
        Json(
            json!({"id": id, "name": name, "channel_url": url, "mode": mode, "enabled": enabled != 0}),
        ),
    ))
}

async fn update_channel(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<ChannelBody>,
) -> Result<Json<Value>, ApiError> {
    let url = body.channel_url.trim();
    if url.is_empty() {
        return Err(api_err(StatusCode::BAD_REQUEST, "channel_url is required"));
    }
    let mode = validate_mode(body.mode.trim())?;
    let enabled = if body.enabled.unwrap_or(true) { 1 } else { 0 };
    let name = body.name.unwrap_or_default();
    let result = sqlx::query("UPDATE notification_channels SET name = ?, channel_url = ?, mode = ?, enabled = ? WHERE id = ? AND company_id = ?")
        .bind(&name).bind(url).bind(mode).bind(enabled).bind(&id).bind(&user.company_id)
        .execute(&pool).await.map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    if result.rows_affected() == 0 {
        return Err(api_err(StatusCode::NOT_FOUND, "channel not found"));
    }
    Ok(Json(
        json!({"id": id, "name": name, "channel_url": url, "mode": mode, "enabled": enabled != 0}),
    ))
}

async fn delete_channel(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query("DELETE FROM notification_channels WHERE id = ? AND company_id = ?")
        .bind(&id)
        .bind(&user.company_id)
        .execute(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    if result.rows_affected() == 0 {
        return Err(api_err(StatusCode::NOT_FOUND, "channel not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn test_channel(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT channel_url FROM notification_channels WHERE id = ? AND company_id = ?",
    )
    .bind(&id)
    .bind(&user.company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    let (url,) = row.ok_or_else(|| api_err(StatusCode::NOT_FOUND, "channel not found"))?;
    match crate::app::notifications::send_test_message(&url).await {
        Ok(()) => Ok(Json(json!({"ok": true, "message": "测试消息已发送"}))),
        Err(err) => Ok(Json(
            json!({"ok": false, "message": format!("发送失败: {err}")}),
        )),
    }
}
