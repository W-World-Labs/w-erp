use super::error::{ApiError, api_err};
use crate::auth::AuthUser;
use crate::db::models::Budget;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
};
use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::SqlitePool;

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/", get(list_budgets).post(create_budget))
        .route("/overview", get(budget_overview))
        .route("/form_options", get(budget_form_options))
        .route(
            "/{id}",
            get(get_budget).put(update_budget).delete(delete_budget),
        )
        .with_state(pool)
}

#[derive(Deserialize)]
pub struct CreateBudgetRequest {
    pub category: String,
    pub budget_type: String,
    pub period: Option<String>,
    pub budget_amount: Option<Value>,
    pub project_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Serialize)]
pub struct BudgetsResponse {
    pub data: Vec<Budget>,
    pub count: i64,
}

#[derive(Deserialize)]
struct BudgetOverviewQuery {
    budget_type: Option<String>,
    period: Option<String>,
}

#[derive(Serialize)]
struct BudgetOverviewResponse {
    budget_type: String,
    period: String,
    budgets: Vec<BudgetExecution>,
    stats: BudgetStats,
    trend: Vec<BudgetTrendPoint>,
    budget_type_options: Vec<Value>,
    create_action: Value,
    permissions: Value,
    empty_message: String,
}

#[derive(Serialize)]
struct BudgetExecution {
    id: String,
    category: String,
    budget_type: String,
    period: String,
    budget_amount: f64,
    actual_amount: f64,
    status: String,
    notes: Option<String>,
    percentage: f64,
    is_exceeded: bool,
    category_label: String,
    category_icon: String,
    category_class: String,
    status_badge: Value,
    budget_amount_label: String,
    actual_amount_label: String,
    usage_label: String,
    remaining_label: String,
    progress_value: f64,
    progress_class: String,
    update_action: Value,
    can_delete: bool,
    delete_action: Value,
}

#[derive(Default, Serialize)]
struct BudgetStats {
    total_budget: f64,
    total_actual: f64,
    exceeded_count: i64,
    saved_amount: f64,
    utilization: f64,
    total_budget_label: String,
    total_actual_label: String,
    saved_amount_label: String,
    utilization_label: String,
}

#[derive(Serialize)]
struct BudgetTrendPoint {
    period: String,
    period_label: String,
    budget: f64,
    actual: f64,
    budget_label: String,
    actual_label: String,
}

struct PeriodRange {
    start_date: String,
    end_date: String,
    start_month: String,
    end_month: String,
    months: i64,
}

async fn current_user_can_manage_budgets(
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

fn budget_format_cny(amount: f64) -> String {
    let rounded = (amount * 100.0).round() / 100.0;
    if (rounded.fract()).abs() < 0.005 {
        format!("¥{}", rounded as i64)
    } else {
        let mut value = format!("{rounded:.2}");
        while value.contains('.') && value.ends_with('0') {
            value.pop();
        }
        if value.ends_with('.') {
            value.pop();
        }
        format!("¥{value}")
    }
}

fn budget_percent_label(value: f64) -> String {
    let rounded = (value * 10.0).round() / 10.0;
    if (rounded.fract()).abs() < 0.05 {
        format!("{}%", rounded as i64)
    } else {
        format!("{rounded:.1}%")
    }
}

fn budget_category_display(category: &str) -> (&'static str, &'static str, &'static str) {
    match category {
        "salary" => ("工资支出", "Users", "text-primary"),
        "invoice" => ("发票支出", "FileText", "text-accent"),
        "rent" => ("租金支出", "Building2", "text-secondary"),
        "utility" => ("水电支出", "Zap", "text-warning"),
        _ => ("其他支出", "DollarSign", "text-muted-foreground"),
    }
}

fn budget_status_badge(is_exceeded: bool, percentage: f64) -> Value {
    if is_exceeded {
        json!({
            "value": "exceeded",
            "label": "超支",
            "variant": "destructive",
            "icon": "AlertTriangle",
            "class_name": "gap-1",
        })
    } else if percentage > 90.0 {
        json!({
            "value": "warning",
            "label": "预警",
            "variant": "outline",
            "icon": "AlertTriangle",
            "class_name": "gap-1 text-warning",
        })
    } else {
        json!({
            "value": "normal",
            "label": "正常",
            "variant": "default",
            "icon": "CheckCircle2",
            "class_name": "gap-1",
        })
    }
}

fn budget_progress_class(is_exceeded: bool, percentage: f64) -> String {
    if is_exceeded {
        "[&>div]:bg-destructive".to_string()
    } else if percentage > 90.0 {
        "[&>div]:bg-warning".to_string()
    } else {
        String::new()
    }
}

fn budget_type_options() -> Vec<Value> {
    vec![
        json!({"value": "monthly", "label": "月度预算"}),
        json!({"value": "quarterly", "label": "季度预算"}),
        json!({"value": "annual", "label": "年度预算"}),
        json!({"value": "project", "label": "项目预算"}),
    ]
}

fn budget_category_options() -> Vec<Value> {
    ["salary", "invoice", "rent", "utility", "other"]
        .into_iter()
        .map(|category| {
            let (label, icon, class_name) = budget_category_display(category);
            json!({
                "value": category,
                "label": label,
                "icon": icon,
                "class_name": class_name,
            })
        })
        .collect()
}

async fn budget_form_options() -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({
        "budget_type_options": budget_type_options(),
        "category_options": budget_category_options(),
        "default_values": {
            "budget_type": "monthly",
            "category": "salary",
        },
    })))
}

fn budget_period_label(budget_type: &str, period: &str) -> String {
    if budget_type == "monthly"
        && let Some(month) = period.split('-').nth(1)
        && let Ok(month_number) = month.parse::<u32>()
    {
        return format!("{}月", month_number);
    }
    period.to_string()
}

async fn list_budgets(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(q): Query<super::generic::ListQuery>,
) -> Result<Json<BudgetsResponse>, ApiError> {
    let limit = q.limit.or(q.page_size).unwrap_or(100).min(1000).max(1);
    let offset = q
        .offset
        .or_else(|| q.page.map(|p| (p.max(1) - 1) * limit))
        .unwrap_or(0)
        .max(0);
    let budgets: Vec<Budget> = sqlx::query_as(
        "SELECT * FROM budgets WHERE company_id = ? ORDER BY period DESC LIMIT ? OFFSET ?",
    )
    .bind(&user.company_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM budgets WHERE company_id = ?")
        .bind(&user.company_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(Json(BudgetsResponse {
        data: budgets,
        count,
    }))
}

async fn get_budget(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Budget>, ApiError> {
    let budget: Option<Budget> =
        sqlx::query_as("SELECT * FROM budgets WHERE id = ? AND company_id = ?")
            .bind(&id)
            .bind(&user.company_id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    budget
        .map(Json)
        .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "budget not found"))
}

async fn create_budget(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Json(req): Json<CreateBudgetRequest>,
) -> Result<Json<Budget>, ApiError> {
    let budget_amount = parse_optional_budget_money("budget_amount", req.budget_amount.as_ref())?;
    let period = req
        .period
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| default_period_for_type(&req.budget_type));
    period_range(&req.budget_type, &period)?;

    // R2-3: Validate non-negative budget amount
    if let Some(amt) = budget_amount
        && amt < 0.0
    {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "budget_amount must be non-negative",
        ));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let budget_amount = budget_amount.unwrap_or(0.0);
    let actual_amount = calculate_actual_expense(
        &pool,
        &user.company_id,
        &req.budget_type,
        &period,
        &req.category,
    )
    .await?;
    let status = budget_status(actual_amount, budget_amount);

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    sqlx::query(
        "INSERT INTO budgets (id, company_id, category, budget_type, period, budget_amount, actual_amount, status, project_id, notes) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&user.company_id)
    .bind(&req.category)
    .bind(&req.budget_type)
    .bind(&period)
    .bind(budget_amount)
    .bind(actual_amount)
    .bind(status)
    .bind(&req.project_id)
    .bind(&req.notes)
    .execute(&mut *tx)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    // Audit log
    let log_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO operation_logs (id, company_id, user_id, action, entity_type, entity_id) VALUES (?, ?, ?, 'create', 'budgets', ?)")
        .bind(&log_id).bind(&user.company_id).bind(&user.user_id).bind(&id)
        .execute(&mut *tx).await.ok();

    tx.commit()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    // R2-1: Re-fetch with company_id scoping
    let budget: Budget = sqlx::query_as("SELECT * FROM budgets WHERE id = ? AND company_id = ?")
        .bind(&id)
        .bind(&user.company_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(Json(budget))
}

fn parse_optional_budget_money(
    field: &str,
    value: Option<&Value>,
) -> Result<Option<f64>, ApiError> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    crate::api::generic::parse_form_money(value)
        .map(Some)
        .ok_or_else(|| {
            api_err(
                StatusCode::BAD_REQUEST,
                &format!("{} must be a valid number", field),
            )
        })
}

async fn update_budget(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(mut body): Json<Value>,
) -> Result<Json<Budget>, ApiError> {
    let existing: Budget = sqlx::query_as("SELECT * FROM budgets WHERE id = ? AND company_id = ?")
        .bind(&id)
        .bind(&user.company_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| api_err(StatusCode::NOT_FOUND, "budget not found"))?;

    let category = body
        .get("category")
        .and_then(|value| value.as_str())
        .unwrap_or(&existing.category)
        .to_string();
    let budget_type = body
        .get("budget_type")
        .and_then(|value| value.as_str())
        .unwrap_or(&existing.budget_type)
        .to_string();
    let period = body
        .get("period")
        .and_then(|value| value.as_str())
        .unwrap_or(&existing.period)
        .to_string();
    let budget_amount = body
        .get("budget_amount")
        .and_then(crate::api::generic::parse_form_money)
        .or(existing.budget_amount)
        .unwrap_or(0.0);

    if budget_amount < 0.0 {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "budget_amount must be non-negative",
        ));
    }

    let actual_amount =
        calculate_actual_expense(&pool, &user.company_id, &budget_type, &period, &category).await?;
    let status = budget_status(actual_amount, budget_amount);

    if let Some(obj) = body.as_object_mut() {
        obj.insert("actual_amount".to_string(), json!(actual_amount));
        obj.insert("status".to_string(), json!(status));
    }

    let _ = crate::api::generic::generic_update(
        &pool,
        "budgets",
        &id,
        body,
        &user.company_id,
        &user.user_id,
    )
    .await?;

    // Audit log
    let log_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO operation_logs (id, company_id, user_id, action, entity_type, entity_id) VALUES (?, ?, ?, 'update', 'budgets', ?)")
        .bind(&log_id).bind(&user.company_id).bind(&user.user_id).bind(&id)
        .execute(&pool).await.ok();

    let budget: Budget = sqlx::query_as("SELECT * FROM budgets WHERE id = ? AND company_id = ?")
        .bind(&id)
        .bind(&user.company_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| api_err(StatusCode::NOT_FOUND, "budget not found"))?;

    Ok(Json(budget))
}

async fn budget_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(q): Query<BudgetOverviewQuery>,
) -> Result<Json<BudgetOverviewResponse>, ApiError> {
    let budget_type = q.budget_type.unwrap_or_else(|| "monthly".to_string());
    let period = q
        .period
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| default_period_for_type(&budget_type));
    period_range(&budget_type, &period)?;

    let budgets: Vec<Budget> = sqlx::query_as(
        "SELECT * FROM budgets WHERE company_id = ? AND budget_type = ? AND period = ? ORDER BY category ASC",
    )
    .bind(&user.company_id)
    .bind(&budget_type)
    .bind(&period)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let mut rows = Vec::new();
    let mut stats = BudgetStats::default();
    let can_delete = current_user_can_manage_budgets(&pool, &user).await?;

    for budget in budgets {
        let budget_amount = budget.budget_amount.unwrap_or(0.0);
        let actual_amount = calculate_actual_expense(
            &pool,
            &user.company_id,
            &budget.budget_type,
            &budget.period,
            &budget.category,
        )
        .await?;
        let is_exceeded = actual_amount > budget_amount;
        let percentage = if budget_amount > 0.0 {
            (actual_amount / budget_amount) * 100.0
        } else {
            0.0
        };
        let saved = budget_amount - actual_amount;

        stats.total_budget += budget_amount;
        stats.total_actual += actual_amount;
        if is_exceeded {
            stats.exceeded_count += 1;
        }
        if saved > 0.0 {
            stats.saved_amount += saved;
        }
        let (category_label, category_icon, category_class) =
            budget_category_display(&budget.category);
        let delete_action =
            budget_delete_action(&budget.id, category_label, &budget.period, can_delete);
        let update_action = budget_update_action(&budget.id, can_delete);
        let remaining_label = if is_exceeded {
            format!("超支 {}", budget_format_cny(actual_amount - budget_amount))
        } else {
            format!("剩余 {}", budget_format_cny(budget_amount - actual_amount))
        };
        let progress_value = percentage.min(100.0);

        rows.push(BudgetExecution {
            id: budget.id,
            category: budget.category,
            budget_type: budget.budget_type,
            period: budget.period,
            budget_amount,
            actual_amount,
            status: budget_status(actual_amount, budget_amount).to_string(),
            notes: budget.notes,
            percentage,
            is_exceeded,
            category_label: category_label.to_string(),
            category_icon: category_icon.to_string(),
            category_class: category_class.to_string(),
            status_badge: budget_status_badge(is_exceeded, percentage),
            budget_amount_label: budget_format_cny(budget_amount),
            actual_amount_label: budget_format_cny(actual_amount),
            usage_label: format!("{} 已使用", budget_percent_label(percentage)),
            remaining_label,
            progress_value,
            progress_class: budget_progress_class(is_exceeded, percentage),
            update_action,
            can_delete,
            delete_action,
        });
    }

    stats.utilization = if stats.total_budget > 0.0 {
        (stats.total_actual / stats.total_budget) * 100.0
    } else {
        0.0
    };
    stats.total_budget_label = budget_format_cny(stats.total_budget);
    stats.total_actual_label = budget_format_cny(stats.total_actual);
    stats.saved_amount_label = budget_format_cny(stats.saved_amount);
    stats.utilization_label = budget_percent_label(stats.utilization);

    let mut trend = Vec::new();
    for offset in -5..=0 {
        let trend_period = shift_period(&budget_type, &period, offset)?;
        let period_budgets: Vec<Budget> = sqlx::query_as(
            "SELECT * FROM budgets WHERE company_id = ? AND budget_type = ? AND period = ?",
        )
        .bind(&user.company_id)
        .bind(&budget_type)
        .bind(&trend_period)
        .fetch_all(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

        let mut budget_total = 0.0;
        let mut actual_total = 0.0;
        for budget in period_budgets {
            budget_total += budget.budget_amount.unwrap_or(0.0);
            actual_total += calculate_actual_expense(
                &pool,
                &user.company_id,
                &budget.budget_type,
                &budget.period,
                &budget.category,
            )
            .await?;
        }

        trend.push(BudgetTrendPoint {
            period_label: budget_period_label(&budget_type, &trend_period),
            period: trend_period.clone(),
            budget: budget_total,
            actual: actual_total,
            budget_label: budget_format_cny(budget_total),
            actual_label: budget_format_cny(actual_total),
        });
    }

    let empty_message = if rows.is_empty() {
        "暂无预算设置，点击\"新建预算\"开始设置"
    } else {
        ""
    }
    .to_string();

    Ok(Json(BudgetOverviewResponse {
        budget_type,
        period,
        budgets: rows,
        stats,
        trend,
        budget_type_options: budget_type_options(),
        create_action: budget_create_action(can_delete),
        permissions: json!({
            "can_create": can_delete,
            "can_update": can_delete,
            "can_delete": can_delete,
        }),
        empty_message,
    }))
}

fn budget_create_action(can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }

    json!({
        "method": "POST",
        "endpoint": "/api/budgets",
        "label": "新建预算",
        "success_title": "创建成功",
        "success_message": "预算已创建",
    })
}

fn budget_update_action(id: &str, can_manage: bool) -> Value {
    if !can_manage {
        return json!(null);
    }

    json!({
        "method": "PUT",
        "endpoint": format!("/api/budgets/{id}"),
        "label": "保存修改",
        "success_title": "更新成功",
        "success_message": "预算已更新",
    })
}

fn budget_delete_action(id: &str, category_label: &str, period: &str, can_delete: bool) -> Value {
    if !can_delete {
        return json!(null);
    }

    json!({
        "method": "DELETE",
        "endpoint": format!("/api/budgets/{id}"),
        "confirm_message": format!("确定要删除预算「{category_label} {period}」吗？"),
        "success_title": "删除成功",
        "success_message": "预算已删除",
    })
}

async fn delete_budget(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    if !current_user_can_manage_budgets(&pool, &user).await? {
        return Err(api_err(StatusCode::FORBIDDEN, "forbidden"));
    }

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let result = sqlx::query("DELETE FROM budgets WHERE id = ? AND company_id = ?")
        .bind(&id)
        .bind(&user.company_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    if result.rows_affected() == 0 {
        return Err(api_err(StatusCode::NOT_FOUND, "budget not found"));
    }

    // Audit log
    let log_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO operation_logs (id, company_id, user_id, action, entity_type, entity_id) VALUES (?, ?, ?, 'delete', 'budgets', ?)")
        .bind(&log_id).bind(&user.company_id).bind(&user.user_id).bind(&id)
        .execute(&mut *tx).await.ok();

    tx.commit()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(StatusCode::NO_CONTENT)
}

async fn calculate_actual_expense(
    pool: &SqlitePool,
    company_id: &str,
    budget_type: &str,
    period: &str,
    category: &str,
) -> Result<f64, ApiError> {
    let range = period_range(budget_type, period)?;

    let total = match category {
        "salary" => {
            let (total,): (f64,) = sqlx::query_as(
                "SELECT COALESCE(SUM(COALESCE(base_salary, 0.0) + COALESCE(bonus, 0.0)), 0.0)
                 FROM salary_records
                 WHERE company_id = ? AND period >= ? AND period < ?",
            )
            .bind(company_id)
            .bind(&range.start_month)
            .bind(&range.end_month)
            .fetch_one(pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
            total
        }
        "invoice" => {
            let (total,): (f64,) = sqlx::query_as(
                "SELECT COALESCE(SUM(COALESCE(amount, 0.0)), 0.0)
                 FROM invoices
                 WHERE company_id = ? AND invoice_date >= ? AND invoice_date < ?",
            )
            .bind(company_id)
            .bind(&range.start_date)
            .bind(&range.end_date)
            .fetch_one(pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
            total
        }
        "rent" => {
            let (monthly_total,): (f64,) = sqlx::query_as(
                "SELECT COALESCE(SUM(COALESCE(monthly_rent, 0.0)), 0.0)
                 FROM rent_records
                 WHERE company_id = ?
                   AND status = 'active'
                   AND contract_start_date IS NOT NULL
                   AND contract_end_date IS NOT NULL
                   AND contract_start_date < ?
                   AND contract_end_date >= ?",
            )
            .bind(company_id)
            .bind(&range.end_date)
            .bind(&range.start_date)
            .fetch_one(pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
            monthly_total * range.months as f64
        }
        "utility" => {
            let (total,): (f64,) = sqlx::query_as(
                "SELECT COALESCE(SUM(COALESCE(amount, 0.0)), 0.0)
                 FROM utilities
                 WHERE company_id = ? AND period >= ? AND period < ?",
            )
            .bind(company_id)
            .bind(&range.start_month)
            .bind(&range.end_month)
            .fetch_one(pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
            total
        }
        _ => 0.0,
    };

    Ok(total)
}

fn budget_status(actual_amount: f64, budget_amount: f64) -> &'static str {
    if actual_amount > budget_amount {
        "exceeded"
    } else {
        "active"
    }
}

fn current_period() -> String {
    Utc::now().format("%Y-%m").to_string()
}

fn default_period_for_type(budget_type: &str) -> String {
    let now = Utc::now();
    match budget_type {
        "quarterly" => format!("{}-Q{}", now.year(), ((now.month() - 1) / 3) + 1),
        "annual" | "project" => now.year().to_string(),
        _ => current_period(),
    }
}

fn period_range(budget_type: &str, period: &str) -> Result<PeriodRange, ApiError> {
    match budget_type {
        "monthly" => {
            let (year, month) = parse_month_period(period)?;
            let (end_year, end_month) = add_months(year, month, 1);
            Ok(PeriodRange {
                start_date: format!("{year:04}-{month:02}-01"),
                end_date: format!("{end_year:04}-{end_month:02}-01"),
                start_month: format!("{year:04}-{month:02}"),
                end_month: format!("{end_year:04}-{end_month:02}"),
                months: 1,
            })
        }
        "quarterly" => {
            let (year, quarter) = parse_quarter_period(period)?;
            let start_month = (quarter - 1) * 3 + 1;
            let (end_year, end_month) = add_months(year, start_month, 3);
            Ok(PeriodRange {
                start_date: format!("{year:04}-{start_month:02}-01"),
                end_date: format!("{end_year:04}-{end_month:02}-01"),
                start_month: format!("{year:04}-{start_month:02}"),
                end_month: format!("{end_year:04}-{end_month:02}"),
                months: 3,
            })
        }
        "annual" | "project" => {
            let year = period
                .parse::<i32>()
                .map_err(|_| api_err(StatusCode::BAD_REQUEST, "invalid period"))?;
            Ok(PeriodRange {
                start_date: format!("{year:04}-01-01"),
                end_date: format!("{:04}-01-01", year + 1),
                start_month: format!("{year:04}-01"),
                end_month: format!("{:04}-01", year + 1),
                months: 12,
            })
        }
        _ => Err(api_err(StatusCode::BAD_REQUEST, "invalid budget_type")),
    }
}

fn shift_period(budget_type: &str, period: &str, offset: i32) -> Result<String, ApiError> {
    match budget_type {
        "monthly" => {
            let (year, month) = parse_month_period(period)?;
            let (shifted_year, shifted_month) = add_months(year, month, offset);
            Ok(format!("{shifted_year:04}-{shifted_month:02}"))
        }
        "quarterly" => {
            let (year, quarter) = parse_quarter_period(period)?;
            let start_month = (quarter - 1) * 3 + 1;
            let (shifted_year, shifted_month) = add_months(year, start_month, offset * 3);
            let shifted_quarter = ((shifted_month - 1) / 3) + 1;
            Ok(format!("{shifted_year:04}-Q{shifted_quarter}"))
        }
        "annual" | "project" => {
            let year = period
                .parse::<i32>()
                .map_err(|_| api_err(StatusCode::BAD_REQUEST, "invalid period"))?;
            Ok(format!("{:04}", year + offset))
        }
        _ => Err(api_err(StatusCode::BAD_REQUEST, "invalid budget_type")),
    }
}

fn parse_month_period(period: &str) -> Result<(i32, u32), ApiError> {
    let mut parts = period.split('-');
    let year = parts
        .next()
        .and_then(|value| value.parse::<i32>().ok())
        .ok_or_else(|| api_err(StatusCode::BAD_REQUEST, "invalid period"))?;
    let month = parts
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|month| (1..=12).contains(month))
        .ok_or_else(|| api_err(StatusCode::BAD_REQUEST, "invalid period"))?;

    Ok((year, month))
}

fn parse_quarter_period(period: &str) -> Result<(i32, u32), ApiError> {
    let (year, quarter) = period
        .split_once("-Q")
        .ok_or_else(|| api_err(StatusCode::BAD_REQUEST, "invalid period"))?;
    let year = year
        .parse::<i32>()
        .map_err(|_| api_err(StatusCode::BAD_REQUEST, "invalid period"))?;
    let quarter = quarter
        .parse::<u32>()
        .ok()
        .filter(|quarter| (1..=4).contains(quarter))
        .ok_or_else(|| api_err(StatusCode::BAD_REQUEST, "invalid period"))?;

    Ok((year, quarter))
}

fn add_months(year: i32, month: u32, offset: i32) -> (i32, u32) {
    let total_months = year * 12 + month as i32 - 1 + offset;
    let next_year = total_months.div_euclid(12);
    let next_month = total_months.rem_euclid(12) + 1;
    (next_year, next_month as u32)
}
