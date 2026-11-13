use super::error::{ApiError, api_err};
use crate::auth::AuthUser;
use crate::pdf_fonts::pdf_font_family;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::get,
};
use chrono::{Datelike, NaiveDate, Utc};
use genpdfi::Element;
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::{Row, SqlitePool};
use std::path::PathBuf;
use uuid::Uuid;

const UPLOAD_DIR: &str = "data/uploads";

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route(
            "/reports/monthly-close",
            get(list_monthly_closes).post(create_monthly_close),
        )
        .with_state(pool)
}

#[derive(Debug, Deserialize, Default)]
struct PeriodQuery {
    period: Option<String>,
}

#[derive(Debug)]
struct PeriodWindow {
    label: String,
    start: NaiveDate,
    end: NaiveDate,
    prefix: String,
}

fn parse_period(raw: Option<&str>) -> Result<PeriodWindow, String> {
    let label = raw
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| Utc::now().format("%Y-%m").to_string());
    let start = NaiveDate::parse_from_str(&format!("{label}-01"), "%Y-%m-%d")
        .map_err(|_| "period must be YYYY-MM".to_string())?;
    let end = if start.month() == 12 {
        NaiveDate::from_ymd_opt(start.year() + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(start.year(), start.month() + 1, 1)
    }
    .and_then(|next| next.pred_opt())
    .ok_or_else(|| "invalid period".to_string())?;
    Ok(PeriodWindow {
        prefix: format!("{label}%"),
        label,
        start,
        end,
    })
}

fn money(amount: f64) -> String {
    format!("¥{amount:.2}")
}

async fn scalar_f64(pool: &SqlitePool, sql: &str, company_id: &str, prefix: &str) -> f64 {
    sqlx::query_scalar::<_, f64>(sql)
        .bind(company_id)
        .bind(prefix)
        .fetch_one(pool)
        .await
        .unwrap_or(0.0)
}

async fn create_monthly_close(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(query): Query<PeriodQuery>,
) -> Result<Json<Value>, ApiError> {
    let window = match parse_period(query.period.as_deref()) {
        Ok(window) => window,
        Err(message) => return Err(api_err(StatusCode::BAD_REQUEST, &message)),
    };
    let company_id = &user.company_id;
    let prefix = &window.prefix;

    let company_name: String = sqlx::query_scalar("SELECT name FROM companies WHERE id = ?")
        .bind(company_id)
        .fetch_optional(&pool)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "OPC".to_string());

    let income = scalar_f64(
        &pool,
        "SELECT COALESCE(SUM(amount), 0) FROM payments \
         WHERE status IN ('completed', 'paid') AND company_id = ? AND payment_type = 'income' AND payment_date LIKE ?",
        company_id,
        prefix,
    )
    .await;
    let expense = scalar_f64(
        &pool,
        "SELECT COALESCE(SUM(amount), 0) FROM payments \
         WHERE status IN ('completed', 'paid') AND company_id = ? AND payment_type = 'expense' AND payment_date LIKE ?",
        company_id,
        prefix,
    )
    .await;
    let billed = scalar_f64(
        &pool,
        "SELECT COALESCE(SUM(amount), 0) FROM invoices \
         WHERE company_id = ? AND invoice_type = 'receivable' AND invoice_date LIKE ?",
        company_id,
        prefix,
    )
    .await;
    let outstanding = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(SUM(amount), 0) FROM invoices \
         WHERE company_id = ? AND invoice_type = 'receivable' \
           AND status IN ('pending', 'overdue', 'sent', 'partially_paid')",
    )
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(0.0);
    let overdue = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(SUM(amount), 0) FROM invoices \
         WHERE company_id = ? AND invoice_type = 'receivable' \
           AND status IN ('overdue', 'pending', 'sent', 'partially_paid') \
           AND due_date IS NOT NULL AND due_date != '' AND due_date < ?",
    )
    .bind(company_id)
    .bind(window.end.to_string())
    .fetch_one(&pool)
    .await
    .unwrap_or(0.0);

    let tax_due = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(SUM(tax_amount), 0) FROM tax_records \
         WHERE company_id = ? AND status IN ('pending', 'due', 'unpaid') \
           AND (period LIKE ? OR due_date LIKE ?)",
    )
    .bind(company_id)
    .bind(prefix)
    .bind(prefix)
    .fetch_one(&pool)
    .await
    .unwrap_or(0.0);
    let tax_paid = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(SUM(tax_amount), 0) FROM tax_records \
         WHERE company_id = ? AND status IN ('paid', 'filed', 'declared') \
           AND (period LIKE ? OR payment_date LIKE ?)",
    )
    .bind(company_id)
    .bind(prefix)
    .bind(prefix)
    .fetch_one(&pool)
    .await
    .unwrap_or(0.0);

    let bank_total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM bank_transactions \
         WHERE company_id = ? AND transaction_date LIKE ?",
    )
    .bind(company_id)
    .bind(prefix)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);
    let bank_unmatched = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM bank_transactions \
         WHERE company_id = ? AND transaction_date LIKE ? \
           AND (payment_id IS NULL OR payment_id = '')",
    )
    .bind(company_id)
    .bind(prefix)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    let empty = income == 0.0
        && expense == 0.0
        && billed == 0.0
        && outstanding == 0.0
        && tax_due == 0.0
        && tax_paid == 0.0
        && bank_total == 0;
    let net = income - expense;

    let pdf = render_monthly_close_pdf(
        &company_name,
        &window,
        income,
        expense,
        net,
        billed,
        outstanding,
        overdue,
        tax_due,
        tax_paid,
        bank_total,
        bank_unmatched,
        empty,
    )?;

    let file = archive_monthly_close_pdf(&pool, &user, &window.label, &pdf).await?;

    sqlx::query(
        "INSERT INTO operation_logs \
         (id, company_id, user_id, action, entity_type, entity_id, details) \
         VALUES (?, ?, ?, 'create', 'monthly_report', ?, ?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .bind(&user.user_id)
    .bind(&window.label)
    .bind(
        json!({
            "income": income,
            "expense": expense,
            "file_id": file["id"],
        })
        .to_string(),
    )
    .execute(&pool)
    .await
    .ok();

    Ok(Json(json!({
        "period": window.label,
        "period_start": window.start.to_string(),
        "period_end": window.end.to_string(),
        "empty": empty,
        "income": income,
        "income_label": money(income),
        "expense": expense,
        "expense_label": money(expense),
        "net": net,
        "net_label": money(net),
        "billed": billed,
        "billed_label": money(billed),
        "outstanding": outstanding,
        "outstanding_label": money(outstanding),
        "overdue": overdue,
        "overdue_label": money(overdue),
        "tax_due": tax_due,
        "tax_due_label": money(tax_due),
        "tax_paid": tax_paid,
        "tax_paid_label": money(tax_paid),
        "bank_transactions": bank_total,
        "bank_unmatched": bank_unmatched,
        "file": file,
    })))
}

async fn list_monthly_closes(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<Value>, ApiError> {
    let rows = sqlx::query(
        "SELECT id, original_name, size, entity_id, description, created_at \
         FROM files WHERE company_id = ? AND entity_type = 'monthly_report' \
         ORDER BY created_at DESC",
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "list reports failed"))?;

    let data = rows
        .into_iter()
        .map(|row| {
            let id: String = row.get("id");
            json!({
                "id": id,
                "period": row.get::<String, _>("entity_id"),
                "original_name": row.get::<String, _>("original_name"),
                "size": row.get::<i64, _>("size"),
                "description": row.get::<Option<String>, _>("description"),
                "created_at": row.get::<Option<String>, _>("created_at"),
                "url": format!("/api/v1/files/{id}"),
            })
        })
        .collect::<Vec<_>>();
    Ok(Json(json!({ "data": data })))
}

fn render_monthly_close_pdf(
    company_name: &str,
    window: &PeriodWindow,
    income: f64,
    expense: f64,
    net: f64,
    billed: f64,
    outstanding: f64,
    overdue: f64,
    tax_due: f64,
    tax_paid: f64,
    bank_total: i64,
    bank_unmatched: i64,
    empty: bool,
) -> Result<Vec<u8>, ApiError> {
    let font_family = pdf_font_family().map_err(|error| {
        tracing::error!("Font loading error: {error:?}");
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "no fonts available for PDF generation",
        )
    })?;
    let mut doc = genpdfi::Document::new(font_family);
    doc.set_title(format!("OPC 月度关账 {}", window.label));
    doc.set_minimal_conformance();
    let mut decorator = genpdfi::SimplePageDecorator::new();
    decorator.set_margins(16);
    doc.set_page_decorator(decorator);

    let heading = |text: &str, size: u8| {
        genpdfi::elements::Paragraph::new(text)
            .styled(genpdfi::style::Style::new().bold().with_font_size(size))
    };
    let line = |text: String| {
        genpdfi::elements::Paragraph::new(text)
            .styled(genpdfi::style::Style::new().with_font_size(11))
    };

    doc.push(heading(company_name, 18));
    doc.push(heading(&format!("月度关账包  {}", window.label), 14));
    doc.push(line(format!("期间：{} 至 {}", window.start, window.end)));
    doc.push(genpdfi::elements::Break::new(0.6));

    if empty {
        doc.push(line(
            "本月暂无流水。以下为占位关账报告，便于按月归档。".into(),
        ));
        doc.push(genpdfi::elements::Break::new(0.4));
    }

    doc.push(heading("1. 收支与毛利", 13));
    doc.push(line(format!("收入  {}", money(income))));
    doc.push(line(format!("支出  {}", money(expense))));
    doc.push(line(format!("净额  {}", money(net))));
    doc.push(genpdfi::elements::Break::new(0.4));

    doc.push(heading("2. 应收账龄快照", 13));
    doc.push(line(format!("本月开出应收  {}", money(billed))));
    doc.push(line(format!("未结应收合计  {}", money(outstanding))));
    doc.push(line(format!("其中已逾期    {}", money(overdue))));
    doc.push(genpdfi::elements::Break::new(0.4));

    doc.push(heading("3. 税务", 13));
    doc.push(line(format!("待缴/待申报  {}", money(tax_due))));
    doc.push(line(format!("本月已缴/已报  {}", money(tax_paid))));
    doc.push(genpdfi::elements::Break::new(0.4));

    doc.push(heading("4. 银行流水核对", 13));
    doc.push(line(format!("本月流水 {bank_total} 笔")));
    doc.push(line(format!("尚未关联付款 {bank_unmatched} 笔")));
    doc.push(genpdfi::elements::Break::new(0.4));

    doc.push(heading("5. 现金流摘要", 13));
    doc.push(line(format!(
        "经营净额 {}（收入减支出，未含应收兑现）",
        money(net)
    )));
    doc.push(genpdfi::elements::Break::new(0.8));
    doc.push(line(format!(
        "Generated by OPC | {}",
        Utc::now().format("%Y-%m-%d %H:%M UTC")
    )));

    let mut buf = Vec::new();
    doc.render(&mut buf).map_err(|error| {
        tracing::error!("PDF render error: {error:?}");
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "PDF generation failed")
    })?;
    Ok(buf)
}

async fn archive_monthly_close_pdf(
    pool: &SqlitePool,
    user: &AuthUser,
    period: &str,
    pdf: &[u8],
) -> Result<Value, ApiError> {
    let upload_path = PathBuf::from(UPLOAD_DIR).join(&user.company_id);
    tokio::fs::create_dir_all(&upload_path).await.map_err(|_| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "create upload dir failed",
        )
    })?;
    let hash = w_crypto::sha256_hex(pdf);
    let file_path = upload_path.join(&hash);
    if !file_path.exists() {
        tokio::fs::write(&file_path, pdf)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "write report failed"))?;
    }

    let file_id = Uuid::new_v4().to_string();
    let original_name = format!("opc-monthly-close-{period}.pdf");
    sqlx::query(
        "INSERT INTO files (id, company_id, filename, original_name, content_type, size, entity_type, entity_id, description, uploaded_by) \
         VALUES (?, ?, ?, ?, 'application/pdf', ?, 'monthly_report', ?, ?, ?)",
    )
    .bind(&file_id)
    .bind(&user.company_id)
    .bind(&hash)
    .bind(&original_name)
    .bind(pdf.len() as i64)
    .bind(period)
    .bind(format!("OPC monthly close {period}"))
    .bind(&user.user_id)
    .execute(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "archive report failed"))?;

    Ok(json!({
        "id": file_id,
        "original_name": original_name,
        "size": pdf.len(),
        "url": format!("/api/v1/files/{file_id}"),
    }))
}
