use axum::http::StatusCode;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{FromRow, SqlitePool};

use crate::api::error::{ApiError, api_err};
use crate::api::generic::{
    ListQuery, generic_create, generic_delete, generic_get, generic_list, generic_update,
};
use crate::auth::AuthUser;

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/", get(list_policies))
        .route("/match", get(match_policies))
        .route("/dashboard/overview", get(dashboard_overview))
        .route("/{id}", get(get_policy))
        .route(
            "/applications",
            get(list_applications).post(create_application),
        )
        .route("/applications/overview", get(applications_overview))
        .route(
            "/applications/{id}",
            get(get_application)
                .put(update_application)
                .delete(delete_application),
        )
        .with_state(pool)
}

#[derive(Serialize, FromRow, Clone)]
pub struct Policy {
    pub id: String,
    pub name: String,
    pub name_en: Option<String>,
    pub name_local: Option<String>,
    pub country: String,
    pub region: Option<String>,
    pub category: String,
    pub subcategory: Option<String>,
    pub provider: Option<String>,
    pub description: Option<String>,
    pub eligibility: Option<String>,
    pub benefits: Option<String>,
    pub amount_min: Option<f64>,
    pub amount_max: Option<f64>,
    pub currency: Option<String>,
    pub equity_required: Option<f64>,
    pub deadline: Option<String>,
    pub cycle: Option<String>,
    pub url: Option<String>,
    pub match_industries: Option<String>,
    pub match_min_employees: Option<i64>,
    pub match_max_employees: Option<i64>,
    pub match_min_revenue: Option<f64>,
    pub match_max_revenue: Option<f64>,
    pub match_countries: Option<String>,
    pub match_company_age_max: Option<i64>,
    pub match_founder_education: Option<String>,
    pub match_founder_is_expat: Option<i64>,
    pub match_founder_is_overseas_returnee: Option<i64>,
    pub is_active: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Clone)]
pub struct ScoredPolicy {
    #[serde(flatten)]
    pub policy: Policy,
    pub match_score: i32,
    pub match_reasons: Vec<String>,
}

#[derive(Serialize)]
pub struct MatchResponse {
    pub data: Vec<Value>,
    pub count: usize,
}

#[derive(Deserialize)]
struct PolicyListQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    page: Option<i64>,
    page_size: Option<i64>,
    search: Option<String>,
    category: Option<String>,
    country: Option<String>,
}

#[derive(Serialize)]
struct PolicyListResponse {
    data: Vec<Value>,
    count: i64,
    category_options: Vec<Value>,
    country_options: Vec<Value>,
}

// -----------------------------------------------------------------------------
// Policies (Global, Read-Only for regular users)
// -----------------------------------------------------------------------------

async fn list_policies(
    State(pool): State<SqlitePool>,
    _user: AuthUser,
    Query(q): Query<PolicyListQuery>,
) -> Result<Json<PolicyListResponse>, ApiError> {
    Ok(Json(policy_list_response(&pool, &q).await?))
}

async fn policy_list_response(
    pool: &SqlitePool,
    q: &PolicyListQuery,
) -> Result<PolicyListResponse, ApiError> {
    let limit = q.limit.or(q.page_size).unwrap_or(100).min(1000).max(1);
    let offset = q
        .offset
        .or_else(|| q.page.map(|p| (p.max(1) - 1) * limit))
        .unwrap_or(0)
        .max(0);

    let items = sqlx::query_as::<_, Policy>(
        "SELECT * FROM policies WHERE is_active = 1 ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("db error: {}", e),
        )
    })?;

    let search = q
        .search
        .as_deref()
        .unwrap_or_default()
        .trim()
        .to_lowercase();
    let category = normalized_filter(q.category.as_deref());
    let country = normalized_filter(q.country.as_deref());
    let mut filtered = items
        .into_iter()
        .filter(|policy| {
            category
                .as_deref()
                .is_none_or(|value| policy.category == value)
                && country
                    .as_deref()
                    .is_none_or(|value| policy.country == value)
                && (search.is_empty()
                    || policy.name.to_lowercase().contains(&search)
                    || policy
                        .name_en
                        .as_deref()
                        .is_some_and(|value| value.to_lowercase().contains(&search))
                    || policy
                        .description
                        .as_deref()
                        .is_some_and(|value| value.to_lowercase().contains(&search)))
        })
        .map(|policy| policy_display_value(&policy))
        .collect::<Vec<_>>();
    let count = filtered.len() as i64;
    let end = (offset as usize).saturating_add(limit as usize);
    let data = if offset as usize >= filtered.len() {
        Vec::new()
    } else {
        filtered
            .drain(offset as usize..filtered.len().min(end))
            .collect()
    };

    Ok(PolicyListResponse {
        data,
        count,
        category_options: policy_category_options(),
        country_options: policy_country_options(),
    })
}

async fn get_policy(
    State(pool): State<SqlitePool>,
    _user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let policy =
        sqlx::query_as::<_, Policy>("SELECT * FROM policies WHERE id = ? AND is_active = 1")
            .bind(&id)
            .fetch_one(&pool)
            .await
            .map_err(|_| api_err(StatusCode::NOT_FOUND, "policy not found"))?;

    Ok(Json(policy_display_value(&policy)))
}

// -----------------------------------------------------------------------------
// Smart Matching Engine
// -----------------------------------------------------------------------------
async fn match_policies(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<MatchResponse>, ApiError> {
    let data = policy_match_data(&pool, &user).await?;

    Ok(Json(MatchResponse {
        count: data.len(),
        data,
    }))
}

async fn policy_match_data(pool: &SqlitePool, user: &AuthUser) -> Result<Vec<Value>, ApiError> {
    // 1. Extract contextual company data for matching
    let (company_country,): (Option<String>,) =
        sqlx::query_as("SELECT country FROM companies WHERE id = ?")
            .bind(&user.company_id)
            .fetch_one(pool)
            .await
            .unwrap_or((None,));

    let company_country = company_country.unwrap_or_else(|| "US".to_string());

    let (emp_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM employees WHERE company_id = ? AND status = 'active'")
            .bind(&user.company_id)
            .fetch_one(pool)
            .await
            .unwrap_or((0,));

    // Approximate revenue from invoices
    let (revenue,): (f64,) = sqlx::query_as("SELECT SUM(amount) FROM invoices WHERE company_id = ? AND invoice_type = 'receivable' AND status = 'paid'")
        .bind(&user.company_id)
        .fetch_one(pool)
        .await
        .unwrap_or((0.0,));

    // Company age (months)
    let (months_old, founder_edu, founder_expat, founder_returnee): (
        i64,
        Option<String>,
        i64,
        i64,
    ) = sqlx::query_as(
        "SELECT IFNULL(CAST((julianday('now') - julianday(registration_date))/30 AS INTEGER), 12),
                founder_education, founder_is_expat, founder_is_overseas_returnee 
         FROM company_info WHERE company_id = ?",
    )
    .bind(&user.company_id)
    .fetch_one(pool)
    .await
    .unwrap_or((12, None, 0, 0)); // Default 1 year if not found

    // 2. Fetch all active policies
    let policies = sqlx::query_as::<_, Policy>("SELECT * FROM policies WHERE is_active = 1")
        .fetch_all(pool)
        .await
        .unwrap_or_default();

    // 3. Scoring algorithm
    let mut scored = Vec::new();

    for p in policies {
        let mut score = 0;
        let mut reasons = Vec::new();

        // Location match (Highest priority)
        if let Some(ref mc) = p.match_countries
            && (mc.contains(&company_country) || mc.contains("*"))
        {
            score += 40;
            reasons.push(format!("Location match: {}", company_country));
        }

        // Employee count match
        if p.match_min_employees.is_none_or(|min| emp_count >= min)
            && p.match_max_employees.is_none_or(|max| emp_count <= max)
        {
            score += 20;
            reasons.push("Employee criteria met".to_string());
        }

        // Revenue match
        if p.match_min_revenue.is_none_or(|min| revenue >= min)
            && p.match_max_revenue.is_none_or(|max| revenue <= max)
        {
            score += 20;
            reasons.push("Revenue criteria met".to_string());
        }

        // Age match
        if let Some(max_age) = p.match_company_age_max {
            if months_old <= max_age {
                score += 20;
                reasons.push("Early stage criteria met".to_string());
            }
        } else {
            score += 10;
        }

        // Founder matches (OPC logic)
        if let Some(ref req_edu) = p.match_founder_education
            && let Some(ref edu) = founder_edu
            && req_edu.contains(edu)
        {
            score += 15;
            reasons.push(format!("Founder education ({}) match", edu));
        }

        if let Some(req_expat) = p.match_founder_is_expat
            && req_expat == founder_expat
        {
            score += 15;
            if req_expat == 1 {
                reasons.push("Expat founder criteria met".to_string());
            }
        }

        if let Some(req_returnee) = p.match_founder_is_overseas_returnee
            && req_returnee == founder_returnee
        {
            score += 15;
            if req_returnee == 1 {
                reasons.push("Overseas returnee criteria met".to_string());
            }
        }

        // Only recommend high-match policies
        if score >= 40 {
            scored.push(ScoredPolicy {
                policy: p,
                match_score: score,
                match_reasons: reasons,
            });
        }
    }

    // Sort by score descending
    scored.sort_by(|a, b| b.match_score.cmp(&a.match_score));

    // Optional: Take top 20
    scored.truncate(20);
    let data = scored
        .into_iter()
        .enumerate()
        .map(|(index, item)| scored_policy_display_value(item, index))
        .collect::<Vec<_>>();

    Ok(data)
}

async fn dashboard_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(q): Query<PolicyListQuery>,
) -> Result<Json<Value>, ApiError> {
    let policy_list = policy_list_response(&pool, &q).await?;
    let matched_policies = policy_match_data(&pool, &user).await?;
    let match_count = matched_policies.len();
    let applications = policy_applications_overview_data(&pool, &user.company_id).await?;

    Ok(Json(json!({
        "policies": policy_list.data,
        "policy_count": policy_list.count,
        "category_options": policy_list.category_options,
        "country_options": policy_list.country_options,
        "matched_policies": matched_policies,
        "match_count": match_count,
        "applications": applications,
        "applications_empty_message": "追踪看板还是空的",
    })))
}

// -----------------------------------------------------------------------------
// Policy Applications Tracker
// -----------------------------------------------------------------------------
async fn applications_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<Value>, ApiError> {
    let applications = policy_applications_overview_data(&pool, &user.company_id).await?;

    Ok(Json(json!({
        "applications": applications,
        "empty_message": "追踪看板还是空的",
    })))
}

async fn policy_applications_overview_data(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<Vec<Value>, ApiError> {
    let rows: Vec<(
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
    )> = sqlx::query_as(
        r#"
        SELECT
            pa.id,
            pa.policy_id,
            COALESCE(p.name, '未知政策') AS policy_name,
            pa.status,
            pa.applied_date,
            pa.result_date,
            pa.notes,
            pa.updated_at
        FROM policy_applications pa
        LEFT JOIN policies p ON p.id = pa.policy_id
        WHERE pa.company_id = ?
        ORDER BY pa.updated_at DESC, pa.created_at DESC
        "#,
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(rows
        .into_iter()
        .map(
            |(id, policy_id, policy_name, status, applied_date, result_date, notes, updated_at)| {
                let status = status.unwrap_or_else(|| "interested".to_string());
                json!({
                    "id": id,
                    "policy_id": policy_id,
                    "policy_name": policy_name,
                    "status": status,
                    "status_badge": policy_application_status_badge(&status),
                    "applied_date": applied_date,
                    "applied_date_label": date_label(applied_date.as_deref()),
                    "result_date": result_date,
                    "result_date_label": date_label(result_date.as_deref()),
                    "notes": notes,
                    "updated_at": updated_at,
                    "updated_at_label": date_label(Some(&updated_at)),
                })
            },
        )
        .collect::<Vec<_>>())
}

async fn list_applications(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<Value>, ApiError> {
    generic_list(&pool, "policy_applications", &user.company_id, q).await
}

async fn create_application(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let policy_id = body
        .get("policy_id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();
    let policy_name = if policy_id.is_empty() {
        String::new()
    } else {
        sqlx::query_scalar::<_, String>("SELECT name FROM policies WHERE id = ?")
            .bind(&policy_id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "policy lookup failed"))?
            .unwrap_or_else(|| policy_id.clone())
    };

    let body = json!({
        "policy_id": policy_id,
        "status": "preparing",
        "notes": if policy_name.is_empty() {
            Value::Null
        } else {
            json!(format!("已从政策大厅加入追踪：{policy_name}"))
        },
    });

    generic_create(
        &pool,
        "policy_applications",
        body,
        &["policy_id"],
        &user.company_id,
        &user.user_id,
    )
    .await
}

async fn get_application(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    generic_get(&pool, "policy_applications", &id, &user.company_id).await
}

async fn update_application(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    generic_update(
        &pool,
        "policy_applications",
        &id,
        body,
        &user.company_id,
        &user.user_id,
    )
    .await
}

async fn delete_application(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    generic_delete(
        &pool,
        "policy_applications",
        &id,
        &user.company_id,
        &user.user_id,
    )
    .await
}

fn policy_display_value(policy: &Policy) -> Value {
    let mut value = serde_json::to_value(policy).unwrap_or_else(|_| json!({}));
    let Some(map) = value.as_object_mut() else {
        return value;
    };

    let currency = policy.currency.as_deref().unwrap_or("USD");
    map.insert(
        "category_badge".to_string(),
        category_badge(&policy.category),
    );
    map.insert("country_badge".to_string(), country_badge(&policy.country));
    map.insert(
        "amount_min_label".to_string(),
        option_amount_label(policy.amount_min, currency),
    );
    map.insert(
        "amount_max_label".to_string(),
        option_amount_label(policy.amount_max, currency),
    );
    map.insert(
        "amount_range_label".to_string(),
        amount_range_label(policy.amount_min, policy.amount_max, currency).into(),
    );
    map.insert(
        "equity_required_label".to_string(),
        policy
            .equity_required
            .map(equity_label)
            .unwrap_or_default()
            .into(),
    );
    map.insert(
        "eligibility_items".to_string(),
        list_items(policy.eligibility.as_deref(), "number"),
    );
    map.insert(
        "benefit_items".to_string(),
        list_items(policy.benefits.as_deref(), "check"),
    );
    map.insert(
        "match_profile_items".to_string(),
        match_profile_items(policy),
    );
    map.insert("apply_action".to_string(), policy_apply_action(&policy.id));

    value
}

fn policy_apply_action(policy_id: &str) -> Value {
    json!({
        "method": "POST",
        "endpoint": "/api/v1/policies/applications",
        "label": "加入申报追踪",
        "success_title": "添加成功",
        "success_message": "已添加到申报追踪列表",
        "default_values": {
            "policy_id": policy_id
        }
    })
}

fn scored_policy_display_value(item: ScoredPolicy, index: usize) -> Value {
    let policy_value = policy_display_value(&item.policy);
    let mut value = policy_value.clone();
    let Some(map) = value.as_object_mut() else {
        return value;
    };
    let is_best_match = index == 0;

    map.insert("policy".to_string(), policy_value);
    map.insert("match_score".to_string(), item.match_score.into());
    map.insert(
        "match_score_label".to_string(),
        format!("{} 分", item.match_score).into(),
    );
    map.insert("match_reasons".to_string(), json!(item.match_reasons));
    map.insert(
        "rank_badge".to_string(),
        if is_best_match {
            json!({
                "label": "最优匹配",
                "class_name": "bg-primary text-primary-foreground",
                "visible": true,
            })
        } else {
            json!({
                "label": format!("第 {} 位", index + 1),
                "class_name": "bg-muted text-muted-foreground",
                "visible": false,
            })
        },
    );
    map.insert(
        "card_class_name".to_string(),
        if is_best_match {
            "border-primary/50 shadow-md relative overflow-hidden"
        } else {
            ""
        }
        .into(),
    );

    value
}

fn normalized_filter(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|item| !item.is_empty() && *item != "all")
        .map(str::to_string)
}

fn policy_category_options() -> Vec<Value> {
    [
        ("all", "所有类型"),
        ("accelerator", "加速器/孵化器"),
        ("government_grant", "政府无偿资助"),
        ("tax_incentive", "税收优惠"),
        ("vc_fund", "创投基金"),
        ("rd_subsidy", "研发补贴"),
        ("loan", "创业贷款"),
    ]
    .into_iter()
    .map(|(value, label)| json!({ "value": value, "label": label }))
    .collect()
}

fn policy_country_options() -> Vec<Value> {
    [
        ("all", "全球所有地区"),
        ("US", "🇺🇸 美国"),
        ("CN", "🇨🇳 中国"),
        ("HK", "🇭🇰 中国香港"),
        ("SG", "🇸🇬 新加坡"),
        ("JP", "🇯🇵 日本"),
        ("UK", "🇬🇧 英国"),
        ("DE", "🇩🇪 德国"),
        ("KR", "🇰🇷 韩国"),
        ("IL", "🇮🇱 以色列"),
    ]
    .into_iter()
    .map(|(value, label)| json!({ "value": value, "label": label }))
    .collect()
}

fn category_badge(category: &str) -> Value {
    let (label, class_name) = match category {
        "accelerator" => (
            "加速器/孵化器",
            "bg-violet-100 text-violet-800 border-violet-200",
        ),
        "government_grant" => ("政府无偿资助", "bg-blue-100 text-blue-800 border-blue-200"),
        "tax_incentive" => (
            "税收优惠",
            "bg-emerald-100 text-emerald-800 border-emerald-200",
        ),
        "vc_fund" => (
            "创投基金",
            "bg-orange-100 text-orange-800 border-orange-200",
        ),
        "rd_subsidy" => ("研发补贴", "bg-cyan-100 text-cyan-800 border-cyan-200"),
        "loan" => ("创业贷款", "bg-amber-100 text-amber-800 border-amber-200"),
        _ => (category, "bg-gray-100 text-gray-800 border-gray-200"),
    };
    json!({ "label": label, "class_name": class_name })
}

fn country_badge(country: &str) -> Value {
    let label = match country {
        "US" => "🇺🇸 美国",
        "CN" => "🇨🇳 中国",
        "JP" => "🇯🇵 日本",
        "SG" => "🇸🇬 新加坡",
        "HK" => "🇭🇰 中国香港",
        "UK" => "🇬🇧 英国",
        "DE" => "🇩🇪 德国",
        "KR" => "🇰🇷 韩国",
        "IL" => "🇮🇱 以色列",
        "*" => "🌍 全球",
        _ => country,
    };
    json!({
        "label": label,
        "class_name": "bg-blue-50 text-blue-700 border-blue-200",
    })
}

fn option_amount_label(value: Option<f64>, currency: &str) -> Value {
    value
        .map(|amount| format_amount(amount, currency))
        .unwrap_or_default()
        .into()
}

fn amount_range_label(min: Option<f64>, max: Option<f64>, currency: &str) -> String {
    match (min, max) {
        (Some(min), Some(max)) => {
            format!(
                "{} - {}",
                format_amount(min, currency),
                format_amount(max, currency)
            )
        }
        (None, Some(max)) => format!("最高 {}", format_amount(max, currency)),
        (Some(min), None) => format_amount(min, currency),
        (None, None) => String::new(),
    }
}

fn format_amount(value: f64, currency: &str) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    let absolute = rounded.abs();
    let sign = if rounded < 0.0 { "-" } else { "" };
    let number = if (absolute.fract()).abs() < 0.005 {
        comma_i64(absolute.round() as i64)
    } else {
        let whole = absolute.trunc() as i64;
        let cents = ((absolute - whole as f64) * 100.0).round() as i64;
        format!("{}.{cents:02}", comma_i64(whole))
    };
    format!("{} {}{}", currency, sign, number)
}

fn equity_label(value: f64) -> String {
    format!("{:.0}%", value * 100.0)
}

fn list_items(value: Option<&str>, icon: &str) -> Value {
    let normalized = value.unwrap_or_default().replace("\\n", "\n");
    let items = normalized
        .lines()
        .filter_map(|line| {
            let is_sub_item = line.starts_with("   ");
            let text = line
                .trim()
                .trim_start_matches(|ch: char| {
                    ch.is_ascii_digit() || matches!(ch, '.' | ')' | '、')
                })
                .trim()
                .to_string();
            if text.is_empty() {
                None
            } else {
                Some(json!({
                    "text": text,
                    "is_sub_item": is_sub_item,
                    "icon": icon,
                }))
            }
        })
        .collect::<Vec<_>>();
    json!(items)
}

fn match_profile_items(policy: &Policy) -> Value {
    let mut items = Vec::new();
    if let Some(countries) = policy.match_countries.as_deref() {
        items.push(json!({
            "label": "适用地区",
            "value": if countries.contains('*') { "全球".to_string() } else { compact_json_list(countries) },
            "icon": "Globe2",
        }));
    }
    if let Some(industries) = policy.match_industries.as_deref() {
        items.push(json!({
            "label": "适用行业",
            "value": compact_json_list(industries),
            "icon": "Briefcase",
        }));
    }
    if policy.match_min_employees.is_some() || policy.match_max_employees.is_some() {
        items.push(json!({
            "label": "员工规模",
            "value": bounded_label(policy.match_min_employees, policy.match_max_employees, "人"),
            "icon": "Users",
        }));
    }
    if let Some(months) = policy.match_company_age_max {
        items.push(json!({
            "label": "公司年龄",
            "value": format!("≤{}年", ((months as f64) / 12.0).round() as i64),
            "icon": "Clock",
        }));
    }
    if let Some(education) = policy.match_founder_education.as_deref() {
        items.push(json!({
            "label": "创始人学历",
            "value": compact_json_list(education),
            "icon": "GraduationCap",
        }));
    }
    if policy.match_founder_is_expat == Some(1) {
        items.push(json!({
            "label": "创始人身份",
            "value": "外籍人士优先",
            "icon": "Globe2",
        }));
    }
    if policy.match_founder_is_overseas_returnee == Some(1) {
        items.push(json!({
            "label": "创始人留学背景",
            "value": "海归优先",
            "icon": "GraduationCap",
        }));
    }
    if policy.match_min_revenue.is_some() || policy.match_max_revenue.is_some() {
        items.push(json!({
            "label": "营收要求",
            "value": bounded_money_label(policy.match_min_revenue, policy.match_max_revenue),
            "icon": "DollarSign",
        }));
    }

    json!(items)
}

fn bounded_label(min: Option<i64>, max: Option<i64>, unit: &str) -> String {
    match (min, max) {
        (Some(min), Some(max)) => format!("≥{}{} · ≤{}{}", min, unit, max, unit),
        (Some(min), None) => format!("≥{}{}", min, unit),
        (None, Some(max)) => format!("≤{}{}", max, unit),
        (None, None) => String::new(),
    }
}

fn bounded_money_label(min: Option<f64>, max: Option<f64>) -> String {
    match (min, max) {
        (Some(min), Some(max)) => format!("≥{}万 · ≤{}万", min / 10_000.0, max / 10_000.0),
        (Some(min), None) => format!("≥{}万", min / 10_000.0),
        (None, Some(max)) => format!("≤{}万", max / 10_000.0),
        (None, None) => String::new(),
    }
}

fn compact_json_list(value: &str) -> String {
    value
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .replace('"', "")
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>()
        .join(", ")
}

fn policy_application_status_badge(status: &str) -> Value {
    let (label, class_name) = match status {
        "preparing" => ("材料准备中", "bg-yellow-100 text-yellow-800"),
        "submitted" => ("已提交审批", "bg-indigo-100 text-indigo-800"),
        "accepted" => ("成功获批", "bg-green-100 text-green-800"),
        "rejected" => ("申报未通过", "bg-red-100 text-red-800"),
        _ => ("意向评估中", "bg-blue-100 text-blue-800"),
    };
    json!({ "label": label, "class_name": class_name })
}

fn date_label(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(10).collect())
        .unwrap_or_else(|| "-".to_string())
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
