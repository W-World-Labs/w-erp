use axum::http::StatusCode;
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{FromRow, SqlitePool};

use crate::api::error::{ApiError, api_err};
use crate::auth::AuthUser;

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/", get(list_hubs))
        .route("/calculator", post(calculate_opc_tax))
        .with_state(pool)
}

#[derive(Serialize, FromRow, Clone)]
pub struct OpcHub {
    pub id: String,
    pub country_code: String,
    pub city: String,
    pub currency: String,
    pub corporate_tax_rate: f64,
    pub dividend_tax_rate: f64,
    pub capital_gains_tax: f64,
    pub pass_through_allowed: i64,
    pub zero_tax_on_reinvested: i64,
    pub foreign_income_exempt: i64,
    pub digital_nomad_visa: i64,
    pub col_index: f64,
    pub avg_coworking_desk: f64,
    pub avg_apartment_rent_1bed: f64,
    pub setup_cost: f64,
    pub annual_compliance_cost: f64,
    pub hub_notes: Option<String>,
    pub is_active: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct CalculatorReq {
    pub est_annual_revenue: f64,
    pub current_base_col_index: f64,
}

#[derive(Serialize)]
pub struct CalculatorRes {
    pub hub_id: String,
    pub city: String,
    pub friction_costs: f64,
    pub living_costs: f64,
    pub net_take_home: f64,
    pub mrr_survival_line: f64,
    pub take_home_percentage: f64,
    pub setup_and_maintenance_roi: f64,
    pub tax_sandbox_type: String,
    pub total_tax_paid: f64,
    pub rank: i64,
    pub is_best: bool,
    pub friction_costs_label: String,
    pub living_costs_label: String,
    pub net_take_home_label: String,
    pub mrr_survival_line_label: String,
    pub take_home_percentage_label: String,
    pub setup_and_maintenance_roi_label: String,
    pub total_tax_paid_label: String,
    pub tax_type_badge: Value,
}

#[derive(Serialize)]
pub struct HubListRes {
    data: Vec<Value>,
    count: i64,
}

fn money_label(value: f64) -> String {
    let rounded = value.round() as i64;
    let sign = if rounded < 0 { "-$" } else { "$" };
    let digits = rounded.abs().to_string();
    let mut out = String::new();
    for (idx, ch) in digits.chars().rev().enumerate() {
        if idx > 0 && idx % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    let formatted: String = out.chars().rev().collect();
    format!("{sign}{formatted}")
}

fn monthly_money_label(value: f64) -> String {
    format!("{}/月", money_label(value))
}

fn annual_money_label(value: f64) -> String {
    format!("{}/年", money_label(value))
}

fn percent_label(value: f64, multiplier: f64, digits: usize) -> String {
    format!("{:.*}%", digits, value * multiplier)
}

fn country_flag(country_code: &str) -> &'static str {
    match country_code {
        "AE" => "🇦🇪",
        "SG" => "🇸🇬",
        "US" => "🇺🇸",
        "HK" => "🇭🇰",
        "EE" => "🇪🇪",
        "JP" => "🇯🇵",
        "GB" => "🇬🇧",
        "DE" => "🇩🇪",
        "CA" => "🇨🇦",
        "AU" => "🇦🇺",
        _ => "🌐",
    }
}

fn hub_feature_badges(hub: &OpcHub) -> Vec<Value> {
    let mut badges = Vec::new();
    if hub.pass_through_allowed == 1 {
        badges.push(json!({
            "label": "穿透税制",
            "variant": "outline",
            "class_name": "text-xs"
        }));
    }
    if hub.foreign_income_exempt == 1 {
        badges.push(json!({
            "label": "境外免税",
            "variant": "outline",
            "class_name": "text-xs"
        }));
    }
    if hub.zero_tax_on_reinvested == 1 {
        badges.push(json!({
            "label": "留存免税",
            "variant": "outline",
            "class_name": "text-xs"
        }));
    }
    if hub.digital_nomad_visa == 1 {
        badges.push(json!({
            "label": "签证",
            "variant": "outline",
            "icon": "Plane",
            "class_name": "text-xs"
        }));
    }
    badges
}

fn hub_display_value(hub: &OpcHub) -> Value {
    json!({
        "id": hub.id,
        "country_code": hub.country_code,
        "city": hub.city,
        "currency": hub.currency,
        "corporate_tax_rate": hub.corporate_tax_rate,
        "dividend_tax_rate": hub.dividend_tax_rate,
        "capital_gains_tax": hub.capital_gains_tax,
        "pass_through_allowed": hub.pass_through_allowed,
        "zero_tax_on_reinvested": hub.zero_tax_on_reinvested,
        "foreign_income_exempt": hub.foreign_income_exempt,
        "digital_nomad_visa": hub.digital_nomad_visa,
        "col_index": hub.col_index,
        "avg_coworking_desk": hub.avg_coworking_desk,
        "avg_apartment_rent_1bed": hub.avg_apartment_rent_1bed,
        "setup_cost": hub.setup_cost,
        "annual_compliance_cost": hub.annual_compliance_cost,
        "hub_notes": hub.hub_notes,
        "is_active": hub.is_active,
        "created_at": hub.created_at,
        "updated_at": hub.updated_at,
        "flag": country_flag(&hub.country_code),
        "corporate_tax_rate_label": percent_label(hub.corporate_tax_rate, 100.0, 0),
        "dividend_tax_rate_label": percent_label(hub.dividend_tax_rate, 100.0, 0),
        "rent_label": monthly_money_label(hub.avg_apartment_rent_1bed),
        "feature_badges": hub_feature_badges(hub)
    })
}

fn tax_type_badge(tax_sandbox_type: &str) -> Value {
    if tax_sandbox_type.starts_with("Type A") {
        json!({
            "label": "Pass-through LLC",
            "class_name": "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
        })
    } else if tax_sandbox_type.starts_with("Type B") {
        json!({
            "label": "Estonia Model",
            "class_name": "bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400"
        })
    } else if tax_sandbox_type.starts_with("Type C") {
        json!({
            "label": "Offshore Exempt",
            "class_name": "bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-400"
        })
    } else if tax_sandbox_type.starts_with("Type D") {
        json!({
            "label": "Standard Double Tax",
            "class_name": "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400"
        })
    } else {
        json!({
            "label": tax_sandbox_type,
            "class_name": "bg-gray-100 text-gray-800"
        })
    }
}

async fn list_hubs(
    State(pool): State<SqlitePool>,
    _user: AuthUser,
) -> Result<Json<HubListRes>, ApiError> {
    let hubs = sqlx::query_as::<_, OpcHub>("SELECT * FROM global_opc_hubs WHERE is_active = 1")
        .fetch_all(&pool)
        .await
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let count = hubs.len() as i64;
    let data = hubs.iter().map(hub_display_value).collect();
    Ok(Json(HubListRes { data, count }))
}

async fn calculate_opc_tax(
    State(pool): State<SqlitePool>,
    _user: AuthUser,
    Json(payload): Json<CalculatorReq>,
) -> Result<Json<Vec<CalculatorRes>>, ApiError> {
    let hubs = sqlx::query_as::<_, OpcHub>("SELECT * FROM global_opc_hubs WHERE is_active = 1")
        .fetch_all(&pool)
        .await
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let mut results = Vec::new();
    let revenue = payload.est_annual_revenue;
    let base_col = payload.current_base_col_index.max(1.0); // Avoid div by zero

    for hub in hubs {
        // 1. Friction Costs
        let friction_costs = hub.annual_compliance_cost + (hub.avg_coworking_desk * 12.0);

        // 2. Living Costs
        // Col index adjustment: actual cost relative to base col. NYC = 100.
        let living_multiplier = hub.col_index / base_col;
        let living_costs = (hub.avg_apartment_rent_1bed * 12.0) * living_multiplier;

        // 3. Tax Sandbox Simulation
        // Profit before tax
        let profit_before_tax = (revenue - friction_costs - living_costs).max(0.0);

        let (tax_sandbox_type, total_tax_paid) = if hub.foreign_income_exempt == 1 {
            // 0 corporate, 0 personal if sourced outside
            ("Type C (Territorial / Offshore Exempt)", 0.0)
        } else if hub.pass_through_allowed == 1 {
            // Assuming no local nexus for digital nomads
            ("Type A (Pass-through LLC)", 0.0)
        } else if hub.zero_tax_on_reinvested == 1 {
            // Assuming the founder distributes 50% for living and retains 50%
            let distributed = living_costs; // Bare minimum distribution
            let tax_paid = if distributed > 0.0 {
                // Approximate 20% on grossed up dividend
                distributed * hub.dividend_tax_rate
            } else {
                0.0
            };
            ("Type B (Estonia Model)", tax_paid)
        } else {
            let corporate_tax = profit_before_tax * hub.corporate_tax_rate;
            let profit_after_corp = profit_before_tax - corporate_tax;
            let dividend_tax = profit_after_corp * hub.dividend_tax_rate;
            ("Type D (Standard Double Tax)", corporate_tax + dividend_tax)
        };

        let net_take_home = profit_before_tax - total_tax_paid;

        let mrr_survival_line = (friction_costs + living_costs) / 12.0;

        // Use revenue instead of 0 to stop NaNs
        let take_home_percentage = if revenue > 0.0 {
            (net_take_home / revenue) * 100.0
        } else {
            0.0
        };
        let setup_and_maintenance_roi = if revenue > 0.0 {
            (friction_costs / revenue) * 100.0
        } else {
            0.0
        };

        results.push(CalculatorRes {
            hub_id: hub.id,
            city: hub.city,
            friction_costs,
            living_costs,
            net_take_home,
            mrr_survival_line,
            take_home_percentage,
            setup_and_maintenance_roi,
            tax_sandbox_type: tax_sandbox_type.to_string(),
            total_tax_paid,
            rank: 0,
            is_best: false,
            friction_costs_label: annual_money_label(friction_costs),
            living_costs_label: annual_money_label(living_costs),
            net_take_home_label: money_label(net_take_home),
            mrr_survival_line_label: monthly_money_label(mrr_survival_line),
            take_home_percentage_label: percent_label(take_home_percentage, 1.0, 1),
            setup_and_maintenance_roi_label: percent_label(setup_and_maintenance_roi, 1.0, 1),
            total_tax_paid_label: money_label(total_tax_paid),
            tax_type_badge: tax_type_badge(tax_sandbox_type),
        });
    }

    results.sort_by(|a, b| {
        b.net_take_home
            .partial_cmp(&a.net_take_home)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for (idx, result) in results.iter_mut().enumerate() {
        result.rank = idx as i64 + 1;
        result.is_best = idx == 0;
    }

    Ok(Json(results))
}
