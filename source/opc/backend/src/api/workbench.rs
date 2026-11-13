use axum::{Router, extract::State, http::StatusCode, response::Json, routing::get};
use sqlx::SqlitePool;

use crate::{app::workbench, auth::AuthUser};

use super::error::{ApiError, api_err};

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/summary", get(summary))
        .with_state(pool)
}

async fn summary(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<workbench::WorkbenchSummary>, ApiError> {
    workbench::build_summary(&pool, &user.company_id)
        .await
        .map(Json)
        .map_err(|err| {
            tracing::error!("workbench summary failed: {:?}", err);
            api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error")
        })
}
