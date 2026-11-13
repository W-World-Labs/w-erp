/// Scenario 31: 🌍 GlobalOpcHub Seed Data (OPC 出海沙盘数据)
/// Tests OPC hub seeding and data verification
/// Note: global_opc_hubs is a shared table (no company_id), API at /api/global_opc_hubs
use crate::common;
use serde_json::json;

#[tokio::test]
async fn s31_seed_opc_data() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/global_opc_hub/seed")
        .json(&json!({}))
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    assert!(body["success"].as_bool().unwrap());
    assert!(body["hubs_seeded"].as_i64().unwrap() >= 8);
}

#[tokio::test]
async fn s31_verify_seeded_countries() {
    let app = common::TestApp::new().await;

    // Seed via our new endpoint
    app.post("/api/v1/global_opc_hub/seed")
        .json(&json!({}))
        .await
        .assert_status_ok();

    // List via dedicated global_opc_hubs router (no /v1/ prefix)
    let list = app.get("/api/global_opc_hubs").await;
    list.assert_status_ok();
    let data = list.json::<serde_json::Value>();
    let items = data["data"].as_array().unwrap();

    let countries: Vec<&str> = items
        .iter()
        .filter_map(|i| i["country_code"].as_str())
        .collect();
    for code in &["CN", "KR", "JP", "US", "SG", "HK", "GB", "DE"] {
        assert!(countries.contains(code), "Should include {}", code);
    }
}

#[tokio::test]
async fn s31_verify_china_hub() {
    let app = common::TestApp::new().await;
    app.post("/api/v1/global_opc_hub/seed")
        .json(&json!({}))
        .await
        .assert_status_ok();

    let list = app.get("/api/global_opc_hubs").await;
    let items = list.json::<serde_json::Value>()["data"]
        .as_array()
        .unwrap()
        .clone();
    let china: Vec<_> = items
        .iter()
        .filter(|h| h["country_code"].as_str() == Some("CN"))
        .collect();
    assert!(!china.is_empty());
    assert_eq!(china[0]["currency"].as_str().unwrap(), "CNY");
    assert!(china[0]["corporate_tax_rate"].as_f64().unwrap() > 20.0);
}

#[tokio::test]
async fn s31_verify_korea_hub() {
    let app = common::TestApp::new().await;
    app.post("/api/v1/global_opc_hub/seed")
        .json(&json!({}))
        .await
        .assert_status_ok();

    let list = app.get("/api/global_opc_hubs").await;
    let items = list.json::<serde_json::Value>()["data"]
        .as_array()
        .unwrap()
        .clone();
    let kr: Vec<_> = items
        .iter()
        .filter(|h| h["country_code"].as_str() == Some("KR"))
        .collect();
    assert!(!kr.is_empty());
    assert_eq!(kr[0]["currency"].as_str().unwrap(), "KRW");
    assert_eq!(kr[0]["city"].as_str().unwrap(), "Seoul");
}

#[tokio::test]
async fn s31_hub_tax_calculator() {
    let app = common::TestApp::new().await;
    app.post("/api/v1/global_opc_hub/seed")
        .json(&json!({}))
        .await
        .assert_status_ok();

    // Test the existing tax calculator endpoint
    let res = app
        .post("/api/global_opc_hubs/calculator")
        .json(&json!({ "est_annual_revenue": 100000.0, "current_base_col_index": 80.0 }))
        .await;
    res.assert_status_ok();
    let results = res.json::<serde_json::Value>();
    // Should return calculation results for each hub
    assert!(results.as_array().map(|a| a.len()).unwrap_or(0) > 0);
}
