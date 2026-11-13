use crate::common;

use chrono::{Duration, Utc};
use serde_json::Value;

async fn seed_asset(
    app: &common::TestApp,
    id: &str,
    name: &str,
    asset_type: &str,
    status: &str,
    current_value: f64,
    accumulated_depreciation: f64,
    monthly_depreciation: f64,
    expiry_date: Option<String>,
) {
    sqlx::query(
        r#"
        INSERT INTO assets (
            id, company_id, name, asset_type, status, current_value,
            accumulated_depreciation, monthly_depreciation, expiry_date
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
    .bind(asset_type)
    .bind(status)
    .bind(current_value)
    .bind(accumulated_depreciation)
    .bind(monthly_depreciation)
    .bind(expiry_date)
    .execute(&app.pool)
    .await
    .expect("seed asset");
}

#[tokio::test]
async fn s73_asset_overview_calculates_stats_on_backend() {
    let app = common::TestApp::new().await;
    let soon = (Utc::now() + Duration::days(10))
        .format("%Y-%m-%d")
        .to_string();
    let later = (Utc::now() + Duration::days(45))
        .format("%Y-%m-%d")
        .to_string();
    let expired = (Utc::now() - Duration::days(3))
        .format("%Y-%m-%d")
        .to_string();

    seed_asset(
        &app,
        "asset-physical",
        "笔记本",
        "physical",
        "active",
        1000.0,
        100.0,
        10.0,
        None,
    )
    .await;
    seed_asset(
        &app,
        "asset-virtual",
        "域名",
        "virtual",
        "in_use",
        200.0,
        0.0,
        0.0,
        Some(soon),
    )
    .await;
    seed_asset(
        &app,
        "asset-data",
        "数据集",
        "data",
        "in_use",
        400.0,
        0.0,
        0.0,
        Some(expired),
    )
    .await;
    sqlx::query(
        r#"
        INSERT INTO data_asset_meta (
            id, company_id, asset_id, data_type, tech_stack, version, access_url
        )
        VALUES ('meta-data', ?, 'asset-data', 'dataset', 'Rust', '1.0', 'https://data.example')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed data asset meta");
    sqlx::query(
        r#"
        INSERT INTO asset_categories (id, company_id, name, category_type)
        VALUES ('category-overview', ?, '后端分类动作', 'physical')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed category action");
    seed_asset(
        &app,
        "asset-intangible",
        "商标",
        "intangible",
        "idle",
        300.0,
        0.0,
        0.0,
        Some(later),
    )
    .await;

    let res = app.get("/api/assets/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["total"].as_i64().unwrap(), 4);
    assert_eq!(body["in_use"].as_i64().unwrap(), 3);
    assert_eq!(body["type_counts"]["physical"].as_i64().unwrap(), 1);
    assert_eq!(body["type_counts"]["virtual"].as_i64().unwrap(), 1);
    assert_eq!(body["type_counts"]["data"].as_i64().unwrap(), 1);
    assert_eq!(body["type_counts"]["intangible"].as_i64().unwrap(), 1);
    assert_eq!(body["expiring_soon"].as_i64().unwrap(), 1);
    assert_eq!(body["expired"].as_i64().unwrap(), 1);
    assert_eq!(body["total_value"].as_f64().unwrap(), 1900.0);
    assert_eq!(body["total_depreciation"].as_f64().unwrap(), 100.0);
    let data_asset = body["assets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|asset| asset["id"] == "asset-data")
        .expect("data asset");
    assert_eq!(data_asset["data_asset_meta"]["id"], "meta-data");
    assert_eq!(data_asset["data_asset_meta"]["data_type"], "dataset");
    assert_eq!(data_asset["data_asset_meta"]["tech_stack"], "Rust");
    assert_eq!(data_asset["can_mark_maintenance"], true);
    assert_eq!(data_asset["can_mark_scrapped"], true);
    assert_eq!(data_asset["can_mark_in_use"], false);
    assert_eq!(data_asset["mark_maintenance_action"]["method"], "POST");
    assert_eq!(
        data_asset["mark_maintenance_action"]["endpoint"],
        "/api/assets/asset-data/mark_maintenance"
    );
    assert_eq!(data_asset["mark_maintenance_action"]["label"], "标记维修中");
    assert_eq!(
        data_asset["mark_maintenance_action"]["success_message"],
        "资产已标记为维修中"
    );
    assert_eq!(
        data_asset["mark_scrapped_action"]["endpoint"],
        "/api/assets/asset-data/mark_scrapped"
    );
    assert!(data_asset["mark_in_use_action"].is_null());
    assert_eq!(
        data_asset["delete_action"]["endpoint"],
        "/api/assets/asset-data"
    );
    assert_eq!(
        data_asset["delete_action"]["confirm_message"],
        "确定要删除资产「数据集」吗？此操作无法撤销。"
    );
    assert_eq!(
        data_asset["delete_action"]["success_message"],
        "资产删除成功"
    );
    assert_eq!(body["create_action"]["method"], "POST");
    assert_eq!(body["create_action"]["endpoint"], "/api/assets");
    assert_eq!(body["create_action"]["label"], "添加资产");
    assert_eq!(body["create_action"]["success_message"], "资产添加成功");
    assert_eq!(data_asset["update_action"]["method"], "PUT");
    assert_eq!(
        data_asset["update_action"]["endpoint"],
        "/api/assets/asset-data"
    );
    assert_eq!(data_asset["update_action"]["label"], "保存资产");
    assert_eq!(
        data_asset["update_action"]["success_message"],
        "资产更新成功"
    );

    let category = body["categories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|category| category["id"] == "category-overview")
        .expect("category action");
    assert_eq!(body["category_create_action"]["method"], "POST");
    assert_eq!(
        body["category_create_action"]["endpoint"],
        "/api/categories"
    );
    assert_eq!(body["category_create_action"]["label"], "添加分类");
    assert_eq!(
        body["category_create_action"]["success_message"],
        "分类添加成功"
    );
    assert_eq!(category["update_action"]["method"], "PUT");
    assert_eq!(
        category["update_action"]["endpoint"],
        "/api/categories/category-overview"
    );
    assert_eq!(category["update_action"]["label"], "保存分类");
    assert_eq!(category["update_action"]["success_message"], "分类更新成功");
    assert_eq!(
        category["delete_action"]["endpoint"],
        "/api/categories/category-overview"
    );
    assert_eq!(
        category["delete_action"]["confirm_message"],
        "确定要删除分类「后端分类动作」吗？已关联的资产将取消分类关联。"
    );
    assert_eq!(category["delete_action"]["success_message"], "分类删除成功");

    let form_options = app
        .get("/api/assets/form_options?asset_type=physical")
        .await;
    form_options.assert_status_ok();
    let options = form_options.json::<Value>();
    assert_eq!(options["default_values"]["asset_type"], "physical");
    assert_eq!(options["default_values"]["status"], "in_use");
    assert_eq!(options["default_values"]["salvage_rate"], "5");
    assert_eq!(
        options["default_values"]["depreciation_method"],
        "straight_line"
    );
    assert!(
        options["status_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "maintenance" && option["label"] == "维修中")
    );
    assert!(
        options["data_type_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "dataset" && option["label"] == "数据集")
    );
    assert!(
        options["depreciation_method_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "straight_line")
    );
}

#[tokio::test]
async fn s73_asset_lifecycle_status_is_backend_owned() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    let created = app
        .post("/api/assets")
        .json(&serde_json::json!({
            "name": "后端流程资产",
            "asset_type": "physical",
            "status": "scrapped"
        }))
        .await;
    created.assert_status_ok();
    let created_body = created.json::<Value>();
    let asset_id = created_body["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(created_body["data"]["status"], "in_use");

    let update = app
        .put(&format!("/api/assets/{asset_id}"))
        .json(&serde_json::json!({
            "status": "scrapped",
            "location": "前端不能改状态"
        }))
        .await;
    update.assert_status_ok();
    let updated = update.json::<Value>();
    assert_eq!(updated["data"]["status"], "in_use");
    assert_eq!(updated["data"]["location"], "前端不能改状态");

    let maintenance = app
        .post(&format!("/api/assets/{asset_id}/mark_maintenance"))
        .json(&serde_json::json!({}))
        .await;
    maintenance.assert_status_ok();
    let maintenance_body = maintenance.json::<Value>();
    assert_eq!(maintenance_body["status"], "maintenance");
    assert_eq!(maintenance_body["status_badge"]["label"], "维修中");
    assert_eq!(maintenance_body["can_mark_in_use"], true);
    assert_eq!(
        maintenance_body["mark_in_use_action"]["endpoint"],
        format!("/api/assets/{asset_id}/mark_in_use")
    );

    let scrapped = app
        .post(&format!("/api/assets/{asset_id}/mark_scrapped"))
        .json(&serde_json::json!({}))
        .await;
    scrapped.assert_status_ok();
    let scrapped_body = scrapped.json::<Value>();
    assert_eq!(scrapped_body["status"], "scrapped");
    assert_eq!(scrapped_body["status_badge"]["label"], "已报废");
    assert_eq!(scrapped_body["can_mark_scrapped"], false);
    assert!(scrapped_body["mark_scrapped_action"].is_null());

    let in_use = app
        .post(&format!("/api/assets/{asset_id}/mark_in_use"))
        .json(&serde_json::json!({}))
        .await;
    in_use.assert_status_ok();
    let in_use_body = in_use.json::<Value>();
    assert_eq!(in_use_body["status"], "in_use");
    assert_eq!(in_use_body["status_badge"]["label"], "在用");
    assert_eq!(in_use_body["can_mark_in_use"], false);
    assert!(in_use_body["mark_in_use_action"].is_null());

    let other_get = other.get(&format!("/api/assets/{asset_id}")).await;
    other_get.assert_status_not_found();
}

#[tokio::test]
async fn s73_asset_detail_returns_backend_owned_label_fields() {
    let app = common::TestApp::new().await;

    sqlx::query(
        r#"
        INSERT INTO asset_categories (id, company_id, name, category_type)
        VALUES ('category-detail', ?, '后端分类', 'virtual')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed category");

    sqlx::query(
        r#"
        INSERT INTO assets (
            id, company_id, name, asset_type, status, category_id, serial_number
        )
        VALUES (
            'asset-detail', ?, '后端标签资产', 'virtual', 'maintenance',
            'category-detail', 'SN-DETAIL'
        )
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed asset");

    let res = app.get("/api/assets/asset-detail").await;
    res.assert_status_ok();
    let body = res.json::<Value>();
    let asset = &body["data"];
    assert_eq!(asset["name"], "后端标签资产");
    assert_eq!(asset["asset_type_badge"]["label"], "虚拟资产");
    assert_eq!(asset["status_badge"]["label"], "维修中");
    assert_eq!(asset["category_name"], "后端分类");
    assert_eq!(asset["asset_categories"]["name"], "后端分类");
}

#[tokio::test]
async fn s73_asset_list_includes_backend_derived_display_fields() {
    let app = common::TestApp::new().await;
    let soon = (Utc::now() + Duration::days(6))
        .format("%Y-%m-%d")
        .to_string();

    seed_asset(
        &app,
        "asset-row",
        "许可证",
        "virtual",
        "active",
        500.0,
        120.0,
        20.0,
        Some(soon),
    )
    .await;

    let res = app.get("/api/assets").await;
    res.assert_status_ok();
    let body = res.json::<Value>();
    let row = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|asset| asset["id"] == "asset-row")
        .expect("seeded asset row");

    assert_eq!(row["expiry_status"], "urgent");
    assert_eq!(row["days_until_expiry"].as_i64().unwrap(), 6);
    assert_eq!(row["net_value"].as_f64().unwrap(), 380.0);
    assert_eq!(row["monthly_depreciation"].as_f64().unwrap(), 20.0);
    assert_eq!(row["accumulated_depreciation"].as_f64().unwrap(), 120.0);
}

#[tokio::test]
async fn s73_asset_create_calculates_depreciation_on_backend() {
    let app = common::TestApp::new().await;

    let create = app
        .post("/api/assets")
        .json(&serde_json::json!({
            "name": "折旧设备",
            "asset_type": "physical",
            "purchase_price": 1200.0,
            "current_value": 1200.0,
            "useful_life": 12,
            "salvage_rate": 0.1,
            "depreciation_method": "straight_line",
            "monthly_depreciation": 9999.0
        }))
        .await;
    create.assert_status_ok();

    let list = app.get("/api/assets").await;
    list.assert_status_ok();
    let body = list.json::<Value>();
    let row = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|asset| asset["name"] == "折旧设备")
        .expect("created asset row");

    assert_eq!(row["useful_life"].as_i64().unwrap(), 12);
    assert_eq!(row["salvage_rate"].as_f64().unwrap(), 0.1);
    assert_eq!(row["monthly_depreciation"].as_f64().unwrap(), 90.0);
    assert_ne!(row["monthly_depreciation"].as_f64().unwrap(), 9999.0);
}

#[tokio::test]
async fn s73_asset_overview_is_company_scoped() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_asset(
        &app,
        "asset-owned",
        "本公司资产",
        "physical",
        "active",
        100.0,
        0.0,
        0.0,
        None,
    )
    .await;
    seed_asset(
        &other,
        "asset-other",
        "其他公司资产",
        "physical",
        "active",
        900.0,
        0.0,
        0.0,
        None,
    )
    .await;

    let res = app.get("/api/assets/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["total"].as_i64().unwrap(), 1);
    assert_eq!(body["total_value"].as_f64().unwrap(), 100.0);
}
