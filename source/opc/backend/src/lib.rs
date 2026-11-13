pub mod api;
pub mod app;
pub mod auth;
pub mod cli;
/// OPC library root.
/// Exposes modules for integration testing.
pub mod db;
pub mod error_log_mw;
pub mod integrations;
pub mod openapi;
pub mod pdf_fonts;
pub mod runtime_config;
pub mod self_update;

use axum::{
    Extension, Router,
    extract::DefaultBodyLimit,
    http::StatusCode,
    middleware,
    response::{IntoResponse, Json},
    routing::get,
};
use sqlx::SqlitePool;
use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
    time::Instant,
};

/// Process start time for `/health` uptime reporting.
static PROCESS_STARTED_AT: LazyLock<Instant> = LazyLock::new(Instant::now);

const HEALTH_INFO: w_web_kit::HealthInfo =
    w_web_kit::HealthInfo::new("w-opc", env!("CARGO_PKG_VERSION"), "unknown");

/// Soft disk warning threshold (still HTTP 200, status=degraded).
const DISK_WARN_BYTES: u64 = 200 * 1024 * 1024;
/// Critical disk threshold (HTTP 503).
const DISK_CRIT_BYTES: u64 = 20 * 1024 * 1024;

/// Install the fleet-approved Ring provider before any reqwest/rustls client
/// is constructed. Installation is process-global and intentionally idempotent.
pub fn install_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

/// Build the full application router (used by main.rs and tests)
pub fn build_app(pool: SqlitePool) -> Router {
    install_crypto_provider();
    let _ = *PROCESS_STARTED_AT;
    let auth_router = auth::router_with_root(pool.clone());

    // Layer order (outer → inner on request): require_auth → error_log → handler.
    // error_log therefore sees AuthUser in extensions when present.
    let api_routes = api::router(pool.clone())
        .layer(middleware::from_fn_with_state(
            pool.clone(),
            error_log_mw::log_server_errors,
        ))
        .layer(middleware::from_fn_with_state(
            pool.clone(),
            auth::middleware::require_auth,
        ));

    // DaaS document-fusion client (Phase 1). `None` when DAAS_ENABLED is off, in
    // which case file uploads behave exactly as before (rows stay `pending`).
    // Shared via Arc and propagated to handlers (e.g. files) as an Extension.
    let daas: Option<std::sync::Arc<crate::integrations::daas::DaasClient>> =
        crate::integrations::daas::DaasConfig::from_env()
            .map(|cfg| std::sync::Arc::new(crate::integrations::daas::DaasClient::new(cfg)));

    // Dual OpenAPI discovery (/openapi.json + /api/openapi.json) via shared helper.
    // Previously /api/openapi.json was exposed by nesting a second /openapi.json under /api.
    // Pool is exposed as Extension so /health can probe without forcing root Router state
    // (nested auth/api routers already own their own SqlitePool state).
    let mut app = Router::new()
        .route("/health", get(health_check))
        .merge(w_web_kit::openapi_routes(openapi::document()))
        .merge(auth_router)
        .nest("/api/v1", api_routes.clone())
        .nest("/api", api_routes)
        .fallback(app::spa::spa_handler)
        .layer(Extension(pool))
        .layer(Extension(daas))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024));

    // Telemetry: shared xradar axum middleware (replaces the former local
    // `radar::track_request` copy). Same spans/paths/trace propagation; layered
    // outermost as before. No-op when W_RADAR_CODE is unset (from_env → None),
    // identical to the prior in-middleware short-circuit.
    if let Some(client) = xradar::XRadarClient::from_env() {
        app = app.layer(middleware::from_fn_with_state(
            client,
            xradar::track_axum_request,
        ));
    }

    app
}

/// Resolve the on-disk data directory used for disk free-space probes.
pub fn data_dir_from_env() -> PathBuf {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:data/database.sqlite?mode=rwc".to_string());
    let db_file = database_url
        .trim_start_matches("sqlite:")
        .split('?')
        .next()
        .unwrap_or("data/database.sqlite");
    Path::new(db_file)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn disk_free_bytes(path: &Path) -> Option<u64> {
    // Prefer an existing path; walk up if the data dir is not created yet.
    let mut probe = path.to_path_buf();
    loop {
        if probe.exists() {
            return fs2::available_space(&probe).ok();
        }
        if !probe.pop() {
            return fs2::available_space(Path::new(".")).ok();
        }
    }
}

/// Real health probe: DB read/write + disk free space.
/// Served at /health and (via the api router + auth whitelist) /api/health.
pub(crate) async fn health_check(Extension(pool): Extension<SqlitePool>) -> impl IntoResponse {
    let uptime_s = PROCESS_STARTED_AT.elapsed().as_secs();
    let data_dir = data_dir_from_env();
    let disk_free = disk_free_bytes(&data_dir);
    let disk_free_mb = disk_free.map(|b| (b / (1024 * 1024)) as i64);

    // DB read probe
    let read_ok = sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(&pool)
        .await
        .is_ok();

    // DB write probe (catches readonly / full disk / lock)
    let write_ok = if read_ok {
        sqlx::query(
            "INSERT INTO app_meta(key, value) VALUES('health_probe', datetime('now')) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
        .execute(&pool)
        .await
        .is_ok()
    } else {
        false
    };

    let db_status = if !read_ok {
        "unreachable"
    } else if !write_ok {
        "readonly"
    } else {
        "ok"
    };

    let mut status = w_web_kit::HealthStatus::Ok;
    if db_status != "ok" {
        status = w_web_kit::HealthStatus::Error;
    } else if let Some(free) = disk_free {
        if free < DISK_CRIT_BYTES {
            status = w_web_kit::HealthStatus::Error;
        } else if free < DISK_WARN_BYTES {
            status = w_web_kit::HealthStatus::Degraded;
            // Soft-warn into error_logs (best-effort, throttled by middleware map key).
            let _ = error_log_mw::record_server_error(
                &pool,
                None,
                None,
                "HEALTH",
                "/health",
                200,
                &format!(
                    "disk free space low: {} MB under {} MB warn threshold",
                    free / (1024 * 1024),
                    DISK_WARN_BYTES / (1024 * 1024)
                ),
            )
            .await;
        }
    }

    let extras = serde_json::json!({
        "db": db_status,
        "disk_free_mb": disk_free_mb,
        "uptime_s": uptime_s,
        "ok": status == w_web_kit::HealthStatus::Ok,
    });

    let body = w_web_kit::health_json_status(&HEALTH_INFO, status, extras);
    // Keep status:"ok" for healthy probes (external monitors key on it).
    // Map Error → 503; Degraded stays 200 with body truth.
    let http = if status == w_web_kit::HealthStatus::Error {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    };
    (http, Json(body))
}
