use super::error::{ApiError, api_err};
use crate::app::collections::{
    CollectionSettings, history_for_invoice, load_settings, save_settings,
};
use crate::auth::AuthUser;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::get,
};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::{Row, SqlitePool};

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route(
            "/settings/collections",
            get(get_collection_settings).put(update_collection_settings),
        )
        .route("/collections/history", get(collection_history))
        .with_state(pool)
}

async fn get_collection_settings(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<Value>, ApiError> {
    let settings = load_settings(&pool, &user.company_id)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "fetch settings failed"))?;
    Ok(Json(settings_payload(settings)))
}

async fn update_collection_settings(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Json(settings): Json<CollectionSettings>,
) -> Result<Json<Value>, ApiError> {
    if !can_manage(&pool, &user).await? {
        return Err(api_err(StatusCode::FORBIDDEN, "forbidden"));
    }
    settings
        .validate()
        .map_err(|message| api_err(StatusCode::BAD_REQUEST, message))?;
    save_settings(&pool, &user.company_id, &settings)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "save settings failed"))?;

    let details = json!({
        "enabled": settings.enabled,
        "friendly_days": settings.friendly_days,
        "formal_days": settings.formal_days,
        "final_days": settings.final_days,
        "bad_debt_days": settings.bad_debt_days,
    })
    .to_string();
    sqlx::query(
        "INSERT INTO operation_logs \
         (id, company_id, user_id, action, entity_type, entity_id, details) \
         VALUES (?, ?, ?, 'update', 'collection_settings', ?, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&user.company_id)
    .bind(&user.user_id)
    .bind(&user.company_id)
    .bind(details)
    .execute(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "write audit log failed"))?;

    Ok(Json(settings_payload(settings)))
}

#[derive(Debug, Deserialize, Default)]
struct HistoryQuery {
    invoice_id: Option<String>,
    client_id: Option<String>,
    limit: Option<i64>,
}

async fn collection_history(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Value>, ApiError> {
    if let Some(invoice_id) = query
        .invoice_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let history = history_for_invoice(&pool, &user.company_id, invoice_id)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "fetch history failed"))?;
        let total = history.len();
        return Ok(Json(json!({
            "history": history,
            "total": total,
            "invoice_id": invoice_id,
        })));
    }

    let limit = query.limit.unwrap_or(100).clamp(1, 200);
    let rows = sqlx::query(
        r#"
        SELECT id, invoice_id, client_id, invoice_number, client_name, amount_due,
               days_overdue, level_code, level_label, threshold_days,
               channel_id, channel_name, channel_mode, status, attempt_count,
               error, created_at, last_attempt_at, sent_at
        FROM collection_deliveries
        WHERE company_id = ?
          AND (? IS NULL OR client_id = ?)
        ORDER BY created_at DESC
        LIMIT ?
        "#,
    )
    .bind(&user.company_id)
    .bind(&query.client_id)
    .bind(&query.client_id)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "fetch history failed"))?;

    let history: Vec<Value> = rows
        .into_iter()
        .map(|row| {
            let status: String = row.try_get("status").unwrap_or_default();
            json!({
                "id": row.try_get::<String, _>("id").unwrap_or_default(),
                "invoice_id": row.try_get::<String, _>("invoice_id").unwrap_or_default(),
                "client_id": row.try_get::<Option<String>, _>("client_id").unwrap_or(None),
                "invoice_number": row.try_get::<String, _>("invoice_number").unwrap_or_default(),
                "client_name": row.try_get::<Option<String>, _>("client_name").unwrap_or(None),
                "amount_due": row.try_get::<f64, _>("amount_due").unwrap_or(0.0),
                "days_overdue": row.try_get::<i64, _>("days_overdue").unwrap_or(0),
                "level": row.try_get::<String, _>("level_code").unwrap_or_default(),
                "level_label": row.try_get::<String, _>("level_label").unwrap_or_default(),
                "threshold_days": row.try_get::<i64, _>("threshold_days").unwrap_or(0),
                "channel_id": row.try_get::<String, _>("channel_id").unwrap_or_default(),
                "channel_name": row.try_get::<Option<String>, _>("channel_name").unwrap_or(None),
                "channel_mode": row.try_get::<String, _>("channel_mode").unwrap_or_default(),
                "status": status,
                "status_label": status_label(&status),
                "attempt_count": row.try_get::<i64, _>("attempt_count").unwrap_or(0),
                "error": row.try_get::<Option<String>, _>("error").unwrap_or(None),
                "created_at": row.try_get::<Option<String>, _>("created_at").unwrap_or(None),
                "last_attempt_at": row.try_get::<Option<String>, _>("last_attempt_at").unwrap_or(None),
                "sent_at": row.try_get::<Option<String>, _>("sent_at").unwrap_or(None),
            })
        })
        .collect();
    Ok(Json(json!({
        "total": history.len(),
        "history": history,
    })))
}

fn settings_payload(settings: CollectionSettings) -> Value {
    json!({
        "enabled": settings.enabled,
        "friendly_days": settings.friendly_days,
        "formal_days": settings.formal_days,
        "final_days": settings.final_days,
        "bad_debt_days": settings.bad_debt_days,
        "levels": [
            {"key": "friendly", "label": "友好提醒", "days": settings.friendly_days},
            {"key": "formal", "label": "正式催款", "days": settings.formal_days},
            {"key": "final", "label": "最后通知", "days": settings.final_days},
            {"key": "bad_debt", "label": "坏账风险", "days": settings.bad_debt_days},
        ],
        "delivery_help": "即时渠道在计费周期当场发送；摘要渠道在每日 09:00 合并发送。",
        "update_action": {
            "method": "PUT",
            "endpoint": "/api/v1/settings/collections",
            "label": "保存催收设置",
            "success_message": "催收自动化设置已保存"
        }
    })
}

async fn can_manage(pool: &SqlitePool, user: &AuthUser) -> Result<bool, ApiError> {
    let role: Option<String> = sqlx::query_scalar(
        "SELECT role FROM company_members WHERE company_id = ? AND user_id = ? LIMIT 1",
    )
    .bind(&user.company_id)
    .bind(&user.user_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "permission check failed"))?;
    Ok(matches!(role.as_deref(), Some("owner" | "admin")))
}

fn status_label(status: &str) -> &'static str {
    match status {
        "pending" => "待发送",
        "dispatching" => "发送中",
        "sent" => "已送达",
        "failed" => "发送失败",
        _ => "未知",
    }
}
