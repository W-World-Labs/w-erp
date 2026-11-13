/// Scenario 115: backend-owned KPI display metadata.
/// KPI cards, category tabs, alerts, and charts receive display-ready data from the backend.
use crate::common;

use chrono::{Datelike, Months, Utc};
use serde_json::Value;

#[tokio::test]
async fn s115_kpi_dashboard_returns_backend_display_fields() {
    let app = common::TestApp::new().await;
    let revenue_kpi = seed_kpi_config(
        &app,
        "本月营收",
        "financial",
        "¥",
        2_000.0,
        1_600.0,
        1_000.0,
    )
    .await;
    let success_kpi =
        seed_kpi_config(&app, "项目成功率", "operational", "%", 80.0, 60.0, 30.0).await;
    seed_kpi_records(
        &app,
        &revenue_kpi,
        &[700.0, 800.0, 900.0, 1000.0, 1100.0, 1200.0, 1300.0],
    )
    .await;
    seed_kpi_records(&app, &success_kpi, &[40.0, 45.0]).await;
    seed_business_basis(&app).await;

    let res = app.get("/api/v1/kpi/dashboard").await;
    res.assert_status_ok();

    let body: Value = res.json();
    let categories = body["category_options"].as_array().unwrap();
    let all = categories
        .iter()
        .find(|category| category["value"] == "all")
        .expect("missing all category");
    assert_eq!(all["label"], "全部指标");
    assert_eq!(all["items"].as_array().unwrap().len(), 2);

    let financial = categories
        .iter()
        .find(|category| category["value"] == "financial")
        .expect("missing financial category");
    assert_eq!(financial["label"], "财务指标");
    assert_eq!(financial["items"].as_array().unwrap().len(), 1);

    let kpis = body["kpis"].as_array().unwrap();
    let revenue = kpis
        .iter()
        .find(|item| item["name"] == "本月营收")
        .expect("missing revenue KPI");
    assert_eq!(revenue["current_value_label"], "¥1,500.00");
    assert_eq!(revenue["previous_value_label"], "¥1,300.00");
    assert_eq!(revenue["target_value_label"], "目标: ¥2,000.00");
    assert_eq!(revenue["change_icon"], "TrendingUp");
    assert_eq!(revenue["alert_type"], "warning");
    assert_eq!(revenue["alert_badge"]["label"], "警告预警");

    let chart_points = revenue["chart_points"].as_array().unwrap();
    assert_eq!(chart_points.len(), 6);
    assert_eq!(chart_points[0]["value"], 800.0);
    assert!(
        chart_points[0]["period_label"]
            .as_str()
            .unwrap()
            .ends_with('月')
    );
    assert_eq!(chart_points[0]["value_label"], "¥800.00");
    assert_eq!(chart_points[5]["value_label"], "¥1,300.00");

    let project = kpis
        .iter()
        .find(|item| item["name"] == "项目成功率")
        .expect("missing project KPI");
    assert_eq!(project["current_value_label"], "50.0%");
    assert_eq!(project["previous_value_label"], "45.0%");
    assert_eq!(project["target_value_label"], "目标: 80.0%");

    let alerts = body["alerts"].as_array().unwrap();
    let alert = alerts
        .iter()
        .find(|alert| alert["kpi"] == "本月营收")
        .expect("missing revenue alert");
    assert_eq!(alert["current_value_label"], "¥1,500.00");
    assert_eq!(alert["threshold_label"], "¥1,600.00");
    assert_eq!(alert["variant"], "default");
    assert_eq!(alert["badge"]["icon"], "AlertTriangle");
}

#[tokio::test]
async fn s115_kpi_config_form_options_are_backend_owned() {
    let app = common::TestApp::new().await;

    let res = app.get("/api/v1/kpi_configs/form_options").await;
    res.assert_status_ok();
    let body: Value = res.json();

    assert_eq!(body["default_values"]["category"], "financial");
    assert_eq!(body["default_values"]["unit"], "元");
    assert_eq!(body["category_options"][0]["value"], "financial");
    assert_eq!(body["category_options"][0]["label"], "财务指标");
    assert_eq!(body["unit_options"][1]["value"], "%");
    assert_eq!(body["unit_options"][1]["label"], "百分比(%)");
    assert_eq!(body["create_action"]["method"], "POST");
    assert_eq!(body["create_action"]["endpoint"], "/api/v1/kpi_configs");
    assert_eq!(body["create_action"]["label"], "新增KPI指标");
    assert_eq!(body["create_action"]["success_message"], "KPI指标创建成功");
}

async fn seed_kpi_config(
    app: &common::TestApp,
    name: &str,
    category: &str,
    unit: &str,
    target: f64,
    warning: f64,
    danger: f64,
) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO kpi_configs (id, company_id, name, category, unit, target_value, warning_threshold, danger_threshold, is_active)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1)",
    )
    .bind(&id)
    .bind(&app.company_id)
    .bind(name)
    .bind(category)
    .bind(unit)
    .bind(target)
    .bind(warning)
    .bind(danger)
    .execute(&app.pool)
    .await
    .unwrap();
    id
}

async fn seed_kpi_records(app: &common::TestApp, kpi_id: &str, values: &[f64]) {
    let current_month = Utc::now().date_naive().with_day(1).unwrap();
    let start_month = current_month
        .checked_sub_months(Months::new(values.len().saturating_sub(1) as u32))
        .unwrap();

    for (index, value) in values.iter().enumerate() {
        let period = start_month
            .checked_add_months(Months::new(index as u32))
            .unwrap()
            .format("%Y-%m")
            .to_string();
        sqlx::query(
            "INSERT INTO kpi_records (id, company_id, kpi_id, value, period) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&app.company_id)
        .bind(kpi_id)
        .bind(value)
        .bind(period)
        .execute(&app.pool)
        .await
        .unwrap();
    }
}

async fn seed_business_basis(app: &common::TestApp) {
    let today = Utc::now().format("%Y-%m-%d").to_string();
    sqlx::query(
        "INSERT INTO invoices (id, company_id, invoice_number, amount, status, invoice_date)
         VALUES (?, ?, 'KPI-DISPLAY-PAID', 1500, 'paid', ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .bind(&today)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO projects (id, company_id, name, status) VALUES (?, ?, 'KPI Done', 'completed')",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO projects (id, company_id, name, status) VALUES (?, ?, 'KPI Running', 'ongoing')",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();
}
