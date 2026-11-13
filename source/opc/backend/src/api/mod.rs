use axum::Router;
use sqlx::SqlitePool;

mod agent;
mod ai;
mod api_keys;
mod assets;
mod backups;
mod bank;
mod budgets;
mod collections;
pub mod companies;
mod document_parse;
mod document_parser_settings;
mod employees;
pub mod error;
mod files;
mod generic;
mod global_opc_hubs;
mod imports;
pub mod invoice_mail_imports;
mod invoices;
mod kpi;
mod nav;
mod notification_channels;
pub(crate) mod onboarding;
mod operations;
mod policies;
mod reports;
mod workbench;

// Re-export for main.rs auto-catchup
pub use generic::spawn_ai_contract_review as trigger_contract_review;

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        // Health alias (/api/health, /api/v1/health). Exempted from auth by an
        // explicit GET /health whitelist in require_auth; external monitors
        // probe /api/health and previously got a 404 instead of a real probe.
        .route("/health", axum::routing::get(crate::health_check))
        // Auth-protected modules (with company_id based isolation)
        .nest("/assets", assets::router(pool.clone()))
        .nest("/invoices", invoices::router(pool.clone()))
        .nest("/budgets", budgets::router(pool.clone()))
        .nest("/employees", employees::router(pool.clone()))
        .nest("/bank", bank::router(pool.clone()))
        .nest("/companies", companies::router(pool.clone()))
        .merge(collections::router(pool.clone()))
        .merge(nav::router(pool.clone()))
        .nest("/files", files::router(pool.clone()))
        .nest("/agent", agent::router(pool.clone()))
        .merge(agent::mcp_router(pool.clone()))
        .nest("/ai", ai::router(pool.clone()))
        .nest("/workbench", workbench::router(pool.clone()))
        .nest("/imports", imports::router(pool.clone()))
        .merge(invoice_mail_imports::router(pool.clone()))
        .nest("/kpi", kpi::router(pool.clone()))
        .nest("/policies", policies::router(pool.clone()))
        .nest("/global_opc_hubs", global_opc_hubs::router(pool.clone()))
        .nest("/api-keys", api_keys::router(pool.clone()))
        .merge(document_parser_settings::router(pool.clone()))
        .merge(notification_channels::router(pool.clone()))
        .merge(operations::router(pool.clone()))
        .merge(reports::router(pool.clone()))
        .merge(generic::router(pool.clone()))
        .merge(backups::router(pool.clone()))
}
