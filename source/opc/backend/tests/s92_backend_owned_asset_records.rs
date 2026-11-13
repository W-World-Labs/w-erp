use crate::common;

use chrono::Utc;
use serde_json::Value;

async fn seed_asset(app: &common::TestApp, id: &str, name: &str, serial_number: &str) {
    sqlx::query(
        r#"
        INSERT INTO assets (id, company_id, name, serial_number, asset_type)
        VALUES (?, ?, ?, ?, 'physical')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
    .bind(serial_number)
    .execute(&app.pool)
    .await
    .expect("seed asset");
}

async fn seed_asset_record(
    app: &common::TestApp,
    id: &str,
    asset_id: &str,
    record_type: &str,
    description: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO asset_records (
            id, company_id, asset_id, record_type, description, record_date
        )
        VALUES (?, ?, ?, ?, ?, '2026-05-01')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(asset_id)
    .bind(record_type)
    .bind(description)
    .execute(&app.pool)
    .await
    .expect("seed asset record");
}

#[tokio::test]
async fn s92_asset_records_overview_filters_stats_labels_and_export_on_backend() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_asset(&app, "asset-alpha", "Alpha 服务器", "SN-A").await;
    seed_asset(&app, "asset-beta", "Beta 设备", "SN-B").await;
    seed_asset(&other, "asset-other", "其他租户资产", "SN-X").await;

    seed_asset_record(&app, "record-checkout", "asset-alpha", "checkout", "领用").await;
    seed_asset_record(&app, "record-transfer", "asset-beta", "transfer", "调拨").await;
    seed_asset_record(
        &app,
        "record-maintenance",
        "asset-beta",
        "maintenance",
        "维修",
    )
    .await;
    seed_asset_record(&other, "record-other", "asset-other", "checkout", "隔离").await;

    let res = app
        .get("/api/v1/asset_records/overview?search=Alpha&action=checkout")
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total"].as_i64().unwrap(), 3);
    assert_eq!(body["stats"]["checkout"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["transfer"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["maintenance"].as_i64().unwrap(), 1);

    let records = body["records"].as_array().unwrap();
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record["id"], "record-checkout");
    assert_eq!(record["asset_name"], "Alpha 服务器");
    assert_eq!(record["asset_serial_number"], "SN-A");
    assert_eq!(record["action_label"], "领用");
    assert_eq!(record["action_badge"]["label"], "领用");

    assert!(
        body["action_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| { action["value"] == "maintenance" && action["label"] == "维修" })
    );

    let today = Utc::now().format("%Y%m%d").to_string();
    assert_eq!(
        body["export"]["filename"].as_str().unwrap(),
        format!("asset_records_{today}.csv")
    );
    let csv = body["export"]["csv"].as_str().unwrap();
    assert!(csv.contains("Alpha 服务器"));
    assert!(!csv.contains("Beta 设备"));
    assert!(!csv.contains("其他租户资产"));
}
