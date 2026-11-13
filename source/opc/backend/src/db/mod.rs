//! Database pool, schema bootstrap, and versioned ALTER migrations.
//!
//! **Schema 唯一来源** = `schema.rs` 的 `INIT_SQL` + 本文件带版本号的 ALTER 列表。
//! 不存在自动执行的 `migrations/` 目录；往仓库里丢 `.sql` 不会被应用。

use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::env;
use std::path::{Path, PathBuf};

pub mod demo_seed;
pub mod models;
pub mod schema;

/// Parse a SQLite file path from DATABASE_URL-style strings.
/// Returns `None` for in-memory URLs (no file to snapshot).
pub fn parse_sqlite_path(database_url: &str) -> Option<PathBuf> {
    if database_url.contains(":memory:") {
        return None;
    }
    let rest = database_url.strip_prefix("sqlite:").unwrap_or(database_url);
    let path = rest.split('?').next().unwrap_or(rest).trim();
    // Accept sqlite:///abs/path and sqlite:rel/path
    let path = path.strip_prefix("//").unwrap_or(path);
    if path.is_empty() || path == ":memory:" {
        return None;
    }
    Some(PathBuf::from(path))
}

/// Resolve the live DATABASE_URL (env or default) to a file path, if any.
pub fn database_file_from_env() -> Option<PathBuf> {
    let url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:data/database.sqlite?mode=rwc".to_string());
    parse_sqlite_path(&url)
}

fn pre_migrate_path(db_path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.pre-migrate", db_path.display()))
}

/// Copy the SQLite main file (+ WAL/SHM if present) to `<db>.pre-migrate`.
/// No-op when the DB file does not yet exist (fresh install).
/// Fails hard on copy errors so we never migrate without a rollback point.
pub fn snapshot_before_migrate(db_path: &Path) -> Result<(), String> {
    if !db_path.exists() {
        return Ok(());
    }
    let dest = pre_migrate_path(db_path);
    std::fs::copy(db_path, &dest).map_err(|e| {
        format!(
            "迁移前快照失败（无法复制 {} → {}）: {} — 请检查磁盘空间后重试",
            db_path.display(),
            dest.display(),
            e
        )
    })?;

    for suffix in ["-wal", "-shm"] {
        let src = PathBuf::from(format!("{}{}", db_path.display(), suffix));
        if src.exists() {
            let side_dest = PathBuf::from(format!("{}{}", dest.display(), suffix));
            std::fs::copy(&src, &side_dest).map_err(|e| {
                format!(
                    "迁移前快照失败（无法复制 {}）: {} — 请检查磁盘空间后重试",
                    src.display(),
                    e
                )
            })?;
        } else {
            // Remove stale side files from a previous snapshot so rollback
            // does not re-attach an outdated WAL.
            let side_dest = PathBuf::from(format!("{}{}", dest.display(), suffix));
            let _ = std::fs::remove_file(side_dest);
        }
    }

    tracing::info!("📦 Pre-migrate snapshot written to {}", dest.display());
    Ok(())
}

/// Restore `<db>.pre-migrate` over the live DB file (and WAL/SHM).
pub fn rollback_pre_migrate(db_path: &Path) -> Result<(), String> {
    let snap = pre_migrate_path(db_path);
    if !snap.exists() {
        return Err(format!("无法回滚：找不到迁移前快照 {}", snap.display()));
    }

    // Drop live side files first so SQLite does not reopen a mixed pair.
    for suffix in ["-wal", "-shm"] {
        let live = PathBuf::from(format!("{}{}", db_path.display(), suffix));
        let _ = std::fs::remove_file(&live);
    }

    std::fs::copy(&snap, db_path)
        .map_err(|e| format!("回滚失败（无法恢复 {}）: {}", db_path.display(), e))?;

    for suffix in ["-wal", "-shm"] {
        let side_snap = PathBuf::from(format!("{}{}", snap.display(), suffix));
        let live = PathBuf::from(format!("{}{}", db_path.display(), suffix));
        if side_snap.exists() {
            std::fs::copy(&side_snap, &live)
                .map_err(|e| format!("回滚失败（无法恢复 {}）: {}", live.display(), e))?;
        }
    }

    Ok(())
}

/// Apply a pending restore written by the backup subsystem (issue #2).
/// Call **before** opening the connection pool.
pub fn apply_restore_pending(db_path: &Path) -> Result<(), String> {
    let pending = PathBuf::from(format!("{}.restore-pending", db_path.display()));
    if !pending.exists() {
        return Ok(());
    }

    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
    if db_path.exists() {
        let pre = PathBuf::from(format!("{}.pre-restore-{}", db_path.display(), ts));
        std::fs::copy(db_path, &pre)
            .map_err(|e| format!("恢复失败：无法保留当前库副本 {}: {}", pre.display(), e))?;
        tracing::info!("🔒 Current DB saved as {}", pre.display());
    }

    for suffix in ["-wal", "-shm"] {
        let live = PathBuf::from(format!("{}{}", db_path.display(), suffix));
        let _ = std::fs::remove_file(&live);
    }

    std::fs::rename(&pending, db_path)
        .map_err(|e| format!("恢复失败：无法应用 {}: {}", pending.display(), e))?;
    tracing::info!("✅ Applied restore-pending → {}", db_path.display());

    // Optional staged uploads directory next to the DB's data root.
    if let Some(data_dir) = db_path.parent() {
        let uploads_pending = data_dir.join("uploads.restore-pending");
        if uploads_pending.exists() {
            let uploads = data_dir.join("uploads");
            if uploads.exists() {
                let backup_uploads = data_dir.join(format!("uploads.pre-restore-{}", ts));
                let _ = std::fs::rename(&uploads, &backup_uploads);
            }
            std::fs::rename(&uploads_pending, &uploads)
                .map_err(|e| format!("恢复失败：无法应用 uploads.restore-pending: {}", e))?;
            tracing::info!("✅ Applied uploads.restore-pending");
        }
    }

    Ok(())
}

pub async fn init_pool() -> Result<SqlitePool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:data/database.sqlite?mode=rwc".to_string());

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(schema::INIT_SQL).execute(pool).await?;

    run_alter_migrations(pool).await?;

    post_migration_self_check(pool).await?;

    tracing::info!("✅ Database migrations completed");
    Ok(())
}

/// Only these SQLite errors are treated as "already applied" for legacy DBs
/// that pre-date the `schema_migrations` table.
fn is_tolerable_schema_error(msg: &str) -> bool {
    let lower = msg.to_lowercase();
    lower.contains("duplicate column")
        || lower.contains("duplicate column name")
        || lower.contains("already exists")
}

async fn migration_already_applied(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT 1 FROM schema_migrations WHERE id = ? LIMIT 1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
    Ok(row.is_some())
}

async fn mark_migration_applied(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR IGNORE INTO schema_migrations (id, applied_at) VALUES (?, datetime('now'))",
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Apply a single versioned schema statement. Skips if `id` is already recorded.
/// Tolerates duplicate-column / already-exists on first upgrade of legacy DBs
/// (records the id and continues). Any other error propagates as `Err`.
async fn apply_versioned_statement(
    pool: &SqlitePool,
    id: &str,
    sql: &str,
) -> Result<(), sqlx::Error> {
    if migration_already_applied(pool, id).await? {
        return Ok(());
    }

    match sqlx::query(sql).execute(pool).await {
        Ok(_) => {
            mark_migration_applied(pool, id).await?;
            Ok(())
        }
        Err(e) => {
            let msg = e.to_string();
            if is_tolerable_schema_error(&msg) {
                tracing::info!(
                    "Migration {} already reflected in schema (tolerated): {}",
                    id,
                    msg
                );
                mark_migration_applied(pool, id).await?;
                Ok(())
            } else {
                tracing::error!("Migration {} failed: {} — {}", id, sql, msg);
                Err(e)
            }
        }
    }
}

/// Versioned ALTER / index statements for existing databases.
async fn run_alter_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // (id, sql) — ids are stable; do not renumber historically applied ones.
    let alter_statements: &[(&str, &str)] = &[
        (
            "2026-10-email-provider",
            "ALTER TABLE email_configurations ADD COLUMN provider TEXT NOT NULL DEFAULT 'smtp' CHECK(provider IN ('smtp','w-mail'))",
        ),
        (
            "2026-10-email-delivery-metadata",
            "ALTER TABLE email_logs ADD COLUMN mail_delivery TEXT",
        ),
        (
            "2026-10-email-delivery-due",
            "CREATE INDEX IF NOT EXISTS idx_email_logs_mail_due ON email_logs(json_extract(mail_delivery,'$.nextCheck')) WHERE mail_delivery IS NOT NULL",
        ),
        // Companies soft-delete support
        (
            "2026-companies-is-deleted",
            "ALTER TABLE companies ADD COLUMN is_deleted INTEGER DEFAULT 0",
        ),
        (
            "2026-companies-deleted-at",
            "ALTER TABLE companies ADD COLUMN deleted_at TEXT",
        ),
        // Companies i18n columns
        (
            "2026-companies-country",
            "ALTER TABLE companies ADD COLUMN country TEXT DEFAULT 'CN'",
        ),
        (
            "2026-companies-timezone",
            "ALTER TABLE companies ADD COLUMN timezone TEXT DEFAULT 'Asia/Shanghai'",
        ),
        (
            "2026-companies-locale",
            "ALTER TABLE companies ADD COLUMN locale TEXT DEFAULT 'zh'",
        ),
        (
            "2026-companies-base-currency",
            "ALTER TABLE companies ADD COLUMN base_currency TEXT DEFAULT 'CNY'",
        ),
        (
            "2026-companies-date-format",
            "ALTER TABLE companies ADD COLUMN date_format TEXT DEFAULT 'YYYY-MM-DD'",
        ),
        (
            "2026-companies-is-demo",
            "ALTER TABLE companies ADD COLUMN is_demo INTEGER DEFAULT 0",
        ),
        // Asset depreciation fields
        (
            "2026-assets-useful-life",
            "ALTER TABLE assets ADD COLUMN useful_life INTEGER",
        ),
        (
            "2026-assets-salvage-rate",
            "ALTER TABLE assets ADD COLUMN salvage_rate REAL DEFAULT 0.05",
        ),
        (
            "2026-assets-depreciation-method",
            "ALTER TABLE assets ADD COLUMN depreciation_method TEXT DEFAULT 'straight_line'",
        ),
        (
            "2026-assets-monthly-depreciation",
            "ALTER TABLE assets ADD COLUMN monthly_depreciation REAL",
        ),
        (
            "2026-assets-accumulated-depreciation",
            "ALTER TABLE assets ADD COLUMN accumulated_depreciation REAL DEFAULT 0",
        ),
        (
            "2026-asset-records-updated-at",
            "ALTER TABLE asset_records ADD COLUMN updated_at TEXT DEFAULT CURRENT_TIMESTAMP",
        ),
        // File attachment metadata
        (
            "2026-files-description",
            "ALTER TABLE files ADD COLUMN description TEXT",
        ),
        // DaaS document-fusion mirror reference
        (
            "2026-files-daas-workspace-id",
            "ALTER TABLE files ADD COLUMN daas_workspace_id TEXT",
        ),
        (
            "2026-files-daas-document-asset-id",
            "ALTER TABLE files ADD COLUMN daas_document_asset_id TEXT",
        ),
        (
            "2026-files-daas-object-key",
            "ALTER TABLE files ADD COLUMN daas_object_key TEXT",
        ),
        (
            "2026-files-daas-sync-status",
            "ALTER TABLE files ADD COLUMN daas_sync_status TEXT DEFAULT 'pending'",
        ),
        (
            "2026-files-daas-synced-at",
            "ALTER TABLE files ADD COLUMN daas_synced_at TEXT",
        ),
        (
            "2026-files-daas-sync-error",
            "ALTER TABLE files ADD COLUMN daas_sync_error TEXT",
        ),
        // Invoice data-axis: currency + client FK
        (
            "2026-07-invoices-currency",
            "ALTER TABLE invoices ADD COLUMN currency TEXT",
        ),
        (
            "2026-07-invoices-client-id",
            "ALTER TABLE invoices ADD COLUMN client_id TEXT",
        ),
        (
            "2026-reminders-notified-at",
            "ALTER TABLE reminders ADD COLUMN notified_at TEXT",
        ),
    ];

    for (id, stmt) in alter_statements {
        apply_versioned_statement(pool, id, stmt).await?;
    }

    let post_alter_statements: &[(&str, &str)] = &[
        (
            "2026-idx-files-daas-sync-status",
            "CREATE INDEX IF NOT EXISTS idx_files_daas_sync_status ON files(daas_sync_status)",
        ),
        (
            "2026-idx-files-daas-document-asset",
            "CREATE INDEX IF NOT EXISTS idx_files_daas_document_asset ON files(daas_document_asset_id)",
        ),
        (
            "2026-idx-invoices-client",
            "CREATE INDEX IF NOT EXISTS idx_invoices_client ON invoices(client_id)",
        ),
        (
            "2026-idx-invoices-type",
            "CREATE INDEX IF NOT EXISTS idx_invoices_type ON invoices(invoice_type)",
        ),
        // Health write-probe KV (also in INIT_SQL; keep for DBs that predate the table)
        (
            "2026-app-meta",
            "CREATE TABLE IF NOT EXISTS app_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        ),
        (
            "2026-document-sequences",
            "CREATE TABLE IF NOT EXISTS document_sequences (company_id TEXT NOT NULL, doc_type TEXT NOT NULL, year INTEGER NOT NULL, next_seq INTEGER NOT NULL DEFAULT 1, PRIMARY KEY (company_id, doc_type, year))",
        ),
        (
            "2026-scheduler-runs",
            "CREATE TABLE IF NOT EXISTS scheduler_runs (id TEXT PRIMARY KEY, scheduler_name TEXT NOT NULL, last_run_at TEXT NOT NULL, generated_count INTEGER DEFAULT 0, details TEXT, created_at TEXT DEFAULT CURRENT_TIMESTAMP)",
        ),
        (
            "2026-runtime-settings",
            "CREATE TABLE IF NOT EXISTS runtime_settings (key TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at TEXT DEFAULT CURRENT_TIMESTAMP)",
        ),
        (
            "2026-notification-channels",
            "CREATE TABLE IF NOT EXISTS notification_channels (id TEXT PRIMARY KEY, company_id TEXT NOT NULL, name TEXT, channel_url TEXT NOT NULL, mode TEXT DEFAULT 'digest', enabled INTEGER DEFAULT 1, created_at TEXT DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (company_id) REFERENCES companies(id))",
        ),
        (
            "2026-idx-notification-channels-company",
            "CREATE INDEX IF NOT EXISTS idx_notification_channels_company ON notification_channels(company_id)",
        ),
        (
            "2026-invoice-mailboxes",
            "CREATE TABLE IF NOT EXISTS invoice_mailboxes (id TEXT PRIMARY KEY, company_id TEXT NOT NULL, name TEXT NOT NULL, imap_host TEXT NOT NULL, imap_port INTEGER NOT NULL DEFAULT 993, imap_username TEXT NOT NULL, imap_password TEXT NOT NULL, folder TEXT NOT NULL DEFAULT 'INBOX', enabled INTEGER NOT NULL DEFAULT 0, last_uid INTEGER NOT NULL DEFAULT 0, last_sync_at TEXT, last_sync_status TEXT, last_sync_error TEXT, created_at TEXT DEFAULT CURRENT_TIMESTAMP, updated_at TEXT DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (company_id) REFERENCES companies(id))",
        ),
        (
            "2026-idx-invoice-mailboxes-company",
            "CREATE INDEX IF NOT EXISTS idx_invoice_mailboxes_company ON invoice_mailboxes(company_id, enabled)",
        ),
        (
            "2026-invoice-mail-imports",
            "CREATE TABLE IF NOT EXISTS invoice_mail_imports (id TEXT PRIMARY KEY, company_id TEXT NOT NULL, mailbox_id TEXT NOT NULL, message_uid INTEGER NOT NULL, message_id TEXT, subject TEXT, attachment_index INTEGER NOT NULL, attachment_name TEXT NOT NULL, content_sha256 TEXT NOT NULL, xml_content BLOB NOT NULL, invoice_number TEXT, parsed_json TEXT, status TEXT NOT NULL DEFAULT 'pending_confirmation' CHECK(status IN ('pending_confirmation', 'imported', 'duplicate', 'failed', 'rejected')), error TEXT, duplicate_of TEXT, invoice_id TEXT, confirmed_by TEXT, confirmed_at TEXT, created_at TEXT DEFAULT CURRENT_TIMESTAMP, updated_at TEXT DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (company_id) REFERENCES companies(id), FOREIGN KEY (mailbox_id) REFERENCES invoice_mailboxes(id), FOREIGN KEY (invoice_id) REFERENCES invoices(id), UNIQUE(company_id, mailbox_id, message_uid, attachment_index))",
        ),
        (
            "2026-idx-invoice-mail-imports-queue",
            "CREATE INDEX IF NOT EXISTS idx_invoice_mail_imports_queue ON invoice_mail_imports(company_id, status, created_at)",
        ),
        (
            "2026-idx-invoice-mail-imports-hash",
            "CREATE INDEX IF NOT EXISTS idx_invoice_mail_imports_hash ON invoice_mail_imports(company_id, content_sha256)",
        ),
        (
            "2026-collection-settings",
            "CREATE TABLE IF NOT EXISTS collection_settings (company_id TEXT PRIMARY KEY, enabled INTEGER NOT NULL DEFAULT 1, friendly_days INTEGER NOT NULL DEFAULT 1, formal_days INTEGER NOT NULL DEFAULT 7, final_days INTEGER NOT NULL DEFAULT 30, bad_debt_days INTEGER NOT NULL DEFAULT 60, updated_at TEXT DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (company_id) REFERENCES companies(id))",
        ),
        (
            "2026-collection-deliveries",
            "CREATE TABLE IF NOT EXISTS collection_deliveries (id TEXT PRIMARY KEY, company_id TEXT NOT NULL, invoice_id TEXT NOT NULL, client_id TEXT, invoice_number TEXT NOT NULL, client_name TEXT, amount_due REAL NOT NULL DEFAULT 0, days_overdue INTEGER NOT NULL, level_code TEXT NOT NULL, level_label TEXT NOT NULL, threshold_days INTEGER NOT NULL, channel_id TEXT NOT NULL, channel_name TEXT, channel_mode TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'pending', attempt_count INTEGER NOT NULL DEFAULT 0, error TEXT, audit_user_id TEXT, last_attempt_at TEXT, sent_at TEXT, created_at TEXT DEFAULT CURRENT_TIMESTAMP, updated_at TEXT DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (company_id) REFERENCES companies(id), FOREIGN KEY (invoice_id) REFERENCES invoices(id), UNIQUE(company_id, invoice_id, level_code, channel_id))",
        ),
        (
            "2026-idx-collection-deliveries-company-created",
            "CREATE INDEX IF NOT EXISTS idx_collection_deliveries_company_created ON collection_deliveries(company_id, created_at DESC)",
        ),
        (
            "2026-idx-collection-deliveries-invoice",
            "CREATE INDEX IF NOT EXISTS idx_collection_deliveries_invoice ON collection_deliveries(company_id, invoice_id, threshold_days DESC)",
        ),
        (
            "2026-idx-collection-deliveries-dispatch",
            "CREATE INDEX IF NOT EXISTS idx_collection_deliveries_dispatch ON collection_deliveries(channel_id, channel_mode, status)",
        ),
        (
            "2026-user-nav-preferences",
            "CREATE TABLE IF NOT EXISTS user_nav_preferences (user_id TEXT NOT NULL, company_id TEXT NOT NULL, preferences TEXT NOT NULL, updated_at TEXT DEFAULT CURRENT_TIMESTAMP, PRIMARY KEY (user_id, company_id), FOREIGN KEY (company_id) REFERENCES companies(id))",
        ),
    ];

    for (id, stmt) in post_alter_statements {
        apply_versioned_statement(pool, id, stmt).await?;
    }

    // Deduplicate document numbers before unique indexes (idempotent).
    dedupe_document_numbers(pool, "invoices", "invoice_number").await;
    dedupe_document_numbers(pool, "quotes", "quote_number").await;

    let unique_indexes = [
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_invoices_company_number ON invoices(company_id, invoice_number)",
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_quotes_company_number ON quotes(company_id, quote_number)",
    ];
    for stmt in &unique_indexes {
        if let Err(e) = sqlx::query(stmt).execute(pool).await {
            tracing::warn!("Migration warning: {} — {}", stmt, e);
        }
    }

    // Data backfills — idempotent UPDATEs; warn-only on failure (not schema).
    if let Err(e) = sqlx::query(
        "UPDATE companies SET is_demo = 1 WHERE unified_social_credit_code = ? AND COALESCE(is_demo, 0) = 0",
    )
    .bind(demo_seed::DEMO_COMPANY_USCC)
    .execute(pool)
    .await
    {
        tracing::warn!("Migration warning: backfill is_demo flag — {}", e);
    }

    let invoice_backfills = [
        "UPDATE invoices SET invoice_type = 'receivable' WHERE LOWER(COALESCE(invoice_type, '')) IN ('output', 'income', 'receivable', '销项')",
        "UPDATE invoices SET invoice_type = 'payable' WHERE LOWER(COALESCE(invoice_type, '')) IN ('input', 'expense', 'payable', '进项')",
        "UPDATE invoices SET currency = 'CNY' WHERE currency IS NULL OR TRIM(currency) = ''",
        "UPDATE invoices SET client_id = (
            SELECT c.id FROM clients c
            WHERE c.company_id = invoices.company_id
              AND c.name = invoices.supplier
            LIMIT 1
        ) WHERE client_id IS NULL
          AND invoice_type = 'receivable'
          AND supplier IS NOT NULL
          AND TRIM(supplier) != ''",
    ];
    for stmt in &invoice_backfills {
        if let Err(e) = sqlx::query(stmt).execute(pool).await {
            tracing::warn!("Migration warning: invoice backfill — {} — {}", stmt, e);
        }
    }

    migrate_asset_type_check(pool).await?;
    Ok(())
}

/// Append -2/-3 suffixes to duplicate document numbers within a company so a
/// UNIQUE index can be created. Idempotent: no-ops when no duplicates remain.
async fn dedupe_document_numbers(pool: &SqlitePool, table: &str, column: &str) {
    // Safety: table/column are hardcoded callers only.
    let sql = format!(
        r#"
        SELECT company_id, {column} AS num, COUNT(*) AS cnt
        FROM {table}
        WHERE {column} IS NOT NULL AND TRIM({column}) != ''
        GROUP BY company_id, {column}
        HAVING cnt > 1
        "#
    );
    let dups: Vec<(String, String, i64)> = match sqlx::query_as(&sql).fetch_all(pool).await {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!("dedupe {table}.{column} scan failed: {e}");
            return;
        }
    };

    for (company_id, num, _) in dups {
        let ids_sql = format!(
            "SELECT id FROM {table} WHERE company_id = ? AND {column} = ? ORDER BY created_at ASC, id ASC"
        );
        let ids: Vec<(String,)> = match sqlx::query_as(&ids_sql)
            .bind(&company_id)
            .bind(&num)
            .fetch_all(pool)
            .await
        {
            Ok(rows) => rows,
            Err(_) => continue,
        };
        // Keep the first row as-is; suffix subsequent ones.
        for (idx, (id,)) in ids.into_iter().enumerate().skip(1) {
            let mut suffix = 2 + (idx as i64 - 1);
            loop {
                let candidate = format!("{num}-{suffix}");
                let exists_sql =
                    format!("SELECT COUNT(*) FROM {table} WHERE company_id = ? AND {column} = ?");
                let exists: i64 = sqlx::query_scalar(&exists_sql)
                    .bind(&company_id)
                    .bind(&candidate)
                    .fetch_one(pool)
                    .await
                    .unwrap_or(0);
                if exists == 0 {
                    let upd =
                        format!("UPDATE {table} SET {column} = ? WHERE id = ? AND company_id = ?");
                    let _ = sqlx::query(&upd)
                        .bind(&candidate)
                        .bind(&id)
                        .bind(&company_id)
                        .execute(pool)
                        .await;
                    break;
                }
                suffix += 1;
                if suffix > 10_000 {
                    break;
                }
            }
        }
    }
}

/// SQLite cannot ALTER CHECK constraints — must recreate the table.
/// Protected by the pre-migrate snapshot; failures return Err (no silent half-state swallow).
async fn migrate_asset_type_check(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let id = "2026-assets-check-data-intangible";
    if migration_already_applied(pool, id).await? {
        return Ok(());
    }

    let row: Option<(String,)> =
        sqlx::query_as("SELECT sql FROM sqlite_master WHERE type='table' AND name='assets'")
            .fetch_optional(pool)
            .await?;

    let Some((table_sql,)) = row else {
        // Fresh DB without assets table yet — nothing to do; mark applied so we
        // do not re-check forever (INIT_SQL creates assets on next full bootstrap).
        return Ok(());
    };

    // Already migrated?
    if table_sql.contains("'data'") && table_sql.contains("'intangible'") {
        mark_migration_applied(pool, id).await?;
        return Ok(());
    }

    tracing::info!("Migrating assets CHECK constraint...");

    let columns: Vec<(String,)> = sqlx::query_as("SELECT name FROM pragma_table_info('assets')")
        .fetch_all(pool)
        .await?;

    if columns.is_empty() {
        return Err(sqlx::Error::Protocol(
            "Cannot read assets table columns for CHECK migration".into(),
        ));
    }

    let col_list = columns
        .iter()
        .map(|(n,)| n.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    let new_table_sql = table_sql
        .replacen("CREATE TABLE assets", "CREATE TABLE assets_new", 1)
        .replace(
            "CHECK(asset_type IN ('physical', 'virtual'))",
            "CHECK(asset_type IN ('physical', 'virtual', 'data', 'intangible'))",
        );

    let steps: Vec<String> = vec![
        "PRAGMA foreign_keys=OFF".to_string(),
        "DROP TABLE IF EXISTS assets_new".to_string(),
        new_table_sql,
        format!(
            "INSERT INTO assets_new ({c}) SELECT {c} FROM assets",
            c = col_list
        ),
        "DROP TABLE assets".to_string(),
        "ALTER TABLE assets_new RENAME TO assets".to_string(),
        "CREATE INDEX IF NOT EXISTS idx_assets_company ON assets(company_id)".to_string(),
        "PRAGMA foreign_keys=ON".to_string(),
    ];

    for step in &steps {
        if let Err(e) = sqlx::query(step).execute(pool).await {
            let _ = sqlx::query("PRAGMA foreign_keys=ON").execute(pool).await;
            tracing::error!(
                "Asset CHECK migration failed: {}... — {}",
                step.chars().take(80).collect::<String>(),
                e
            );
            return Err(e);
        }
    }

    mark_migration_applied(pool, id).await?;
    tracing::info!("✅ assets CHECK constraint migration completed");
    Ok(())
}

/// After successful migrations: quick_check + key tables must be readable.
async fn post_migration_self_check(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let check: (String,) = sqlx::query_as("PRAGMA quick_check").fetch_one(pool).await?;
    if !check.0.eq_ignore_ascii_case("ok") {
        return Err(sqlx::Error::Protocol(format!(
            "PRAGMA quick_check failed: {}",
            check.0
        )));
    }

    for table in ["invoices", "payments", "quotes", "clients"] {
        sqlx::query(&format!("SELECT 1 FROM {} LIMIT 1", table))
            .execute(pool)
            .await
            .map_err(|e| {
                tracing::error!(
                    "Post-migration self-check failed for table {}: {}",
                    table,
                    e
                );
                e
            })?;
    }
    Ok(())
}

pub mod seed_opc_hubs;
pub mod seed_policies;

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use std::io::Write;

    async fn memory_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.expect("migrations");
        pool
    }

    #[tokio::test]
    async fn run_migrations_handles_existing_files_table_without_daas_columns() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        sqlx::query(
            r#"
            CREATE TABLE files (
                id TEXT PRIMARY KEY,
                company_id TEXT NOT NULL,
                filename TEXT NOT NULL,
                original_name TEXT NOT NULL,
                content_type TEXT DEFAULT 'application/octet-stream',
                size INTEGER DEFAULT 0,
                entity_type TEXT,
                entity_id TEXT,
                description TEXT,
                uploaded_by TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        run_migrations(&pool).await.unwrap();

        let columns: Vec<(String,)> = sqlx::query_as("SELECT name FROM pragma_table_info('files')")
            .fetch_all(&pool)
            .await
            .unwrap();
        let column_names = columns.into_iter().map(|(name,)| name).collect::<Vec<_>>();

        assert!(column_names.contains(&"daas_sync_status".to_string()));
        assert!(column_names.contains(&"daas_document_asset_id".to_string()));

        sqlx::query("SELECT id FROM files WHERE daas_sync_status = 'pending'")
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn bad_alter_returns_err_not_panic() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        // Minimal schema_migrations so apply_versioned_statement can record.
        sqlx::query(
            "CREATE TABLE schema_migrations (id TEXT PRIMARY KEY, applied_at TEXT DEFAULT CURRENT_TIMESTAMP)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let err = apply_versioned_statement(
            &pool,
            "test-bad-alter-nonexistent",
            "ALTER TABLE definitely_does_not_exist ADD COLUMN x TEXT",
        )
        .await;
        assert!(err.is_err(), "nonexistent-table ALTER must return Err");
    }

    #[tokio::test]
    async fn versioned_migration_is_idempotent() {
        let pool = memory_pool().await;

        // Second full run must not fail and must not duplicate version rows.
        run_migrations(&pool).await.expect("second migration run");

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM schema_migrations WHERE id = '2026-07-invoices-currency'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count.0, 1);

        // Re-apply same id directly — should no-op.
        apply_versioned_statement(
            &pool,
            "2026-07-invoices-currency",
            "ALTER TABLE invoices ADD COLUMN currency TEXT",
        )
        .await
        .expect("idempotent re-apply");

        let count2: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM schema_migrations WHERE id = '2026-07-invoices-currency'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count2.0, 1);
    }

    #[test]
    fn parse_sqlite_path_handles_common_forms() {
        assert_eq!(
            parse_sqlite_path("sqlite:data/database.sqlite?mode=rwc"),
            Some(PathBuf::from("data/database.sqlite"))
        );
        assert_eq!(
            parse_sqlite_path("sqlite:///tmp/erp.db"),
            Some(PathBuf::from("/tmp/erp.db"))
        );
        assert!(parse_sqlite_path("sqlite::memory:").is_none());
    }

    #[test]
    fn snapshot_and_rollback_roundtrip() {
        let dir = std::env::temp_dir().join(format!("opc-migrate-snap-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = dir.join("database.sqlite");
        {
            let mut f = std::fs::File::create(&db).unwrap();
            f.write_all(b"ORIGINAL-DB-BYTES").unwrap();
        }

        snapshot_before_migrate(&db).expect("snapshot");
        // Corrupt live file
        std::fs::write(&db, b"CORRUPTED").unwrap();
        assert_eq!(std::fs::read(&db).unwrap(), b"CORRUPTED");

        rollback_pre_migrate(&db).expect("rollback");
        assert_eq!(std::fs::read(&db).unwrap(), b"ORIGINAL-DB-BYTES");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn is_tolerable_only_duplicate_or_already_exists() {
        assert!(is_tolerable_schema_error("duplicate column name: currency"));
        assert!(is_tolerable_schema_error("table already exists"));
        assert!(!is_tolerable_schema_error("no such table: foo"));
        assert!(!is_tolerable_schema_error("disk I/O error"));
    }
}
