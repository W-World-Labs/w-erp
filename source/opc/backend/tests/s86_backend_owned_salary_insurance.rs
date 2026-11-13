use crate::common;

use axum::http::StatusCode;
use serde_json::{Value, json};

async fn seed_employee(
    app: &common::TestApp,
    id: &str,
    base_salary: f64,
    social_security_base: f64,
    housing_fund_base: f64,
) {
    sqlx::query(
        r#"
        INSERT INTO employees (
            id, company_id, name, department, position, base_salary,
            social_security_base, housing_fund_base, status
        )
        VALUES (?, ?, ?, 'Operations', 'Manager', ?, ?, ?, 'active')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(format!("Employee {id}"))
    .bind(base_salary)
    .bind(social_security_base)
    .bind(housing_fund_base)
    .execute(&app.pool)
    .await
    .expect("seed employee");
}

async fn seed_insurance_config(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO social_insurance_configs (
            id, company_id, city, social_security_base_min, social_security_base_max,
            housing_fund_base_min, housing_fund_base_max,
            pension_personal_rate, medical_personal_rate, unemployment_personal_rate,
            pension_company_rate, medical_company_rate, unemployment_company_rate,
            work_injury_company_rate, maternity_company_rate,
            housing_fund_personal_rate, housing_fund_company_rate
        )
        VALUES (
            'cfg-main', ?, 'Hangzhou', 5000.0, 10000.0,
            3000.0, 8000.0,
            8.0, 2.0, 0.5,
            14.0, 9.5, 0.5,
            0.2, 0.0,
            12.0, 12.0
        )
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed insurance config");
}

fn assert_f64(actual: &Value, expected: f64) {
    let actual = actual.as_f64().expect("number");
    assert!(
        (actual - expected).abs() < 0.001,
        "expected {expected}, got {actual}"
    );
}

#[tokio::test]
async fn s86_salary_insurance_preview_uses_employee_bases_and_company_config_on_backend() {
    let app = common::TestApp::new().await;
    seed_insurance_config(&app).await;
    seed_employee(&app, "emp-a", 7000.0, 12000.0, 2000.0).await;

    let res = app
        .post("/api/v1/salary/insurance-preview")
        .json(&json!({
            "employee_id": "emp-a",
            "base_salary": 6000.0
        }))
        .await;

    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_f64(&body["social_security_base"], 10000.0);
    assert_f64(&body["housing_fund_base"], 3000.0);
    assert_f64(&body["social_security"], 1050.0);
    assert_f64(&body["social_security_company"], 2420.0);
    assert_f64(&body["housing_fund"], 360.0);
    assert_f64(&body["housing_fund_company"], 360.0);
}

#[tokio::test]
async fn s86_salary_insurance_preview_falls_back_to_salary_when_employee_bases_are_missing() {
    let app = common::TestApp::new().await;
    seed_insurance_config(&app).await;
    seed_employee(&app, "emp-b", 7000.0, 0.0, 0.0).await;

    let res = app
        .post("/api/v1/salary/insurance-preview")
        .json(&json!({
            "employee_id": "emp-b",
            "base_salary": 4500.0
        }))
        .await;

    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_f64(&body["social_security_base"], 5000.0);
    assert_f64(&body["housing_fund_base"], 4500.0);
    assert_f64(&body["social_security"], 525.0);
    assert_f64(&body["housing_fund"], 540.0);
}

#[tokio::test]
async fn s86_salary_insurance_preview_accepts_form_strings_and_returns_input_values() {
    let app = common::TestApp::new().await;
    seed_insurance_config(&app).await;

    let res = app
        .post("/api/v1/salary/insurance-preview")
        .json(&json!({
            "base_salary": "4500"
        }))
        .await;

    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_f64(&body["social_security"], 525.0);
    assert_f64(&body["housing_fund"], 540.0);
    assert_eq!(body["social_security_input"], "525.00");
    assert_eq!(body["housing_fund_input"], "540.00");
    assert_eq!(body["social_security_company_input"], "1210.00");
    assert_eq!(body["housing_fund_company_input"], "540.00");
}

#[tokio::test]
async fn s86_salary_insurance_preview_is_company_scoped() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;
    seed_insurance_config(&app).await;
    seed_employee(&other, "emp-other", 7000.0, 7000.0, 7000.0).await;

    let res = app
        .post("/api/v1/salary/insurance-preview")
        .json(&json!({
            "employee_id": "emp-other",
            "base_salary": 7000.0
        }))
        .await;

    res.assert_status(StatusCode::NOT_FOUND);
}
