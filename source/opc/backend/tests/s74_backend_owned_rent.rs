use crate::common;

use chrono::{Duration, Utc};
use serde_json::Value;

async fn seed_rent(
    app: &common::TestApp,
    id: &str,
    monthly_rent: f64,
    area: f64,
    status: &str,
    contract_end_date: Option<String>,
) {
    sqlx::query(
        r#"
        INSERT INTO rent_records (
            id, company_id, location_name, monthly_rent, area, status, contract_end_date
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(format!("场地 {}", id))
    .bind(monthly_rent)
    .bind(area)
    .bind(status)
    .bind(contract_end_date)
    .execute(&app.pool)
    .await
    .expect("seed rent");
}

async fn seed_utility(
    app: &common::TestApp,
    id: &str,
    utility_type: &str,
    period: &str,
    amount: f64,
    status: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO utilities (
            id, company_id, location_name, utility_type, period, amount, status
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind("总部")
    .bind(utility_type)
    .bind(period)
    .bind(amount)
    .bind(status)
    .execute(&app.pool)
    .await
    .expect("seed utility");
}

#[tokio::test]
async fn s74_rent_overview_calculates_rent_and_utility_stats_on_backend() {
    let app = common::TestApp::new().await;
    let current_period = Utc::now().format("%Y-%m").to_string();
    let expiring = (Utc::now() + Duration::days(60))
        .format("%Y-%m-%d")
        .to_string();
    let later = (Utc::now() + Duration::days(180))
        .format("%Y-%m-%d")
        .to_string();

    seed_rent(&app, "active-a", 100.0, 50.0, "active", Some(expiring)).await;
    seed_rent(&app, "active-b", 200.0, 30.0, "active", Some(later)).await;
    seed_rent(&app, "expired", 900.0, 90.0, "expired", None).await;

    seed_utility(
        &app,
        "electricity-pending",
        "electricity",
        &current_period,
        40.0,
        "pending",
    )
    .await;
    seed_utility(
        &app,
        "water-pending",
        "water",
        &current_period,
        20.0,
        "pending",
    )
    .await;
    seed_utility(&app, "gas-pending", "gas", &current_period, 30.0, "pending").await;
    seed_utility(&app, "gas-paid", "gas", &current_period, 5.0, "paid").await;

    let res = app.get("/api/v1/rent/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["rent_stats"]["total"].as_f64().unwrap(), 300.0);
    assert_eq!(body["rent_stats"]["locations"].as_i64().unwrap(), 2);
    assert_eq!(body["rent_stats"]["area"].as_f64().unwrap(), 80.0);
    assert_eq!(body["rent_stats"]["expiring"].as_i64().unwrap(), 1);

    assert_eq!(body["utility_stats"]["total"].as_f64().unwrap(), 90.0);
    assert_eq!(body["utility_stats"]["electricity"].as_f64().unwrap(), 40.0);
    assert_eq!(body["utility_stats"]["water"].as_f64().unwrap(), 20.0);
    assert_eq!(body["utility_stats"]["gas"].as_f64().unwrap(), 30.0);

    let rent_trend = body["rent_trend"].as_array().unwrap();
    assert_eq!(rent_trend.len(), 6);
    assert_eq!(rent_trend.last().unwrap()["total"].as_f64().unwrap(), 300.0);

    let current_utility = body["utility_trend"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["period"] == current_period)
        .expect("current utility trend row");
    assert_eq!(current_utility["electricity"].as_f64().unwrap(), 40.0);
    assert_eq!(current_utility["water"].as_f64().unwrap(), 20.0);
    assert_eq!(current_utility["gas"].as_f64().unwrap(), 35.0);
}

#[tokio::test]
async fn s74_rent_overview_is_company_scoped() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_rent(&app, "owned", 100.0, 10.0, "active", None).await;
    seed_rent(&other, "other", 900.0, 90.0, "active", None).await;

    let res = app.get("/api/v1/rent/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["rent_stats"]["total"].as_f64().unwrap(), 100.0);
    assert_eq!(body["rent_stats"]["locations"].as_i64().unwrap(), 1);
}
