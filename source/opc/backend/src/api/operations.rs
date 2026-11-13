use super::error::{ApiError, api_err};
use crate::auth::AuthUser;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{Row, SqlitePool, sqlite::SqliteRow};
use std::collections::{BTreeMap, HashMap};

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/rent/form_options", get(rent_form_options))
        .route("/rent/overview", get(rent_overview))
        .route("/rents/overview", get(rent_overview))
        .route("/payments/form_options", get(payment_form_options))
        .route("/payments/overview", get(payments_overview))
        .route("/salary/overview", get(salary_overview))
        .route("/salary/insurance-preview", post(salary_insurance_preview))
        .route("/salary/{id}/mark_paid", post(mark_salary_paid))
        .route("/salaries/overview", get(salary_overview))
        .route("/contracts/form_options", get(contracts_form_options))
        .route("/contracts/overview", get(contracts_overview))
        .route("/asset_records/overview", get(asset_records_overview))
        .route("/utilities/{id}/mark_paid", post(mark_utility_paid))
        .route("/reminders/overview", get(reminders_overview))
        .route(
            "/reminders/check_operating_deadlines",
            post(check_operating_deadline_reminders),
        )
        .route("/reminders/{id}/complete", post(complete_reminder))
        .route("/reminders/{id}/dismiss", post(dismiss_reminder))
        .route(
            "/reminders/check_asset_expiry",
            post(check_asset_expiry_reminders),
        )
        .route("/operation_logs/overview", get(operation_logs_overview))
        .route("/projects/form_options", get(projects_form_options))
        .route("/projects/overview", get(projects_overview))
        .route("/deals/overview", get(deals_overview))
        .route("/quotes/overview", get(quotes_overview))
        .route("/time_entries/overview", get(time_entries_overview))
        .route("/reconciliation/overview", get(reconciliation_overview))
        .route("/vendors/overview", get(vendors_overview))
        .route("/licenses/form_options", get(licenses_form_options))
        .route("/licenses/overview", get(licenses_overview))
        .route(
            "/recurring_invoices/form_options",
            get(recurring_invoices_form_options),
        )
        .route(
            "/recurring_invoices/overview",
            get(recurring_invoices_overview),
        )
        .route("/clients/overview", get(clients_overview))
        .route("/cloud_cost/form_options", get(cloud_cost_form_options))
        .route("/cloud_cost/overview", get(cloud_cost_overview))
        .with_state(pool)
}

#[derive(Serialize)]
struct RentStats {
    total: f64,
    total_label: String,
    locations: i64,
    area: f64,
    area_label: String,
    expiring: i64,
}

#[derive(Serialize)]
struct UtilityStats {
    total: f64,
    total_label: String,
    electricity: f64,
    electricity_label: String,
    water: f64,
    water_label: String,
    gas: f64,
    gas_label: String,
}

#[derive(Serialize)]
struct RentTrendPoint {
    period: String,
    month: String,
    total: f64,
    total_label: String,
}

#[derive(Serialize)]
struct UtilityTrendPoint {
    period: String,
    month: String,
    electricity: f64,
    electricity_label: String,
    water: f64,
    water_label: String,
    gas: f64,
    gas_label: String,
}

#[derive(Serialize)]
struct RentOverview {
    rent_stats: RentStats,
    utility_stats: UtilityStats,
    rent_trend: Vec<RentTrendPoint>,
    utility_trend: Vec<UtilityTrendPoint>,
    rents: Vec<Value>,
    utilities: Vec<Value>,
    rent_create_action: Value,
    utility_create_action: Value,
    permissions: Value,
    empty_messages: Value,
}

#[derive(Deserialize)]
struct RentOverviewQuery {
    rent_search: Option<String>,
    utility_search: Option<String>,
}

#[derive(Serialize)]
struct PaymentStats {
    total: i64,
    expense: f64,
    income: f64,
    expense_label: String,
    income_label: String,
    pending_invoice: i64,
    overdue_invoice: i64,
}

#[derive(Serialize)]
struct PaymentsOverview {
    stats: PaymentStats,
    payments: Vec<Value>,
    invoice_reminder_stats: Value,
    pending_invoice_payments: Vec<Value>,
    permissions: Value,
    create_action: Value,
    payment_type_options: Vec<Value>,
    payment_method_options: Vec<Value>,
    invoice_status_options: Vec<Value>,
    empty_message: String,
}

#[derive(Deserialize)]
struct PaymentsOverviewQuery {
    search: Option<String>,
    payment_type: Option<String>,
    payment_method: Option<String>,
    invoice_status: Option<String>,
}

#[derive(Serialize)]
struct VendorStats {
    total: i64,
    suppliers: i64,
    customers: i64,
    active: i64,
}

#[derive(Serialize)]
struct VendorsOverview {
    stats: VendorStats,
    vendors: Vec<Value>,
    type_options: Vec<Value>,
    status_options: Vec<Value>,
    create_action: Value,
    empty_message: String,
}

#[derive(Deserialize)]
struct VendorsOverviewQuery {
    search: Option<String>,
    vendor_type: Option<String>,
    status: Option<String>,
}

#[derive(Serialize)]
struct RecurringInvoiceStats {
    active_recurring: i64,
    total_monthly: f64,
    total_monthly_label: String,
    pending_reminders: i64,
    sent_reminders: i64,
}

#[derive(Serialize)]
struct RecurringInvoicesOverview {
    stats: RecurringInvoiceStats,
    recurring: Vec<Value>,
    reminders: Vec<Value>,
    clients: Vec<Value>,
    invoices: Vec<Value>,
    permissions: Value,
    create_action: Value,
    reminder_create_action: Value,
    empty_messages: Value,
}

#[derive(Serialize)]
struct ClientStats {
    total: i64,
    active: i64,
    prospect: i64,
    total_revenue: f64,
    total_revenue_label: String,
}

#[derive(Serialize)]
struct ClientsOverview {
    stats: ClientStats,
    clients: Vec<Value>,
    contacts: Vec<Value>,
    interactions: Vec<Value>,
    detail_client: Option<Value>,
    detail_contacts: Vec<Value>,
    detail_interactions: Vec<Value>,
    type_options: Vec<Value>,
    source_options: Vec<Value>,
    interaction_type_options: Vec<Value>,
    create_action: Value,
    empty_message: String,
}

#[derive(Deserialize)]
struct ClientsOverviewQuery {
    search: Option<String>,
    selected_client_id: Option<String>,
}

#[derive(Serialize)]
struct CloudCostOverview {
    total_providers: i64,
    total_resources: i64,
    total_domains: i64,
    total_software: i64,
    monthly_cloud_cost: f64,
    monthly_domain_cost: f64,
    monthly_software_cost: f64,
    total_monthly_cost: f64,
    monthly_cloud_cost_label: String,
    monthly_domain_cost_label: String,
    monthly_software_cost_label: String,
    total_monthly_cost_label: String,
    providers: Vec<Value>,
    provider_options: Vec<Value>,
    resources: Vec<Value>,
    bills: Vec<Value>,
    domains: Vec<Value>,
    software: Vec<Value>,
    permissions: Value,
    empty_messages: Value,
    bill_create_action: Value,
}

#[derive(Deserialize)]
struct ProjectsOverviewQuery {
    search: Option<String>,
    status: Option<String>,
}

#[derive(Deserialize)]
struct ContractsOverviewQuery {
    search: Option<String>,
    status: Option<String>,
}

#[derive(Deserialize)]
struct AssetRecordsOverviewQuery {
    search: Option<String>,
    action: Option<String>,
}

#[derive(Deserialize)]
struct OperationLogsOverviewQuery {
    search: Option<String>,
    #[serde(rename = "type")]
    operation_type: Option<String>,
    table: Option<String>,
}

#[derive(Serialize)]
struct ProjectsOverview {
    stats: ProjectOverviewStats,
    projects: Vec<Value>,
    status_options: Vec<Value>,
    rd_cost_type_options: Vec<Value>,
    employee_options: Vec<Value>,
    asset_options: Vec<Value>,
    create_action: Value,
    permissions: Value,
    empty_message: String,
}

#[derive(Serialize)]
struct ProjectOverviewStats {
    total_projects: i64,
    ongoing_count: i64,
    total_budget: f64,
    total_budget_label: String,
    total_members: i64,
    total_rd_cost: f64,
    total_rd_cost_label: String,
}

#[derive(Clone, Serialize)]
struct DealStageOption {
    value: String,
    label: String,
    color: String,
}

#[derive(Serialize)]
struct DealPipelineStageGroup {
    value: String,
    label: String,
    color: String,
    count: i64,
    amount: f64,
    amount_label: String,
    deals: Vec<Value>,
}

#[derive(Serialize)]
struct SalaryOverview {
    stats: SalaryStats,
    monthly_trend: Vec<SalaryTrendPoint>,
    salaries: Vec<Value>,
    employees: Vec<Value>,
    current_insurance_config: Option<Value>,
    cost_analysis: SalaryCostAnalysis,
    create_action: Value,
    insurance_config_create_action: Value,
    insurance_preview_action: Value,
    permissions: Value,
    empty_message: String,
}

#[derive(Deserialize)]
struct SalaryOverviewQuery {
    search: Option<String>,
}

#[derive(Deserialize)]
struct SalaryInsurancePreviewRequest {
    employee_id: Option<String>,
    base_salary: Option<Value>,
}

#[derive(Serialize)]
struct SalaryInsurancePreview {
    social_security_base: f64,
    housing_fund_base: f64,
    social_security: f64,
    housing_fund: f64,
    social_security_company: f64,
    housing_fund_company: f64,
    social_security_input: String,
    housing_fund_input: String,
    social_security_company_input: String,
    housing_fund_company_input: String,
}

#[derive(Serialize)]
struct SalaryStats {
    total: f64,
    total_label: String,
    employees: i64,
    employees_label: String,
    average: f64,
    average_label: String,
    pending: i64,
    pending_label: String,
}

#[derive(Serialize)]
struct SalaryTrendPoint {
    period: String,
    month: String,
    total: f64,
    total_label: String,
}

#[derive(Serialize)]
struct SalaryCostTrendPoint {
    period: String,
    month: String,
    employee_cost: f64,
    employee_cost_label: String,
    social_security_cost: f64,
    social_security_cost_label: String,
    housing_fund_cost: f64,
    housing_fund_cost_label: String,
    total_cost: f64,
    total_cost_label: String,
}

#[derive(Serialize)]
struct SalaryCostStats {
    total_cost: f64,
    total_cost_label: String,
    employee_cost: f64,
    employee_cost_label: String,
    company_cost: f64,
    company_cost_label: String,
    departments: i64,
    departments_label: String,
    month_over_month: f64,
    month_over_month_label: String,
    employee_cost_share_label: String,
}

#[derive(Serialize)]
struct SalaryDepartmentCost {
    name: String,
    value: f64,
    value_label: String,
    percent_label: String,
}

#[derive(Serialize)]
struct SalaryCostAnalysis {
    stats: SalaryCostStats,
    monthly_trend: Vec<SalaryCostTrendPoint>,
    department_costs: Vec<SalaryDepartmentCost>,
}

#[derive(Serialize)]
struct TimeEntriesOverview {
    today_minutes: i64,
    total_records: i64,
    billable_minutes: i64,
    billable_amount: f64,
    unbilled_count: i64,
    stats: Value,
    entries: Vec<Value>,
    project_options: Vec<Value>,
    client_options: Vec<Value>,
    billable_options: Vec<Value>,
    active_timer: Value,
    invoice_action: Value,
    permissions: Value,
    create_action: Value,
    empty_message: String,
    today: String,
}

#[derive(sqlx::FromRow)]
struct RentStatsRow {
    total: f64,
    locations: i64,
    area: f64,
    expiring: i64,
}

#[derive(sqlx::FromRow)]
struct UtilityStatsRow {
    total: f64,
    electricity: f64,
    water: f64,
    gas: f64,
}

#[derive(sqlx::FromRow)]
struct UtilityTrendRow {
    utility_type: String,
    total: f64,
}

#[derive(sqlx::FromRow)]
struct PaymentStatsRow {
    total: i64,
    expense: f64,
    income: f64,
    pending_invoice: i64,
    overdue_invoice: i64,
}

#[derive(sqlx::FromRow)]
struct SalaryStatsRow {
    total: f64,
    employees: i64,
    average: f64,
    pending: i64,
}

#[derive(sqlx::FromRow)]
struct SalaryCostTrendRow {
    employee_cost: f64,
    social_security_cost: f64,
    housing_fund_cost: f64,
}

#[derive(sqlx::FromRow)]
struct SalaryDepartmentRow {
    name: String,
    value: f64,
}

#[derive(sqlx::FromRow)]
struct SalaryInsuranceConfigRow {
    social_security_base_min: f64,
    social_security_base_max: f64,
    housing_fund_base_min: f64,
    housing_fund_base_max: f64,
    pension_personal_rate: f64,
    pension_company_rate: f64,
    medical_personal_rate: f64,
    medical_company_rate: f64,
    unemployment_personal_rate: f64,
    unemployment_company_rate: f64,
    work_injury_company_rate: f64,
    maternity_company_rate: f64,
    housing_fund_personal_rate: f64,
    housing_fund_company_rate: f64,
}

#[derive(sqlx::FromRow)]
struct SalaryInsuranceConfigOverviewRow {
    id: String,
    city: String,
    social_security_base_min: f64,
    social_security_base_max: f64,
    housing_fund_base_min: f64,
    housing_fund_base_max: f64,
    pension_personal_rate: f64,
    pension_company_rate: f64,
    medical_personal_rate: f64,
    medical_company_rate: f64,
    unemployment_personal_rate: f64,
    unemployment_company_rate: f64,
    work_injury_company_rate: f64,
    maternity_company_rate: f64,
    housing_fund_personal_rate: f64,
    housing_fund_company_rate: f64,
    notes: String,
}

#[derive(sqlx::FromRow)]
struct SalaryEmployeeBaseRow {
    base_salary: f64,
    social_security_base: f64,
    housing_fund_base: f64,
}

fn salary_signed_cny(value: f64, sign: &str) -> String {
    if value.abs() < 0.005 {
        format!("{sign}¥0")
    } else {
        format!("{sign}{}", format_cny(value.abs()))
    }
}

fn salary_percent_label(value: f64) -> String {
    format!("{:.1}%", (value * 10.0).round() / 10.0)
}

fn salary_status_badge(status: &str) -> Value {
    if status == "paid" {
        json!({
            "value": "paid",
            "label": "已发放",
            "class_name": "bg-accent",
        })
    } else {
        json!({
            "value": "pending",
            "label": "待发放",
            "class_name": "bg-warning",
        })
    }
}

#[derive(sqlx::FromRow)]
struct TimeEntriesOverviewRow {
    today_minutes: i64,
    total_records: i64,
    billable_minutes: i64,
    billable_amount: f64,
    unbilled_count: i64,
}

fn time_minutes_label(minutes: i64) -> String {
    let safe_minutes = minutes.max(0);
    let hours = safe_minutes / 60;
    let mins = safe_minutes % 60;
    if hours > 0 {
        format!("{hours}h {mins}m")
    } else {
        format!("{mins}m")
    }
}

fn time_billable_badge(is_billable: bool, is_invoiced: bool) -> Value {
    if is_invoiced {
        return json!({
            "value": "invoiced",
            "label": "已开票",
            "variant": "outline"
        });
    }

    if is_billable {
        json!({
            "value": true,
            "label": "可计费",
            "variant": "default"
        })
    } else {
        json!({
            "value": false,
            "label": "不计费",
            "variant": "secondary"
        })
    }
}

fn time_entry_delete_action(
    id: Option<String>,
    description: Option<String>,
    can_delete: bool,
) -> Value {
    if !can_delete {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };
    let description = description
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("该工时记录");
    operations_delete_action(
        format!("/api/v1/time_entries/{id}"),
        format!("确定要删除工时记录「{description}」吗？"),
        "工时记录已删除",
    )
}

fn time_entry_create_action(can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    operations_create_action("/api/v1/time_entries".to_string(), "手动记录", "工时已记录")
}

fn time_billable_options() -> Vec<Value> {
    [("1", "可计费"), ("0", "不可计费")]
        .into_iter()
        .map(|(value, label)| {
            json!({
                "value": value,
                "label": label,
            })
        })
        .collect()
}

fn time_status_badge(status: &str) -> Value {
    match status {
        "running" => json!({
            "value": "running",
            "label": "计时中",
            "variant": "secondary"
        }),
        "invoiced" => json!({
            "value": "invoiced",
            "label": "已开票",
            "variant": "outline"
        }),
        _ => json!({
            "value": "logged",
            "label": "已记录",
            "variant": "default"
        }),
    }
}

fn reconciliation_text_cell(key: &str, value: impl Into<String>) -> Value {
    json!({
        "key": key,
        "value": value.into(),
    })
}

fn reconciliation_amount_cell(key: &str, amount: f64, class_name: &str) -> Value {
    json!({
        "key": key,
        "value": format_cny(amount),
        "class_name": class_name,
    })
}

fn reconciliation_badge_cell(key: &str, label: &str, variant: &str) -> Value {
    json!({
        "key": key,
        "value": label,
        "badge": {
            "label": label,
            "variant": variant,
        }
    })
}

fn reconciliation_action() -> Value {
    json!({
        "label": "匹配",
        "icon": "Link",
        "enabled": true,
    })
}

fn reconciliation_section(key: &str, title: &str, headers: Vec<&str>, rows: Vec<Value>) -> Value {
    json!({
        "key": key,
        "title": title,
        "export_label": "导出",
        "headers": headers,
        "rows": rows,
        "empty_message": format!("暂无{title}"),
    })
}

fn reconciliation_transaction_type_badge(transaction_type: &str) -> (&'static str, &'static str) {
    if matches!(transaction_type, "credit" | "deposit" | "income") {
        ("收入", "default")
    } else {
        ("支出", "secondary")
    }
}

fn reconciliation_payment_type_badge(payment_type: &str) -> (&'static str, &'static str) {
    if matches!(payment_type, "收入" | "income" | "receipt") {
        ("收款", "default")
    } else {
        ("付款", "secondary")
    }
}

fn reconciliation_invoice_type_badge(invoice_type: &str) -> (String, &'static str) {
    if invoice_type.trim().is_empty() {
        ("未分类".to_string(), "secondary")
    } else if invoice_type == "增值税专用发票" {
        (invoice_type.to_string(), "default")
    } else {
        (invoice_type.to_string(), "secondary")
    }
}

fn reconciliation_billing_cycle_badge(cycle: &str) -> (&'static str, &'static str) {
    match cycle {
        "monthly" => ("月付", "outline"),
        "annual" | "yearly" => ("年付", "outline"),
        _ => ("永久", "outline"),
    }
}

fn reconciliation_date_label(date: Option<String>) -> String {
    date.filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "-".to_string())
}

#[derive(sqlx::FromRow)]
struct VendorStatsRow {
    total: i64,
    suppliers: i64,
    customers: i64,
    active: i64,
}

#[derive(sqlx::FromRow)]
struct RecurringInvoiceStatsRow {
    active_recurring: i64,
    total_monthly: f64,
    pending_reminders: i64,
    sent_reminders: i64,
}

#[derive(sqlx::FromRow)]
struct ClientStatsRow {
    total: i64,
    active: i64,
    prospect: i64,
    total_revenue: f64,
}

#[derive(sqlx::FromRow)]
struct CloudCostOverviewRow {
    total_providers: i64,
    total_resources: i64,
    total_domains: i64,
    total_software: i64,
    monthly_cloud_cost: f64,
    monthly_domain_cost: f64,
    monthly_software_cost: f64,
}

#[derive(Default)]
struct ProjectRdCostGroup {
    total: f64,
    by_type: BTreeMap<String, f64>,
    items: Vec<Value>,
}

fn rent_number_label(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    if (rounded.fract()).abs() < 0.005 {
        format!("{}", rounded as i64)
    } else {
        let mut text = format!("{rounded:.2}");
        while text.contains('.') && text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
        text
    }
}

fn rent_area_label(value: f64) -> String {
    format!("{}㎡", rent_number_label(value))
}

fn rent_date_label(value: Option<&str>) -> String {
    value
        .filter(|date| !date.trim().is_empty())
        .unwrap_or("-")
        .to_string()
}

fn rent_contract_period_label(start: Option<&str>, end: Option<&str>) -> String {
    match (start, end) {
        (Some(start), Some(end)) if !start.is_empty() && !end.is_empty() => {
            format!("{start} 至 {end}")
        }
        _ => "-".to_string(),
    }
}

fn rent_status_badge(status: &str) -> Value {
    match status {
        "active" => json!({
            "value": "active",
            "label": "活跃",
            "variant": "default",
            "class_name": "bg-accent",
        }),
        "paid" => json!({
            "value": "paid",
            "label": "已支付",
            "variant": "default",
            "class_name": "bg-accent",
        }),
        "pending" | "unpaid" => json!({
            "value": "pending",
            "label": "待支付",
            "variant": "secondary",
            "class_name": "bg-warning",
        }),
        _ => json!({
            "value": status,
            "label": "已到期",
            "variant": "secondary",
            "class_name": "bg-warning",
        }),
    }
}

fn utility_type_display(utility_type: &str) -> (&'static str, &'static str, &'static str) {
    match utility_type {
        "electricity" => ("电费", "Zap", "text-warning"),
        "water" => ("水费", "Droplet", "text-primary"),
        "gas" => ("燃气费", "Flame", "text-destructive"),
        _ => ("其他", "Zap", "text-muted-foreground"),
    }
}

async fn rent_form_options(_user: AuthUser) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({
        "payment_cycle_options": rent_payment_cycle_options(),
        "utility_type_options": utility_type_options(),
        "default_values": {
            "payment_cycle": "monthly",
            "utility_type": "electricity"
        }
    })))
}

fn rent_payment_cycle_options() -> Vec<Value> {
    [
        ("monthly", "月付"),
        ("quarterly", "季付"),
        ("yearly", "年付"),
    ]
    .into_iter()
    .map(|(value, label)| {
        json!({
            "value": value,
            "label": label,
        })
    })
    .collect()
}

fn utility_type_options() -> Vec<Value> {
    ["electricity", "water", "gas"]
        .into_iter()
        .map(|value| {
            let (label, icon, class_name) = utility_type_display(value);
            json!({
                "value": value,
                "label": label,
                "icon": icon,
                "class_name": class_name,
            })
        })
        .collect()
}

fn rent_delete_action(id: Option<String>) -> Value {
    let id = id.unwrap_or_default();
    json!({
        "endpoint": format!("/api/v1/rent/{id}"),
        "success_message": "场地已删除",
        "confirm_message": "此操作无法撤销。确定要删除这个场地吗？",
    })
}

fn rent_create_action(can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }

    operations_create_action("/api/v1/rent".to_string(), "添加场地", "场地已添加")
}

fn rent_update_action(id: Option<String>, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };

    operations_update_action(format!("/api/v1/rent/{id}"), "保存修改", "场地信息已更新")
}

fn utility_create_action(can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }

    operations_create_action("/api/v1/utilities".to_string(), "录入账单", "账单已添加")
}

fn utility_update_action(id: Option<String>, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };

    operations_update_action(format!("/api/v1/utilities/{id}"), "保存修改", "账单已更新")
}

fn utility_delete_action(id: Option<String>) -> Value {
    let id = id.unwrap_or_default();
    json!({
        "endpoint": format!("/api/v1/utilities/{id}"),
        "success_message": "账单已删除",
        "confirm_message": "此操作无法撤销。确定要删除这条账单吗？",
    })
}

fn utility_mark_paid_action(id: Option<String>, status: &str, can_manage: bool) -> Value {
    if !can_manage || !matches!(status, "pending" | "unpaid") {
        return json!(null);
    }
    let id = id.unwrap_or_default();
    if id.is_empty() {
        return json!(null);
    }

    operations_post_action(
        format!("/api/v1/utilities/{id}/mark_paid"),
        "标记已支付",
        "账单已标记为已支付",
        "paid",
    )
}

fn utility_display_value(row: &SqliteRow, can_manage: bool) -> Value {
    let location_name = text(row, "location_name").unwrap_or_default();
    let utility_type = text(row, "utility_type").unwrap_or_default();
    let (type_label, type_icon, type_class) = utility_type_display(&utility_type);
    let status = text(row, "status").unwrap_or_else(|| "pending".to_string());
    let amount = number(row, "amount");
    let usage = number(row, "usage");
    let unit_price = number(row, "unit_price");
    let due_date = text(row, "due_date");
    let status_badge = rent_status_badge(&status);
    let due_date_label = rent_date_label(due_date.as_deref());
    let can_mark_paid = can_manage && matches!(status.as_str(), "pending" | "unpaid");
    let id = text(row, "id");

    json!({
        "id": id.clone(),
        "location_name": location_name,
        "utility_type": utility_type,
        "utility_type_label": type_label,
        "utility_type_icon": type_icon,
        "utility_type_class": type_class,
        "period": text(row, "period"),
        "usage": usage,
        "usage_label": rent_number_label(usage),
        "unit_price": unit_price,
        "unit_price_label": format_cny(unit_price),
        "amount": amount,
        "amount_label": format_cny(amount),
        "due_date": due_date,
        "due_date_label": due_date_label,
        "status": status,
        "status_badge": status_badge,
        "notes": text(row, "notes"),
        "created_at": text(row, "created_at"),
        "updated_at": text(row, "updated_at"),
        "can_mark_paid": can_mark_paid,
        "mark_paid_action": utility_mark_paid_action(id.clone(), &status, can_manage),
        "can_delete": can_manage,
        "delete_action": utility_delete_action(id),
    })
}

async fn rent_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(query): Query<RentOverviewQuery>,
) -> Result<Json<RentOverview>, ApiError> {
    let rent_row: RentStatsRow = sqlx::query_as(
        r#"
        SELECT
            COALESCE(SUM(COALESCE(monthly_rent, 0.0)), 0.0) AS total,
            COUNT(*) AS locations,
            COALESCE(SUM(COALESCE(area, 0.0)), 0.0) AS area,
            COALESCE(SUM(CASE
                WHEN contract_end_date IS NOT NULL
                 AND date(contract_end_date) >= date('now')
                 AND date(contract_end_date) <= date('now', '+3 months')
                THEN 1 ELSE 0 END), 0) AS expiring
        FROM rent_records
        WHERE company_id = ? AND status = 'active'
        "#,
    )
    .bind(&user.company_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let utility_row: UtilityStatsRow = sqlx::query_as(
        r#"
        SELECT
            COALESCE(SUM(CASE WHEN status IN ('pending', 'unpaid') THEN COALESCE(amount, 0.0) ELSE 0.0 END), 0.0) AS total,
            COALESCE(SUM(CASE WHEN utility_type = 'electricity' AND status IN ('pending', 'unpaid') THEN COALESCE(amount, 0.0) ELSE 0.0 END), 0.0) AS electricity,
            COALESCE(SUM(CASE WHEN utility_type = 'water' AND status IN ('pending', 'unpaid') THEN COALESCE(amount, 0.0) ELSE 0.0 END), 0.0) AS water,
            COALESCE(SUM(CASE WHEN utility_type = 'gas' AND status IN ('pending', 'unpaid') THEN COALESCE(amount, 0.0) ELSE 0.0 END), 0.0) AS gas
        FROM utilities
        WHERE company_id = ?
        "#,
    )
    .bind(&user.company_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let rent_rows = sqlx::query(
        r#"
        SELECT
            id,
            location_name,
            property_type,
            address,
            area,
            monthly_rent,
            deposit,
            contract_start_date,
            contract_end_date,
            next_payment_date,
            payment_cycle,
            landlord,
            landlord_contact,
            contract_id,
            status,
            notes,
            created_at,
            updated_at
        FROM rent_records
        WHERE company_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let utility_rows = sqlx::query(
        r#"
        SELECT
            id,
            location_name,
            utility_type,
            period,
            usage,
            unit_price,
            amount,
            due_date,
            status,
            notes,
            created_at,
            updated_at
        FROM utilities
        WHERE company_id = ?
        ORDER BY period DESC, created_at DESC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let rent_search = query.rent_search.unwrap_or_default().trim().to_lowercase();
    let utility_search = query
        .utility_search
        .unwrap_or_default()
        .trim()
        .to_lowercase();
    let can_delete = current_user_can_manage(&pool, &user).await?;

    let rents = rent_rows
        .into_iter()
        .filter_map(|row| {
            let location_name = text(&row, "location_name").unwrap_or_default();
            let address = text(&row, "address");
            if !rent_search.is_empty() {
                let matches = [Some(location_name.as_str()), address.as_deref()]
                    .into_iter()
                    .flatten()
                    .any(|value| value.to_lowercase().contains(&rent_search));
                if !matches {
                    return None;
                }
            }
            let area = number(&row, "area");
            let monthly_rent = number(&row, "monthly_rent");
            let status = text(&row, "status").unwrap_or_else(|| "active".to_string());
            let contract_start_date = text(&row, "contract_start_date");
            let contract_end_date = text(&row, "contract_end_date");
            let next_payment_date = text(&row, "next_payment_date");
            let status_badge = rent_status_badge(&status);
            let contract_period_label = rent_contract_period_label(
                contract_start_date.as_deref(),
                contract_end_date.as_deref(),
            );
            let next_payment_label = rent_date_label(next_payment_date.as_deref());
            let id = text(&row, "id");

            Some(json!({
                "id": id.clone(),
                "location_name": location_name,
                "property_type": text(&row, "property_type"),
                "address": address,
                "area": area,
                "area_label": rent_area_label(area),
                "monthly_rent": monthly_rent,
                "monthly_rent_label": format_cny(monthly_rent),
                "deposit": number(&row, "deposit"),
                "contract_start_date": contract_start_date,
                "contract_end_date": contract_end_date,
                "contract_period_label": contract_period_label,
                "next_payment_date": next_payment_date,
                "next_payment_label": next_payment_label,
                "payment_cycle": text(&row, "payment_cycle"),
                "landlord": text(&row, "landlord"),
                "landlord_contact": text(&row, "landlord_contact"),
                "contract_id": text(&row, "contract_id"),
                "status": status.clone(),
                "status_badge": status_badge,
                "notes": text(&row, "notes"),
                "created_at": text(&row, "created_at"),
                "updated_at": text(&row, "updated_at"),
                "update_action": rent_update_action(id.clone(), can_delete),
                "can_delete": can_delete,
                "delete_action": rent_delete_action(id),
            }))
        })
        .collect::<Vec<_>>();

    let utilities = utility_rows
        .into_iter()
        .filter_map(|row| {
            let location_name = text(&row, "location_name").unwrap_or_default();
            let utility_type = text(&row, "utility_type").unwrap_or_default();
            let (type_label, type_icon, type_class) = utility_type_display(&utility_type);
            if !utility_search.is_empty() {
                let matches = [location_name.as_str(), type_label, utility_type.as_str()]
                    .into_iter()
                    .any(|value| value.to_lowercase().contains(&utility_search));
                if !matches {
                    return None;
                }
            }
            let status = text(&row, "status").unwrap_or_else(|| "pending".to_string());
            let amount = number(&row, "amount");
            let usage = number(&row, "usage");
            let unit_price = number(&row, "unit_price");
            let due_date = text(&row, "due_date");
            let status_badge = rent_status_badge(&status);
            let due_date_label = rent_date_label(due_date.as_deref());
            let can_mark_paid = can_delete && matches!(status.as_str(), "pending" | "unpaid");
            let id = text(&row, "id");

            Some(json!({
                "id": id.clone(),
                "location_name": location_name,
                "utility_type": utility_type,
                "utility_type_label": type_label,
                "utility_type_icon": type_icon,
                "utility_type_class": type_class,
                "period": text(&row, "period"),
                "usage": usage,
                "usage_label": rent_number_label(usage),
                "unit_price": unit_price,
                "unit_price_label": format_cny(unit_price),
                "amount": amount,
                "amount_label": format_cny(amount),
                "due_date": due_date,
                "due_date_label": due_date_label,
                "status": status,
                "status_badge": status_badge,
                "notes": text(&row, "notes"),
                "created_at": text(&row, "created_at"),
                "updated_at": text(&row, "updated_at"),
                "update_action": utility_update_action(id.clone(), can_delete),
                "can_mark_paid": can_mark_paid,
                "mark_paid_action": utility_mark_paid_action(
                    id.clone(),
                    &status,
                    can_delete,
                ),
                "can_delete": can_delete,
                "delete_action": utility_delete_action(id),
            }))
        })
        .collect::<Vec<_>>();

    let periods = last_six_periods();
    let rent_trend = periods
        .iter()
        .map(|(period, month)| RentTrendPoint {
            period: period.clone(),
            month: month.clone(),
            total: rent_row.total,
            total_label: format_cny(rent_row.total),
        })
        .collect();

    let mut utility_trend = Vec::with_capacity(periods.len());
    for (period, month) in periods {
        let rows: Vec<UtilityTrendRow> = sqlx::query_as(
            r#"
            SELECT utility_type, COALESCE(SUM(COALESCE(amount, 0.0)), 0.0) AS total
            FROM utilities
            WHERE company_id = ? AND period = ?
            GROUP BY utility_type
            "#,
        )
        .bind(&user.company_id)
        .bind(&period)
        .fetch_all(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

        let mut point = UtilityTrendPoint {
            period,
            month,
            electricity: 0.0,
            electricity_label: format_cny(0.0),
            water: 0.0,
            water_label: format_cny(0.0),
            gas: 0.0,
            gas_label: format_cny(0.0),
        };
        for row in rows {
            match row.utility_type.as_str() {
                "electricity" => point.electricity = row.total,
                "water" => point.water = row.total,
                "gas" => point.gas = row.total,
                _ => {}
            }
        }
        point.electricity_label = format_cny(point.electricity);
        point.water_label = format_cny(point.water);
        point.gas_label = format_cny(point.gas);
        utility_trend.push(point);
    }

    Ok(Json(RentOverview {
        rent_stats: RentStats {
            total: rent_row.total,
            total_label: format_cny(rent_row.total),
            locations: rent_row.locations,
            area: rent_row.area,
            area_label: rent_area_label(rent_row.area),
            expiring: rent_row.expiring,
        },
        utility_stats: UtilityStats {
            total: utility_row.total,
            total_label: format_cny(utility_row.total),
            electricity: utility_row.electricity,
            electricity_label: format_cny(utility_row.electricity),
            water: utility_row.water,
            water_label: format_cny(utility_row.water),
            gas: utility_row.gas,
            gas_label: format_cny(utility_row.gas),
        },
        rent_trend,
        utility_trend,
        rents,
        utilities,
        rent_create_action: rent_create_action(can_delete),
        utility_create_action: utility_create_action(can_delete),
        permissions: json!({
            "can_create": can_delete,
            "can_update": can_delete,
            "can_delete": can_delete,
            "can_mark_paid": can_delete,
        }),
        empty_messages: json!({
            "rents": if rent_search.is_empty() { "暂无场地记录" } else { "未找到匹配的场地" },
            "utilities": if utility_search.is_empty() { "暂无账单记录" } else { "未找到匹配的账单" },
        }),
    }))
}

async fn mark_utility_paid(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let can_manage = current_user_can_manage(&pool, &user).await?;
    if !can_manage {
        return Err(api_err(StatusCode::FORBIDDEN, "forbidden"));
    }

    let result = sqlx::query(
        r#"
        UPDATE utilities
        SET status = 'paid', updated_at = CURRENT_TIMESTAMP
        WHERE id = ? AND company_id = ?
        "#,
    )
    .bind(&id)
    .bind(&user.company_id)
    .execute(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    if result.rows_affected() == 0 {
        return Err(api_err(StatusCode::NOT_FOUND, "utility not found"));
    }

    let row = sqlx::query(
        r#"
        SELECT
            id,
            location_name,
            utility_type,
            period,
            usage,
            unit_price,
            amount,
            due_date,
            status,
            notes,
            created_at,
            updated_at
        FROM utilities
        WHERE id = ? AND company_id = ?
        LIMIT 1
        "#,
    )
    .bind(&id)
    .bind(&user.company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?
    .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "utility not found"))?;

    Ok(Json(utility_display_value(&row, can_manage)))
}

async fn vendors_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(query): Query<VendorsOverviewQuery>,
) -> Result<Json<VendorsOverview>, ApiError> {
    let stats: VendorStatsRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) AS total,
            COALESCE(SUM(CASE WHEN vendor_type IN ('supplier', 'both') THEN 1 ELSE 0 END), 0) AS suppliers,
            COALESCE(SUM(CASE WHEN vendor_type IN ('customer', 'both') THEN 1 ELSE 0 END), 0) AS customers,
            COALESCE(SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END), 0) AS active
        FROM vendors
        WHERE company_id = ?
        "#,
    )
    .bind(&user.company_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let rows = sqlx::query(
        r#"
        SELECT
            id,
            name,
            short_name,
            unified_code,
            legal_person,
            registered_capital,
            establishment_date,
            business_scope,
            address,
            contact_person,
            contact_phone,
            contact_email,
            bank_name,
            bank_account,
            vendor_type,
            status,
            notes,
            license_file_path,
            license_file_name,
            created_at,
            updated_at
        FROM vendors
        WHERE company_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let search = query.search.unwrap_or_default().trim().to_lowercase();
    let vendor_type_filter = query.vendor_type.unwrap_or_default();
    let status_filter = query.status.unwrap_or_default();
    let has_filters = !search.is_empty()
        || (!vendor_type_filter.is_empty() && vendor_type_filter != "all")
        || (!status_filter.is_empty() && status_filter != "all");
    let can_manage = current_user_can_manage(&pool, &user).await?;

    let vendors = rows
        .into_iter()
        .filter_map(|row| {
            let id = text(&row, "id");
            let name = text(&row, "name");
            let short_name = text(&row, "short_name");
            let unified_code = text(&row, "unified_code");
            let contact_person = text(&row, "contact_person");
            let vendor_type = text(&row, "vendor_type").unwrap_or_else(|| "supplier".to_string());
            let status = text(&row, "status").unwrap_or_else(|| "active".to_string());

            if !vendor_type_filter.is_empty()
                && vendor_type_filter != "all"
                && vendor_type != vendor_type_filter
            {
                return None;
            }
            if !status_filter.is_empty() && status_filter != "all" && status != status_filter {
                return None;
            }
            if !search.is_empty() {
                let search_fields = [
                    name.as_deref(),
                    short_name.as_deref(),
                    unified_code.as_deref(),
                    contact_person.as_deref(),
                ];
                let matches_search = search_fields
                    .into_iter()
                    .flatten()
                    .any(|value| value.to_lowercase().contains(&search));
                if !matches_search {
                    return None;
                }
            }

            let type_badge = vendor_type_badge(&vendor_type);
            let status_badge = vendor_status_badge(&status);
            let delete_action = vendor_delete_action(id.clone(), name.clone(), can_manage);
            let update_action = vendor_update_action(id.clone(), can_manage);

            Some(json!({
                "id": id.clone(),
                "name": name,
                "short_name": short_name,
                "unified_code": unified_code,
                "legal_person": text(&row, "legal_person"),
                "registered_capital": text(&row, "registered_capital"),
                "establishment_date": text(&row, "establishment_date"),
                "business_scope": text(&row, "business_scope"),
                "address": text(&row, "address"),
                "contact_person": contact_person,
                "contact_phone": text(&row, "contact_phone"),
                "contact_email": text(&row, "contact_email"),
                "bank_name": text(&row, "bank_name"),
                "bank_account": text(&row, "bank_account"),
                "vendor_type": vendor_type,
                "type_badge": type_badge,
                "status": status,
                "status_badge": status_badge,
                "notes": text(&row, "notes"),
                "license_file_path": text(&row, "license_file_path"),
                "license_file_name": text(&row, "license_file_name"),
                "created_at": text(&row, "created_at"),
                "updated_at": text(&row, "updated_at"),
                "can_delete": can_manage,
                "update_action": update_action,
                "delete_action": delete_action,
            }))
        })
        .collect();

    let empty_message = if has_filters {
        "未找到匹配的友商"
    } else {
        "暂无友商资料，点击「新增友商」添加"
    }
    .to_string();

    Ok(Json(VendorsOverview {
        stats: VendorStats {
            total: stats.total,
            suppliers: stats.suppliers,
            customers: stats.customers,
            active: stats.active,
        },
        vendors,
        type_options: vendor_type_options(),
        status_options: vendor_status_options(),
        create_action: vendor_create_action(can_manage),
        empty_message,
    }))
}

fn vendor_create_action(can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }

    operations_create_action(
        "/api/v1/vendors".to_string(),
        "新增友商",
        "友商资料已成功创建",
    )
}

fn vendor_update_action(id: Option<String>, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };

    operations_update_action(
        format!("/api/v1/vendors/{id}"),
        "保存修改",
        "友商资料已成功更新",
    )
}

fn vendor_delete_action(id: Option<String>, name: Option<String>, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };
    let name = name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("该友商");
    operations_delete_action(
        format!("/api/vendors/{id}"),
        format!("确定要删除友商「{name}」吗？"),
        "友商资料已成功删除",
    )
}

fn vendor_type_badge(vendor_type: &str) -> Value {
    match vendor_type {
        "customer" => json!({
            "value": "customer",
            "label": "客户",
            "variant": "secondary",
        }),
        "both" => json!({
            "value": "both",
            "label": "供应商/客户",
            "variant": "outline",
        }),
        _ => json!({
            "value": "supplier",
            "label": "供应商",
            "variant": "default",
        }),
    }
}

fn vendor_status_badge(status: &str) -> Value {
    match status {
        "inactive" => json!({
            "value": "inactive",
            "label": "已停用",
            "variant": "secondary",
            "icon": "XCircle",
        }),
        _ => json!({
            "value": "active",
            "label": "合作中",
            "variant": "default",
            "icon": "CheckCircle",
        }),
    }
}

fn vendor_type_options() -> Vec<Value> {
    let mut options = vec![json!({
        "value": "all",
        "label": "全部类型",
        "variant": "outline",
    })];
    options.extend(
        ["supplier", "customer", "both"]
            .into_iter()
            .map(vendor_type_badge)
            .collect::<Vec<_>>(),
    );
    options
}

fn vendor_status_options() -> Vec<Value> {
    let mut options = vec![json!({
        "value": "all",
        "label": "全部状态",
        "variant": "outline",
    })];
    options.extend(
        ["active", "inactive"]
            .into_iter()
            .map(vendor_status_badge)
            .collect::<Vec<_>>(),
    );
    options
}

async fn licenses_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<Value>, ApiError> {
    let can_manage = current_user_can_manage(&pool, &user).await?;
    let today = Utc::now().date_naive();
    let rows = sqlx::query(
        r#"
        SELECT
            id,
            license_name,
            license_type,
            license_number,
            issuing_authority,
            issue_date,
            expiry_date,
            file_path,
            status,
            notes,
            created_at,
            updated_at
        FROM licenses
        WHERE company_id = ?
        ORDER BY expiry_date IS NULL, expiry_date ASC, created_at DESC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let mut valid = 0_i64;
    let mut expiring = 0_i64;
    let mut expired = 0_i64;
    let mut licenses = Vec::new();

    for row in rows {
        let status = text(&row, "status").unwrap_or_else(|| "valid".to_string());
        let expiry_date = text(&row, "expiry_date");
        let display_status = license_effective_status(&status, expiry_date.as_deref(), today);
        match display_status.as_str() {
            "expired" => expired += 1,
            "expiring" => expiring += 1,
            _ => valid += 1,
        }

        let file_path = text(&row, "file_path");
        let license_type = text(&row, "license_type").unwrap_or_else(|| "other".to_string());
        let license_type_label = license_type_label(&license_type);
        let issue_date = text(&row, "issue_date");
        let issue_date_label = cloud_date_label(issue_date.as_deref());
        let expiry_date_label = cloud_date_label(expiry_date.as_deref());
        let expiry_state = license_expiry_state(expiry_date.as_deref(), today);
        let status_badge = license_status_badge(&display_status);
        let id = text(&row, "id");
        let license_name = text(&row, "license_name");
        let has_file = file_path
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty());
        let delete_action = license_delete_action(id.clone(), license_name.clone(), can_manage);

        licenses.push(json!({
            "id": id,
            "license_name": license_name,
            "license_type": license_type,
            "license_type_label": license_type_label,
            "license_number": text(&row, "license_number"),
            "issuing_authority": text(&row, "issuing_authority"),
            "issue_date": issue_date,
            "issue_date_label": issue_date_label,
            "expiry_date": expiry_date,
            "expiry_date_label": expiry_date_label,
            "expiry_state": expiry_state,
            "file_path": file_path,
            "has_file": has_file,
            "status": status,
            "display_status": display_status,
            "status_badge": status_badge,
            "notes": text(&row, "notes"),
            "created_at": text(&row, "created_at"),
            "updated_at": text(&row, "updated_at"),
            "can_edit": can_manage,
            "update_action": license_update_action(id.clone(), can_manage),
            "can_delete": can_manage,
            "delete_action": delete_action,
        }));
    }

    Ok(Json(json!({
        "stats": {
            "total": licenses.len() as i64,
            "valid": valid,
            "expiring": expiring,
            "expired": expired,
        },
        "licenses": licenses,
        "permissions": {
            "can_manage": can_manage,
            "can_create": can_manage,
            "can_edit": can_manage,
            "can_delete": can_manage,
        },
        "create_action": license_create_action(can_manage),
        "empty_message": "还没有证照记录",
    })))
}

fn license_create_action(can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }

    operations_create_action("/api/v1/licenses".to_string(), "添加证照", "证照已添加")
}

fn license_update_action(id: Option<String>, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };

    operations_update_action(format!("/api/v1/licenses/{id}"), "保存证照", "证照已更新")
}

fn license_delete_action(
    id: Option<String>,
    license_name: Option<String>,
    can_manage: bool,
) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };
    let license_name = license_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("该证照");
    operations_delete_action(
        format!("/api/v1/licenses/{id}"),
        format!("确定要删除证照「{license_name}」吗？"),
        "证照已删除",
    )
}

async fn licenses_form_options(_user: AuthUser) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({
        "license_type_options": license_type_options(),
        "status_options": license_status_options(),
        "default_values": {
            "license_type": "business_license",
            "status": "valid"
        }
    })))
}

fn license_type_options() -> Vec<Value> {
    [
        "business_license",
        "tax_registration",
        "opening_permit",
        "organization_code",
        "qualification",
        "other",
    ]
    .into_iter()
    .map(|value| {
        json!({
            "value": value,
            "label": license_type_label(value),
        })
    })
    .collect()
}

fn license_type_label(license_type: &str) -> &'static str {
    match license_type {
        "business_license" => "营业执照",
        "tax_registration" => "税务登记证",
        "opening_permit" => "开户许可证",
        "organization_code" => "组织机构代码证",
        "qualification" => "资质证书",
        _ => "其他",
    }
}

fn license_effective_status(status: &str, expiry_date: Option<&str>, today: NaiveDate) -> String {
    if let Some(days_left) = expiry_date.and_then(|date| days_until_date(date, today)) {
        if days_left < 0 {
            return "expired".to_string();
        }
        if days_left <= 30 {
            return "expiring".to_string();
        }
    }

    match status {
        "expired" => "expired",
        "expiring" => "expiring",
        _ => "valid",
    }
    .to_string()
}

fn license_status_badge(status: &str) -> Value {
    match status {
        "expired" => json!({
            "value": "expired",
            "label": "已过期",
            "variant": "destructive",
            "class_name": "text-destructive",
            "icon": "AlertCircle",
        }),
        "expiring" => json!({
            "value": "expiring",
            "label": "即将到期",
            "variant": "outline",
            "class_name": "text-warning",
            "icon": "AlertCircle",
        }),
        _ => json!({
            "value": "valid",
            "label": "有效",
            "variant": "default",
            "class_name": "text-primary",
            "icon": "CheckCircle2",
        }),
    }
}

fn license_status_options() -> Vec<Value> {
    ["valid", "expiring", "expired"]
        .into_iter()
        .map(license_status_badge)
        .collect()
}

fn license_expiry_state(expiry_date: Option<&str>, today: NaiveDate) -> Value {
    match expiry_date.and_then(|date| days_until_date(date, today)) {
        Some(days_left) if days_left < 0 => json!({
            "state": "expired",
            "label": "已过期",
            "class_name": "text-destructive",
        }),
        Some(0) => json!({
            "state": "today",
            "label": "今天到期",
            "class_name": "text-destructive",
        }),
        Some(days_left) if days_left <= 30 => json!({
            "state": "expiring",
            "label": format!("{} 天后", days_left),
            "class_name": "text-warning",
        }),
        Some(days_left) => json!({
            "state": "valid",
            "label": format!("{} 天后", days_left),
            "class_name": "text-muted-foreground",
        }),
        None => json!({
            "state": "none",
            "label": "长期有效",
            "class_name": "text-muted-foreground",
        }),
    }
}

fn days_until_date(value: &str, today: NaiveDate) -> Option<i64> {
    let prefix = value.chars().take(10).collect::<String>();
    NaiveDate::parse_from_str(&prefix, "%Y-%m-%d")
        .ok()
        .map(|date| (date - today).num_days())
}

async fn clients_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(query): Query<ClientsOverviewQuery>,
) -> Result<Json<ClientsOverview>, ApiError> {
    let stats: ClientStatsRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) AS total,
            COALESCE(SUM(CASE WHEN client_type = 'active' THEN 1 ELSE 0 END), 0) AS active,
            COALESCE(SUM(CASE WHEN client_type = 'prospect' THEN 1 ELSE 0 END), 0) AS prospect,
            COALESCE(SUM(COALESCE(annual_revenue, 0.0)), 0.0) AS total_revenue
        FROM clients
        WHERE company_id = ?
        "#,
    )
    .bind(&user.company_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let search = query.search.unwrap_or_default().trim().to_lowercase();
    let selected_client_id = query
        .selected_client_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let can_manage = current_user_can_manage(&pool, &user).await?;
    let credit_profiles = crate::app::collections::credit_profiles(&pool, &user.company_id)
        .await
        .map_err(|_| {
            api_err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "credit profile query failed",
            )
        })?;
    let clients = client_overview_rows(
        &pool,
        &user.company_id,
        &search,
        can_manage,
        &credit_profiles,
    )
    .await?;
    let detail_client = if let Some(client_id) = selected_client_id.as_deref() {
        client_detail_row(
            &pool,
            &user.company_id,
            client_id,
            can_manage,
            &credit_profiles,
        )
        .await?
    } else {
        None
    };
    let detail_contacts = if let Some(client_id) = selected_client_id.as_deref() {
        contact_detail_rows(&pool, &user.company_id, client_id).await?
    } else {
        Vec::new()
    };
    let detail_interactions = if let Some(client_id) = selected_client_id.as_deref() {
        interaction_detail_rows(&pool, &user.company_id, client_id).await?
    } else {
        Vec::new()
    };
    let empty_message = if search.is_empty() {
        "暂无客户数据"
    } else {
        "未找到匹配的客户"
    }
    .to_string();

    Ok(Json(ClientsOverview {
        stats: ClientStats {
            total: stats.total,
            active: stats.active,
            prospect: stats.prospect,
            total_revenue: stats.total_revenue,
            total_revenue_label: format_wan(stats.total_revenue),
        },
        clients,
        contacts: contact_overview_rows(&pool, &user.company_id).await?,
        interactions: interaction_overview_rows(&pool, &user.company_id).await?,
        detail_client,
        detail_contacts,
        detail_interactions,
        type_options: client_type_options(),
        source_options: client_source_options(),
        interaction_type_options: client_interaction_type_options(),
        create_action: client_create_action(can_manage),
        empty_message,
    }))
}

async fn client_overview_rows(
    pool: &SqlitePool,
    company_id: &str,
    search: &str,
    can_manage: bool,
    credit_profiles: &HashMap<String, Value>,
) -> Result<Vec<Value>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT
            id,
            name,
            short_name,
            client_type,
            industry,
            website,
            email,
            phone,
            address,
            city,
            country,
            tax_id,
            source,
            notes,
            tags,
            annual_revenue,
            employee_count,
            created_at,
            updated_at
        FROM clients
        WHERE company_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let name = text(&row, "name");
            let short_name = text(&row, "short_name");
            let email = text(&row, "email");
            if !search.is_empty() {
                let search_fields = [name.as_deref(), short_name.as_deref(), email.as_deref()];
                let matches_search = search_fields
                    .into_iter()
                    .flatten()
                    .any(|value| value.to_lowercase().contains(search));
                if !matches_search {
                    return None;
                }
            }

            let client_type =
                text(&row, "client_type").unwrap_or_else(|| "prospect".to_string());
            let annual_revenue = number(&row, "annual_revenue");
            let type_badge = client_type_badge(&client_type);
            let id = text(&row, "id");
            let delete_action = client_delete_action(id.clone(), name.clone(), can_manage);
            let contact_create_action = client_contact_create_action(id.clone(), can_manage);
            let update_action = client_update_action(id.clone(), can_manage);
            let credit_profile = id
                .as_deref()
                .and_then(|client_id| credit_profiles.get(client_id))
                .cloned()
                .unwrap_or_else(crate::app::collections::default_credit_profile);

            Some(json!({
                "id": id,
                "name": name,
                "short_name": short_name,
                "client_type": client_type,
                "type_badge": type_badge,
                "industry": text(&row, "industry"),
                "website": text(&row, "website"),
                "email": email,
                "phone": text(&row, "phone"),
                "address": text(&row, "address"),
                "city": text(&row, "city"),
                "country": text(&row, "country"),
                "tax_id": text(&row, "tax_id"),
                "source": text(&row, "source"),
                "notes": text(&row, "notes"),
                "tags": text(&row, "tags"),
                "annual_revenue": annual_revenue,
                "annual_revenue_label": if annual_revenue > 0.0 { format_cny(annual_revenue) } else { "-".to_string() },
                "employee_count": integer(&row, "employee_count"),
                "created_at": text(&row, "created_at"),
                "updated_at": text(&row, "updated_at"),
                "update_action": update_action,
                "can_delete": can_manage,
                "delete_action": delete_action,
                "contact_create_action": contact_create_action,
                "credit_profile": credit_profile,
            }))
        })
        .collect())
}

async fn client_detail_row(
    pool: &SqlitePool,
    company_id: &str,
    client_id: &str,
    can_manage: bool,
    credit_profiles: &HashMap<String, Value>,
) -> Result<Option<Value>, ApiError> {
    let row = sqlx::query(
        r#"
        SELECT
            id,
            name,
            short_name,
            client_type,
            industry,
            website,
            email,
            phone,
            address,
            city,
            country,
            tax_id,
            source,
            notes,
            tags,
            annual_revenue,
            employee_count,
            created_at,
            updated_at
        FROM clients
        WHERE company_id = ? AND id = ?
        "#,
    )
    .bind(company_id)
    .bind(client_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(row.map(|row| {
        let id = text(&row, "id");
        let name = text(&row, "name");
        let client_type = text(&row, "client_type").unwrap_or_else(|| "prospect".to_string());
        let annual_revenue = number(&row, "annual_revenue");
        let type_badge = client_type_badge(&client_type);
        let delete_action = client_delete_action(id.clone(), name.clone(), can_manage);
        let contact_create_action = client_contact_create_action(id.clone(), can_manage);
        let interaction_create_action = client_interaction_create_action(id.clone(), can_manage);
        let update_action = client_update_action(id.clone(), can_manage);
        let credit_profile = id
            .as_deref()
            .and_then(|client_id| credit_profiles.get(client_id))
            .cloned()
            .unwrap_or_else(crate::app::collections::default_credit_profile);
        json!({
            "id": id,
            "name": name,
            "short_name": text(&row, "short_name"),
            "client_type": client_type,
            "type_badge": type_badge,
            "industry": text(&row, "industry"),
            "website": text(&row, "website"),
            "email": text(&row, "email"),
            "phone": text(&row, "phone"),
            "address": text(&row, "address"),
            "city": text(&row, "city"),
            "country": text(&row, "country"),
            "tax_id": text(&row, "tax_id"),
            "source": text(&row, "source"),
            "notes": text(&row, "notes"),
            "tags": text(&row, "tags"),
            "annual_revenue": annual_revenue,
            "annual_revenue_label": if annual_revenue > 0.0 { format_cny(annual_revenue) } else { "-".to_string() },
            "employee_count": integer(&row, "employee_count"),
            "created_at": text(&row, "created_at"),
            "updated_at": text(&row, "updated_at"),
            "update_action": update_action,
            "can_delete": can_manage,
            "delete_action": delete_action,
            "contact_create_action": contact_create_action,
            "interaction_create_action": interaction_create_action,
            "credit_profile": credit_profile,
        })
    }))
}

fn client_create_action(can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }

    operations_create_action("/api/v1/clients".to_string(), "新增客户", "客户创建成功")
}

fn client_update_action(id: Option<String>, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };

    operations_update_action(
        format!("/api/v1/clients/{id}"),
        "保存修改",
        "客户信息已更新",
    )
}

fn client_delete_action(id: Option<String>, name: Option<String>, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };
    let name = name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("该客户");
    operations_delete_action(
        format!("/api/v1/clients/{id}"),
        format!("确定要删除客户「{name}」吗？"),
        "客户已删除",
    )
}

fn client_interaction_create_action(id: Option<String>, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };

    json!({
        "method": "POST",
        "endpoint": "/api/v1/interactions",
        "label": "添加互动记录",
        "success_title": "添加成功",
        "success_message": "互动记录已添加",
        "default_values": {
            "client_id": id,
        },
    })
}

fn client_contact_create_action(id: Option<String>, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };

    json!({
        "method": "POST",
        "endpoint": "/api/v1/contacts",
        "label": "添加联系人",
        "success_title": "添加成功",
        "success_message": "联系人已添加",
        "default_values": {
            "client_id": id,
        },
    })
}

async fn contact_overview_rows(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<Vec<Value>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT
            id,
            client_id,
            name,
            title,
            email,
            phone,
            mobile,
            department,
            is_primary,
            notes,
            created_at,
            updated_at
        FROM contacts
        WHERE company_id = ?
        ORDER BY is_primary DESC, created_at DESC
        "#,
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            json!({
                "id": text(&row, "id"),
                "client_id": text(&row, "client_id"),
                "name": text(&row, "name"),
                "title": text(&row, "title"),
                "email": text(&row, "email"),
                "phone": text(&row, "phone"),
                "mobile": text(&row, "mobile"),
                "department": text(&row, "department"),
                "is_primary": integer(&row, "is_primary"),
                "notes": text(&row, "notes"),
                "created_at": text(&row, "created_at"),
                "updated_at": text(&row, "updated_at"),
            })
        })
        .collect())
}

async fn contact_detail_rows(
    pool: &SqlitePool,
    company_id: &str,
    client_id: &str,
) -> Result<Vec<Value>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT
            id,
            client_id,
            name,
            title,
            email,
            phone,
            mobile,
            department,
            is_primary,
            notes,
            created_at,
            updated_at
        FROM contacts
        WHERE company_id = ? AND client_id = ?
        ORDER BY is_primary DESC, created_at DESC
        "#,
    )
    .bind(company_id)
    .bind(client_id)
    .fetch_all(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            json!({
                "id": text(&row, "id"),
                "client_id": text(&row, "client_id"),
                "name": text(&row, "name"),
                "title": text(&row, "title"),
                "email": text(&row, "email"),
                "phone": text(&row, "phone"),
                "mobile": text(&row, "mobile"),
                "department": text(&row, "department"),
                "is_primary": integer(&row, "is_primary"),
                "notes": text(&row, "notes"),
                "created_at": text(&row, "created_at"),
                "updated_at": text(&row, "updated_at"),
            })
        })
        .collect())
}

async fn interaction_overview_rows(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<Vec<Value>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT
            id,
            client_id,
            contact_id,
            interaction_type,
            subject,
            description,
            interaction_date,
            duration_minutes,
            outcome,
            next_action,
            next_action_date,
            created_at,
            updated_at
        FROM interactions
        WHERE company_id = ?
        ORDER BY interaction_date DESC, created_at DESC
        "#,
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let interaction_type =
                text(&row, "interaction_type").unwrap_or_else(|| "note".to_string());
            let interaction_badge = client_interaction_type_badge(&interaction_type);
            json!({
                "id": text(&row, "id"),
                "client_id": text(&row, "client_id"),
                "contact_id": text(&row, "contact_id"),
                "interaction_type": interaction_type,
                "interaction_badge": interaction_badge,
                "subject": text(&row, "subject"),
                "description": text(&row, "description"),
                "interaction_date": text(&row, "interaction_date"),
                "duration_minutes": integer(&row, "duration_minutes"),
                "outcome": text(&row, "outcome"),
                "next_action": text(&row, "next_action"),
                "next_action_date": text(&row, "next_action_date"),
                "created_at": text(&row, "created_at"),
                "updated_at": text(&row, "updated_at"),
            })
        })
        .collect())
}

async fn interaction_detail_rows(
    pool: &SqlitePool,
    company_id: &str,
    client_id: &str,
) -> Result<Vec<Value>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT
            id,
            client_id,
            contact_id,
            interaction_type,
            subject,
            description,
            interaction_date,
            duration_minutes,
            outcome,
            next_action,
            next_action_date,
            created_at,
            updated_at
        FROM interactions
        WHERE company_id = ? AND client_id = ?
        ORDER BY interaction_date DESC, created_at DESC
        "#,
    )
    .bind(company_id)
    .bind(client_id)
    .fetch_all(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let interaction_type =
                text(&row, "interaction_type").unwrap_or_else(|| "note".to_string());
            let interaction_badge = client_interaction_type_badge(&interaction_type);
            json!({
                "id": text(&row, "id"),
                "client_id": text(&row, "client_id"),
                "contact_id": text(&row, "contact_id"),
                "interaction_type": interaction_type,
                "interaction_badge": interaction_badge,
                "subject": text(&row, "subject"),
                "description": text(&row, "description"),
                "interaction_date": text(&row, "interaction_date"),
                "duration_minutes": integer(&row, "duration_minutes"),
                "outcome": text(&row, "outcome"),
                "next_action": text(&row, "next_action"),
                "next_action_date": text(&row, "next_action_date"),
                "created_at": text(&row, "created_at"),
                "updated_at": text(&row, "updated_at"),
            })
        })
        .collect())
}

fn client_type_badge(client_type: &str) -> Value {
    match client_type {
        "active" => json!({
            "value": "active",
            "label": "活跃客户",
            "variant": "default",
        }),
        "inactive" => json!({
            "value": "inactive",
            "label": "休眠客户",
            "variant": "secondary",
        }),
        "churned" => json!({
            "value": "churned",
            "label": "流失客户",
            "variant": "destructive",
        }),
        _ => json!({
            "value": "prospect",
            "label": "潜在客户",
            "variant": "outline",
        }),
    }
}

fn client_type_options() -> Vec<Value> {
    ["prospect", "active", "inactive", "churned"]
        .into_iter()
        .map(client_type_badge)
        .collect()
}

fn client_source_options() -> Vec<Value> {
    [
        ("referral", "转介绍"),
        ("website", "官网"),
        ("cold_call", "陌拜"),
        ("social_media", "社交媒体"),
        ("other", "其他"),
    ]
    .into_iter()
    .map(|(value, label)| {
        json!({
            "value": value,
            "label": label,
        })
    })
    .collect()
}

fn client_interaction_type_badge(interaction_type: &str) -> Value {
    match interaction_type {
        "call" => json!({
            "value": "call",
            "label": "电话",
            "icon": "Phone",
        }),
        "email" => json!({
            "value": "email",
            "label": "邮件",
            "icon": "Mail",
        }),
        "meeting" => json!({
            "value": "meeting",
            "label": "会议",
            "icon": "Users",
        }),
        _ => json!({
            "value": "note",
            "label": "备注",
            "icon": "MessageSquare",
        }),
    }
}

fn client_interaction_type_options() -> Vec<Value> {
    ["call", "email", "meeting", "note"]
        .into_iter()
        .map(client_interaction_type_badge)
        .collect()
}

fn cloud_row_string(row: &SqliteRow, column: &str) -> String {
    row.try_get::<Option<String>, _>(column)
        .ok()
        .flatten()
        .unwrap_or_default()
}

fn cloud_row_optional_string(row: &SqliteRow, column: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(column).ok().flatten()
}

fn cloud_row_f64(row: &SqliteRow, column: &str) -> f64 {
    row.try_get::<Option<f64>, _>(column)
        .ok()
        .flatten()
        .unwrap_or(0.0)
}

fn cloud_row_i64(row: &SqliteRow, column: &str) -> i64 {
    row.try_get::<Option<i64>, _>(column)
        .ok()
        .flatten()
        .unwrap_or(0)
}

fn cloud_date_label(value: Option<&str>) -> String {
    let date = value.map(str::trim).filter(|date| !date.is_empty());
    match date {
        Some(date) if date.len() >= 10 => date[..10].to_string(),
        Some(date) => date.to_string(),
        None => "-".to_string(),
    }
}

fn cloud_datetime_label(value: Option<&str>) -> String {
    let Some(raw) = value.map(str::trim).filter(|date| !date.is_empty()) else {
        return "-".to_string();
    };
    let normalized = raw.replace('T', " ").trim_end_matches('Z').to_string();
    if let Some((date_time, _)) = normalized.split_once('.') {
        date_time.to_string()
    } else {
        normalized
    }
}

fn cloud_text_label(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .unwrap_or("-")
        .to_string()
}

fn cloud_badge(label: &str, variant: &str, class_name: &str) -> Value {
    json!({
        "label": label,
        "variant": variant,
        "class_name": class_name,
    })
}

fn cloud_provider_type_label(provider_type: &str) -> String {
    match provider_type {
        "aliyun" => "阿里云",
        "aws" => "AWS",
        "tencent" => "腾讯云",
        "huawei" => "华为云",
        "azure" => "Azure",
        "other" | "" => "其他",
        value => value,
    }
    .to_string()
}

fn cloud_resource_type_label(resource_type: &str) -> String {
    match resource_type {
        "compute" | "server" => "计算",
        "storage" => "存储",
        "database" => "数据库",
        "network" => "网络",
        "cdn" => "CDN",
        "other" | "" => "其他",
        value => value,
    }
    .to_string()
}

fn cloud_resource_status_badge(status: &str) -> Value {
    match status {
        "running" | "" => cloud_badge("运行中", "default", ""),
        "stopped" => cloud_badge("已停止", "secondary", ""),
        "expired" => cloud_badge("已过期", "destructive", ""),
        "terminated" => cloud_badge("已终止", "destructive", ""),
        value => cloud_badge(value, "secondary", ""),
    }
}

fn cloud_bill_status_badge(status: &str) -> Value {
    match status {
        "paid" => cloud_badge("已支付", "default", ""),
        "overdue" => cloud_badge("已逾期", "destructive", ""),
        "pending" | "" => cloud_badge("待支付", "secondary", ""),
        value => cloud_badge(value, "secondary", ""),
    }
}

fn cloud_domain_status_badge(status: &str) -> Value {
    match status {
        "active" | "" => cloud_badge("正常", "default", ""),
        "expired" => cloud_badge("已过期", "destructive", ""),
        "pending_transfer" => cloud_badge("转移中", "secondary", ""),
        value => cloud_badge(value, "secondary", ""),
    }
}

fn cloud_software_status_badge(status: &str) -> Value {
    match status {
        "active" | "" => cloud_badge("使用中", "default", ""),
        "trial" => cloud_badge("试用期", "secondary", ""),
        "expired" => cloud_badge("已过期", "destructive", ""),
        "cancelled" => cloud_badge("已取消", "destructive", ""),
        value => cloud_badge(value, "secondary", ""),
    }
}

fn cloud_resource_can_transition(current_status: &str, target_status: &str) -> bool {
    if current_status == target_status {
        return false;
    }
    match target_status {
        "running" => current_status == "stopped",
        "stopped" => current_status == "running",
        "expired" => current_status != "terminated",
        "terminated" => current_status != "terminated",
        _ => false,
    }
}

fn domain_subscription_can_transition(current_status: &str, target_status: &str) -> bool {
    if current_status == target_status {
        return false;
    }
    match target_status {
        "active" => current_status != "expired",
        "expired" => true,
        "pending_transfer" => current_status == "active",
        _ => false,
    }
}

fn software_subscription_can_transition(current_status: &str, target_status: &str) -> bool {
    if current_status == target_status {
        return false;
    }
    match target_status {
        "active" => matches!(current_status, "trial" | "expired"),
        "trial" => current_status == "active",
        "expired" => current_status != "cancelled",
        "cancelled" => current_status != "cancelled",
        _ => false,
    }
}

fn cloud_lifecycle_action(
    endpoint: String,
    can_execute: bool,
    label: &str,
    success_message: &str,
    target_status: &str,
) -> Value {
    if !can_execute {
        return json!(null);
    }

    operations_post_action(endpoint, label, success_message, target_status)
}

fn cloud_billing_cycle_label(billing_cycle: &str) -> String {
    match billing_cycle {
        "monthly" => "每月",
        "quarterly" => "每季度",
        "annually" | "yearly" => "每年",
        "perpetual" => "永久",
        "" => "-",
        value => value,
    }
    .to_string()
}

fn cloud_auto_renewal_badge(auto_renewal: bool) -> Value {
    if auto_renewal {
        json!({
            "label": "自动续费",
            "icon": "CheckCircle2",
            "class_name": "text-green-600",
        })
    } else {
        json!({
            "label": "手动续费",
            "icon": "XCircle",
            "class_name": "text-muted-foreground",
        })
    }
}

fn cloud_expiry_state(expiry_date: Option<&str>) -> Value {
    let label = cloud_date_label(expiry_date);
    if label == "-" {
        return json!({
            "label": "-",
            "class_name": "",
        });
    }

    let parsed = NaiveDate::parse_from_str(&label, "%Y-%m-%d");
    match parsed {
        Ok(date) => {
            let days_left = (date - Utc::now().date_naive()).num_days();
            if days_left <= 0 {
                json!({
                    "label": "已过期",
                    "class_name": "text-destructive font-medium",
                })
            } else {
                json!({
                    "label": format!("{days_left}天"),
                    "class_name": if days_left < 30 { "text-destructive font-medium" } else { "" },
                })
            }
        }
        Err(_) => json!({
            "label": label,
            "class_name": "",
        }),
    }
}

async fn cloud_cost_form_options(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<Value>, ApiError> {
    let provider_options = cloud_provider_option_rows(&pool, &user.company_id).await?;

    Ok(Json(json!({
        "provider_type_options": cloud_provider_type_options(),
        "provider_options": provider_options,
        "resource_type_options": cloud_resource_type_options(),
        "purchase_type_options": cloud_purchase_type_options(),
        "bill_status_options": cloud_bill_status_options(),
        "software_subscription_type_options": cloud_software_subscription_type_options(),
        "billing_cycle_options": cloud_billing_cycle_options(),
        "bill_create_action": cloud_bill_create_action(true),
        "default_values": {
            "resource_type": "",
            "provider_type": "",
            "purchase_type": "subscription",
            "bill_status": "pending",
            "software_subscription_type": "annual",
            "billing_cycle": "annually",
            "license_count": 1,
            "auto_renewal": true
        }
    })))
}

async fn cloud_provider_option_rows(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<Vec<Value>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT id, provider_name
        FROM cloud_providers
        WHERE company_id = ?
        ORDER BY provider_name ASC, created_at DESC
        "#,
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            cloud_option(
                &cloud_row_string(&row, "id"),
                &cloud_row_string(&row, "provider_name"),
            )
        })
        .collect())
}

fn cloud_option(value: &str, label: &str) -> Value {
    json!({
        "value": value,
        "label": label,
    })
}

fn cloud_badge_option(value: &str, badge: Value) -> Value {
    json!({
        "value": value,
        "label": badge["label"].clone(),
        "variant": badge["variant"].clone(),
        "class_name": badge["class_name"].clone(),
    })
}

fn cloud_resource_type_options() -> Vec<Value> {
    ["compute", "storage", "database", "network", "cdn", "other"]
        .into_iter()
        .map(|value| cloud_option(value, &cloud_resource_type_label(value)))
        .collect()
}

fn cloud_provider_type_options() -> Vec<Value> {
    ["aliyun", "aws", "tencent", "huawei", "azure", "other"]
        .into_iter()
        .map(|value| cloud_option(value, &cloud_provider_type_label(value)))
        .collect()
}

fn cloud_purchase_type_options() -> Vec<Value> {
    [("subscription", "包年包月"), ("pay_as_you_go", "按量付费")]
        .into_iter()
        .map(|(value, label)| cloud_option(value, label))
        .collect()
}

fn cloud_bill_status_options() -> Vec<Value> {
    ["pending", "paid", "overdue"]
        .into_iter()
        .map(|value| cloud_badge_option(value, cloud_bill_status_badge(value)))
        .collect()
}

fn cloud_software_subscription_type_options() -> Vec<Value> {
    [
        ("monthly", "按月订阅"),
        ("annual", "按年订阅"),
        ("perpetual", "永久许可"),
    ]
    .into_iter()
    .map(|(value, label)| cloud_option(value, label))
    .collect()
}

fn cloud_billing_cycle_options() -> Vec<Value> {
    ["monthly", "quarterly", "annually"]
        .into_iter()
        .map(|value| cloud_option(value, &cloud_billing_cycle_label(value)))
        .collect()
}

async fn cloud_cost_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<CloudCostOverview>, ApiError> {
    let row: CloudCostOverviewRow = sqlx::query_as(
        r#"
        SELECT
            (SELECT COUNT(*) FROM cloud_providers WHERE company_id = ?) AS total_providers,
            (SELECT COUNT(*) FROM cloud_resources WHERE company_id = ?) AS total_resources,
            (SELECT COUNT(*) FROM domain_subscriptions WHERE company_id = ?) AS total_domains,
            (SELECT COUNT(*) FROM software_subscriptions WHERE company_id = ?) AS total_software,
            (
                SELECT COALESCE(SUM(COALESCE(monthly_cost, 0.0)), 0.0)
                FROM cloud_resources
                WHERE company_id = ?
            ) AS monthly_cloud_cost,
            (
                SELECT COALESCE(SUM(COALESCE(annual_fee, 0.0)), 0.0) / 12.0
                FROM domain_subscriptions
                WHERE company_id = ?
            ) AS monthly_domain_cost,
            (
                SELECT COALESCE(SUM(CASE
                    WHEN billing_cycle = 'monthly' THEN COALESCE(total_price, 0.0)
                    WHEN billing_cycle = 'quarterly' THEN COALESCE(total_price, 0.0) / 3.0
                    WHEN billing_cycle IN ('annually', 'yearly') THEN COALESCE(total_price, 0.0) / 12.0
                    ELSE 0.0
                END), 0.0)
                FROM software_subscriptions
                WHERE company_id = ?
            ) AS monthly_software_cost
        "#,
    )
    .bind(&user.company_id)
    .bind(&user.company_id)
    .bind(&user.company_id)
    .bind(&user.company_id)
    .bind(&user.company_id)
    .bind(&user.company_id)
    .bind(&user.company_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let can_manage = current_user_can_manage(&pool, &user).await?;
    let total_monthly_cost =
        row.monthly_cloud_cost + row.monthly_domain_cost + row.monthly_software_cost;

    let provider_rows = sqlx::query(
        r#"
        SELECT
            id, provider_name, provider_type, account_name, account_id,
            region, contact_person, contact_email, contact_phone, notes,
            created_at, updated_at
        FROM cloud_providers
        WHERE company_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let providers = provider_rows
        .into_iter()
        .map(|provider| {
            let provider_id = cloud_row_string(&provider, "id");
            let provider_name = cloud_row_string(&provider, "provider_name");
            let provider_type = cloud_row_string(&provider, "provider_type");
            let provider_type_label = cloud_provider_type_label(&provider_type);
            json!({
                "id": provider_id,
                "provider_name": provider_name.clone(),
                "provider_type": provider_type,
                "provider_type_label": provider_type_label,
                "account_name": cloud_row_string(&provider, "account_name"),
                "account_id": cloud_row_optional_string(&provider, "account_id"),
                "region": cloud_row_optional_string(&provider, "region"),
                "contact_person": cloud_row_optional_string(&provider, "contact_person"),
                "contact_email": cloud_row_optional_string(&provider, "contact_email"),
                "contact_phone": cloud_row_optional_string(&provider, "contact_phone"),
                "notes": cloud_row_optional_string(&provider, "notes"),
                "status": "active",
                "status_badge": cloud_badge("正常", "default", ""),
                "can_edit": can_manage,
                "can_delete": can_manage,
                "delete_action": cloud_provider_delete_action(&provider_id, &provider_name, can_manage),
                "created_at": cloud_row_optional_string(&provider, "created_at"),
                "updated_at": cloud_row_optional_string(&provider, "updated_at"),
            })
        })
        .collect::<Vec<_>>();
    let provider_options = providers
        .iter()
        .map(|provider| {
            json!({
                "value": provider["id"].clone(),
                "label": provider["provider_name"].clone(),
            })
        })
        .collect::<Vec<_>>();

    let resource_rows = sqlx::query(
        r#"
        SELECT
            cr.id, cr.provider_id, cr.resource_type, cr.resource_name,
            cr.instance_id, cr.specification, cr.region, cr.purchase_type,
            cr.start_date, cr.end_date, cr.monthly_cost, cr.status, cr.notes,
            cr.created_at, cr.updated_at, cp.provider_name
        FROM cloud_resources cr
        LEFT JOIN cloud_providers cp
            ON cp.id = cr.provider_id AND cp.company_id = cr.company_id
        WHERE cr.company_id = ?
        ORDER BY cr.created_at DESC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let resources = resource_rows
        .into_iter()
        .map(|resource| {
            let resource_id = cloud_row_string(&resource, "id");
            let resource_name = cloud_row_string(&resource, "resource_name");
            let resource_type = cloud_row_string(&resource, "resource_type");
            let resource_type_label = cloud_resource_type_label(&resource_type);
            let status = cloud_row_string(&resource, "status");
            let status_badge = cloud_resource_status_badge(&status);
            let monthly_cost = cloud_row_f64(&resource, "monthly_cost");
            let start_date = cloud_row_optional_string(&resource, "start_date");
            let end_date = cloud_row_optional_string(&resource, "end_date");
            json!({
                "id": resource_id,
                "provider_id": cloud_row_optional_string(&resource, "provider_id"),
                "provider_name": cloud_text_label(cloud_row_optional_string(&resource, "provider_name").as_deref()),
                "resource_type": resource_type,
                "resource_type_label": resource_type_label,
                "resource_name": resource_name.clone(),
                "instance_id": cloud_row_optional_string(&resource, "instance_id"),
                "specification": cloud_row_optional_string(&resource, "specification"),
                "region": cloud_row_optional_string(&resource, "region"),
                "purchase_type": cloud_row_optional_string(&resource, "purchase_type"),
                "start_date": start_date,
                "start_date_label": cloud_date_label(start_date.as_deref()),
                "end_date": end_date,
                "end_date_label": cloud_date_label(end_date.as_deref()),
                "monthly_cost": monthly_cost,
                "monthly_cost_label": format_cny(monthly_cost),
                "status": status,
                "status_badge": status_badge,
                "notes": cloud_row_optional_string(&resource, "notes"),
                "can_edit": can_manage,
                "can_delete": can_manage,
                "can_start": can_manage && cloud_resource_can_transition(&status, "running"),
                "can_stop": can_manage && cloud_resource_can_transition(&status, "stopped"),
                "can_mark_expired": can_manage && cloud_resource_can_transition(&status, "expired"),
                "can_terminate": can_manage && cloud_resource_can_transition(&status, "terminated"),
                "start_action": cloud_lifecycle_action(
                    format!("/api/v1/cloud_resources/{resource_id}/start"),
                    can_manage && cloud_resource_can_transition(&status, "running"),
                    "启动",
                    "云资源已启动",
                    "running",
                ),
                "stop_action": cloud_lifecycle_action(
                    format!("/api/v1/cloud_resources/{resource_id}/stop"),
                    can_manage && cloud_resource_can_transition(&status, "stopped"),
                    "停止",
                    "云资源已停止",
                    "stopped",
                ),
                "mark_expired_action": cloud_lifecycle_action(
                    format!("/api/v1/cloud_resources/{resource_id}/mark_expired"),
                    can_manage && cloud_resource_can_transition(&status, "expired"),
                    "标记过期",
                    "云资源已标记为过期",
                    "expired",
                ),
                "terminate_action": cloud_lifecycle_action(
                    format!("/api/v1/cloud_resources/{resource_id}/terminate"),
                    can_manage && cloud_resource_can_transition(&status, "terminated"),
                    "终止",
                    "云资源已终止",
                    "terminated",
                ),
                "delete_action": cloud_resource_delete_action(&resource_id, &resource_name, can_manage),
                "created_at": cloud_row_optional_string(&resource, "created_at"),
                "updated_at": cloud_row_optional_string(&resource, "updated_at"),
            })
        })
        .collect::<Vec<_>>();

    let bill_rows = sqlx::query(
        r#"
        SELECT
            cb.id, cb.provider_id, cb.billing_period, cb.total_amount,
            cb.currency, cb.status, cb.notes, cb.created_at, cb.updated_at,
            cp.provider_name
        FROM cloud_bills cb
        LEFT JOIN cloud_providers cp
            ON cp.id = cb.provider_id AND cp.company_id = cb.company_id
        WHERE cb.company_id = ?
        ORDER BY COALESCE(cb.billing_period, cb.created_at) DESC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let bills = bill_rows
        .into_iter()
        .map(|bill| {
            let bill_id = cloud_row_string(&bill, "id");
            let billing_period = cloud_row_string(&bill, "billing_period");
            let billing_period_label = cloud_date_label(Some(&billing_period));
            let bill_month = billing_period.clone();
            let provider_name = cloud_row_optional_string(&bill, "provider_name")
                .filter(|name| !name.trim().is_empty())
                .unwrap_or_else(|| "云服务账单".to_string());
            let total_amount = cloud_row_f64(&bill, "total_amount");
            let status = cloud_row_string(&bill, "status");
            let payment_status = status.clone();
            let status_badge = cloud_bill_status_badge(&status);
            let can_mark_paid = can_manage && matches!(status.as_str(), "pending" | "overdue");
            let bill_date = cloud_row_optional_string(&bill, "created_at");
            let bill_date_label = cloud_date_label(bill_date.as_deref());
            let currency =
                cloud_row_optional_string(&bill, "currency").unwrap_or_else(|| "CNY".to_string());
            json!({
                "id": bill_id,
                "provider_id": cloud_row_optional_string(&bill, "provider_id"),
                "provider_name": provider_name,
                "billing_period": billing_period,
                "billing_period_label": billing_period_label,
                "bill_month": bill_month,
                "item_name": "云服务账单",
                "bill_date": bill_date,
                "bill_date_label": bill_date_label,
                "total_amount": total_amount,
                "total_amount_label": format_cny(total_amount),
                "discount_amount": 0.0,
                "discount_amount_label": format_cny(0.0),
                "actual_amount": total_amount,
                "actual_amount_label": format_cny(total_amount),
                "currency": currency,
                "status": status,
                "payment_status": payment_status,
                "status_badge": status_badge,
                "notes": cloud_row_optional_string(&bill, "notes"),
                "can_mark_paid": can_mark_paid,
                "mark_paid_action": cloud_lifecycle_action(
                    format!("/api/v1/cloud_bills/{bill_id}/mark_paid"),
                    can_mark_paid,
                    "标记已支付",
                    "云账单已标记为已支付",
                    "paid",
                ),
                "can_edit": can_manage,
                "update_action": cloud_bill_update_action(&bill_id, can_manage),
                "can_delete": can_manage,
                "delete_action": cloud_bill_delete_action(&bill_id, &provider_name, &billing_period_label, can_manage),
                "created_at": cloud_row_optional_string(&bill, "created_at"),
                "updated_at": cloud_row_optional_string(&bill, "updated_at"),
            })
        })
        .collect::<Vec<_>>();

    let domain_rows = sqlx::query(
        r#"
        SELECT
            id, domain_name, registrar, registration_date, expiry_date,
            initial_purchase_cost, annual_fee, auto_renewal, dns_provider,
            status, contact_email, notes, created_at, updated_at
        FROM domain_subscriptions
        WHERE company_id = ?
        ORDER BY expiry_date ASC, created_at DESC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let domains = domain_rows
        .into_iter()
        .map(|domain| {
            let domain_id = cloud_row_string(&domain, "id");
            let domain_name = cloud_row_string(&domain, "domain_name");
            let registration_date = cloud_row_optional_string(&domain, "registration_date");
            let expiry_date = cloud_row_optional_string(&domain, "expiry_date");
            let registration_date_label = cloud_date_label(registration_date.as_deref());
            let expiry_date_label = cloud_date_label(expiry_date.as_deref());
            let expiry_state = cloud_expiry_state(expiry_date.as_deref());
            let initial_purchase_cost = cloud_row_f64(&domain, "initial_purchase_cost");
            let annual_fee = cloud_row_f64(&domain, "annual_fee");
            let auto_renewal = cloud_row_i64(&domain, "auto_renewal") != 0;
            let auto_renewal_badge = cloud_auto_renewal_badge(auto_renewal);
            let status = cloud_row_string(&domain, "status");
            let status_badge = cloud_domain_status_badge(&status);
            json!({
                "id": domain_id,
                "domain_name": domain_name.clone(),
                "registrar": cloud_row_string(&domain, "registrar"),
                "registration_date": registration_date,
                "registration_date_label": registration_date_label,
                "expiry_date": expiry_date,
                "expiry_date_label": expiry_date_label,
                "expiry_state": expiry_state,
                "initial_purchase_cost": initial_purchase_cost,
                "initial_purchase_cost_label": format_cny(initial_purchase_cost),
                "annual_fee": annual_fee,
                "annual_fee_label": format_cny(annual_fee),
                "auto_renewal": auto_renewal,
                "auto_renewal_badge": auto_renewal_badge,
                "dns_provider": cloud_row_optional_string(&domain, "dns_provider"),
                "status": status,
                "status_badge": status_badge,
                "contact_email": cloud_row_optional_string(&domain, "contact_email"),
                "notes": cloud_row_optional_string(&domain, "notes"),
                "can_edit": can_manage,
                "can_delete": can_manage,
                "can_activate": can_manage && domain_subscription_can_transition(&status, "active"),
                "can_mark_expired": can_manage && domain_subscription_can_transition(&status, "expired"),
                "can_mark_pending_transfer": can_manage && domain_subscription_can_transition(&status, "pending_transfer"),
                "activate_action": cloud_lifecycle_action(
                    format!("/api/v1/domain_subscriptions/{domain_id}/activate"),
                    can_manage && domain_subscription_can_transition(&status, "active"),
                    "激活",
                    "域名已激活",
                    "active",
                ),
                "mark_expired_action": cloud_lifecycle_action(
                    format!("/api/v1/domain_subscriptions/{domain_id}/mark_expired"),
                    can_manage && domain_subscription_can_transition(&status, "expired"),
                    "标记过期",
                    "域名已标记为过期",
                    "expired",
                ),
                "mark_pending_transfer_action": cloud_lifecycle_action(
                    format!("/api/v1/domain_subscriptions/{domain_id}/mark_pending_transfer"),
                    can_manage && domain_subscription_can_transition(&status, "pending_transfer"),
                    "标记转移中",
                    "域名已标记为转移中",
                    "pending_transfer",
                ),
                "delete_action": cloud_domain_delete_action(&domain_id, &domain_name, can_manage),
                "created_at": cloud_row_optional_string(&domain, "created_at"),
                "updated_at": cloud_row_optional_string(&domain, "updated_at"),
            })
        })
        .collect::<Vec<_>>();

    let software_rows = sqlx::query(
        r#"
        SELECT
            id, software_name, vendor, subscription_type, plan_name,
            license_count, total_price, billing_cycle, start_date,
            next_billing_date, auto_renewal, status, usage_department,
            account_email, notes, created_at, updated_at
        FROM software_subscriptions
        WHERE company_id = ?
        ORDER BY next_billing_date ASC, created_at DESC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let software = software_rows
        .into_iter()
        .map(|subscription| {
            let software_id = cloud_row_string(&subscription, "id");
            let software_name = cloud_row_string(&subscription, "software_name");
            let billing_cycle = cloud_row_string(&subscription, "billing_cycle");
            let billing_cycle_label = cloud_billing_cycle_label(&billing_cycle);
            let start_date = cloud_row_optional_string(&subscription, "start_date");
            let next_billing_date = cloud_row_optional_string(&subscription, "next_billing_date");
            let start_date_label = cloud_date_label(start_date.as_deref());
            let next_billing_date_label = cloud_date_label(next_billing_date.as_deref());
            let total_price = cloud_row_f64(&subscription, "total_price");
            let auto_renewal = cloud_row_i64(&subscription, "auto_renewal") != 0;
            let auto_renewal_badge = cloud_auto_renewal_badge(auto_renewal);
            let status = cloud_row_string(&subscription, "status");
            let status_badge = cloud_software_status_badge(&status);
            json!({
                "id": software_id,
                "software_name": software_name.clone(),
                "vendor": cloud_row_string(&subscription, "vendor"),
                "subscription_type": cloud_row_string(&subscription, "subscription_type"),
                "plan_name": cloud_row_optional_string(&subscription, "plan_name"),
                "license_count": cloud_row_i64(&subscription, "license_count"),
                "total_price": total_price,
                "total_price_label": format_cny(total_price),
                "billing_cycle": billing_cycle,
                "billing_cycle_label": billing_cycle_label,
                "start_date": start_date,
                "start_date_label": start_date_label,
                "next_billing_date": next_billing_date,
                "next_billing_date_label": next_billing_date_label,
                "auto_renewal": auto_renewal,
                "auto_renewal_badge": auto_renewal_badge,
                "status": status,
                "status_badge": status_badge,
                "usage_department": cloud_row_optional_string(&subscription, "usage_department"),
                "account_email": cloud_row_optional_string(&subscription, "account_email"),
                "notes": cloud_row_optional_string(&subscription, "notes"),
                "can_edit": can_manage,
                "can_delete": can_manage,
                "can_activate": can_manage && software_subscription_can_transition(&status, "active"),
                "can_start_trial": can_manage && software_subscription_can_transition(&status, "trial"),
                "can_mark_expired": can_manage && software_subscription_can_transition(&status, "expired"),
                "can_cancel": can_manage && software_subscription_can_transition(&status, "cancelled"),
                "activate_action": cloud_lifecycle_action(
                    format!("/api/v1/software_subscriptions/{software_id}/activate"),
                    can_manage && software_subscription_can_transition(&status, "active"),
                    "激活",
                    "软件订阅已激活",
                    "active",
                ),
                "start_trial_action": cloud_lifecycle_action(
                    format!("/api/v1/software_subscriptions/{software_id}/start_trial"),
                    can_manage && software_subscription_can_transition(&status, "trial"),
                    "开始试用",
                    "软件订阅已进入试用期",
                    "trial",
                ),
                "mark_expired_action": cloud_lifecycle_action(
                    format!("/api/v1/software_subscriptions/{software_id}/mark_expired"),
                    can_manage && software_subscription_can_transition(&status, "expired"),
                    "标记过期",
                    "软件订阅已标记为过期",
                    "expired",
                ),
                "cancel_action": cloud_lifecycle_action(
                    format!("/api/v1/software_subscriptions/{software_id}/cancel"),
                    can_manage && software_subscription_can_transition(&status, "cancelled"),
                    "取消",
                    "软件订阅已取消",
                    "cancelled",
                ),
                "delete_action": cloud_software_delete_action(&software_id, &software_name, can_manage),
                "created_at": cloud_row_optional_string(&subscription, "created_at"),
                "updated_at": cloud_row_optional_string(&subscription, "updated_at"),
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(CloudCostOverview {
        total_providers: row.total_providers,
        total_resources: row.total_resources,
        total_domains: row.total_domains,
        total_software: row.total_software,
        monthly_cloud_cost: row.monthly_cloud_cost,
        monthly_domain_cost: row.monthly_domain_cost,
        monthly_software_cost: row.monthly_software_cost,
        total_monthly_cost,
        monthly_cloud_cost_label: format_cny(row.monthly_cloud_cost),
        monthly_domain_cost_label: format_cny(row.monthly_domain_cost),
        monthly_software_cost_label: format_cny(row.monthly_software_cost),
        total_monthly_cost_label: format_cny(total_monthly_cost),
        providers,
        provider_options,
        resources,
        bills,
        domains,
        software,
        permissions: json!({
            "can_manage": can_manage,
        }),
        empty_messages: json!({
            "providers": "还没有添加云服务商账户",
            "resources": "还没有添加云资源",
            "bills": "还没有账单记录",
            "domains": "还没有添加域名",
            "software": "还没有添加软件订阅",
        }),
        bill_create_action: cloud_bill_create_action(can_manage),
    }))
}

fn cloud_provider_delete_action(id: &str, provider_name: &str, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let provider_name = cloud_action_label(provider_name, "该云服务商");
    operations_delete_action(
        format!("/api/v1/cloud_providers/{id}"),
        format!(
            "确定要删除云服务商「{provider_name}」吗？相关云资源或账单存在时删除将被后端阻止。"
        ),
        "云服务商已删除",
    )
}

fn cloud_resource_delete_action(id: &str, resource_name: &str, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let resource_name = cloud_action_label(resource_name, "该云资源");
    operations_delete_action(
        format!("/api/v1/cloud_resources/{id}"),
        format!("确定要删除云资源「{resource_name}」吗？此操作无法撤销。"),
        "云资源已删除",
    )
}

fn cloud_bill_delete_action(
    id: &str,
    provider_name: &str,
    billing_period_label: &str,
    can_manage: bool,
) -> Value {
    if !can_manage {
        return json!(null);
    }
    let provider_name = cloud_action_label(provider_name, "云服务账单");
    let billing_period_label = billing_period_label.trim();
    let target = if billing_period_label.is_empty() || billing_period_label == "-" {
        provider_name
    } else {
        format!("{provider_name} {billing_period_label}")
    };
    operations_delete_action(
        format!("/api/v1/cloud_bills/{id}"),
        format!("确定要删除云账单「{target}」吗？此操作无法撤销。"),
        "云账单已删除",
    )
}

fn cloud_bill_create_action(can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }

    operations_create_action(
        "/api/v1/cloud_bills".to_string(),
        "录入账单",
        "云账单已添加",
    )
}

fn cloud_bill_update_action(id: &str, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }

    operations_update_action(
        format!("/api/v1/cloud_bills/{id}"),
        "保存账单",
        "云账单已更新",
    )
}

fn cloud_domain_delete_action(id: &str, domain_name: &str, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let domain_name = cloud_action_label(domain_name, "该域名");
    operations_delete_action(
        format!("/api/v1/domain_subscriptions/{id}"),
        format!("确定要删除域名「{domain_name}」吗？此操作无法撤销。"),
        "域名记录已删除",
    )
}

fn cloud_software_delete_action(id: &str, software_name: &str, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let software_name = cloud_action_label(software_name, "该软件订阅");
    operations_delete_action(
        format!("/api/v1/software_subscriptions/{id}"),
        format!("确定要删除软件订阅「{software_name}」吗？此操作无法撤销。"),
        "软件订阅已删除",
    )
}

fn cloud_action_label(value: &str, fallback: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

async fn recurring_invoices_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<RecurringInvoicesOverview>, ApiError> {
    let stats: RecurringInvoiceStatsRow = sqlx::query_as(
        r#"
        SELECT
            (
                SELECT COUNT(*)
                FROM recurring_invoices
                WHERE company_id = ? AND status = 'active'
            ) AS active_recurring,
            (
                SELECT COALESCE(SUM(CASE
                    WHEN frequency = 'monthly' THEN COALESCE(amount, 0.0)
                    WHEN frequency = 'quarterly' THEN COALESCE(amount, 0.0) / 3.0
                    WHEN frequency = 'yearly' THEN COALESCE(amount, 0.0) / 12.0
                    WHEN frequency = 'weekly' THEN COALESCE(amount, 0.0) * 4.33
                    ELSE 0.0
                END), 0.0)
                FROM recurring_invoices
                WHERE company_id = ? AND status = 'active'
            ) AS total_monthly,
            (
                SELECT COUNT(*)
                FROM invoice_reminders
                WHERE company_id = ? AND status = 'pending'
            ) AS pending_reminders,
            (
                SELECT COUNT(*)
                FROM invoice_reminders
                WHERE company_id = ? AND status = 'sent'
            ) AS sent_reminders
        "#,
    )
    .bind(&user.company_id)
    .bind(&user.company_id)
    .bind(&user.company_id)
    .bind(&user.company_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let can_manage = current_user_can_manage(&pool, &user).await?;

    Ok(Json(RecurringInvoicesOverview {
        stats: RecurringInvoiceStats {
            active_recurring: stats.active_recurring,
            total_monthly: stats.total_monthly,
            total_monthly_label: format_cny(stats.total_monthly),
            pending_reminders: stats.pending_reminders,
            sent_reminders: stats.sent_reminders,
        },
        recurring: recurring_invoice_rows(&pool, &user.company_id, can_manage).await?,
        reminders: invoice_reminder_rows(&pool, &user.company_id, can_manage).await?,
        clients: recurring_client_rows(&pool, &user.company_id).await?,
        invoices: recurring_invoice_option_rows(&pool, &user.company_id).await?,
        permissions: json!({
            "can_manage": can_manage,
        }),
        create_action: recurring_invoice_create_action(can_manage),
        reminder_create_action: invoice_reminder_create_action(can_manage),
        empty_messages: json!({
            "recurring": "暂无周期发票",
            "reminders": "暂无催款提醒",
        }),
    }))
}

async fn recurring_invoices_form_options(_user: AuthUser) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({
        "frequency_options": recurring_invoice_frequency_options(),
        "reminder_type_options": invoice_reminder_type_options(),
        "default_values": {
            "frequency": "monthly",
            "payment_terms_days": 30,
            "tax_rate": 0,
            "auto_send": 0,
            "reminder_type": "before_due",
            "channel": "email"
        }
    })))
}

fn recurring_invoice_frequency_label(frequency: &str) -> String {
    match frequency {
        "weekly" => "每周",
        "monthly" => "每月",
        "quarterly" => "每季度",
        "yearly" | "annually" => "每年",
        "" => "-",
        value => value,
    }
    .to_string()
}

fn recurring_invoice_frequency_options() -> Vec<Value> {
    ["weekly", "monthly", "quarterly", "yearly"]
        .into_iter()
        .map(|value| {
            json!({
                "value": value,
                "label": recurring_invoice_frequency_label(value),
            })
        })
        .collect()
}

fn recurring_invoice_status_badge(status: &str) -> Value {
    match status {
        "active" | "" => cloud_badge("运行中", "default", ""),
        "paused" => cloud_badge("已暂停", "secondary", ""),
        "completed" => cloud_badge("已完成", "outline", ""),
        value => cloud_badge(value, "outline", ""),
    }
}

fn recurring_invoice_toggle_action(id: Option<String>, status: &str, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };
    let (next_status, label, icon, success_message) = if status == "active" {
        ("paused", "暂停", "Pause", "周期发票已暂停")
    } else {
        ("active", "启用", "Play", "周期发票已启用")
    };
    let mut action = operations_update_action(
        format!("/api/v1/recurring_invoices/{id}"),
        label,
        success_message,
    );
    if let Value::Object(ref mut object) = action {
        object.insert("next_status".to_string(), json!(next_status));
        object.insert("icon".to_string(), json!(icon));
        object.insert(
            "default_values".to_string(),
            json!({ "status": next_status }),
        );
    };
    action
}

fn invoice_reminder_type_label(reminder_type: &str) -> String {
    match reminder_type {
        "before_due" => "到期前提醒",
        "on_due" => "到期日提醒",
        "overdue_1" => "逾期1天",
        "overdue_7" => "逾期7天",
        "overdue_30" => "逾期30天",
        "" => "-",
        value => value,
    }
    .to_string()
}

fn invoice_reminder_type_options() -> Vec<Value> {
    [
        "before_due",
        "on_due",
        "overdue_1",
        "overdue_7",
        "overdue_30",
    ]
    .into_iter()
    .map(|value| {
        json!({
            "value": value,
            "label": invoice_reminder_type_label(value),
        })
    })
    .collect()
}

fn invoice_reminder_status_badge(status: &str) -> Value {
    match status {
        "pending" | "" => cloud_badge("待发送", "outline", ""),
        "sent" => cloud_badge("已发送", "default", ""),
        "skipped" => cloud_badge("已跳过", "secondary", ""),
        "failed" => cloud_badge("失败", "destructive", ""),
        value => cloud_badge(value, "outline", ""),
    }
}

fn invoice_reminder_channel_label(channel: &str) -> String {
    match channel {
        "email" | "" => "邮件",
        "sms" => "短信",
        "in_app" => "站内",
        value => value,
    }
    .to_string()
}

fn short_identifier(value: Option<&str>) -> String {
    value
        .map(|text| text.chars().take(8).collect::<String>())
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| "-".to_string())
}

fn operations_delete_action(
    endpoint: String,
    confirm_message: impl Into<String>,
    success_message: impl Into<String>,
) -> Value {
    json!({
        "method": "DELETE",
        "endpoint": endpoint,
        "confirm_message": confirm_message.into(),
        "success_title": "删除成功",
        "success_message": success_message.into()
    })
}

fn operations_write_action(
    method: &'static str,
    endpoint: String,
    label: impl Into<String>,
    success_title: impl Into<String>,
    success_message: impl Into<String>,
) -> Value {
    json!({
        "method": method,
        "endpoint": endpoint,
        "label": label.into(),
        "success_title": success_title.into(),
        "success_message": success_message.into(),
    })
}

fn operations_create_action(
    endpoint: String,
    label: impl Into<String>,
    success_message: impl Into<String>,
) -> Value {
    operations_write_action("POST", endpoint, label, "创建成功", success_message)
}

fn operations_update_action(
    endpoint: String,
    label: impl Into<String>,
    success_message: impl Into<String>,
) -> Value {
    operations_write_action("PUT", endpoint, label, "更新成功", success_message)
}

fn operations_post_action(
    endpoint: String,
    label: impl Into<String>,
    success_message: impl Into<String>,
    target_status: impl Into<String>,
) -> Value {
    json!({
        "method": "POST",
        "endpoint": endpoint,
        "label": label.into(),
        "success_title": "状态已更新",
        "success_message": success_message.into(),
        "target_status": target_status.into(),
    })
}

fn recurring_invoice_create_action(can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    operations_create_action(
        "/api/v1/recurring_invoices".to_string(),
        "新建周期发票",
        "周期发票已创建",
    )
}

fn recurring_invoice_update_action(id: Option<String>, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };
    operations_update_action(
        format!("/api/v1/recurring_invoices/{id}"),
        "保存修改",
        "周期发票已更新",
    )
}

fn invoice_reminder_create_action(can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    operations_create_action(
        "/api/v1/invoice_reminders".to_string(),
        "新建催款提醒",
        "催款提醒已创建",
    )
}

fn recurring_invoice_delete_action(
    id: Option<String>,
    title: Option<String>,
    can_manage: bool,
) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };
    let title = title
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("该周期发票");
    operations_delete_action(
        format!("/api/v1/recurring_invoices/{id}"),
        format!("确定要删除周期发票「{title}」吗？"),
        "周期发票已删除",
    )
}

fn invoice_reminder_delete_action(
    id: Option<String>,
    invoice_display: Option<String>,
    can_manage: bool,
) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };
    let invoice_display = invoice_display
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("该提醒");
    operations_delete_action(
        format!("/api/v1/invoice_reminders/{id}"),
        format!("确定要删除催款提醒「{invoice_display}」吗？"),
        "催款提醒已删除",
    )
}

async fn recurring_invoice_rows(
    pool: &SqlitePool,
    company_id: &str,
    can_manage: bool,
) -> Result<Vec<Value>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT
            ri.*,
            c.name AS client_name
        FROM recurring_invoices ri
        LEFT JOIN clients c ON c.id = ri.client_id AND c.company_id = ri.company_id
        WHERE ri.company_id = ?
        ORDER BY COALESCE(ri.next_invoice_date, ri.created_at) ASC, ri.created_at DESC
        "#,
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let id = text(&row, "id");
            let title = text(&row, "title");
            let amount = number(&row, "amount");
            let frequency = text(&row, "frequency").unwrap_or_default();
            let frequency_label = recurring_invoice_frequency_label(&frequency);
            let next_invoice_date = text(&row, "next_invoice_date");
            let next_invoice_date_label = cloud_date_label(next_invoice_date.as_deref());
            let status = text(&row, "status").unwrap_or_else(|| "active".to_string());
            let status_badge = recurring_invoice_status_badge(&status);
            let toggle_action = recurring_invoice_toggle_action(id.clone(), &status, can_manage);
            let update_action = recurring_invoice_update_action(id.clone(), can_manage);
            let total_generated = integer(&row, "total_generated");
            let last_generated_at = text(&row, "last_generated_at");
            let last_generated_at_label = cloud_date_label(last_generated_at.as_deref());
            let delete_action =
                recurring_invoice_delete_action(id.clone(), title.clone(), can_manage);
            json!({
                "id": id,
                "client_id": text(&row, "client_id"),
                "client_name": text(&row, "client_name"),
                "title": title,
                "description": text(&row, "description"),
                "amount": amount,
                "amount_label": format_cny(amount),
                "currency": text(&row, "currency").unwrap_or_else(|| "CNY".to_string()),
                "tax_rate": number(&row, "tax_rate"),
                "frequency": frequency,
                "frequency_label": frequency_label,
                "start_date": text(&row, "start_date"),
                "end_date": text(&row, "end_date"),
                "next_invoice_date": next_invoice_date,
                "next_invoice_date_label": next_invoice_date_label,
                "day_of_month": integer(&row, "day_of_month"),
                "auto_send": integer(&row, "auto_send"),
                "payment_terms_days": integer(&row, "payment_terms_days"),
                "status": status,
                "status_badge": status_badge,
                "toggle_action": toggle_action,
                "total_generated": total_generated,
                "total_generated_label": format!("{total_generated} 次"),
                "last_generated_at": last_generated_at,
                "last_generated_at_label": last_generated_at_label,
                "notes": text(&row, "notes"),
                "can_edit": can_manage,
                "update_action": update_action,
                "can_delete": can_manage,
                "delete_action": delete_action,
            })
        })
        .collect())
}

async fn invoice_reminder_rows(
    pool: &SqlitePool,
    company_id: &str,
    can_manage: bool,
) -> Result<Vec<Value>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT
            ir.*,
            i.invoice_number
        FROM invoice_reminders ir
        LEFT JOIN invoices i ON i.id = ir.invoice_id AND i.company_id = ir.company_id
        WHERE ir.company_id = ?
        ORDER BY COALESCE(ir.scheduled_date, ir.created_at) ASC, ir.created_at DESC
        "#,
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let id = text(&row, "id");
            let invoice_id = text(&row, "invoice_id");
            let invoice_number = text(&row, "invoice_number");
            let reminder_type = text(&row, "reminder_type").unwrap_or_default();
            let reminder_type_label = invoice_reminder_type_label(&reminder_type);
            let scheduled_date = text(&row, "scheduled_date");
            let scheduled_date_label = cloud_date_label(scheduled_date.as_deref());
            let sent_at = text(&row, "sent_at");
            let sent_at_label = cloud_date_label(sent_at.as_deref());
            let status = text(&row, "status").unwrap_or_else(|| "pending".to_string());
            let status_badge = invoice_reminder_status_badge(&status);
            let channel = text(&row, "channel").unwrap_or_else(|| "email".to_string());
            let channel_label = invoice_reminder_channel_label(&channel);
            let invoice_display = invoice_number
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| short_identifier(invoice_id.as_deref()));
            let delete_action = invoice_reminder_delete_action(
                id.clone(),
                Some(invoice_display.clone()),
                can_manage,
            );
            json!({
                "id": id,
                "invoice_id": invoice_id,
                "invoice_number": invoice_number,
                "invoice_display": invoice_display,
                "client_id": text(&row, "client_id"),
                "reminder_type": reminder_type,
                "reminder_type_label": reminder_type_label,
                "scheduled_date": scheduled_date,
                "scheduled_date_label": scheduled_date_label,
                "sent_at": sent_at,
                "sent_at_label": sent_at_label,
                "status": status,
                "status_badge": status_badge,
                "subject": text(&row, "subject"),
                "message": text(&row, "message"),
                "channel": channel,
                "channel_label": channel_label,
                "can_delete": can_manage,
                "delete_action": delete_action,
            })
        })
        .collect())
}

async fn recurring_client_rows(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<Vec<Value>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT id, name
        FROM clients
        WHERE company_id = ?
        ORDER BY name ASC
        "#,
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            json!({
                "id": text(&row, "id"),
                "name": text(&row, "name"),
            })
        })
        .collect())
}

async fn recurring_invoice_option_rows(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<Vec<Value>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT id, invoice_number, amount, status, due_date
        FROM invoices
        WHERE company_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            json!({
                "id": text(&row, "id"),
                "invoice_number": text(&row, "invoice_number"),
                "amount": number(&row, "amount"),
                "status": text(&row, "status"),
                "due_date": text(&row, "due_date"),
            })
        })
        .collect())
}

async fn projects_form_options(_user: AuthUser) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({
        "project_type_options": project_type_options(),
        "default_values": {
            "project_type": "",
            "team_members": 0
        }
    })))
}

fn project_type_options() -> Vec<Value> {
    [
        ("tech", "技术研发"),
        ("product", "产品开发"),
        ("research", "基础研究"),
        ("upgrade", "产品升级"),
        ("innovation", "创新研究"),
    ]
    .into_iter()
    .map(|(value, label)| {
        json!({
            "value": value,
            "label": label,
        })
    })
    .collect()
}

async fn projects_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(query): Query<ProjectsOverviewQuery>,
) -> Result<Json<ProjectsOverview>, ApiError> {
    let can_manage = current_user_can_manage(&pool, &user).await?;
    let project_rows = sqlx::query(
        r#"
        SELECT
            id,
            name,
            description,
            project_type,
            leader_name,
            status,
            priority,
            budget,
            spent,
            start_date,
            end_date,
            client,
            progress,
            team_members,
            goals,
            notes,
            milestones,
            created_at,
            updated_at
        FROM projects
        WHERE company_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let cost_rows = sqlx::query(
        r#"
        SELECT
            id,
            project_id,
            cost_type,
            amount,
            description,
            cost_date,
            employee_id,
            asset_id,
            invoice_id,
            created_at,
            updated_at
        FROM rd_costs
        WHERE company_id = ?
        ORDER BY cost_date DESC, created_at DESC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let mut rd_costs_by_project: HashMap<String, ProjectRdCostGroup> = HashMap::new();
    let mut total_rd_cost = 0.0;

    for row in cost_rows {
        let id = text(&row, "id");
        let project_id = text(&row, "project_id").unwrap_or_default();
        let cost_type = text(&row, "cost_type").unwrap_or_else(|| "other".to_string());
        let amount = number(&row, "amount");
        total_rd_cost += amount;

        let group = rd_costs_by_project.entry(project_id).or_default();
        group.total += amount;
        *group.by_type.entry(cost_type.clone()).or_insert(0.0) += amount;
        let cost_type_label = rd_cost_type_label(&cost_type);
        let cost_date = text(&row, "cost_date");
        let delete_action = rd_cost_delete_action(
            id.clone(),
            cost_type_label,
            cost_date.as_deref(),
            can_manage,
        );
        group.items.push(json!({
            "id": id,
            "project_id": text(&row, "project_id"),
            "cost_type": cost_type,
            "cost_type_label": cost_type_label,
            "cost_type_badge": rd_cost_type_badge(cost_type_label),
            "amount": round_money(amount),
            "amount_label": format_cny(amount),
            "description": text(&row, "description"),
            "cost_date": cost_date,
            "employee_id": text(&row, "employee_id"),
            "asset_id": text(&row, "asset_id"),
            "invoice_id": text(&row, "invoice_id"),
            "created_at": text(&row, "created_at"),
            "updated_at": text(&row, "updated_at"),
            "can_edit": can_manage,
            "update_action": rd_cost_update_action(id.clone(), can_manage),
            "can_delete": can_manage,
            "delete_action": delete_action,
        }));
    }

    let search = query
        .search
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .to_lowercase();
    let status = query.status.as_deref().map(str::trim).unwrap_or("all");

    let mut total_projects = 0_i64;
    let mut ongoing_count = 0_i64;
    let mut total_budget = 0.0;
    let mut total_members = 0_i64;
    let mut projects = Vec::new();

    for row in project_rows {
        total_projects += 1;
        let project_status = text(&row, "status").unwrap_or_else(|| "planning".to_string());
        let budget = number(&row, "budget");
        let team_members = integer(&row, "team_members");
        total_budget += budget;
        total_members += team_members;
        if project_status == "ongoing" {
            ongoing_count += 1;
        }

        let id = text(&row, "id").unwrap_or_default();
        let name = text(&row, "name").unwrap_or_default();
        let matches_search = search.is_empty()
            || name.to_lowercase().contains(&search)
            || id.to_lowercase().contains(&search);
        let matches_status = status == "all" || project_status == status;
        if !matches_search || !matches_status {
            continue;
        }

        let rd_summary = project_rd_cost_summary(rd_costs_by_project.get(&id));
        let delete_action = project_delete_action(&id, &name, can_manage);
        let update_action = project_update_action(&id, can_manage);
        projects.push(json!({
            "id": id,
            "project_no": id,
            "name": name,
            "description": text(&row, "description"),
            "project_type": text(&row, "project_type"),
            "leader_name": text(&row, "leader_name"),
            "status": project_status,
            "status_badge": project_status_badge(&project_status),
            "priority": text(&row, "priority"),
            "budget": round_money(budget),
            "budget_label": format_wan(budget),
            "spent": number(&row, "spent"),
            "start_date": text(&row, "start_date"),
            "end_date": text(&row, "end_date"),
            "client": text(&row, "client"),
            "progress": integer(&row, "progress"),
            "progress_label": format!("{}%", integer(&row, "progress")),
            "team_members": team_members,
            "goals": text(&row, "goals"),
            "notes": text(&row, "notes"),
            "milestones": text(&row, "milestones"),
            "created_at": text(&row, "created_at"),
            "updated_at": text(&row, "updated_at"),
            "rd_cost_summary": rd_summary,
            "can_edit": can_manage,
            "update_action": update_action,
            "can_delete": can_manage,
            "delete_action": delete_action,
            "can_add_rd_cost": can_manage,
            "rd_cost_create_action": rd_cost_create_action(&id, can_manage),
        }));
    }

    let has_filters = !search.is_empty() || status != "all";
    let empty_message = if has_filters {
        "未找到匹配的项目"
    } else {
        "暂无项目数据"
    }
    .to_string();

    let employee_options = project_employee_options(&pool, &user.company_id).await?;
    let asset_options = project_asset_options(&pool, &user.company_id).await?;

    Ok(Json(ProjectsOverview {
        stats: ProjectOverviewStats {
            total_projects,
            ongoing_count,
            total_budget: round_money(total_budget),
            total_budget_label: format_wan(total_budget),
            total_members,
            total_rd_cost: round_money(total_rd_cost),
            total_rd_cost_label: format_wan(total_rd_cost),
        },
        projects,
        status_options: project_status_options(),
        rd_cost_type_options: rd_cost_type_options(),
        employee_options,
        asset_options,
        create_action: project_create_action(can_manage),
        permissions: json!({
            "can_create": can_manage,
            "can_edit": can_manage,
            "can_delete": can_manage,
            "can_manage_rd_costs": can_manage,
        }),
        empty_message,
    }))
}

fn project_create_action(can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }

    operations_create_action("/api/projects".to_string(), "新建项目", "项目已成功创建")
}

fn project_update_action(id: &str, can_manage: bool) -> Value {
    if !can_manage || id.trim().is_empty() {
        return json!(null);
    }

    operations_update_action(format!("/api/projects/{id}"), "保存修改", "项目已成功更新")
}

fn project_delete_action(id: &str, name: &str, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let name = name.trim();
    let name = if name.is_empty() { "该项目" } else { name };
    operations_delete_action(
        format!("/api/projects/{id}"),
        format!("确定要删除项目「{name}」吗？"),
        "项目已成功删除",
    )
}

fn rd_cost_delete_action(
    id: Option<String>,
    cost_type_label: &str,
    cost_date: Option<&str>,
    can_manage: bool,
) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };
    let cost_date = cost_date
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("未设置日期");
    operations_delete_action(
        format!("/api/v1/rd_costs/{id}"),
        format!("确定要删除研发费用「{cost_type_label} {cost_date}」吗？"),
        "研发费用已删除",
    )
}

fn rd_cost_create_action(project_id: &str, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }

    let mut action =
        operations_create_action("/api/v1/rd_costs".to_string(), "添加费用", "研发费用已添加");
    if let Value::Object(ref mut object) = action {
        object.insert(
            "default_values".to_string(),
            json!({
                "project_id": project_id,
            }),
        );
    }
    action
}

fn rd_cost_update_action(id: Option<String>, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };

    operations_update_action(
        format!("/api/v1/rd_costs/{id}"),
        "保存费用",
        "研发费用已更新",
    )
}

async fn deals_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<Value>, ApiError> {
    let can_manage = current_user_can_manage(&pool, &user).await?;
    let client_rows = sqlx::query(
        r#"
        SELECT id, name, short_name, client_type
        FROM clients
        WHERE company_id = ?
        ORDER BY name ASC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let clients: Vec<Value> = client_rows
        .into_iter()
        .map(|row| {
            json!({
                "id": text(&row, "id"),
                "name": text(&row, "name"),
                "short_name": text(&row, "short_name"),
                "client_type": text(&row, "client_type"),
            })
        })
        .collect();

    let rows = sqlx::query(
        r#"
        SELECT
            d.id,
            d.client_id,
            COALESCE(c.name, '-') AS client_name,
            d.contact_id,
            d.title,
            d.description,
            d.stage,
            d.amount,
            d.currency,
            d.probability,
            d.expected_close_date,
            d.actual_close_date,
            d.lost_reason,
            d.source,
            d.assigned_to,
            d.tags,
            d.created_at,
            d.updated_at
        FROM deals d
        LEFT JOIN clients c ON c.id = d.client_id AND c.company_id = d.company_id
        WHERE d.company_id = ?
        ORDER BY d.created_at DESC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let stage_options = deal_stage_options();
    let mut pipeline_order: Vec<String> = stage_options
        .iter()
        .filter(|stage| !is_closed_deal_stage(&stage.value))
        .map(|stage| stage.value.clone())
        .collect();
    let mut pipeline_groups: HashMap<String, DealPipelineStageGroup> = pipeline_order
        .iter()
        .map(|stage| {
            let option = deal_stage_option(stage);
            (
                stage.clone(),
                DealPipelineStageGroup {
                    value: option.value,
                    label: option.label,
                    color: option.color,
                    count: 0,
                    amount: 0.0,
                    amount_label: format_cny(0.0),
                    deals: Vec::new(),
                },
            )
        })
        .collect();

    let mut total_pipeline = 0.0;
    let mut won_amount = 0.0;
    let mut active_count = 0_i64;
    let mut deal_count = 0_i64;
    let mut probability_sum = 0_i64;
    let mut won_deals = Vec::new();
    let mut lost_deals = Vec::new();

    for row in rows {
        let id = text(&row, "id");
        let stage = text(&row, "stage").unwrap_or_else(|| "lead".to_string());
        let amount = number(&row, "amount");
        let probability = integer(&row, "probability");
        let amount_label = format_cny(amount);
        let probability_label = format!("{probability}%");
        deal_count += 1;
        probability_sum += probability;

        let next_stages: Vec<Value> = stage_options
            .iter()
            .filter(|option| option.value != stage)
            .take(3)
            .map(|option| {
                json!({
                    "value": option.value.clone(),
                    "label": option.label.clone(),
                    "color": option.color.clone(),
                    "action": deal_stage_update_action(
                        id.clone(),
                        option.value.as_str(),
                        option.label.as_str(),
                        can_manage,
                    ),
                })
            })
            .collect();

        let deal = json!({
            "id": id.clone(),
            "client_id": text(&row, "client_id"),
            "client_name": text(&row, "client_name").unwrap_or_else(|| "-".to_string()),
            "contact_id": text(&row, "contact_id"),
            "title": text(&row, "title"),
            "description": text(&row, "description"),
            "stage": stage.clone(),
            "stage_label": deal_stage_label(&stage),
            "amount": round_money(amount),
            "amount_label": amount_label,
            "currency": text(&row, "currency").unwrap_or_else(|| "CNY".to_string()),
            "probability": probability,
            "probability_label": probability_label,
            "expected_close_date": text(&row, "expected_close_date"),
            "expected_close_date_label": cloud_date_label(text(&row, "expected_close_date").as_deref()),
            "actual_close_date": text(&row, "actual_close_date"),
            "actual_close_date_label": cloud_date_label(text(&row, "actual_close_date").as_deref()),
            "lost_reason": text(&row, "lost_reason"),
            "source": text(&row, "source"),
            "assigned_to": text(&row, "assigned_to"),
            "tags": text(&row, "tags"),
            "created_at": text(&row, "created_at"),
            "updated_at": text(&row, "updated_at"),
            "next_stages": next_stages,
            "can_update_stage": can_manage,
            "can_delete": can_manage,
            "update_action": deal_update_action(id.clone(), can_manage),
        });

        match stage.as_str() {
            "won" => {
                won_amount += amount;
                won_deals.push(deal);
            }
            "lost" => {
                lost_deals.push(deal);
            }
            _ => {
                total_pipeline += amount;
                active_count += 1;
                if !pipeline_groups.contains_key(&stage) {
                    let option = deal_stage_option(&stage);
                    pipeline_order.push(stage.clone());
                    pipeline_groups.insert(
                        stage.clone(),
                        DealPipelineStageGroup {
                            value: option.value,
                            label: option.label,
                            color: option.color,
                            count: 0,
                            amount: 0.0,
                            amount_label: format_cny(0.0),
                            deals: Vec::new(),
                        },
                    );
                }

                if let Some(group) = pipeline_groups.get_mut(&stage) {
                    group.count += 1;
                    group.amount = round_money(group.amount + amount);
                    group.amount_label = format_cny(group.amount);
                    group.deals.push(deal);
                }
            }
        }
    }

    let avg_probability = if deal_count > 0 {
        ((probability_sum as f64) / (deal_count as f64)).round() as i64
    } else {
        0
    };

    let pipeline_stages: Vec<DealPipelineStageGroup> = pipeline_order
        .into_iter()
        .filter_map(|stage| pipeline_groups.remove(&stage))
        .collect();

    Ok(Json(json!({
        "stats": {
            "active_count": active_count,
            "total_pipeline": round_money(total_pipeline),
            "total_pipeline_label": format_cny(total_pipeline),
            "won_amount": round_money(won_amount),
            "won_amount_label": format_cny(won_amount),
            "avg_probability": avg_probability,
            "avg_probability_label": format!("{avg_probability}%"),
        },
        "stage_options": stage_options,
        "pipeline_stages": pipeline_stages,
        "won_deals": won_deals,
        "lost_deals": lost_deals,
        "clients": clients,
        "create_action": deal_create_action(can_manage),
        "permissions": {
            "can_manage": can_manage,
        },
        "empty_messages": {
            "pipeline": "暂无",
        },
    })))
}

fn deal_create_action(can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }

    operations_create_action("/api/v1/deals".to_string(), "新增机会", "销售机会已创建")
}

fn deal_update_action(id: Option<String>, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };

    operations_update_action(format!("/api/v1/deals/{id}"), "保存修改", "销售机会已更新")
}

fn deal_stage_update_action(
    id: Option<String>,
    stage: &str,
    stage_label: &str,
    can_manage: bool,
) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };

    let mut action = operations_update_action(
        format!("/api/v1/deals/{id}"),
        format!("转为{stage_label}"),
        format!("销售机会已转为{stage_label}"),
    );
    if let Some(object) = action.as_object_mut() {
        object.insert("default_values".to_string(), json!({ "stage": stage }));
    }
    action
}

async fn quotes_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<Value>, ApiError> {
    let can_manage = current_user_can_manage(&pool, &user).await?;
    let client_rows = sqlx::query(
        r#"
        SELECT id, name, short_name, client_type
        FROM clients
        WHERE company_id = ?
        ORDER BY name ASC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let clients: Vec<Value> = client_rows
        .into_iter()
        .map(|row| {
            json!({
                "id": text(&row, "id"),
                "name": text(&row, "name"),
                "short_name": text(&row, "short_name"),
                "client_type": text(&row, "client_type"),
            })
        })
        .collect();

    // Preview next QUO-YYYY-NNNN without consuming the sequence.
    let next_quote_number =
        crate::app::numbering::peek_document_number(&pool, &user.company_id, "QUO").await?;

    let rows = sqlx::query(
        r#"
        SELECT
            q.id,
            q.client_id,
            COALESCE(c.name, '-') AS client_name,
            q.contact_id,
            q.deal_id,
            q.quote_number,
            q.title,
            q.status,
            q.subtotal,
            q.tax_rate,
            q.tax_amount,
            q.discount,
            q.total,
            q.currency,
            q.valid_until,
            q.notes,
            q.terms,
            q.invoice_id,
            q.created_at,
            q.updated_at
        FROM quotes q
        LEFT JOIN clients c ON c.id = q.client_id AND c.company_id = q.company_id
        WHERE q.company_id = ?
        ORDER BY q.created_at DESC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let mut total = 0_i64;
    let mut draft = 0_i64;
    let mut sent = 0_i64;
    let mut total_amount = 0.0;
    let mut quotes = Vec::new();

    for row in rows {
        let id = text(&row, "id");
        let quote_number = text(&row, "quote_number");
        let status = text(&row, "status").unwrap_or_else(|| "draft".to_string());
        let quote_total = number(&row, "total");
        let invoice_id = text(&row, "invoice_id");
        let valid_until = text(&row, "valid_until");
        let valid_until_label = cloud_date_label(valid_until.as_deref());
        let status_label = quote_status_label(&status);
        let status_variant = quote_status_variant(&status);
        let can_convert_to_invoice =
            status == "accepted" && invoice_id.as_deref().unwrap_or("").is_empty();
        let delete_action = quote_delete_action(id.clone(), quote_number.clone(), can_manage);
        let update_action = quote_update_action(id.clone(), can_manage);
        let convert_action = quote_convert_action(id.clone(), can_convert_to_invoice, can_manage);

        total += 1;
        total_amount += quote_total;
        if status == "draft" {
            draft += 1;
        }
        if status == "sent" {
            sent += 1;
        }

        quotes.push(json!({
            "id": id,
            "client_id": text(&row, "client_id"),
            "client_name": text(&row, "client_name").unwrap_or_else(|| "-".to_string()),
            "contact_id": text(&row, "contact_id"),
            "deal_id": text(&row, "deal_id"),
            "quote_number": quote_number,
            "title": text(&row, "title"),
            "status": status.clone(),
            "status_label": status_label,
            "status_variant": status_variant,
            "status_badge": cloud_badge(status_label, status_variant, ""),
            "subtotal": round_money(number(&row, "subtotal")),
            "subtotal_label": format_cny(number(&row, "subtotal")),
            "tax_rate": number(&row, "tax_rate"),
            "tax_amount": round_money(number(&row, "tax_amount")),
            "tax_amount_label": format_cny(number(&row, "tax_amount")),
            "discount": round_money(number(&row, "discount")),
            "discount_label": format_cny(number(&row, "discount")),
            "total": round_money(quote_total),
            "total_label": format_cny(quote_total),
            "currency": text(&row, "currency").unwrap_or_else(|| "CNY".to_string()),
            "valid_until": valid_until,
            "valid_until_label": valid_until_label,
            "notes": text(&row, "notes"),
            "terms": text(&row, "terms"),
            "invoice_id": invoice_id.clone(),
            "can_convert_to_invoice": can_convert_to_invoice,
            "convert_action": convert_action,
            "can_edit": can_manage,
            "can_delete": can_manage,
            "update_action": update_action,
            "delete_action": delete_action,
            "created_at": text(&row, "created_at"),
            "updated_at": text(&row, "updated_at"),
        }));
    }

    Ok(Json(json!({
        "stats": {
            "total": total,
            "draft": draft,
            "sent": sent,
            "total_amount": round_money(total_amount),
            "total_amount_label": format_cny(total_amount),
        },
        "next_quote_number": next_quote_number,
        "quotes": quotes,
        "clients": clients,
        "status_options": quote_status_options(),
        "create_action": quote_create_action(can_manage),
        "permissions": {
            "can_manage": can_manage,
        },
        "empty_message": "暂无报价单",
    })))
}

fn quote_create_action(can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }

    operations_create_action("/api/v1/quotes".to_string(), "新建报价单", "报价单已创建")
}

fn quote_update_action(id: Option<String>, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };

    operations_update_action(format!("/api/v1/quotes/{id}"), "保存修改", "报价单已更新")
}

fn quote_convert_action(id: Option<String>, can_convert: bool, can_manage: bool) -> Value {
    if !can_manage || !can_convert {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };
    operations_post_action(
        format!("/api/v1/quotes/{id}/convert-to-invoice"),
        "转为发票",
        "已根据报价单生成发票",
        "invoiced",
    )
}

fn quote_delete_action(
    id: Option<String>,
    quote_number: Option<String>,
    can_manage: bool,
) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };
    let quote_number = quote_number
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("该报价单");
    operations_delete_action(
        format!("/api/v1/quotes/{id}"),
        format!("确定要删除报价单「{quote_number}」吗？"),
        "报价单已删除",
    )
}

async fn contracts_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(query): Query<ContractsOverviewQuery>,
) -> Result<Json<Value>, ApiError> {
    let can_manage = current_user_can_manage(&pool, &user).await?;
    let rows = sqlx::query(
        r#"
        SELECT
            id,
            contract_name,
            contract_number,
            contract_type,
            counterparty,
            party_a,
            party_b,
            amount,
            start_date,
            end_date,
            status,
            file_path,
            project_id,
            content,
            notes,
            ai_review_status,
            ai_risk_level,
            ai_review_result,
            ai_reviewed_at,
            counterparty_type,
            created_at,
            updated_at
        FROM contracts
        WHERE company_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let search = query
        .search
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .to_lowercase();
    let status_filter = query.status.as_deref().map(str::trim).unwrap_or("all");

    let mut total = 0_i64;
    let mut active = 0_i64;
    let mut expired = 0_i64;
    let mut high_risk = 0_i64;
    let mut total_amount = 0.0;
    let mut contracts = Vec::new();

    for row in rows {
        total += 1;
        let id = text(&row, "id").unwrap_or_default();
        let name = text(&row, "contract_name").unwrap_or_default();
        let number_value = text(&row, "contract_number").unwrap_or_default();
        let status = text(&row, "status").unwrap_or_else(|| "active".to_string());
        let amount = number(&row, "amount");
        let risk = text(&row, "ai_risk_level");
        let review_status = text(&row, "ai_review_status").unwrap_or_else(|| "pending".to_string());
        let review_result = text(&row, "ai_review_result");
        let counterparty_type = text(&row, "counterparty_type");

        total_amount += amount;
        if status == "active" {
            active += 1;
        }
        if status == "expired" {
            expired += 1;
        }
        if is_high_contract_risk(risk.as_deref()) {
            high_risk += 1;
        }

        let matches_search = search.is_empty()
            || id.to_lowercase().contains(&search)
            || name.to_lowercase().contains(&search)
            || number_value.to_lowercase().contains(&search);
        let matches_status = status_filter == "all" || status == status_filter;
        if !matches_search || !matches_status {
            continue;
        }

        let ai_reviewed_at = text(&row, "ai_reviewed_at");
        let ai_reviewed_at_label = cloud_datetime_label(ai_reviewed_at.as_deref());
        let delete_action = contract_delete_action(&id, &name, can_manage);
        let update_action = contract_update_action(&id, can_manage);
        contracts.push(json!({
            "id": id,
            "contract_name": name,
            "contract_number": number_value,
            "contract_type": text(&row, "contract_type"),
            "counterparty": text(&row, "counterparty"),
            "party_a": text(&row, "party_a"),
            "party_b": text(&row, "party_b"),
            "amount": round_money(amount),
            "amount_label": format_cny(amount),
            "start_date": text(&row, "start_date"),
            "end_date": text(&row, "end_date"),
            "status": status.clone(),
            "status_badge": contract_status_badge(&status),
            "file_path": text(&row, "file_path"),
            "file_name": Value::Null,
            "project_id": text(&row, "project_id"),
            "content": text(&row, "content"),
            "notes": text(&row, "notes"),
            "ai_review_status": review_status.clone(),
            "ai_risk_level": risk.clone(),
            "ai_review_result": review_result.clone(),
            "ai_reviewed_at": ai_reviewed_at,
            "ai_reviewed_at_label": ai_reviewed_at_label,
            "ai_risk_badge": contract_ai_risk_badge(&review_status, risk.as_deref()),
            "ai_review_display": contract_ai_review_display(&review_status, risk.as_deref(), review_result.as_deref()),
            "counterparty_type": counterparty_type.clone(),
            "counterparty_type_badge": contract_counterparty_type_badge(counterparty_type.as_deref()),
            "can_edit": can_manage,
            "can_delete": can_manage,
            "update_action": update_action,
            "delete_action": delete_action,
            "retrigger_review_action": contract_retrigger_review_action(&id, can_manage),
            "created_at": text(&row, "created_at"),
            "updated_at": text(&row, "updated_at"),
        }));
    }

    let ai_guard_tone = if high_risk > 0 { "danger" } else { "safe" };
    let ai_guard_label = if high_risk > 0 {
        format!("{high_risk} 份高风险")
    } else {
        "安全".to_string()
    };

    Ok(Json(json!({
        "stats": {
            "total": total,
            "active": active,
            "expired": expired,
            "total_amount": round_money(total_amount),
            "total_amount_label": format_wan(total_amount),
            "high_risk": high_risk,
            "ai_guard_label": ai_guard_label,
            "ai_guard_tone": ai_guard_tone,
            "ai_guard_icon": if high_risk > 0 { "ShieldAlert" } else { "CheckCircle" },
            "ai_guard_display": contract_ai_guard_display(high_risk, &ai_guard_label),
        },
        "contracts": contracts,
        "status_options": contract_status_options(),
        "create_action": contract_create_action(can_manage),
        "permissions": {
            "can_manage": can_manage,
            "can_create": can_manage,
            "can_edit": can_manage,
            "can_delete": can_manage,
        },
    })))
}

fn contract_create_action(can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }

    operations_create_action("/api/contracts".to_string(), "新建合同", "合同已成功创建")
}

fn contract_update_action(id: &str, can_manage: bool) -> Value {
    if !can_manage || id.trim().is_empty() {
        return json!(null);
    }

    operations_update_action(format!("/api/contracts/{id}"), "保存修改", "合同已成功更新")
}

fn contract_delete_action(id: &str, name: &str, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let name = name.trim();
    let name = if name.is_empty() { "该合同" } else { name };
    operations_delete_action(
        format!("/api/contracts/{id}"),
        format!("确定要删除合同「{name}」吗？"),
        "合同已成功删除",
    )
}

fn contract_retrigger_review_action(id: &str, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }

    operations_post_action(
        format!("/api/contracts/{id}/retrigger_review"),
        "重新审查",
        "AI 正在重新分析合同",
        "pending",
    )
}

fn contract_ai_guard_display(high_risk: i64, label: &str) -> Value {
    if high_risk > 0 {
        json!({
            "label": label,
            "icon": "ShieldAlert",
            "card_class_name": "border-red-500/50",
            "icon_class_name": "text-red-500",
            "value_class_name": "text-red-500",
        })
    } else {
        json!({
            "label": label,
            "icon": "CheckCircle",
            "card_class_name": "",
            "icon_class_name": "text-green-500",
            "value_class_name": "text-green-500",
        })
    }
}

async fn contracts_form_options(_user: AuthUser) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({
        "contract_type_options": contract_type_options(),
        "counterparty_type_options": counterparty_type_options(),
        "default_values": {
            "contract_type": "",
            "counterparty_type": ""
        }
    })))
}

fn contract_type_options() -> Vec<Value> {
    [
        ("purchase", "采购合同"),
        ("rental", "租赁合同"),
        ("cooperation", "合作协议"),
        ("service", "服务合同"),
        ("labor", "劳动合同"),
        ("investment", "投资协议"),
        ("nda", "保密协议"),
    ]
    .into_iter()
    .map(|(value, label)| {
        json!({
            "value": value,
            "label": label,
        })
    })
    .collect()
}

fn counterparty_type_options() -> Vec<Value> {
    [
        "vc",
        "angel",
        "government",
        "incubator",
        "employee",
        "agency",
        "client",
        "landlord",
    ]
    .into_iter()
    .map(|value| {
        json!({
            "value": value,
            "label": counterparty_type_label(value),
        })
    })
    .collect()
}

fn counterparty_type_label(counterparty_type: &str) -> String {
    match counterparty_type {
        "vc" => "VC / 风投",
        "angel" => "天使投资人",
        "government" => "政府机构",
        "incubator" => "孵化器",
        "employee" => "员工",
        "agency" => "中介 / 外包",
        "client" => "客户",
        "landlord" => "房东",
        value => value,
    }
    .to_string()
}

fn contract_counterparty_type_badge(counterparty_type: Option<&str>) -> Value {
    let Some(value) = counterparty_type
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Value::Null;
    };

    json!({
        "value": value,
        "label": counterparty_type_label(value),
        "variant": "outline",
    })
}

async fn asset_records_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(query): Query<AssetRecordsOverviewQuery>,
) -> Result<Json<Value>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT
            ar.id,
            ar.asset_id,
            ar.record_type,
            ar.description,
            ar.amount,
            ar.record_date,
            ar.created_at,
            ar.updated_at,
            a.name AS asset_name,
            a.serial_number AS asset_serial_number
        FROM asset_records ar
        LEFT JOIN assets a ON a.id = ar.asset_id AND a.company_id = ar.company_id
        WHERE ar.company_id = ?
        ORDER BY COALESCE(ar.record_date, ar.created_at) DESC, ar.created_at DESC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let search = query
        .search
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .to_lowercase();
    let action_filter = query.action.as_deref().map(str::trim).unwrap_or("all");

    let mut total = 0_i64;
    let mut checkout = 0_i64;
    let mut transfer = 0_i64;
    let mut maintenance = 0_i64;
    let mut records = Vec::new();
    let mut export_rows = Vec::new();

    for row in rows {
        total += 1;
        let id = text(&row, "id").unwrap_or_default();
        let action = text(&row, "record_type").unwrap_or_else(|| "inventory".to_string());
        let asset_name = text(&row, "asset_name").unwrap_or_else(|| "-".to_string());
        let asset_serial_number =
            text(&row, "asset_serial_number").unwrap_or_else(|| "-".to_string());
        let description = text(&row, "description").unwrap_or_default();
        let display_date = text(&row, "record_date")
            .or_else(|| text(&row, "created_at"))
            .unwrap_or_else(|| "-".to_string());
        let action_label = asset_record_action_label(&action);

        match action.as_str() {
            "checkout" => checkout += 1,
            "transfer" => transfer += 1,
            "maintenance" => maintenance += 1,
            _ => {}
        }

        let matches_search = search.is_empty()
            || id.to_lowercase().contains(&search)
            || asset_name.to_lowercase().contains(&search)
            || asset_serial_number.to_lowercase().contains(&search)
            || description.to_lowercase().contains(&search);
        let matches_action = action_filter == "all" || action == action_filter;
        if !matches_search || !matches_action {
            continue;
        }

        export_rows.push(vec![
            id.clone(),
            asset_name.clone(),
            asset_serial_number.clone(),
            action_label.to_string(),
            "-".to_string(),
            "-".to_string(),
            display_date.clone(),
            "-".to_string(),
        ]);

        records.push(json!({
            "id": id,
            "asset_id": text(&row, "asset_id"),
            "action": action.clone(),
            "record_type": action,
            "action_label": action_label,
            "action_badge": asset_record_action_badge(action_label),
            "description": description,
            "amount": round_money(number(&row, "amount")),
            "record_date": text(&row, "record_date"),
            "created_at": text(&row, "created_at"),
            "updated_at": text(&row, "updated_at"),
            "display_date": display_date,
            "handler_name": Value::Null,
            "department": Value::Null,
            "from_location": Value::Null,
            "to_location": Value::Null,
            "location_label": "-",
            "asset_name": asset_name,
            "asset_serial_number": asset_serial_number,
            "assets": {
                "name": text(&row, "asset_name"),
                "serial_number": text(&row, "asset_serial_number"),
            },
        }));
    }

    Ok(Json(json!({
        "stats": {
            "total": total,
            "checkout": checkout,
            "transfer": transfer,
            "maintenance": maintenance,
        },
        "records": records,
        "action_options": asset_record_action_options(),
        "export": {
            "filename": format!("asset_records_{}.csv", Utc::now().format("%Y%m%d")),
            "csv": asset_record_export_csv(export_rows),
        },
    })))
}

async fn reminders_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<Value>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT
            id,
            title,
            description,
            reminder_type,
            priority,
            due_date,
            remind_date,
            status,
            is_completed,
            entity_type,
            entity_id,
            created_at,
            updated_at
        FROM reminders
        WHERE company_id = ?
        ORDER BY COALESCE(remind_date, due_date, created_at) ASC, created_at DESC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let today = Utc::now().date_naive();
    let mut pending = 0_i64;
    let mut completed = 0_i64;
    let mut dismissed = 0_i64;
    let mut pending_reminders = Vec::new();
    let mut history_reminders = Vec::new();

    for row in rows {
        let status = text(&row, "status").unwrap_or_else(|| "pending".to_string());

        match status.as_str() {
            "completed" => completed += 1,
            "dismissed" => dismissed += 1,
            _ => pending += 1,
        }

        let reminder = reminder_display_value(&row, today);

        if status == "completed" || status == "dismissed" {
            history_reminders.push(reminder);
        } else {
            pending_reminders.push(reminder);
        }
    }

    Ok(Json(json!({
        "stats": {
            "pending": pending,
            "completed": completed,
            "dismissed": dismissed,
        },
        "pending_reminders": pending_reminders,
        "history_reminders": history_reminders,
        "check_asset_expiry_action": {
            "method": "POST",
            "endpoint": "/api/v1/reminders/check_asset_expiry",
            "label": "检查资产到期",
            "success_message": "资产到期提醒检查完成"
        },
        "check_operating_deadlines_action": {
            "method": "POST",
            "endpoint": "/api/v1/reminders/check_operating_deadlines",
            "label": "检查运营到期",
            "success_message": "运营到期提醒检查完成"
        },
    })))
}

async fn complete_reminder(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    update_reminder_status(&pool, &user, &id, "completed", 1).await
}

async fn dismiss_reminder(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    update_reminder_status(&pool, &user, &id, "dismissed", 0).await
}

async fn update_reminder_status(
    pool: &SqlitePool,
    user: &AuthUser,
    id: &str,
    status: &str,
    is_completed: i64,
) -> Result<Json<Value>, ApiError> {
    let result = sqlx::query(
        r#"
        UPDATE reminders
        SET status = ?, is_completed = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ? AND company_id = ?
        "#,
    )
    .bind(status)
    .bind(is_completed)
    .bind(id)
    .bind(&user.company_id)
    .execute(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    if result.rows_affected() == 0 {
        return Err(api_err(StatusCode::NOT_FOUND, "reminder not found"));
    }

    let row = fetch_reminder_row(pool, &user.company_id, id).await?;
    Ok(Json(reminder_display_value(&row, Utc::now().date_naive())))
}

async fn check_asset_expiry_reminders(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<Value>, ApiError> {
    let today = Utc::now().date_naive();
    let window_end = today + Duration::days(30);
    let today_label = today.to_string();
    let window_end_label = window_end.to_string();

    let assets = sqlx::query(
        r#"
        SELECT id, name, expiry_date
        FROM assets a
        WHERE a.company_id = ?
          AND a.asset_type IN ('virtual', 'data', 'intangible')
          AND a.expiry_date IS NOT NULL
          AND a.expiry_date != ''
          AND a.expiry_date >= ?
          AND a.expiry_date <= ?
          AND NOT EXISTS (
              SELECT 1
              FROM reminders r
              WHERE r.company_id = a.company_id
                AND r.entity_type = 'assets'
                AND r.entity_id = a.id
                AND r.reminder_type = 'asset_expiry'
          )
        ORDER BY a.expiry_date ASC, a.name ASC
        "#,
    )
    .bind(&user.company_id)
    .bind(&today_label)
    .bind(&window_end_label)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let mut created_reminders = Vec::new();
    for asset in assets {
        let asset_id = text(&asset, "id").unwrap_or_default();
        let asset_name = text(&asset, "name").unwrap_or_else(|| "资产".to_string());
        let expiry_date = text(&asset, "expiry_date").unwrap_or_default();
        let priority = asset_expiry_reminder_priority(&expiry_date, today);
        let reminder_id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT INTO reminders (
                id, company_id, title, description, reminder_type, priority,
                due_date, remind_date, status, is_completed, entity_type, entity_id
            )
            VALUES (?, ?, ?, ?, 'asset_expiry', ?, ?, ?, 'pending', 0, 'assets', ?)
            "#,
        )
        .bind(&reminder_id)
        .bind(&user.company_id)
        .bind(format!("资产即将到期：{asset_name}"))
        .bind(format!("{asset_name} 将于 {expiry_date} 到期，请及时处理"))
        .bind(priority)
        .bind(&expiry_date)
        .bind(&today_label)
        .bind(&asset_id)
        .execute(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

        let row = fetch_reminder_row(&pool, &user.company_id, &reminder_id).await?;
        created_reminders.push(reminder_display_value(&row, today));
    }

    Ok(Json(json!({
        "reminders_created": created_reminders.len(),
        "created_reminders": created_reminders,
        "message": format!("检查完成：创建了 {} 条提醒", created_reminders.len()),
    })))
}

struct OperatingDeadline {
    entity_type: &'static str,
    entity_id: String,
    reminder_type: &'static str,
    title: String,
    description: String,
    due_date: String,
}

async fn check_operating_deadline_reminders(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<Value>, ApiError> {
    let today = Utc::now().date_naive();
    let window_end = today + Duration::days(30);
    let today_label = today.to_string();
    let window_end_label = window_end.to_string();

    let mut deadlines = Vec::new();
    deadlines.extend(
        fetch_contract_deadlines(&pool, &user.company_id, &today_label, &window_end_label).await?,
    );
    deadlines.extend(
        fetch_license_deadlines(&pool, &user.company_id, &today_label, &window_end_label).await?,
    );
    deadlines.extend(
        fetch_rent_deadlines(&pool, &user.company_id, &today_label, &window_end_label).await?,
    );
    deadlines.extend(
        fetch_asset_deadlines(&pool, &user.company_id, &today_label, &window_end_label).await?,
    );

    let mut created_reminders = Vec::new();
    for deadline in deadlines {
        let priority = asset_expiry_reminder_priority(&deadline.due_date, today);
        let reminder_id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT INTO reminders (
                id, company_id, title, description, reminder_type, priority,
                due_date, remind_date, status, is_completed, entity_type, entity_id
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'pending', 0, ?, ?)
            "#,
        )
        .bind(&reminder_id)
        .bind(&user.company_id)
        .bind(&deadline.title)
        .bind(&deadline.description)
        .bind(deadline.reminder_type)
        .bind(priority)
        .bind(&deadline.due_date)
        .bind(&today_label)
        .bind(deadline.entity_type)
        .bind(&deadline.entity_id)
        .execute(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

        let row = fetch_reminder_row(&pool, &user.company_id, &reminder_id).await?;
        created_reminders.push(reminder_display_value(&row, today));
    }

    Ok(Json(json!({
        "reminders_created": created_reminders.len(),
        "created_reminders": created_reminders,
        "message": format!("检查完成：创建了 {} 条提醒", created_reminders.len()),
    })))
}

async fn fetch_contract_deadlines(
    pool: &SqlitePool,
    company_id: &str,
    today: &str,
    window_end: &str,
) -> Result<Vec<OperatingDeadline>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT id, contract_name, end_date
        FROM contracts c
        WHERE c.company_id = ?
          AND LOWER(COALESCE(c.status, 'active')) NOT IN (
              'inactive', 'expired', 'terminated', 'cancelled', 'canceled', 'archived', 'completed'
          )
          AND c.end_date IS NOT NULL
          AND c.end_date != ''
          AND c.end_date >= ?
          AND c.end_date <= ?
          AND NOT EXISTS (
              SELECT 1
              FROM reminders r
              WHERE r.company_id = c.company_id
                AND r.entity_type = 'contracts'
                AND r.entity_id = c.id
                AND r.reminder_type = 'contract_expiry'
          )
        ORDER BY c.end_date ASC, c.contract_name ASC
        "#,
    )
    .bind(company_id)
    .bind(today)
    .bind(window_end)
    .fetch_all(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let entity_id = text(&row, "id").unwrap_or_default();
            let name = text(&row, "contract_name").unwrap_or_else(|| "合同".to_string());
            let due_date = text(&row, "end_date").unwrap_or_default();
            OperatingDeadline {
                entity_type: "contracts",
                entity_id,
                reminder_type: "contract_expiry",
                title: format!("合同即将到期：{name}"),
                description: format!("{name} 将于 {due_date} 到期，请及时处理"),
                due_date,
            }
        })
        .collect())
}

async fn fetch_license_deadlines(
    pool: &SqlitePool,
    company_id: &str,
    today: &str,
    window_end: &str,
) -> Result<Vec<OperatingDeadline>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT id, license_name, expiry_date
        FROM licenses l
        WHERE l.company_id = ?
          AND LOWER(COALESCE(l.status, 'valid')) NOT IN (
              'inactive', 'expired', 'revoked', 'cancelled', 'canceled', 'archived'
          )
          AND l.expiry_date IS NOT NULL
          AND l.expiry_date != ''
          AND l.expiry_date >= ?
          AND l.expiry_date <= ?
          AND NOT EXISTS (
              SELECT 1
              FROM reminders r
              WHERE r.company_id = l.company_id
                AND r.entity_type = 'licenses'
                AND r.entity_id = l.id
                AND r.reminder_type = 'license_expiry'
          )
        ORDER BY l.expiry_date ASC, l.license_name ASC
        "#,
    )
    .bind(company_id)
    .bind(today)
    .bind(window_end)
    .fetch_all(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let entity_id = text(&row, "id").unwrap_or_default();
            let name = text(&row, "license_name").unwrap_or_else(|| "证照".to_string());
            let due_date = text(&row, "expiry_date").unwrap_or_default();
            OperatingDeadline {
                entity_type: "licenses",
                entity_id,
                reminder_type: "license_expiry",
                title: format!("证照即将到期：{name}"),
                description: format!("{name} 将于 {due_date} 到期，请及时处理"),
                due_date,
            }
        })
        .collect())
}

async fn fetch_rent_deadlines(
    pool: &SqlitePool,
    company_id: &str,
    today: &str,
    window_end: &str,
) -> Result<Vec<OperatingDeadline>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT id, location_name, contract_end_date
        FROM rent_records rr
        WHERE rr.company_id = ?
          AND LOWER(COALESCE(rr.status, 'active')) = 'active'
          AND rr.contract_end_date IS NOT NULL
          AND rr.contract_end_date != ''
          AND rr.contract_end_date >= ?
          AND rr.contract_end_date <= ?
          AND NOT EXISTS (
              SELECT 1
              FROM reminders r
              WHERE r.company_id = rr.company_id
                AND r.entity_type = 'rent_records'
                AND r.entity_id = rr.id
                AND r.reminder_type = 'rent_expiry'
          )
        ORDER BY rr.contract_end_date ASC, rr.location_name ASC
        "#,
    )
    .bind(company_id)
    .bind(today)
    .bind(window_end)
    .fetch_all(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let entity_id = text(&row, "id").unwrap_or_default();
            let name = text(&row, "location_name").unwrap_or_else(|| "租赁记录".to_string());
            let due_date = text(&row, "contract_end_date").unwrap_or_default();
            OperatingDeadline {
                entity_type: "rent_records",
                entity_id,
                reminder_type: "rent_expiry",
                title: format!("租赁即将到期：{name}"),
                description: format!("{name} 将于 {due_date} 到期，请及时处理"),
                due_date,
            }
        })
        .collect())
}

async fn fetch_asset_deadlines(
    pool: &SqlitePool,
    company_id: &str,
    today: &str,
    window_end: &str,
) -> Result<Vec<OperatingDeadline>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, expiry_date
        FROM assets a
        WHERE a.company_id = ?
          AND a.asset_type IN ('virtual', 'data', 'intangible')
          AND a.expiry_date IS NOT NULL
          AND a.expiry_date != ''
          AND a.expiry_date >= ?
          AND a.expiry_date <= ?
          AND NOT EXISTS (
              SELECT 1
              FROM reminders r
              WHERE r.company_id = a.company_id
                AND r.entity_type = 'assets'
                AND r.entity_id = a.id
                AND r.reminder_type = 'asset_expiry'
          )
        ORDER BY a.expiry_date ASC, a.name ASC
        "#,
    )
    .bind(company_id)
    .bind(today)
    .bind(window_end)
    .fetch_all(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let entity_id = text(&row, "id").unwrap_or_default();
            let name = text(&row, "name").unwrap_or_else(|| "资产".to_string());
            let due_date = text(&row, "expiry_date").unwrap_or_default();
            OperatingDeadline {
                entity_type: "assets",
                entity_id,
                reminder_type: "asset_expiry",
                title: format!("资产即将到期：{name}"),
                description: format!("{name} 将于 {due_date} 到期，请及时处理"),
                due_date,
            }
        })
        .collect())
}

async fn operation_logs_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(query): Query<OperationLogsOverviewQuery>,
) -> Result<Json<Value>, ApiError> {
    let can_view = current_user_can_manage(&pool, &user).await?;
    if !can_view {
        return Ok(Json(json!({
            "stats": {
                "total": 0,
                "create": 0,
                "update": 0,
                "delete": 0,
            },
            "logs": [],
            "type_options": operation_log_type_options(),
            "table_options": [{
                "value": "all",
                "label": "全部表",
            }],
            "export": {
                "filename": format!("operation_logs_{}.csv", Utc::now().format("%Y%m%d")),
                "csv": "",
            },
            "permissions": {
                "can_view": false,
                "can_export": false,
            },
        })));
    }

    let rows = sqlx::query(
        r#"
        SELECT
            id,
            user_id,
            action,
            entity_type,
            entity_id,
            details,
            ip_address,
            created_at
        FROM operation_logs
        WHERE company_id = ?
        ORDER BY created_at DESC
        LIMIT 200
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let search = query
        .search
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .to_lowercase();
    let type_filter = query
        .operation_type
        .as_deref()
        .map(str::trim)
        .unwrap_or("all")
        .to_uppercase();
    let table_filter = query.table.as_deref().map(str::trim).unwrap_or("all");

    let mut total = 0_i64;
    let mut create = 0_i64;
    let mut update = 0_i64;
    let mut delete = 0_i64;
    let mut table_options = BTreeMap::new();
    let mut logs = Vec::new();
    let mut export_rows = Vec::new();

    for row in rows {
        total += 1;
        let action = text(&row, "action").unwrap_or_else(|| "view".to_string());
        let operation_type = action.to_uppercase();
        let table_name = text(&row, "entity_type").unwrap_or_else(|| "-".to_string());
        let table_label = operation_log_table_label(&table_name);
        let details = text(&row, "details");
        let created_at = text(&row, "created_at").unwrap_or_else(|| "-".to_string());
        let record_id = text(&row, "entity_id");

        table_options.insert(table_name.clone(), table_label.to_string());
        match operation_type.as_str() {
            "CREATE" => create += 1,
            "UPDATE" => update += 1,
            "DELETE" => delete += 1,
            _ => {}
        }

        let matches_search = search.is_empty()
            || operation_type.to_lowercase().contains(&search)
            || table_name.to_lowercase().contains(&search)
            || table_label.to_lowercase().contains(&search);
        let matches_type = type_filter == "ALL" || operation_type == type_filter;
        let matches_table = table_filter == "all" || table_name == table_filter;
        if !matches_search || !matches_type || !matches_table {
            continue;
        }

        let operation_label = operation_log_action_label(&operation_type);
        let details_label = details.clone().unwrap_or_else(|| "-".to_string());
        let record_id_label = record_id.clone().unwrap_or_else(|| "-".to_string());

        export_rows.push(vec![
            created_at.clone(),
            operation_label.to_string(),
            table_label.to_string(),
            record_id_label.clone(),
            details_label.clone(),
        ]);

        logs.push(json!({
            "id": text(&row, "id"),
            "user_id": text(&row, "user_id"),
            "operation_type": operation_type.clone(),
            "operation_label": operation_label,
            "operation_badge": operation_log_action_badge(&operation_type),
            "table_name": table_name,
            "table_label": table_label,
            "record_id": record_id,
            "record_id_short": short_record_id(&record_id_label),
            "operation_detail": operation_log_detail_value(details.as_deref()),
            "details_label": details_label,
            "ip_address": text(&row, "ip_address"),
            "created_at": created_at.clone(),
            "created_at_label": created_at,
        }));
    }

    let mut table_option_values = vec![json!({
        "value": "all",
        "label": "全部表",
    })];
    table_option_values.extend(table_options.into_iter().map(|(value, label)| {
        json!({
            "value": value,
            "label": label,
        })
    }));

    Ok(Json(json!({
        "stats": {
            "total": total,
            "create": create,
            "update": update,
            "delete": delete,
        },
        "logs": logs,
        "type_options": operation_log_type_options(),
        "table_options": table_option_values,
        "export": {
            "filename": format!("operation_logs_{}.csv", Utc::now().format("%Y%m%d")),
            "csv": operation_log_export_csv(export_rows),
        },
        "permissions": {
            "can_view": true,
            "can_export": true,
        },
    })))
}

async fn salary_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(query): Query<SalaryOverviewQuery>,
) -> Result<Json<SalaryOverview>, ApiError> {
    let can_manage = current_user_can_manage(&pool, &user).await?;
    let stats: SalaryStatsRow = sqlx::query_as(
        r#"
        SELECT
            COALESCE(SUM(COALESCE(base_salary, 0.0) + COALESCE(bonus, 0.0)), 0.0) AS total,
            COUNT(DISTINCT employee_id) AS employees,
            CASE
                WHEN COUNT(*) > 0 THEN COALESCE(SUM(COALESCE(base_salary, 0.0) + COALESCE(bonus, 0.0)), 0.0) / COUNT(*)
                ELSE 0.0
            END AS average,
            COALESCE(SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END), 0) AS pending
        FROM salary_records
        WHERE company_id = ?
        "#,
    )
    .bind(&user.company_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let periods = last_six_periods();
    let mut monthly_trend = Vec::with_capacity(periods.len());
    let mut cost_trend = Vec::with_capacity(periods.len());

    for (period, month) in &periods {
        let total: (f64,) = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(COALESCE(base_salary, 0.0) + COALESCE(bonus, 0.0)), 0.0)
            FROM salary_records
            WHERE company_id = ? AND period = ?
            "#,
        )
        .bind(&user.company_id)
        .bind(period)
        .fetch_one(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

        let costs: SalaryCostTrendRow = sqlx::query_as(
            r#"
            SELECT
                COALESCE(SUM(COALESCE(net_salary, COALESCE(base_salary, 0.0) + COALESCE(bonus, 0.0) - COALESCE(deduction, deductions, 0.0))), 0.0) AS employee_cost,
                COALESCE(SUM(COALESCE(social_security_company, 0.0)), 0.0) AS social_security_cost,
                COALESCE(SUM(COALESCE(housing_fund_company, 0.0)), 0.0) AS housing_fund_cost
            FROM salary_records
            WHERE company_id = ? AND period = ?
            "#,
        )
        .bind(&user.company_id)
        .bind(period)
        .fetch_one(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

        monthly_trend.push(SalaryTrendPoint {
            period: period.clone(),
            month: month.clone(),
            total: total.0,
            total_label: format_cny(total.0),
        });
        let total_cost = costs.employee_cost + costs.social_security_cost + costs.housing_fund_cost;
        cost_trend.push(SalaryCostTrendPoint {
            period: period.clone(),
            month: month.clone(),
            employee_cost: costs.employee_cost,
            employee_cost_label: format_cny(costs.employee_cost),
            social_security_cost: costs.social_security_cost,
            social_security_cost_label: format_cny(costs.social_security_cost),
            housing_fund_cost: costs.housing_fund_cost,
            housing_fund_cost_label: format_cny(costs.housing_fund_cost),
            total_cost,
            total_cost_label: format_cny(total_cost),
        });
    }

    let current_period = Utc::now().format("%Y-%m").to_string();
    let department_rows: Vec<SalaryDepartmentRow> = sqlx::query_as::<_, SalaryDepartmentRow>(
        r#"
        SELECT
            COALESCE(e.department, sr.department, '未分配部门') AS name,
            COALESCE(SUM(
                COALESCE(sr.net_salary, COALESCE(sr.base_salary, 0.0) + COALESCE(sr.bonus, 0.0) - COALESCE(sr.deduction, sr.deductions, 0.0))
                + COALESCE(sr.social_security_company, 0.0)
                + COALESCE(sr.housing_fund_company, 0.0)
            ), 0.0) AS value
        FROM salary_records sr
        LEFT JOIN employees e ON e.id = sr.employee_id AND e.company_id = sr.company_id
        WHERE sr.company_id = ? AND sr.period = ?
        GROUP BY COALESCE(e.department, sr.department, '未分配部门')
        ORDER BY value DESC
        "#,
    )
    .bind(&user.company_id)
    .bind(&current_period)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    let department_total = department_rows.iter().map(|row| row.value).sum::<f64>();
    let department_costs = department_rows
        .into_iter()
        .map(|row| {
            let percent = if department_total.abs() > f64::EPSILON {
                (row.value / department_total) * 100.0
            } else {
                0.0
            };
            SalaryDepartmentCost {
                name: row.name,
                value: row.value,
                value_label: format_cny(row.value),
                percent_label: salary_percent_label(percent),
            }
        })
        .collect::<Vec<_>>();

    let current_cost = cost_trend.last().map(|row| row.total_cost).unwrap_or(0.0);
    let previous_cost = cost_trend
        .iter()
        .rev()
        .nth(1)
        .map(|row| row.total_cost)
        .unwrap_or(0.0);
    let month_over_month = if previous_cost.abs() > f64::EPSILON {
        ((current_cost - previous_cost) / previous_cost) * 100.0
    } else {
        0.0
    };
    let current_employee_cost = cost_trend
        .last()
        .map(|row| row.employee_cost)
        .unwrap_or(0.0);
    let current_company_cost = cost_trend
        .last()
        .map(|row| row.social_security_cost + row.housing_fund_cost)
        .unwrap_or(0.0);

    let salaries =
        salary_rows(&pool, &user.company_id, query.search.as_deref(), can_manage).await?;
    let employees = employee_rows(&pool, &user.company_id).await?;
    let current_insurance_config =
        current_salary_insurance_config(&pool, &user.company_id, can_manage).await?;
    let empty_message = if query
        .search
        .as_deref()
        .is_some_and(|search| !search.trim().is_empty())
    {
        "未找到匹配的记录"
    } else {
        "暂无工资记录"
    }
    .to_string();
    let employee_cost_share = if current_cost.abs() > f64::EPSILON {
        (current_employee_cost / current_cost) * 100.0
    } else {
        0.0
    };

    Ok(Json(SalaryOverview {
        stats: SalaryStats {
            total: stats.total,
            total_label: format_cny(stats.total),
            employees: stats.employees,
            employees_label: stats.employees.to_string(),
            average: stats.average,
            average_label: format_cny(stats.average),
            pending: stats.pending,
            pending_label: stats.pending.to_string(),
        },
        monthly_trend,
        salaries,
        employees,
        current_insurance_config,
        cost_analysis: SalaryCostAnalysis {
            stats: SalaryCostStats {
                total_cost: current_cost,
                total_cost_label: format_cny(current_cost),
                employee_cost: current_employee_cost,
                employee_cost_label: format_cny(current_employee_cost),
                company_cost: current_company_cost,
                company_cost_label: format_cny(current_company_cost),
                departments: department_costs.len() as i64,
                departments_label: department_costs.len().to_string(),
                month_over_month,
                month_over_month_label: format!(
                    "{}{}",
                    if month_over_month >= 0.0 { "+" } else { "" },
                    salary_percent_label(month_over_month)
                ),
                employee_cost_share_label: salary_percent_label(employee_cost_share),
            },
            monthly_trend: cost_trend,
            department_costs,
        },
        create_action: salary_create_action(can_manage),
        insurance_config_create_action: social_insurance_config_create_action(can_manage),
        insurance_preview_action: salary_insurance_preview_action(can_manage),
        permissions: json!({
            "can_manage": can_manage,
            "can_create": can_manage,
            "can_update": can_manage,
            "can_delete": can_manage,
            "can_mark_paid": can_manage,
        }),
        empty_message,
    }))
}

async fn salary_insurance_preview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Json(body): Json<SalaryInsurancePreviewRequest>,
) -> Result<Json<SalaryInsurancePreview>, ApiError> {
    let config: SalaryInsuranceConfigRow = sqlx::query_as(
        r#"
        SELECT
            COALESCE(social_security_base_min, 0.0) AS social_security_base_min,
            COALESCE(social_security_base_max, 0.0) AS social_security_base_max,
            COALESCE(housing_fund_base_min, 0.0) AS housing_fund_base_min,
            COALESCE(housing_fund_base_max, 0.0) AS housing_fund_base_max,
            COALESCE(pension_personal_rate, 0.0) AS pension_personal_rate,
            COALESCE(pension_company_rate, 0.0) AS pension_company_rate,
            COALESCE(medical_personal_rate, 0.0) AS medical_personal_rate,
            COALESCE(medical_company_rate, 0.0) AS medical_company_rate,
            COALESCE(unemployment_personal_rate, 0.0) AS unemployment_personal_rate,
            COALESCE(unemployment_company_rate, 0.0) AS unemployment_company_rate,
            COALESCE(work_injury_company_rate, 0.0) AS work_injury_company_rate,
            COALESCE(maternity_company_rate, 0.0) AS maternity_company_rate,
            COALESCE(housing_fund_personal_rate, 0.0) AS housing_fund_personal_rate,
            COALESCE(housing_fund_company_rate, 0.0) AS housing_fund_company_rate
        FROM social_insurance_configs
        WHERE company_id = ?
        ORDER BY datetime(created_at) DESC, id DESC
        LIMIT 1
        "#,
    )
    .bind(&user.company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?
    .ok_or_else(|| api_err(StatusCode::BAD_REQUEST, "social insurance config not found"))?;

    let request_base = body
        .base_salary
        .as_ref()
        .and_then(json_number)
        .unwrap_or(0.0)
        .max(0.0);
    let mut social_security_base = request_base;
    let mut housing_fund_base = request_base;

    if let Some(employee_id) = body
        .employee_id
        .as_deref()
        .map(str::trim)
        .filter(|employee_id| !employee_id.is_empty())
    {
        let employee: SalaryEmployeeBaseRow = sqlx::query_as(
            r#"
            SELECT
                COALESCE(base_salary, 0.0) AS base_salary,
                COALESCE(social_security_base, 0.0) AS social_security_base,
                COALESCE(housing_fund_base, 0.0) AS housing_fund_base
            FROM employees
            WHERE company_id = ? AND id = ?
            "#,
        )
        .bind(&user.company_id)
        .bind(employee_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?
        .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "employee not found"))?;

        if employee.social_security_base > 0.0 {
            social_security_base = employee.social_security_base;
        } else if request_base <= 0.0 {
            social_security_base = employee.base_salary;
        }

        if employee.housing_fund_base > 0.0 {
            housing_fund_base = employee.housing_fund_base;
        } else if request_base <= 0.0 {
            housing_fund_base = employee.base_salary;
        }
    }

    let social_security_base = clamp_salary_base(
        social_security_base,
        config.social_security_base_min,
        config.social_security_base_max,
    );
    let housing_fund_base = clamp_salary_base(
        housing_fund_base,
        config.housing_fund_base_min,
        config.housing_fund_base_max,
    );

    let personal_social_rate = config.pension_personal_rate
        + config.medical_personal_rate
        + config.unemployment_personal_rate;
    let company_social_rate = config.pension_company_rate
        + config.medical_company_rate
        + config.unemployment_company_rate
        + config.work_injury_company_rate
        + config.maternity_company_rate;

    let social_security = round_money(social_security_base * personal_social_rate / 100.0);
    let social_security_company = round_money(social_security_base * company_social_rate / 100.0);
    let housing_fund = round_money(housing_fund_base * config.housing_fund_personal_rate / 100.0);
    let housing_fund_company =
        round_money(housing_fund_base * config.housing_fund_company_rate / 100.0);

    Ok(Json(SalaryInsurancePreview {
        social_security_base: round_money(social_security_base),
        housing_fund_base: round_money(housing_fund_base),
        social_security,
        social_security_company,
        housing_fund,
        housing_fund_company,
        social_security_input: salary_amount_input(social_security),
        housing_fund_input: salary_amount_input(housing_fund),
        social_security_company_input: salary_amount_input(social_security_company),
        housing_fund_company_input: salary_amount_input(housing_fund_company),
    }))
}

async fn mark_salary_paid(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let can_manage = current_user_can_manage(&pool, &user).await?;
    if !can_manage {
        return Err(api_err(StatusCode::FORBIDDEN, "forbidden"));
    }

    let payment_date = Utc::now().format("%Y-%m-%d").to_string();
    let result = sqlx::query(
        r#"
        UPDATE salary_records
        SET status = 'paid',
            payment_date = COALESCE(NULLIF(payment_date, ''), ?),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ? AND company_id = ?
        "#,
    )
    .bind(&payment_date)
    .bind(&id)
    .bind(&user.company_id)
    .execute(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    if result.rows_affected() == 0 {
        return Err(api_err(StatusCode::NOT_FOUND, "salary not found"));
    }

    let row = sqlx::query(
        r#"
        SELECT id, status, payment_date, updated_at
        FROM salary_records
        WHERE id = ? AND company_id = ?
        LIMIT 1
        "#,
    )
    .bind(&id)
    .bind(&user.company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?
    .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "salary not found"))?;

    let status = text(&row, "status").unwrap_or_else(|| "paid".to_string());
    Ok(Json(json!({
        "id": text(&row, "id"),
        "status": status.clone(),
        "payment_date": text(&row, "payment_date"),
        "status_badge": salary_status_badge(&status),
        "can_mark_paid": can_manage && status == "pending",
        "mark_paid_action": salary_mark_paid_action(
            text(&row, "id"),
            &status,
            can_manage,
        ),
        "updated_at": text(&row, "updated_at"),
    })))
}

fn json_number(value: &Value) -> Option<f64> {
    value.as_f64().or_else(|| {
        value
            .as_str()
            .and_then(|raw| raw.trim().parse::<f64>().ok())
    })
}

fn salary_amount_input(value: f64) -> String {
    format!("{:.2}", round_money(value))
}

async fn current_salary_insurance_config(
    pool: &SqlitePool,
    company_id: &str,
    can_manage: bool,
) -> Result<Option<Value>, ApiError> {
    let config = sqlx::query_as::<_, SalaryInsuranceConfigOverviewRow>(
        r#"
        SELECT
            id,
            COALESCE(city, '') AS city,
            COALESCE(social_security_base_min, 0.0) AS social_security_base_min,
            COALESCE(social_security_base_max, 0.0) AS social_security_base_max,
            COALESCE(housing_fund_base_min, 0.0) AS housing_fund_base_min,
            COALESCE(housing_fund_base_max, 0.0) AS housing_fund_base_max,
            COALESCE(pension_personal_rate, 0.0) AS pension_personal_rate,
            COALESCE(pension_company_rate, 0.0) AS pension_company_rate,
            COALESCE(medical_personal_rate, 0.0) AS medical_personal_rate,
            COALESCE(medical_company_rate, 0.0) AS medical_company_rate,
            COALESCE(unemployment_personal_rate, 0.0) AS unemployment_personal_rate,
            COALESCE(unemployment_company_rate, 0.0) AS unemployment_company_rate,
            COALESCE(work_injury_company_rate, 0.0) AS work_injury_company_rate,
            COALESCE(maternity_company_rate, 0.0) AS maternity_company_rate,
            COALESCE(housing_fund_personal_rate, 0.0) AS housing_fund_personal_rate,
            COALESCE(housing_fund_company_rate, 0.0) AS housing_fund_company_rate,
            COALESCE(notes, '') AS notes
        FROM social_insurance_configs
        WHERE company_id = ?
        ORDER BY datetime(created_at) DESC, id DESC
        LIMIT 1
        "#,
    )
    .bind(company_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(config.map(|config| {
        let id = config.id.clone();
        json!({
            "id": config.id,
            "city": config.city,
            "social_security_base_min": config.social_security_base_min,
            "social_security_base_min_input": salary_amount_input(config.social_security_base_min),
            "social_security_base_max": config.social_security_base_max,
            "social_security_base_max_input": salary_amount_input(config.social_security_base_max),
            "housing_fund_base_min": config.housing_fund_base_min,
            "housing_fund_base_min_input": salary_amount_input(config.housing_fund_base_min),
            "housing_fund_base_max": config.housing_fund_base_max,
            "housing_fund_base_max_input": salary_amount_input(config.housing_fund_base_max),
            "pension_personal_rate": config.pension_personal_rate,
            "pension_personal_rate_input": salary_amount_input(config.pension_personal_rate),
            "pension_company_rate": config.pension_company_rate,
            "pension_company_rate_input": salary_amount_input(config.pension_company_rate),
            "medical_personal_rate": config.medical_personal_rate,
            "medical_personal_rate_input": salary_amount_input(config.medical_personal_rate),
            "medical_company_rate": config.medical_company_rate,
            "medical_company_rate_input": salary_amount_input(config.medical_company_rate),
            "unemployment_personal_rate": config.unemployment_personal_rate,
            "unemployment_personal_rate_input": salary_amount_input(config.unemployment_personal_rate),
            "unemployment_company_rate": config.unemployment_company_rate,
            "unemployment_company_rate_input": salary_amount_input(config.unemployment_company_rate),
            "work_injury_company_rate": config.work_injury_company_rate,
            "work_injury_company_rate_input": salary_amount_input(config.work_injury_company_rate),
            "maternity_company_rate": config.maternity_company_rate,
            "maternity_company_rate_input": salary_amount_input(config.maternity_company_rate),
            "housing_fund_personal_rate": config.housing_fund_personal_rate,
            "housing_fund_personal_rate_input": salary_amount_input(config.housing_fund_personal_rate),
            "housing_fund_company_rate": config.housing_fund_company_rate,
            "housing_fund_company_rate_input": salary_amount_input(config.housing_fund_company_rate),
            "notes": config.notes,
            "update_action": social_insurance_config_update_action(Some(id), can_manage),
        })
    }))
}

fn social_insurance_config_create_action(can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    operations_create_action(
        "/api/v1/social_insurance_config".to_string(),
        "保存社保配置",
        "社保公积金配置已保存",
    )
}

fn salary_insurance_preview_action(can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    json!({
        "method": "POST",
        "endpoint": "/api/v1/salary/insurance-preview",
        "label": "自动计算",
        "success_message": "社保公积金已计算"
    })
}

fn social_insurance_config_update_action(id: Option<String>, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };
    operations_update_action(
        format!("/api/v1/social_insurance_config/{id}"),
        "保存社保配置",
        "社保公积金配置已保存",
    )
}

async fn salary_rows(
    pool: &SqlitePool,
    company_id: &str,
    search: Option<&str>,
    can_manage: bool,
) -> Result<Vec<Value>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT
            sr.*,
            COALESCE(e.name, sr.employee_name, sr.employee_id) AS display_employee_name,
            COALESCE(e.department, sr.department, '') AS display_department,
            COALESCE(e.position, sr.position, '') AS display_position,
            COALESCE(sr.base_salary, 0.0) + COALESCE(sr.bonus, 0.0) AS total_salary,
            COALESCE(sr.net_salary, COALESCE(sr.base_salary, 0.0) + COALESCE(sr.bonus, 0.0) - COALESCE(sr.deduction, sr.deductions, 0.0)) AS actual_salary
        FROM salary_records sr
        LEFT JOIN employees e ON e.id = sr.employee_id AND e.company_id = sr.company_id
        WHERE sr.company_id = ?
        ORDER BY sr.period DESC, sr.created_at DESC
        "#,
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let search = search
        .map(str::trim)
        .filter(|search| !search.is_empty())
        .map(|search| search.to_lowercase());

    Ok(rows
        .into_iter()
        .filter(|row| {
            let Some(search) = search.as_ref() else {
                return true;
            };
            [
                text(row, "display_employee_name"),
                text(row, "display_department"),
                text(row, "display_position"),
                text(row, "period"),
            ]
            .into_iter()
            .flatten()
            .any(|value| value.to_lowercase().contains(search))
        })
        .map(|row| {
            let base_salary = number(&row, "base_salary");
            let bonus = number(&row, "bonus");
            let deduction = number(&row, "deduction");
            let social_security = number(&row, "social_security");
            let social_security_company = number(&row, "social_security_company");
            let housing_fund = number(&row, "housing_fund");
            let housing_fund_company = number(&row, "housing_fund_company");
            let actual_salary = number(&row, "actual_salary");
            let status = text(&row, "status").unwrap_or_else(|| "pending".to_string());
            let id = text(&row, "id");
            let employee_name = text(&row, "display_employee_name");
            let period = text(&row, "period");
            let delete_action = salary_delete_action(
                id.clone(),
                employee_name.as_deref(),
                period.as_deref(),
                can_manage,
            );
            let update_action = salary_update_action(id.clone(), can_manage);
            json!({
                "id": id,
                "employee_id": text(&row, "employee_id"),
                "employee_name": employee_name,
                "department": text(&row, "display_department"),
                "position": text(&row, "display_position"),
                "period": period,
                "base_salary": base_salary,
                "base_salary_label": format_cny(base_salary),
                "bonus": bonus,
                "bonus_label": salary_signed_cny(bonus, "+"),
                "deduction": deduction,
                "deduction_label": salary_signed_cny(deduction, "-"),
                "deductions": number(&row, "deductions"),
                "social_security": social_security,
                "social_security_label": format_cny(social_security),
                "social_security_company": social_security_company,
                "social_security_company_label": format_cny(social_security_company),
                "housing_fund": housing_fund,
                "housing_fund_label": format_cny(housing_fund),
                "housing_fund_company": housing_fund_company,
                "housing_fund_company_label": format_cny(housing_fund_company),
                "net_salary": number(&row, "net_salary"),
                "total_salary": number(&row, "total_salary"),
                "actual_salary": actual_salary,
                "actual_salary_label": format_cny(actual_salary),
                "status": status,
                "status_badge": salary_status_badge(&status),
                "payment_date": text(&row, "payment_date"),
                "notes": text(&row, "notes"),
                "can_mark_paid": can_manage && status == "pending",
                "mark_paid_action": salary_mark_paid_action(
                    id.clone(),
                    &status,
                    can_manage,
                ),
                "update_action": update_action,
                "can_delete": can_manage,
                "delete_action": delete_action,
            })
        })
        .collect())
}

fn salary_create_action(can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }

    operations_create_action("/api/v1/salary".to_string(), "录入工资", "工资记录已添加")
}

fn salary_update_action(id: Option<String>, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };

    operations_update_action(format!("/api/v1/salary/{id}"), "保存修改", "工资记录已更新")
}

fn salary_mark_paid_action(id: Option<String>, status: &str, can_manage: bool) -> Value {
    if !can_manage || status != "pending" {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };

    operations_post_action(
        format!("/api/v1/salary/{id}/mark_paid"),
        "标记已发放",
        "工资已标记为已发放",
        "paid",
    )
}

fn salary_delete_action(
    id: Option<String>,
    employee_name: Option<&str>,
    period: Option<&str>,
    can_manage: bool,
) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };
    let employee_name = employee_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("该员工");
    let period = period
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("该期间");
    operations_delete_action(
        format!("/api/v1/salary/{id}"),
        format!("确定要删除工资记录「{employee_name} {period}」吗？"),
        "工资记录已删除",
    )
}

async fn employee_rows(pool: &SqlitePool, company_id: &str) -> Result<Vec<Value>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, department, position, base_salary, social_security_base, housing_fund_base
        FROM employees
        WHERE company_id = ?
        ORDER BY name ASC
        "#,
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            json!({
                "id": text(&row, "id"),
                "name": text(&row, "name"),
                "department": text(&row, "department"),
                "position": text(&row, "position"),
                "base_salary": number(&row, "base_salary"),
                "social_security_base": number(&row, "social_security_base"),
                "housing_fund_base": number(&row, "housing_fund_base"),
            })
        })
        .collect())
}

async fn time_entries_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<TimeEntriesOverview>, ApiError> {
    let can_delete = current_user_can_manage(&pool, &user).await?;
    let row: TimeEntriesOverviewRow = sqlx::query_as(
        r#"
        SELECT
            COALESCE(SUM(CASE WHEN date = date('now') THEN COALESCE(duration_minutes, 0) ELSE 0 END), 0) AS today_minutes,
            COUNT(*) AS total_records,
            COALESCE(SUM(CASE WHEN COALESCE(is_billable, 0) = 1 THEN COALESCE(duration_minutes, 0) ELSE 0 END), 0) AS billable_minutes,
            COALESCE(SUM(CASE WHEN COALESCE(is_billable, 0) = 1 THEN COALESCE(amount, 0.0) ELSE 0.0 END), 0.0) AS billable_amount,
            COALESCE(SUM(CASE
                WHEN COALESCE(is_billable, 0) = 1
                 AND COALESCE(status, 'logged') != 'invoiced'
                 AND (invoice_id IS NULL OR invoice_id = '')
                THEN 1 ELSE 0 END), 0) AS unbilled_count
        FROM time_entries
        WHERE company_id = ?
        "#,
    )
    .bind(&user.company_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let entry_rows = sqlx::query(
        r#"
        SELECT
            t.id, t.description, t.project_id, t.client_id, t.date,
            t.duration_minutes, t.is_billable, t.hourly_rate, t.amount, t.status, t.invoice_id,
            p.name AS project_name,
            c.name AS client_name
        FROM time_entries t
        LEFT JOIN projects p ON p.id = t.project_id AND p.company_id = t.company_id
        LEFT JOIN clients c ON c.id = t.client_id AND c.company_id = t.company_id
        WHERE t.company_id = ?
        ORDER BY COALESCE(t.date, '') DESC, t.created_at DESC, t.id DESC
        LIMIT 200
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let entries = entry_rows
        .iter()
        .map(|entry| {
            let id = text(entry, "id");
            let description = text(entry, "description");
            let duration_minutes = integer(entry, "duration_minutes");
            let is_billable = integer(entry, "is_billable") != 0;
            let amount = number(entry, "amount");
            let status = text(entry, "status").unwrap_or_else(|| "logged".to_string());
            let invoice_id = text(entry, "invoice_id");
            let is_invoiced = status == "invoiced"
                || invoice_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .is_some();
            let delete_action =
                time_entry_delete_action(id.clone(), description.clone(), can_delete);
            json!({
                "id": id.unwrap_or_default(),
                "description": description.unwrap_or_default(),
                "project_id": text(entry, "project_id"),
                "project_name": text(entry, "project_name").unwrap_or_else(|| "-".to_string()),
                "client_id": text(entry, "client_id"),
                "client_name": text(entry, "client_name").unwrap_or_else(|| "-".to_string()),
                "date": text(entry, "date"),
                "date_label": text(entry, "date").unwrap_or_else(|| "-".to_string()),
                "duration_minutes": duration_minutes,
                "duration_label": time_minutes_label(duration_minutes),
                "is_billable": is_billable,
                "billable_badge": time_billable_badge(is_billable, is_invoiced),
                "hourly_rate": number(entry, "hourly_rate"),
                "amount": amount,
                "amount_label": if amount.abs() < 0.005 { "-".to_string() } else { format_cny(amount) },
                "status": status,
                "status_badge": time_status_badge(&status),
                "can_delete": can_delete,
                "delete_action": delete_action,
            })
        })
        .collect::<Vec<_>>();

    let project_rows = sqlx::query(
        r#"
        SELECT id, name
        FROM projects
        WHERE company_id = ?
        ORDER BY name ASC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    let project_options = project_rows
        .iter()
        .map(|project| {
            let name = text(project, "name").unwrap_or_default();
            json!({
                "value": text(project, "id").unwrap_or_default(),
                "label": name,
                "name": name,
            })
        })
        .collect::<Vec<_>>();

    let client_rows = sqlx::query(
        r#"
        SELECT id, name
        FROM clients
        WHERE company_id = ?
        ORDER BY name ASC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    let client_options = client_rows
        .iter()
        .map(|client| {
            let name = text(client, "name").unwrap_or_default();
            json!({
                "value": text(client, "id").unwrap_or_default(),
                "label": name,
                "name": name,
            })
        })
        .collect::<Vec<_>>();
    let active_timer =
        active_time_entry_timer_overview(&pool, &user.company_id, &user.user_id).await?;

    Ok(Json(TimeEntriesOverview {
        today_minutes: row.today_minutes,
        total_records: row.total_records,
        billable_minutes: row.billable_minutes,
        billable_amount: row.billable_amount,
        unbilled_count: row.unbilled_count,
        stats: json!({
            "today_minutes": row.today_minutes,
            "today_minutes_label": time_minutes_label(row.today_minutes),
            "total_records": row.total_records,
            "total_records_label": row.total_records.to_string(),
            "billable_minutes": row.billable_minutes,
            "billable_minutes_label": time_minutes_label(row.billable_minutes),
            "billable_amount": row.billable_amount,
            "billable_amount_label": format_cny(row.billable_amount),
            "unbilled_count": row.unbilled_count,
            "unbilled_count_label": row.unbilled_count.to_string(),
        }),
        entries,
        project_options,
        client_options,
        billable_options: time_billable_options(),
        active_timer,
        invoice_action: json!({
            "can_generate": row.unbilled_count > 0,
            "label": format!("生成发票 ({})", row.unbilled_count),
            "confirm_message": format!("确定将 {} 条未开票工时转为发票？", row.unbilled_count),
            "action": time_invoice_action(row.unbilled_count),
        }),
        permissions: json!({
            "can_delete": can_delete,
        }),
        create_action: time_entry_create_action(can_delete),
        empty_message: "暂无工时记录，点击上方开始计时或手动记录".to_string(),
        today: Utc::now().format("%Y-%m-%d").to_string(),
    }))
}

async fn active_time_entry_timer_overview(
    pool: &SqlitePool,
    company_id: &str,
    user_id: &str,
) -> Result<Value, ApiError> {
    let row = sqlx::query(
        r#"
        SELECT
            t.id, t.description, t.project_id, t.client_id, t.start_time,
            p.name AS project_name,
            c.name AS client_name
        FROM time_entries t
        LEFT JOIN projects p ON p.id = t.project_id AND p.company_id = t.company_id
        LEFT JOIN clients c ON c.id = t.client_id AND c.company_id = t.company_id
        WHERE t.company_id = ? AND t.created_by = ? AND t.status = 'running'
        ORDER BY t.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(company_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let Some(row) = row else {
        return Ok(json!({
            "is_running": false,
            "elapsed_seconds": 0,
            "elapsed_label": "0m",
            "start_action": operations_post_action(
                "/api/v1/time_entries/timer/start".to_string(),
                "开始",
                "计时已开始",
                "running",
            ),
            "stop_action": json!(null),
        }));
    };
    let start_time = text(&row, "start_time").unwrap_or_default();
    let started_at = DateTime::parse_from_rfc3339(&start_time)
        .map(|value| value.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());
    let elapsed_seconds = Utc::now()
        .signed_duration_since(started_at)
        .num_seconds()
        .max(0);

    Ok(json!({
        "is_running": true,
        "id": text(&row, "id").unwrap_or_default(),
        "description": text(&row, "description").unwrap_or_default(),
        "project_id": text(&row, "project_id"),
        "project_name": text(&row, "project_name").unwrap_or_else(|| "-".to_string()),
        "client_id": text(&row, "client_id"),
        "client_name": text(&row, "client_name").unwrap_or_else(|| "-".to_string()),
        "started_at": start_time,
        "started_at_label": started_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        "elapsed_seconds": elapsed_seconds,
        "elapsed_label": time_minutes_label(elapsed_seconds / 60),
        "start_action": json!(null),
        "stop_action": operations_post_action(
            "/api/v1/time_entries/timer/stop".to_string(),
            "停止",
            "工时已记录",
            "logged",
        ),
    }))
}

fn time_invoice_action(unbilled_count: i64) -> Value {
    if unbilled_count <= 0 {
        return json!(null);
    }

    operations_post_action(
        "/api/v1/time_entries/invoice".to_string(),
        format!("生成发票 ({unbilled_count})"),
        "工时发票已生成",
        "invoiced",
    )
}

async fn reconciliation_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<Value>, ApiError> {
    let bank_rows = sqlx::query(
        r#"
        SELECT
            bt.id, bt.transaction_date, bt.transaction_type, bt.amount,
            bt.counterparty, bt.description, ba.account_name, ba.bank_name
        FROM bank_transactions bt
        LEFT JOIN bank_accounts ba ON ba.id = bt.account_id AND ba.company_id = bt.company_id
        WHERE bt.company_id = ?
          AND (bt.payment_id IS NULL OR bt.payment_id = '')
        ORDER BY bt.transaction_date DESC, bt.created_at DESC
        LIMIT 100
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    let bank_items = bank_rows
        .iter()
        .map(|row| {
            let transaction_type = text(row, "transaction_type").unwrap_or_default();
            let (type_label, type_variant) =
                reconciliation_transaction_type_badge(&transaction_type);
            let amount_class = if type_label == "收入" {
                "text-green-600"
            } else {
                "text-red-600"
            };
            json!({
                "id": text(row, "id").unwrap_or_default(),
                "cells": [
                    reconciliation_text_cell("transaction_date", reconciliation_date_label(text(row, "transaction_date"))),
                    reconciliation_text_cell("bank_account", format!(
                        "{} {}",
                        text(row, "account_name").unwrap_or_else(|| "-".to_string()),
                        text(row, "bank_name").unwrap_or_default()
                    ).trim().to_string()),
                    reconciliation_badge_cell("transaction_type", type_label, type_variant),
                    reconciliation_text_cell("counterparty", text(row, "counterparty").unwrap_or_else(|| "-".to_string())),
                    reconciliation_amount_cell("amount", number(row, "amount"), amount_class),
                    reconciliation_text_cell("description", text(row, "description").unwrap_or_else(|| "-".to_string())),
                ],
                "action": reconciliation_action(),
            })
        })
        .collect::<Vec<_>>();

    let payment_rows = sqlx::query(
        r#"
        SELECT id, payment_date, payment_no, payer, payee, amount, payment_type
        FROM payments p
        WHERE p.company_id = ?
          AND NOT EXISTS (
              SELECT 1 FROM bank_transactions bt
              WHERE bt.company_id = p.company_id AND bt.payment_id = p.id
          )
        ORDER BY COALESCE(payment_date, created_at) DESC, created_at DESC
        LIMIT 100
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    let payment_items = payment_rows
        .iter()
        .map(|row| {
            let payment_type = text(row, "payment_type").unwrap_or_default();
            let (type_label, type_variant) = reconciliation_payment_type_badge(&payment_type);
            json!({
                "id": text(row, "id").unwrap_or_default(),
                "cells": [
                    reconciliation_text_cell("payment_date", reconciliation_date_label(text(row, "payment_date"))),
                    reconciliation_text_cell("payment_no", text(row, "payment_no").unwrap_or_else(|| "-".to_string())),
                    reconciliation_text_cell("payer", text(row, "payer").unwrap_or_else(|| "-".to_string())),
                    reconciliation_text_cell("payee", text(row, "payee").unwrap_or_else(|| "-".to_string())),
                    reconciliation_amount_cell("amount", number(row, "amount"), "text-foreground font-medium"),
                    reconciliation_badge_cell("payment_type", type_label, type_variant),
                ],
                "action": reconciliation_action(),
            })
        })
        .collect::<Vec<_>>();

    let invoice_rows = sqlx::query(
        r#"
        SELECT id, invoice_date, invoice_number, description, supplier, amount, invoice_type
        FROM invoices i
        WHERE i.company_id = ?
          AND NOT EXISTS (
              SELECT 1 FROM payments p
              WHERE p.company_id = i.company_id AND p.invoice_id = i.id
          )
        ORDER BY COALESCE(invoice_date, created_at) DESC, created_at DESC
        LIMIT 100
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    let invoice_items = invoice_rows
        .iter()
        .map(|row| {
            let invoice_type = text(row, "invoice_type").unwrap_or_default();
            let (type_label, type_variant) = reconciliation_invoice_type_badge(&invoice_type);
            json!({
                "id": text(row, "id").unwrap_or_default(),
                "cells": [
                    reconciliation_text_cell("invoice_date", reconciliation_date_label(text(row, "invoice_date"))),
                    reconciliation_text_cell("invoice_number", text(row, "invoice_number").unwrap_or_else(|| "-".to_string())),
                    reconciliation_text_cell("title", text(row, "description").unwrap_or_else(|| "-".to_string())),
                    reconciliation_text_cell("supplier", text(row, "supplier").unwrap_or_else(|| "-".to_string())),
                    reconciliation_amount_cell("amount", number(row, "amount"), "text-foreground font-medium"),
                    reconciliation_badge_cell("invoice_type", &type_label, type_variant),
                ],
                "action": reconciliation_action(),
            })
        })
        .collect::<Vec<_>>();

    let salary_rows = sqlx::query(
        r#"
        SELECT id, payment_date, employee_name, department, period, net_salary
        FROM salary_records
        WHERE company_id = ?
          AND (payment_id IS NULL OR payment_id = '')
        ORDER BY COALESCE(payment_date, created_at) DESC, created_at DESC
        LIMIT 100
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    let salary_items = salary_rows
        .iter()
        .map(|row| {
            json!({
                "id": text(row, "id").unwrap_or_default(),
                "cells": [
                    reconciliation_text_cell("payment_date", reconciliation_date_label(text(row, "payment_date"))),
                    reconciliation_text_cell("employee_name", text(row, "employee_name").unwrap_or_else(|| "-".to_string())),
                    reconciliation_text_cell("department", text(row, "department").unwrap_or_else(|| "-".to_string())),
                    reconciliation_text_cell("period", text(row, "period").unwrap_or_else(|| "-".to_string())),
                    reconciliation_amount_cell("amount", number(row, "net_salary"), "text-foreground font-medium"),
                ],
                "action": reconciliation_action(),
            })
        })
        .collect::<Vec<_>>();

    let rent_rows = sqlx::query(
        r#"
        SELECT id, next_payment_date, location_name, payment_cycle, monthly_rent
        FROM rent_records
        WHERE company_id = ? AND COALESCE(status, 'active') = 'active'
        ORDER BY COALESCE(next_payment_date, created_at) DESC, created_at DESC
        LIMIT 100
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    let rent_items = rent_rows
        .iter()
        .map(|row| {
            json!({
                "id": text(row, "id").unwrap_or_default(),
                "cells": [
                    reconciliation_text_cell("payment_date", reconciliation_date_label(text(row, "next_payment_date"))),
                    reconciliation_text_cell("location_name", text(row, "location_name").unwrap_or_else(|| "-".to_string())),
                    reconciliation_text_cell("period", text(row, "payment_cycle").unwrap_or_else(|| "-".to_string())),
                    reconciliation_amount_cell("amount", number(row, "monthly_rent"), "text-foreground font-medium"),
                ],
                "action": reconciliation_action(),
            })
        })
        .collect::<Vec<_>>();

    let cloud_rows = sqlx::query(
        r#"
        SELECT cb.id, cb.billing_period, cb.total_amount, cb.status, cp.provider_name
        FROM cloud_bills cb
        LEFT JOIN cloud_providers cp ON cp.id = cb.provider_id AND cp.company_id = cb.company_id
        WHERE cb.company_id = ?
          AND (cb.payment_id IS NULL OR cb.payment_id = '')
        ORDER BY COALESCE(cb.billing_period, cb.created_at) DESC, cb.created_at DESC
        LIMIT 100
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();
    let cloud_items = cloud_rows
        .iter()
        .map(|row| {
            json!({
                "id": text(row, "id").unwrap_or_default(),
                "cells": [
                    reconciliation_text_cell("payment_date", reconciliation_date_label(None)),
                    reconciliation_text_cell("provider_name", text(row, "provider_name").unwrap_or_else(|| "-".to_string())),
                    reconciliation_text_cell("item_name", text(row, "status").unwrap_or_else(|| "-".to_string())),
                    reconciliation_text_cell("billing_period", text(row, "billing_period").unwrap_or_else(|| "-".to_string())),
                    reconciliation_amount_cell("amount", number(row, "total_amount"), "text-foreground font-medium"),
                ],
                "action": reconciliation_action(),
            })
        })
        .collect::<Vec<_>>();

    let software_rows = sqlx::query(
        r#"
        SELECT id, software_name, vendor, billing_cycle, next_billing_date, total_price
        FROM software_subscriptions
        WHERE company_id = ?
          AND (payment_id IS NULL OR payment_id = '')
        ORDER BY COALESCE(next_billing_date, created_at) DESC, created_at DESC
        LIMIT 100
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();
    let software_items = software_rows
        .iter()
        .map(|row| {
            let billing_cycle = text(row, "billing_cycle").unwrap_or_default();
            let (cycle_label, cycle_variant) = reconciliation_billing_cycle_badge(&billing_cycle);
            json!({
                "id": text(row, "id").unwrap_or_default(),
                "cells": [
                    reconciliation_text_cell("software_name", text(row, "software_name").unwrap_or_else(|| "-".to_string())),
                    reconciliation_text_cell("vendor", text(row, "vendor").unwrap_or_else(|| "-".to_string())),
                    reconciliation_badge_cell("billing_cycle", cycle_label, cycle_variant),
                    reconciliation_text_cell("next_billing_date", reconciliation_date_label(text(row, "next_billing_date"))),
                    reconciliation_amount_cell("amount", number(row, "total_price"), "text-foreground font-medium"),
                ],
                "action": reconciliation_action(),
            })
        })
        .collect::<Vec<_>>();

    let matched_rows = sqlx::query(
        r#"
        SELECT
            bt.id, bt.transaction_date, bt.transaction_type, bt.amount,
            ba.account_name, ba.bank_name, p.payment_no, i.invoice_number
        FROM bank_transactions bt
        LEFT JOIN bank_accounts ba ON ba.id = bt.account_id AND ba.company_id = bt.company_id
        LEFT JOIN payments p ON p.id = bt.payment_id AND p.company_id = bt.company_id
        LEFT JOIN invoices i ON i.id = p.invoice_id AND i.company_id = p.company_id
        WHERE bt.company_id = ?
          AND bt.payment_id IS NOT NULL AND bt.payment_id != ''
        ORDER BY bt.transaction_date DESC, bt.created_at DESC
        LIMIT 100
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    let matched_items = matched_rows
        .iter()
        .map(|row| {
            let transaction_type = text(row, "transaction_type").unwrap_or_default();
            let amount_class =
                if matches!(transaction_type.as_str(), "credit" | "deposit" | "income") {
                    "text-green-600"
                } else {
                    "text-red-600"
                };
            json!({
                "id": text(row, "id").unwrap_or_default(),
                "cells": [
                    reconciliation_text_cell("transaction_date", reconciliation_date_label(text(row, "transaction_date"))),
                    reconciliation_text_cell("bank_account", format!(
                        "{} {}",
                        text(row, "account_name").unwrap_or_else(|| "-".to_string()),
                        text(row, "bank_name").unwrap_or_default()
                    ).trim().to_string()),
                    reconciliation_amount_cell("amount", number(row, "amount"), amount_class),
                    reconciliation_badge_cell("payment_no", &text(row, "payment_no").unwrap_or_else(|| "-".to_string()), "outline"),
                    reconciliation_badge_cell("invoice_number", &text(row, "invoice_number").unwrap_or_else(|| "-".to_string()), "outline"),
                    reconciliation_badge_cell("status", "已匹配", "default"),
                ],
            })
        })
        .collect::<Vec<_>>();

    let unmatched_sections = vec![
        reconciliation_section(
            "bank_transactions",
            "未匹配银行流水",
            vec![
                "交易日期",
                "银行账户",
                "交易类型",
                "交易对手",
                "金额",
                "描述",
                "操作",
            ],
            bank_items,
        ),
        reconciliation_section(
            "payments",
            "未匹配支付记录",
            vec![
                "支付日期",
                "支付编号",
                "付款方",
                "收款方",
                "金额",
                "类型",
                "操作",
            ],
            payment_items,
        ),
        reconciliation_section(
            "invoices",
            "未匹配发票",
            vec![
                "开票日期",
                "发票号",
                "标题",
                "供应商",
                "金额",
                "类型",
                "操作",
            ],
            invoice_items,
        ),
        reconciliation_section(
            "salaries",
            "未匹配工资记录",
            vec!["发放日期", "员工姓名", "部门", "期间", "金额", "操作"],
            salary_items,
        ),
        reconciliation_section(
            "rent_payments",
            "未匹配租金支付",
            vec!["支付日期", "场地名称", "期间", "金额", "操作"],
            rent_items,
        ),
        reconciliation_section(
            "cloud_bills",
            "未匹配云服务账单",
            vec!["支付日期", "服务商", "账单项目", "账单月份", "金额", "操作"],
            cloud_items,
        ),
        reconciliation_section(
            "software_subscriptions",
            "未匹配软件订阅",
            vec![
                "软件名称",
                "供应商",
                "订阅类型",
                "下次扣费日期",
                "金额",
                "操作",
            ],
            software_items,
        ),
    ];

    let bank_count = unmatched_sections[0]["rows"].as_array().map_or(0, Vec::len);
    let payment_count = unmatched_sections[1]["rows"].as_array().map_or(0, Vec::len);
    let invoice_count = unmatched_sections[2]["rows"].as_array().map_or(0, Vec::len);
    let salary_count = unmatched_sections[3]["rows"].as_array().map_or(0, Vec::len);
    let rent_count = unmatched_sections[4]["rows"].as_array().map_or(0, Vec::len);
    let cloud_count = unmatched_sections[5]["rows"].as_array().map_or(0, Vec::len);
    let software_count = unmatched_sections[6]["rows"].as_array().map_or(0, Vec::len);
    let bill_count = rent_count + cloud_count + software_count;
    let unmatched_total = bank_count + payment_count + invoice_count + salary_count + bill_count;

    let visible_unmatched_sections = unmatched_sections
        .into_iter()
        .filter(|section| {
            section["rows"]
                .as_array()
                .is_some_and(|rows| !rows.is_empty())
        })
        .collect::<Vec<_>>();

    Ok(Json(json!({
        "title": "财务对账中心",
        "subtitle": "银行账单、支付记录、发票信息三方对账核验",
        "tabs": [
            { "value": "unmatched", "label": "待匹配项目" },
            { "value": "matched", "label": "已匹配记录" }
        ],
        "stats": [
            reconciliation_stat("unmatched_total", "待匹配项目", unmatched_total, "yellow"),
            reconciliation_stat("bank_transactions", "未匹配银行流水", bank_count, "red"),
            reconciliation_stat("payments", "未匹配支付", payment_count, "orange"),
            reconciliation_stat("invoices", "未匹配发票", invoice_count, "blue"),
            reconciliation_stat("salaries", "未匹配工资", salary_count, "purple"),
            reconciliation_stat("bills", "未匹配租金/账单", bill_count, "green")
        ],
        "unmatched_sections": visible_unmatched_sections,
        "matched_section": reconciliation_section(
            "matched",
            "已匹配记录",
            vec!["交易日期", "银行账户", "金额", "关联支付", "关联发票", "状态"],
            matched_items,
        ),
        "empty_messages": {
            "unmatched": "暂无待匹配项目",
            "matched": "暂无已匹配记录"
        }
    })))
}

fn reconciliation_stat(key: &str, label: &str, value: usize, tone: &str) -> Value {
    json!({
        "key": key,
        "label": label,
        "value": value,
        "tone": tone,
        "class_name": reconciliation_stat_class_name(tone),
    })
}

fn reconciliation_stat_class_name(tone: &str) -> &'static str {
    match tone {
        "red" => "bg-red-500/10 text-red-600",
        "orange" => "bg-orange-500/10 text-orange-600",
        "blue" => "bg-blue-500/10 text-blue-600",
        "purple" => "bg-purple-500/10 text-purple-600",
        "green" => "bg-green-500/10 text-green-600",
        _ => "bg-yellow-500/10 text-yellow-600",
    }
}

async fn payments_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(query): Query<PaymentsOverviewQuery>,
) -> Result<Json<PaymentsOverview>, ApiError> {
    let stats: PaymentStatsRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) AS total,
            COALESCE(SUM(CASE WHEN status IN ('completed', 'paid') AND payment_type IN ('支出', 'expense', 'salary') THEN COALESCE(amount, 0.0) ELSE 0.0 END), 0.0) AS expense,
            COALESCE(SUM(CASE WHEN status IN ('completed', 'paid') AND payment_type IN ('收入', 'income', 'receipt') THEN COALESCE(amount, 0.0) ELSE 0.0 END), 0.0) AS income,
            COALESCE(SUM(CASE
                WHEN COALESCE(invoice_required, 0) = 1
                 AND (invoice_id IS NULL OR invoice_id = '')
                THEN 1 ELSE 0 END), 0) AS pending_invoice,
            COALESCE(SUM(CASE
                WHEN COALESCE(invoice_required, 0) = 1
                 AND (invoice_id IS NULL OR invoice_id = '')
                 AND invoice_deadline IS NOT NULL
                 AND date(invoice_deadline) < date('now')
                THEN 1 ELSE 0 END), 0) AS overdue_invoice
        FROM payments
        WHERE company_id = ?
        "#,
    )
    .bind(&user.company_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let rows = sqlx::query(
        r#"
        SELECT
            id,
            payment_no,
            payment_date,
            amount,
            payment_method,
            payment_type,
            payer,
            payee,
            description,
            category,
            COALESCE(invoice_required, 0) AS invoice_required,
            CASE
                WHEN COALESCE(invoice_required, 0) = 1 AND invoice_id IS NOT NULL AND invoice_id != '' THEN 1
                ELSE 0
            END AS invoice_received,
            invoice_deadline,
            CASE
                WHEN COALESCE(invoice_required, 0) != 1 THEN 'not_required'
                WHEN invoice_id IS NOT NULL AND invoice_id != '' THEN 'received'
                WHEN invoice_deadline IS NOT NULL AND date(invoice_deadline) < date('now') THEN 'overdue'
                ELSE 'pending'
            END AS invoice_status,
            status,
            notes
        FROM payments
        WHERE company_id = ?
        ORDER BY COALESCE(payment_date, created_at) DESC, created_at DESC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let search = query.search.unwrap_or_default().trim().to_lowercase();
    let payment_type_filter = query.payment_type.unwrap_or_default();
    let payment_method_filter = query.payment_method.unwrap_or_default();
    let invoice_status_filter = query.invoice_status.unwrap_or_default();
    let has_filters = !search.is_empty()
        || (!payment_type_filter.is_empty() && payment_type_filter != "all")
        || (!payment_method_filter.is_empty() && payment_method_filter != "all")
        || (!invoice_status_filter.is_empty() && invoice_status_filter != "all");
    let can_delete = current_user_can_manage(&pool, &user).await?;

    let mut payments = Vec::new();
    let mut pending_invoice_payments = Vec::new();
    let mut reminder_total = 0_i64;
    let mut reminder_overdue = 0_i64;
    let mut reminder_urgent = 0_i64;
    let mut reminder_total_amount = 0.0;

    for row in rows {
        let payment_no = text(&row, "payment_no");
        let id = text(&row, "id");
        let payer = text(&row, "payer");
        let payee = text(&row, "payee");
        let description = text(&row, "description");
        let payment_type = text(&row, "payment_type").unwrap_or_else(|| "expense".to_string());
        let payment_method = text(&row, "payment_method").unwrap_or_default();
        let invoice_status =
            text(&row, "invoice_status").unwrap_or_else(|| "not_required".to_string());
        let amount = number(&row, "amount");
        let payment_date = text(&row, "payment_date");
        let payment_date_label = cloud_date_label(payment_date.as_deref());
        let status = text(&row, "status").unwrap_or_else(|| "completed".to_string());
        let is_expense = is_expense_payment(&payment_type);
        let payment_type_badge_value = payment_type_badge(&payment_type);
        let invoice_badge_value = payment_invoice_badge(&invoice_status);
        let status_badge_value = payment_status_badge(&status);
        let method_label = payment_method_label(&payment_method);
        let invoice_required = integer(&row, "invoice_required") == 1;
        let invoice_received = integer(&row, "invoice_received") == 1;
        let invoice_deadline = text(&row, "invoice_deadline");
        let invoice_deadline_label = cloud_date_label(invoice_deadline.as_deref());
        let urgency_badge = payment_invoice_urgency_badge(invoice_deadline.as_deref());
        let days_left = payment_invoice_days_left(invoice_deadline.as_deref());
        let delete_action = payment_delete_action(id.clone(), payment_no.clone(), can_delete);
        let update_action = payment_update_action(id.clone(), can_delete);

        let payment = json!({
            "id": id,
            "payment_no": payment_no,
            "payment_date": payment_date,
            "payment_date_label": payment_date_label,
            "amount": amount,
            "amount_label": format_cny(amount),
            "signed_amount_label": format!("{}{}", if is_expense { "-" } else { "+" }, format_cny(amount)),
            "amount_class": if is_expense { "text-red-600" } else { "text-green-600" },
            "payment_method": payment_method,
            "payment_method_label": method_label,
            "payment_type": payment_type,
            "payment_type_badge": payment_type_badge_value,
            "payer": payer,
            "payee": payee,
            "description": description,
            "category": text(&row, "category"),
            "invoice_required": invoice_required,
            "invoice_received": invoice_received,
            "invoice_deadline": invoice_deadline,
            "invoice_deadline_label": invoice_deadline_label,
            "invoice_status": invoice_status,
            "invoice_badge": invoice_badge_value,
            "urgency_badge": urgency_badge,
            "status": status,
            "status_badge": status_badge_value,
            "notes": text(&row, "notes"),
            "can_delete": can_delete,
            "delete_action": delete_action,
            "update_action": update_action,
        });

        if invoice_required && !invoice_received {
            reminder_total += 1;
            reminder_total_amount += amount;
            if matches!(days_left, Some(days) if days < 0) {
                reminder_overdue += 1;
            }
            if matches!(days_left, Some(0..=3)) {
                reminder_urgent += 1;
            }
            pending_invoice_payments.push(payment.clone());
        }

        if !payment_type_filter.is_empty()
            && payment_type_filter != "all"
            && !payment_type_matches(
                payment["payment_type"].as_str().unwrap_or_default(),
                &payment_type_filter,
            )
        {
            continue;
        }
        if !payment_method_filter.is_empty()
            && payment_method_filter != "all"
            && payment["payment_method"].as_str().unwrap_or_default() != payment_method_filter
        {
            continue;
        }
        if !invoice_status_filter.is_empty()
            && invoice_status_filter != "all"
            && payment["invoice_status"].as_str().unwrap_or_default() != invoice_status_filter
        {
            continue;
        }
        if !search.is_empty() {
            let matches_search = [
                payment["payment_no"].as_str(),
                payment["payer"].as_str(),
                payment["payee"].as_str(),
                payment["description"].as_str(),
            ]
            .into_iter()
            .flatten()
            .any(|value| value.to_lowercase().contains(&search));
            if !matches_search {
                continue;
            }
        }

        payments.push(payment);
    }

    let empty_message = if has_filters {
        "未找到匹配的支付记录"
    } else {
        "暂无支付记录"
    }
    .to_string();

    Ok(Json(PaymentsOverview {
        stats: PaymentStats {
            total: stats.total,
            expense: stats.expense,
            income: stats.income,
            expense_label: format_cny(stats.expense),
            income_label: format_cny(stats.income),
            pending_invoice: stats.pending_invoice,
            overdue_invoice: stats.overdue_invoice,
        },
        payments,
        invoice_reminder_stats: json!({
            "total": reminder_total,
            "overdue": reminder_overdue,
            "urgent": reminder_urgent,
            "total_amount": round_money(reminder_total_amount),
            "total_amount_label": format_cny(reminder_total_amount),
        }),
        pending_invoice_payments,
        permissions: json!({ "can_delete": can_delete }),
        create_action: payment_create_action(can_delete),
        payment_type_options: payment_overview_type_options(),
        payment_method_options: payment_overview_method_options(),
        invoice_status_options: payment_overview_invoice_status_options(),
        empty_message,
    }))
}

fn payment_create_action(can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    operations_create_action("/api/payments".to_string(), "新增支付", "支付记录已创建")
}

fn payment_update_action(id: Option<String>, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };
    operations_update_action(format!("/api/payments/{id}"), "保存修改", "支付记录已更新")
}

fn payment_delete_action(
    id: Option<String>,
    payment_no: Option<String>,
    can_delete: bool,
) -> Value {
    if !can_delete {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };
    let payment_no = payment_no
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("该支付记录");
    operations_delete_action(
        format!("/api/payments/{id}"),
        format!("确定要删除支付记录「{payment_no}」吗？"),
        "支付记录已删除",
    )
}

async fn payment_form_options(_user: AuthUser) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({
        "payment_type_options": payment_type_options(),
        "payment_method_options": payment_method_options(),
        "category_options": payment_category_options(),
        "status_options": payment_status_options(),
        "default_values": {
            "payment_type": "expense",
            "payment_method": "",
            "category": "",
            "status": "completed",
            "invoice_required": false
        }
    })))
}

async fn current_user_can_manage(pool: &SqlitePool, user: &AuthUser) -> Result<bool, ApiError> {
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

fn is_expense_payment(payment_type: &str) -> bool {
    matches!(payment_type, "支出" | "expense" | "salary")
}

fn payment_type_matches(payment_type: &str, filter: &str) -> bool {
    match filter {
        "expense" | "支出" => is_expense_payment(payment_type),
        "income" | "收入" => matches!(payment_type, "收入" | "income" | "receipt"),
        _ => payment_type == filter,
    }
}

fn payment_invoice_days_left(deadline: Option<&str>) -> Option<i64> {
    let deadline = deadline.map(str::trim).filter(|value| !value.is_empty())?;
    let parsed = NaiveDate::parse_from_str(deadline, "%Y-%m-%d").ok()?;
    Some((parsed - Utc::now().date_naive()).num_days())
}

fn payment_invoice_urgency_badge(deadline: Option<&str>) -> Value {
    match payment_invoice_days_left(deadline) {
        None => json!({
            "label": "未设定",
            "variant": "outline",
        }),
        Some(days) if days < 0 => json!({
            "label": format!("已逾期 {} 天", days.abs()),
            "variant": "destructive",
        }),
        Some(days) if days <= 3 => json!({
            "label": format!("紧急 ({days} 天)"),
            "variant": "destructive",
        }),
        Some(days) if days <= 7 => json!({
            "label": format!("即将到期 ({days} 天)"),
            "variant": "secondary",
        }),
        Some(days) => json!({
            "label": format!("{days} 天后"),
            "variant": "outline",
        }),
    }
}

fn payment_type_badge(payment_type: &str) -> Value {
    if is_expense_payment(payment_type) {
        json!({
            "value": "expense",
            "label": "支出",
            "variant": "destructive",
        })
    } else {
        json!({
            "value": "income",
            "label": "收入",
            "variant": "default",
        })
    }
}

fn payment_type_options() -> Vec<Value> {
    vec![
        json!({
            "value": "expense",
            "label": "支出",
            "variant": "destructive",
        }),
        json!({
            "value": "income",
            "label": "收入",
            "variant": "default",
        }),
    ]
}

fn payment_overview_type_options() -> Vec<Value> {
    let mut options = vec![json!({
        "value": "all",
        "label": "全部",
        "variant": "outline",
    })];
    options.extend(payment_type_options());
    options
}

fn payment_method_label(method: &str) -> String {
    match method {
        "bank_transfer_b2b" => "公对公",
        "bank_transfer_b2p" => "公对私",
        "alipay_corporate" => "企业支付宝",
        "jd_corporate" => "企业京东",
        "personal_advance" => "个人代付",
        "" => "-",
        value => value,
    }
    .to_string()
}

fn payment_method_options() -> Vec<Value> {
    [
        ("bank_transfer_b2b", "公对公"),
        ("bank_transfer_b2p", "公对私"),
        ("alipay_corporate", "企业支付宝"),
        ("jd_corporate", "企业京东"),
        ("personal_advance", "个人代付"),
    ]
    .into_iter()
    .map(|(value, label)| {
        json!({
            "value": value,
            "label": label,
        })
    })
    .collect()
}

fn payment_overview_method_options() -> Vec<Value> {
    let mut options = vec![json!({
        "value": "all",
        "label": "全部方式",
    })];
    options.extend(payment_method_options());
    options
}

fn payment_category_options() -> Vec<Value> {
    [
        "工资支出",
        "社保公积金",
        "办公费用",
        "房租物业",
        "差旅费",
        "采购支出",
        "税费",
        "其他支出",
        "销售收入",
        "服务收入",
        "其他收入",
    ]
    .into_iter()
    .map(|value| {
        json!({
            "value": value,
            "label": value,
        })
    })
    .collect()
}

fn payment_invoice_badge(invoice_status: &str) -> Value {
    match invoice_status {
        "received" => json!({
            "value": "received",
            "label": "已收发票",
            "variant": "default",
            "icon": "CheckCircle",
            "visible": true,
        }),
        "overdue" => json!({
            "value": "overdue",
            "label": "发票逾期",
            "variant": "destructive",
            "icon": "AlertCircle",
            "visible": true,
        }),
        "pending" => json!({
            "value": "pending",
            "label": "待收发票",
            "variant": "secondary",
            "icon": "AlertCircle",
            "visible": true,
        }),
        _ => json!({
            "value": "not_required",
            "label": "无需发票",
            "variant": "outline",
            "icon": "FileText",
            "visible": false,
        }),
    }
}

fn payment_invoice_status_options() -> Vec<Value> {
    ["pending", "received", "overdue", "not_required"]
        .into_iter()
        .map(payment_invoice_badge)
        .collect()
}

fn payment_overview_invoice_status_options() -> Vec<Value> {
    let mut options = vec![json!({
        "value": "all",
        "label": "全部发票",
        "variant": "outline",
        "icon": "FileText",
        "visible": true,
    })];
    options.extend(payment_invoice_status_options());
    options
}

fn payment_status_badge(status: &str) -> Value {
    match status {
        "pending" => json!({
            "value": "pending",
            "label": "待处理",
            "variant": "secondary",
            "icon": "Clock",
        }),
        "cancelled" => json!({
            "value": "cancelled",
            "label": "已取消",
            "variant": "destructive",
            "icon": "XCircle",
        }),
        _ => json!({
            "value": "completed",
            "label": "已完成",
            "variant": "default",
            "icon": "CheckCircle",
        }),
    }
}

fn payment_status_options() -> Vec<Value> {
    ["completed", "pending", "cancelled"]
        .into_iter()
        .map(payment_status_badge)
        .collect()
}

fn last_six_periods() -> Vec<(String, String)> {
    let today = Utc::now().date_naive();
    (0..6)
        .rev()
        .map(|months_ago| {
            let (year, month) =
                shift_month(today.year(), today.month() as i32, -(months_ago as i32));
            (format!("{year:04}-{month:02}"), format!("{month}月"))
        })
        .collect()
}

fn shift_month(year: i32, month: i32, delta: i32) -> (i32, i32) {
    let zero_based = year * 12 + (month - 1) + delta;
    (zero_based.div_euclid(12), zero_based.rem_euclid(12) + 1)
}

fn project_rd_cost_summary(group: Option<&ProjectRdCostGroup>) -> Value {
    let Some(group) = group else {
        return json!({
            "total": 0.0,
            "total_label": format_cny(0.0),
            "total_wan_label": format_wan(0.0),
            "by_type": {},
            "items": [],
            "pie_data": [],
            "empty_message": "暂无研发费用记录，点击\"添加费用\"开始录入",
        });
    };

    let mut by_type = serde_json::Map::new();
    let mut pie_data = Vec::with_capacity(group.by_type.len());
    for (cost_type, amount) in &group.by_type {
        let rounded = round_money(*amount);
        by_type.insert(cost_type.clone(), json!(rounded));
        pie_data.push(json!({
            "type": cost_type,
            "name": rd_cost_type_label(cost_type),
            "value": rounded,
            "amount_label": format_cny(*amount),
        }));
    }

    json!({
        "total": round_money(group.total),
        "total_label": format_cny(group.total),
        "total_wan_label": format_wan(group.total),
        "by_type": by_type,
        "items": group.items.clone(),
        "pie_data": pie_data,
        "empty_message": "暂无研发费用记录，点击\"添加费用\"开始录入",
    })
}

fn project_status_options() -> Vec<Value> {
    [
        ("all", "全部状态"),
        ("planning", "规划中"),
        ("ongoing", "进行中"),
        ("review", "评审中"),
        ("completed", "已完成"),
        ("paused", "已暂停"),
    ]
    .into_iter()
    .map(|(value, label)| json!({ "value": value, "label": label }))
    .collect()
}

fn rd_cost_type_options() -> Vec<Value> {
    [
        ("personnel", "人员费用", "研发人员工资、奖金"),
        ("equipment", "设备费用", "设备采购、设备折旧"),
        ("material", "材料费用", "实验材料、耗材"),
        ("outsourcing", "外协费用", "委托开发、测试服务"),
        ("depreciation", "折旧费用", "研发设备折旧摊销"),
        ("other", "其他费用", "差旅、会议、培训等"),
    ]
    .into_iter()
    .map(|(value, label, description)| {
        json!({ "value": value, "label": label, "description": description })
    })
    .collect()
}

async fn project_employee_options(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<Vec<Value>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, position
        FROM employees
        WHERE company_id = ? AND COALESCE(status, 'active') = 'active'
        ORDER BY name ASC
        "#,
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let id = text(&row, "id").unwrap_or_default();
            let name = text(&row, "name").unwrap_or_default();
            let position = text(&row, "position")
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "未知职位".to_string());
            json!({
                "value": id,
                "label": format!("{name} - {position}"),
            })
        })
        .collect())
}

async fn project_asset_options(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<Vec<Value>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, serial_number
        FROM assets
        WHERE company_id = ? AND COALESCE(asset_type, 'physical') = 'physical'
        ORDER BY name ASC
        "#,
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let id = text(&row, "id").unwrap_or_default();
            let name = text(&row, "name").unwrap_or_default();
            let serial = text(&row, "serial_number")
                .filter(|value| !value.trim().is_empty())
                .map(|value| format!(" ({value})"))
                .unwrap_or_default();
            json!({
                "value": id,
                "label": format!("{name}{serial}"),
            })
        })
        .collect())
}

fn project_status_badge(status: &str) -> Value {
    let (label, variant, icon) = match status {
        "ongoing" => ("进行中", "default", "TrendingUp"),
        "review" => ("评审中", "outline", "Clock"),
        "completed" => ("已完成", "default", "CheckCircle2"),
        "paused" => ("已暂停", "destructive", "AlertTriangle"),
        _ => ("规划中", "secondary", "Calendar"),
    };
    json!({
        "label": label,
        "variant": variant,
        "icon": icon,
        "class_name": "gap-1",
    })
}

fn rd_cost_type_badge(label: &str) -> Value {
    json!({
        "label": label,
        "variant": "outline",
        "class_name": "",
    })
}

fn rd_cost_type_label(cost_type: &str) -> &'static str {
    match cost_type {
        "personnel" => "人员费用",
        "equipment" => "设备费用",
        "material" => "材料费用",
        "outsourcing" => "外协费用",
        "depreciation" => "折旧费用",
        _ => "其他费用",
    }
}

fn deal_stage_options() -> Vec<DealStageOption> {
    [
        "lead",
        "qualified",
        "proposal",
        "negotiation",
        "won",
        "lost",
    ]
    .into_iter()
    .map(deal_stage_option)
    .collect()
}

fn deal_stage_option(stage: &str) -> DealStageOption {
    DealStageOption {
        value: stage.to_string(),
        label: deal_stage_label(stage).to_string(),
        color: deal_stage_color(stage).to_string(),
    }
}

fn deal_stage_label(stage: &str) -> &'static str {
    match stage {
        "lead" => "线索",
        "qualified" => "合格",
        "proposal" => "提案",
        "negotiation" => "谈判",
        "won" => "赢单",
        "lost" => "丢单",
        _ => "其他",
    }
}

fn deal_stage_color(stage: &str) -> &'static str {
    match stage {
        "lead" => "bg-gray-200 text-gray-800",
        "qualified" => "bg-blue-100 text-blue-800",
        "proposal" => "bg-purple-100 text-purple-800",
        "negotiation" => "bg-yellow-100 text-yellow-800",
        "won" => "bg-green-100 text-green-800",
        "lost" => "bg-red-100 text-red-800",
        _ => "bg-slate-100 text-slate-800",
    }
}

fn is_closed_deal_stage(stage: &str) -> bool {
    stage == "won" || stage == "lost"
}

fn quote_status_options() -> Vec<Value> {
    [
        "draft", "sent", "accepted", "rejected", "expired", "invoiced",
    ]
    .into_iter()
    .map(|status| {
        json!({
            "value": status,
            "label": quote_status_label(status),
            "variant": quote_status_variant(status),
        })
    })
    .collect()
}

fn quote_status_label(status: &str) -> &'static str {
    match status {
        "draft" => "草稿",
        "sent" => "已发送",
        "accepted" => "已接受",
        "rejected" => "已拒绝",
        "expired" => "已过期",
        "invoiced" => "已开票",
        _ => "未知",
    }
}

fn quote_status_variant(status: &str) -> &'static str {
    match status {
        "accepted" | "invoiced" => "default",
        "sent" | "expired" => "secondary",
        "rejected" => "destructive",
        _ => "outline",
    }
}

fn contract_status_options() -> Vec<Value> {
    ["all", "draft", "active", "expired", "archived"]
        .into_iter()
        .map(|status| {
            if status == "all" {
                json!({
                    "value": "all",
                    "label": "全部状态",
                    "variant": "outline",
                    "icon": "FileText",
                })
            } else {
                let badge = contract_status_badge(status);
                json!({
                    "value": status,
                    "label": badge["label"],
                    "variant": badge["variant"],
                    "icon": badge["icon"],
                })
            }
        })
        .collect()
}

fn contract_status_badge(status: &str) -> Value {
    match status {
        "active" => json!({
            "label": "执行中",
            "variant": "default",
            "icon": "CheckCircle",
        }),
        "expired" => json!({
            "label": "已到期",
            "variant": "destructive",
            "icon": "AlertCircle",
        }),
        "archived" => json!({
            "label": "已归档",
            "variant": "outline",
            "icon": "Archive",
        }),
        _ => json!({
            "label": "草稿",
            "variant": "secondary",
            "icon": "Clock",
        }),
    }
}

fn contract_ai_risk_badge(review_status: &str, risk: Option<&str>) -> Value {
    match review_status {
        "reviewing" | "pending" => json!({
            "label": "审查中",
            "variant": "outline",
            "tone": "muted",
            "icon": "Loader2",
            "spin": true,
            "class_name": "text-muted-foreground",
        }),
        "unavailable" => json!({
            "label": "待审查",
            "variant": "outline",
            "tone": "muted",
            "icon": "FileText",
            "spin": false,
            "class_name": "text-muted-foreground",
        }),
        "failed" => json!({
            "label": "审查失败",
            "variant": "destructive",
            "tone": "danger",
            "icon": "AlertCircle",
            "spin": false,
            "class_name": "",
        }),
        _ => match contract_risk_tone(risk) {
            "danger" => json!({
                "label": "高风险",
                "variant": "default",
                "tone": "danger",
                "icon": "ShieldAlert",
                "spin": false,
                "class_name": "bg-red-500 hover:bg-red-600",
            }),
            "warning" => json!({
                "label": "中风险",
                "variant": "default",
                "tone": "warning",
                "icon": "ShieldAlert",
                "spin": false,
                "class_name": "bg-yellow-500 hover:bg-yellow-600 text-black",
            }),
            "safe" => json!({
                "label": "低风险",
                "variant": "default",
                "tone": "safe",
                "icon": "CheckCircle",
                "spin": false,
                "class_name": "bg-green-500 hover:bg-green-600",
            }),
            _ => Value::Null,
        },
    }
}

fn contract_ai_review_empty_state(
    icon: &str,
    icon_class_name: &str,
    title: &str,
    description: Option<&str>,
) -> Value {
    json!({
        "icon": icon,
        "icon_class_name": icon_class_name,
        "title": title,
        "description": description,
    })
}

fn contract_ai_review_display(
    review_status: &str,
    risk: Option<&str>,
    review_result: Option<&str>,
) -> Value {
    let has_review = review_status == "completed"
        && review_result
            .map(str::trim)
            .filter(|result| !result.is_empty())
            .is_some();

    match review_status {
        "reviewing" | "pending" => json!({
            "label": "审查中",
            "icon": "RefreshCw",
            "class_name": "bg-muted text-muted-foreground",
            "border_class_name": "border-muted",
            "has_review": false,
            "empty_state": contract_ai_review_empty_state(
                "RefreshCw",
                "animate-spin",
                "AI 正在分析合同内容...",
                Some("通常需要 10-30 秒"),
            ),
        }),
        "unavailable" => json!({
            "label": "待审查",
            "icon": "AlertTriangle",
            "class_name": "bg-muted text-muted-foreground",
            "border_class_name": "border-muted",
            "has_review": false,
            "empty_state": contract_ai_review_empty_state(
                "AlertTriangle",
                "",
                "AI 审查服务暂不可用",
                Some("服务恢复后将自动补全审查"),
            ),
        }),
        "failed" => json!({
            "label": "审查失败",
            "icon": "ShieldAlert",
            "class_name": "bg-red-500/10 text-red-600",
            "border_class_name": "border-red-500/30",
            "has_review": false,
            "empty_state": contract_ai_review_empty_state(
                "ShieldAlert",
                "text-red-400",
                "审查过程遇到问题",
                Some("请点击\"重新审查\"重试"),
            ),
        }),
        _ if has_review => match contract_risk_tone(risk) {
            "danger" => json!({
                "label": "高风险",
                "icon": "ShieldAlert",
                "class_name": "bg-red-500/10 text-red-600",
                "border_class_name": "border-red-500/30",
                "has_review": true,
                "empty_state": Value::Null,
            }),
            "warning" => json!({
                "label": "中风险",
                "icon": "AlertTriangle",
                "class_name": "bg-yellow-500/10 text-yellow-700",
                "border_class_name": "border-yellow-500/30",
                "has_review": true,
                "empty_state": Value::Null,
            }),
            "safe" => json!({
                "label": "低风险",
                "icon": "ShieldCheck",
                "class_name": "bg-green-500/10 text-green-600",
                "border_class_name": "border-green-500/30",
                "has_review": true,
                "empty_state": Value::Null,
            }),
            _ => json!({
                "label": "未知",
                "icon": "Clock",
                "class_name": "bg-muted text-muted-foreground",
                "border_class_name": "border-muted",
                "has_review": true,
                "empty_state": Value::Null,
            }),
        },
        _ => json!({
            "label": "暂无审查结果",
            "icon": "Clock",
            "class_name": "bg-muted text-muted-foreground",
            "border_class_name": "border-muted",
            "has_review": false,
            "empty_state": contract_ai_review_empty_state(
                "Clock",
                "",
                "暂无审查结果",
                None,
            ),
        }),
    }
}

fn contract_risk_tone(risk: Option<&str>) -> &'static str {
    let Some(risk) = risk.map(str::trim).filter(|risk| !risk.is_empty()) else {
        return "none";
    };
    let lower = risk.to_lowercase();
    if lower.contains("high") || risk.contains('高') {
        "danger"
    } else if lower.contains("medium") || risk.contains('中') {
        "warning"
    } else {
        "safe"
    }
}

fn is_high_contract_risk(risk: Option<&str>) -> bool {
    contract_risk_tone(risk) == "danger"
}

fn asset_record_action_options() -> Vec<Value> {
    [
        "all",
        "checkout",
        "checkin",
        "transfer",
        "maintenance",
        "inventory",
    ]
    .into_iter()
    .map(|action| {
        if action == "all" {
            json!({
                "value": "all",
                "label": "全部类型",
                "variant": "outline",
                "class_name": "",
            })
        } else {
            let label = asset_record_action_label(action);
            let badge = asset_record_action_badge(label);
            json!({
                "value": action,
                "label": label,
                "variant": badge["variant"],
                "class_name": badge["class_name"],
            })
        }
    })
    .collect()
}

fn asset_record_action_label(action: &str) -> &'static str {
    match action {
        "checkout" => "领用",
        "checkin" => "归还",
        "transfer" => "调拨",
        "maintenance" => "维修",
        "inventory" => "盘点",
        _ => "其他",
    }
}

fn asset_record_action_badge(label: &str) -> Value {
    match label {
        "领用" => json!({
            "label": label,
            "variant": "default",
            "class_name": "bg-accent",
        }),
        "归还" => json!({
            "label": label,
            "variant": "default",
            "class_name": "bg-primary",
        }),
        "调拨" => json!({
            "label": label,
            "variant": "default",
            "class_name": "bg-warning",
        }),
        "维修" => json!({
            "label": label,
            "variant": "destructive",
            "class_name": "",
        }),
        "盘点" => json!({
            "label": label,
            "variant": "secondary",
            "class_name": "",
        }),
        _ => json!({
            "label": label,
            "variant": "outline",
            "class_name": "",
        }),
    }
}

fn asset_record_export_csv(rows: Vec<Vec<String>>) -> String {
    let mut lines = vec![
        [
            "记录编号",
            "资产名称",
            "资产编号",
            "操作类型",
            "处理人",
            "部门",
            "时间",
            "位置",
        ]
        .join(","),
    ];

    for row in rows {
        lines.push(
            row.into_iter()
                .map(|value| csv_escape(&value))
                .collect::<Vec<_>>()
                .join(","),
        );
    }

    lines.join("\n")
}

fn reminder_status_badge(status: &str) -> Value {
    match status {
        "completed" => json!({
            "label": "已完成",
            "variant": "outline",
            "icon": "CheckCircle2",
        }),
        "dismissed" => json!({
            "label": "已忽略",
            "variant": "secondary",
            "icon": "XCircle",
        }),
        _ => json!({
            "label": "待处理",
            "variant": "default",
            "icon": "Bell",
        }),
    }
}

async fn fetch_reminder_row(
    pool: &SqlitePool,
    company_id: &str,
    id: &str,
) -> Result<SqliteRow, ApiError> {
    sqlx::query(
        r#"
        SELECT
            id,
            title,
            description,
            reminder_type,
            priority,
            due_date,
            remind_date,
            status,
            is_completed,
            entity_type,
            entity_id,
            created_at,
            updated_at
        FROM reminders
        WHERE company_id = ? AND id = ?
        LIMIT 1
        "#,
    )
    .bind(company_id)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?
    .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "reminder not found"))
}

fn reminder_display_value(row: &SqliteRow, today: NaiveDate) -> Value {
    let status = text(row, "status").unwrap_or_else(|| "pending".to_string());
    let priority = text(row, "priority").unwrap_or_else(|| "normal".to_string());
    let reminder_type = text(row, "reminder_type").unwrap_or_else(|| "general".to_string());
    let remind_date = text(row, "remind_date");
    let date_state = reminder_date_state(remind_date.as_deref(), today);
    let id = text(row, "id");

    json!({
        "id": id.clone(),
        "title": text(row, "title"),
        "description": text(row, "description"),
        "reminder_type": reminder_type.clone(),
        "type_label": reminder_type_label(&reminder_type),
        "priority": priority.clone(),
        "priority_badge": reminder_priority_badge(&priority),
        "due_date": text(row, "due_date"),
        "remind_date": remind_date,
        "remind_date_label": reminder_date_label(text(row, "remind_date").as_deref(), true),
        "history_remind_date_label": reminder_date_label(text(row, "remind_date").as_deref(), false),
        "status": status.clone(),
        "status_badge": reminder_status_badge(&status),
        "is_completed": integer(row, "is_completed") == 1,
        "entity_type": text(row, "entity_type"),
        "entity_id": text(row, "entity_id"),
        "created_at": text(row, "created_at"),
        "updated_at": text(row, "updated_at"),
        "date_state": date_state,
        "date_badge": reminder_date_badge(&date_state),
        "card_class": reminder_card_class(&date_state),
        "date_text_class": reminder_date_text_class(&date_state),
        "show_alert_icon": date_state == "overdue",
        "complete_action": reminder_status_action(
            id.clone(),
            &status,
            "complete",
            "完成",
            "提醒已标记完成",
            "completed",
        ),
        "dismiss_action": reminder_status_action(
            id,
            &status,
            "dismiss",
            "忽略",
            "提醒已忽略",
            "dismissed",
        ),
    })
}

fn reminder_status_action(
    id: Option<String>,
    status: &str,
    action_path: &str,
    label: &str,
    success_message: &str,
    target_status: &str,
) -> Value {
    if matches!(status, "completed" | "dismissed") {
        return json!(null);
    }
    let Some(id) = id else {
        return json!(null);
    };

    operations_post_action(
        format!("/api/v1/reminders/{id}/{action_path}"),
        label,
        success_message,
        target_status,
    )
}

fn asset_expiry_reminder_priority(expiry_date: &str, today: NaiveDate) -> &'static str {
    let days_left = NaiveDate::parse_from_str(expiry_date, "%Y-%m-%d")
        .map(|date| date.signed_duration_since(today).num_days())
        .unwrap_or(30);
    if days_left <= 3 {
        "urgent"
    } else if days_left <= 7 {
        "high"
    } else {
        "normal"
    }
}

fn reminder_priority_badge(priority: &str) -> Value {
    match priority {
        "low" => json!({
            "label": "低",
            "variant": "outline",
        }),
        "high" => json!({
            "label": "高",
            "variant": "default",
        }),
        "urgent" => json!({
            "label": "紧急",
            "variant": "destructive",
        }),
        _ => json!({
            "label": "普通",
            "variant": "secondary",
        }),
    }
}

fn reminder_type_label(reminder_type: &str) -> &'static str {
    match reminder_type {
        "license" | "license_expiry" => "证照",
        "contract" | "contract_expiry" => "合同",
        "project" | "project_deadline" => "项目",
        "invoice" | "invoice_due" => "发票",
        "employee" => "员工",
        "asset" | "asset_expiry" => "资产到期",
        "rent" | "rent_expiry" => "租赁",
        _ => "一般",
    }
}

fn reminder_date_state(remind_date: Option<&str>, today: NaiveDate) -> &'static str {
    let Some(date) = parse_reminder_date(remind_date) else {
        return "none";
    };
    if date < today {
        "overdue"
    } else if date > today && date < today + Duration::days(3) {
        "upcoming"
    } else {
        "normal"
    }
}

fn reminder_date_badge(date_state: &str) -> Value {
    match date_state {
        "overdue" => json!({
            "label": "已逾期",
            "variant": "destructive",
            "class_name": "",
        }),
        "upcoming" => json!({
            "label": "即将到期",
            "variant": "outline",
            "class_name": "border-yellow-500 text-yellow-600",
        }),
        _ => Value::Null,
    }
}

fn reminder_card_class(date_state: &str) -> &'static str {
    match date_state {
        "overdue" => "border-destructive",
        "upcoming" => "border-yellow-500",
        _ => "",
    }
}

fn reminder_date_text_class(date_state: &str) -> &'static str {
    match date_state {
        "overdue" => "text-destructive",
        _ => "text-muted-foreground",
    }
}

fn reminder_date_label(remind_date: Option<&str>, chinese: bool) -> String {
    let Some(date) = parse_reminder_date(remind_date) else {
        return "-".to_string();
    };
    if chinese {
        format!("{}年{:02}月{:02}日", date.year(), date.month(), date.day())
    } else {
        date.to_string()
    }
}

fn parse_reminder_date(remind_date: Option<&str>) -> Option<NaiveDate> {
    let value = remind_date?.trim();
    if value.is_empty() {
        return None;
    }
    NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
}

fn operation_log_type_options() -> Vec<Value> {
    [
        ("all", "全部类型"),
        ("CREATE", "新建"),
        ("UPDATE", "修改"),
        ("DELETE", "删除"),
        ("VIEW", "查看"),
    ]
    .into_iter()
    .map(|(value, label)| {
        json!({
            "value": value,
            "label": label,
        })
    })
    .collect()
}

fn operation_log_action_label(operation_type: &str) -> &'static str {
    match operation_type {
        "CREATE" => "新建",
        "UPDATE" => "修改",
        "DELETE" => "删除",
        "VIEW" => "查看",
        _ => "其他",
    }
}

fn operation_log_action_badge(operation_type: &str) -> Value {
    match operation_type {
        "CREATE" => json!({
            "label": "新建",
            "variant": "default",
            "class_name": "bg-primary",
        }),
        "UPDATE" => json!({
            "label": "修改",
            "variant": "default",
            "class_name": "bg-accent",
        }),
        "DELETE" => json!({
            "label": "删除",
            "variant": "destructive",
            "class_name": "",
        }),
        "VIEW" => json!({
            "label": "查看",
            "variant": "secondary",
            "class_name": "",
        }),
        _ => json!({
            "label": operation_type,
            "variant": "outline",
            "class_name": "",
        }),
    }
}

fn operation_log_table_label(table_name: &str) -> &'static str {
    match table_name {
        "employees" => "员工",
        "salary_records" | "salaries" => "薪资",
        "contracts" => "合同",
        "invoices" => "发票",
        "payments" => "支付",
        "company_info" => "公司信息",
        "user_roles" => "用户角色",
        "tax_declarations" | "tax_records" => "税务申报",
        "budgets" => "预算",
        "collection_deliveries" => "自动催收",
        "collection_settings" => "催收设置",
        _ => "其他",
    }
}

fn operation_log_detail_value(details: Option<&str>) -> Value {
    let Some(details) = details else {
        return Value::Null;
    };
    serde_json::from_str(details).unwrap_or_else(|_| json!(details))
}

fn short_record_id(record_id: &str) -> String {
    if record_id == "-" {
        return "-".to_string();
    }
    let prefix: String = record_id.chars().take(8).collect();
    if record_id.chars().count() > 8 {
        format!("{prefix}...")
    } else {
        prefix
    }
}

fn operation_log_export_csv(rows: Vec<Vec<String>>) -> String {
    let mut lines = vec![["时间", "操作类型", "数据表", "记录ID", "详情"].join(",")];
    for row in rows {
        lines.push(
            row.into_iter()
                .map(|value| csv_escape(&value))
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    lines.join("\n")
}

fn clamp_salary_base(base: f64, min: f64, max: f64) -> f64 {
    let lower_bounded = if min > 0.0 { base.max(min) } else { base };
    if max > 0.0 {
        lower_bounded.min(max)
    } else {
        lower_bounded
    }
}

fn round_money(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn format_wan(value: f64) -> String {
    format!("¥{:.1}万", round_money(value) / 10_000.0)
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

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn text(row: &sqlx::sqlite::SqliteRow, column: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(column).ok().flatten()
}

fn number(row: &sqlx::sqlite::SqliteRow, column: &str) -> f64 {
    row.try_get::<Option<f64>, _>(column)
        .ok()
        .flatten()
        .unwrap_or(0.0)
}

fn integer(row: &sqlx::sqlite::SqliteRow, column: &str) -> i64 {
    row.try_get::<i64, _>(column).unwrap_or(0)
}
