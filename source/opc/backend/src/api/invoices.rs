use super::error::{ApiError, api_err};
use crate::auth::AuthUser;
use crate::db::models::Invoice;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use std::collections::HashMap;

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/", get(list_invoices).post(create_invoice))
        .route("/overview", get(invoices_overview))
        .route("/form_options", get(invoice_form_options))
        .route("/{id}/mark_sent", post(mark_invoice_sent))
        .route("/{id}/mark_paid", post(mark_invoice_paid))
        .route("/{id}/record-payment", post(record_invoice_payment))
        .route(
            "/{id}",
            get(get_invoice).put(update_invoice).delete(delete_invoice),
        )
        .with_state(pool)
}

#[derive(Deserialize)]
pub struct CreateInvoiceRequest {
    /// Optional; empty → auto-allocate INV-YYYY-NNNN.
    pub invoice_number: Option<String>,
    pub invoice_type: Option<String>,
    pub amount: Value,
    pub tax_amount: Option<Value>,
    pub invoice_date: Option<String>,
    pub due_date: Option<String>,
    pub supplier: Option<String>,
    pub description: Option<String>,
    pub file_path: Option<String>,
    pub contract_id: Option<String>,
    /// Optional customer link (receivable/销项). Display name still uses `supplier`.
    pub client_id: Option<String>,
    /// ISO currency code; defaults to company base_currency then CNY.
    pub currency: Option<String>,
}

#[derive(Deserialize)]
pub struct RecordInvoicePaymentRequest {
    /// Optional payment amount; defaults to remaining balance.
    pub amount: Option<f64>,
    pub payment_date: Option<String>,
    pub note: Option<String>,
}

#[derive(Serialize)]
pub struct InvoicesResponse {
    pub data: Vec<Invoice>,
    pub count: i64,
}

#[derive(Default, Serialize)]
pub struct InvoiceStats {
    pub total: i64,
    pub pending: i64,
    pub sent: i64,
    pub paid: i64,
    pub partially_paid: i64,
    pub total_amount: f64,
    pub total_amount_label: String,
    pub total_tax: f64,
    pub total_tax_label: String,
    pub pending_message: String,
}

#[derive(Serialize)]
pub struct InvoicesOverviewResponse {
    pub invoices: Vec<Value>,
    pub output_invoices: Vec<Value>,
    pub input_invoices: Vec<Value>,
    pub output_stats: InvoiceStats,
    pub input_stats: InvoiceStats,
    pub status_options: Vec<Value>,
    pub permissions: Value,
    pub empty_messages: Value,
    pub create_action: Value,
    pub import_preview_action: Value,
    pub import_commit_action: Value,
}

#[derive(Deserialize)]
struct InvoicesOverviewQuery {
    search: Option<String>,
    status: Option<String>,
}

async fn list_invoices(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(q): Query<super::generic::ListQuery>,
) -> Result<Json<InvoicesResponse>, ApiError> {
    let limit = q.limit.or(q.page_size).unwrap_or(100).min(1000).max(1);
    let offset = q
        .offset
        .or_else(|| q.page.map(|p| (p.max(1) - 1) * limit))
        .unwrap_or(0)
        .max(0);
    let invoices: Vec<Invoice> = sqlx::query_as(
        "SELECT * FROM invoices WHERE company_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
    )
    .bind(&user.company_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM invoices WHERE company_id = ?")
        .bind(&user.company_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(Json(InvoicesResponse {
        data: invoices,
        count,
    }))
}

async fn invoices_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(query): Query<InvoicesOverviewQuery>,
) -> Result<Json<InvoicesOverviewResponse>, ApiError> {
    let can_manage = current_user_can_manage_invoice(&pool, &user).await?;
    let invoices: Vec<Invoice> =
        sqlx::query_as("SELECT * FROM invoices WHERE company_id = ? ORDER BY created_at DESC")
            .bind(&user.company_id)
            .fetch_all(&pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let mut all_invoices = Vec::new();
    let mut output_invoices = Vec::new();
    let mut input_invoices = Vec::new();
    let mut output_stats = InvoiceStats::default();
    let mut input_stats = InvoiceStats::default();
    let file_names = invoice_file_names(&pool, &user.company_id).await?;
    let paid_map = invoice_paid_totals(&pool, &user.company_id).await?;
    let search = query
        .search
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .to_lowercase();
    let status_filter = query.status.as_deref().map(str::trim).unwrap_or("all");

    for invoice in &invoices {
        let paid_total = paid_map.get(&invoice.id).copied().unwrap_or(0.0);
        let row = invoice_overview_row(invoice, can_manage, &user.user_id, &file_names, paid_total);
        let matches_filters = invoice_matches_filters(invoice, &search, status_filter);
        if invoice_is_input(invoice) {
            apply_invoice_stats(&mut input_stats, invoice);
            if matches_filters {
                input_invoices.push(row.clone());
            }
        } else {
            apply_invoice_stats(&mut output_stats, invoice);
            if matches_filters {
                output_invoices.push(row.clone());
            }
        }
        if matches_filters {
            all_invoices.push(row);
        }
    }
    finalize_invoice_stats(&mut output_stats);
    finalize_invoice_stats(&mut input_stats);

    Ok(Json(InvoicesOverviewResponse {
        invoices: all_invoices,
        output_invoices,
        input_invoices,
        output_stats,
        input_stats,
        status_options: invoice_status_options(),
        permissions: json!({
            "can_create": true,
            "can_import": true,
            "can_manage": can_manage,
        }),
        empty_messages: json!({
            "filtered": "没有找到匹配的发票",
            "output": "暂无销项发票，点击上方按钮添加",
            "input": "暂无进项发票，点击上方按钮添加",
        }),
        create_action: invoice_create_action(true),
        import_preview_action: invoice_import_preview_action(true),
        import_commit_action: invoice_import_commit_action(true),
    }))
}

async fn invoice_form_options(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<Value>, ApiError> {
    // Customer dropdown for receivable (销项) manual invoices. Payable keeps free-text supplier.
    let client_rows: Vec<(String, String)> =
        sqlx::query_as("SELECT id, name FROM clients WHERE company_id = ? ORDER BY name ASC")
            .bind(&user.company_id)
            .fetch_all(&pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    let client_options: Vec<Value> = client_rows
        .into_iter()
        .map(|(id, name)| {
            json!({
                "value": id,
                "label": name,
                "name": name,
            })
        })
        .collect();

    Ok(Json(json!({
        "status_options": invoice_form_status_options(),
        "invoice_type_options": invoice_type_options(),
        "client_options": client_options,
        "default_status": "pending",
        "default_invoice_type": "receivable",
    })))
}

/// Canonical invoice axis: receivable (AR/销项) vs payable (AP/进项).
/// Accepts legacy aliases from older UI/clients and seed data.
pub fn normalize_invoice_type(raw: Option<&str>) -> Option<String> {
    let value = raw?.trim();
    if value.is_empty() {
        return None;
    }
    match value.to_lowercase().as_str() {
        "receivable" | "output" | "income" | "销项" | "销项发票" => {
            Some("receivable".to_string())
        }
        "payable" | "input" | "expense" | "进项" | "进项发票" => Some("payable".to_string()),
        other => Some(other.to_string()),
    }
}

fn invoice_is_input(invoice: &Invoice) -> bool {
    matches!(
        invoice
            .invoice_type
            .as_deref()
            .unwrap_or("")
            .to_lowercase()
            .as_str(),
        "input" | "expense" | "payable" | "进项" | "进项发票"
    )
}

fn apply_invoice_stats(stats: &mut InvoiceStats, invoice: &Invoice) {
    stats.total += 1;
    stats.total_amount += invoice.amount;
    stats.total_tax += invoice.tax_amount.unwrap_or(0.0);

    match invoice.status.as_deref().unwrap_or("pending") {
        "pending" => stats.pending += 1,
        "sent" | "approved" => stats.sent += 1,
        "paid" => stats.paid += 1,
        // Partial receipts still count as outstanding (待收).
        "partially_paid" => {
            stats.partially_paid += 1;
            stats.pending += 1;
        }
        "overdue" => stats.pending += 1,
        _ => {}
    }
}

fn finalize_invoice_stats(stats: &mut InvoiceStats) {
    stats.total_amount = round_money(stats.total_amount);
    stats.total_tax = round_money(stats.total_tax);
    stats.total_amount_label = format_cny(stats.total_amount);
    stats.total_tax_label = format_cny(stats.total_tax);
    stats.pending_message = format!("共 {} 张待审批", stats.pending);
}

fn invoice_matches_filters(invoice: &Invoice, search: &str, status_filter: &str) -> bool {
    let status = invoice.status.as_deref().unwrap_or("pending");
    let matches_status = status_filter == "all" || status == status_filter;
    let matches_search = search.is_empty()
        || invoice.invoice_number.to_lowercase().contains(search)
        || invoice
            .description
            .as_deref()
            .unwrap_or("")
            .to_lowercase()
            .contains(search)
        || invoice
            .supplier
            .as_deref()
            .unwrap_or("")
            .to_lowercase()
            .contains(search);

    matches_status && matches_search
}

async fn invoice_paid_totals(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<HashMap<String, f64>, ApiError> {
    let rows: Vec<(String, f64)> = sqlx::query_as(
        "SELECT invoice_id, COALESCE(SUM(amount), 0) FROM payments WHERE status IN ('completed', 'paid') AND company_id = ? AND invoice_id IS NOT NULL AND invoice_id != '' GROUP BY invoice_id",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    Ok(rows.into_iter().collect())
}

async fn invoice_file_names(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<HashMap<String, String>, ApiError> {
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT id, original_name FROM files WHERE company_id = ?")
            .bind(company_id)
            .fetch_all(pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(rows.into_iter().collect())
}

fn invoice_file_name(
    file_path: Option<&str>,
    file_names: &HashMap<String, String>,
) -> Option<String> {
    let value = file_path?.trim();
    if value.is_empty() {
        return None;
    }

    file_names.get(value).cloned().or_else(|| {
        value
            .rsplit('/')
            .next()
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_string)
    })
}

fn invoice_overview_row(
    invoice: &Invoice,
    can_manage: bool,
    user_id: &str,
    file_names: &HashMap<String, String>,
    paid_total: f64,
) -> Value {
    let status = invoice.status.as_deref().unwrap_or("pending");
    let can_modify = invoice_can_modify(can_manage, user_id, invoice.created_by.as_deref());
    let can_mark_sent =
        can_modify && !invoice_is_input(invoice) && matches!(status, "pending" | "approved");
    let remaining = (invoice.amount - paid_total).max(0.0);
    let can_mark_paid = can_modify
        && matches!(
            status,
            "pending" | "approved" | "sent" | "overdue" | "partially_paid"
        );
    let can_record_payment = can_mark_paid && !invoice_is_input(invoice) && remaining > 0.01;
    let file_name = invoice_file_name(invoice.file_path.as_deref(), file_names);
    json!({
        "id": &invoice.id,
        "invoice_number": &invoice.invoice_number,
        "invoice_type": &invoice.invoice_type,
        "amount": round_money(invoice.amount),
        "amount_label": format_cny(invoice.amount),
        "tax_amount": invoice.tax_amount.map(round_money),
        "tax_amount_label": invoice.tax_amount.map(format_cny).unwrap_or_else(|| "-".to_string()),
        "total_with_tax_label": format_cny(invoice.amount + invoice.tax_amount.unwrap_or(0.0)),
        "paid_total": round_money(paid_total),
        "paid_total_label": format_cny(paid_total),
        "remaining": round_money(remaining),
        "remaining_label": format_cny(remaining),
        "invoice_date": &invoice.invoice_date,
        "invoice_date_label": invoice_date_label(invoice.invoice_date.as_deref()),
        "due_date": &invoice.due_date,
        "due_date_label": invoice_date_label(invoice.due_date.as_deref()),
        "status": status,
        "status_badge": invoice_status_badge(status),
        "supplier": &invoice.supplier,
        "client_id": &invoice.client_id,
        "currency": invoice.currency.as_deref().unwrap_or("CNY"),
        "description": &invoice.description,
        "file_path": &invoice.file_path,
        "file_name": file_name.clone(),
        "file_label": file_name,
        "has_file": file_name.is_some(),
        "created_by": &invoice.created_by,
        "created_at": &invoice.created_at,
        "created_at_label": invoice_datetime_label(invoice.created_at.as_deref()),
        "updated_at": &invoice.updated_at,
        "can_edit": can_modify,
        "update_action": invoice_update_action(invoice, can_modify),
        "can_delete": can_modify,
        "delete_action": invoice_delete_action(invoice, can_modify),
        "can_mark_sent": can_mark_sent,
        "can_mark_paid": can_mark_paid,
        "can_record_payment": can_record_payment,
        "mark_sent_action": invoice_mark_sent_action(invoice, status, can_modify),
        "mark_paid_action": invoice_mark_paid_action(invoice, status, can_modify),
        "record_payment_action": invoice_record_payment_action(
            invoice, status, can_modify, paid_total, remaining
        ),
    })
}

fn invoice_create_action(can_create: bool) -> Value {
    if !can_create {
        return json!(null);
    }

    json!({
        "method": "POST",
        "endpoint": "/api/invoices",
        "label": "添加发票",
        "success_title": "创建成功",
        "success_message": "发票添加成功",
    })
}

fn invoice_import_preview_action(can_import: bool) -> Value {
    if !can_import {
        return json!(null);
    }
    json!({
        "method": "POST",
        "endpoint": "/api/v1/imports/invoices/preview",
        "label": "预览导入",
        "success_message": "发票文件解析完成"
    })
}

fn invoice_import_commit_action(can_import: bool) -> Value {
    if !can_import {
        return json!(null);
    }
    json!({
        "method": "POST",
        "endpoint": "/api/v1/imports/invoices/commit",
        "label": "确认导入",
        "success_message": "发票导入完成"
    })
}

fn invoice_update_action(invoice: &Invoice, can_modify: bool) -> Value {
    if !can_modify {
        return json!(null);
    }

    json!({
        "method": "PUT",
        "endpoint": format!("/api/invoices/{}", invoice.id),
        "label": "保存发票",
        "success_title": "更新成功",
        "success_message": "发票更新成功",
    })
}

fn invoice_delete_action(invoice: &Invoice, can_modify: bool) -> Value {
    if !can_modify {
        return json!(null);
    }
    let invoice_number = invoice.invoice_number.trim();
    let invoice_number = if invoice_number.is_empty() {
        "该发票"
    } else {
        invoice_number
    };

    json!({
        "method": "DELETE",
        "endpoint": format!("/api/invoices/{}", invoice.id),
        "confirm_message": format!("确定要删除发票「{invoice_number}」吗？相关附件也将被删除。"),
        "success_title": "删除成功",
        "success_message": "发票删除成功",
    })
}

fn invoice_post_action(
    endpoint: String,
    label: &str,
    success_message: &str,
    target_status: &str,
) -> Value {
    json!({
        "method": "POST",
        "endpoint": endpoint,
        "label": label,
        "success_title": "状态已更新",
        "success_message": success_message,
        "target_status": target_status,
    })
}

fn invoice_mark_sent_action(invoice: &Invoice, status: &str, can_modify: bool) -> Value {
    if !can_modify || invoice_is_input(invoice) || !matches!(status, "pending" | "approved") {
        return json!(null);
    }

    invoice_post_action(
        format!("/api/invoices/{}/mark_sent", invoice.id),
        "标记已开具",
        "发票已标记为已开具",
        "sent",
    )
}

fn invoice_mark_paid_action(invoice: &Invoice, status: &str, can_modify: bool) -> Value {
    if !can_modify
        || !matches!(
            status,
            "pending" | "approved" | "sent" | "overdue" | "partially_paid"
        )
    {
        return json!(null);
    }

    invoice_post_action(
        format!("/api/invoices/{}/mark_paid", invoice.id),
        "标记已支付",
        "发票已标记为已支付",
        "paid",
    )
}

fn invoice_record_payment_action(
    invoice: &Invoice,
    status: &str,
    can_modify: bool,
    paid_total: f64,
    remaining: f64,
) -> Value {
    if !can_modify
        || invoice_is_input(invoice)
        || !matches!(
            status,
            "pending" | "approved" | "sent" | "overdue" | "partially_paid"
        )
        || remaining <= 0.01
    {
        return json!(null);
    }

    json!({
        "method": "POST",
        "endpoint": format!("/api/invoices/{}/record-payment", invoice.id),
        "label": "记录收款",
        "success_title": "收款已记录",
        "success_message": "收款已记录",
        "target_status": if remaining <= invoice.amount { "partially_paid" } else { "paid" },
        "amount": round_money(invoice.amount),
        "amount_label": format_cny(invoice.amount),
        "paid_total": round_money(paid_total),
        "paid_total_label": format_cny(paid_total),
        "remaining": round_money(remaining),
        "remaining_label": format_cny(remaining),
        "default_amount": round_money(remaining),
        "fields": [
            {
                "name": "amount",
                "label": "收款金额",
                "type": "number",
                "required": true,
                "default": round_money(remaining),
            },
            {
                "name": "payment_date",
                "label": "收款日期",
                "type": "date",
                "required": false,
            },
            {
                "name": "note",
                "label": "备注",
                "type": "text",
                "required": false,
            },
        ],
    })
}

fn invoice_can_modify(can_manage: bool, user_id: &str, created_by: Option<&str>) -> bool {
    can_manage || created_by == Some(user_id)
}

fn invoice_date_label(value: Option<&str>) -> String {
    value
        .filter(|date| !date.trim().is_empty())
        .unwrap_or("-")
        .to_string()
}

fn invoice_datetime_label(value: Option<&str>) -> String {
    value
        .filter(|date| !date.trim().is_empty())
        .map(|date| date.replace('T', " ").trim_end_matches('Z').to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn invoice_status_options() -> Vec<Value> {
    [
        ("all", "全部状态"),
        ("pending", "待审批"),
        ("approved", "已批准"),
        ("rejected", "已拒绝"),
        ("paid", "已支付"),
        ("partially_paid", "部分已收"),
        ("sent", "已开具"),
        ("overdue", "逾期"),
        ("cancelled", "已取消"),
    ]
    .into_iter()
    .map(|(value, label)| json!({ "value": value, "label": label }))
    .collect()
}

fn invoice_form_status_options() -> Vec<Value> {
    [
        ("pending", "待审批"),
        ("approved", "已批准"),
        ("rejected", "已拒绝"),
        ("paid", "已支付"),
        ("partially_paid", "部分已收"),
        ("sent", "已开具"),
        ("overdue", "逾期"),
        ("cancelled", "已取消"),
    ]
    .into_iter()
    .map(|(value, label)| json!({ "value": value, "label": label }))
    .collect()
}

fn invoice_type_options() -> Vec<Value> {
    [
        ("receivable", "销项发票(应收)"),
        ("payable", "进项发票(应付)"),
    ]
    .into_iter()
    .map(|(value, label)| json!({ "value": value, "label": label }))
    .collect()
}

fn invoice_status_badge(status: &str) -> Value {
    let (label, variant, icon) = match status {
        "approved" => ("已批准", "default", "CheckCircle"),
        "rejected" => ("已拒绝", "destructive", "XCircle"),
        "paid" => ("已支付", "outline", "CheckCircle"),
        "partially_paid" => ("部分已收", "secondary", "Clock"),
        "sent" => ("已开具", "default", "CheckCircle"),
        "overdue" => ("逾期", "destructive", "XCircle"),
        "cancelled" => ("已取消", "outline", "XCircle"),
        _ => ("待审批", "secondary", "Clock"),
    };
    json!({
        "label": label,
        "variant": variant,
        "icon": icon,
    })
}

async fn current_user_can_manage_invoice(
    pool: &SqlitePool,
    user: &AuthUser,
) -> Result<bool, ApiError> {
    let role: Option<(String,)> = sqlx::query_as(
        "SELECT role FROM company_members WHERE user_id = ? AND company_id = ? LIMIT 1",
    )
    .bind(&user.user_id)
    .bind(&user.company_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(matches!(
        role.as_ref().map(|row| row.0.as_str()),
        Some("owner" | "admin")
    ))
}

fn round_money(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn format_cny(value: f64) -> String {
    let rounded = round_money(value);
    let sign = if rounded < 0.0 { "-" } else { "" };
    let absolute = rounded.abs();
    if (absolute.fract()).abs() < 0.005 {
        format!("¥{sign}{}", comma_i64(absolute.round() as i64))
    } else {
        let whole = absolute.trunc() as i64;
        let cents = ((absolute - whole as f64) * 100.0).round() as i64;
        format!("¥{sign}{}.{cents:02}", comma_i64(whole))
    }
}

fn comma_i64(value: i64) -> String {
    let digits = value.to_string();
    let mut output = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, ch) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            output.push(',');
        }
        output.push(ch);
    }
    output.chars().rev().collect()
}

async fn get_invoice(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Invoice>, ApiError> {
    let invoice: Option<Invoice> =
        sqlx::query_as("SELECT * FROM invoices WHERE id = ? AND company_id = ?")
            .bind(&id)
            .bind(&user.company_id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    invoice
        .map(Json)
        .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "invoice not found"))
}

async fn create_invoice(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Json(req): Json<CreateInvoiceRequest>,
) -> Result<Json<Invoice>, ApiError> {
    let amount = parse_invoice_money("amount", &req.amount)?;
    let tax_amount = parse_optional_invoice_money("tax_amount", req.tax_amount.as_ref())?;
    let invoice_date =
        normalize_optional_invoice_date("invoice_date", req.invoice_date.as_deref())?;
    let due_date = normalize_optional_invoice_date("due_date", req.due_date.as_deref())?;

    // R2-3: Validate non-negative amounts
    if amount < 0.0 {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "amount must be non-negative",
        ));
    }
    if let Some(tax) = tax_amount
        && tax < 0.0
    {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "tax_amount must be non-negative",
        ));
    }

    let invoice_type = normalize_invoice_type(req.invoice_type.as_deref())
        .unwrap_or_else(|| "receivable".to_string());
    if invoice_type != "receivable" && invoice_type != "payable" {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "invoice_type must be receivable or payable",
        ));
    }

    let client_id = req
        .client_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    if let Some(ref cid) = client_id {
        let exists: Option<(i64,)> =
            sqlx::query_as("SELECT 1 FROM clients WHERE id = ? AND company_id = ? LIMIT 1")
                .bind(cid)
                .bind(&user.company_id)
                .fetch_optional(&pool)
                .await
                .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
        if exists.is_none() {
            return Err(api_err(StatusCode::BAD_REQUEST, "client_id not found"));
        }
    }

    let mut currency = req
        .currency
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_uppercase());
    if currency.is_none() {
        let base: Option<(Option<String>,)> =
            sqlx::query_as("SELECT base_currency FROM companies WHERE id = ?")
                .bind(&user.company_id)
                .fetch_optional(&pool)
                .await
                .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
        currency = base
            .and_then(|row| row.0)
            .map(|c| c.trim().to_uppercase())
            .filter(|c| !c.is_empty());
    }
    let currency = currency.unwrap_or_else(|| "CNY".to_string());

    // Prefer explicit supplier; if empty and client_id present, fill display name.
    let mut supplier = req
        .supplier
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    if supplier.is_none()
        && let Some(ref cid) = client_id
    {
        let name: Option<(String,)> =
            sqlx::query_as("SELECT name FROM clients WHERE id = ? AND company_id = ?")
                .bind(cid)
                .bind(&user.company_id)
                .fetch_optional(&pool)
                .await
                .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
        supplier = name.map(|row| row.0);
    }

    let id = uuid::Uuid::new_v4().to_string();

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let provided_number = req
        .invoice_number
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let invoice_number = match provided_number {
        Some(n) => n,
        None => {
            crate::app::numbering::next_document_number(&mut tx, &user.company_id, "INV").await?
        }
    };

    let insert_result = sqlx::query(
        "INSERT INTO invoices (id, company_id, invoice_number, invoice_type, amount, tax_amount, invoice_date, due_date, supplier, description, file_path, contract_id, currency, client_id, created_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&user.company_id)
    .bind(&invoice_number)
    .bind(&invoice_type)
    .bind(amount)
    .bind(tax_amount)
    .bind(&invoice_date)
    .bind(&due_date)
    .bind(&supplier)
    .bind(&req.description)
    .bind(req.file_path.as_deref().map(str::trim).filter(|value| !value.is_empty()))
    .bind(req.contract_id.as_deref().map(str::trim).filter(|value| !value.is_empty()))
    .bind(&currency)
    .bind(&client_id)
    .bind(&user.user_id)
    .execute(&mut *tx)
    .await;

    if let Err(e) = insert_result {
        let msg = e.to_string();
        if msg.contains("UNIQUE") || msg.contains("unique") {
            return Err(api_err(StatusCode::CONFLICT, "发票号已存在"));
        }
        return Err(api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"));
    }

    if let Some(file_path) = req
        .file_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        sqlx::query(
            "UPDATE files SET entity_type = 'invoices', entity_id = ? WHERE id = ? AND company_id = ?",
        )
        .bind(&id)
        .bind(file_path)
        .bind(&user.company_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    }

    // Audit log
    let log_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO operation_logs (id, company_id, user_id, action, entity_type, entity_id, details) VALUES (?, ?, ?, 'create', 'invoices', ?, ?)")
        .bind(&log_id).bind(&user.company_id).bind(&user.user_id).bind(&id)
        .bind(format!("amount={}", amount))
        .execute(&mut *tx).await.ok();

    tx.commit()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    // R2-1: Re-fetch with company_id scoping
    let invoice: Invoice = sqlx::query_as("SELECT * FROM invoices WHERE id = ? AND company_id = ?")
        .bind(&id)
        .bind(&user.company_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(Json(invoice))
}

fn parse_invoice_money(field: &str, value: &Value) -> Result<f64, ApiError> {
    crate::api::generic::parse_form_money(value).ok_or_else(|| {
        api_err(
            StatusCode::BAD_REQUEST,
            &format!("{} must be a valid number", field),
        )
    })
}

fn parse_optional_invoice_money(
    field: &str,
    value: Option<&Value>,
) -> Result<Option<f64>, ApiError> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    Ok(Some(parse_invoice_money(field, value)?))
}

fn normalize_optional_invoice_date(
    field: &str,
    value: Option<&str>,
) -> Result<Option<String>, ApiError> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.trim().is_empty() {
        return Ok(None);
    }
    crate::api::generic::normalize_form_date(value)
        .map(Some)
        .ok_or_else(|| {
            api_err(
                StatusCode::BAD_REQUEST,
                &format!("{} must be a valid date", field),
            )
        })
}

async fn update_invoice(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(mut body): Json<Value>,
) -> Result<Json<Invoice>, ApiError> {
    let existing: Option<(Option<String>,)> =
        sqlx::query_as("SELECT created_by FROM invoices WHERE id = ? AND company_id = ?")
            .bind(&id)
            .bind(&user.company_id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    let Some((created_by,)) = existing else {
        return Err(api_err(StatusCode::NOT_FOUND, "invoice not found"));
    };
    let can_manage = current_user_can_manage_invoice(&pool, &user).await?;
    if !invoice_can_modify(can_manage, &user.user_id, created_by.as_deref()) {
        return Err(api_err(StatusCode::FORBIDDEN, "forbidden"));
    }

    let file_path_update = body
        .get("file_path")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    if let Some(map) = body.as_object_mut() {
        map.remove("status");
        // Normalize legacy invoice_type aliases on partial update.
        if let Some(raw) = map.get("invoice_type").and_then(Value::as_str) {
            match normalize_invoice_type(Some(raw)) {
                Some(normalized) if normalized == "receivable" || normalized == "payable" => {
                    map.insert("invoice_type".into(), json!(normalized));
                }
                Some(_) => {
                    return Err(api_err(
                        StatusCode::BAD_REQUEST,
                        "invoice_type must be receivable or payable",
                    ));
                }
                None => {
                    map.remove("invoice_type");
                }
            }
        }
        if let Some(cur) = map.get("currency").and_then(Value::as_str) {
            let trimmed = cur.trim().to_uppercase();
            if trimmed.is_empty() {
                map.remove("currency");
            } else {
                map.insert("currency".into(), json!(trimmed));
            }
        }
    }

    let has_update_fields = body.as_object().map(|map| !map.is_empty()).unwrap_or(true);
    if has_update_fields {
        // Use generic_update for partial update support
        let _ = crate::api::generic::generic_update(
            &pool,
            "invoices",
            &id,
            body,
            &user.company_id,
            &user.user_id,
        )
        .await?;
    }

    if let Some(file_path) = file_path_update {
        sqlx::query(
            "UPDATE files SET entity_type = 'invoices', entity_id = ? WHERE id = ? AND company_id = ?",
        )
        .bind(&id)
        .bind(&file_path)
        .bind(&user.company_id)
        .execute(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    }

    // Audit log
    let log_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO operation_logs (id, company_id, user_id, action, entity_type, entity_id) VALUES (?, ?, ?, 'update', 'invoices', ?)")
        .bind(&log_id).bind(&user.company_id).bind(&user.user_id).bind(&id)
        .execute(&pool).await.ok();

    let invoice: Invoice = sqlx::query_as("SELECT * FROM invoices WHERE id = ? AND company_id = ?")
        .bind(&id)
        .bind(&user.company_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| api_err(StatusCode::NOT_FOUND, "invoice not found"))?;

    Ok(Json(invoice))
}

async fn mark_invoice_sent(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    update_invoice_status(&pool, &user, &id, "sent").await
}

async fn mark_invoice_paid(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    update_invoice_status(&pool, &user, &id, "paid").await
}

/// Record an incoming payment against an output (销项) invoice.
/// Supports partial / installment amounts; settlement goes through `sync_payment_links`.
async fn record_invoice_payment(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
    body: Option<Json<RecordInvoicePaymentRequest>>,
) -> Result<Json<Value>, ApiError> {
    let req = body
        .map(|Json(b)| b)
        .unwrap_or(RecordInvoicePaymentRequest {
            amount: None,
            payment_date: None,
            note: None,
        });

    let invoice: Option<(String, f64, Option<String>, Option<String>, Option<String>)> =
        sqlx::query_as(
            "SELECT invoice_number, amount, supplier, invoice_type, created_by FROM invoices WHERE id = ? AND company_id = ?",
        )
        .bind(&id)
        .bind(&user.company_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "invoice lookup failed"))?;

    let Some((invoice_number, invoice_amount, supplier, invoice_type, created_by)) = invoice else {
        return Err(api_err(StatusCode::NOT_FOUND, "invoice not found"));
    };

    let can_manage = current_user_can_manage_invoice(&pool, &user).await?;
    if !invoice_can_modify(can_manage, &user.user_id, created_by.as_deref()) {
        return Err(api_err(StatusCode::FORBIDDEN, "forbidden"));
    }

    if matches!(
        invoice_type
            .as_deref()
            .unwrap_or("")
            .to_lowercase()
            .as_str(),
        "input" | "expense" | "payable" | "进项" | "进项发票"
    ) {
        return Err(api_err(StatusCode::BAD_REQUEST, "只能对销项发票记录收款"));
    }

    // CAST to REAL: with no payment rows SQLite returns INTEGER 0, which sqlx
    // refuses to decode as f64 (same fix as workflows::sync_payment_links).
    let paid_total: f64 = sqlx::query_scalar(
        "SELECT CAST(COALESCE(SUM(amount), 0) AS REAL) FROM payments WHERE status IN ('completed', 'paid') AND invoice_id = ? AND company_id = ?",
    )
    .bind(&id)
    .bind(&user.company_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "payment lookup failed"))?;

    let remaining = (invoice_amount - paid_total).max(0.0);
    if remaining <= 0.01 {
        return Err(api_err(StatusCode::BAD_REQUEST, "该发票已收齐，无需再收款"));
    }

    let pay_amount = req.amount.unwrap_or(remaining);
    if pay_amount <= 0.0 {
        return Err(api_err(StatusCode::BAD_REQUEST, "收款金额必须大于 0"));
    }
    if pay_amount > remaining + 0.01 {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            &format!(
                "收款金额超过剩余应收 {}",
                format_cny(round_money(remaining))
            ),
        ));
    }

    let payment_date = req
        .payment_date
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .or_else(|| Some(chrono::Local::now().format("%Y-%m-%d").to_string()))
        .unwrap();

    let payment_id = uuid::Uuid::new_v4().to_string();
    let payer = supplier.unwrap_or_default();
    let note = req
        .note
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|n| format!("发票 {invoice_number} 收款 · {n}"))
        .unwrap_or_else(|| format!("发票 {invoice_number} 收款"));

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "tx begin failed"))?;

    sqlx::query(
        "INSERT INTO payments (id, company_id, payment_type, amount, status, payer, invoice_id, payment_date, description) VALUES (?, ?, 'income', ?, 'completed', ?, ?, ?, ?)",
    )
    .bind(&payment_id)
    .bind(&user.company_id)
    .bind(pay_amount)
    .bind(&payer)
    .bind(&id)
    .bind(&payment_date)
    .bind(&note)
    .execute(&mut *tx)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "record payment failed"))?;

    crate::app::workflows::sync_payment_links(
        &mut tx,
        &user.company_id,
        &payment_id,
        Some(&payment_date),
        Some(&id),
        None,
    )
    .await?;

    tx.commit()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "tx commit failed"))?;

    let new_paid = paid_total + pay_amount;
    let new_remaining = (invoice_amount - new_paid).max(0.0);
    let invoice_status: String =
        sqlx::query_scalar("SELECT COALESCE(status, 'pending') FROM invoices WHERE id = ?")
            .bind(&id)
            .fetch_one(&pool)
            .await
            .unwrap_or_else(|_| "pending".into());

    Ok(Json(json!({
        "success": true,
        "payment_id": payment_id,
        "invoice_id": id,
        "amount": round_money(pay_amount),
        "amount_label": format_cny(pay_amount),
        "paid_total": round_money(new_paid),
        "paid_total_label": format_cny(new_paid),
        "remaining": round_money(new_remaining),
        "remaining_label": format_cny(new_remaining),
        "invoice_status": invoice_status,
        "toast_message": format!("已记录发票 {} 的收款 {}", invoice_number, format_cny(pay_amount)),
    })))
}

async fn update_invoice_status(
    pool: &SqlitePool,
    user: &AuthUser,
    id: &str,
    status: &str,
) -> Result<Json<Value>, ApiError> {
    let invoice = fetch_scoped_invoice(pool, id, &user.company_id).await?;
    let can_manage = current_user_can_manage_invoice(pool, user).await?;
    if !invoice_can_modify(can_manage, &user.user_id, invoice.created_by.as_deref()) {
        return Err(api_err(StatusCode::FORBIDDEN, "forbidden"));
    }

    sqlx::query(
        "UPDATE invoices SET status = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? AND company_id = ?",
    )
    .bind(status)
    .bind(id)
    .bind(&user.company_id)
    .execute(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let invoice = fetch_scoped_invoice(pool, id, &user.company_id).await?;
    Ok(Json(invoice_status_action_value(
        &invoice,
        can_manage,
        &user.user_id,
    )))
}

async fn fetch_scoped_invoice(
    pool: &SqlitePool,
    id: &str,
    company_id: &str,
) -> Result<Invoice, ApiError> {
    sqlx::query_as("SELECT * FROM invoices WHERE id = ? AND company_id = ?")
        .bind(id)
        .bind(company_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?
        .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "invoice not found"))
}

fn invoice_status_action_value(invoice: &Invoice, can_manage: bool, user_id: &str) -> Value {
    let status = invoice.status.as_deref().unwrap_or("pending");
    let can_modify = invoice_can_modify(can_manage, user_id, invoice.created_by.as_deref());
    json!({
        "id": invoice.id,
        "status": status,
        "status_badge": invoice_status_badge(status),
        "can_mark_sent": can_modify && !invoice_is_input(invoice) && matches!(status, "pending" | "approved"),
        "can_mark_paid": can_modify && matches!(status, "pending" | "approved" | "sent" | "overdue" | "partially_paid"),
        "mark_sent_action": invoice_mark_sent_action(invoice, status, can_modify),
        "mark_paid_action": invoice_mark_paid_action(invoice, status, can_modify),
    })
}

async fn delete_invoice(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let existing: Option<(Option<String>,)> =
        sqlx::query_as("SELECT created_by FROM invoices WHERE id = ? AND company_id = ?")
            .bind(&id)
            .bind(&user.company_id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    let Some((created_by,)) = existing else {
        return Err(api_err(StatusCode::NOT_FOUND, "invoice not found"));
    };
    let can_manage = current_user_can_manage_invoice(&pool, &user).await?;
    if !invoice_can_modify(can_manage, &user.user_id, created_by.as_deref()) {
        return Err(api_err(StatusCode::FORBIDDEN, "forbidden"));
    }

    // Check for dependent payment records before deleting
    if let Ok((n,)) = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM payments WHERE invoice_id = ? AND company_id = ?",
    )
    .bind(&id)
    .bind(&user.company_id)
    .fetch_one(&pool)
    .await
        && n > 0
    {
        return Err(api_err(
            StatusCode::CONFLICT,
            "cannot delete: has dependent payment records",
        ));
    }

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let result = sqlx::query("DELETE FROM invoices WHERE id = ? AND company_id = ?")
        .bind(&id)
        .bind(&user.company_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    if result.rows_affected() == 0 {
        return Err(api_err(StatusCode::NOT_FOUND, "invoice not found"));
    }

    // Audit log
    let log_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO operation_logs (id, company_id, user_id, action, entity_type, entity_id) VALUES (?, ?, ?, 'delete', 'invoices', ?)")
        .bind(&log_id).bind(&user.company_id).bind(&user.user_id).bind(&id)
        .execute(&mut *tx).await.ok();

    tx.commit()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(StatusCode::NO_CONTENT)
}
