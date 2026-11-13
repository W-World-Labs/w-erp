use super::error::{ApiError, api_err};
use crate::auth::AuthUser;
use crate::db::models::Employee;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::SqlitePool;

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/", get(list_employees).post(create_employee))
        .route("/overview", get(employees_overview))
        .route("/form_options", get(employee_form_options))
        .route(
            "/{id}",
            get(get_employee)
                .put(update_employee)
                .delete(delete_employee),
        )
        .with_state(pool)
}

#[derive(Deserialize)]
pub struct EmployeeRequest {
    pub name: Option<String>,
    pub full_name: Option<String>,
    pub employee_number: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub department: Option<String>,
    pub position: Option<String>,
    pub hire_date: Option<String>,
    pub employment_type: Option<String>,
    pub status: Option<String>,
    pub base_salary: Option<Value>,
    pub social_security_base: Option<Value>,
    pub housing_fund_base: Option<Value>,
    pub notes: Option<String>,
    pub avatar_url: Option<String>,
    pub contract_start_date: Option<String>,
    pub contract_end_date: Option<String>,
    pub bank_account: Option<String>,
    pub bank_name: Option<String>,
    pub emergency_contact: Option<String>,
    pub emergency_phone: Option<String>,
    pub id_number: Option<String>,
}

#[derive(Serialize)]
pub struct EmployeesResponse {
    pub data: Vec<Employee>,
    pub count: i64,
}

#[derive(Serialize)]
struct EmployeeStats {
    total: i64,
    active: i64,
    resigned: i64,
    suspended: i64,
}

#[derive(Serialize)]
struct EmployeesOverview {
    stats: EmployeeStats,
    employees: Vec<Value>,
    permissions: Value,
    status_options: Vec<Value>,
    employment_type_options: Vec<Value>,
    empty_message: &'static str,
    create_action: Value,
}

async fn list_employees(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(q): Query<super::generic::ListQuery>,
) -> Result<Json<EmployeesResponse>, ApiError> {
    let limit = q.limit.or(q.page_size).unwrap_or(100).min(1000).max(1);
    let offset = q
        .offset
        .or_else(|| q.page.map(|p| (p.max(1) - 1) * limit))
        .unwrap_or(0)
        .max(0);
    let employees: Vec<Employee> = sqlx::query_as(
        "SELECT * FROM employees WHERE company_id = ? ORDER BY name LIMIT ? OFFSET ?",
    )
    .bind(&user.company_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM employees WHERE company_id = ?")
        .bind(&user.company_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(Json(EmployeesResponse {
        data: employees,
        count,
    }))
}

async fn employees_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<EmployeesOverview>, ApiError> {
    let employees: Vec<Employee> =
        sqlx::query_as("SELECT * FROM employees WHERE company_id = ? ORDER BY name")
            .bind(&user.company_id)
            .fetch_all(&pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let can_delete = current_user_can_delete_employees(&pool, &user).await?;
    let stats = EmployeeStats {
        total: employees.len() as i64,
        active: employees
            .iter()
            .filter(|employee| employee.status.as_deref().unwrap_or("active") == "active")
            .count() as i64,
        resigned: employees
            .iter()
            .filter(|employee| employee.status.as_deref() == Some("resigned"))
            .count() as i64,
        suspended: employees
            .iter()
            .filter(|employee| employee.status.as_deref() == Some("suspended"))
            .count() as i64,
    };
    let employees = employees
        .into_iter()
        .map(|employee| employee_overview_item(&employee, can_delete))
        .collect();

    Ok(Json(EmployeesOverview {
        stats,
        employees,
        permissions: json!({ "can_delete": can_delete }),
        status_options: employee_status_options(),
        employment_type_options: employee_type_options(),
        empty_message: "还没有员工记录",
        create_action: employee_create_action(true),
    }))
}

async fn current_user_can_delete_employees(
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

fn employee_overview_item(employee: &Employee, can_delete: bool) -> Value {
    let status = employee.status.as_deref().unwrap_or("active");
    let employment_type = employee.employment_type.as_deref().unwrap_or("full_time");
    let display_name = employee
        .full_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&employee.name)
        .to_string();
    let hire_date_label = employee
        .hire_date
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "未设置".to_string());
    let base_salary = employee.base_salary.unwrap_or(0.0);
    let delete_action = employee_delete_action(&employee.id, &display_name, can_delete);

    let mut value = serde_json::to_value(employee).unwrap_or_else(|_| json!({}));
    if let Some(obj) = value.as_object_mut() {
        obj.insert("display_name".to_string(), json!(display_name));
        obj.insert("status".to_string(), json!(status));
        obj.insert("status_badge".to_string(), employee_status_badge(status));
        obj.insert("employment_type".to_string(), json!(employment_type));
        obj.insert(
            "employment_type_badge".to_string(),
            employee_type_badge(employment_type),
        );
        obj.insert("hire_date_label".to_string(), json!(hire_date_label));
        obj.insert(
            "base_salary_label".to_string(),
            json!(if base_salary > 0.0 {
                format_cny(base_salary)
            } else {
                "-".to_string()
            }),
        );
        obj.insert("can_delete".to_string(), json!(can_delete));
        obj.insert("delete_action".to_string(), delete_action);
        obj.insert(
            "update_action".to_string(),
            employee_update_action(&employee.id, true),
        );
    }
    value
}

fn employee_create_action(can_create: bool) -> Value {
    if !can_create {
        return json!(null);
    }

    json!({
        "method": "POST",
        "endpoint": "/api/employees",
        "label": "添加员工",
        "success_title": "创建成功",
        "success_message": "员工添加成功",
    })
}

fn employee_update_action(id: &str, can_update: bool) -> Value {
    if !can_update {
        return json!(null);
    }

    json!({
        "method": "PUT",
        "endpoint": format!("/api/employees/{id}"),
        "label": "保存员工",
        "success_title": "更新成功",
        "success_message": "员工更新成功",
    })
}

fn employee_delete_action(id: &str, display_name: &str, can_delete: bool) -> Value {
    if !can_delete {
        return json!(null);
    }
    let display_name = display_name.trim();
    let display_name = if display_name.is_empty() {
        "该员工"
    } else {
        display_name
    };

    json!({
        "method": "DELETE",
        "endpoint": format!("/api/employees/{id}"),
        "confirm_message": format!("确定要删除员工「{display_name}」吗？如果该员工有关联的薪资或社保记录，删除将被阻止。"),
        "success_title": "删除成功",
        "success_message": "员工已删除",
    })
}

fn employee_status_badge(status: &str) -> Value {
    match status {
        "resigned" => json!({
            "value": "resigned",
            "label": "离职",
            "variant": "secondary",
        }),
        "suspended" => json!({
            "value": "suspended",
            "label": "停职",
            "variant": "outline",
        }),
        _ => json!({
            "value": "active",
            "label": "在职",
            "variant": "default",
        }),
    }
}

fn employee_status_options() -> Vec<Value> {
    ["active", "resigned", "suspended"]
        .into_iter()
        .map(employee_status_badge)
        .collect()
}

fn employee_type_badge(employment_type: &str) -> Value {
    match employment_type {
        "part_time" => json!({
            "value": "part_time",
            "label": "兼职",
            "variant": "outline",
        }),
        "intern" => json!({
            "value": "intern",
            "label": "实习",
            "variant": "outline",
        }),
        "contract" => json!({
            "value": "contract",
            "label": "合同工",
            "variant": "outline",
        }),
        _ => json!({
            "value": "full_time",
            "label": "全职",
            "variant": "outline",
        }),
    }
}

fn employee_type_options() -> Vec<Value> {
    ["full_time", "part_time", "intern", "contract"]
        .into_iter()
        .map(employee_type_badge)
        .collect()
}

async fn employee_form_options() -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({
        "status_options": employee_status_options(),
        "employment_type_options": employee_type_options(),
        "default_values": {
            "status": "active",
            "employment_type": "full_time",
        },
    })))
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

async fn get_employee(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Employee>, ApiError> {
    let emp: Option<Employee> =
        sqlx::query_as("SELECT * FROM employees WHERE id = ? AND company_id = ?")
            .bind(&id)
            .bind(&user.company_id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    emp.map(Json)
        .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "employee not found"))
}

async fn create_employee(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Json(req): Json<EmployeeRequest>,
) -> Result<Json<Employee>, ApiError> {
    let name = req
        .name
        .as_deref()
        .or(req.full_name.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| api_err(StatusCode::BAD_REQUEST, "name is required"))?
        .to_string();
    let full_name = req
        .full_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| name.clone());
    let base_salary = parse_optional_employee_money("base_salary", req.base_salary.as_ref())?;
    let social_security_base =
        parse_optional_employee_money("social_security_base", req.social_security_base.as_ref())?;
    let housing_fund_base =
        parse_optional_employee_money("housing_fund_base", req.housing_fund_base.as_ref())?;
    let hire_date = normalize_optional_employee_date("hire_date", req.hire_date.as_deref())?;
    let contract_start_date = normalize_optional_employee_date(
        "contract_start_date",
        req.contract_start_date.as_deref(),
    )?;
    let contract_end_date =
        normalize_optional_employee_date("contract_end_date", req.contract_end_date.as_deref())?;

    // R2-3: Validate non-negative salary fields
    if let Some(sal) = base_salary
        && sal < 0.0
    {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "base_salary must be non-negative",
        ));
    }
    if let Some(ss) = social_security_base
        && ss < 0.0
    {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "social_security_base must be non-negative",
        ));
    }
    if let Some(hf) = housing_fund_base
        && hf < 0.0
    {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "housing_fund_base must be non-negative",
        ));
    }

    let id = uuid::Uuid::new_v4().to_string();

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    sqlx::query(
        "INSERT INTO employees (
            id, company_id, name, full_name, employee_number, email, phone, department, position, hire_date, 
            employment_type, status, base_salary, social_security_base, housing_fund_base, 
            notes, avatar_url, contract_start_date, contract_end_date, 
            bank_account, bank_name, emergency_contact, emergency_phone, id_number
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&user.company_id)
    .bind(&name)
    .bind(&full_name)
    .bind(&req.employee_number)
    .bind(&req.email)
    .bind(&req.phone)
    .bind(&req.department)
    .bind(&req.position)
    .bind(&hire_date)
    .bind(&req.employment_type)
    .bind(&req.status)
    .bind(base_salary)
    .bind(social_security_base)
    .bind(housing_fund_base)
    .bind(&req.notes)
    .bind(&req.avatar_url)
    .bind(&contract_start_date)
    .bind(&contract_end_date)
    .bind(&req.bank_account)
    .bind(&req.bank_name)
    .bind(&req.emergency_contact)
    .bind(&req.emergency_phone)
    .bind(&req.id_number)
    .execute(&mut *tx)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    // Audit log
    let log_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO operation_logs (id, company_id, user_id, action, entity_type, entity_id) VALUES (?, ?, ?, 'create', 'employees', ?)")
        .bind(&log_id).bind(&user.company_id).bind(&user.user_id).bind(&id)
        .execute(&mut *tx).await.ok();

    tx.commit()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    // R2-1: Re-fetch with company_id scoping
    let emp: Employee = sqlx::query_as("SELECT * FROM employees WHERE id = ? AND company_id = ?")
        .bind(&id)
        .bind(&user.company_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(Json(emp))
}

fn parse_optional_employee_money(
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

fn normalize_optional_employee_date(
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

async fn update_employee(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Employee>, ApiError> {
    // Use generic_update for partial update support (no longer requires all 20+ fields)
    let _ = crate::api::generic::generic_update(
        &pool,
        "employees",
        &id,
        body,
        &user.company_id,
        &user.user_id,
    )
    .await?;

    // Audit log
    let log_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO operation_logs (id, company_id, user_id, action, entity_type, entity_id) VALUES (?, ?, ?, 'update', 'employees', ?)")
        .bind(&log_id).bind(&user.company_id).bind(&user.user_id).bind(&id)
        .execute(&pool).await.ok();

    let emp: Employee = sqlx::query_as("SELECT * FROM employees WHERE id = ? AND company_id = ?")
        .bind(&id)
        .bind(&user.company_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| api_err(StatusCode::NOT_FOUND, "employee not found"))?;

    Ok(Json(emp))
}

async fn delete_employee(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    // Check for dependent salary records before deleting
    if let Ok((n,)) = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM salary_records WHERE employee_id = ? AND company_id = ?",
    )
    .bind(&id)
    .bind(&user.company_id)
    .fetch_one(&pool)
    .await
        && n > 0
    {
        return Err(api_err(
            StatusCode::CONFLICT,
            "cannot delete: has dependent salary records",
        ));
    }

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let result = sqlx::query("DELETE FROM employees WHERE id = ? AND company_id = ?")
        .bind(&id)
        .bind(&user.company_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    if result.rows_affected() == 0 {
        return Err(api_err(StatusCode::NOT_FOUND, "employee not found"));
    }

    // Audit log
    let log_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO operation_logs (id, company_id, user_id, action, entity_type, entity_id) VALUES (?, ?, ?, 'delete', 'employees', ?)")
        .bind(&log_id).bind(&user.company_id).bind(&user.user_id).bind(&id)
        .execute(&mut *tx).await.ok();

    tx.commit()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(StatusCode::NO_CONTENT)
}
