//! Managed W-Mail transport. The existing tenant email log is the operation
//! ledger; message bodies and platform credentials never enter this ledger.
use crate::api::error::{ApiError, api_err};
use crate::auth::AuthUser;
use axum::{Json, http::StatusCode};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};
use std::time::Duration;

struct Config {
    origin: Url,
    project: String,
    mailbox: String,
    key: String,
}

impl Config {
    fn load() -> Option<Self> {
        let [origin, project, mailbox, key] = ["URL", "PROJECT_ID", "MAILBOX_ID", "API_KEY"]
            .map(|name| std::env::var(format!("W_ERP_MAIL_{name}")).unwrap_or_default());
        let origin = Url::parse(&origin).ok()?;
        let allowed_origin = origin.scheme() == "https"
            && origin.host_str() == Some("wmail.myfastools.com")
            || origin.scheme() == "http"
                && matches!(origin.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"));
        if !allowed_origin
            || !origin.username().is_empty()
            || origin.password().is_some()
            || origin.path() != "/"
            || origin.query().is_some()
            || origin.fragment().is_some()
            || project.is_empty()
            || project != std::env::var("W_ID_CLIENT_ID").unwrap_or_default()
            || !hex_id(&mailbox)
            || !key.starts_with("wid_sk_")
            || key.len() > 8192
            || key.chars().any(char::is_whitespace)
        {
            return None;
        }
        Some(Self {
            origin,
            project,
            mailbox,
            key,
        })
    }

    fn matches(&self, delivery: &Delivery) -> bool {
        self.origin.as_str() == delivery.origin
            && self.project == delivery.project
            && self.mailbox == delivery.mailbox
    }
}

pub fn available() -> bool {
    Config::load().is_some()
}

fn operation_hash(user: &AuthUser, key: &str) -> String {
    hex::encode(Sha256::digest(format!(
        "{}\0{}\0{}",
        user.company_id, user.user_id, key
    )))
}

pub async fn recorded(pool: &SqlitePool, user: &AuthUser, body: &Value) -> Result<bool, ApiError> {
    let Some(key) = body["operation_id"].as_str() else {
        return Ok(false);
    };
    let hash = operation_hash(user, key);
    sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM email_logs WHERE id=? AND company_id=? AND mail_delivery IS NOT NULL",
    )
    .bind(&hash[..32])
    .bind(&user.company_id)
    .fetch_one(pool)
    .await
    .map(|n| n == 1)
    .map_err(|_| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "mail_operation_lookup_failed",
        )
    })
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Delivery {
    operation: String,
    fingerprint: String,
    origin: String,
    project: String,
    mailbox: String,
    posted: bool,
    checks: i64,
    expires_at: i64,
    next_check: Option<i64>,
    job_id: Option<String>,
    provider_id: Option<String>,
}

fn hex_id(value: &str) -> bool {
    value.len() == 32
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn provider_id(value: &str) -> bool {
    let Some((id, domain)) = value
        .strip_prefix('<')
        .and_then(|v| v.strip_suffix('>'))
        .and_then(|v| v.split_once('@'))
    else {
        return false;
    };
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
        && matches!(
            domain,
            "agentmail.myfastools.com" | "botmail.myfastools.com"
        )
}

fn client() -> Result<Client, ApiError> {
    Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .build()
        .map_err(|_| {
            api_err(
                StatusCode::SERVICE_UNAVAILABLE,
                "mail_transport_unavailable",
            )
        })
}

pub async fn send(
    pool: &SqlitePool,
    user: &AuthUser,
    body: &Value,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let config = Config::load()
        .ok_or_else(|| api_err(StatusCode::SERVICE_UNAVAILABLE, "w_mail_unavailable"))?;
    let request_key = body["operation_id"]
        .as_str()
        .filter(|v| {
            v.len() >= 8
                && v.len() <= 128
                && v.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
        })
        .ok_or_else(|| api_err(StatusCode::BAD_REQUEST, "operation_id_required"))?;
    let to = body["to"]
        .as_str()
        .unwrap_or_default()
        .trim()
        .to_lowercase();
    let subject = body["subject"].as_str().unwrap_or_default();
    let html = body["body"].as_str().unwrap_or_default();
    let text = body["text"]
        .as_str()
        .unwrap_or(if html.is_empty() { subject } else { "" });
    if !to.contains('@')
        || to.len() > 254
        || to.chars().any(char::is_whitespace)
        || subject.is_empty()
        || subject.len() > 998
        || subject.contains(['\r', '\n', '\0'])
        || html.len() > 256 * 1024
        || text.len() > 256 * 1024
    {
        return Err(api_err(StatusCode::BAD_REQUEST, "invalid_mail_content"));
    }
    let key_hash = operation_hash(user, request_key);
    let id = &key_hash[..32];
    let operation = format!("erp_{key_hash}");
    let mut payload = json!({"to":[to],"subject":subject,"context":{"eventId":operation,"eventType":"erp.mail.send","attemptId":operation}});
    if !html.is_empty() {
        payload["html"] = json!(html);
    }
    if !text.is_empty() {
        payload["text"] = json!(text);
    }
    let fingerprint = hex::encode(Sha256::digest(
        serde_json::to_vec(&json!([
            payload,
            body.get("template"),
            body.get("entity_type"),
            body.get("entity_id")
        ]))
        .unwrap(),
    ));
    let now = chrono::Utc::now().timestamp();
    let delivery = Delivery {
        operation,
        fingerprint,
        origin: config.origin.to_string(),
        project: config.project.clone(),
        mailbox: config.mailbox.clone(),
        posted: true,
        checks: 1,
        expires_at: now + 2 * 86400,
        next_check: Some(now + 60),
        job_id: None,
        provider_id: None,
    };
    let serialized = serde_json::to_string(&delivery).unwrap();
    // A durable unknown marker precedes the only POST. A restored marker or a
    // lost HTTP acknowledgement can only produce an original-key GET later.
    let inserted = sqlx::query("INSERT OR IGNORE INTO email_logs (id,company_id,to_address,subject,template,entity_type,entity_id,status,error,sent_by,mail_delivery) VALUES (?,?,?,?,?,?,?,'unknown','submission_unknown',?,?)")
        .bind(id).bind(&user.company_id).bind(&to).bind(subject)
        .bind(body["template"].as_str().unwrap_or("custom"))
        .bind(body["entity_type"].as_str().unwrap_or_default())
        .bind(body["entity_id"].as_str().unwrap_or_default()).bind(&user.user_id).bind(&serialized)
        .execute(pool).await.map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR,"mail_operation_store_failed"))?.rows_affected();
    if inserted == 0 {
        let stored: Option<String> =
            sqlx::query_scalar("SELECT mail_delivery FROM email_logs WHERE id=? AND company_id=?")
                .bind(id)
                .bind(&user.company_id)
                .fetch_optional(pool)
                .await
                .map_err(|_| {
                    api_err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "mail_operation_lookup_failed",
                    )
                })?
                .flatten();
        let stored: Delivery = stored
            .and_then(|v| serde_json::from_str(&v).ok())
            .ok_or_else(|| api_err(StatusCode::CONFLICT, "mail_operation_conflict"))?;
        if stored.fingerprint != delivery.fingerprint {
            return Err(api_err(StatusCode::CONFLICT, "mail_operation_conflict"));
        }
        return current(pool, &user.company_id, id).await;
    }
    let http = client()?;
    let url = config
        .origin
        .join(&format!("api/v1/mailboxes/{}/sends", config.mailbox))
        .unwrap();
    let response = http
        .post(url)
        .bearer_auth(&config.key)
        .header("X-W-Project-Id", &config.project)
        .header("Idempotency-Key", &delivery.operation)
        .json(&payload)
        .send()
        .await;
    finish(
        pool,
        id,
        &user.company_id,
        &to,
        &serialized,
        delivery,
        response,
    )
    .await?;
    current(pool, &user.company_id, id).await
}

async fn current(
    pool: &SqlitePool,
    company: &str,
    id: &str,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let row = sqlx::query(
        "SELECT status,error,mail_delivery FROM email_logs WHERE id=? AND company_id=?",
    )
    .bind(id)
    .bind(company)
    .fetch_one(pool)
    .await
    .map_err(|_| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "mail_operation_lookup_failed",
        )
    })?;
    let status: String = row.get("status");
    let details: Value =
        serde_json::from_str(row.get::<&str, _>("mail_delivery")).unwrap_or(Value::Null);
    let code = if status == "failed" {
        StatusCode::BAD_GATEWAY
    } else if status == "delivered" {
        StatusCode::OK
    } else {
        StatusCode::ACCEPTED
    };
    Ok((
        code,
        Json(
            json!({"success":matches!(status.as_str(),"provider_accepted"|"delivered"),"status":status,"delivered":status=="delivered","log_id":id,"job_id":details["jobId"],"provider_id":details["providerId"],"error":row.get::<Option<String>,_>("error"),"message":"投递状态以 status 为准；上游受理不等于收件服务器已收到。"}),
        ),
    ))
}

async fn read_receipt(response: reqwest::Response) -> Option<Value> {
    if !response.status().is_success() {
        return None;
    }
    let mut response = response;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.ok()? {
        if bytes.len() + chunk.len() > 32 * 1024 {
            return None;
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).ok()
}

async fn finish(
    pool: &SqlitePool,
    id: &str,
    company: &str,
    recipient: &str,
    before: &str,
    mut delivery: Delivery,
    response: Result<reqwest::Response, reqwest::Error>,
) -> Result<(), ApiError> {
    let receipt = match response {
        Ok(r) => read_receipt(r).await,
        Err(_) => None,
    };
    let (mut status, mut error) = ("unknown", Some("submission_unknown"));
    if let Some(r) = receipt.filter(|r| {
        r["mailboxId"].as_str() == Some(&delivery.mailbox)
            && r["actor"]["kind"] == "project"
            && r["actor"]["projectId"].as_str() == Some(&delivery.project)
            && r["context"]["eventId"].as_str() == Some(&delivery.operation)
            && r["context"]["attemptId"].as_str() == Some(&delivery.operation)
            && r["context"]["eventType"] == "erp.mail.send"
            && r["id"].as_str().is_some_and(hex_id)
            && delivery
                .job_id
                .as_deref()
                .is_none_or(|id| r["id"].as_str() == Some(id))
    }) {
        delivery.job_id = r["id"].as_str().map(str::to_owned);
        delivery.provider_id = r["providerId"]
            .as_str()
            .filter(|v| provider_id(v))
            .map(str::to_owned)
            .or(delivery.provider_id);
        let recipient_status = r["recipients"][recipient]["status"].as_str();
        if recipient_status == Some("delivered") {
            status = "delivered";
            error = None;
        } else if matches!(
            recipient_status,
            Some("bounced" | "rejected" | "dropped" | "complained")
        ) || matches!(r["status"].as_str(), Some("failed" | "cancelled"))
        {
            status = "failed";
            error = Some("recipient_delivery_failed");
        } else if r["status"] == "submitted" {
            status = "provider_accepted";
            error = None;
        } else if r["status"] == "queued" {
            status = "queued";
            error = None;
        } else if r["status"] == "submitting" {
            status = "submitting";
            error = None;
        }
    }
    let now = chrono::Utc::now().timestamp();
    if !matches!(
        status,
        "queued" | "submitting" | "unknown" | "provider_accepted"
    ) || delivery.checks >= 6
        || delivery.expires_at <= now
    {
        delivery.next_check = None;
    } else {
        delivery.next_check = Some((now + 60).min(delivery.expires_at));
    }
    let after = serde_json::to_string(&delivery).unwrap();
    let changed=sqlx::query("UPDATE email_logs SET status=?,error=?,mail_delivery=? WHERE id=? AND company_id=? AND mail_delivery=?")
        .bind(status).bind(error).bind(after).bind(id).bind(company).bind(before).execute(pool).await
        .map_err(|_|api_err(StatusCode::INTERNAL_SERVER_ERROR,"mail_receipt_store_failed"))?.rows_affected();
    if changed == 1 {
        tracing::info!(event_id=%delivery.operation,job_id=?delivery.job_id,provider_id=?delivery.provider_id,status,code=error.unwrap_or("ok"),"mail_delivery_state");
    }
    Ok(())
}

/// Only GET reconciliation; no historical row can be turned into a POST job.
pub fn spawn_reconciliation(pool: SqlitePool) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(30));
        loop {
            ticker.tick().await;
            if reconcile(&pool).await.is_err() {
                tracing::warn!(code = "mail_reconciliation_failed", "mail_delivery_state");
            }
        }
    });
}

async fn reconcile(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let Some(config) = Config::load() else {
        return Ok(());
    };
    let Ok(http) = client() else {
        return Ok(());
    };
    let rows=sqlx::query("SELECT id,company_id,to_address,mail_delivery FROM email_logs WHERE mail_delivery IS NOT NULL AND json_extract(mail_delivery,'$.nextCheck')<=unixepoch() ORDER BY json_extract(mail_delivery,'$.nextCheck') LIMIT 10").fetch_all(pool).await?;
    for row in rows {
        let before: String = row.get("mail_delivery");
        let Ok(mut delivery) = serde_json::from_str::<Delivery>(&before) else {
            continue;
        };
        let id: String = row.get("id");
        let company: String = row.get("company_id");
        let recipient: String = row.get("to_address");
        if delivery.expires_at <= chrono::Utc::now().timestamp() || delivery.checks >= 6 {
            delivery.next_check = None;
            sqlx::query("UPDATE email_logs SET mail_delivery=? WHERE id=? AND company_id=? AND mail_delivery=?").bind(serde_json::to_string(&delivery).unwrap()).bind(&id).bind(&company).bind(&before).execute(pool).await?;
            tracing::info!(event_id=%delivery.operation,code="receipt_deadline_elapsed","mail_delivery_state");
            continue;
        }
        // Configuration changes never redirect an old operation or its key.
        if !config.matches(&delivery) {
            continue;
        }
        delivery.checks += 1;
        delivery.next_check = Some((chrono::Utc::now().timestamp() + 60).min(delivery.expires_at));
        let claimed = serde_json::to_string(&delivery).unwrap();
        let changed = sqlx::query(
            "UPDATE email_logs SET mail_delivery=? WHERE id=? AND company_id=? AND mail_delivery=?",
        )
        .bind(&claimed)
        .bind(&id)
        .bind(&company)
        .bind(&before)
        .execute(pool)
        .await?
        .rows_affected();
        if changed != 1 {
            continue;
        }
        let url = config
            .origin
            .join(&format!("api/v1/mailboxes/{}/sends", config.mailbox))
            .unwrap();
        let response = http
            .get(url)
            .bearer_auth(&config.key)
            .header("X-W-Project-Id", &config.project)
            .header("Idempotency-Key", &delivery.operation)
            .send()
            .await;
        let _ = finish(
            pool, &id, &company, &recipient, &claimed, delivery, response,
        )
        .await;
    }
    Ok(())
}
