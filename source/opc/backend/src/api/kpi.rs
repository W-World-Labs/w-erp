use axum::{Router, extract::State, http::StatusCode, response::Json, routing::get};
use chrono::Utc;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::auth::AuthUser;

use super::error::{ApiError, api_err};

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/dashboard", get(dashboard))
        .with_state(pool)
}

#[derive(Debug, Serialize)]
struct KpiDashboardResponse {
    period: String,
    kpis: Vec<KpiRuntimeItem>,
    alerts: Vec<KpiAlert>,
    category_options: Vec<KpiCategoryOption>,
    empty_message: String,
}

#[derive(Debug, Clone, Serialize)]
struct KpiRuntimeItem {
    id: String,
    name: String,
    category: Option<String>,
    unit: Option<String>,
    target_value: Option<f64>,
    warning_threshold: Option<f64>,
    danger_threshold: Option<f64>,
    current_value: f64,
    previous_value: f64,
    records: Vec<KpiRecordPoint>,
    chart_points: Vec<KpiChartPoint>,
    current_value_label: String,
    previous_value_label: String,
    change_label: String,
    change_icon: String,
    change_class: String,
    has_previous_value: bool,
    target_value_label: Option<String>,
    alert_type: String,
    alert_badge: KpiDisplayBadge,
    chart_empty_message: String,
}

#[derive(Debug, Clone, Serialize)]
struct KpiRecordPoint {
    value: f64,
    period: String,
    recorded_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct KpiChartPoint {
    value: f64,
    value_label: String,
    period: String,
    period_label: String,
    recorded_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct KpiCategoryOption {
    value: String,
    label: String,
    items: Vec<KpiRuntimeItem>,
    chart_items: Vec<KpiRuntimeItem>,
    empty_message: String,
}

#[derive(Debug, Clone, Serialize)]
struct KpiDisplayBadge {
    label: String,
    class_name: String,
    icon: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct KpiAlert {
    #[serde(rename = "type")]
    alert_type: String,
    kpi: String,
    message: String,
    current_value: f64,
    threshold: f64,
    current_value_label: String,
    threshold_label: String,
    variant: String,
    badge: KpiDisplayBadge,
}

#[derive(Debug)]
struct KpiConfigRow {
    id: String,
    name: String,
    category: Option<String>,
    unit: Option<String>,
    target_value: Option<f64>,
    warning_threshold: Option<f64>,
    danger_threshold: Option<f64>,
}

#[derive(Debug)]
struct KpiBasis {
    revenue: f64,
    salary_cost: f64,
    completed_projects: i64,
    total_projects: i64,
    paid_invoices: i64,
    processed_invoices: i64,
    total_invoices: i64,
    current_salary_records: i64,
    salary_records_with_bonus: i64,
}

async fn dashboard(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<KpiDashboardResponse>, ApiError> {
    let period = Utc::now().format("%Y-%m").to_string();
    let configs = load_kpi_configs(&pool, &user.company_id).await?;
    let basis = load_kpi_basis(&pool, &user.company_id, &period).await?;
    let records = load_kpi_records(&pool, &user.company_id).await?;

    let mut kpis = Vec::new();
    let mut alerts = Vec::new();

    for config in configs {
        let current_value = calculate_value(&config, &basis);
        let mut config_records = records
            .iter()
            .filter(|record| record.0 == config.id)
            .map(|record| KpiRecordPoint {
                value: record.1,
                period: record.2.clone(),
                recorded_at: record.3.clone(),
            })
            .collect::<Vec<_>>();
        config_records.sort_by(|a, b| {
            a.period
                .cmp(&b.period)
                .then_with(|| a.recorded_at.cmp(&b.recorded_at))
        });
        let previous_value = config_records
            .last()
            .map(|record| record.value)
            .unwrap_or(0.0);
        let unit = config.unit.as_deref();
        let chart_points = kpi_chart_points(&config_records, unit);
        let (change_label, change_icon, change_class, has_previous_value) =
            kpi_change_display(current_value, previous_value);
        let current_value_label = format_kpi_value(current_value, unit);
        let previous_value_label = format_kpi_value(previous_value, unit);
        let target_value_label = config
            .target_value
            .map(|target| format!("目标: {}", format_kpi_value(target, unit)));
        let alert = alert_for(&config, current_value);
        let alert_type = alert
            .as_ref()
            .map(|alert| alert.alert_type.clone())
            .unwrap_or_else(|| "normal".to_string());
        let alert_badge = kpi_alert_badge(&alert_type);

        if let Some(alert) = alert {
            alerts.push(alert);
        }

        kpis.push(KpiRuntimeItem {
            id: config.id,
            name: config.name,
            category: config.category,
            unit: config.unit,
            target_value: config.target_value,
            warning_threshold: config.warning_threshold,
            danger_threshold: config.danger_threshold,
            current_value,
            previous_value,
            records: config_records,
            chart_points,
            current_value_label,
            previous_value_label,
            change_label,
            change_icon,
            change_class,
            has_previous_value,
            target_value_label,
            alert_type,
            alert_badge,
            chart_empty_message: "暂无历史数据".to_string(),
        });
    }

    let category_options = kpi_category_options(&kpis);

    Ok(Json(KpiDashboardResponse {
        period,
        kpis,
        alerts,
        category_options,
        empty_message: "暂无KPI指标，请点击\"新增指标\"创建".to_string(),
    }))
}

async fn load_kpi_configs(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<Vec<KpiConfigRow>, ApiError> {
    let rows: Vec<(
        String,
        String,
        Option<String>,
        Option<String>,
        Option<f64>,
        Option<f64>,
        Option<f64>,
    )> = sqlx::query_as(
        "SELECT id, name, category, unit, target_value, warning_threshold, danger_threshold
         FROM kpi_configs
         WHERE company_id = ? AND COALESCE(is_active, 1) = 1
         ORDER BY created_at DESC",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(|err| {
        tracing::error!("load kpi configs failed: {:?}", err);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error")
    })?;

    Ok(rows
        .into_iter()
        .map(
            |(id, name, category, unit, target_value, warning_threshold, danger_threshold)| {
                KpiConfigRow {
                    id,
                    name,
                    category,
                    unit,
                    target_value,
                    warning_threshold,
                    danger_threshold,
                }
            },
        )
        .collect())
}

async fn load_kpi_records(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<Vec<(String, f64, String, Option<String>)>, ApiError> {
    sqlx::query_as(
        "SELECT kpi_id, COALESCE(value, 0.0), period, created_at
         FROM kpi_records
         WHERE company_id = ?
         ORDER BY period ASC, created_at ASC",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(|err| {
        tracing::error!("load kpi records failed: {:?}", err);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error")
    })
}

async fn load_kpi_basis(
    pool: &SqlitePool,
    company_id: &str,
    period: &str,
) -> Result<KpiBasis, ApiError> {
    let (revenue,): (f64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(amount), 0.0)
         FROM invoices
         WHERE company_id = ? AND substr(COALESCE(invoice_date, ''), 1, 7) = ?",
    )
    .bind(company_id)
    .bind(period)
    .fetch_one(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let (salary_cost,): (f64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(COALESCE(base_salary, 0.0) + COALESCE(bonus, 0.0)), 0.0)
         FROM salary_records
         WHERE company_id = ? AND period = ?",
    )
    .bind(company_id)
    .bind(period)
    .fetch_one(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let (completed_projects,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM projects WHERE company_id = ? AND status = 'completed'",
    )
    .bind(company_id)
    .fetch_one(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let (total_projects,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM projects WHERE company_id = ?")
            .bind(company_id)
            .fetch_one(pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let (paid_invoices,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM invoices WHERE company_id = ? AND status = 'paid'")
            .bind(company_id)
            .fetch_one(pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let (processed_invoices,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM invoices WHERE company_id = ? AND status != 'pending'",
    )
    .bind(company_id)
    .fetch_one(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let (total_invoices,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM invoices WHERE company_id = ?")
            .bind(company_id)
            .fetch_one(pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let (current_salary_records,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM salary_records WHERE company_id = ? AND period = ?")
            .bind(company_id)
            .bind(period)
            .fetch_one(pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let (salary_records_with_bonus,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM salary_records WHERE company_id = ? AND period = ? AND COALESCE(bonus, 0.0) > 0.0",
    )
    .bind(company_id)
    .bind(period)
    .fetch_one(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(KpiBasis {
        revenue,
        salary_cost,
        completed_projects,
        total_projects,
        paid_invoices,
        processed_invoices,
        total_invoices,
        current_salary_records,
        salary_records_with_bonus,
    })
}

fn calculate_value(config: &KpiConfigRow, basis: &KpiBasis) -> f64 {
    let category = config.category.as_deref().unwrap_or_default();
    let name = config.name.as_str();

    match category {
        "financial" => {
            if name.contains("营收") {
                basis.revenue
            } else if name.contains("人力") || name.contains("成本") {
                basis.salary_cost
            } else if name.contains("毛利率") {
                if basis.revenue > 0.0 {
                    ((basis.revenue - basis.salary_cost) / basis.revenue) * 100.0
                } else {
                    0.0
                }
            } else if name.contains("现金流") {
                basis.revenue - basis.salary_cost
            } else {
                0.0
            }
        }
        "operational" => {
            if name.contains("项目成功率") {
                percentage(basis.completed_projects, basis.total_projects)
            } else if name.contains("客户满意度") {
                if basis.total_invoices > 0 {
                    (percentage(basis.paid_invoices, basis.total_invoices) + 10.0).min(95.0)
                } else {
                    0.0
                }
            } else if name.contains("发票处理率") {
                percentage(basis.processed_invoices, basis.total_invoices)
            } else {
                0.0
            }
        }
        "hr" => {
            if name.contains("人效") {
                if basis.current_salary_records > 0 {
                    basis.revenue / basis.current_salary_records as f64
                } else {
                    0.0
                }
            } else if name.contains("员工满意度") {
                if basis.current_salary_records > 0 {
                    70.0 + percentage(
                        basis.salary_records_with_bonus,
                        basis.current_salary_records,
                    ) * 0.25
                } else {
                    0.0
                }
            } else {
                0.0
            }
        }
        "project" => {
            if name.contains("项目交付率") {
                percentage(basis.completed_projects, basis.total_projects)
            } else if name.contains("预算偏差率") {
                if basis.revenue > 0.0 {
                    ((basis.salary_cost / basis.revenue) * 100.0 - 40.0).abs()
                } else {
                    0.0
                }
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

fn percentage(numerator: i64, denominator: i64) -> f64 {
    if denominator > 0 {
        numerator as f64 / denominator as f64 * 100.0
    } else {
        0.0
    }
}

fn kpi_category_options(kpis: &[KpiRuntimeItem]) -> Vec<KpiCategoryOption> {
    [
        ("all", "全部指标"),
        ("financial", "财务指标"),
        ("operational", "运营指标"),
        ("hr", "人力指标"),
        ("project", "项目指标"),
    ]
    .into_iter()
    .map(|(value, label)| {
        let items = if value == "all" {
            kpis.to_vec()
        } else {
            kpis.iter()
                .filter(|kpi| kpi.category.as_deref() == Some(value))
                .cloned()
                .collect::<Vec<_>>()
        };
        let chart_items = items.iter().take(4).cloned().collect::<Vec<_>>();

        KpiCategoryOption {
            value: value.to_string(),
            label: label.to_string(),
            items,
            chart_items,
            empty_message: "暂无KPI指标，请点击\"新增指标\"创建".to_string(),
        }
    })
    .collect()
}

fn kpi_chart_points(records: &[KpiRecordPoint], unit: Option<&str>) -> Vec<KpiChartPoint> {
    records
        .iter()
        .skip(records.len().saturating_sub(6))
        .map(|record| KpiChartPoint {
            value: record.value,
            value_label: format_kpi_value(record.value, unit),
            period: record.period.clone(),
            period_label: kpi_period_label(&record.period),
            recorded_at: record.recorded_at.clone(),
        })
        .collect()
}

fn kpi_period_label(period: &str) -> String {
    let mut parts = period.split('-');
    if let (Some(_year), Some(month), None) = (parts.next(), parts.next(), parts.next())
        && let Ok(month) = month.parse::<u32>()
        && (1..=12).contains(&month)
    {
        return format!("{:02}月", month);
    }

    period.to_string()
}

fn kpi_change_display(current_value: f64, previous_value: f64) -> (String, String, String, bool) {
    if previous_value <= 0.0 {
        return (
            "0.0%".to_string(),
            "TrendingUp".to_string(),
            "text-muted-foreground".to_string(),
            false,
        );
    }

    let change = ((current_value - previous_value) / previous_value) * 100.0;
    let icon = if change >= 0.0 {
        "TrendingUp"
    } else {
        "TrendingDown"
    };
    let class_name = if change >= 0.0 {
        "text-green-500"
    } else {
        "text-red-500"
    };
    let sign = if change >= 0.0 { "+" } else { "" };

    (
        format!("{}{:.1}%", sign, change),
        icon.to_string(),
        class_name.to_string(),
        true,
    )
}

fn kpi_alert_badge(alert_type: &str) -> KpiDisplayBadge {
    match alert_type {
        "danger" => KpiDisplayBadge {
            label: "危险预警".to_string(),
            class_name: "text-destructive border-destructive".to_string(),
            icon: Some("AlertCircle".to_string()),
        },
        "warning" => KpiDisplayBadge {
            label: "警告预警".to_string(),
            class_name: "text-yellow-600 border-yellow-500".to_string(),
            icon: Some("AlertTriangle".to_string()),
        },
        _ => KpiDisplayBadge {
            label: "正常".to_string(),
            class_name: "text-muted-foreground".to_string(),
            icon: None,
        },
    }
}

fn format_kpi_value(value: f64, unit: Option<&str>) -> String {
    let safe_value = if value.is_finite() { value } else { 0.0 };
    match unit.unwrap_or_default().trim() {
        "%" => format!("{:.1}%", safe_value),
        "¥" | "CNY" | "RMB" => format!("¥{}", format_decimal_with_commas(safe_value, 2)),
        unit if !unit.is_empty() => {
            format!("{}{}", format_decimal_with_commas(safe_value, 2), unit)
        }
        _ => format_decimal_with_commas(safe_value, 2),
    }
}

fn format_decimal_with_commas(value: f64, decimals: usize) -> String {
    let sign = if value < 0.0 { "-" } else { "" };
    let raw = format!("{:.*}", decimals, value.abs());
    let (integer, fraction) = raw.split_once('.').unwrap_or((raw.as_str(), ""));
    let mut grouped_reversed = String::new();

    for (index, ch) in integer.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped_reversed.push(',');
        }
        grouped_reversed.push(ch);
    }

    let grouped_integer = grouped_reversed.chars().rev().collect::<String>();
    if decimals == 0 {
        format!("{sign}{grouped_integer}")
    } else {
        format!("{sign}{grouped_integer}.{fraction}")
    }
}

fn alert_for(config: &KpiConfigRow, current_value: f64) -> Option<KpiAlert> {
    if let Some(threshold) = config.danger_threshold
        && current_value < threshold
    {
        let current_value_label = format_kpi_value(current_value, config.unit.as_deref());
        let threshold_label = format_kpi_value(threshold, config.unit.as_deref());
        return Some(KpiAlert {
            alert_type: "danger".to_string(),
            kpi: config.name.clone(),
            message: format!("当前值 {current_value_label} 低于危险阈值 {threshold_label}"),
            current_value,
            threshold,
            current_value_label,
            threshold_label,
            variant: "destructive".to_string(),
            badge: kpi_alert_badge("danger"),
        });
    }

    if let Some(threshold) = config.warning_threshold
        && current_value < threshold
    {
        let current_value_label = format_kpi_value(current_value, config.unit.as_deref());
        let threshold_label = format_kpi_value(threshold, config.unit.as_deref());
        return Some(KpiAlert {
            alert_type: "warning".to_string(),
            kpi: config.name.clone(),
            message: format!("当前值 {current_value_label} 低于警告阈值 {threshold_label}"),
            current_value,
            threshold,
            current_value_label,
            threshold_label,
            variant: "default".to_string(),
            badge: kpi_alert_badge("warning"),
        });
    }

    None
}
