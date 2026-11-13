//! Instance archives include every tenant and must never be exposed through
//! product-user authorization. Automatic backups remain an operator concern.
use super::error::{ApiError, api_err};
use crate::auth::AuthUser;
use axum::{
    Router,
    http::StatusCode,
    routing::{get, post},
};
use sqlx::SqlitePool;

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route(
            "/settings/backups",
            get(instance_backup_denied).post(instance_backup_denied),
        )
        .route(
            "/settings/backups/{file_name}/download",
            get(instance_backup_denied),
        )
        .route("/settings/restore", post(instance_backup_denied))
        .with_state(pool)
}

async fn instance_backup_denied(_user: AuthUser) -> Result<(), ApiError> {
    Err(api_err(
        StatusCode::FORBIDDEN,
        "Instance backup and restore are unavailable through tenant APIs",
    ))
}
