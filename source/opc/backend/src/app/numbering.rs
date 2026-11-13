//! Company-scoped document sequence service.
//! Formats: INV-YYYY-NNNN / QUO-YYYY-NNNN (per-year, per-company).

use axum::http::StatusCode;
use chrono::Datelike;
use sqlx::{Sqlite, Transaction};

use crate::api::error::{ApiError, api_err};

/// Allocate the next document number inside an open transaction.
/// `doc_type` is the short prefix: `"INV"` or `"QUO"`.
pub async fn next_document_number(
    tx: &mut Transaction<'_, Sqlite>,
    company_id: &str,
    doc_type: &str,
) -> Result<String, ApiError> {
    let year = local_year();
    // Atomic allocate: insert next_seq=2 (so first RETURNING next_seq-1 = 1),
    // or on conflict increment and return previous value.
    let seq: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO document_sequences (company_id, doc_type, year, next_seq)
        VALUES (?, ?, ?, 2)
        ON CONFLICT(company_id, doc_type, year)
        DO UPDATE SET next_seq = next_seq + 1
        RETURNING next_seq - 1
        "#,
    )
    .bind(company_id)
    .bind(doc_type)
    .bind(year)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("document sequence allocate failed: {:?}", e);
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "document number allocation failed",
        )
    })?;

    Ok(format_document_number(doc_type, year, seq))
}

/// Peek the next number without consuming (for UI preview).
pub async fn peek_document_number(
    pool: &sqlx::SqlitePool,
    company_id: &str,
    doc_type: &str,
) -> Result<String, ApiError> {
    let year = local_year();
    let seq: Option<i64> = sqlx::query_scalar(
        "SELECT next_seq FROM document_sequences WHERE company_id = ? AND doc_type = ? AND year = ?",
    )
    .bind(company_id)
    .bind(doc_type)
    .bind(year)
    .fetch_optional(pool)
    .await
    .map_err(|_| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "document number peek failed",
        )
    })?;

    Ok(format_document_number(doc_type, year, seq.unwrap_or(1)))
}

pub fn format_document_number(doc_type: &str, year: i32, seq: i64) -> String {
    format!("{}-{}-{:04}", doc_type, year, seq)
}

pub fn local_year() -> i32 {
    chrono::Local::now().year()
}

/// Recalculate quote subtotal / tax / total from line items.
/// `total = (subtotal - discount) * (1 + tax_rate)` when tax_rate is a fraction (e.g. 0.06),
/// or subtotal sum of line amounts when no tax.
pub async fn recalc_quote_total(
    pool: &sqlx::SqlitePool,
    company_id: &str,
    quote_id: &str,
) -> Result<(), ApiError> {
    let mut conn = pool
        .acquire()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "quote connection failed"))?;
    recalc_quote_total_connection(&mut conn, company_id, quote_id).await
}

pub(crate) async fn recalc_quote_total_connection(
    conn: &mut sqlx::SqliteConnection,
    company_id: &str,
    quote_id: &str,
) -> Result<(), ApiError> {
    let quote: Option<(f64, f64)> = sqlx::query_as(
        "SELECT COALESCE(tax_rate, 0), COALESCE(discount, 0) FROM quotes WHERE id = ? AND company_id = ?",
    )
    .bind(quote_id)
    .bind(company_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "quote lookup failed"))?;

    let Some((tax_rate, discount)) = quote else {
        return Ok(());
    };

    let (subtotal,): (f64,) = sqlx::query_as(
        r#"
        SELECT CAST(COALESCE(SUM(
            CASE
                WHEN amount IS NOT NULL AND amount != 0 THEN amount
                ELSE COALESCE(quantity, 1) * COALESCE(unit_price, 0)
                     * (1.0 - COALESCE(discount_percent, 0) / 100.0)
            END
        ), 0) AS REAL)
        FROM quote_items
        WHERE quote_id = ? AND company_id = ?
        "#,
    )
    .bind(quote_id)
    .bind(company_id)
    .fetch_one(&mut *conn)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "quote items sum failed"))?;

    let taxable = (subtotal - discount).max(0.0);
    // tax_rate stored as fraction (0.06) or percent (6); treat > 1 as percent.
    let rate = if tax_rate > 1.0 {
        tax_rate / 100.0
    } else {
        tax_rate
    };
    let tax_amount = taxable * rate;
    let total = taxable + tax_amount;

    sqlx::query(
        r#"
        UPDATE quotes
        SET subtotal = ?, tax_amount = ?, total = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ? AND company_id = ?
        "#,
    )
    .bind(round2(subtotal))
    .bind(round2(tax_amount))
    .bind(round2(total))
    .bind(quote_id)
    .bind(company_id)
    .execute(&mut *conn)
    .await
    .map_err(|_| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "quote total update failed",
        )
    })?;

    Ok(())
}

/// Sum of quote line items (amount column preferred, else qty*price).
pub async fn quote_items_sum(
    pool: &sqlx::SqlitePool,
    company_id: &str,
    quote_id: &str,
) -> Result<(i64, f64), ApiError> {
    let mut conn = pool
        .acquire()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "quote connection failed"))?;
    quote_items_sum_connection(&mut conn, company_id, quote_id).await
}

pub(crate) async fn quote_items_sum_connection(
    conn: &mut sqlx::SqliteConnection,
    company_id: &str,
    quote_id: &str,
) -> Result<(i64, f64), ApiError> {
    let row: (i64, f64) = sqlx::query_as(
        r#"
        SELECT COUNT(*), CAST(COALESCE(SUM(
            CASE
                WHEN amount IS NOT NULL AND amount != 0 THEN amount
                ELSE COALESCE(quantity, 1) * COALESCE(unit_price, 0)
                     * (1.0 - COALESCE(discount_percent, 0) / 100.0)
            END
        ), 0) AS REAL)
        FROM quote_items
        WHERE quote_id = ? AND company_id = ?
        "#,
    )
    .bind(quote_id)
    .bind(company_id)
    .fetch_one(&mut *conn)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "quote items sum failed"))?;
    Ok(row)
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use sqlx::SqlitePool;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("in-memory db");
        db::run_migrations(&pool).await.expect("migrations");
        pool
    }

    async fn seed_company(pool: &SqlitePool, id: &str) {
        sqlx::query("INSERT INTO companies (id, name) VALUES (?, ?)")
            .bind(id)
            .bind("TestCo")
            .execute(pool)
            .await
            .expect("company");
    }

    #[tokio::test]
    async fn next_document_number_increments_per_company_and_type() {
        let pool = test_pool().await;
        seed_company(&pool, "c1").await;
        seed_company(&pool, "c2").await;
        let year = local_year();

        let mut tx = pool.begin().await.unwrap();
        let a = next_document_number(&mut tx, "c1", "INV").await.unwrap();
        let b = next_document_number(&mut tx, "c1", "INV").await.unwrap();
        let c = next_document_number(&mut tx, "c1", "INV").await.unwrap();
        let d = next_document_number(&mut tx, "c2", "INV").await.unwrap();
        let e = next_document_number(&mut tx, "c1", "QUO").await.unwrap();
        tx.commit().await.unwrap();

        assert_eq!(a, format!("INV-{}-0001", year));
        assert_eq!(b, format!("INV-{}-0002", year));
        assert_eq!(c, format!("INV-{}-0003", year));
        assert_eq!(d, format!("INV-{}-0001", year));
        assert_eq!(e, format!("QUO-{}-0001", year));
    }

    #[tokio::test]
    async fn concurrent_allocate_produces_unique_numbers() {
        let pool = test_pool().await;
        seed_company(&pool, "c1").await;

        let mut handles = Vec::new();
        for _ in 0..10 {
            let pool = pool.clone();
            handles.push(tokio::spawn(async move {
                let mut tx = pool.begin().await.unwrap();
                let n = next_document_number(&mut tx, "c1", "INV").await.unwrap();
                tx.commit().await.unwrap();
                n
            }));
        }

        let mut nums = Vec::new();
        for h in handles {
            nums.push(h.await.unwrap());
        }
        nums.sort();
        nums.dedup();
        assert_eq!(nums.len(), 10, "expected 10 unique numbers, got {:?}", nums);
    }

    #[tokio::test]
    async fn year_rows_are_independent() {
        let pool = test_pool().await;
        seed_company(&pool, "c1").await;

        sqlx::query(
            "INSERT INTO document_sequences (company_id, doc_type, year, next_seq) VALUES (?, 'INV', 2025, 99)",
        )
        .bind("c1")
        .execute(&pool)
        .await
        .unwrap();

        let mut tx = pool.begin().await.unwrap();
        let n = next_document_number(&mut tx, "c1", "INV").await.unwrap();
        tx.commit().await.unwrap();

        let year = local_year();
        if year != 2025 {
            assert_eq!(n, format!("INV-{}-0001", year));
        }
    }

    #[tokio::test]
    async fn recalc_quote_total_from_items() {
        let pool = test_pool().await;
        seed_company(&pool, "c1").await;
        sqlx::query("INSERT INTO clients (id, company_id, name) VALUES ('cl1', 'c1', 'Acme')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO quotes (id, company_id, client_id, quote_number, tax_rate, discount, total) VALUES ('q1', 'c1', 'cl1', 'QUO-1', 0.1, 0, 999)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO quote_items (id, company_id, quote_id, description, quantity, unit_price, amount) VALUES ('i1', 'c1', 'q1', 'A', 1, 100, 100)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO quote_items (id, company_id, quote_id, description, quantity, unit_price, amount) VALUES ('i2', 'c1', 'q1', 'B', 2, 100, 200)",
        )
        .execute(&pool)
        .await
        .unwrap();

        recalc_quote_total(&pool, "c1", "q1").await.unwrap();

        let row: (f64, f64, f64) =
            sqlx::query_as("SELECT subtotal, tax_amount, total FROM quotes WHERE id = 'q1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!((row.0 - 300.0).abs() < 0.01);
        assert!((row.1 - 30.0).abs() < 0.01);
        assert!((row.2 - 330.0).abs() < 0.01);
    }
}
