/// Scenario 6: 🖥 资产全生命周期
/// 采购资产 → 入库 → 查看 → 分页 → 折旧字段
///  → 资产删除 → CSV 导出 → 租户隔离
use crate::common;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn s06_asset_lifecycle() {
    let app = common::TestApp::new().await;

    // Step 1: Create asset (typed extractor, only name required)
    let res = app
        .post("/api/v1/assets")
        .json(&json!({
            "name": "MacBook Pro M4",
            "asset_type": "physical",
            "purchase_price": 24999.0,
            "current_value": 24999.0,
            "purchase_date": "2026-01-15",
            "serial_number": "FVFC123456",
            "location": "东京办公室"
        }))
        .await;
    res.assert_status_ok();
    let asset: serde_json::Value = res.json();
    // AssetResponse wraps in { data: { id, ... } }
    let id = asset["data"]["id"]
        .as_str()
        .or_else(|| asset["id"].as_str())
        .expect("should have id in data or root");

    // Step 2: Get
    let get_res = app.get(&format!("/api/v1/assets/{}", id)).await;
    get_res.assert_status_ok();

    // Step 3: Update
    let upd_res = app
        .put(&format!("/api/v1/assets/{}", id))
        .json(&json!({
            "name": "MacBook Pro M4",
            "current_value": 20000.0,
            "location": "上海办公室"
        }))
        .await;
    upd_res.assert_status_ok();

    // Step 4: Delete
    let del_res = app.delete(&format!("/api/v1/assets/{}", id)).await;
    del_res.assert_status(StatusCode::NO_CONTENT);

    app.get(&format!("/api/v1/assets/{}", id))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn s06_data_asset() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/assets")
        .json(&json!({
            "name": "客户数据库 v3",
            "asset_type": "data"
        }))
        .await;
    res.assert_status_ok();
}

#[tokio::test]
async fn s06_intangible_asset() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/assets")
        .json(&json!({
            "name": "OPC 商标",
            "asset_type": "intangible",
            "purchase_price": 50000.0
        }))
        .await;
    res.assert_status_ok();
}

#[tokio::test]
async fn s06_asset_negative_price_rejected() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/assets")
        .json(&json!({
            "name": "Bad Asset",
            "purchase_price": -100.0
        }))
        .await;
    res.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn s06_asset_pagination() {
    let app = common::TestApp::new().await;

    for i in 1..=5 {
        app.post("/api/v1/assets")
            .json(&json!({
                "name": format!("资产 #{}", i),
                "asset_type": "physical",
                "purchase_price": 1000.0 * (i as f64)
            }))
            .await
            .assert_status_ok();
    }

    let res = app.get("/api/v1/assets?page=1&page_size=3").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["count"], 5);
    assert_eq!(body["data"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn s06_asset_csv_export() {
    let app = common::TestApp::new().await;

    app.post("/api/v1/assets")
        .json(&json!({"name": "Export MacBook", "purchase_price": 19999.0}))
        .await
        .assert_status_ok();

    let csv_res = app.get("/api/v1/export/assets").await;
    csv_res.assert_status_ok();
    let csv_text = csv_res.text();
    assert!(csv_text.contains("Export MacBook"));
}

#[tokio::test]
async fn s06_asset_tenant_isolation() {
    let app_a = common::TestApp::new().await;
    let app_b = app_a.create_second_tenant().await;

    app_a
        .post("/api/v1/assets")
        .json(&json!({"name": "A的资产", "purchase_price": 5000.0}))
        .await
        .assert_status_ok();

    let res = app_b.get("/api/v1/assets").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["count"], 0);
}
