use axum::{
    Extension, Router,
    body::Body,
    extract::{Multipart, Path, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::integrations::daas::DaasClient;

/// Shared DaaS client, injected as a request Extension at the app root. `None`
/// when DAAS_ENABLED is off — uploads then behave exactly as before.
type DaasExt = Extension<Option<Arc<DaasClient>>>;

/// File storage API with permission checks
/// All file operations are company-scoped
///

const UPLOAD_DIR: &str = "data/uploads";
const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB per file

/// Stream a multipart field into a size-capped buffer.
///
/// Rejects with [`StatusCode::PAYLOAD_TOO_LARGE`] as soon as the running total
/// would exceed `max_bytes`. Replaces the previous `field.bytes().await` +
/// post-check pattern that buffered the entire body before applying the cap
/// (DoS / memory spike under oversized multipart uploads).
async fn read_field_capped(
    mut field: axum::extract::multipart::Field<'_>,
    max_bytes: usize,
) -> Result<Vec<u8>, StatusCode> {
    let mut data = Vec::new();
    while let Some(chunk) = field.chunk().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let next = data
            .len()
            .checked_add(chunk.len())
            .ok_or(StatusCode::PAYLOAD_TOO_LARGE)?;
        if next > max_bytes {
            // Stop reading further chunks; do not retain the oversize tail.
            return Err(StatusCode::PAYLOAD_TOO_LARGE);
        }
        data.extend_from_slice(&chunk);
    }
    Ok(data)
}

fn compact_number_label(value: f64) -> String {
    if (value.fract()).abs() < f64::EPSILON {
        format!("{}", value as i64)
    } else {
        format!("{value:.2}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

fn file_size_label(bytes: i64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{} KB", compact_number_label(bytes as f64 / 1024.0))
    } else {
        format!(
            "{} MB",
            compact_number_label(bytes as f64 / 1024.0 / 1024.0)
        )
    }
}

fn date_label(value: &str) -> String {
    if value.trim().is_empty() {
        "-".to_string()
    } else {
        value.chars().take(10).collect()
    }
}

fn file_icon(content_type: &str) -> (&'static str, &'static str) {
    if content_type.starts_with("image/") {
        ("ImageIcon", "h-5 w-5 text-blue-500")
    } else if content_type == "application/pdf" {
        ("FileText", "h-5 w-5 text-red-500")
    } else if content_type.contains("word") {
        ("FileText", "h-5 w-5 text-blue-600")
    } else if content_type.contains("excel") || content_type.contains("spreadsheet") {
        ("FileText", "h-5 w-5 text-green-600")
    } else {
        ("File", "h-5 w-5 text-gray-500")
    }
}

fn file_response(
    id: String,
    filename: String,
    original_name: String,
    content_type: String,
    size: i64,
    created_at: Option<String>,
    entity_type: Option<String>,
    entity_id: Option<String>,
    description: Option<String>,
) -> Value {
    let size_label = file_size_label(size);
    let created_at_label = created_at
        .as_deref()
        .map(date_label)
        .unwrap_or_else(|| "-".to_string());
    let (icon, icon_class_name) = file_icon(&content_type);
    json!({
        "id": id.clone(),
        "filename": filename,
        "original_name": original_name.clone(),
        "content_type": content_type.clone(),
        "size": size,
        "size_label": size_label,
        "created_at": created_at.clone(),
        "created_at_label": created_at_label.clone(),
        "file_name": original_name,
        "file_path": id,
        "file_type": content_type.clone(),
        "description": description,
        "upload_date": created_at,
        "upload_date_label": created_at_label,
        "file_icon": icon,
        "file_icon_class_name": icon_class_name,
        "delete_action": file_delete_action(&id, &original_name),
        "metadata_label": format!("{} · {}", content_type, size_label),
        "entity_type": entity_type,
        "entity_id": entity_id,
        "url": format!("/api/files/{}", id)
    })
}

fn file_delete_action(id: &str, original_name: &str) -> Value {
    let file_name = original_name.trim();
    let file_name = if file_name.is_empty() {
        "该附件"
    } else {
        file_name
    };
    json!({
        "method": "DELETE",
        "endpoint": format!("/api/files/{id}"),
        "confirm_message": format!("确定要删除附件「{file_name}」吗？此操作无法撤销。"),
        "success_title": "删除成功",
        "success_message": "附件已删除",
    })
}

fn normalize_entity_type(entity_type: &str) -> Result<String, StatusCode> {
    let normalized = entity_type.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let canonical = match normalized.as_str() {
        "invoice" | "invoices" => "invoices",
        "contract" | "contracts" => "contracts",
        "asset" | "assets" => "assets",
        "employee" | "employees" => "employees",
        "budget" | "budgets" => "budgets",
        "quote" | "quotes" => "quotes",
        "payment" | "payments" => "payments",
        "application" | "applications" => "applications",
        "material" | "materials" => "materials",
        "bank_account" | "bank_accounts" => "bank_accounts",
        _ => normalized.as_str(),
    };

    if !canonical
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(canonical.to_string())
}

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/upload", post(upload_file))
        .route("/upload/{entity_type}/{entity_id}", post(upload_for_entity))
        // Admin: re-mirror this company's pending/failed files into DaaS.
        .route("/daas/backfill", post(daas_backfill))
        .route("/{id}", get(download_file).delete(delete_file))
        .route("/list/{entity_type}/{entity_id}", get(list_files))
        // Public static file serving (for images in img tags)
        .route("/static/{*path}", get(serve_static_file))
        .with_state(pool)
}

/// Extract user and company from session.
async fn get_auth_context(
    headers: &HeaderMap,
    pool: &SqlitePool,
) -> Result<(String, String), StatusCode> {
    let (user_id, company_id, _) = get_auth_context_full(headers, pool).await?;
    Ok((user_id, company_id))
}

/// Like [`get_auth_context`] but also returns the session's W-ID access token,
/// forwarded to DaaS so it scopes the mirror per real user + workspace.
async fn get_auth_context_full(
    headers: &HeaderMap,
    pool: &SqlitePool,
) -> Result<(String, String, Option<String>), StatusCode> {
    // Keep this handler-local lookup aligned with the outer auth middleware:
    // HttpOnly browser cookie first, then explicit CLI session headers.
    let session_id =
        crate::auth::middleware::extract_session_id(headers).ok_or(StatusCode::UNAUTHORIZED)?;

    // S5: Check session exists and is not expired. Expiry is compared in SQL
    // (same as auth middleware): sessions store SQLite "YYYY-MM-DD HH:MM:SS"
    // timestamps, so comparing them against an RFC3339 string in Rust would
    // mis-order on the expiry day (' ' vs 'T' at position 10).
    let session: Option<(String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT user_id, current_company_id, access_token FROM sessions WHERE id = ? AND (expires_at IS NULL OR expires_at > datetime('now'))",
    )
    .bind(&session_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (user_id, company_id, access_token) = session.ok_or(StatusCode::UNAUTHORIZED)?;

    let company_id = company_id.ok_or(StatusCode::FORBIDDEN)?;

    Ok((user_id, company_id, access_token))
}

/// Spawn the best-effort DaaS mirror for a freshly-inserted file, if the
/// integration is enabled. Runs after the upload response is returned, so a
/// DaaS outage never affects the user-facing upload.
fn spawn_daas_mirror(
    daas: &Option<Arc<DaasClient>>,
    pool: &SqlitePool,
    file_id: &str,
    company_id: &str,
    original_name: &str,
    content_type: &str,
    bytes: Vec<u8>,
    wid_token: Option<String>,
) {
    let Some(daas) = daas.clone() else { return };
    let pool = pool.clone();
    let file_id = file_id.to_string();
    let company_id = company_id.to_string();
    let original_name = original_name.to_string();
    let content_type = content_type.to_string();
    tokio::spawn(async move {
        daas.mirror_file(
            &pool,
            &file_id,
            &company_id,
            &original_name,
            &content_type,
            bytes,
            wid_token,
        )
        .await;
    });
}

/// Check if user has access to company (and optionally specific permission)
async fn check_permission(
    pool: &SqlitePool,
    user_id: &str,
    company_id: &str,
    required_role: Option<&str>,
) -> Result<String, StatusCode> {
    let member: Option<(String,)> =
        sqlx::query_as("SELECT role FROM company_members WHERE user_id = ? AND company_id = ?")
            .bind(user_id)
            .bind(company_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let role = member.ok_or(StatusCode::FORBIDDEN)?.0;

    // Check role hierarchy if required
    if let Some(required) = required_role {
        let allowed = match required {
            "owner" => role == "owner",
            "admin" => role == "owner" || role == "admin",
            "member" => role == "owner" || role == "admin" || role == "member",
            "accountant" => role == "owner" || role == "admin" || role == "accountant",
            "auditor" => true, // auditors can only read
            _ => role == "owner" || role == "admin" || role == "member",
        };
        if !allowed {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    Ok(role)
}

// Upload a file
async fn upload_file(
    State(pool): State<SqlitePool>,
    Extension(daas): DaasExt,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<Value>, StatusCode> {
    let (user_id, company_id, wid_token) = get_auth_context_full(&headers, &pool).await?;
    check_permission(&pool, &user_id, &company_id, Some("member")).await?;

    let upload_path = PathBuf::from(UPLOAD_DIR).join(&company_id);
    fs::create_dir_all(&upload_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let original_name = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let content_type = field
            .content_type()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        // Stream + early abort at MAX_FILE_SIZE (never buffer unbounded body).
        let data = read_field_capped(field, MAX_FILE_SIZE).await?;
        let size = data.len() as i64;

        // Use SHA-256 content hash as filename
        let hash = w_crypto::sha256_hex(&data);

        let file_id = uuid::Uuid::new_v4().to_string();
        // Store file with hash name only (no extension on disk)
        let file_path = upload_path.join(&hash);

        // Only write if not already present (content-addressable dedup)
        if !file_path.exists() {
            let mut file = fs::File::create(&file_path)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            file.write_all(&data)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        sqlx::query(
            "INSERT INTO files (id, company_id, filename, original_name, content_type, size, uploaded_by) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&file_id)
        .bind(&company_id)
        .bind(&hash)
        .bind(&original_name)
        .bind(&content_type)
        .bind(size)
        .bind(&user_id)
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to save file record: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        spawn_daas_mirror(
            &daas,
            &pool,
            &file_id,
            &company_id,
            &original_name,
            &content_type,
            data.to_vec(),
            wid_token,
        );

        return Ok(Json(json!({
            "id": file_id,
            "original_name": original_name,
            "content_type": content_type,
            "size": size,
            "size_label": file_size_label(size),
            "created_at_label": date_label(&chrono::Utc::now().to_rfc3339()),
            "metadata_label": format!("{} · {}", content_type, file_size_label(size)),
            "url": format!("/api/files/{}", file_id)
        })));
    }

    Err(StatusCode::BAD_REQUEST)
}

// Upload file for a specific entity
async fn upload_for_entity(
    State(pool): State<SqlitePool>,
    Extension(daas): DaasExt,
    headers: HeaderMap,
    Path((entity_type, entity_id)): Path<(String, String)>,
    mut multipart: Multipart,
) -> Result<Json<Value>, StatusCode> {
    let (user_id, company_id, wid_token) = get_auth_context_full(&headers, &pool).await?;
    check_permission(&pool, &user_id, &company_id, Some("member")).await?;
    let entity_type = normalize_entity_type(&entity_type)?;

    let upload_path = PathBuf::from(UPLOAD_DIR).join(&company_id);
    fs::create_dir_all(&upload_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut payload: Option<(String, String, Vec<u8>)> = None;
    let mut description: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let field_name = field.name().unwrap_or_default().to_string();
        if field_name == "description" {
            let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            let value = value.trim().to_string();
            if !value.is_empty() {
                description = Some(value);
            }
            continue;
        }

        let original_name = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let content_type = field
            .content_type()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());
        // Stream + early abort at MAX_FILE_SIZE (never buffer unbounded body).
        let data = read_field_capped(field, MAX_FILE_SIZE).await?;
        payload = Some((original_name, content_type, data));
    }

    let Some((original_name, content_type, data)) = payload else {
        return Err(StatusCode::BAD_REQUEST);
    };
    let size = data.len() as i64;

    // Use SHA-256 content hash as filename
    let hash = w_crypto::sha256_hex(&data);

    let file_id = uuid::Uuid::new_v4().to_string();
    let file_path = upload_path.join(&hash);

    // Only write if not already present (content-addressable dedup)
    if !file_path.exists() {
        let mut file = fs::File::create(&file_path)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        file.write_all(&data)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    sqlx::query(
        "INSERT INTO files (id, company_id, filename, original_name, content_type, size, entity_type, entity_id, description, uploaded_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&file_id)
    .bind(&company_id)
    .bind(&hash)
    .bind(&original_name)
    .bind(&content_type)
    .bind(size)
    .bind(&entity_type)
    .bind(&entity_id)
    .bind(&description)
    .bind(&user_id)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    spawn_daas_mirror(
        &daas,
        &pool,
        &file_id,
        &company_id,
        &original_name,
        &content_type,
        data,
        wid_token,
    );

    Ok(Json(json!({
        "id": file_id.clone(),
        "original_name": original_name,
        "content_type": content_type.clone(),
        "size": size,
        "size_label": file_size_label(size),
        "created_at_label": date_label(&chrono::Utc::now().to_rfc3339()),
        "metadata_label": format!("{} · {}", content_type, file_size_label(size)),
        "entity_type": entity_type,
        "entity_id": entity_id,
        "description": description,
        "url": format!("/api/files/{}", file_id)
    })))
}

/// Admin: re-mirror this company's `pending`/`failed` files into DaaS,
/// forwarding the caller's W-ID token. Idempotent — `synced` rows are skipped,
/// and unsupported types settle as `skipped`. Covers both the one-time backfill
/// of pre-fusion files and retries of transient mirror failures.
async fn daas_backfill(
    State(pool): State<SqlitePool>,
    Extension(daas): DaasExt,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    let (user_id, company_id, wid_token) = get_auth_context_full(&headers, &pool).await?;
    check_permission(&pool, &user_id, &company_id, Some("admin")).await?;

    let Some(daas) = daas else {
        return Ok(Json(json!({
            "enabled": false,
            "message": "DaaS integration disabled (DAAS_ENABLED is off)",
        })));
    };

    let (synced, failed, skipped) = daas
        .backfill_company(&pool, &company_id, wid_token, UPLOAD_DIR)
        .await;

    Ok(Json(json!({
        "enabled": true,
        "synced": synced,
        "failed": failed,
        "skipped": skipped,
    })))
}

// Download file (with company scope check)
async fn download_file(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<([(axum::http::header::HeaderName, String); 2], Body), StatusCode> {
    let (user_id, company_id) = get_auth_context(&headers, &pool).await?;
    check_permission(&pool, &user_id, &company_id, None).await?;

    // Get file record - ensure it belongs to user's company
    let record: Option<(String, String, String)> = sqlx::query_as(
        "SELECT filename, original_name, content_type FROM files WHERE id = ? AND company_id = ?",
    )
    .bind(&id)
    .bind(&company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (filename, original_name, content_type) = record.ok_or(StatusCode::NOT_FOUND)?;

    let file_path = PathBuf::from(UPLOAD_DIR).join(&company_id).join(&filename);
    let data = fs::read(&file_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let headers = [
        (axum::http::header::CONTENT_TYPE, content_type),
        (
            axum::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", original_name),
        ),
    ];

    Ok((headers, Body::from(data)))
}

// Delete file (with permission check)
async fn delete_file(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let (user_id, company_id) = get_auth_context(&headers, &pool).await?;
    check_permission(&pool, &user_id, &company_id, Some("member")).await?;

    // Get file record - ensure it belongs to user's company
    let record: Option<(String,)> =
        sqlx::query_as("SELECT filename FROM files WHERE id = ? AND company_id = ?")
            .bind(&id)
            .bind(&company_id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (filename,) = record.ok_or(StatusCode::NOT_FOUND)?;

    // L1: Check if other records reference the same hash before deleting physical file
    let ref_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM files WHERE filename = ? AND company_id = ?")
            .bind(&filename)
            .bind(&company_id)
            .fetch_one(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Delete DB record first
    sqlx::query("DELETE FROM files WHERE id = ? AND company_id = ?")
        .bind(&id)
        .bind(&company_id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Only delete physical file if no other records reference it
    if ref_count.0 <= 1 {
        let file_path = PathBuf::from(UPLOAD_DIR).join(&company_id).join(&filename);
        let _ = fs::remove_file(&file_path).await;
    }

    Ok(StatusCode::NO_CONTENT)
}

// List files for an entity (with company scope)
async fn list_files(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Path((entity_type, entity_id)): Path<(String, String)>,
) -> Result<Json<Value>, StatusCode> {
    let (user_id, company_id) = get_auth_context(&headers, &pool).await?;
    check_permission(&pool, &user_id, &company_id, None).await?;
    let entity_type = normalize_entity_type(&entity_type)?;

    let files: Vec<(
        String,
        String,
        String,
        String,
        i64,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT id, filename, original_name, content_type, size, created_at, entity_type, entity_id, description FROM files WHERE company_id = ? AND entity_type = ? AND entity_id = ? ORDER BY created_at DESC"
    )
    .bind(&company_id)
    .bind(&entity_type)
    .bind(&entity_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let data: Vec<Value> = files
        .into_iter()
        .map(|f| file_response(f.0, f.1, f.2, f.3, f.4, f.5, f.6, f.7, f.8))
        .collect();

    Ok(Json(json!({ "data": data })))
}

// Serve stored files for authenticated users (for img tags via fetch/blob)
// Path format: /api/files/static/{company_id}/{filename}
async fn serve_static_file(
    user: crate::auth::AuthUser,
    Path(path): Path<String>,
) -> Result<([(axum::http::header::HeaderName, String); 1], Body), StatusCode> {
    // R3-5: Enhanced path traversal protection
    let path = path.trim_start_matches('/');
    if path.contains("..") || path.contains("./") || path.contains("//") {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Tenant isolation: the {company_id} path prefix must be the caller's own
    // company. Without this check any authenticated user could read another
    // company's files by content hash.
    let company_prefix = path.split('/').next().unwrap_or_default();
    if company_prefix.is_empty() || company_prefix != user.company_id {
        return Err(StatusCode::NOT_FOUND);
    }

    // Path is expected to be {company_id}/{filename}
    let file_path = PathBuf::from(UPLOAD_DIR).join(path);

    // Canonicalize and verify it's within UPLOAD_DIR
    let upload_root = std::path::Path::new(UPLOAD_DIR)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(UPLOAD_DIR));
    let canonical = match file_path.canonicalize() {
        Ok(p) => p,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };
    if !canonical.starts_with(&upload_root) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let data = fs::read(&file_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Determine content type from extension via the shared fleet MIME map.
    // Case-insensitive (a strict superset of the previous case-sensitive map):
    // png/jpg/jpeg/gif/pdf/webp resolve identically, and `.PNG` etc. now also resolve.
    let content_type = file_path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(w_util::mime::extension_to_mime)
        .unwrap_or("application/octet-stream");

    Ok((
        [(axum::http::header::CONTENT_TYPE, content_type.to_string())],
        Body::from(data),
    ))
}

#[cfg(test)]
mod sha256_golden_tests {
    //! Content-addressable file storage uses the SHA-256 hex of the upload as
    //! the on-disk filename (dedup key). Migrating the inline
    //! `format!("{:x}", Sha256::digest(..))` to `w_crypto::sha256_hex` MUST NOT
    //! change a single stored digest, or existing files become unreachable.
    use sha2::{Digest, Sha256};

    // Pinned known-answer vectors (computed with `printf '%s' .. | sha256sum`).
    const ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    const EMPTY: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    const FIXTURE: &str = "ab0de80b2f9a456940f5bcb84ae2ed4ee7a5b5989827cbcf728e2de50a7b90d4";

    /// The exact inline expression these handlers used BEFORE the migration.
    fn legacy_inline(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    #[test]
    fn w_crypto_matches_legacy_inline_byte_for_byte() {
        for data in [b"".as_slice(), b"abc", b"OneMan-erp-golden-fixture"] {
            assert_eq!(w_crypto::sha256_hex(data), legacy_inline(data));
        }
    }

    #[test]
    fn w_crypto_matches_pinned_golden() {
        assert_eq!(w_crypto::sha256_hex(b"abc"), ABC);
        assert_eq!(w_crypto::sha256_hex(b""), EMPTY);
        assert_eq!(w_crypto::sha256_hex(b"OneMan-erp-golden-fixture"), FIXTURE);
    }
}

#[cfg(test)]
mod upload_stream_cap_tests {
    //! Contract: multipart file fields must stream via `field.chunk()` with an
    //! early size abort — never `field.bytes().await` then post-check.
    use super::read_field_capped;
    use axum::{
        Router,
        body::Body,
        extract::Multipart,
        http::{Request, StatusCode},
        routing::post,
    };
    use tower::ServiceExt;

    #[test]
    fn source_does_not_buffer_multipart_with_field_bytes() {
        let source = include_str!("files.rs");
        // Build needles at runtime so this test body never embeds the forbidden
        // call-site text as a contiguous literal that would self-match.
        let forbidden = format!("{}.{}().await", "field", "bytes");
        for (lineno, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with('*') {
                continue;
            }
            // Skip the runtime-built format! line and assertion messages that
            // reconstruct the anti-pattern for the contract check itself.
            if trimmed.contains("format!") && trimmed.contains("\"bytes\"") {
                continue;
            }
            assert!(
                !trimmed.contains(&forbidden),
                "files.rs:{} must not call {forbidden} on upload path: {trimmed}",
                lineno + 1
            );
        }
        assert!(
            source.contains("read_field_capped") && source.contains(".chunk()"),
            "files.rs must stream multipart via read_field_capped / field.chunk()"
        );
        assert!(
            source.contains("MAX_FILE_SIZE"),
            "files.rs must keep the 10 MiB MAX_FILE_SIZE cap"
        );
    }

    #[tokio::test]
    async fn read_field_capped_accepts_body_at_limit() {
        let cap = 64usize;
        let payload = vec![b'a'; cap];
        let app = Router::new().route(
            "/",
            post(move |mut multipart: Multipart| async move {
                let field = multipart
                    .next_field()
                    .await
                    .expect("multipart parse")
                    .expect("field");
                match read_field_capped(field, cap).await {
                    Ok(data) => {
                        assert_eq!(data.len(), cap);
                        StatusCode::OK
                    }
                    Err(status) => status,
                }
            }),
        );

        let body = build_multipart_body(&payload);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/")
                    .header("content-type", "multipart/form-data; boundary=TESTBOUNDARY")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn read_field_capped_rejects_oversize_without_needing_full_buffer_postcheck() {
        // Cap is tiny so the test proves early abort without a multi-MB body.
        // Payload is several chunks larger than the cap; server must 413 and
        // must not successfully return Ok with the full body.
        let cap = 128usize;
        let payload = vec![b'x'; cap + 512];
        let app = Router::new().route(
            "/",
            post(move |mut multipart: Multipart| async move {
                let field = multipart
                    .next_field()
                    .await
                    .expect("multipart parse")
                    .expect("field");
                match read_field_capped(field, cap).await {
                    Ok(data) => {
                        // If this fires, the helper buffered past the cap.
                        panic!(
                            "oversize field must not succeed; got {} bytes (cap={cap})",
                            data.len()
                        );
                    }
                    Err(status) => status,
                }
            }),
        );

        let body = build_multipart_body(&payload);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/")
                    .header("content-type", "multipart/form-data; boundary=TESTBOUNDARY")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    fn build_multipart_body(payload: &[u8]) -> Vec<u8> {
        let mut body = Vec::new();
        body.extend_from_slice(b"--TESTBOUNDARY\r\n");
        body.extend_from_slice(
            b"Content-Disposition: form-data; name=\"file\"; filename=\"blob.bin\"\r\n",
        );
        body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
        body.extend_from_slice(payload);
        body.extend_from_slice(b"\r\n--TESTBOUNDARY--\r\n");
        body
    }
}
