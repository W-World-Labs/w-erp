use super::error::{ApiError, api_err};
use crate::app::nav_preferences::{self, NavPreferences};
use crate::auth::AuthUser;
use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde_json::Value;
use sqlx::SqlitePool;

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/settings/nav", get(get_nav).put(put_nav))
        .with_state(pool)
}

async fn get_nav(State(pool): State<SqlitePool>, user: AuthUser) -> Result<Json<Value>, ApiError> {
    let (prefs, updated_at) = nav_preferences::load(&pool, &user.user_id, &user.company_id)
        .await
        .map_err(|_| {
            api_err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "fetch nav preferences failed",
            )
        })?;
    Ok(Json(nav_preferences::to_json(
        &prefs,
        updated_at.as_deref(),
    )))
}

async fn put_nav(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    if !body.is_object() {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "nav preferences must be an object",
        ));
    }
    let prefs: NavPreferences = nav_preferences::sanitize(&body);
    let updated_at = nav_preferences::save(&pool, &user.user_id, &user.company_id, &prefs)
        .await
        .map_err(|_| {
            api_err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "save nav preferences failed",
            )
        })?;
    Ok(Json(nav_preferences::to_json(&prefs, Some(&updated_at))))
}
