use crate::common;

use chrono::{Duration, Utc};
use serde_json::{Value, json};

async fn seed_employee(
    app: &common::TestApp,
    id: &str,
    name: &str,
    department: &str,
    position: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO employees (id, company_id, name, department, position, status)
        VALUES (?, ?, ?, ?, ?, 'active')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
    .bind(department)
    .bind(position)
    .execute(&app.pool)
    .await
    .expect("seed employee");
}

async fn seed_salary(
    app: &common::TestApp,
    id: &str,
    employee_id: &str,
    period: &str,
    base_salary: f64,
    bonus: f64,
    deduction: f64,
    social_security: f64,
    housing_fund: f64,
    social_security_company: f64,
    housing_fund_company: f64,
    net_salary: f64,
    status: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO salary_records (
            id, company_id, employee_id, period, base_salary, bonus, deduction,
            social_security, housing_fund, social_security_company, housing_fund_company,
            net_salary, status
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(employee_id)
    .bind(period)
    .bind(base_salary)
    .bind(bonus)
    .bind(deduction)
    .bind(social_security)
    .bind(housing_fund)
    .bind(social_security_company)
    .bind(housing_fund_company)
    .bind(net_salary)
    .bind(status)
    .execute(&app.pool)
    .await
    .expect("seed salary");
}

#[tokio::test]
async fn s77_salary_overview_calculates_stats_trends_and_analysis_on_backend() {
    let app = common::TestApp::new().await;
    let current_period = Utc::now().format("%Y-%m").to_string();
    let previous_period = (Utc::now() - Duration::days(31))
        .format("%Y-%m")
        .to_string();

    seed_employee(&app, "emp-a", "张三", "研发部", "工程师").await;
    seed_employee(&app, "emp-b", "李四", "市场部", "经理").await;

    seed_salary(
        &app,
        "salary-a",
        "emp-a",
        &current_period,
        1000.0,
        100.0,
        50.0,
        20.0,
        30.0,
        40.0,
        50.0,
        1000.0,
        "pending",
    )
    .await;
    seed_salary(
        &app,
        "salary-b",
        "emp-b",
        &current_period,
        2000.0,
        0.0,
        0.0,
        100.0,
        100.0,
        200.0,
        200.0,
        1800.0,
        "paid",
    )
    .await;
    seed_salary(
        &app,
        "salary-old",
        "emp-a",
        &previous_period,
        500.0,
        0.0,
        0.0,
        0.0,
        0.0,
        20.0,
        20.0,
        500.0,
        "paid",
    )
    .await;

    let res = app.get("/api/v1/salary/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total"].as_f64().unwrap(), 3600.0);
    assert_eq!(body["stats"]["employees"].as_i64().unwrap(), 2);
    assert_eq!(body["stats"]["average"].as_f64().unwrap(), 1200.0);
    assert_eq!(body["stats"]["pending"].as_i64().unwrap(), 1);

    let current_trend = body["monthly_trend"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["period"] == current_period)
        .expect("current month trend");
    assert_eq!(current_trend["total"].as_f64().unwrap(), 3100.0);

    let salary = body["salaries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "salary-a")
        .expect("salary row");
    assert_eq!(salary["employee_name"], "张三");
    assert_eq!(salary["department"], "研发部");
    assert_eq!(salary["position"], "工程师");
    assert_eq!(salary["total_salary"].as_f64().unwrap(), 1100.0);
    assert_eq!(salary["actual_salary"].as_f64().unwrap(), 1000.0);

    assert_eq!(
        body["cost_analysis"]["stats"]["total_cost"]
            .as_f64()
            .unwrap(),
        3290.0
    );
    assert_eq!(
        body["cost_analysis"]["stats"]["employee_cost"]
            .as_f64()
            .unwrap(),
        2800.0
    );
    assert_eq!(
        body["cost_analysis"]["stats"]["company_cost"]
            .as_f64()
            .unwrap(),
        490.0
    );
    assert_eq!(
        body["cost_analysis"]["department_costs"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[tokio::test]
async fn s77_salary_create_calculates_net_salary_on_backend() {
    let app = common::TestApp::new().await;
    seed_employee(&app, "emp-a", "张三", "研发部", "工程师").await;

    let create = app
        .post("/api/v1/salary")
        .json(&json!({
            "employee_id": "emp-a",
            "period": "2026-05",
            "base_salary": 1000.0,
            "bonus": 100.0,
            "deduction": 50.0,
            "social_security": 20.0,
            "housing_fund": 30.0,
            "net_salary": 1.0,
            "actual_salary": 1.0,
            "total_salary": 1.0
        }))
        .await;
    create.assert_status_ok();
    let id = create.json::<Value>()["id"].as_str().unwrap().to_string();

    let get = app.get(&format!("/api/v1/salary/{}", id)).await;
    get.assert_status_ok();
    let salary = get.json::<Value>();

    assert_eq!(salary["net_salary"].as_f64().unwrap(), 1000.0);
    assert!(salary.get("actual_salary").is_none());
    assert!(salary.get("total_salary").is_none());
}

#[tokio::test]
async fn s77_salary_overview_is_company_scoped() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;
    let current_period = Utc::now().format("%Y-%m").to_string();

    seed_employee(&app, "emp-owned", "本公司员工", "研发部", "工程师").await;
    seed_employee(&other, "emp-other", "其他公司员工", "市场部", "经理").await;
    seed_salary(
        &app,
        "salary-owned",
        "emp-owned",
        &current_period,
        100.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        100.0,
        "pending",
    )
    .await;
    seed_salary(
        &other,
        "salary-other",
        "emp-other",
        &current_period,
        900.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        900.0,
        "pending",
    )
    .await;

    let res = app.get("/api/v1/salary/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total"].as_f64().unwrap(), 100.0);
    assert_eq!(body["stats"]["employees"].as_i64().unwrap(), 1);
}
