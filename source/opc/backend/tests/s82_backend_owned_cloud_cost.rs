use crate::common;

use serde_json::Value;

async fn seed_provider(app: &common::TestApp, id: &str) {
    sqlx::query(
        r#"
        INSERT INTO cloud_providers (
            id, company_id, provider_name, provider_type, account_name
        )
        VALUES (?, ?, 'Provider', 'aws', 'Main')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed provider");
}

async fn seed_resource(app: &common::TestApp, id: &str, monthly_cost: f64) {
    sqlx::query(
        r#"
        INSERT INTO cloud_resources (
            id, company_id, resource_type, resource_name, start_date, monthly_cost
        )
        VALUES (?, ?, 'server', 'Instance', '2026-01-01', ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(monthly_cost)
    .execute(&app.pool)
    .await
    .expect("seed resource");
}

async fn seed_domain(app: &common::TestApp, id: &str, annual_fee: f64) {
    sqlx::query(
        r#"
        INSERT INTO domain_subscriptions (
            id, company_id, domain_name, registrar, registration_date, expiry_date, annual_fee
        )
        VALUES (?, ?, 'example.com', 'Registrar', '2026-01-01', '2027-01-01', ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(annual_fee)
    .execute(&app.pool)
    .await
    .expect("seed domain");
}

async fn seed_software(app: &common::TestApp, id: &str, total_price: f64, billing_cycle: &str) {
    sqlx::query(
        r#"
        INSERT INTO software_subscriptions (
            id, company_id, software_name, vendor, subscription_type, total_price,
            billing_cycle, start_date, next_billing_date
        )
        VALUES (?, ?, 'SaaS', 'Vendor', 'saas', ?, ?, '2026-01-01', '2026-02-01')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(total_price)
    .bind(billing_cycle)
    .execute(&app.pool)
    .await
    .expect("seed software");
}

#[tokio::test]
async fn s82_cloud_cost_overview_calculates_monthly_costs_on_backend() {
    let app = common::TestApp::new().await;

    seed_provider(&app, "provider-a").await;
    seed_resource(&app, "resource-a", 100.0).await;
    seed_resource(&app, "resource-b", 200.0).await;
    seed_domain(&app, "domain-a", 120.0).await;
    seed_domain(&app, "domain-b", 60.0).await;
    seed_software(&app, "software-monthly", 30.0, "monthly").await;
    seed_software(&app, "software-quarterly", 90.0, "quarterly").await;
    seed_software(&app, "software-annually", 120.0, "annually").await;

    let res = app.get("/api/v1/cloud_cost/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["total_providers"].as_i64().unwrap(), 1);
    assert_eq!(body["total_resources"].as_i64().unwrap(), 2);
    assert_eq!(body["total_domains"].as_i64().unwrap(), 2);
    assert_eq!(body["total_software"].as_i64().unwrap(), 3);
    assert_eq!(body["monthly_cloud_cost"].as_f64().unwrap(), 300.0);
    assert_eq!(body["monthly_domain_cost"].as_f64().unwrap(), 15.0);
    assert_eq!(body["monthly_software_cost"].as_f64().unwrap(), 70.0);
    assert_eq!(body["total_monthly_cost"].as_f64().unwrap(), 385.0);
}

#[tokio::test]
async fn s82_cloud_cost_overview_is_company_scoped() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_provider(&app, "owned-provider").await;
    seed_resource(&app, "owned-resource", 100.0).await;
    seed_resource(&other, "other-resource", 900.0).await;

    let res = app.get("/api/v1/cloud_cost/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["total_providers"].as_i64().unwrap(), 1);
    assert_eq!(body["total_resources"].as_i64().unwrap(), 1);
    assert_eq!(body["monthly_cloud_cost"].as_f64().unwrap(), 100.0);
    assert_eq!(body["total_monthly_cost"].as_f64().unwrap(), 100.0);
}
