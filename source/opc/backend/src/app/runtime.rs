use sqlx::SqlitePool;

pub fn spawn_contract_review_catchup(pool: SqlitePool) {
    tokio::spawn(async move {
        let rows = sqlx::query_scalar::<_, String>(
            r#"
            SELECT id FROM contracts
            WHERE ai_review_status = 'reviewing'
               OR (ai_review_status IN ('pending', 'unavailable', 'failed')
                   AND (trim(COALESCE(content, '')) != '' OR trim(COALESCE(notes, '')) != ''))
            "#,
        )
        .fetch_all(&pool)
        .await;
        let rows = match rows {
            Ok(rows) => rows,
            Err(error) => {
                tracing::warn!("contract review catchup query failed: {error}");
                return;
            }
        };
        for id in rows {
            if let Err(error) = crate::api::trigger_contract_review(pool.clone(), id).await {
                tracing::warn!("contract review catchup failed: {error:?}");
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    });
}

pub fn spawn_expiry_reminder_scheduler(pool: SqlitePool) {
    tokio::spawn(async move {
        loop {
            if let Err(err) = generate_expiry_reminders(&pool).await {
                tracing::warn!("Reminder generation failed: {}", err);
            }
            if let Err(err) = crate::error_log_mw::purge_old_error_logs(&pool, 90).await {
                tracing::warn!("error_logs purge failed: {}", err);
            }
            tokio::time::sleep(std::time::Duration::from_secs(6 * 3600)).await;
        }
    });
}

/// Hourly billing loop: generate due recurring invoices + mark overdue.
/// Runs once immediately on startup (catch-up), then every hour.
pub fn spawn_billing_scheduler(pool: SqlitePool) {
    tokio::spawn(async move {
        loop {
            if let Err(err) = run_billing_cycle(&pool).await {
                tracing::warn!("Billing scheduler cycle failed: {}", err);
            }
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    });
}

/// One full billing cycle (used by scheduler and tests).
pub async fn run_billing_cycle(pool: &SqlitePool) -> Result<(), String> {
    let generated = generate_due_recurring_invoices(pool)
        .await
        .map_err(|e| e.to_string())?;
    let overdue = mark_overdue_invoices(pool)
        .await
        .map_err(|e| e.to_string())?;
    let collections = crate::app::collections::run_collection_cycle(pool)
        .await
        .map_err(|e| e.to_string())?;

    let details = format!(
        "generated={generated}, overdue={overdue}, collections_queued={}, collections_sent={}, collections_failed={}",
        collections.queued, collections.sent, collections.failed
    );
    let run_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let _ = sqlx::query(
        "INSERT INTO scheduler_runs (id, scheduler_name, last_run_at, generated_count, details) VALUES (?, 'billing', ?, ?, ?)",
    )
    .bind(&run_id)
    .bind(&now)
    .bind((generated + overdue + collections.queued) as i64)
    .bind(&details)
    .execute(pool)
    .await;

    if generated > 0 || overdue > 0 || collections.queued > 0 {
        tracing::info!(
            "💳 Billing cycle: {} invoices generated, {} marked overdue, {} collection deliveries queued ({} sent, {} failed)",
            generated,
            overdue,
            collections.queued,
            collections.sent,
            collections.failed,
        );
    }
    Ok(())
}

/// Generate invoices for active recurring templates whose next_invoice_date <= today.
/// Advances next_invoice_date and backfills up to 12 periods per template per cycle.
pub async fn generate_due_recurring_invoices(pool: &SqlitePool) -> Result<usize, sqlx::Error> {
    use chrono::{Duration, NaiveDate};

    let rows = sqlx::query(
        r#"
        SELECT id, company_id, client_id, title, description, amount, currency, tax_rate,
               frequency, next_invoice_date, payment_terms_days, total_generated, created_by
        FROM recurring_invoices
        WHERE status = 'active'
          AND next_invoice_date IS NOT NULL
          AND next_invoice_date != ''
          AND date(next_invoice_date) <= date('now', 'localtime')
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut total_generated = 0usize;
    let today = chrono::Local::now().date_naive();

    for row in rows {
        use sqlx::Row;
        let recurring_id: String = row.try_get("id").unwrap_or_default();
        let company_id: String = row.try_get("company_id").unwrap_or_default();
        let client_id: Option<String> = row.try_get("client_id").ok().flatten();
        let title: String = row.try_get("title").unwrap_or_default();
        let description: Option<String> = row.try_get("description").ok().flatten();
        let amount: f64 = row.try_get("amount").unwrap_or(0.0);
        let currency: Option<String> = row.try_get("currency").ok().flatten();
        let tax_rate: f64 = row.try_get("tax_rate").unwrap_or(0.0);
        let frequency: String = row
            .try_get("frequency")
            .unwrap_or_else(|_| "monthly".into());
        let next_str: String = row.try_get("next_invoice_date").unwrap_or_default();
        let payment_terms_days: i64 = row.try_get("payment_terms_days").unwrap_or(30);
        let mut total_gen: i64 = row.try_get("total_generated").unwrap_or(0);
        let created_by: Option<String> = row.try_get("created_by").ok().flatten();

        let mut next_date = match NaiveDate::parse_from_str(&next_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => continue,
        };

        let client_name: String = if let Some(ref cid) = client_id {
            sqlx::query_scalar("SELECT name FROM clients WHERE id = ? AND company_id = ?")
                .bind(cid)
                .bind(&company_id)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten()
                .unwrap_or_default()
        } else {
            String::new()
        };

        let mut periods = 0usize;
        while next_date <= today && periods < 12 {
            let period_date = next_date.format("%Y-%m-%d").to_string();
            let desc_marker = format!("循环发票 {recurring_id} · 周期 {period_date}");

            // Idempotency guard: skip if already generated for this period.
            let existing: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM invoices WHERE company_id = ? AND description LIKE ?",
            )
            .bind(&company_id)
            .bind(format!("%{desc_marker}%"))
            .fetch_one(pool)
            .await
            .unwrap_or(0);

            if existing == 0 {
                let mut tx = pool.begin().await?;
                let invoice_number =
                    match crate::app::numbering::next_document_number(&mut tx, &company_id, "INV")
                        .await
                    {
                        Ok(n) => n,
                        Err(e) => {
                            tracing::warn!("numbering for recurring failed: {:?}", e);
                            format!("INV-R-{}", &uuid::Uuid::new_v4().to_string()[..8])
                        }
                    };

                let invoice_id = uuid::Uuid::new_v4().to_string();
                let due_date = (next_date + Duration::days(payment_terms_days.max(0)))
                    .format("%Y-%m-%d")
                    .to_string();
                let tax_amount = if tax_rate > 1.0 {
                    amount * tax_rate / 100.0
                } else {
                    amount * tax_rate
                };
                let inv_desc = format!(
                    "由{desc_marker}自动生成 · {}",
                    description.as_deref().unwrap_or(&title)
                );
                let currency = currency
                    .as_deref()
                    .map(str::trim)
                    .filter(|c| !c.is_empty())
                    .unwrap_or("CNY");

                if let Err(e) = sqlx::query(
                    r#"
                    INSERT INTO invoices (
                        id, company_id, invoice_number, invoice_type, amount, tax_amount,
                        status, invoice_date, due_date, supplier, description,
                        currency, client_id, created_by
                    ) VALUES (?, ?, ?, 'receivable', ?, ?, 'pending', ?, ?, ?, ?, ?, ?, ?)
                    "#,
                )
                .bind(&invoice_id)
                .bind(&company_id)
                .bind(&invoice_number)
                .bind(amount)
                .bind(tax_amount)
                .bind(&period_date)
                .bind(&due_date)
                .bind(&client_name)
                .bind(&inv_desc)
                .bind(currency)
                .bind(&client_id)
                .bind(&created_by)
                .execute(&mut *tx)
                .await
                {
                    tracing::warn!("recurring invoice insert failed: {:?}", e);
                    let _ = tx.rollback().await;
                } else {
                    let reminder_id = uuid::Uuid::new_v4().to_string();
                    let rem_title = format!("已自动生成发票 {invoice_number}，请确认并发送");
                    let rem_notes = format!("[recurring_invoice]{recurring_id}");
                    let _ = sqlx::query(
                        "INSERT INTO reminders (id, company_id, title, description, status, priority, reminder_type) VALUES (?, ?, ?, ?, 'pending', 'normal', 'billing')",
                    )
                    .bind(&reminder_id)
                    .bind(&company_id)
                    .bind(&rem_title)
                    .bind(&rem_notes)
                    .execute(&mut *tx)
                    .await;

                    if tx.commit().await.is_ok() {
                        total_generated += 1;
                        total_gen += 1;
                    }
                }
            }

            // Advance next_invoice_date by frequency.
            next_date = advance_recurring_date(next_date, &frequency);
            periods += 1;
        }

        let next_str = next_date.format("%Y-%m-%d").to_string();
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let _ = sqlx::query(
            r#"
            UPDATE recurring_invoices
            SET next_invoice_date = ?, total_generated = ?, last_generated_at = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ? AND company_id = ?
            "#,
        )
        .bind(&next_str)
        .bind(total_gen)
        .bind(&now)
        .bind(&recurring_id)
        .bind(&company_id)
        .execute(pool)
        .await;
    }

    Ok(total_generated)
}

fn advance_recurring_date(date: chrono::NaiveDate, frequency: &str) -> chrono::NaiveDate {
    use chrono::Months;
    match frequency.to_lowercase().as_str() {
        "weekly" => date + chrono::Duration::days(7),
        "quarterly" => date
            .checked_add_months(Months::new(3))
            .unwrap_or(date + chrono::Duration::days(90)),
        "yearly" | "annual" => date
            .checked_add_months(Months::new(12))
            .unwrap_or(date + chrono::Duration::days(365)),
        // monthly (default) — Months handles month-end (1/31 → 2/28)
        _ => date
            .checked_add_months(Months::new(1))
            .unwrap_or(date + chrono::Duration::days(30)),
    }
}

/// Mark pending/sent invoices past due_date as overdue. Does not touch partially_paid
/// (keep partial-payment signal; aging queries include partially_paid separately).
pub async fn mark_overdue_invoices(pool: &SqlitePool) -> Result<usize, sqlx::Error> {
    let rows: Vec<(String, Option<String>, Option<String>)> = sqlx::query_as(
        r#"
        SELECT id, invoice_number, due_date FROM invoices
        WHERE status IN ('pending', 'sent', 'approved')
          AND due_date IS NOT NULL AND due_date != ''
          AND date(due_date) < date('now', 'localtime')
        "#,
    )
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Ok(0);
    }

    let result = sqlx::query(
        r#"
        UPDATE invoices
        SET status = 'overdue', updated_at = CURRENT_TIMESTAMP
        WHERE status IN ('pending', 'sent', 'approved')
          AND due_date IS NOT NULL AND due_date != ''
          AND date(due_date) < date('now', 'localtime')
        "#,
    )
    .execute(pool)
    .await?;

    let today = chrono::Local::now().date_naive();
    for (id, invoice_number, due_date) in &rows {
        let number = invoice_number.as_deref().unwrap_or(&id);
        let days = due_date
            .as_deref()
            .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
            .map(|d| (today - d).num_days())
            .unwrap_or(0);
        let company_id: Option<String> =
            sqlx::query_scalar("SELECT company_id FROM invoices WHERE id = ?")
                .bind(id)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten();
        let Some(company_id) = company_id else {
            continue;
        };

        let existing: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM reminders WHERE company_id = ? AND description LIKE ? AND created_at >= date('now', '-7 days')",
        )
        .bind(&company_id)
        .bind(format!("[invoice_overdue]{id}%"))
        .fetch_one(pool)
        .await
        .unwrap_or(0);
        if existing > 0 {
            continue;
        }

        let rem_id = uuid::Uuid::new_v4().to_string();
        let title = format!("发票 {number} 已逾期 {days} 天");
        let notes = format!("[invoice_overdue]{id}");
        let _ = sqlx::query(
            "INSERT INTO reminders (id, company_id, title, description, status, priority, reminder_type) VALUES (?, ?, ?, ?, 'pending', 'high', 'invoice_overdue')",
        )
        .bind(&rem_id)
        .bind(&company_id)
        .bind(&title)
        .bind(&notes)
        .execute(pool)
        .await;
    }

    Ok(result.rows_affected() as usize)
}

pub async fn generate_expiry_reminders(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let checks = vec![
        (
            "contracts",
            "contract_name",
            "end_date",
            "contract_expiry",
            30,
            "合同即将到期",
        ),
        (
            "domain_subscriptions",
            "domain_name",
            "expiry_date",
            "domain_renewal",
            30,
            "域名即将到期",
        ),
        (
            "software_subscriptions",
            "software_name",
            "next_billing_date",
            "saas_renewal",
            7,
            "SaaS 即将续费",
        ),
        (
            "licenses",
            "name",
            "expiry_date",
            "license_expiry",
            30,
            "证照即将到期",
        ),
    ];

    let mut total = 0;

    for (table, name_col, date_col, entity_type, days, prefix) in &checks {
        let sql = format!(
            "SELECT id, company_id, {}, {} FROM {} WHERE {} IS NOT NULL AND {} != '' \
             AND date({}) <= date('now', '+{} days') AND date({}) >= date('now')",
            name_col, date_col, table, date_col, date_col, date_col, days, date_col
        );

        let rows: Vec<(String, String, String, String)> = sqlx::query_as(&sql)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

        for (entity_id, company_id, name, due_date) in &rows {
            let existing: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM reminders WHERE company_id = ? \
                 AND description LIKE ? AND created_at >= date('now', '-30 days')",
            )
            .bind(company_id)
            .bind(format!("[{}]%{}", entity_type, entity_id))
            .fetch_one(pool)
            .await
            .unwrap_or((0,));

            if existing.0 > 0 {
                continue;
            }

            let id = uuid::Uuid::new_v4().to_string();
            let title = format!("{}: {}", prefix, name);
            let notes = format!("[{}]{}", entity_type, entity_id);

            let _ = sqlx::query(
                "INSERT INTO reminders (id, company_id, title, description, status, priority, reminder_type, due_date) \
                 VALUES (?, ?, ?, ?, 'pending', 'high', ?, ?)",
            )
            .bind(&id)
            .bind(company_id)
            .bind(&title)
            .bind(&notes)
            .bind(entity_type)
            .bind(due_date)
            .execute(pool)
            .await;

            total += 1;
        }
    }

    if total > 0 {
        tracing::info!("🔔 Generated {} expiry reminders", total);
    }

    Ok(())
}

/// Daily automatic backup: wake hourly; if no successful backup for local today,
/// create one and prune old archives (14 daily + 12 first-of-month).
pub fn spawn_backup_scheduler(pool: SqlitePool, data_dir: std::path::PathBuf) {
    tokio::spawn(async move {
        // Small initial delay so startup is not blocked by a long first backup.
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        loop {
            if let Err(err) = run_backup_scheduler_tick(&pool, &data_dir).await {
                tracing::warn!("Backup scheduler tick failed: {}", err);
            }
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    });
}

async fn run_backup_scheduler_tick(
    pool: &SqlitePool,
    data_dir: &std::path::Path,
) -> Result<(), String> {
    if crate::app::backup::has_backup_today(pool).await {
        return Ok(());
    }
    tracing::info!("💾 Daily backup: no success record for today, creating…");
    match crate::app::backup::create_backup(pool, data_dir).await {
        Ok(meta) => {
            tracing::info!("💾 Daily backup complete: {}", meta.file_name);
        }
        Err(e) => {
            tracing::error!("Daily backup failed: {}", e);
            return Err(e);
        }
    }
    crate::app::backup::prune_old_backups(pool, data_dir).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        advance_recurring_date, generate_due_recurring_invoices, generate_expiry_reminders,
        mark_overdue_invoices, run_billing_cycle,
    };
    use crate::db;
    use chrono::NaiveDate;
    use sqlx::SqlitePool;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("in-memory db");
        db::run_migrations(&pool).await.expect("migrations");
        pool
    }

    async fn insert_company(pool: &SqlitePool, company_id: &str) {
        sqlx::query("INSERT INTO companies (id, name) VALUES (?, ?)")
            .bind(company_id)
            .bind("TestCo")
            .execute(pool)
            .await
            .expect("company");
    }

    #[tokio::test]
    async fn generate_expiry_reminders_creates_pending_reminders() {
        let pool = test_pool().await;
        let company_id = "company-1";
        let contract_id = "contract-1";

        insert_company(&pool, company_id).await;

        sqlx::query(
            "INSERT INTO contracts (id, company_id, contract_name, end_date) \
             VALUES (?, ?, ?, date('now', '+10 days'))",
        )
        .bind(contract_id)
        .bind(company_id)
        .bind("Bridge Financing Agreement")
        .execute(&pool)
        .await
        .expect("contract");

        generate_expiry_reminders(&pool)
            .await
            .expect("reminders generated");

        let reminder: (String, String, String, String) = sqlx::query_as(
            "SELECT title, description, status, reminder_type FROM reminders WHERE company_id = ?",
        )
        .bind(company_id)
        .fetch_one(&pool)
        .await
        .expect("reminder");

        assert_eq!(reminder.0, "合同即将到期: Bridge Financing Agreement");
        assert_eq!(reminder.1, "[contract_expiry]contract-1");
        assert_eq!(reminder.2, "pending");
        assert_eq!(reminder.3, "contract_expiry");
    }

    #[tokio::test]
    async fn generate_expiry_reminders_deduplicates_recent_existing_records() {
        let pool = test_pool().await;
        let company_id = "company-1";
        let contract_id = "contract-1";

        insert_company(&pool, company_id).await;

        sqlx::query(
            "INSERT INTO contracts (id, company_id, contract_name, end_date) \
             VALUES (?, ?, ?, date('now', '+10 days'))",
        )
        .bind(contract_id)
        .bind(company_id)
        .bind("Bridge Financing Agreement")
        .execute(&pool)
        .await
        .expect("contract");

        sqlx::query(
            "INSERT INTO reminders (id, company_id, title, description, status, priority, reminder_type) \
             VALUES (?, ?, ?, ?, 'pending', 'high', ?)",
        )
        .bind("reminder-1")
        .bind(company_id)
        .bind("合同即将到期: Bridge Financing Agreement")
        .bind("[contract_expiry]contract-1")
        .bind("contract_expiry")
        .execute(&pool)
        .await
        .expect("existing reminder");

        generate_expiry_reminders(&pool)
            .await
            .expect("reminders generated");

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM reminders WHERE company_id = ?")
            .bind(company_id)
            .fetch_one(&pool)
            .await
            .expect("count reminders");

        assert_eq!(count.0, 1);
    }

    #[tokio::test]
    async fn billing_generates_recurring_invoice_once_and_advances() {
        let pool = test_pool().await;
        let company_id = "company-1";
        insert_company(&pool, company_id).await;

        let yesterday = (chrono::Local::now().date_naive() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();

        sqlx::query(
            r#"
            INSERT INTO recurring_invoices (
                id, company_id, title, amount, currency, frequency,
                start_date, next_invoice_date, payment_terms_days, status, total_generated
            ) VALUES ('rec-1', ?, 'Monthly retainer', 1000, 'CNY', 'monthly', ?, ?, 30, 'active', 0)
            "#,
        )
        .bind(company_id)
        .bind(&yesterday)
        .bind(&yesterday)
        .execute(&pool)
        .await
        .expect("recurring");

        let n = generate_due_recurring_invoices(&pool)
            .await
            .expect("generate");
        assert_eq!(n, 1);

        let inv_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM invoices WHERE company_id = ? AND description LIKE '%循环发票 rec-1%'",
        )
        .bind(company_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(inv_count.0, 1);

        let next: (String,) =
            sqlx::query_as("SELECT next_invoice_date FROM recurring_invoices WHERE id = 'rec-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        let expected_next = advance_recurring_date(
            NaiveDate::parse_from_str(&yesterday, "%Y-%m-%d").unwrap(),
            "monthly",
        )
        .format("%Y-%m-%d")
        .to_string();
        assert_eq!(next.0, expected_next);

        // Second run must not duplicate.
        let n2 = generate_due_recurring_invoices(&pool)
            .await
            .expect("generate2");
        assert_eq!(n2, 0);
        let inv_count2: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM invoices WHERE company_id = ? AND description LIKE '%循环发票 rec-1%'",
        )
        .bind(company_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(inv_count2.0, 1);
    }

    #[tokio::test]
    async fn billing_backfills_multiple_missed_periods() {
        let pool = test_pool().await;
        insert_company(&pool, "company-1").await;

        let three_months_ago = (chrono::Local::now().date_naive() - chrono::Months::new(3))
            .format("%Y-%m-%d")
            .to_string();

        sqlx::query(
            r#"
            INSERT INTO recurring_invoices (
                id, company_id, title, amount, frequency,
                start_date, next_invoice_date, payment_terms_days, status, total_generated
            ) VALUES ('rec-2', 'company-1', 'Backfill', 500, 'monthly', ?, ?, 30, 'active', 0)
            "#,
        )
        .bind(&three_months_ago)
        .bind(&three_months_ago)
        .execute(&pool)
        .await
        .expect("recurring");

        let n = generate_due_recurring_invoices(&pool)
            .await
            .expect("generate");
        assert!(n >= 3, "expected >=3 backfill invoices, got {n}");

        let inv_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM invoices WHERE company_id = 'company-1' AND description LIKE '%循环发票 rec-2%'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(inv_count.0, n as i64);
    }

    #[tokio::test]
    async fn billing_marks_overdue_pending_not_paid() {
        let pool = test_pool().await;
        insert_company(&pool, "company-1").await;

        let yesterday = (chrono::Local::now().date_naive() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();

        sqlx::query(
            "INSERT INTO invoices (id, company_id, invoice_number, amount, status, due_date) VALUES ('inv-over', 'company-1', 'INV-O', 100, 'pending', ?)",
        )
        .bind(&yesterday)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO invoices (id, company_id, invoice_number, amount, status, due_date) VALUES ('inv-paid', 'company-1', 'INV-P', 100, 'paid', ?)",
        )
        .bind(&yesterday)
        .execute(&pool)
        .await
        .unwrap();

        let n = mark_overdue_invoices(&pool).await.expect("mark");
        assert_eq!(n, 1);

        let st: (String,) = sqlx::query_as("SELECT status FROM invoices WHERE id = 'inv-over'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(st.0, "overdue");
        let st2: (String,) = sqlx::query_as("SELECT status FROM invoices WHERE id = 'inv-paid'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(st2.0, "paid");

        let rem: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM reminders WHERE description LIKE '%[invoice_overdue]inv-over%'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(rem.0, 1);
    }

    #[test]
    fn advance_monthly_handles_month_end() {
        let d = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
        let next = advance_recurring_date(d, "monthly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2026, 2, 28).unwrap());
    }

    #[tokio::test]
    async fn run_billing_cycle_writes_scheduler_run() {
        let pool = test_pool().await;
        insert_company(&pool, "company-1").await;
        run_billing_cycle(&pool).await.expect("cycle");
        let n: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM scheduler_runs WHERE scheduler_name = 'billing'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(n.0, 1);
    }
}
