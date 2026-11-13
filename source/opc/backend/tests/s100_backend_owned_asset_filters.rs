use crate::common;

use chrono::{Duration, Utc};
use serde_json::Value;

async fn seed_category(app: &common::TestApp, id: &str, name: &str, category_type: &str) {
    sqlx::query(
        r#"
        INSERT INTO asset_categories (id, company_id, name, category_type, description)
        VALUES (?, ?, ?, ?, 'seed category')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
    .bind(category_type)
    .execute(&app.pool)
    .await
    .expect("seed category");
}

async fn seed_asset(
    app: &common::TestApp,
    id: &str,
    name: &str,
    asset_type: &str,
    status: &str,
    category_id: Option<&str>,
    current_value: f64,
    accumulated_depreciation: f64,
    monthly_depreciation: f64,
    expiry_date: Option<String>,
    created_by: Option<&str>,
) {
    sqlx::query(
        r#"
        INSERT INTO assets (
            id, company_id, name, asset_type, status, category_id, serial_number,
            location, current_value, accumulated_depreciation, monthly_depreciation,
            expiry_date, created_by
        )
        VALUES (?, ?, ?, ?, ?, ?, 'SN-NEEDLE', 'Needle room', ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
    .bind(asset_type)
    .bind(status)
    .bind(category_id)
    .bind(current_value)
    .bind(accumulated_depreciation)
    .bind(monthly_depreciation)
    .bind(expiry_date)
    .bind(created_by)
    .execute(&app.pool)
    .await
    .expect("seed asset");
}

#[tokio::test]
async fn s100_assets_overview_filters_labels_and_permissions_on_backend() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;
    let soon = (Utc::now() + Duration::days(6))
        .format("%Y-%m-%d")
        .to_string();

    seed_category(&app, "cat-physical", "设备", "physical").await;
    seed_category(&app, "cat-virtual", "订阅", "virtual").await;
    seed_category(&other, "cat-other", "其他租户分类", "physical").await;

    seed_asset(
        &app,
        "asset-needle",
        "Needle MacBook",
        "physical",
        "active",
        Some("cat-physical"),
        1200.0,
        200.0,
        50.0,
        None,
        Some(&app.user_id),
    )
    .await;
    seed_asset(
        &app,
        "asset-domain",
        "Needle Domain",
        "virtual",
        "idle",
        Some("cat-virtual"),
        300.0,
        0.0,
        0.0,
        Some(soon),
        Some("someone-else"),
    )
    .await;
    seed_asset(
        &other,
        "asset-other",
        "Needle Rival",
        "physical",
        "active",
        Some("cat-other"),
        999.0,
        0.0,
        0.0,
        None,
        Some(&other.user_id),
    )
    .await;

    let res = app
        .get("/api/assets/overview?search=macbook&status=active&category_id=cat-physical&asset_type=physical")
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["total"].as_i64().unwrap(), 2);
    assert_eq!(body["total_value_label"], "¥1500");
    assert_eq!(body["total_depreciation_label"], "¥200");
    assert_eq!(body["permissions"]["can_manage_categories"], true);

    let assets = body["assets"].as_array().unwrap();
    assert_eq!(assets.len(), 1);
    let asset = &assets[0];
    assert_eq!(asset["id"], "asset-needle");
    assert_eq!(asset["status_badge"]["label"], "在用");
    assert_eq!(asset["asset_type_badge"]["label"], "固定资产");
    assert_eq!(asset["expiry_badge"]["visible"], false);
    assert_eq!(asset["current_value_label"], "¥1200");
    assert_eq!(asset["net_value_label"], "¥1000");
    assert_eq!(asset["monthly_depreciation_label"], "月折旧 ¥50");
    assert_eq!(asset["can_delete"], true);
    assert!(assets.iter().all(|item| item["id"] != "asset-other"));

    assert!(
        body["status_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "maintenance" && option["label"] == "维修中")
    );
    assert_eq!(body["status_options"][0]["value"], "all");
    assert_eq!(body["status_options"][0]["label"], "全部状态");
    assert!(
        body["asset_type_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "virtual" && option["label"] == "虚拟资产")
    );
    assert_eq!(body["asset_type_options"][0]["value"], "all");
    assert_eq!(body["asset_type_options"][0]["label"], "全部类型");
    assert!(
        body["category_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "cat-physical" && option["label"] == "设备")
    );
    assert_eq!(body["category_options"][0]["value"], "all");
    assert_eq!(body["category_options"][0]["label"], "全部分类");

    let form_options = app.get("/api/categories/form_options").await;
    form_options.assert_status_ok();
    let form_options = form_options.json::<Value>();
    assert_eq!(form_options["default_values"]["category_type"], "physical");
    assert!(
        form_options["category_type_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "both" && option["label"] == "通用")
    );
}
