use axum::http::StatusCode;
use sqlx::{Row, Sqlite, SqlitePool, Transaction};

use crate::api::error::{ApiError, api_err};

pub struct ConvertTimeEntriesToInvoiceInput<'a> {
    pub company_id: &'a str,
    pub user_id: &'a str,
    pub project_id: Option<&'a str>,
    pub client_id: Option<&'a str>,
}

pub struct ConvertTimeEntriesToInvoiceResult {
    pub invoice_id: String,
    pub invoice_number: String,
    pub amount: f64,
    pub entries_count: usize,
    pub total_hours: f64,
    pub message: String,
}

pub struct ConvertQuoteToInvoiceInput<'a> {
    pub company_id: &'a str,
    pub user_id: &'a str,
    pub quote_id: &'a str,
}

pub struct ConvertQuoteToInvoiceResult {
    pub invoice_id: String,
    pub invoice_number: String,
    pub quote_id: String,
    pub quote_number: String,
    pub amount: f64,
    pub message: String,
}

pub async fn sync_payment_links(
    tx: &mut Transaction<'_, Sqlite>,
    company_id: &str,
    _payment_id: &str,
    _payment_date: Option<&str>,
    invoice_id: Option<&str>,
    salary_record_id: Option<&str>,
) -> Result<(), ApiError> {
    if let Some(invoice_id) = invoice_id.filter(|value| !value.is_empty()) {
        let invoice_amount: f64 = match sqlx::query_scalar(
            "SELECT COALESCE(amount, 0) FROM invoices WHERE id = ? AND company_id = ?",
        )
        .bind(invoice_id)
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        {
            Ok(Some(v)) => v,
            Ok(None) => {
                return Err(api_err(
                    StatusCode::NOT_FOUND,
                    "invoice not found for payment sync",
                ));
            }
            Err(e) => {
                tracing::error!("invoice amount lookup failed: {e:?}");
                return Err(api_err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "invoice sync failed",
                ));
            }
        };

        let total_paid: f64 = match sqlx::query_scalar(
            "SELECT CAST(COALESCE(SUM(amount), 0) AS REAL) FROM payments WHERE invoice_id = ? AND company_id = ? AND status IN ('completed','paid')",
        )
        .bind(invoice_id)
        .bind(company_id)
        .fetch_one(&mut **tx)
        .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("invoice paid sum failed: {e:?}");
                return Err(api_err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "invoice sync failed",
                ));
            }
        };

        // Single settlement path for all payment create/update/delete flows.
        // paid when covered; partially_paid when 0 < paid < amount; pending when paid out.
        // Do not clobber cancelled/rejected; overdue is re-evaluated by billing scheduler.
        let current_status: String = match sqlx::query_scalar(
            "SELECT COALESCE(status, 'pending') FROM invoices WHERE id = ? AND company_id = ?",
        )
        .bind(invoice_id)
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        {
            Ok(Some(s)) => s,
            Ok(None) => "pending".to_string(),
            Err(e) => {
                tracing::error!("invoice status lookup failed: {e:?}");
                return Err(api_err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "invoice sync failed",
                ));
            }
        };

        if matches!(current_status.as_str(), "cancelled" | "rejected") {
            // Leave terminal statuses alone.
        } else {
            let new_status = if total_paid + 0.01 >= invoice_amount && invoice_amount > 0.0 {
                "paid".to_string()
            } else if total_paid > 0.01 {
                "partially_paid".to_string()
            } else if current_status == "paid" || current_status == "partially_paid" {
                // Payments wiped → fall back to pending (not overdue; scheduler will re-mark).
                "pending".to_string()
            } else {
                // Keep pending/sent/approved/overdue when nothing paid.
                current_status.clone()
            };

            if new_status != current_status {
                if let Err(e) = sqlx::query(
                    "UPDATE invoices SET status = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? AND company_id = ?",
                )
                .bind(&new_status)
                .bind(invoice_id)
                .bind(company_id)
                .execute(&mut **tx)
                .await
                {
                    tracing::error!("invoice status update failed: {e:?}");
                    return Err(api_err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "invoice sync failed",
                    ));
                }
            }
        }
    }

    if let Some(salary_record_id) = salary_record_id.filter(|value| !value.is_empty()) {
        // CAST to REAL: SQLite yields INTEGER 0 for the COALESCE fallback,
        // which sqlx refuses to decode as f64.
        let (net_salary,): (f64,) = sqlx::query_as(
            "SELECT CAST(COALESCE(net_salary, 0) AS REAL) FROM salary_records WHERE id = ? AND company_id = ?",
        )
        .bind(salary_record_id)
        .bind(company_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "salary sync failed"))?;

        let (total_paid,): (f64,) = sqlx::query_as(
            "SELECT CAST(COALESCE(SUM(amount), 0) AS REAL) FROM payments WHERE salary_record_id = ? AND company_id = ? AND status IN ('completed','paid')"
        )
        .bind(salary_record_id)
        .bind(company_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "salary sync failed"))?;

        if total_paid > 0.0 && total_paid + 0.000001 >= net_salary {
            // Reassignment/deletion may have removed the triggering payment.
            // Point payroll at an actual remaining payment, never a deleted ID.
            let (remaining_id, remaining_date): (String, Option<String>) = sqlx::query_as(
                "SELECT id, NULLIF(payment_date,'') FROM payments WHERE salary_record_id=? AND company_id=? AND status IN ('completed','paid') ORDER BY COALESCE(payment_date,'') DESC, created_at DESC, id DESC LIMIT 1"
            ).bind(salary_record_id).bind(company_id).fetch_one(&mut **tx).await
                .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "salary payment lookup failed"))?;
            sqlx::query(
                "UPDATE salary_records SET status='paid', payment_date=?, payment_id=?, updated_at=CURRENT_TIMESTAMP WHERE id=? AND company_id=? AND status NOT IN ('cancelled','rejected')"
            ).bind(remaining_date).bind(remaining_id).bind(salary_record_id).bind(company_id)
                .execute(&mut **tx).await
                .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "salary sync failed"))?;
        } else {
            // Only reverse a payment-derived settlement; preserve a manual
            // paid mark (no payment_id) and unrelated workflow states.
            sqlx::query(
                "UPDATE salary_records SET status='pending', payment_date=NULL, payment_id=NULL, updated_at=CURRENT_TIMESTAMP WHERE id=? AND company_id=? AND status='paid' AND payment_id IS NOT NULL"
            ).bind(salary_record_id).bind(company_id).execute(&mut **tx).await
                .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "salary sync failed"))?;
        }
    }

    Ok(())
}

pub async fn convert_time_entries_to_invoice(
    pool: &SqlitePool,
    input: ConvertTimeEntriesToInvoiceInput<'_>,
) -> Result<ConvertTimeEntriesToInvoiceResult, ApiError> {
    let mut tx = pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "tx begin failed"))?;

    let mut sql = String::from(
        "SELECT id, description, duration_minutes, amount FROM time_entries WHERE company_id = ? AND is_billable = 1 AND (status = 'logged' OR status IS NULL) AND (invoice_id IS NULL OR invoice_id = '')",
    );
    let mut bind_values: Vec<&str> = vec![input.company_id];

    if let Some(project_id) = input.project_id.filter(|value| !value.is_empty()) {
        sql.push_str(" AND project_id = ?");
        bind_values.push(project_id);
    }
    if let Some(client_id) = input.client_id.filter(|value| !value.is_empty()) {
        sql.push_str(" AND client_id = ?");
        bind_values.push(client_id);
    }

    let mut query = sqlx::query(&sql);
    for value in bind_values {
        query = query.bind(value);
    }

    let rows = query.fetch_all(&mut *tx).await.map_err(|error| {
        tracing::error!("time_entries query error: {:?}", error);
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "fetch time entries failed",
        )
    })?;

    if rows.is_empty() {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "no unbilled time entries found",
        ));
    }

    let mut total_amount = 0.0;
    let mut total_minutes = 0_i64;
    let mut entry_ids = Vec::with_capacity(rows.len());
    let mut descriptions = Vec::new();

    for row in &rows {
        let id: String = row.try_get("id").unwrap_or_default();
        let amount: f64 = row.try_get("amount").unwrap_or(0.0);
        let minutes: i64 = row.try_get("duration_minutes").unwrap_or(0);
        let description: String = row.try_get("description").unwrap_or_default();

        total_amount += amount;
        total_minutes += minutes;
        entry_ids.push(id);

        if !description.is_empty() && descriptions.len() < 5 {
            descriptions.push(description);
        }
    }

    let mut invoice_description = format!(
        "Time entries: {} entries, {:.1}h",
        entry_ids.len(),
        total_minutes as f64 / 60.0
    );
    if !descriptions.is_empty() {
        invoice_description.push_str(&format!(" ({})", descriptions.join(", ")));
    }

    let supplier_name = fetch_client_name(&mut tx, input.company_id, input.client_id).await;
    let invoice_id = uuid::Uuid::new_v4().to_string();
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let invoice_number =
        crate::app::numbering::next_document_number(&mut tx, input.company_id, "INV").await?;

    let currency = company_base_currency(&mut tx, input.company_id).await;
    let client_id = input.client_id.filter(|v| !v.trim().is_empty());

    sqlx::query(
        "INSERT INTO invoices (id, company_id, invoice_number, invoice_type, amount, status, invoice_date, supplier, description, created_by, currency, client_id) VALUES (?, ?, ?, 'receivable', ?, 'pending', ?, ?, ?, ?, ?, ?)"
    )
    .bind(&invoice_id)
    .bind(input.company_id)
    .bind(&invoice_number)
    .bind(total_amount)
    .bind(&today)
    .bind(&supplier_name)
    .bind(&invoice_description)
    .bind(input.user_id)
    .bind(&currency)
    .bind(client_id)
    .execute(&mut *tx)
    .await
    .map_err(|error| {
        tracing::error!("Invoice creation from time entries error: {:?}", error);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "create invoice failed")
    })?;

    for entry_id in &entry_ids {
        let changed = sqlx::query(
            "UPDATE time_entries SET invoice_id = ?, status = 'invoiced', updated_at = CURRENT_TIMESTAMP WHERE id = ? AND company_id = ? AND is_billable = 1 AND (status = 'logged' OR status IS NULL) AND (invoice_id IS NULL OR invoice_id = '')"
        )
        .bind(&invoice_id)
        .bind(entry_id)
        .bind(input.company_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "update time entry failed"))?;
        if changed.rows_affected() != 1 {
            return Err(api_err(
                StatusCode::CONFLICT,
                "time entry already converted",
            ));
        }
    }

    let log_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO operation_logs (id, company_id, user_id, action, entity_type, entity_id, details) VALUES (?, ?, ?, 'convert', 'time_entries', ?, ?)"
    )
    .bind(&log_id)
    .bind(input.company_id)
    .bind(input.user_id)
    .bind(&invoice_id)
    .bind(&format!(
        "Converted {} time entries ({:.1}h) to invoice {}",
        entry_ids.len(),
        total_minutes as f64 / 60.0,
        invoice_number
    ))
    .execute(&mut *tx)
    .await
    .ok();

    tx.commit()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "tx commit failed"))?;

    Ok(ConvertTimeEntriesToInvoiceResult {
        message: format!(
            "Converted {} time entries to invoice {}",
            entry_ids.len(),
            invoice_number
        ),
        invoice_id,
        invoice_number,
        amount: total_amount,
        entries_count: entry_ids.len(),
        total_hours: total_minutes as f64 / 60.0,
    })
}

pub async fn convert_quote_to_invoice(
    pool: &SqlitePool,
    input: ConvertQuoteToInvoiceInput<'_>,
) -> Result<ConvertQuoteToInvoiceResult, ApiError> {
    let mut tx = pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "tx begin failed"))?;

    let quote_row = sqlx::query("SELECT * FROM quotes WHERE id = ? AND company_id = ?")
        .bind(input.quote_id)
        .bind(input.company_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "fetch quote failed"))?
        .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "quote not found"))?;

    let status: String = quote_row.try_get("status").unwrap_or_default();
    if status == "invoiced"
        || quote_row
            .try_get::<Option<String>, _>("invoice_id")
            .ok()
            .flatten()
            .is_some_and(|id| !id.is_empty())
    {
        return Err(api_err(
            StatusCode::CONFLICT,
            "quote already converted to invoice",
        ));
    }
    if status == "rejected" || status == "expired" {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            &format!("cannot convert {} quote", status),
        ));
    }

    let mut total: f64 = quote_row.try_get("total").unwrap_or(0.0);
    let mut tax_amount: f64 = quote_row.try_get("tax_amount").unwrap_or(0.0);
    let title: String = quote_row.try_get("title").unwrap_or_default();
    let quote_number: String = quote_row.try_get("quote_number").unwrap_or_default();
    let client_id: String = quote_row.try_get("client_id").unwrap_or_default();

    // Reuse the quote calculation on this transaction's source snapshot.
    let (item_count, _) = crate::app::numbering::quote_items_sum_connection(
        &mut tx,
        input.company_id,
        input.quote_id,
    )
    .await?;
    if item_count > 0 {
        crate::app::numbering::recalc_quote_total_connection(
            &mut tx,
            input.company_id,
            input.quote_id,
        )
        .await?;
        let row =
            sqlx::query("SELECT total, tax_amount FROM quotes WHERE id = ? AND company_id = ?")
                .bind(input.quote_id)
                .bind(input.company_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|_| {
                    api_err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "fetch quote totals failed",
                    )
                })?;
        total = row
            .try_get("total")
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "invalid quote total"))?;
        tax_amount = row
            .try_get("tax_amount")
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "invalid quote tax"))?;
    }

    let client_name = fetch_client_name(
        &mut tx,
        input.company_id,
        if client_id.is_empty() {
            None
        } else {
            Some(client_id.as_str())
        },
    )
    .await;

    let invoice_id = uuid::Uuid::new_v4().to_string();
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let invoice_number =
        crate::app::numbering::next_document_number(&mut tx, input.company_id, "INV").await?;

    let currency = match quote_row
        .try_get::<Option<String>, _>("currency")
        .ok()
        .flatten()
        .map(|c| c.trim().to_uppercase())
        .filter(|c| !c.is_empty())
    {
        Some(currency) => currency,
        None => company_base_currency(&mut tx, input.company_id).await,
    };
    let linked_client_id = if client_id.is_empty() {
        None
    } else {
        Some(client_id.as_str())
    };

    sqlx::query(
        "INSERT INTO invoices (id, company_id, invoice_number, invoice_type, amount, tax_amount, status, invoice_date, supplier, description, created_by, currency, client_id) VALUES (?, ?, ?, 'receivable', ?, ?, 'pending', ?, ?, ?, ?, ?, ?)"
    )
    .bind(&invoice_id)
    .bind(input.company_id)
    .bind(&invoice_number)
    .bind(total)
    .bind(tax_amount)
    .bind(&today)
    .bind(&client_name)
    .bind(&format!("From Quote: {} - {}", quote_number, title))
    .bind(input.user_id)
    .bind(&currency)
    .bind(linked_client_id)
    .execute(&mut *tx)
    .await
    .map_err(|error| {
        tracing::error!("Invoice creation error: {:?}", error);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "create invoice failed")
    })?;

    let changed = sqlx::query(
        "UPDATE quotes SET status = 'invoiced', invoice_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? AND company_id = ? AND (invoice_id IS NULL OR invoice_id = '') AND COALESCE(status, '') NOT IN ('invoiced','rejected','expired')"
    )
    .bind(&invoice_id)
    .bind(input.quote_id)
    .bind(input.company_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "update quote failed"))?;
    if changed.rows_affected() != 1 {
        return Err(api_err(StatusCode::CONFLICT, "quote already converted"));
    }

    let log_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO operation_logs (id, company_id, user_id, action, entity_type, entity_id, details) VALUES (?, ?, ?, 'convert', 'quotes', ?, ?)"
    )
    .bind(&log_id)
    .bind(input.company_id)
    .bind(input.user_id)
    .bind(input.quote_id)
    .bind(&format!("Converted to invoice {}", invoice_id))
    .execute(&mut *tx)
    .await
    .ok();

    tx.commit()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "tx commit failed"))?;

    let message = format!(
        "Quote {} converted to invoice {}",
        quote_number, invoice_number
    );

    Ok(ConvertQuoteToInvoiceResult {
        invoice_id,
        invoice_number,
        quote_id: input.quote_id.to_string(),
        quote_number,
        amount: total,
        message,
    })
}

async fn fetch_client_name(
    conn: &mut sqlx::SqliteConnection,
    company_id: &str,
    client_id: Option<&str>,
) -> String {
    let Some(client_id) = client_id.filter(|value| !value.is_empty()) else {
        return String::new();
    };

    sqlx::query("SELECT name FROM clients WHERE id = ? AND company_id = ?")
        .bind(client_id)
        .bind(company_id)
        .fetch_optional(conn)
        .await
        .ok()
        .flatten()
        .and_then(|row| row.try_get("name").ok())
        .unwrap_or_default()
}

async fn company_base_currency(conn: &mut sqlx::SqliteConnection, company_id: &str) -> String {
    let row = sqlx::query("SELECT base_currency FROM companies WHERE id = ?")
        .bind(company_id)
        .fetch_optional(conn)
        .await
        .ok()
        .flatten();
    row.and_then(|r| r.try_get::<Option<String>, _>("base_currency").ok())
        .flatten()
        .map(|c| c.trim().to_uppercase())
        .filter(|c| !c.is_empty())
        .unwrap_or_else(|| "CNY".to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        ConvertQuoteToInvoiceInput, ConvertTimeEntriesToInvoiceInput, convert_quote_to_invoice,
        convert_time_entries_to_invoice, sync_payment_links,
    };
    use crate::db;
    use sqlx::SqlitePool;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("in-memory db");
        db::run_migrations(&pool).await.expect("migrations");
        pool
    }

    async fn seed_company_and_user(pool: &SqlitePool, company_id: &str, user_id: &str) {
        sqlx::query("INSERT INTO companies (id, name) VALUES (?, ?)")
            .bind(company_id)
            .bind("TestCo")
            .execute(pool)
            .await
            .expect("company");

        sqlx::query("INSERT INTO users (id, email, name) VALUES (?, ?, ?)")
            .bind(user_id)
            .bind(format!("{user_id}@example.com"))
            .bind("Test User")
            .execute(pool)
            .await
            .expect("user");
    }

    async fn seed_client(
        pool: &SqlitePool,
        company_id: &str,
        user_id: &str,
        client_id: &str,
        name: &str,
    ) {
        sqlx::query("INSERT INTO clients (id, company_id, name, created_by) VALUES (?, ?, ?, ?)")
            .bind(client_id)
            .bind(company_id)
            .bind(name)
            .bind(user_id)
            .execute(pool)
            .await
            .expect("client");
    }

    #[tokio::test]
    async fn sync_payment_links_marks_invoice_paid_when_total_is_covered() {
        let pool = test_pool().await;
        let company_id = "company-1";
        let user_id = "user-1";
        let invoice_id = "invoice-1";

        seed_company_and_user(&pool, company_id, user_id).await;

        sqlx::query(
            "INSERT INTO invoices (id, company_id, invoice_number, amount, status, created_by) VALUES (?, ?, ?, ?, 'pending', ?)"
        )
        .bind(invoice_id)
        .bind(company_id)
        .bind("INV-001")
        .bind(100.0)
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("invoice");

        sqlx::query(
            "INSERT INTO payments (id, company_id, payment_type, amount, invoice_id, status) VALUES (?, ?, 'incoming', ?, ?, 'completed')"
        )
        .bind("payment-existing")
        .bind(company_id)
        .bind(40.0)
        .bind(invoice_id)
        .execute(&pool)
        .await
        .expect("existing payment");

        sqlx::query(
            "INSERT INTO payments (id, company_id, payment_type, amount, invoice_id, payment_date, status) VALUES (?, ?, 'incoming', ?, ?, ?, 'completed')"
        )
        .bind("payment-new")
        .bind(company_id)
        .bind(60.0)
        .bind(invoice_id)
        .bind("2026-04-16")
        .execute(&pool)
        .await
        .expect("new payment");

        let mut tx = pool.begin().await.expect("tx");

        sync_payment_links(
            &mut tx,
            company_id,
            "payment-new",
            Some("2026-04-16"),
            Some(invoice_id),
            None,
        )
        .await
        .expect("sync payment links");

        tx.commit().await.expect("commit");

        let status: (String,) =
            sqlx::query_as("SELECT status FROM invoices WHERE id = ? AND company_id = ?")
                .bind(invoice_id)
                .bind(company_id)
                .fetch_one(&pool)
                .await
                .expect("invoice status");

        assert_eq!(status.0, "paid");
    }

    #[tokio::test]
    async fn sync_payment_links_marks_partially_paid_and_reverts_on_zero() {
        let pool = test_pool().await;
        let company_id = "company-1";
        let user_id = "user-1";
        let invoice_id = "invoice-1";

        seed_company_and_user(&pool, company_id, user_id).await;

        sqlx::query(
            "INSERT INTO invoices (id, company_id, invoice_number, amount, status, created_by) VALUES (?, ?, ?, ?, 'pending', ?)"
        )
        .bind(invoice_id)
        .bind(company_id)
        .bind("INV-PARTIAL")
        .bind(100.0)
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("invoice");

        sqlx::query(
            "INSERT INTO payments (id, company_id, payment_type, amount, invoice_id, status) VALUES (?, ?, 'income', ?, ?, 'completed')"
        )
        .bind("payment-partial")
        .bind(company_id)
        .bind(30.0)
        .bind(invoice_id)
        .execute(&pool)
        .await
        .expect("partial payment");

        let mut tx = pool.begin().await.expect("tx");
        sync_payment_links(
            &mut tx,
            company_id,
            "payment-partial",
            None,
            Some(invoice_id),
            None,
        )
        .await
        .expect("sync");
        tx.commit().await.expect("commit");

        let status: (String,) = sqlx::query_as("SELECT status FROM invoices WHERE id = ?")
            .bind(invoice_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(status.0, "partially_paid");

        sqlx::query("DELETE FROM payments WHERE id = 'payment-partial'")
            .execute(&pool)
            .await
            .unwrap();

        let mut tx = pool.begin().await.expect("tx");
        sync_payment_links(
            &mut tx,
            company_id,
            "payment-partial",
            None,
            Some(invoice_id),
            None,
        )
        .await
        .expect("sync after delete");
        tx.commit().await.expect("commit");

        let status: (String,) = sqlx::query_as("SELECT status FROM invoices WHERE id = ?")
            .bind(invoice_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(status.0, "pending");
    }

    #[tokio::test]
    async fn sync_payment_links_marks_salary_record_paid_with_payment_metadata() {
        let pool = test_pool().await;
        let company_id = "company-1";
        let user_id = "user-1";
        let employee_id = "employee-1";
        let salary_record_id = "salary-1";

        seed_company_and_user(&pool, company_id, user_id).await;

        sqlx::query("INSERT INTO employees (id, company_id, name) VALUES (?, ?, ?)")
            .bind(employee_id)
            .bind(company_id)
            .bind("Ada Lovelace")
            .execute(&pool)
            .await
            .expect("employee");

        sqlx::query(
            "INSERT INTO salary_records (id, company_id, employee_id, period, net_salary, status) VALUES (?, ?, ?, ?, ?, 'pending')"
        )
        .bind(salary_record_id)
        .bind(company_id)
        .bind(employee_id)
        .bind("2026-04")
        .bind(1000.0)
        .execute(&pool)
        .await
        .expect("salary record");

        sqlx::query(
            "INSERT INTO payments (id, company_id, payment_type, amount, salary_record_id, status) VALUES (?, ?, 'salary', ?, ?, 'completed')"
        )
        .bind("payment-existing")
        .bind(company_id)
        .bind(250.0)
        .bind(salary_record_id)
        .execute(&pool)
        .await
        .expect("existing payment");

        sqlx::query(
            "INSERT INTO payments (id, company_id, payment_type, amount, salary_record_id, payment_date, status) VALUES (?, ?, 'salary', ?, ?, ?, 'completed')"
        )
        .bind("payment-new")
        .bind(company_id)
        .bind(750.0)
        .bind(salary_record_id)
        .bind("2026-04-16")
        .execute(&pool)
        .await
        .expect("new payment");

        let mut tx = pool.begin().await.expect("tx");

        sync_payment_links(
            &mut tx,
            company_id,
            "payment-new",
            Some("2026-04-16"),
            None,
            Some(salary_record_id),
        )
        .await
        .expect("sync payment links");

        tx.commit().await.expect("commit");

        let row: (String, Option<String>, Option<String>) = sqlx::query_as(
            "SELECT status, payment_date, payment_id FROM salary_records WHERE id = ? AND company_id = ?"
        )
        .bind(salary_record_id)
        .bind(company_id)
        .fetch_one(&pool)
        .await
        .expect("salary record");

        assert_eq!(row.0, "paid");
        assert_eq!(row.1.as_deref(), Some("2026-04-16"));
        assert_eq!(row.2.as_deref(), Some("payment-new"));
    }

    #[tokio::test]
    async fn convert_time_entries_to_invoice_creates_invoice_and_links_entries() {
        let pool = test_pool().await;
        let company_id = "company-1";
        let user_id = "user-1";
        let client_id = "client-1";

        seed_company_and_user(&pool, company_id, user_id).await;
        seed_client(&pool, company_id, user_id, client_id, "Acme Corp").await;

        sqlx::query(
            "INSERT INTO time_entries (id, company_id, client_id, description, duration_minutes, date, is_billable, hourly_rate, amount, status, created_by) VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?, 'logged', ?)"
        )
        .bind("time-1")
        .bind(company_id)
        .bind(client_id)
        .bind("Feature workshop")
        .bind(120_i64)
        .bind("2026-04-15")
        .bind(100.0)
        .bind(200.0)
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("time entry 1");

        sqlx::query(
            "INSERT INTO time_entries (id, company_id, client_id, description, duration_minutes, date, is_billable, hourly_rate, amount, status, created_by) VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?, 'logged', ?)"
        )
        .bind("time-2")
        .bind(company_id)
        .bind(client_id)
        .bind("Implementation")
        .bind(60_i64)
        .bind("2026-04-16")
        .bind(150.0)
        .bind(150.0)
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("time entry 2");

        let result = convert_time_entries_to_invoice(
            &pool,
            ConvertTimeEntriesToInvoiceInput {
                company_id,
                user_id,
                project_id: None,
                client_id: Some(client_id),
            },
        )
        .await
        .expect("convert time entries");

        assert_eq!(result.amount, 350.0);
        assert_eq!(result.entries_count, 2);
        assert_eq!(result.total_hours, 3.0);

        let invoice: (String, String, f64, String, String, Option<String>, Option<String>) =
            sqlx::query_as(
            "SELECT invoice_type, status, amount, supplier, description, client_id, currency FROM invoices WHERE id = ? AND company_id = ?"
        )
        .bind(&result.invoice_id)
        .bind(company_id)
        .fetch_one(&pool)
        .await
        .expect("invoice");

        assert_eq!(invoice.0, "receivable");
        assert_eq!(invoice.1, "pending");
        assert_eq!(invoice.2, 350.0);
        assert_eq!(invoice.3, "Acme Corp");
        assert!(invoice.4.contains("Feature workshop"));
        assert_eq!(invoice.5.as_deref(), Some(client_id));
        assert_eq!(invoice.6.as_deref(), Some("CNY"));

        let linked_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM time_entries WHERE company_id = ? AND invoice_id = ? AND status = 'invoiced'"
        )
        .bind(company_id)
        .bind(&result.invoice_id)
        .fetch_one(&pool)
        .await
        .expect("linked count");

        assert_eq!(linked_count.0, 2);
    }

    #[tokio::test]
    async fn convert_quote_to_invoice_creates_invoice_and_marks_quote_invoiced() {
        let pool = test_pool().await;
        let company_id = "company-1";
        let user_id = "user-1";
        let client_id = "client-1";
        let quote_id = "quote-1";

        seed_company_and_user(&pool, company_id, user_id).await;
        seed_client(&pool, company_id, user_id, client_id, "Acme Corp").await;

        sqlx::query(
            "INSERT INTO quotes (id, company_id, client_id, quote_number, title, status, tax_amount, total, created_by) VALUES (?, ?, ?, ?, ?, 'accepted', ?, ?, ?)"
        )
        .bind(quote_id)
        .bind(company_id)
        .bind(client_id)
        .bind("QT-001")
        .bind("Website redesign")
        .bind(30.0)
        .bind(330.0)
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("quote");

        let result = convert_quote_to_invoice(
            &pool,
            ConvertQuoteToInvoiceInput {
                company_id,
                user_id,
                quote_id,
            },
        )
        .await
        .expect("convert quote");

        assert_eq!(result.quote_id, quote_id);
        assert_eq!(result.amount, 330.0);

        let invoice: (String, f64, f64, String, String, Option<String>, Option<String>) =
            sqlx::query_as(
            "SELECT invoice_type, amount, tax_amount, supplier, description, client_id, currency FROM invoices WHERE id = ? AND company_id = ?"
        )
        .bind(&result.invoice_id)
        .bind(company_id)
        .fetch_one(&pool)
        .await
        .expect("invoice");

        assert_eq!(invoice.0, "receivable");
        assert_eq!(invoice.1, 330.0);
        assert_eq!(invoice.2, 30.0);
        assert_eq!(invoice.3, "Acme Corp");
        assert!(invoice.4.contains("QT-001"));
        assert_eq!(invoice.5.as_deref(), Some(client_id));
        assert_eq!(invoice.6.as_deref(), Some("CNY"));

        let quote: (String, Option<String>) =
            sqlx::query_as("SELECT status, invoice_id FROM quotes WHERE id = ? AND company_id = ?")
                .bind(quote_id)
                .bind(company_id)
                .fetch_one(&pool)
                .await
                .expect("quote");

        assert_eq!(quote.0, "invoiced");
        assert_eq!(quote.1.as_deref(), Some(result.invoice_id.as_str()));
    }
    async fn file_test_pool() -> (SqlitePool, std::path::PathBuf) {
        let path =
            std::env::temp_dir().join(format!("opc-conversion-{}.sqlite", uuid::Uuid::new_v4()));
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .busy_timeout(std::time::Duration::from_secs(5));
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .unwrap();
        db::run_migrations(&pool).await.unwrap();
        seed_company_and_user(&pool, "company-1", "user-1").await;
        seed_client(&pool, "company-1", "user-1", "client-1", "Client").await;
        (pool, path)
    }

    #[tokio::test]
    async fn quote_conversion_preserves_foreign_currency() {
        let pool = test_pool().await;
        seed_company_and_user(&pool, "company-1", "user-1").await;
        seed_client(&pool, "company-1", "user-1", "client-1", "Client").await;
        sqlx::query("INSERT INTO quotes(id,company_id,client_id,quote_number,status,total,tax_amount,currency) VALUES ('q','company-1','client-1','Q-USD','accepted',110,10,'USD')")
            .execute(&pool).await.unwrap();
        let result = convert_quote_to_invoice(
            &pool,
            ConvertQuoteToInvoiceInput {
                company_id: "company-1",
                user_id: "user-1",
                quote_id: "q",
            },
        )
        .await
        .unwrap();
        let invoice: (f64, f64, String) =
            sqlx::query_as("SELECT amount,tax_amount,currency FROM invoices WHERE id=?")
                .bind(result.invoice_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(invoice, (110.0, 10.0, "USD".into()));
    }

    #[tokio::test]
    async fn quote_conversion_recalculates_tax_even_when_stale_total_equals_subtotal() {
        let pool = test_pool().await;
        seed_company_and_user(&pool, "company-1", "user-1").await;
        seed_client(&pool, "company-1", "user-1", "client-1", "Client").await;
        sqlx::query("INSERT INTO quotes(id,company_id,client_id,quote_number,status,total,tax_amount,currency,tax_rate) VALUES ('q','company-1','client-1','Q-USD','accepted',100,0,'USD',0.1)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO quote_items(id,company_id,quote_id,description,quantity,unit_price,amount) VALUES ('item','company-1','q','Service',1,100,100)")
            .execute(&pool).await.unwrap();
        let result = convert_quote_to_invoice(
            &pool,
            ConvertQuoteToInvoiceInput {
                company_id: "company-1",
                user_id: "user-1",
                quote_id: "q",
            },
        )
        .await
        .unwrap();
        let invoice: (f64, f64, String) =
            sqlx::query_as("SELECT amount,tax_amount,currency FROM invoices WHERE id=?")
                .bind(result.invoice_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(invoice, (110.0, 10.0, "USD".into()));
    }

    async fn assert_concurrent_conversion_is_single(quote: bool) {
        let (pool, path) = file_test_pool().await;
        if quote {
            sqlx::query("INSERT INTO quotes(id,company_id,client_id,quote_number,status,total) VALUES ('q','company-1','client-1','Q-CONCURRENT','accepted',100)")
                .execute(&pool).await.unwrap();
        } else {
            sqlx::query("INSERT INTO time_entries(id,company_id,client_id,description,date,duration_minutes,amount,is_billable,status) VALUES ('t','company-1','client-1','Work','2026-09-01',60,100,1,'logged')")
                .execute(&pool).await.unwrap();
        }
        // WAL permits both old implementations to read the same source while
        // the writer is held. Correct implementations wait before reading it.
        let blocker = pool.begin_with("BEGIN IMMEDIATE").await.unwrap();
        let mut tasks = Vec::new();
        for _ in 0..2 {
            let pool = pool.clone();
            tasks.push(tokio::spawn(async move {
                if quote {
                    convert_quote_to_invoice(
                        &pool,
                        ConvertQuoteToInvoiceInput {
                            company_id: "company-1",
                            user_id: "user-1",
                            quote_id: "q",
                        },
                    )
                    .await
                    .map(|r| r.invoice_id)
                } else {
                    convert_time_entries_to_invoice(
                        &pool,
                        ConvertTimeEntriesToInvoiceInput {
                            company_id: "company-1",
                            user_id: "user-1",
                            project_id: None,
                            client_id: Some("client-1"),
                        },
                    )
                    .await
                    .map(|r| r.invoice_id)
                }
            }));
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        blocker.rollback().await.unwrap();
        let mut successes = Vec::new();
        let mut errors = Vec::new();
        for task in tasks {
            match tokio::time::timeout(std::time::Duration::from_secs(10), task)
                .await
                .unwrap()
                .unwrap()
            {
                Ok(id) => successes.push(id),
                Err((status, _)) => errors.push(status),
            }
        }
        let count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM invoices WHERE company_id='company-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        let link: String = sqlx::query_scalar(if quote {
            "SELECT invoice_id FROM quotes WHERE id='q'"
        } else {
            "SELECT invoice_id FROM time_entries WHERE id='t'"
        })
        .fetch_one(&pool)
        .await
        .unwrap();
        pool.close().await;
        std::fs::remove_file(path).unwrap();
        assert_eq!(
            count, 1,
            "concurrent conversion created duplicate receivables"
        );
        assert_eq!(successes, vec![link]);
        assert_eq!(
            errors,
            vec![if quote {
                axum::http::StatusCode::CONFLICT
            } else {
                axum::http::StatusCode::BAD_REQUEST
            }]
        );
    }

    #[tokio::test]
    async fn quote_conversion_concurrent_requests_create_one_invoice() {
        assert_concurrent_conversion_is_single(true).await;
    }

    #[tokio::test]
    async fn time_conversion_concurrent_requests_create_one_invoice() {
        assert_concurrent_conversion_is_single(false).await;
    }
}
