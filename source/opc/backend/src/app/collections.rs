//! Automated accounts-receivable collection workflow (issue #25).
//!
//! The billing scheduler creates at most one due collection level per invoice
//! per cycle. Delivery idempotency is enforced by the database unique key on
//! `(company_id, invoice_id, level_code, channel_id)`.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CollectionSettings {
    pub enabled: bool,
    pub friendly_days: i64,
    pub formal_days: i64,
    pub final_days: i64,
    pub bad_debt_days: i64,
}

impl Default for CollectionSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            friendly_days: 1,
            formal_days: 7,
            final_days: 30,
            bad_debt_days: 60,
        }
    }
}

impl CollectionSettings {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.friendly_days < 1
            || self.friendly_days >= self.formal_days
            || self.formal_days >= self.final_days
            || self.final_days >= self.bad_debt_days
        {
            return Err("collection days must be positive and strictly increasing");
        }
        if self.bad_debt_days > 3650 {
            return Err("collection days must not exceed 3650");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CollectionRunSummary {
    pub queued: usize,
    pub sent: usize,
    pub failed: usize,
}

#[derive(Debug, Clone)]
struct OutstandingInvoice {
    id: String,
    company_id: String,
    invoice_number: String,
    client_id: Option<String>,
    client_name: String,
    amount_due: f64,
    due_date: String,
    audit_user_id: Option<String>,
}

#[derive(Debug, Clone, Copy)]
struct DueLevel<'a> {
    code: &'a str,
    label: &'a str,
    threshold_days: i64,
}

#[derive(Debug, Clone)]
pub struct PendingDigestDelivery {
    pub id: String,
    pub company_id: String,
    pub invoice_id: String,
    pub invoice_number: String,
    pub client_name: String,
    pub amount_due: f64,
    pub days_overdue: i64,
    pub level_code: String,
    pub level_label: String,
    pub channel_id: String,
    pub channel_name: String,
    pub audit_user_id: Option<String>,
}

pub async fn load_settings(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<CollectionSettings, sqlx::Error> {
    let row: Option<(i64, i64, i64, i64, i64)> = sqlx::query_as(
        "SELECT enabled, friendly_days, formal_days, final_days, bad_debt_days \
         FROM collection_settings WHERE company_id = ?",
    )
    .bind(company_id)
    .fetch_optional(pool)
    .await?;

    Ok(row
        .map(
            |(enabled, friendly_days, formal_days, final_days, bad_debt_days)| CollectionSettings {
                enabled: enabled != 0,
                friendly_days,
                formal_days,
                final_days,
                bad_debt_days,
            },
        )
        .unwrap_or_default())
}

pub async fn save_settings(
    pool: &SqlitePool,
    company_id: &str,
    settings: &CollectionSettings,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO collection_settings (
            company_id, enabled, friendly_days, formal_days, final_days, bad_debt_days, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
        ON CONFLICT(company_id) DO UPDATE SET
            enabled = excluded.enabled,
            friendly_days = excluded.friendly_days,
            formal_days = excluded.formal_days,
            final_days = excluded.final_days,
            bad_debt_days = excluded.bad_debt_days,
            updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(company_id)
    .bind(i64::from(settings.enabled))
    .bind(settings.friendly_days)
    .bind(settings.formal_days)
    .bind(settings.final_days)
    .bind(settings.bad_debt_days)
    .execute(pool)
    .await?;
    Ok(())
}

/// Create and deliver all collection events currently due.
///
/// Instant channels are sent during the billing cycle. Digest channels are
/// queued for the existing 09:00 notification dispatcher.
pub async fn run_collection_cycle(pool: &SqlitePool) -> Result<CollectionRunSummary, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
            i.id,
            i.company_id,
            i.invoice_number,
            i.client_id,
            COALESCE(NULLIF(c.name, ''), NULLIF(i.supplier, ''), '客户') AS client_name,
            MAX(0, i.amount - COALESCE((
                SELECT SUM(p.amount)
                FROM payments p
                WHERE p.company_id = i.company_id AND p.invoice_id = i.id AND p.status IN ('completed', 'paid')
            ), 0)) AS amount_due,
            i.due_date,
            COALESCE(
                NULLIF(i.created_by, ''),
                (SELECT cm.user_id FROM company_members cm
                 WHERE cm.company_id = i.company_id
                 ORDER BY CASE cm.role WHEN 'owner' THEN 0 WHEN 'admin' THEN 1 ELSE 2 END,
                          cm.created_at ASC
                 LIMIT 1)
            ) AS audit_user_id
        FROM invoices i
        LEFT JOIN clients c ON c.id = i.client_id AND c.company_id = i.company_id
        WHERE i.invoice_type = 'receivable'
          AND i.status IN ('overdue', 'partially_paid')
          AND i.due_date IS NOT NULL
          AND TRIM(i.due_date) != ''
          AND date(i.due_date) < date('now', 'localtime')
        "#,
    )
    .fetch_all(pool)
    .await?;

    let today = chrono::Local::now().date_naive();
    let mut summary = CollectionRunSummary::default();

    for row in rows {
        let invoice = OutstandingInvoice {
            id: row.try_get("id").unwrap_or_default(),
            company_id: row.try_get("company_id").unwrap_or_default(),
            invoice_number: row.try_get("invoice_number").unwrap_or_default(),
            client_id: row.try_get("client_id").ok().flatten(),
            client_name: row
                .try_get("client_name")
                .unwrap_or_else(|_| "客户".to_string()),
            amount_due: row.try_get("amount_due").unwrap_or(0.0),
            due_date: row.try_get("due_date").unwrap_or_default(),
            audit_user_id: row.try_get("audit_user_id").ok().flatten(),
        };
        if invoice.amount_due <= 0.0 {
            continue;
        }

        let settings = load_settings(pool, &invoice.company_id).await?;
        if !settings.enabled {
            continue;
        }
        let Some(due_date) = parse_date(&invoice.due_date) else {
            continue;
        };
        let days_overdue = (today - due_date).num_days();
        let Some(level) = due_level(&settings, days_overdue) else {
            continue;
        };

        let channels: Vec<(String, String, String, String)> = sqlx::query_as(
            "SELECT id, COALESCE(NULLIF(name, ''), '通知渠道'), channel_url, mode \
             FROM notification_channels \
             WHERE company_id = ? AND enabled = 1 AND mode IN ('instant', 'digest')",
        )
        .bind(&invoice.company_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        for (channel_id, channel_name, channel_url, mode) in channels {
            let delivery_id = ensure_delivery(
                pool,
                &invoice,
                level,
                days_overdue,
                &channel_id,
                &channel_name,
                &mode,
            )
            .await?;
            let Some(delivery_id) = delivery_id else {
                continue;
            };
            summary.queued += 1;

            if mode == "instant" {
                let title = collection_title(level, &invoice.invoice_number);
                let body = collection_message(&invoice, level, days_overdue);
                match crate::app::notifications::send_message(&channel_url, &title, &body).await {
                    Ok(()) => {
                        finish_delivery(
                            pool,
                            &delivery_id,
                            "sent",
                            None,
                            &invoice.company_id,
                            &invoice.id,
                            &invoice.invoice_number,
                            level.code,
                            level.label,
                            &channel_id,
                            &channel_name,
                            invoice.audit_user_id.as_deref(),
                        )
                        .await?;
                        summary.sent += 1;
                    }
                    Err(error) => {
                        finish_delivery(
                            pool,
                            &delivery_id,
                            "failed",
                            Some(&error),
                            &invoice.company_id,
                            &invoice.id,
                            &invoice.invoice_number,
                            level.code,
                            level.label,
                            &channel_id,
                            &channel_name,
                            invoice.audit_user_id.as_deref(),
                        )
                        .await?;
                        summary.failed += 1;
                    }
                }
            }
        }
    }

    Ok(summary)
}

async fn ensure_delivery(
    pool: &SqlitePool,
    invoice: &OutstandingInvoice,
    level: DueLevel<'_>,
    days_overdue: i64,
    channel_id: &str,
    channel_name: &str,
    mode: &str,
) -> Result<Option<String>, sqlx::Error> {
    let candidate_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO collection_deliveries (
            id, company_id, invoice_id, client_id, invoice_number, client_name,
            amount_due, days_overdue, level_code, level_label, threshold_days,
            channel_id, channel_name, channel_mode, status, audit_user_id
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'pending', ?)
        "#,
    )
    .bind(&candidate_id)
    .bind(&invoice.company_id)
    .bind(&invoice.id)
    .bind(&invoice.client_id)
    .bind(&invoice.invoice_number)
    .bind(&invoice.client_name)
    .bind(invoice.amount_due)
    .bind(days_overdue)
    .bind(level.code)
    .bind(level.label)
    .bind(level.threshold_days)
    .bind(channel_id)
    .bind(channel_name)
    .bind(mode)
    .bind(&invoice.audit_user_id)
    .execute(pool)
    .await?;

    let existing: Option<(String, String)> = sqlx::query_as(
        "SELECT id, status FROM collection_deliveries \
         WHERE company_id = ? AND invoice_id = ? AND level_code = ? AND channel_id = ?",
    )
    .bind(&invoice.company_id)
    .bind(&invoice.id)
    .bind(level.code)
    .bind(channel_id)
    .fetch_optional(pool)
    .await?;
    let Some((id, status)) = existing else {
        return Ok(None);
    };

    if mode == "digest" {
        return Ok((status == "pending" || status == "failed").then_some(id));
    }
    let claim = sqlx::query(
        "UPDATE collection_deliveries \
         SET status = 'dispatching', attempt_count = attempt_count + 1, \
             last_attempt_at = CURRENT_TIMESTAMP, error = NULL \
         WHERE id = ? AND status IN ('pending', 'failed')",
    )
    .bind(&id)
    .execute(pool)
    .await?;
    Ok((claim.rows_affected() == 1).then_some(id))
}

pub async fn claim_digest_deliveries(
    pool: &SqlitePool,
    company_id: &str,
    channel_id: &str,
) -> Result<Vec<PendingDigestDelivery>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, company_id, invoice_id, invoice_number, client_name,
               amount_due, days_overdue, level_code, level_label,
               channel_id, channel_name, audit_user_id
        FROM collection_deliveries
        WHERE company_id = ? AND channel_id = ? AND channel_mode = 'digest'
          AND status IN ('pending', 'failed')
          AND NOT EXISTS (
              SELECT 1 FROM collection_settings settings
              WHERE settings.company_id = collection_deliveries.company_id
                AND settings.enabled = 0
          )
        ORDER BY threshold_days DESC, created_at ASC
        LIMIT 100
        "#,
    )
    .bind(company_id)
    .bind(channel_id)
    .fetch_all(pool)
    .await?;

    let mut claimed = Vec::new();
    for row in rows {
        let id: String = row.try_get("id").unwrap_or_default();
        let result = sqlx::query(
            "UPDATE collection_deliveries \
             SET status = 'dispatching', attempt_count = attempt_count + 1, \
                 last_attempt_at = CURRENT_TIMESTAMP, error = NULL \
             WHERE id = ? AND status IN ('pending', 'failed')",
        )
        .bind(&id)
        .execute(pool)
        .await?;
        if result.rows_affected() != 1 {
            continue;
        }
        claimed.push(PendingDigestDelivery {
            id,
            company_id: row.try_get("company_id").unwrap_or_default(),
            invoice_id: row.try_get("invoice_id").unwrap_or_default(),
            invoice_number: row.try_get("invoice_number").unwrap_or_default(),
            client_name: row
                .try_get("client_name")
                .unwrap_or_else(|_| "客户".to_string()),
            amount_due: row.try_get("amount_due").unwrap_or(0.0),
            days_overdue: row.try_get("days_overdue").unwrap_or(0),
            level_code: row.try_get("level_code").unwrap_or_default(),
            level_label: row.try_get("level_label").unwrap_or_default(),
            channel_id: row.try_get("channel_id").unwrap_or_default(),
            channel_name: row.try_get("channel_name").unwrap_or_default(),
            audit_user_id: row.try_get("audit_user_id").ok().flatten(),
        });
    }
    Ok(claimed)
}

pub fn format_digest_section(deliveries: &[PendingDigestDelivery]) -> String {
    if deliveries.is_empty() {
        return String::new();
    }
    let mut lines = vec![format!("自动催收 {} 条:", deliveries.len())];
    for item in deliveries.iter().take(10) {
        lines.push(format!(
            "• {} · {} · ¥{:.2} · 逾期{}天 · {}",
            item.invoice_number,
            item.client_name,
            item.amount_due,
            item.days_overdue,
            item.level_label
        ));
    }
    if deliveries.len() > 10 {
        lines.push(format!("等 {} 条", deliveries.len() - 10));
    }
    lines.join("\n")
}

pub async fn finish_digest_deliveries(
    pool: &SqlitePool,
    deliveries: &[PendingDigestDelivery],
    error: Option<&str>,
) -> Result<(), sqlx::Error> {
    let status = if error.is_some() { "failed" } else { "sent" };
    for item in deliveries {
        finish_delivery(
            pool,
            &item.id,
            status,
            error,
            &item.company_id,
            &item.invoice_id,
            &item.invoice_number,
            &item.level_code,
            &item.level_label,
            &item.channel_id,
            &item.channel_name,
            item.audit_user_id.as_deref(),
        )
        .await?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn finish_delivery(
    pool: &SqlitePool,
    delivery_id: &str,
    status: &str,
    error: Option<&str>,
    company_id: &str,
    invoice_id: &str,
    invoice_number: &str,
    level_code: &str,
    level_label: &str,
    channel_id: &str,
    channel_name: &str,
    audit_user_id: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE collection_deliveries \
         SET status = ?, error = ?, sent_at = CASE WHEN ? = 'sent' THEN CURRENT_TIMESTAMP ELSE sent_at END, \
             updated_at = CURRENT_TIMESTAMP WHERE id = ?",
    )
    .bind(status)
    .bind(error)
    .bind(status)
    .bind(delivery_id)
    .execute(pool)
    .await?;

    if let Some(user_id) = audit_user_id.filter(|value| !value.is_empty()) {
        let details = json!({
            "event": "automated_collection",
            "invoice_number": invoice_number,
            "level": level_code,
            "level_label": level_label,
            "channel_id": channel_id,
            "channel_name": channel_name,
            "result": status,
            "error": error,
        })
        .to_string();
        sqlx::query(
            "INSERT INTO operation_logs \
             (id, company_id, user_id, action, entity_type, entity_id, details) \
             VALUES (?, ?, ?, 'create', 'collection_deliveries', ?, ?)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(company_id)
        .bind(user_id)
        .bind(invoice_id)
        .bind(details)
        .execute(pool)
        .await?;
    }
    Ok(())
}

fn due_level(settings: &CollectionSettings, days_overdue: i64) -> Option<DueLevel<'static>> {
    [
        DueLevel {
            code: "friendly",
            label: "友好提醒",
            threshold_days: settings.friendly_days,
        },
        DueLevel {
            code: "formal",
            label: "正式催款",
            threshold_days: settings.formal_days,
        },
        DueLevel {
            code: "final",
            label: "最后通知",
            threshold_days: settings.final_days,
        },
        DueLevel {
            code: "bad_debt",
            label: "坏账风险",
            threshold_days: settings.bad_debt_days,
        },
    ]
    .into_iter()
    .filter(|level| days_overdue >= level.threshold_days)
    .max_by_key(|level| level.threshold_days)
}

fn collection_title(level: DueLevel<'_>, invoice_number: &str) -> String {
    format!("{}：发票 {}", level.label, invoice_number)
}

fn collection_message(
    invoice: &OutstandingInvoice,
    level: DueLevel<'_>,
    days_overdue: i64,
) -> String {
    let instruction = match level.code {
        "friendly" => "请友好提醒客户确认付款安排。",
        "formal" => "请正式跟进客户并确认明确回款日期。",
        "final" => "请发送最后付款通知并升级人工跟进。",
        _ => "请标记坏账风险并立即进行人工评估。",
    };
    format!(
        "客户: {}\n发票号: {}\n待收金额: ¥{:.2}\n到期日: {}\n逾期天数: {}\n{}\n查看: https://opc.myfastools.com/payments",
        invoice.client_name,
        invoice.invoice_number,
        invoice.amount_due,
        invoice.due_date,
        days_overdue,
        instruction
    )
}

fn parse_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(&value.chars().take(10).collect::<String>(), "%Y-%m-%d").ok()
}

pub async fn history_for_invoice(
    pool: &SqlitePool,
    company_id: &str,
    invoice_id: &str,
) -> Result<Vec<Value>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, level_code, level_label, threshold_days, days_overdue,
               channel_id, channel_name, channel_mode, status, attempt_count,
               error, created_at, last_attempt_at, sent_at
        FROM collection_deliveries
        WHERE company_id = ? AND invoice_id = ?
        ORDER BY threshold_days DESC, created_at DESC
        "#,
    )
    .bind(company_id)
    .bind(invoice_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let status: String = row.try_get("status").unwrap_or_default();
            json!({
                "id": row.try_get::<String, _>("id").unwrap_or_default(),
                "level": row.try_get::<String, _>("level_code").unwrap_or_default(),
                "level_label": row.try_get::<String, _>("level_label").unwrap_or_default(),
                "threshold_days": row.try_get::<i64, _>("threshold_days").unwrap_or(0),
                "days_overdue": row.try_get::<i64, _>("days_overdue").unwrap_or(0),
                "channel_id": row.try_get::<String, _>("channel_id").unwrap_or_default(),
                "channel_name": row.try_get::<String, _>("channel_name").unwrap_or_default(),
                "channel_mode": row.try_get::<String, _>("channel_mode").unwrap_or_default(),
                "status": status,
                "status_label": delivery_status_label(&status),
                "attempt_count": row.try_get::<i64, _>("attempt_count").unwrap_or(0),
                "error": row.try_get::<Option<String>, _>("error").unwrap_or(None),
                "created_at": row.try_get::<Option<String>, _>("created_at").unwrap_or(None),
                "last_attempt_at": row.try_get::<Option<String>, _>("last_attempt_at").unwrap_or(None),
                "sent_at": row.try_get::<Option<String>, _>("sent_at").unwrap_or(None),
            })
        })
        .collect())
}

pub async fn credit_profiles(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<HashMap<String, Value>, sqlx::Error> {
    #[derive(Default)]
    struct CreditAccumulator {
        collection_events: i64,
        escalation_penalty: i64,
        payment_delays: Vec<f64>,
    }

    let mut profiles: HashMap<String, CreditAccumulator> = HashMap::new();
    let escalation_rows = sqlx::query(
        r#"
        SELECT client_id, invoice_id, MAX(threshold_days) AS max_threshold,
               COUNT(DISTINCT level_code) AS event_count
        FROM collection_deliveries
        WHERE company_id = ? AND client_id IS NOT NULL
          AND status = 'sent'
        GROUP BY client_id, invoice_id
        "#,
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;
    for row in escalation_rows {
        let client_id: String = row.try_get("client_id").unwrap_or_default();
        if client_id.is_empty() {
            continue;
        }
        let threshold: i64 = row.try_get("max_threshold").unwrap_or(0);
        let event_count: i64 = row.try_get("event_count").unwrap_or(0);
        let acc = profiles.entry(client_id).or_default();
        acc.collection_events += event_count;
        acc.escalation_penalty += match threshold {
            0..=1 => 2,
            2..=7 => 8,
            8..=30 => 20,
            _ => 40,
        };
    }

    let payment_rows = sqlx::query(
        r#"
        SELECT i.client_id,
               MAX(0, julianday(MAX(p.payment_date)) - julianday(i.due_date)) AS delay_days
        FROM invoices i
        JOIN payments p ON p.invoice_id = i.id AND p.company_id = i.company_id
        WHERE i.company_id = ? AND i.client_id IS NOT NULL
          AND i.invoice_type = 'receivable'
          AND i.due_date IS NOT NULL AND TRIM(i.due_date) != ''
          AND p.status IN ('completed', 'paid')
          AND p.payment_date IS NOT NULL AND TRIM(p.payment_date) != ''
        GROUP BY i.client_id, i.id
        "#,
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;
    for row in payment_rows {
        let client_id: String = row.try_get("client_id").unwrap_or_default();
        if client_id.is_empty() {
            continue;
        }
        let delay: f64 = row.try_get("delay_days").unwrap_or(0.0);
        profiles
            .entry(client_id)
            .or_default()
            .payment_delays
            .push(delay.max(0.0));
    }

    Ok(profiles
        .into_iter()
        .map(|(client_id, acc)| {
            let paid_invoice_count = acc.payment_delays.len() as i64;
            let average_delay = if acc.payment_delays.is_empty() {
                0.0
            } else {
                acc.payment_delays.iter().sum::<f64>() / acc.payment_delays.len() as f64
            };
            let escalation_penalty = acc.escalation_penalty.min(60);
            let speed_penalty = (average_delay / 3.0).ceil() as i64;
            let score = (100 - escalation_penalty - speed_penalty.min(30)).clamp(0, 100);
            (
                client_id,
                credit_profile_value(
                    score,
                    average_delay,
                    paid_invoice_count,
                    acc.collection_events,
                    escalation_penalty,
                ),
            )
        })
        .collect())
}

pub fn default_credit_profile() -> Value {
    credit_profile_value(100, 0.0, 0, 0, 0)
}

fn credit_profile_value(
    score: i64,
    average_delay: f64,
    paid_invoice_count: i64,
    collection_events: i64,
    escalation_penalty: i64,
) -> Value {
    let (grade, label) = match score {
        90..=100 => ("A", "优秀"),
        75..=89 => ("B", "良好"),
        60..=74 => ("C", "关注"),
        _ => ("D", "高风险"),
    };
    json!({
        "score": score,
        "score_label": format!("{} 分", score),
        "grade": grade,
        "label": label,
        "average_payment_delay_days": (average_delay * 10.0).round() / 10.0,
        "average_payment_delay_label": format!("平均逾期 {:.1} 天回款", average_delay),
        "paid_invoice_count": paid_invoice_count,
        "collection_event_count": collection_events,
        "collection_penalty": escalation_penalty,
        "basis": "按实际回款延迟与已送达催收级别计算",
    })
}

fn delivery_status_label(status: &str) -> &'static str {
    match status {
        "pending" => "待发送",
        "dispatching" => "发送中",
        "sent" => "已送达",
        "failed" => "发送失败",
        _ => "未知",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_selection_uses_highest_due_threshold() {
        let settings = CollectionSettings::default();
        assert_eq!(due_level(&settings, 0).map(|l| l.code), None);
        assert_eq!(due_level(&settings, 1).map(|l| l.code), Some("friendly"));
        assert_eq!(due_level(&settings, 7).map(|l| l.code), Some("formal"));
        assert_eq!(due_level(&settings, 30).map(|l| l.code), Some("final"));
        assert_eq!(due_level(&settings, 60).map(|l| l.code), Some("bad_debt"));
    }

    #[test]
    fn settings_require_strictly_increasing_days() {
        assert!(CollectionSettings::default().validate().is_ok());
        let invalid = CollectionSettings {
            formal_days: 1,
            ..CollectionSettings::default()
        };
        assert!(invalid.validate().is_err());
    }
}
