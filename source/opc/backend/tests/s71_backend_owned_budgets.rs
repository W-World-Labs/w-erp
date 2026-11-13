/// Scenario 71: backend-owned budget execution.
/// Actual spend, budget status, and budget overview summaries are calculated server-side.
use crate::common;

use chrono::Utc;
use serde_json::Value;
use serde_json::json;

fn current_quarter_period() -> String {
    let now = Utc::now();
    let quarter = ((now.format("%m").to_string().parse::<u32>().unwrap() - 1) / 3) + 1;
    format!("{}-Q{}", now.format("%Y"), quarter)
}

#[tokio::test]
async fn s71_budget_create_calculates_actual_amount_and_status() {
    let app = common::TestApp::new().await;
    let period = Utc::now().format("%Y-%m").to_string();
    seed_budget_expense_fixture(&app, &period).await;

    let res = app
        .post("/api/budgets")
        .json(&json!({
            "category": "salary",
            "budget_type": "monthly",
            "period": period,
            "budget_amount": 400.0
        }))
        .await;
    res.assert_status_ok();

    let body: Value = res.json();
    assert_eq!(body["actual_amount"].as_f64().unwrap(), 500.0);
    assert_eq!(body["status"], "exceeded");
}

#[tokio::test]
async fn s71_budget_overview_returns_backend_summary_and_trend() {
    let app = common::TestApp::new().await;
    let period = Utc::now().format("%Y-%m").to_string();
    seed_budget_expense_fixture(&app, &period).await;

    create_budget(&app, "salary", &period, 400.0).await;
    create_budget(&app, "invoice", &period, 1_000.0).await;

    let res = app
        .get(&format!(
            "/api/budgets/overview?budget_type=monthly&period={}",
            period
        ))
        .await;
    res.assert_status_ok();

    let body: Value = res.json();
    assert_eq!(body["budget_type"], "monthly");
    assert_eq!(body["period"], period);
    assert_eq!(body["budgets"].as_array().unwrap().len(), 2);
    assert_eq!(body["stats"]["total_budget"].as_f64().unwrap(), 1_400.0);
    assert_eq!(body["stats"]["total_actual"].as_f64().unwrap(), 1_300.0);
    assert_eq!(body["stats"]["exceeded_count"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["saved_amount"].as_f64().unwrap(), 200.0);
    assert!(body["trend"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn s71_budget_overview_is_company_scoped() {
    let app = common::TestApp::new().await;
    let period = Utc::now().format("%Y-%m").to_string();
    seed_budget_expense_fixture(&app, &period).await;
    create_budget(&app, "salary", &period, 400.0).await;

    let rival = app.create_second_tenant().await;
    let res = rival
        .get(&format!(
            "/api/budgets/overview?budget_type=monthly&period={}",
            period
        ))
        .await;
    res.assert_status_ok();

    let body: Value = res.json();
    assert_eq!(body["budgets"].as_array().unwrap().len(), 0);
    assert_eq!(body["stats"]["total_actual"].as_f64().unwrap(), 0.0);
}

#[tokio::test]
async fn s71_budget_period_defaults_are_owned_by_backend() {
    let app = common::TestApp::new().await;
    let current_month = Utc::now().format("%Y-%m").to_string();

    let monthly_overview = app.get("/api/budgets/overview?budget_type=monthly").await;
    monthly_overview.assert_status_ok();
    let monthly_overview: Value = monthly_overview.json();
    assert_eq!(monthly_overview["period"], current_month);

    let quarterly_overview = app.get("/api/budgets/overview?budget_type=quarterly").await;
    quarterly_overview.assert_status_ok();
    let quarterly_overview: Value = quarterly_overview.json();
    assert_eq!(quarterly_overview["period"], current_quarter_period());

    let created = app
        .post("/api/budgets")
        .json(&json!({
            "category": "salary",
            "budget_type": "monthly",
            "budget_amount": "¥800.00"
        }))
        .await;
    created.assert_status_ok();
    let created: Value = created.json();
    assert_eq!(created["period"], current_month);
    assert_eq!(created["budget_amount"].as_f64().unwrap(), 800.0);
}

async fn create_budget(app: &common::TestApp, category: &str, period: &str, amount: f64) {
    app.post("/api/budgets")
        .json(&json!({
            "category": category,
            "budget_type": "monthly",
            "period": period,
            "budget_amount": amount
        }))
        .await
        .assert_status_ok();
}

async fn seed_budget_expense_fixture(app: &common::TestApp, period: &str) {
    let employee_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO employees (id, company_id, name, status) VALUES (?, ?, 'Budget Employee', 'active')")
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
    .bind(period)
    .execute(&app.pool)
    .await
    .unwrap();

    let invoice_date = format!("{}-15", period);
    sqlx::query(
        "INSERT INTO invoices (id, company_id, invoice_number, amount, status, invoice_date)
         VALUES (?, ?, 'BUDGET-INV-1', 800, 'pending', ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .bind(&invoice_date)
    .execute(&app.pool)
    .await
    .unwrap();
}
