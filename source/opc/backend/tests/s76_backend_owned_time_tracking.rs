use crate::common;

use chrono::{Duration, Utc};
use serde_json::{Value, json};

async fn seed_time_entry(
    app: &common::TestApp,
    id: &str,
    date: &str,
    duration_minutes: i64,
    is_billable: bool,
    amount: f64,
    status: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO time_entries (
            id, company_id, description, date, duration_minutes,
            is_billable, amount, status
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(format!("工时 {}", id))
    .bind(date)
    .bind(duration_minutes)
    .bind(if is_billable { 1 } else { 0 })
    .bind(amount)
    .bind(status)
    .execute(&app.pool)
    .await
    .expect("seed time entry");
}

#[tokio::test]
async fn s76_time_overview_calculates_stats_on_backend() {
    let app = common::TestApp::new().await;
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let yesterday = (Utc::now() - Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    seed_time_entry(&app, "today-billable", &today, 60, true, 100.0, "logged").await;
    seed_time_entry(&app, "today-internal", &today, 30, false, 0.0, "logged").await;
    seed_time_entry(
        &app,
        "old-invoiced",
        &yesterday,
        120,
        true,
        200.0,
        "invoiced",
    )
    .await;
    seed_time_entry(&app, "old-unbilled", &yesterday, 90, true, 150.0, "logged").await;

    let res = app.get("/api/v1/time_entries/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["today_minutes"].as_i64().unwrap(), 90);
    assert_eq!(body["total_records"].as_i64().unwrap(), 4);
    assert_eq!(body["billable_minutes"].as_i64().unwrap(), 270);
    assert_eq!(body["billable_amount"].as_f64().unwrap(), 450.0);
    assert_eq!(body["unbilled_count"].as_i64().unwrap(), 2);
}

#[tokio::test]
async fn s76_time_entry_create_calculates_amount_on_backend() {
    let app = common::TestApp::new().await;

    let create = app
        .post("/api/v1/time_entries")
        .json(&json!({
            "description": "后端计费计算",
            "date": "2026-05-09",
            "duration_minutes": 120,
            "hourly_rate": 300.0,
            "is_billable": 1
        }))
        .await;
    create.assert_status_ok();
    let id = create.json::<Value>()["id"].as_str().unwrap().to_string();

    let get = app.get(&format!("/api/v1/time_entries/{}", id)).await;
    get.assert_status_ok();
    let entry = get.json::<Value>();

    assert_eq!(entry["amount"].as_f64().unwrap(), 600.0);
}

#[tokio::test]
async fn s76_time_overview_is_company_scoped() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;
    let today = Utc::now().format("%Y-%m-%d").to_string();

    seed_time_entry(&app, "owned", &today, 30, true, 50.0, "logged").await;
    seed_time_entry(&other, "other", &today, 300, true, 500.0, "logged").await;

    let res = app.get("/api/v1/time_entries/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["total_records"].as_i64().unwrap(), 1);
    assert_eq!(body["today_minutes"].as_i64().unwrap(), 30);
    assert_eq!(body["billable_amount"].as_f64().unwrap(), 50.0);
}
