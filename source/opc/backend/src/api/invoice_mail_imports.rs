use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use mailparse::{DispositionType, MailHeaderMap, ParsedMail};
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};
use std::{
    net::{TcpStream, ToSocketAddrs},
    sync::Arc,
    time::Duration,
};

use crate::auth::AuthUser;

use super::{
    error::{ApiError, api_err},
    imports::{InvoiceImportRow, parse_invoice_xml_attachment},
};

const DEFAULT_IMAP_PORT: u16 = 993;
const DEFAULT_FOLDER: &str = "INBOX";
const MAX_XML_BYTES: usize = 5 * 1024 * 1024;
const IO_TIMEOUT: Duration = Duration::from_secs(30);

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route(
            "/invoice-mailboxes",
            get(list_mailboxes).post(create_mailbox),
        )
        .route(
            "/invoice-mailboxes/{id}",
            axum::routing::put(update_mailbox).delete(delete_mailbox),
        )
        .route("/invoice-mailboxes/{id}/sync", post(sync_mailbox_now))
        .route("/invoice-mail-imports", get(list_imports))
        .route("/invoice-mail-imports/{id}/confirm", post(confirm_import))
        .route("/invoice-mail-imports/{id}/reject", post(reject_import))
        .with_state(pool)
}

#[derive(Debug, Deserialize)]
struct MailboxInput {
    name: String,
    imap_host: String,
    #[serde(default = "default_imap_port")]
    imap_port: u16,
    imap_username: String,
    #[serde(default)]
    imap_password: Option<String>,
    #[serde(default = "default_folder")]
    folder: String,
    #[serde(default)]
    enabled: bool,
}

fn default_imap_port() -> u16 {
    DEFAULT_IMAP_PORT
}

fn default_folder() -> String {
    DEFAULT_FOLDER.to_string()
}

#[derive(Debug, Clone)]
struct MailboxConfig {
    id: String,
    company_id: String,
    name: String,
    imap_host: String,
    imap_port: u16,
    imap_username: String,
    imap_password: String,
    folder: String,
    enabled: bool,
    last_uid: u32,
}

#[derive(Debug)]
struct FetchedMessage {
    uid: u32,
    raw: Vec<u8>,
}

#[derive(Debug)]
struct XmlAttachment {
    index: usize,
    filename: String,
    bytes: Vec<u8>,
    decode_error: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct SyncSummary {
    mailbox_id: String,
    messages_checked: usize,
    xml_attachments: usize,
    pending_confirmation: usize,
    duplicate: usize,
    failed: usize,
    ignored_existing_source: usize,
    last_uid: u32,
}

#[derive(Debug, Deserialize)]
struct ImportListQuery {
    status: Option<String>,
    mailbox_id: Option<String>,
    limit: Option<u32>,
}

async fn list_mailboxes(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<Value>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, imap_host, imap_port, imap_username, folder, enabled,
               last_uid, last_sync_at, last_sync_status, last_sync_error,
               CASE WHEN COALESCE(imap_password, '') = '' THEN 0 ELSE 1 END AS has_password,
               created_at, updated_at
        FROM invoice_mailboxes
        WHERE company_id = ?
        ORDER BY enabled DESC, created_at DESC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "mailbox query failed"))?;

    let data = rows
        .into_iter()
        .map(|row| {
            let id: String = row.get("id");
            let enabled = row.get::<i64, _>("enabled") == 1;
            json!({
                "id": id,
                "name": row.get::<String, _>("name"),
                "imap_host": row.get::<String, _>("imap_host"),
                "imap_port": row.get::<i64, _>("imap_port"),
                "imap_username": row.get::<String, _>("imap_username"),
                "folder": row.get::<String, _>("folder"),
                "enabled": enabled,
                "has_password": row.get::<i64, _>("has_password") == 1,
                "last_uid": row.get::<i64, _>("last_uid"),
                "last_sync_at": row.try_get::<String, _>("last_sync_at").ok(),
                "last_sync_status": row.try_get::<String, _>("last_sync_status").ok(),
                "last_sync_error": row.try_get::<String, _>("last_sync_error").ok(),
                "created_at": row.try_get::<String, _>("created_at").ok(),
                "updated_at": row.try_get::<String, _>("updated_at").ok(),
                "sync_action": if enabled { Some(json!({
                    "method": "POST",
                    "endpoint": format!("/api/v1/invoice-mailboxes/{id}/sync"),
                    "label": "立即拉取 XML 发票",
                    "success_message": "邮箱拉取完成"
                })) } else { None },
                "update_action": {
                    "method": "PUT",
                    "endpoint": format!("/api/v1/invoice-mailboxes/{id}"),
                    "label": "保存邮箱",
                    "success_message": "邮箱配置已更新"
                },
                "delete_action": {
                    "method": "DELETE",
                    "endpoint": format!("/api/v1/invoice-mailboxes/{id}"),
                    "confirm_message": "确定删除这个发票邮箱配置吗？",
                    "success_message": "发票邮箱已删除"
                }
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(json!({
        "data": data,
        "create_action": {
            "method": "POST",
            "endpoint": "/api/v1/invoice-mailboxes",
            "label": "添加发票邮箱",
            "success_message": "发票邮箱已创建"
        }
    })))
}

async fn create_mailbox(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Json(input): Json<MailboxInput>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    validate_mailbox_input(&input, true)?;
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO invoice_mailboxes (
            id, company_id, name, imap_host, imap_port, imap_username,
            imap_password, folder, enabled
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&user.company_id)
    .bind(input.name.trim())
    .bind(input.imap_host.trim())
    .bind(i64::from(input.imap_port))
    .bind(input.imap_username.trim())
    .bind(input.imap_password.as_deref().unwrap_or_default())
    .bind(input.folder.trim())
    .bind(i64::from(input.enabled))
    .execute(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "mailbox create failed"))?;

    Ok((StatusCode::CREATED, Json(json!({ "id": id }))))
}

async fn update_mailbox(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(input): Json<MailboxInput>,
) -> Result<Json<Value>, ApiError> {
    validate_mailbox_input(&input, false)?;
    let password = input
        .imap_password
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let result = sqlx::query(
        r#"
        UPDATE invoice_mailboxes
        SET name = ?, imap_host = ?, imap_port = ?, imap_username = ?,
            imap_password = COALESCE(?, imap_password), folder = ?, enabled = ?,
            last_uid = CASE
                WHEN imap_host != ? OR imap_port != ? OR imap_username != ? OR folder != ?
                THEN 0 ELSE last_uid END,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ? AND company_id = ?
        "#,
    )
    .bind(input.name.trim())
    .bind(input.imap_host.trim())
    .bind(i64::from(input.imap_port))
    .bind(input.imap_username.trim())
    .bind(password)
    .bind(input.folder.trim())
    .bind(i64::from(input.enabled))
    .bind(input.imap_host.trim())
    .bind(i64::from(input.imap_port))
    .bind(input.imap_username.trim())
    .bind(input.folder.trim())
    .bind(&id)
    .bind(&user.company_id)
    .execute(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "mailbox update failed"))?;
    if result.rows_affected() == 0 {
        return Err(api_err(StatusCode::NOT_FOUND, "mailbox not found"));
    }
    Ok(Json(json!({ "id": id })))
}

async fn delete_mailbox(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let import_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM invoice_mail_imports WHERE mailbox_id = ? AND company_id = ?",
    )
    .bind(&id)
    .bind(&user.company_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "mailbox delete failed"))?;
    if import_count > 0 {
        return Err(api_err(
            StatusCode::CONFLICT,
            "mailbox has import history; disable it instead",
        ));
    }
    let result = sqlx::query("DELETE FROM invoice_mailboxes WHERE id = ? AND company_id = ?")
        .bind(&id)
        .bind(&user.company_id)
        .execute(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "mailbox delete failed"))?;
    if result.rows_affected() == 0 {
        return Err(api_err(StatusCode::NOT_FOUND, "mailbox not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

fn validate_mailbox_input(input: &MailboxInput, password_required: bool) -> Result<(), ApiError> {
    if input.name.trim().is_empty()
        || input.imap_host.trim().is_empty()
        || input.imap_username.trim().is_empty()
    {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "name, imap_host and imap_username are required",
        ));
    }
    if input.imap_port == 0 {
        return Err(api_err(StatusCode::BAD_REQUEST, "invalid IMAP port"));
    }
    if password_required
        && input
            .imap_password
            .as_deref()
            .map(str::trim)
            .unwrap_or_default()
            .is_empty()
    {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "imap_password is required",
        ));
    }
    if input.folder.trim().is_empty()
        || input.folder.contains(['\r', '\n', '\0'])
        || input.imap_host.contains(['\r', '\n', '\0'])
    {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "invalid IMAP mailbox value",
        ));
    }
    Ok(())
}

async fn sync_mailbox_now(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<SyncSummary>, ApiError> {
    let summary = sync_one_mailbox(&pool, &user.company_id, &id)
        .await
        .map_err(|message| {
            let status = match message.as_str() {
                "mailbox not found" => StatusCode::NOT_FOUND,
                "mailbox is disabled" => StatusCode::CONFLICT,
                _ => StatusCode::BAD_GATEWAY,
            };
            api_err(status, &message)
        })?;
    Ok(Json(summary))
}

pub async fn sync_enabled_mailboxes(pool: &SqlitePool) -> Result<usize, sqlx::Error> {
    let rows = sqlx::query("SELECT id, company_id FROM invoice_mailboxes WHERE enabled = 1")
        .fetch_all(pool)
        .await?;
    let mut synced = 0usize;
    for row in rows {
        let id: String = row.get("id");
        let company_id: String = row.get("company_id");
        match sync_one_mailbox(pool, &company_id, &id).await {
            Ok(_) => synced += 1,
            Err(error) => tracing::warn!(mailbox_id = %id, "invoice mailbox sync failed: {error}"),
        }
    }
    Ok(synced)
}

pub fn spawn_scheduler(pool: SqlitePool) {
    tokio::spawn(async move {
        loop {
            if let Err(error) = sync_enabled_mailboxes(&pool).await {
                tracing::warn!("invoice mailbox scheduler failed: {error}");
            }
            tokio::time::sleep(Duration::from_secs(15 * 60)).await;
        }
    });
}

async fn sync_one_mailbox(
    pool: &SqlitePool,
    company_id: &str,
    mailbox_id: &str,
) -> Result<SyncSummary, String> {
    let config = load_mailbox(pool, company_id, mailbox_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "mailbox not found".to_string())?;
    if !config.enabled {
        return Err("mailbox is disabled".to_string());
    }

    set_sync_state(pool, mailbox_id, company_id, "running", None, None)
        .await
        .map_err(|error| error.to_string())?;
    let fetch_config = config.clone();
    let fetched = tokio::task::spawn_blocking(move || fetch_imap_messages(&fetch_config))
        .await
        .map_err(|error| format!("IMAP task failed: {error}"))?;

    let messages = match fetched {
        Ok(messages) => messages,
        Err(error) => {
            let _ =
                set_sync_state(pool, mailbox_id, company_id, "failed", Some(&error), None).await;
            return Err(error);
        }
    };

    let mut summary = SyncSummary {
        mailbox_id: mailbox_id.to_string(),
        messages_checked: messages.len(),
        last_uid: config.last_uid,
        ..SyncSummary::default()
    };
    for message in messages {
        summary.last_uid = summary.last_uid.max(message.uid);
        ingest_message(pool, &config, message, &mut summary).await;
    }

    let sync_error =
        (summary.failed > 0).then(|| format!("{} XML attachment(s) failed", summary.failed));
    set_sync_state(
        pool,
        mailbox_id,
        company_id,
        if summary.failed > 0 {
            "completed_with_errors"
        } else {
            "completed"
        },
        sync_error.as_deref(),
        Some(summary.last_uid),
    )
    .await
    .map_err(|error| error.to_string())?;
    Ok(summary)
}

async fn load_mailbox(
    pool: &SqlitePool,
    company_id: &str,
    mailbox_id: &str,
) -> Result<Option<MailboxConfig>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, company_id, name, imap_host, imap_port, imap_username,
               imap_password, folder, enabled, last_uid
        FROM invoice_mailboxes WHERE id = ? AND company_id = ?
        "#,
    )
    .bind(mailbox_id)
    .bind(company_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| MailboxConfig {
        id: row.get("id"),
        company_id: row.get("company_id"),
        name: row.get("name"),
        imap_host: row.get("imap_host"),
        imap_port: row.get::<i64, _>("imap_port") as u16,
        imap_username: row.get("imap_username"),
        imap_password: row.get("imap_password"),
        folder: row.get("folder"),
        enabled: row.get::<i64, _>("enabled") == 1,
        last_uid: row.get::<i64, _>("last_uid").max(0) as u32,
    }))
}

async fn set_sync_state(
    pool: &SqlitePool,
    mailbox_id: &str,
    company_id: &str,
    status: &str,
    error: Option<&str>,
    last_uid: Option<u32>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE invoice_mailboxes
        SET last_sync_at = CURRENT_TIMESTAMP, last_sync_status = ?, last_sync_error = ?,
            last_uid = COALESCE(?, last_uid), updated_at = CURRENT_TIMESTAMP
        WHERE id = ? AND company_id = ?
        "#,
    )
    .bind(status)
    .bind(error)
    .bind(last_uid.map(i64::from))
    .bind(mailbox_id)
    .bind(company_id)
    .execute(pool)
    .await?;
    Ok(())
}

fn wrap_imap_tls(
    host: &str,
    tcp: TcpStream,
) -> Result<StreamOwned<ClientConnection, TcpStream>, String> {
    crate::install_crypto_provider();
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let server_name = ServerName::try_from(host.to_string())
        .map_err(|error| format!("IMAP TLS hostname invalid: {error}"))?;
    let conn = ClientConnection::new(Arc::new(config), server_name)
        .map_err(|error| format!("IMAP TLS setup failed: {error}"))?;
    Ok(StreamOwned::new(conn, tcp))
}

fn fetch_imap_messages(config: &MailboxConfig) -> Result<Vec<FetchedMessage>, String> {
    let mut last_error = None;
    let addresses = (config.imap_host.as_str(), config.imap_port)
        .to_socket_addrs()
        .map_err(|error| format!("IMAP DNS lookup failed: {error}"))?;
    let mut tcp = None;
    for address in addresses {
        match TcpStream::connect_timeout(&address, IO_TIMEOUT) {
            Ok(stream) => {
                tcp = Some(stream);
                break;
            }
            Err(error) => last_error = Some(error),
        }
    }
    let tcp = tcp.ok_or_else(|| {
        format!(
            "IMAP connection failed: {}",
            last_error
                .map(|error| error.to_string())
                .unwrap_or_else(|| "no resolved address".to_string())
        )
    })?;
    tcp.set_read_timeout(Some(IO_TIMEOUT))
        .map_err(|error| format!("IMAP timeout setup failed: {error}"))?;
    tcp.set_write_timeout(Some(IO_TIMEOUT))
        .map_err(|error| format!("IMAP timeout setup failed: {error}"))?;

    let stream = wrap_imap_tls(&config.imap_host, tcp)?;
    let mut client = imap::Client::new(stream);
    client
        .read_greeting()
        .map_err(|error| format!("IMAP greeting failed: {error}"))?;
    let mut session = client
        .login(&config.imap_username, &config.imap_password)
        .map_err(|(error, _)| format!("IMAP login failed: {error}"))?;
    session
        .examine(&config.folder)
        .map_err(|error| format!("IMAP folder open failed: {error}"))?;

    let start_uid = config.last_uid.saturating_add(1);
    let mut uids = session
        .uid_search(format!("UID {start_uid}:*"))
        .map_err(|error| format!("IMAP search failed: {error}"))?
        .into_iter()
        .filter(|uid| *uid > config.last_uid)
        .collect::<Vec<_>>();
    uids.sort_unstable();

    let mut messages = Vec::with_capacity(uids.len());
    for uid in uids.into_iter().take(100) {
        let fetched = session
            .uid_fetch(uid.to_string(), "(UID RFC822)")
            .map_err(|error| format!("IMAP fetch UID {uid} failed: {error}"))?;
        if let Some(fetch) = fetched.iter().next() {
            if let Some(body) = fetch.body() {
                messages.push(FetchedMessage {
                    uid,
                    raw: body.to_vec(),
                });
            }
        }
    }
    let _ = session.logout();
    Ok(messages)
}

async fn ingest_message(
    pool: &SqlitePool,
    config: &MailboxConfig,
    message: FetchedMessage,
    summary: &mut SyncSummary,
) {
    let parsed = match mailparse::parse_mail(&message.raw) {
        Ok(parsed) => parsed,
        Err(error) => {
            summary.failed += 1;
            tracing::warn!(mailbox_id = %config.id, uid = message.uid, "mail parse failed: {error}");
            return;
        }
    };
    let message_id = parsed
        .get_headers()
        .get_first_value("Message-ID")
        .unwrap_or_default();
    let subject = parsed
        .get_headers()
        .get_first_value("Subject")
        .unwrap_or_default();
    let attachments = extract_xml_attachments(&parsed);
    summary.xml_attachments += attachments.len();

    for attachment in attachments {
        match ingest_attachment(pool, config, message.uid, &message_id, &subject, attachment).await
        {
            Ok("pending_confirmation") => summary.pending_confirmation += 1,
            Ok("duplicate") => summary.duplicate += 1,
            Ok("existing_source") => summary.ignored_existing_source += 1,
            Ok(_) => {}
            Err(error) => {
                summary.failed += 1;
                tracing::warn!(mailbox_id = %config.id, uid = message.uid, "XML attachment ingest failed: {error}");
            }
        }
    }
}

fn extract_xml_attachments(mail: &ParsedMail<'_>) -> Vec<XmlAttachment> {
    let mut attachments = Vec::new();
    for (index, part) in mail.parts().enumerate() {
        if !part.subparts.is_empty() {
            continue;
        }
        let disposition = part.get_content_disposition();
        let filename = disposition
            .params
            .get("filename")
            .cloned()
            .or_else(|| part.ctype.params.get("name").cloned());
        let Some(filename) = filename else { continue };
        if !filename.to_ascii_lowercase().ends_with(".xml") {
            continue;
        }
        if disposition.disposition != DispositionType::Attachment
            && disposition.disposition != DispositionType::Inline
        {
            continue;
        }
        match part.get_body_raw() {
            Ok(bytes) => attachments.push(XmlAttachment {
                index,
                filename,
                bytes,
                decode_error: None,
            }),
            Err(error) => attachments.push(XmlAttachment {
                index,
                filename,
                bytes: Vec::new(),
                decode_error: Some(format!("XML attachment decode failed: {error}")),
            }),
        }
    }
    attachments
}

async fn ingest_attachment(
    pool: &SqlitePool,
    config: &MailboxConfig,
    message_uid: u32,
    message_id: &str,
    subject: &str,
    mut attachment: XmlAttachment,
) -> Result<&'static str, String> {
    let existing_source: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT id FROM invoice_mail_imports
        WHERE company_id = ? AND mailbox_id = ? AND message_uid = ? AND attachment_index = ?
        LIMIT 1
        "#,
    )
    .bind(&config.company_id)
    .bind(&config.id)
    .bind(i64::from(message_uid))
    .bind(attachment.index as i64)
    .fetch_optional(pool)
    .await
    .map_err(|error| error.to_string())?;
    if existing_source.is_some() {
        return Ok("existing_source");
    }

    let id = uuid::Uuid::new_v4().to_string();
    let content_hash = hex::encode(Sha256::digest(&attachment.bytes));
    let attachment_error = attachment.decode_error.clone().or_else(|| {
        (attachment.bytes.len() > MAX_XML_BYTES).then(|| {
            format!(
                "XML attachment exceeds {} MiB limit",
                MAX_XML_BYTES / 1024 / 1024
            )
        })
    });
    if let Some(error) = attachment_error {
        // Retain metadata and the original digest, but do not persist oversized
        // or undecodable attachment bytes.
        attachment.bytes.clear();
        insert_import_record(
            pool,
            &id,
            config,
            message_uid,
            message_id,
            subject,
            &attachment,
            &content_hash,
            "failed",
            None,
            None,
            Some(&error),
            None,
        )
        .await?;
        return Err(error);
    }
    let duplicate: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT id FROM invoice_mail_imports
        WHERE company_id = ? AND content_sha256 = ?
          AND status IN ('pending_confirmation', 'imported')
        ORDER BY created_at LIMIT 1
        "#,
    )
    .bind(&config.company_id)
    .bind(&content_hash)
    .fetch_optional(pool)
    .await
    .map_err(|error| error.to_string())?;

    if let Some((duplicate_of,)) = duplicate {
        insert_import_record(
            pool,
            &id,
            config,
            message_uid,
            message_id,
            subject,
            &attachment,
            &content_hash,
            "duplicate",
            None,
            None,
            Some("相同 XML 附件已接收"),
            Some(&duplicate_of),
        )
        .await?;
        return Ok("duplicate");
    }

    let row = match parse_invoice_xml_attachment(&attachment.bytes) {
        Ok(row) => row,
        Err(_) => {
            insert_import_record(
                pool,
                &id,
                config,
                message_uid,
                message_id,
                subject,
                &attachment,
                &content_hash,
                "failed",
                None,
                None,
                Some("XML 解析失败"),
                None,
            )
            .await?;
            return Err("XML 解析失败".to_string());
        }
    };

    if row.status != "valid" {
        let error = if row.error.trim().is_empty() {
            "XML 发票字段校验失败".to_string()
        } else {
            row.error.clone()
        };
        insert_import_record(
            pool,
            &id,
            config,
            message_uid,
            message_id,
            subject,
            &attachment,
            &content_hash,
            "failed",
            Some(&row),
            Some(&row.invoice_number),
            Some(&error),
            None,
        )
        .await?;
        return Err(error);
    }

    let invoice_exists: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM invoices WHERE company_id = ? AND invoice_number = ? LIMIT 1",
    )
    .bind(&config.company_id)
    .bind(&row.invoice_number)
    .fetch_optional(pool)
    .await
    .map_err(|error| error.to_string())?;
    if let Some((invoice_id,)) = invoice_exists {
        insert_import_record(
            pool,
            &id,
            config,
            message_uid,
            message_id,
            subject,
            &attachment,
            &content_hash,
            "duplicate",
            Some(&row),
            Some(&row.invoice_number),
            Some("发票号已入账"),
            None,
        )
        .await?;
        return Ok("duplicate");
    }

    insert_import_record(
        pool,
        &id,
        config,
        message_uid,
        message_id,
        subject,
        &attachment,
        &content_hash,
        "pending_confirmation",
        Some(&row),
        Some(&row.invoice_number),
        None,
        None,
    )
    .await?;
    Ok("pending_confirmation")
}

#[allow(clippy::too_many_arguments)]
async fn insert_import_record(
    pool: &SqlitePool,
    id: &str,
    config: &MailboxConfig,
    message_uid: u32,
    message_id: &str,
    subject: &str,
    attachment: &XmlAttachment,
    content_hash: &str,
    status: &str,
    row: Option<&InvoiceImportRow>,
    invoice_number: Option<&str>,
    error: Option<&str>,
    duplicate_of: Option<&str>,
) -> Result<(), String> {
    let parsed_json = row
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| error.to_string())?;
    sqlx::query(
        r#"
        INSERT INTO invoice_mail_imports (
            id, company_id, mailbox_id, message_uid, message_id, subject,
            attachment_index, attachment_name, content_sha256, xml_content,
            invoice_number, parsed_json, status, error, duplicate_of
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(&config.company_id)
    .bind(&config.id)
    .bind(i64::from(message_uid))
    .bind(message_id)
    .bind(subject)
    .bind(attachment.index as i64)
    .bind(&attachment.filename)
    .bind(content_hash)
    .bind(&attachment.bytes)
    .bind(invoice_number)
    .bind(parsed_json)
    .bind(status)
    .bind(error)
    .bind(duplicate_of)
    .execute(pool)
    .await
    .map_err(|error| error.to_string())?;
    Ok(())
}

async fn list_imports(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(query): Query<ImportListQuery>,
) -> Result<Json<Value>, ApiError> {
    let status = query
        .status
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if let Some(status) = status {
        if !matches!(
            status,
            "pending_confirmation" | "imported" | "duplicate" | "failed" | "rejected"
        ) {
            return Err(api_err(StatusCode::BAD_REQUEST, "invalid import status"));
        }
    }
    let limit = query.limit.unwrap_or(100).clamp(1, 200) as i64;
    let rows = sqlx::query(
        r#"
        SELECT i.id, i.mailbox_id, m.name AS mailbox_name, i.message_uid,
               i.message_id, i.subject, i.attachment_name, i.content_sha256,
               i.invoice_number, i.parsed_json, i.status, i.error, i.invoice_id,
               i.duplicate_of, i.created_at, i.confirmed_at
        FROM invoice_mail_imports i
        JOIN invoice_mailboxes m ON m.id = i.mailbox_id AND m.company_id = i.company_id
        WHERE i.company_id = ?
          AND (? IS NULL OR i.status = ?)
          AND (? IS NULL OR i.mailbox_id = ?)
        ORDER BY i.created_at DESC
        LIMIT ?
        "#,
    )
    .bind(&user.company_id)
    .bind(status)
    .bind(status)
    .bind(query.mailbox_id.as_deref())
    .bind(query.mailbox_id.as_deref())
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(|_| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "mail import query failed",
        )
    })?;

    let mut data = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.get("id");
        let status: String = row.get("status");
        let parsed = row
            .try_get::<String, _>("parsed_json")
            .ok()
            .and_then(|value| serde_json::from_str::<Value>(&value).ok());
        data.push(json!({
            "id": id,
            "mailbox_id": row.get::<String, _>("mailbox_id"),
            "mailbox_name": row.get::<String, _>("mailbox_name"),
            "message_uid": row.get::<i64, _>("message_uid"),
            "message_id": row.try_get::<String, _>("message_id").ok(),
            "subject": row.try_get::<String, _>("subject").ok(),
            "attachment_name": row.get::<String, _>("attachment_name"),
            "content_sha256": row.get::<String, _>("content_sha256"),
            "invoice_number": row.try_get::<String, _>("invoice_number").ok(),
            "invoice": parsed,
            "status": status,
            "status_label": import_status_label(&status),
            "error": row.try_get::<String, _>("error").ok(),
            "invoice_id": row.try_get::<String, _>("invoice_id").ok(),
            "duplicate_of": row.try_get::<String, _>("duplicate_of").ok(),
            "created_at": row.try_get::<String, _>("created_at").ok(),
            "confirmed_at": row.try_get::<String, _>("confirmed_at").ok(),
            "confirm_action": if status == "pending_confirmation" { Some(json!({
                "method": "POST",
                "endpoint": format!("/api/v1/invoice-mail-imports/{id}/confirm"),
                "label": "确认入账",
                "success_message": "发票已确认入账"
            })) } else { None },
            "reject_action": if status == "pending_confirmation" { Some(json!({
                "method": "POST",
                "endpoint": format!("/api/v1/invoice-mail-imports/{id}/reject"),
                "label": "忽略",
                "success_message": "该附件已忽略"
            })) } else { None }
        }));
    }

    let stats_rows = sqlx::query(
        "SELECT status, COUNT(*) AS count FROM invoice_mail_imports WHERE company_id = ? GROUP BY status",
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "mail import stats failed"))?;
    let mut stats = serde_json::Map::new();
    for row in stats_rows {
        stats.insert(
            row.get::<String, _>("status"),
            json!(row.get::<i64, _>("count")),
        );
    }
    Ok(Json(json!({ "data": data, "stats": stats })))
}

fn import_status_label(status: &str) -> &'static str {
    match status {
        "pending_confirmation" => "待确认",
        "imported" => "已入账",
        "duplicate" => "重复",
        "failed" => "失败",
        "rejected" => "已忽略",
        _ => "未知",
    }
}

async fn confirm_import(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "confirm import failed"))?;
    let row = sqlx::query(
        "SELECT status, parsed_json FROM invoice_mail_imports WHERE id = ? AND company_id = ?",
    )
    .bind(&id)
    .bind(&user.company_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "confirm import failed"))?
    .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "mail import not found"))?;
    let status: String = row.get("status");
    if status != "pending_confirmation" {
        return Err(api_err(
            StatusCode::CONFLICT,
            "mail import is not pending confirmation",
        ));
    }
    let parsed_json: String = row
        .try_get("parsed_json")
        .map_err(|_| api_err(StatusCode::CONFLICT, "mail import has no parsed invoice"))?;
    let invoice: InvoiceImportRow = serde_json::from_str(&parsed_json).map_err(|_| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "parsed invoice is invalid",
        )
    })?;
    let invoice_id = uuid::Uuid::new_v4().to_string();
    let description = if invoice.description.trim().is_empty() {
        invoice.title.clone()
    } else {
        format!("{} - {}", invoice.title, invoice.description)
    };
    let insert = sqlx::query(
        r#"
        INSERT INTO invoices (
            id, company_id, invoice_number, invoice_type, amount, tax_amount,
            status, invoice_date, supplier, description, created_by
        ) VALUES (?, ?, ?, ?, ?, ?, 'pending', ?, ?, ?, ?)
        "#,
    )
    .bind(&invoice_id)
    .bind(&user.company_id)
    .bind(&invoice.invoice_number)
    .bind(&invoice.invoice_type)
    .bind(invoice.amount)
    .bind(invoice.tax_amount)
    .bind(&invoice.invoice_date)
    .bind(&invoice.supplier)
    .bind(description)
    .bind(&user.user_id)
    .execute(&mut *tx)
    .await;
    if let Err(error) = insert {
        if error.to_string().to_ascii_lowercase().contains("unique") {
            return Err(api_err(
                StatusCode::CONFLICT,
                "invoice number already exists",
            ));
        }
        return Err(api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "invoice create failed",
        ));
    }
    sqlx::query(
        r#"
        UPDATE invoice_mail_imports
        SET status = 'imported', invoice_id = ?, confirmed_at = CURRENT_TIMESTAMP,
            confirmed_by = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ? AND company_id = ? AND status = 'pending_confirmation'
        "#,
    )
    .bind(&invoice_id)
    .bind(&user.user_id)
    .bind(&id)
    .bind(&user.company_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "confirm import failed"))?;
    tx.commit()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "confirm import failed"))?;
    Ok(Json(json!({
        "id": id,
        "status": "imported",
        "invoice_id": invoice_id,
        "message": "发票已进入待处理账目"
    })))
}

async fn reject_import(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let result = sqlx::query(
        r#"
        UPDATE invoice_mail_imports
        SET status = 'rejected', confirmed_at = CURRENT_TIMESTAMP,
            confirmed_by = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ? AND company_id = ? AND status = 'pending_confirmation'
        "#,
    )
    .bind(&user.user_id)
    .bind(&id)
    .bind(&user.company_id)
    .execute(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "reject import failed"))?;
    if result.rows_affected() == 0 {
        return Err(api_err(
            StatusCode::CONFLICT,
            "mail import is not pending confirmation",
        ));
    }
    Ok(Json(json!({ "id": id, "status": "rejected" })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_imap_tls_rejects_invalid_sni() {
        crate::install_crypto_provider();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let tcp = TcpStream::connect(addr).expect("connect");
        let error = wrap_imap_tls(" ", tcp).expect_err("invalid SNI");
        assert!(
            error.contains("IMAP TLS hostname invalid"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn extracts_only_xml_attachments_and_decodes_base64() {
        let raw = b"From: tax@example-cn.example.invalid\r\nSubject: invoice\r\nMessage-ID: <m-1@example-cn.example.invalid>\r\nContent-Type: multipart/mixed; boundary=abc\r\n\r\n--abc\r\nContent-Type: text/plain\r\n\r\nhello\r\n--abc\r\nContent-Type: application/xml; name=invoice.xml\r\nContent-Disposition: attachment; filename=invoice.xml\r\nContent-Transfer-Encoding: base64\r\n\r\nPEludm9pY2U+PEZwaG0+MTIzPC9GcGhtPjwvSW52b2ljZT4=\r\n--abc\r\nContent-Type: application/pdf; name=invoice.pdf\r\nContent-Disposition: attachment; filename=invoice.pdf\r\n\r\nnot-a-pdf\r\n--abc--\r\n";
        let parsed = mailparse::parse_mail(raw).expect("parse MIME mail");
        let attachments = extract_xml_attachments(&parsed);
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].filename, "invoice.xml");
        assert!(attachments[0].bytes.starts_with(b"<Invoice>"));
    }

    #[test]
    fn rejects_body_xml_without_attachment_filename() {
        let raw = b"Content-Type: application/xml\r\n\r\n<Invoice/>";
        let parsed = mailparse::parse_mail(raw).expect("parse mail");
        assert!(extract_xml_attachments(&parsed).is_empty());
    }

    #[tokio::test]
    async fn simulated_fetched_mail_creates_one_pending_confirmation() {
        use base64::{Engine as _, engine::general_purpose::STANDARD};

        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("memory database");
        crate::db::run_migrations(&pool).await.expect("schema");
        sqlx::query("INSERT INTO companies (id, name) VALUES ('c-mail', 'Mail Test')")
            .execute(&pool)
            .await
            .expect("company");
        sqlx::query(
            "INSERT INTO invoice_mailboxes \
             (id, company_id, name, imap_host, imap_username, imap_password, enabled) \
             VALUES ('m-mail', 'c-mail', 'Inbox', 'imap.example.test', 'u', 'p', 1)",
        )
        .execute(&pool)
        .await
        .expect("mailbox");
        let config = load_mailbox(&pool, "c-mail", "m-mail")
            .await
            .expect("load")
            .expect("mailbox config");

        let xml = r#"<Invoice><Fpdm>044001</Fpdm><Fphm>MAIL-9001</Fphm><Kprq>2026-08-12</Kprq><Xfmc>邮箱供应商</Xfmc><Hjje>300.25</Hjje><Hjse>39.03</Hjse><Spxx>服务费</Spxx></Invoice>"#;
        let raw = format!(
            "From: tax@example-cn.example.invalid\r\nSubject: XML invoice\r\nMessage-ID: <mail-1@example-cn.example.invalid>\r\nContent-Type: multipart/mixed; boundary=invoice-boundary\r\n\r\n--invoice-boundary\r\nContent-Type: text/plain\r\n\r\nPlease find the invoice attached.\r\n--invoice-boundary\r\nContent-Type: application/xml; name=invoice.xml\r\nContent-Disposition: attachment; filename=invoice.xml\r\nContent-Transfer-Encoding: base64\r\n\r\n{}\r\n--invoice-boundary--\r\n",
            STANDARD.encode(xml.as_bytes())
        );
        let mut summary = SyncSummary::default();
        ingest_message(
            &pool,
            &config,
            FetchedMessage {
                uid: 42,
                raw: raw.into_bytes(),
            },
            &mut summary,
        )
        .await;

        assert_eq!(summary.xml_attachments, 1);
        assert_eq!(summary.pending_confirmation, 1);
        assert_eq!(summary.failed, 0);
        let queued: (String, String, String) = sqlx::query_as(
            "SELECT status, invoice_number, attachment_name \
             FROM invoice_mail_imports WHERE company_id = 'c-mail'",
        )
        .fetch_one(&pool)
        .await
        .expect("pending import");
        assert_eq!(queued.0, "pending_confirmation");
        assert_eq!(queued.1, "044001-MAIL-9001");
        assert_eq!(queued.2, "invoice.xml");
    }

    #[tokio::test]
    async fn attachment_ingest_is_idempotent_by_source_and_content() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("memory database");
        crate::db::run_migrations(&pool).await.expect("schema");
        sqlx::query("INSERT INTO users (id, email) VALUES ('u1', 'u1@example.test')")
            .execute(&pool)
            .await
            .expect("user");
        sqlx::query("INSERT INTO companies (id, name) VALUES ('c1', 'Test')")
            .execute(&pool)
            .await
            .expect("company");
        sqlx::query(
            "INSERT INTO invoice_mailboxes (id, company_id, name, imap_host, imap_username, imap_password) VALUES ('m1', 'c1', 'Inbox', 'imap.example.test', 'u', 'p')",
        )
        .execute(&pool)
        .await
        .expect("mailbox");
        let config = load_mailbox(&pool, "c1", "m1")
            .await
            .expect("load")
            .expect("mailbox config");
        let xml = r#"<Invoice><Fpdm>044001</Fpdm><Fphm>9001</Fphm><Kprq>2026-08-12</Kprq><Xfmc>供应商</Xfmc><Hjje>300.25</Hjje><Hjse>39.03</Hjse><Spxx>服务费</Spxx></Invoice>"#.as_bytes();

        let first = ingest_attachment(
            &pool,
            &config,
            10,
            "<one>",
            "invoice",
            XmlAttachment {
                index: 1,
                filename: "invoice.xml".to_string(),
                bytes: xml.to_vec(),
                decode_error: None,
            },
        )
        .await
        .expect("first ingest");
        assert_eq!(first, "pending_confirmation");

        let repeated_source = ingest_attachment(
            &pool,
            &config,
            10,
            "<one>",
            "invoice",
            XmlAttachment {
                index: 1,
                filename: "invoice.xml".to_string(),
                bytes: xml.to_vec(),
                decode_error: None,
            },
        )
        .await
        .expect("repeat source");
        assert_eq!(repeated_source, "existing_source");

        let forwarded_copy = ingest_attachment(
            &pool,
            &config,
            11,
            "<two>",
            "forwarded invoice",
            XmlAttachment {
                index: 1,
                filename: "copy.xml".to_string(),
                bytes: xml.to_vec(),
                decode_error: None,
            },
        )
        .await
        .expect("forwarded copy");
        assert_eq!(forwarded_copy, "duplicate");

        let oversized = ingest_attachment(
            &pool,
            &config,
            12,
            "<three>",
            "oversized invoice",
            XmlAttachment {
                index: 1,
                filename: "oversized.xml".to_string(),
                bytes: vec![b'x'; MAX_XML_BYTES + 1],
                decode_error: None,
            },
        )
        .await
        .expect_err("oversized XML must fail");
        assert!(oversized.contains("exceeds"));

        let statuses: Vec<(String,)> =
            sqlx::query_as("SELECT status FROM invoice_mail_imports ORDER BY message_uid")
                .fetch_all(&pool)
                .await
                .expect("statuses");
        assert_eq!(
            statuses,
            vec![
                ("pending_confirmation".to_string(),),
                ("duplicate".to_string(),),
                ("failed".to_string(),)
            ]
        );
    }
}
