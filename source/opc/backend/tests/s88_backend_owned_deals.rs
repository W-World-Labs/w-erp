use crate::common;

use serde_json::Value;

async fn seed_client(app: &common::TestApp, id: &str, name: &str) {
    sqlx::query(
        r#"
        INSERT INTO clients (id, company_id, name, client_type)
        VALUES (?, ?, ?, 'active')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
    .execute(&app.pool)
    .await
    .expect("seed client");
}

async fn seed_deal(
    app: &common::TestApp,
    id: &str,
    client_id: &str,
    title: &str,
    stage: &str,
    amount: f64,
    probability: i64,
    lost_reason: Option<&str>,
) {
    sqlx::query(
        r#"
        INSERT INTO deals (
            id, company_id, client_id, title, stage, amount, probability, lost_reason
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(client_id)
    .bind(title)
    .bind(stage)
    .bind(amount)
    .bind(probability)
    .bind(lost_reason)
    .execute(&app.pool)
    .await
    .expect("seed deal");
}

fn assert_f64(actual: &Value, expected: f64) {
    let actual = actual.as_f64().expect("number");
    assert!(
        (actual - expected).abs() < 0.001,
        "expected {expected}, got {actual}"
    );
}

#[tokio::test]
async fn s88_deals_overview_calculates_pipeline_and_client_labels_on_backend() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_client(&app, "client-a", "Alpha 客户").await;
    seed_client(&app, "client-b", "Beta 客户").await;
    seed_client(&other, "client-other", "Other 客户").await;

    seed_deal(
        &app,
        "deal-lead",
        "client-a",
        "首轮商机",
        "lead",
        100_000.0,
        10,
        None,
    )
    .await;
    seed_deal(
        &app,
        "deal-proposal",
        "client-b",
        "方案商机",
        "proposal",
        300_000.0,
        60,
        None,
    )
    .await;
    seed_deal(
        &app,
        "deal-won",
        "client-a",
        "已赢单",
        "won",
        500_000.0,
        100,
        None,
    )
    .await;
    seed_deal(
        &app,
        "deal-lost",
        "client-b",
        "已丢单",
        "lost",
        50_000.0,
        5,
        Some("价格原因"),
    )
    .await;
    seed_deal(
        &other,
        "deal-other",
        "client-other",
        "其他租户",
        "lead",
        999_000.0,
        99,
        None,
    )
    .await;

    let res = app.get("/api/v1/deals/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["active_count"].as_i64().unwrap(), 2);
    assert_f64(&body["stats"]["total_pipeline"], 400_000.0);
    assert_f64(&body["stats"]["won_amount"], 500_000.0);
    assert_eq!(body["stats"]["avg_probability"].as_i64().unwrap(), 44);

    let stages = body["pipeline_stages"].as_array().unwrap();
    let lead = stages
        .iter()
        .find(|stage| stage["value"] == "lead")
        .expect("lead stage");
    assert_eq!(lead["count"].as_i64().unwrap(), 1);
    assert_f64(&lead["amount"], 100_000.0);
    assert_eq!(lead["deals"][0]["client_name"], "Alpha 客户");
    assert!(
        lead["deals"][0]["next_stages"]
            .as_array()
            .unwrap()
            .iter()
            .any(|stage| stage["value"] == "qualified")
    );

    assert_eq!(body["won_deals"].as_array().unwrap().len(), 1);
    assert_eq!(body["lost_deals"][0]["lost_reason"], "价格原因");

    let clients = body["clients"].as_array().unwrap();
    assert_eq!(clients.len(), 2);
    assert!(clients.iter().all(|client| client["id"] != "client-other"));
}
