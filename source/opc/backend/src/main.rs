use axum::http::{HeaderName, Method, header};
use std::sync::Arc;
#[tokio::main]
async fn main() {
    opc::install_crypto_provider();
    xradar::init_tracing("opc");

    // Self-update from the anonymous release.myfastools.com manifest. The repo is
    // private, so the old Gitea-API self-update could never authenticate
    // (unauthenticated release reads 404); the release endpoint is public, so this works
    // with no per-host token. Downloads + atomically replaces the on-disk binary
    // on a newer release (applied on next restart). Disable a host with
    // SELF_UPDATE_AUTO_APPLY=0.
    opc::self_update::spawn(env!("CARGO_PKG_VERSION"));

    dotenvy::dotenv().ok();

    // Issue #3 / #2: before any pool holds the SQLite file, apply staged restore
    // and take a pre-migrate snapshot so a bad migration can roll back.
    let db_path = opc::db::database_file_from_env();
    if let Some(ref path) = db_path {
        if let Err(e) = opc::db::apply_restore_pending(path) {
            tracing::error!("Failed to apply restore-pending: {}", e);
            std::process::exit(1);
        }
        if let Err(e) = opc::db::snapshot_before_migrate(path) {
            tracing::error!("{}", e);
            std::process::exit(1);
        }
    }

    // Initialize database
    let pool = opc::db::init_pool()
        .await
        .expect("Failed to connect to database");

    // Run migrations — on failure roll back to pre-migrate snapshot and exit
    // (systemd Restart=always will loop, but each loop keeps data intact).
    if let Err(e) = opc::db::run_migrations(&pool).await {
        tracing::error!("Migration failed: {}", e);
        pool.close().await;
        if let Some(ref path) = db_path {
            match opc::db::rollback_pre_migrate(path) {
                Ok(()) => tracing::error!(
                    "迁移失败已回滚到迁移前快照: {} (path={})",
                    e,
                    path.display()
                ),
                Err(rb) => tracing::error!("迁移失败且回滚也失败: migration={} rollback={}", e, rb),
            }
        } else {
            tracing::error!("迁移失败（内存库无快照可回滚）: {}", e);
        }
        std::process::exit(1);
    }

    // Run semantic seedings for policy and global OPC hubs
    let _ = opc::db::seed_policies::seed_global_policies(&pool).await;
    let _ = opc::db::seed_opc_hubs::seed_global_opc_hubs(&pool).await;

    // Backfill a demo company for pre-Demo-feature users who never received one.
    // Runs in the background so it never delays the listener binding or health
    // checks (idempotent + cheap pre-check; users with a demo company are skipped).
    {
        let pool = pool.clone();
        tokio::spawn(async move {
            match opc::db::demo_seed::backfill_demo_for_all_users(&pool).await {
                Ok(0) => {}
                Ok(n) => tracing::info!("✅ Backfilled demo company for {} existing user(s)", n),
                Err(e) => tracing::warn!("demo backfill skipped: {}", e),
            }
        });
    }

    opc::app::runtime::spawn_contract_review_catchup(pool.clone());
    opc::app::runtime::spawn_expiry_reminder_scheduler(pool.clone());
    opc::app::runtime::spawn_backup_scheduler(pool.clone(), opc::app::backup::default_data_dir());
    opc::app::runtime::spawn_billing_scheduler(pool.clone());
    opc::app::notifications::spawn_notification_dispatcher(pool.clone());
    opc::app::email_delivery::spawn_reconciliation(pool.clone());
    opc::api::invoice_mail_imports::spawn_scheduler(pool.clone());

    // G-1: credentialed SPA CORS via w-web-kit (ALLOWED_ORIGINS comma-list).
    let cors = w_web_kit::cors::CorsCredentials::from_env(
        "ALLOWED_ORIGINS",
        "http://localhost:8987,http://localhost:5173,http://localhost:8080,http://localhost:3000",
    )
    .methods([
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
    ])
    .headers([
        header::CONTENT_TYPE,
        header::AUTHORIZATION,
        header::ACCEPT,
        header::ORIGIN,
        HeaderName::from_static("x-requested-with"),
    ])
    .build();

    // Build app from lib.rs + add CORS (SPA is embedded at compile time)
    let app = opc::build_app(pool).layer(cors.clone());

    let addr = opc::runtime_config::bind_addr_from_env()
        .expect("BIND_HOST and PORT must produce a valid socket address");
    tracing::info!("🚀 Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let heartbeats = start_heartbeats();
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let mut terminate =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .expect("install SIGTERM handler");
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {},
                _ = terminate.recv() => {},
            }
            for heartbeat in heartbeats {
                heartbeat.abort();
                let _ = heartbeat.await;
            }
        })
        .await
        .unwrap();
}

fn start_heartbeats() -> Vec<tokio::task::JoinHandle<()>> {
    let Some(client) = xradar::XRadarClient::from_env() else {
        tracing::warn!("Radar heartbeat disabled: W_RADAR_CODE missing");
        return Vec::new();
    };
    let project = std::env::var("W_RADAR_PROJECT_ID")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "w-erp".to_owned());
    let mut caps = xradar::Capabilities::new(project);
    if let Some(instance) = std::env::var("W_RADAR_INSTANCE_ID")
        .ok()
        .or_else(|| std::env::var("HOSTNAME").ok())
        .filter(|value| !value.trim().is_empty())
    {
        caps = caps.instance_id(instance);
    }
    let mut tasks = vec![xradar::XRadarClient::spawn_capabilities_heartbeat(
        Arc::new(client),
        Arc::new(caps),
    )];
    match xradar::ArchitectureReporter::from_env() {
        Ok(reporter) => tasks.push(reporter.spawn_heartbeat_with_handler(
            || {
                xradar::ArchitectureObservation::new().node(xradar::ArchitectureNode::new(
                    "backend",
                    "OneMan ERP backend",
                    xradar::ArchitectureNodeKind::Service,
                    xradar::ArchitectureNodeStatus::Healthy,
                ))
            },
            |result| {
                if result.is_err() {
                    tracing::warn!("Radar architecture heartbeat failed; retrying next interval");
                }
            },
        )),
        Err(_) => tracing::warn!(
            "Radar architecture heartbeat disabled: check project ID and architecture key/configuration"
        ),
    }
    tasks
}
