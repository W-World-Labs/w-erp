/// Scenario 70: backend-owned KPI runtime dashboard.
/// KPI runtime values, previous values, and alerts are calculated in the backend.
use crate::common;

use axum::http::StatusCode;
use chrono::Utc;
use serde_json::Value;

#[tokio::test]
async fn s70_kpi_dashboard_requires_auth() {
    let app = common::TestApp::new().await;

    let res = app.server.get("/api/v1/kpi/dashboard").await;

    res.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn s70_kpi_dashboard_calculates_runtime_values_in_backend() {
    let app = common::TestApp::new().await;
    seed_kpi_fixture(&app).await;

    let res = app.get("/api/v1/kpi/dashboard").await;
    res.assert_status_ok();

    let body: Value = res.json();
    assert!(body["period"].as_str().is_some());
    let kpis = body["kpis"].as_array().unwrap();

    assert_kpi_value(kpis, "本月营收", 1500.0);
    assert_kpi_value(kpis, "人力成本", 500.0);
    assert_kpi_value(kpis, "毛利率", 66.66666666666666);
    assert_kpi_value(kpis, "项目成功率", 50.0);
    assert_kpi_value(kpis, "发票处理率", 50.0);
    assert_kpi_value(kpis, "人效", 1500.0);

    let revenue = kpis.iter().find(|item| item["name"] == "本月营收").unwrap();
    assert_eq!(revenue["previous_value"].as_f64().unwrap(), 900.0);
    assert_eq!(revenue["records"].as_array().unwrap().len(), 1);

    let alerts = body["alerts"].as_array().unwrap();
    assert!(
        alerts
            .iter()
            .any(|alert| { alert["kpi"] == "本月营收" && alert["type"] == "warning" })
    );
}

#[tokio::test]
async fn s70_kpi_dashboard_is_company_scoped() {
    let app = common::TestApp::new().await;
    seed_kpi_fixture(&app).await;
    let rival = app.create_second_tenant().await;

    let res = rival.get("/api/v1/kpi/dashboard").await;
    res.assert_status_ok();

    let body: Value = res.json();
    assert_eq!(body["kpis"].as_array().unwrap().len(), 0);
    assert_eq!(body["alerts"].as_array().unwrap().len(), 0);
}

fn assert_kpi_value(kpis: &[Value], name: &str, expected: f64) {
    let item = kpis
        .iter()
        .find(|item| item["name"] == name)
        .unwrap_or_else(|| panic!("missing kpi {}", name));
    let actual = item["current_value"].as_f64().unwrap();
    assert!(
        (actual - expected).abs() < 0.0001,
        "expected {} to be {}, got {}",
        name,
        expected,
        actual
    );
}

async fn seed_kpi_fixture(app: &common::TestApp) {
    let period = Utc::now().format("%Y-%m").to_string();
    let today = Utc::now().format("%Y-%m-%d").to_string();

    let revenue_kpi =
        insert_kpi_config(app, "本月营收", "financial", "¥", 2_000.0, 1_600.0, 1_000.0).await;
    insert_kpi_config(app, "人力成本", "financial", "¥", 800.0, 300.0, 100.0).await;
    insert_kpi_config(app, "毛利率", "financial", "%", 50.0, 30.0, 10.0).await;
    insert_kpi_config(app, "项目成功率", "operational", "%", 80.0, 60.0, 30.0).await;
    insert_kpi_config(app, "发票处理率", "operational", "%", 90.0, 70.0, 40.0).await;
    insert_kpi_config(app, "人效", "hr", "¥", 1200.0, 800.0, 400.0).await;

    sqlx::query(
        "INSERT INTO kpi_records (id, company_id, kpi_id, value, period) VALUES (?, ?, ?, 900, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .bind(&revenue_kpi)
    .bind(&period)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO invoices (id, company_id, invoice_number, amount, status, invoice_date)
         VALUES (?, ?, 'KPI-PAID', 1000, 'paid', ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .bind(&today)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO invoices (id, company_id, invoice_number, amount, status, invoice_date)
         VALUES (?, ?, 'KPI-PENDING', 500, 'pending', ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .bind(&today)
    .execute(&app.pool)
    .await
    .unwrap();

    let employee_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO employees (id, company_id, name, status) VALUES (?, ?, 'KPI Employee', 'active')",
    )
    .bind(&employee_id)
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO salary_records (id, company_id, employee_id, period, base_salary, bonus)
         VALUES (?, ?, ?, ?, 400, 100)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .bind(&employee_id)
    .bind(&period)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO projects (id, company_id, name, status) VALUES (?, ?, 'Done', 'completed')",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO projects (id, company_id, name, status) VALUES (?, ?, 'Running', 'ongoing')",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();
}

async fn insert_kpi_config(
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
