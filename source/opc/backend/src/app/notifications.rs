//! Reminder delivery via w-notify (飞书/企微/Telegram/Slack/钉钉/webhook).

use chrono::Timelike;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use w_notify::{Channel, Message};

fn reminders_url() -> String {
    let base = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8987".to_owned());
    format!("{}/reminders", base.trim_end_matches('/'))
}
const MAX_DIGEST_ITEMS: usize = 10;
const MAX_CONSECUTIVE_FAILURES: u32 = 5;

pub fn spawn_notification_dispatcher(pool: SqlitePool) {
    tokio::spawn(async move {
        let failures: Arc<Mutex<HashMap<String, u32>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut last_digest_day = String::new();
        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
        loop {
            if let Err(err) = dispatch_instant_round(&pool, &failures).await {
                tracing::warn!("notification instant dispatch failed: {}", err);
            }
            let now = chrono::Local::now();
            let today = now.format("%Y-%m-%d").to_string();
            if now.hour() == 9 && last_digest_day != today {
                match dispatch_digest_round(&pool, &failures).await {
                    Ok(sent) => {
                        if sent > 0 {
                            tracing::info!("📬 Digest notifications sent to {} channel(s)", sent);
                        }
                        last_digest_day = today;
                    }
                    Err(err) => tracing::warn!("notification digest dispatch failed: {}", err),
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(10 * 60)).await;
        }
    });
}

pub async fn dispatch_instant_round(
    pool: &SqlitePool,
    failures: &Arc<Mutex<HashMap<String, u32>>>,
) -> Result<usize, sqlx::Error> {
    let channels: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT id, company_id, channel_url FROM notification_channels \
         WHERE enabled = 1 AND mode = 'instant'",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut sent_total = 0usize;
    for (channel_id, company_id, channel_url) in channels {
        if should_skip_channel(&channel_id, failures) {
            continue;
        }
        let reminders: Vec<(String, String, Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT id, title, description, due_date FROM reminders \
             WHERE company_id = ? AND status = 'pending' AND notified_at IS NULL \
             AND (due_date IS NULL OR date(due_date) <= date('now', '+3 days')) \
             ORDER BY CASE WHEN due_date IS NULL THEN 1 ELSE 0 END, due_date ASC \
             LIMIT 50",
        )
        .bind(&company_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        for (id, title, description, due_date) in reminders {
            let body = format_instant_message(&title, description.as_deref(), due_date.as_deref());
            match send_message(&channel_url, &title, &body).await {
                Ok(()) => {
                    let _ = sqlx::query(
                        "UPDATE reminders SET notified_at = datetime('now') WHERE id = ?",
                    )
                    .bind(&id)
                    .execute(pool)
                    .await;
                    clear_failure(&channel_id, failures);
                    sent_total += 1;
                }
                Err(err) => {
                    tracing::warn!("notify channel {} failed: {}", channel_id, err);
                    record_failure(&channel_id, failures);
                    break;
                }
            }
        }
    }
    Ok(sent_total)
}

pub async fn dispatch_digest_round(
    pool: &SqlitePool,
    failures: &Arc<Mutex<HashMap<String, u32>>>,
) -> Result<usize, sqlx::Error> {
    let channels: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT id, company_id, channel_url FROM notification_channels \
         WHERE enabled = 1 AND mode = 'digest'",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut sent = 0usize;
    for (channel_id, company_id, channel_url) in channels {
        if should_skip_channel(&channel_id, failures) {
            continue;
        }
        let collection_deliveries =
            crate::app::collections::claim_digest_deliveries(pool, &company_id, &channel_id)
                .await
                .unwrap_or_default();
        let pending: Vec<(String, String, Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT id, title, description, due_date FROM reminders \
             WHERE company_id = ? AND status = 'pending' AND notified_at IS NULL \
             ORDER BY CASE WHEN due_date IS NULL THEN 1 ELSE 0 END, due_date ASC LIMIT 100",
        )
        .bind(&company_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let overdue_invoices: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM invoices WHERE company_id = ? \
             AND status IN ('unpaid','overdue','pending') \
             AND due_date IS NOT NULL AND due_date != '' AND date(due_date) < date('now') \
             AND (NOT EXISTS (SELECT 1 FROM collection_settings cs WHERE cs.company_id = invoices.company_id) \
                  OR EXISTS (SELECT 1 FROM collection_settings cs WHERE cs.company_id = invoices.company_id AND cs.enabled = 1))",
        )
        .bind(&company_id)
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

        if pending.is_empty() && collection_deliveries.is_empty() {
            continue;
        }
        let mut body = format_digest_message(&pending, overdue_invoices.0 as usize);
        let collection_section =
            crate::app::collections::format_digest_section(&collection_deliveries);
        if !collection_section.is_empty() {
            body.push_str("\n\n");
            body.push_str(&collection_section);
        }
        let title = "OPC 每日提醒摘要";
        match send_message(&channel_url, title, &body).await {
            Ok(()) => {
                for (id, _, _, _) in &pending {
                    let _ = sqlx::query(
                        "UPDATE reminders SET notified_at = datetime('now') WHERE id = ?",
                    )
                    .bind(id)
                    .execute(pool)
                    .await;
                }
                let _ = crate::app::collections::finish_digest_deliveries(
                    pool,
                    &collection_deliveries,
                    None,
                )
                .await;
                clear_failure(&channel_id, failures);
                sent += 1;
            }
            Err(err) => {
                let _ = crate::app::collections::finish_digest_deliveries(
                    pool,
                    &collection_deliveries,
                    Some(&err),
                )
                .await;
                tracing::warn!("digest channel {} failed: {}", channel_id, err);
                record_failure(&channel_id, failures);
            }
        }
    }
    Ok(sent)
}

pub fn format_instant_message(
    title: &str,
    description: Option<&str>,
    due_date: Option<&str>,
) -> String {
    let mut lines = vec![title.to_string()];
    if let Some(d) = due_date.filter(|s| !s.is_empty()) {
        lines.push(format!("截止日: {}", d));
    }
    if let Some(desc) = description.filter(|s| !s.is_empty()) {
        lines.push(desc.to_string());
    }
    lines.push(format!("查看: {}", reminders_url()));
    lines.join("\n")
}

pub fn format_digest_message(
    reminders: &[(String, String, Option<String>, Option<String>)],
    overdue_invoices: usize,
) -> String {
    let mut lines = Vec::new();
    lines.push(format!("今日待处理提醒 {} 条", reminders.len()));
    if overdue_invoices > 0 {
        lines.push(format!("逾期发票 {} 张", overdue_invoices));
    }
    lines.push(String::new());
    lines.push("近期事项:".to_string());
    let shown = reminders.len().min(MAX_DIGEST_ITEMS);
    for (_, title, _, due_date) in reminders.iter().take(shown) {
        let due = due_date
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or("无截止日");
        lines.push(format!("• {} ({})", title, due));
    }
    if reminders.len() > MAX_DIGEST_ITEMS {
        lines.push(format!("等 {} 条", reminders.len() - MAX_DIGEST_ITEMS));
    }
    lines.push(String::new());
    lines.push(format!("打开提醒列表: {}", reminders_url()));
    lines.join("\n")
}

pub(crate) async fn send_message(url: &str, title: &str, body: &str) -> Result<(), String> {
    let channel = Channel::from_url(url);
    let msg = Message::new(body).title(title);
    channel.send(&msg).await.map_err(|e| format!("{e}"))
}

pub async fn send_test_message(url: &str) -> Result<(), String> {
    send_message(url, "OPC", "OPC 测试消息").await
}

fn should_skip_channel(id: &str, failures: &Arc<Mutex<HashMap<String, u32>>>) -> bool {
    failures
        .lock()
        .map(|m| m.get(id).copied().unwrap_or(0) >= MAX_CONSECUTIVE_FAILURES)
        .unwrap_or(false)
}
fn record_failure(id: &str, failures: &Arc<Mutex<HashMap<String, u32>>>) {
    if let Ok(mut m) = failures.lock() {
        *m.entry(id.to_string()).or_insert(0) += 1;
    }
}
fn clear_failure(id: &str, failures: &Arc<Mutex<HashMap<String, u32>>>) {
    if let Ok(mut m) = failures.lock() {
        m.remove(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_message_lists_items_and_overflow() {
        let mut items = Vec::new();
        for i in 0..12 {
            items.push((
                format!("id-{i}"),
                format!("事项 {i}"),
                Some(format!("desc-{i}")),
                Some(format!("2026-08-{:02}", (i % 28) + 1)),
            ));
        }
        let body = format_digest_message(&items, 3);
        assert!(body.contains("今日待处理提醒 12 条"));
        assert!(body.contains("逾期发票 3 张"));
        assert!(body.contains("等 2 条"));
        assert!(body.contains(reminders_url().as_str()));
    }

    #[test]
    fn instant_message_includes_due_and_link() {
        let body = format_instant_message(
            "合同即将到期: X",
            Some("[contract_expiry]c1"),
            Some("2026-08-10"),
        );
        assert!(body.contains("截止日: 2026-08-10"));
        assert!(body.contains(reminders_url().as_str()));
    }
}
