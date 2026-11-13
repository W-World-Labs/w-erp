//! Built-in backup / restore of SQLite + uploads (issue #2).
//!
//! Artifacts land in `{data_dir}/backups/backup-YYYYMMDD-HHMMSS.tar.gz` and
//! contain a consistent SQLite snapshot (`VACUUM INTO`) plus the `uploads/` tree.

use chrono::{Datelike, Local, NaiveDate};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::Serialize;
use sqlx::SqlitePool;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};
use uuid::Uuid;

const BACKUP_PREFIX: &str = "backup-";
const BACKUP_SUFFIX: &str = ".tar.gz";

#[derive(Debug, Clone, Serialize)]
pub struct BackupMeta {
    pub id: Option<String>,
    pub file_name: String,
    pub size_bytes: u64,
    pub status: String,
    pub created_at: String,
}

/// Default data directory (same layout as files.rs / docker-compose).
pub fn default_data_dir() -> PathBuf {
    PathBuf::from(std::env::var("OPC_DATA_DIR").unwrap_or_else(|_| "data".to_string()))
}

pub fn backups_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("backups")
}

/// Validate a backup file name: must be `backup-<safechars>.tar.gz` with no path separators.
pub fn validate_backup_file_name(file_name: &str) -> Result<(), String> {
    if file_name.is_empty()
        || file_name.contains("..")
        || file_name.contains('/')
        || file_name.contains('\\')
        || file_name.contains('\0')
    {
        return Err("invalid backup file name".into());
    }
    if !file_name.starts_with(BACKUP_PREFIX) || !file_name.ends_with(BACKUP_SUFFIX) {
        return Err("backup file name must match backup-*.tar.gz".into());
    }
    let mid = &file_name[BACKUP_PREFIX.len()..file_name.len() - BACKUP_SUFFIX.len()];
    if mid.is_empty()
        || !mid
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err("backup file name contains illegal characters".into());
    }
    Ok(())
}

fn sqlite_path_for_data_dir(data_dir: &Path) -> PathBuf {
    // Prefer the live DATABASE_URL when it is absolute (production); otherwise
    // stage restore next to the caller's data_dir (default `data/database.sqlite`).
    if let Some(p) = crate::db::database_file_from_env() {
        if p.is_absolute() {
            return p;
        }
    }
    data_dir.join("database.sqlite")
}

/// Create a consistent backup archive and record it in `backup_records`.
pub async fn create_backup(pool: &SqlitePool, data_dir: &Path) -> Result<BackupMeta, String> {
    let backups = backups_dir(data_dir);
    fs::create_dir_all(&backups).map_err(|e| format!("create backups dir: {e}"))?;

    let stamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let file_name = format!("{BACKUP_PREFIX}{stamp}{BACKUP_SUFFIX}");
    let archive_path = backups.join(&file_name);
    let tmp_sqlite = backups.join(format!("tmp-{}.sqlite", Uuid::new_v4()));

    // Consistent online snapshot via VACUUM INTO (absolute path required).
    let tmp_abs = tmp_sqlite
        .canonicalize()
        .or_else(|_| {
            // File does not exist yet — canonicalize parent + join name.
            let parent = tmp_sqlite
                .parent()
                .ok_or_else(|| std::io::Error::other("no parent"))?;
            fs::create_dir_all(parent)?;
            let parent_abs = parent.canonicalize()?;
            Ok::<_, std::io::Error>(parent_abs.join(tmp_sqlite.file_name().unwrap()))
        })
        .map_err(|e| format!("resolve tmp sqlite path: {e}"))?;

    // Remove if a previous attempt left it behind (VACUUM INTO refuses existing files).
    let _ = fs::remove_file(&tmp_abs);

    sqlx::query("VACUUM INTO ?1")
        .bind(tmp_abs.to_string_lossy().as_ref())
        .execute(pool)
        .await
        .map_err(|e| format!("VACUUM INTO failed: {e}"))?;

    // Pack snapshot + uploads into tar.gz
    {
        let file = File::create(&archive_path).map_err(|e| format!("create archive: {e}"))?;
        let enc = GzEncoder::new(file, Compression::default());
        let mut tar = Builder::new(enc);

        tar.append_path_with_name(&tmp_abs, "database.sqlite")
            .map_err(|e| format!("tar database: {e}"))?;

        let uploads = data_dir.join("uploads");
        if uploads.is_dir() {
            tar.append_dir_all("uploads", &uploads)
                .map_err(|e| format!("tar uploads: {e}"))?;
        }

        let enc = tar.into_inner().map_err(|e| format!("tar finish: {e}"))?;
        enc.finish().map_err(|e| format!("gzip finish: {e}"))?;
    }

    let _ = fs::remove_file(&tmp_abs);

    let size_bytes = fs::metadata(&archive_path).map(|m| m.len()).unwrap_or(0);
    let created_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO backup_records (id, file_name, size_bytes, status, created_at) \
         VALUES (?, ?, ?, 'success', ?)",
    )
    .bind(&id)
    .bind(&file_name)
    .bind(size_bytes as i64)
    .bind(&created_at)
    .execute(pool)
    .await
    .map_err(|e| format!("insert backup_records: {e}"))?;

    tracing::info!("💾 Backup created: {} ({} bytes)", file_name, size_bytes);

    Ok(BackupMeta {
        id: Some(id),
        file_name,
        size_bytes,
        status: "success".into(),
        created_at,
    })
}

/// List backups from disk, enriched with DB records when present.
pub async fn list_backups(pool: &SqlitePool, data_dir: &Path) -> Result<Vec<BackupMeta>, String> {
    let backups = backups_dir(data_dir);
    let mut out: Vec<BackupMeta> = Vec::new();

    if backups.is_dir() {
        let mut entries: Vec<_> = fs::read_dir(&backups)
            .map_err(|e| format!("read backups dir: {e}"))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|n| n.starts_with(BACKUP_PREFIX) && n.ends_with(BACKUP_SUFFIX))
                    .unwrap_or(false)
            })
            .collect();
        entries.sort_by_key(|e| std::cmp::Reverse(e.file_name()));

        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            let row: Option<(String, i64, String, String)> = sqlx::query_as(
                "SELECT id, size_bytes, status, created_at FROM backup_records WHERE file_name = ?",
            )
            .bind(&name)
            .fetch_optional(pool)
            .await
            .unwrap_or(None);

            if let Some((id, _sz, status, created_at)) = row {
                out.push(BackupMeta {
                    id: Some(id),
                    file_name: name,
                    size_bytes: size,
                    status,
                    created_at,
                });
            } else {
                let created_at = entry
                    .metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .map(|t| {
                        let dt: chrono::DateTime<chrono::Utc> = t.into();
                        dt.with_timezone(&Local)
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string()
                    })
                    .unwrap_or_else(|| "unknown".into());
                out.push(BackupMeta {
                    id: None,
                    file_name: name,
                    size_bytes: size,
                    status: "success".into(),
                    created_at,
                });
            }
        }
    }

    Ok(out)
}

pub async fn latest_backup_time(pool: &SqlitePool) -> Option<String> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT created_at FROM backup_records WHERE status = 'success' \
         ORDER BY created_at DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    row.map(|(t,)| t)
}

/// Stage a restore: integrity-check the snapshot, leave
/// `{db}.restore-pending` (+ optional uploads.restore-pending) for next startup.
pub async fn restore_backup(
    pool: &SqlitePool,
    data_dir: &Path,
    file_name: &str,
) -> Result<String, String> {
    validate_backup_file_name(file_name)?;
    let archive_path = backups_dir(data_dir).join(file_name);
    if !archive_path.is_file() {
        return Err(format!("backup not found: {file_name}"));
    }

    let extract_dir = data_dir.join(format!("restore-tmp-{}", Uuid::new_v4()));
    fs::create_dir_all(&extract_dir).map_err(|e| format!("create extract dir: {e}"))?;

    let cleanup = |dir: &Path| {
        let _ = fs::remove_dir_all(dir);
    };

    let file = File::open(&archive_path).map_err(|e| format!("open archive: {e}"))?;
    let dec = GzDecoder::new(file);
    let mut archive = Archive::new(dec);
    if let Err(e) = archive.unpack(&extract_dir) {
        cleanup(&extract_dir);
        return Err(format!("unpack archive: {e}"));
    }

    let snap = extract_dir.join("database.sqlite");
    if !snap.is_file() {
        cleanup(&extract_dir);
        return Err("archive missing database.sqlite".into());
    }

    let url = format!("sqlite:{}?mode=ro", snap.display());
    let check_pool = match SqlitePool::connect(&url).await {
        Ok(p) => p,
        Err(e) => {
            cleanup(&extract_dir);
            return Err(format!("open snapshot for integrity_check: {e}"));
        }
    };
    let check: Result<(String,), _> = sqlx::query_as("PRAGMA integrity_check")
        .fetch_one(&check_pool)
        .await;
    check_pool.close().await;
    match check {
        Ok((msg,)) if msg.eq_ignore_ascii_case("ok") => {}
        Ok((msg,)) => {
            cleanup(&extract_dir);
            return Err(format!("snapshot integrity_check failed: {msg}"));
        }
        Err(e) => {
            cleanup(&extract_dir);
            return Err(format!("snapshot integrity_check error: {e}"));
        }
    }

    let db_path = sqlite_path_for_data_dir(data_dir);

    let ts = Local::now().format("%Y%m%d-%H%M%S");
    if db_path.exists() {
        let pre = PathBuf::from(format!("{}.pre-restore-{}", db_path.display(), ts));
        if let Err(e) = fs::copy(&db_path, &pre) {
            cleanup(&extract_dir);
            return Err(format!("pre-restore copy: {e}"));
        }
    }

    let pending = PathBuf::from(format!("{}.restore-pending", db_path.display()));
    let _ = fs::remove_file(&pending);
    if let Err(e) = fs::copy(&snap, &pending) {
        cleanup(&extract_dir);
        return Err(format!("write restore-pending: {e}"));
    }

    // Stage uploads if present in the archive.
    let extracted_uploads = extract_dir.join("uploads");
    if extracted_uploads.is_dir() {
        let uploads_pending = data_dir.join("uploads.restore-pending");
        if uploads_pending.exists() {
            let _ = fs::remove_dir_all(&uploads_pending);
        }
        if let Err(e) = copy_dir_recursive(&extracted_uploads, &uploads_pending) {
            cleanup(&extract_dir);
            return Err(format!("stage uploads.restore-pending: {e}"));
        }
    }

    cleanup(&extract_dir);
    let _ = pool; // reserved for future status rows; audit is written by API layer
    Ok("备份已就绪，重启服务后生效（当前库已保留为 .pre-restore-* 副本）".into())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let target = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// Retention: keep last 14 daily backups + up to 12 first-of-month backups.
pub async fn prune_old_backups(pool: &SqlitePool, data_dir: &Path) -> Result<(), String> {
    let list = list_backups(pool, data_dir).await?;
    if list.is_empty() {
        return Ok(());
    }

    // Walk newest-first; keep first 14 overall as "daily", and up to 12 whose
    // calendar day is the 1st as long-term monthly archives.
    let mut daily_n = 0usize;
    let mut monthly_n = 0usize;
    let mut keep = std::collections::HashSet::new();
    for meta in &list {
        let is_first = parse_backup_date(meta)
            .map(|d| d.day() == 1)
            .unwrap_or(false);
        if daily_n < 14 {
            keep.insert(meta.file_name.clone());
            daily_n += 1;
        }
        if is_first && monthly_n < 12 {
            keep.insert(meta.file_name.clone());
            monthly_n += 1;
        }
    }

    for meta in &list {
        if keep.contains(&meta.file_name) {
            continue;
        }
        let path = backups_dir(data_dir).join(&meta.file_name);
        if path.exists() {
            if let Err(e) = fs::remove_file(&path) {
                tracing::warn!("Failed to delete old backup {}: {}", meta.file_name, e);
            } else {
                tracing::info!("🗑️ Pruned backup {}", meta.file_name);
            }
        }
        let _ = sqlx::query("DELETE FROM backup_records WHERE file_name = ?")
            .bind(&meta.file_name)
            .execute(pool)
            .await;
    }
    Ok(())
}

fn parse_backup_date(meta: &BackupMeta) -> Option<NaiveDate> {
    // Prefer created_at "YYYY-MM-DD ..."
    if let Some(date_part) = meta.created_at.get(..10)
        && let Ok(d) = NaiveDate::parse_from_str(date_part, "%Y-%m-%d")
    {
        return Some(d);
    }
    // Fall back to file name backup-YYYYMMDD-HHMMSS.tar.gz
    let mid = meta
        .file_name
        .strip_prefix(BACKUP_PREFIX)?
        .strip_suffix(BACKUP_SUFFIX)?;
    let ymd = mid.get(..8)?;
    NaiveDate::parse_from_str(ymd, "%Y%m%d").ok()
}

/// True if a successful backup already exists for the local calendar day.
pub async fn has_backup_today(pool: &SqlitePool) -> bool {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM backup_records \
         WHERE status = 'success' AND date(created_at) = date(?)",
    )
    .bind(&today)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    row.map(|(c,)| c > 0).unwrap_or(false)
}

/// Resolve absolute path of a backup file after validating the name.
pub fn backup_file_path(data_dir: &Path, file_name: &str) -> Result<PathBuf, String> {
    validate_backup_file_name(file_name)?;
    let path = backups_dir(data_dir).join(file_name);
    if !path.is_file() {
        return Err("backup file not found".into());
    }
    // Extra path-traversal guard after join
    let backups = backups_dir(data_dir);
    let canon_backups = backups.canonicalize().unwrap_or(backups);
    let canon_file = path
        .canonicalize()
        .map_err(|_| "backup file not found".to_string())?;
    if !canon_file.starts_with(&canon_backups) {
        return Err("invalid backup path".into());
    }
    Ok(canon_file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use sqlx::SqlitePool;
    use std::io::Write as _;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("memory");
        db::run_migrations(&pool).await.expect("migrations");
        pool
    }

    #[test]
    fn rejects_path_traversal_file_names() {
        assert!(validate_backup_file_name("../../etc/passwd").is_err());
        assert!(validate_backup_file_name("backup-../../../x.tar.gz").is_err());
        assert!(validate_backup_file_name("backup-ok.tar.gz").is_ok());
        assert!(validate_backup_file_name("backup-20260101-120000.tar.gz").is_ok());
        assert!(validate_backup_file_name("not-a-backup.tar.gz").is_err());
        assert!(validate_backup_file_name("backup-ok.zip").is_err());
    }

    #[tokio::test]
    async fn create_backup_packs_sqlite_and_uploads() {
        let pool = test_pool().await;
        let root = std::env::temp_dir().join(format!("opc-backup-{}", Uuid::new_v4()));
        let data_dir = root.join("data");
        fs::create_dir_all(data_dir.join("uploads/co1")).unwrap();
        fs::create_dir_all(data_dir.join("backups")).unwrap();
        {
            let mut f = File::create(data_dir.join("uploads/co1/hello.txt")).unwrap();
            f.write_all(b"hello-upload").unwrap();
        }

        // Point VACUUM INTO at a real on-disk DB — in-memory cannot VACUUM INTO.
        // Use a file-backed pool for this test.
        let db_file = data_dir.join("database.sqlite");
        let url = format!("sqlite:{}?mode=rwc", db_file.display());
        let file_pool = SqlitePool::connect(&url).await.unwrap();
        db::run_migrations(&file_pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS probe (id TEXT PRIMARY KEY, v TEXT);\
             INSERT OR REPLACE INTO probe (id, v) VALUES ('1', 'marker-42')",
        )
        .execute(&file_pool)
        .await
        .unwrap();

        let meta = create_backup(&file_pool, &data_dir)
            .await
            .expect("create_backup");
        assert!(meta.file_name.starts_with("backup-"));
        assert!(meta.size_bytes > 0);

        let archive = backups_dir(&data_dir).join(&meta.file_name);
        assert!(archive.is_file());

        // Unpack and assert contents.
        let extract = root.join("extract");
        fs::create_dir_all(&extract).unwrap();
        let f = File::open(&archive).unwrap();
        let mut ar = Archive::new(GzDecoder::new(f));
        ar.unpack(&extract).unwrap();
        assert!(extract.join("database.sqlite").is_file());
        assert!(extract.join("uploads/co1/hello.txt").is_file());
        assert_eq!(
            fs::read_to_string(extract.join("uploads/co1/hello.txt")).unwrap(),
            "hello-upload"
        );

        let check_url = format!(
            "sqlite:{}?mode=ro",
            extract.join("database.sqlite").display()
        );
        let check_pool = SqlitePool::connect(&check_url).await.unwrap();
        let row: (String,) = sqlx::query_as("SELECT v FROM probe WHERE id = '1'")
            .fetch_one(&check_pool)
            .await
            .unwrap();
        assert_eq!(row.0, "marker-42");
        check_pool.close().await;
        file_pool.close().await;

        let _ = fs::remove_dir_all(&root);
        let _ = pool; // silence
    }

    #[tokio::test]
    async fn restore_stages_pending_file() {
        let root = std::env::temp_dir().join(format!("opc-restore-{}", Uuid::new_v4()));
        let data_dir = root.join("data");
        fs::create_dir_all(data_dir.join("backups")).unwrap();
        fs::create_dir_all(data_dir.join("uploads")).unwrap();

        let db_file = data_dir.join("database.sqlite");
        let url = format!("sqlite:{}?mode=rwc", db_file.display());
        let pool = SqlitePool::connect(&url).await.unwrap();
        db::run_migrations(&pool).await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS probe (id TEXT PRIMARY KEY, v TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO probe (id, v) VALUES ('1', 'before')")
            .execute(&pool)
            .await
            .unwrap();

        // Snapshot "before" state as backup, then change live data.
        let meta = create_backup(&pool, &data_dir).await.unwrap();
        sqlx::query("UPDATE probe SET v = 'after' WHERE id = '1'")
            .execute(&pool)
            .await
            .unwrap();

        let msg = restore_backup(&pool, &data_dir, &meta.file_name)
            .await
            .expect("restore");
        assert!(msg.contains("重启"));

        let pending = PathBuf::from(format!("{}.restore-pending", db_file.display()));
        assert!(pending.is_file(), "restore-pending must exist");

        pool.close().await;

        // Simulate startup apply.
        db::apply_restore_pending(&db_file).expect("apply pending");
        assert!(!pending.exists());
        assert!(db_file.exists());

        let pool2 = SqlitePool::connect(&url).await.unwrap();
        let row: (String,) = sqlx::query_as("SELECT v FROM probe WHERE id = '1'")
            .fetch_one(&pool2)
            .await
            .unwrap();
        assert_eq!(row.0, "before");
        pool2.close().await;

        let _ = fs::remove_dir_all(&root);
    }
}
