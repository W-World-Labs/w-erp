use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sqlx::SqlitePool;
use std::collections::HashSet;

use crate::auth::AuthUser;

use super::{
    error::{ApiError, api_err},
    generic,
};

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/options", get(import_options))
        .route("/bulk/{table}", post(bulk_import))
        .route("/invoices/preview", post(invoice_import_preview))
        .route("/invoices/commit", post(invoice_import_commit))
        .with_state(pool)
}

#[derive(Debug, Deserialize)]
struct BulkImportRequest {
    filename: String,
    file_base64: String,
}

#[derive(Debug, Deserialize, Clone)]
struct InvoiceImportRequest {
    filename: String,
    file_base64: String,
    import_format: Option<String>,
}

#[derive(Debug, Serialize)]
struct BulkImportResponse {
    table: String,
    total: usize,
    success: usize,
    failed: usize,
    errors: Vec<RowImportError>,
}

#[derive(Debug, Serialize)]
struct RowImportError {
    row: usize,
    error: String,
}

#[derive(Debug, Clone)]
struct InvoiceImportDraft {
    row_number: usize,
    invoice_number: String,
    title: String,
    supplier: String,
    amount_raw: String,
    tax_amount_raw: String,
    invoice_date: String,
    description: String,
    invoice_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct InvoiceImportRow {
    pub(crate) row_number: usize,
    pub(crate) invoice_number: String,
    pub(crate) title: String,
    pub(crate) supplier: String,
    pub(crate) amount: f64,
    pub(crate) tax_amount: f64,
    pub(crate) invoice_date: String,
    pub(crate) description: String,
    pub(crate) invoice_type: String,
    pub(crate) amount_label: String,
    pub(crate) tax_amount_label: String,
    pub(crate) status: String,
    pub(crate) error: String,
    pub(crate) status_badge: Value,
}

#[derive(Debug, Default, Serialize)]
struct InvoiceImportStats {
    total_count: usize,
    valid_count: usize,
    invalid_count: usize,
    duplicate_count: usize,
    total_label: String,
    valid_label: String,
    invalid_label: String,
    duplicate_label: String,
}

#[derive(Debug, Serialize)]
struct InvoiceImportResponse {
    rows: Vec<InvoiceImportRow>,
    stats: InvoiceImportStats,
    can_import: bool,
    import_button_label: String,
    preview_message: String,
    progress_value: i64,
    progress_label: String,
}

#[derive(Debug, Serialize)]
struct InvoiceImportCommitResponse {
    rows: Vec<InvoiceImportRow>,
    stats: InvoiceImportStats,
    can_import: bool,
    import_button_label: String,
    preview_message: String,
    progress_value: i64,
    progress_label: String,
    success_count: usize,
    failed_count: usize,
    success_message: String,
    should_close: bool,
}

#[derive(Debug, Clone, Copy)]
struct ImportTarget {
    table: &'static str,
    required: &'static [&'static str],
}

async fn import_options(_user: AuthUser) -> Json<Value> {
    Json(json!({
        "table_options": import_table_options()
            .into_iter()
            .map(|(value, label)| json!({
                "value": value,
                "label": label,
                "import_action": {
                    "method": "POST",
                    "endpoint": format!("/api/v1/imports/bulk/{value}"),
                    "label": format!("导入{label}"),
                    "success_message": "导入完成"
                }
            }))
            .collect::<Vec<_>>()
    }))
}

async fn bulk_import(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(table): Path<String>,
    Json(req): Json<BulkImportRequest>,
) -> Result<Json<BulkImportResponse>, ApiError> {
    let target = import_target(&table)
        .ok_or_else(|| api_err(StatusCode::BAD_REQUEST, "unsupported import table"))?;

    if !req.filename.to_ascii_lowercase().ends_with(".csv") {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "unsupported import file type: only csv is supported",
        ));
    }

    let bytes = decode_base64(&req.file_base64)?;
    let rows = parse_csv_rows(&bytes)?;
    let total = rows.len();
    let mut success = 0usize;
    let mut errors = Vec::new();

    for (index, row) in rows.into_iter().enumerate() {
        let row_number = index + 2;
        if let Some(missing) = missing_required(&row, target.required) {
            errors.push(RowImportError {
                row: row_number,
                error: format!("missing required field: {}", missing),
            });
            continue;
        }

        match generic::generic_create(
            &pool,
            target.table,
            Value::Object(row),
            target.required,
            &user.company_id,
            &user.user_id,
        )
        .await
        {
            Ok(_) => success += 1,
            Err((status, Json(body))) => {
                let message = body
                    .get("error")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("import failed with status {}", status.as_u16()));
                errors.push(RowImportError {
                    row: row_number,
                    error: message,
                });
            }
        }
    }

    Ok(Json(BulkImportResponse {
        table: target.table.to_string(),
        total,
        success,
        failed: errors.len(),
        errors,
    }))
}

async fn invoice_import_preview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Json(req): Json<InvoiceImportRequest>,
) -> Result<Json<InvoiceImportResponse>, ApiError> {
    let rows = build_invoice_import_rows(&pool, &user.company_id, &req).await?;
    Ok(Json(invoice_import_response(rows)))
}

async fn invoice_import_commit(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Json(req): Json<InvoiceImportRequest>,
) -> Result<Json<InvoiceImportCommitResponse>, ApiError> {
    let rows = build_invoice_import_rows(&pool, &user.company_id, &req).await?;
    let mut success_count = 0usize;

    for row in rows.iter().filter(|row| row.status == "valid") {
        let id = uuid::Uuid::new_v4().to_string();
        let description = invoice_import_description(row);
        sqlx::query(
            r#"
            INSERT INTO invoices (
                id, company_id, invoice_number, invoice_type, amount, tax_amount,
                status, invoice_date, supplier, description, created_by
            )
            VALUES (?, ?, ?, ?, ?, ?, 'pending', ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(&user.company_id)
        .bind(&row.invoice_number)
        .bind(&row.invoice_type)
        .bind(row.amount)
        .bind(row.tax_amount)
        .bind(&row.invoice_date)
        .bind(&row.supplier)
        .bind(description)
        .bind(&user.user_id)
        .execute(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "invoice import failed"))?;
        success_count += 1;
    }

    let stats = invoice_import_stats(&rows);
    let failed_count = stats.total_count.saturating_sub(success_count);
    let success_message = if success_count > 0 {
        format!(
            "成功导入 {} 条发票，跳过 {} 条",
            success_count, failed_count
        )
    } else {
        "没有有效的发票数据可导入".to_string()
    };

    Ok(Json(InvoiceImportCommitResponse {
        can_import: false,
        import_button_label: format!("导入 {} 条有效数据", stats.valid_count),
        preview_message: format!("成功解析 {} 条发票数据", stats.total_count),
        progress_value: 100,
        progress_label: "100%".to_string(),
        should_close: success_count > 0,
        rows,
        stats,
        success_count,
        failed_count,
        success_message,
    }))
}

async fn build_invoice_import_rows(
    pool: &SqlitePool,
    company_id: &str,
    req: &InvoiceImportRequest,
) -> Result<Vec<InvoiceImportRow>, ApiError> {
    let drafts = parse_invoice_import_drafts(req)?;
    let existing_numbers = existing_invoice_numbers(pool, company_id).await?;
    let mut seen_numbers = HashSet::new();
    let mut rows = Vec::with_capacity(drafts.len());

    for draft in drafts {
        rows.push(classify_invoice_import_row(
            draft,
            &existing_numbers,
            &mut seen_numbers,
        ));
    }

    Ok(rows)
}

fn invoice_import_response(rows: Vec<InvoiceImportRow>) -> InvoiceImportResponse {
    let stats = invoice_import_stats(&rows);
    InvoiceImportResponse {
        can_import: stats.valid_count > 0,
        import_button_label: format!("导入 {} 条有效数据", stats.valid_count),
        preview_message: format!("成功解析 {} 条发票数据", stats.total_count),
        progress_value: 100,
        progress_label: "100%".to_string(),
        rows,
        stats,
    }
}

fn invoice_import_stats(rows: &[InvoiceImportRow]) -> InvoiceImportStats {
    let total_count = rows.len();
    let valid_count = rows.iter().filter(|row| row.status == "valid").count();
    let invalid_count = rows.iter().filter(|row| row.status == "invalid").count();
    let duplicate_count = rows.iter().filter(|row| row.status == "duplicate").count();

    InvoiceImportStats {
        total_count,
        valid_count,
        invalid_count,
        duplicate_count,
        total_label: format!("{} 条数据", total_count),
        valid_label: format!("{} 条有效数据", valid_count),
        invalid_label: format!("{} 条无效数据", invalid_count),
        duplicate_label: format!("{} 条重复数据", duplicate_count),
    }
}

async fn existing_invoice_numbers(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<HashSet<String>, ApiError> {
    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT invoice_number FROM invoices WHERE company_id = ?")
            .bind(company_id)
            .fetch_all(pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "invoice import failed"))?;
    Ok(rows.into_iter().map(|row| row.0).collect())
}

fn parse_invoice_import_drafts(
    req: &InvoiceImportRequest,
) -> Result<Vec<InvoiceImportDraft>, ApiError> {
    let bytes = decode_base64(&req.file_base64)?;
    let format = req
        .import_format
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)
        .unwrap_or_else(|| invoice_import_format_from_filename(&req.filename));

    match format.as_str() {
        "csv" => parse_invoice_csv_drafts(&bytes),
        "xml" => parse_invoice_xml_drafts(&bytes),
        _ => Err(api_err(
            StatusCode::BAD_REQUEST,
            "unsupported invoice import file type",
        )),
    }
}

fn invoice_import_format_from_filename(filename: &str) -> String {
    let lower = filename.to_ascii_lowercase();
    if lower.ends_with(".xml") {
        "xml".to_string()
    } else {
        "csv".to_string()
    }
}

fn parse_invoice_csv_drafts(bytes: &[u8]) -> Result<Vec<InvoiceImportDraft>, ApiError> {
    let rows = parse_csv_rows(bytes)?;
    let mut drafts = Vec::with_capacity(rows.len());
    for (index, row) in rows.into_iter().enumerate() {
        drafts.push(InvoiceImportDraft {
            row_number: index + 2,
            invoice_number: import_value(&row, &["invoice_number", "发票号码", "发票号"]),
            title: import_value(&row, &["title", "标题", "商品名称", "项目名称"]),
            supplier: import_value(&row, &["vendor_name", "supplier", "供应商名称", "供应商"]),
            amount_raw: import_value(&row, &["amount", "金额", "价税合计", "合计金额"]),
            tax_amount_raw: import_value(&row, &["tax_amount", "税额", "合计税额"]),
            invoice_date: normalize_invoice_date(&import_value(
                &row,
                &["invoice_date", "开票日期", "日期"],
            )),
            description: import_value(&row, &["description", "备注", "说明"]),
            invoice_type: import_value(&row, &["invoice_type", "类型", "发票类型"]),
        });
    }
    Ok(drafts)
}

fn parse_invoice_xml_drafts(bytes: &[u8]) -> Result<Vec<InvoiceImportDraft>, ApiError> {
    // Prefer w-einvoice structured direct-read (数电票 XML / GBK / alternate tags),
    // then fill gaps with the legacy short-tag scrape (Fpdm/Fphm/Xfmc/Hjje…).
    let structured = w_einvoice::parse_xml(bytes).ok();
    let text = match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            // GBK/legacy encodings: if structured parse already succeeded, use it alone.
            if let Some(inv) = structured {
                return Ok(vec![einvoice_to_import_draft(inv, None)]);
            }
            return Err(api_err(StatusCode::BAD_REQUEST, "invalid xml encoding"));
        }
    };

    let invoice_code = xml_import_value(&text, &["Fpdm", "InvoiceCode"]);
    let invoice_number_tag = xml_import_value(&text, &["Fphm", "InvoiceNo", "InvoiceNumber"]);
    let legacy_full_number = if invoice_code.is_empty() {
        invoice_number_tag.clone()
    } else if invoice_number_tag.is_empty() {
        invoice_code.clone()
    } else {
        format!("{}-{}", invoice_code, invoice_number_tag)
    };
    let remark = xml_import_value(&text, &["Bz", "Remark"]);
    let goods = xml_import_value(&text, &["Spxx", "GoodsInfo", "ItemName"]);

    if let Some(inv) = structured {
        let mut draft = einvoice_to_import_draft(inv, Some(&text));
        // Preserve legacy code+number concatenation when both tags exist, and
        // fall back to it whenever the structured parse left the number empty.
        if (!invoice_code.is_empty() && !invoice_number_tag.is_empty())
            || draft.invoice_number.trim().is_empty()
        {
            draft.invoice_number = legacy_full_number;
        }
        if draft.supplier.trim().is_empty() {
            draft.supplier = xml_import_value(&text, &["Xfmc", "SellerName", "XsfMc"]);
        }
        if draft.amount_raw.trim().is_empty() {
            draft.amount_raw =
                xml_import_value(&text, &["Hjje", "TotalAmount", "TaxExclusiveTotalAmount"]);
        }
        if draft.tax_amount_raw.trim().is_empty() {
            draft.tax_amount_raw = xml_import_value(&text, &["Hjse", "TotalTax", "TaxTotalAmount"]);
        }
        if draft.invoice_date.trim().is_empty() {
            draft.invoice_date = normalize_invoice_date(&xml_import_value(
                &text,
                &["Kprq", "InvoiceDate", "IssueDate"],
            ));
        }
        if draft.title == "电子发票" && !goods.is_empty() {
            draft.title = goods;
        }
        if draft.description.trim().is_empty() && !remark.is_empty() {
            draft.description = remark;
        }
        return Ok(vec![draft]);
    }

    Ok(vec![InvoiceImportDraft {
        row_number: 1,
        invoice_number: legacy_full_number,
        title: if goods.is_empty() {
            "电子发票".to_string()
        } else {
            goods
        },
        supplier: xml_import_value(&text, &["Xfmc", "SellerName"]),
        amount_raw: xml_import_value(&text, &["Hjje", "TotalAmount"]),
        tax_amount_raw: xml_import_value(&text, &["Hjse", "TotalTax"]),
        invoice_date: normalize_invoice_date(&xml_import_value(&text, &["Kprq", "InvoiceDate"])),
        description: remark,
        invoice_type: "input".to_string(),
    }])
}

/// Reuse the canonical XML parser and validation rules for mailbox attachments.
/// Invoice-number/content deduplication is applied by the mailbox ingestion layer,
/// so this helper intentionally validates against an empty existing-number set.
pub(crate) fn parse_invoice_xml_attachment(bytes: &[u8]) -> Result<InvoiceImportRow, ApiError> {
    let draft = parse_invoice_xml_drafts(bytes)?
        .into_iter()
        .next()
        .ok_or_else(|| api_err(StatusCode::BAD_REQUEST, "xml contains no invoice"))?;
    Ok(classify_invoice_import_row(
        draft,
        &HashSet::new(),
        &mut HashSet::new(),
    ))
}

fn einvoice_to_import_draft(
    inv: w_einvoice::EInvoice,
    _xml_text: Option<&str>,
) -> InvoiceImportDraft {
    let amount_raw = inv
        .amount
        .or(inv.total_amount)
        .map(|n| format!("{n}"))
        .unwrap_or_default();
    let tax_raw = inv.tax_amount.map(|n| format!("{n}")).unwrap_or_default();
    let title = inv
        .items
        .first()
        .and_then(|i| i.name.clone())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "电子发票".to_string());
    InvoiceImportDraft {
        row_number: 1,
        invoice_number: inv.invoice_number,
        title,
        supplier: inv.seller_name.unwrap_or_default(),
        amount_raw,
        tax_amount_raw: tax_raw,
        invoice_date: inv
            .invoice_date
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default(),
        description: inv.buyer_name.unwrap_or_default(),
        invoice_type: "input".to_string(),
    }
}

fn import_value(row: &Map<String, Value>, keys: &[&str]) -> String {
    for key in keys {
        if let Some(value) = row.get(*key) {
            if let Some(text) = value.as_str() {
                let trimmed = text.trim().trim_start_matches('\u{feff}');
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }
    String::new()
}

fn xml_import_value(xml: &str, tags: &[&str]) -> String {
    for tag in tags {
        let open = format!("<{}>", tag);
        let close = format!("</{}>", tag);
        if let Some(start) = xml.find(&open) {
            let value_start = start + open.len();
            if let Some(end) = xml[value_start..].find(&close) {
                let value = xml[value_start..value_start + end].trim();
                if !value.is_empty() {
                    return value.to_string();
                }
            }
        }
    }
    String::new()
}

fn classify_invoice_import_row(
    draft: InvoiceImportDraft,
    existing_numbers: &HashSet<String>,
    seen_numbers: &mut HashSet<String>,
) -> InvoiceImportRow {
    let amount = parse_import_number(&draft.amount_raw).unwrap_or(0.0);
    let tax_amount = parse_import_number(&draft.tax_amount_raw).unwrap_or(0.0);
    let invoice_type = if draft.invoice_type.trim().is_empty() {
        "payable".to_string()
    } else {
        crate::api::invoices::normalize_invoice_type(Some(draft.invoice_type.trim()))
            .unwrap_or_else(|| draft.invoice_type.trim().to_string())
    };

    let mut errors = Vec::new();
    if draft.invoice_number.trim().is_empty() {
        errors.push("发票号不能为空");
    }
    if draft.title.trim().is_empty() {
        errors.push("标题不能为空");
    }
    if draft.supplier.trim().is_empty() {
        errors.push("供应商不能为空");
    }
    if amount <= 0.0 {
        errors.push("金额无效");
    }
    if draft.invoice_date.trim().is_empty() {
        errors.push("开票日期不能为空");
    } else if NaiveDate::parse_from_str(&draft.invoice_date, "%Y-%m-%d").is_err() {
        errors.push("日期格式错误，应为 YYYY-MM-DD");
    }

    let duplicate = if errors.is_empty() {
        existing_numbers.contains(&draft.invoice_number)
            || !seen_numbers.insert(draft.invoice_number.clone())
    } else {
        false
    };

    let (status, error) = if !errors.is_empty() {
        ("invalid", errors.join("; "))
    } else if duplicate {
        ("duplicate", "发票号已存在".to_string())
    } else {
        ("valid", String::new())
    };

    InvoiceImportRow {
        row_number: draft.row_number,
        invoice_number: draft.invoice_number,
        title: draft.title,
        supplier: draft.supplier,
        amount,
        tax_amount,
        invoice_date: draft.invoice_date,
        description: draft.description,
        invoice_type,
        amount_label: format_cny(amount),
        tax_amount_label: format_cny(tax_amount),
        status: status.to_string(),
        error,
        status_badge: invoice_import_status_badge(status),
    }
}

fn parse_import_number(value: &str) -> Option<f64> {
    let normalized = value
        .trim()
        .trim_start_matches('¥')
        .replace(',', "")
        .replace('%', "");
    if normalized.is_empty() {
        None
    } else {
        normalized.parse::<f64>().ok()
    }
}

fn normalize_invoice_date(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() == 8 && trimmed.chars().all(|ch| ch.is_ascii_digit()) {
        format!("{}-{}-{}", &trimmed[0..4], &trimmed[4..6], &trimmed[6..8])
    } else {
        trimmed.to_string()
    }
}

fn invoice_import_status_badge(status: &str) -> Value {
    match status {
        "valid" => json!({
            "label": "有效",
            "variant": "default",
            "icon_key": "CheckCircle2",
            "class_name": "bg-green-600 text-white hover:bg-green-600"
        }),
        "duplicate" => json!({
            "label": "重复",
            "variant": "secondary",
            "icon_key": "AlertCircle",
            "class_name": "bg-yellow-100 text-yellow-800"
        }),
        _ => json!({
            "label": "无效",
            "variant": "destructive",
            "icon_key": "AlertCircle",
            "class_name": ""
        }),
    }
}

fn invoice_import_description(row: &InvoiceImportRow) -> String {
    if row.description.trim().is_empty() {
        row.title.clone()
    } else {
        format!("{} - {}", row.title, row.description)
    }
}

fn format_cny(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;
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

fn decode_base64(input: &str) -> Result<Vec<u8>, ApiError> {
    let payload = input
        .split_once(',')
        .map(|(_, payload)| payload)
        .unwrap_or(input)
        .trim();

    general_purpose::STANDARD
        .decode(payload)
        .map_err(|_| api_err(StatusCode::BAD_REQUEST, "invalid base64 file content"))
}

fn parse_csv_rows(bytes: &[u8]) -> Result<Vec<Map<String, Value>>, ApiError> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(bytes);
    let headers = reader
        .headers()
        .map_err(|_| api_err(StatusCode::BAD_REQUEST, "invalid csv headers"))?
        .clone();

    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|_| api_err(StatusCode::BAD_REQUEST, "invalid csv row"))?;
        let mut row = Map::new();
        for (index, header) in headers.iter().enumerate() {
            let key = header.trim();
            if key.is_empty() {
                continue;
            }
            if let Some(value) = record.get(index) {
                let value = value.trim();
                if !value.is_empty() {
                    row.insert(key.to_string(), json!(value));
                }
            }
        }
        rows.push(row);
    }

    if rows.is_empty() {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "csv contains no data rows",
        ));
    }

    Ok(rows)
}

fn missing_required(row: &Map<String, Value>, required: &[&str]) -> Option<String> {
    required.iter().find_map(|field| match row.get(*field) {
        Some(Value::String(value)) if !value.trim().is_empty() => None,
        Some(Value::Number(_)) | Some(Value::Bool(_)) => None,
        _ => Some((*field).to_string()),
    })
}

fn import_target(raw: &str) -> Option<ImportTarget> {
    let normalized = raw.trim().to_ascii_lowercase();
    let target = match normalized.as_str() {
        "assets" | "asset" => ImportTarget {
            table: "assets",
            required: &["name"],
        },
        "employees" | "employee" => ImportTarget {
            table: "employees",
            required: &["name"],
        },
        "invoices" | "invoice" => ImportTarget {
            table: "invoices",
            required: &["invoice_number", "amount"],
        },
        "projects" | "project" => ImportTarget {
            table: "projects",
            required: &["name"],
        },
        "contracts" | "contract" => ImportTarget {
            table: "contracts",
            required: &["contract_name"],
        },
        "licenses" | "license" => ImportTarget {
            table: "licenses",
            required: &["license_name"],
        },
        "budgets" | "budget" => ImportTarget {
            table: "budgets",
            required: &["category", "budget_type", "period"],
        },
        "salaries" | "salary" | "salary_records" => ImportTarget {
            table: "salary_records",
            required: &["employee_id", "period"],
        },
        "rents" | "rent" | "rent_records" => ImportTarget {
            table: "rent_records",
            required: &["location_name", "monthly_rent"],
        },
        "utilities" | "utility" => ImportTarget {
            table: "utilities",
            required: &["location_name", "utility_type", "period"],
        },
        _ => return None,
    };

    Some(target)
}

fn import_table_options() -> Vec<(&'static str, &'static str)> {
    vec![
        ("assets", "资产管理"),
        ("employees", "员工管理"),
        ("invoices", "发票管理"),
        ("projects", "项目管理"),
        ("contracts", "合同管理"),
        ("licenses", "证照管理"),
        ("budgets", "预算管理"),
        ("salaries", "工资管理"),
        ("rents", "租金管理"),
        ("utilities", "水电费管理"),
    ]
}
