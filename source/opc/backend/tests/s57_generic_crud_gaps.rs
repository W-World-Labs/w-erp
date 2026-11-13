/// Scenario 57: 📦 Generic CRUD Gap Coverage (通用CRUD补充)
/// Tests: reminders, profiles, asset_records, utilities, exchange_rates
/// Each entity: create → get → update → list → delete + tenant isolation
use crate::common;
use axum::http::StatusCode;
use serde_json::json;

// ══════════════ Reminders (提醒) ══════════════

#[tokio::test]
async fn s57_reminders_full_crud() {
    let app = common::TestApp::new().await;

    // Create — required: ["title"]
    let res = app
        .post("/api/v1/reminders")
        .json(&json!({
            "title": "合同到期提醒",
            "description": "服务合同将于30天后到期",
            "status": "pending",
            "priority": "high",
            "reminder_type": "contract_expiry"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Get
    let get = app.get(&format!("/api/v1/reminders/{}", id)).await;
    get.assert_status_ok();
    let data: serde_json::Value = get.json();
    assert_eq!(data["title"].as_str().unwrap(), "合同到期提醒");

    // Update
    app.put(&format!("/api/v1/reminders/{}", id))
        .json(&json!({ "status": "done", "description": "已处理" }))
        .await
        .assert_status_ok();

    let updated: serde_json::Value = app.get(&format!("/api/v1/reminders/{}", id)).await.json();
    assert_eq!(updated["status"].as_str().unwrap(), "done");

    // List
    let list = app.get("/api/v1/reminders").await;
    list.assert_status_ok();
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 1);

    // Delete
    let del = app.delete(&format!("/api/v1/reminders/{}", id)).await;
    assert!(del.status_code().is_success());
    let after = app.get(&format!("/api/v1/reminders/{}", id)).await;
    assert_eq!(after.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn s57_reminders_tenant_isolation() {
    let app_a = common::TestApp::new().await;
    let app_b = app_a.create_second_tenant().await;

    app_a
        .post("/api/v1/reminders")
        .json(&json!({ "title": "A的提醒" }))
        .await
        .assert_status_ok();

    let b_list = app_b.get("/api/v1/reminders").await;
    assert_eq!(
        b_list.json::<serde_json::Value>()["count"]
            .as_i64()
            .unwrap(),
        0
    );
}

// ══════════════ Profiles (用户配置) ══════════════

#[tokio::test]
async fn s57_profiles_full_crud() {
    let app = common::TestApp::new().await;

    // required: ["display_name"] — profiles table has: display_name, phone, department, position, email, bio
    let res = app
        .post("/api/v1/profiles")
        .json(&json!({
            "display_name": "测试用户",
            "department": "研发部",
            "position": "工程师"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let get: serde_json::Value = app.get(&format!("/api/v1/profiles/{}", id)).await.json();
    assert_eq!(get["display_name"].as_str().unwrap(), "测试用户");

    app.put(&format!("/api/v1/profiles/{}", id))
        .json(&json!({ "bio": "全栈工程师" }))
        .await
        .assert_status_ok();

    let list = app.get("/api/v1/profiles").await;
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 1);

    app.delete(&format!("/api/v1/profiles/{}", id)).await;
    assert_eq!(
        app.get(&format!("/api/v1/profiles/{}", id))
            .await
            .status_code(),
        StatusCode::NOT_FOUND
    );
}

// ══════════════ Asset Records (资产记录) ══════════════

#[tokio::test]
async fn s57_asset_records_full_crud() {
    let app = common::TestApp::new().await;

    // First create an asset via dedicated module to link the record to
    // asset_records required: ["asset_id", "record_type"]
    let asset = app
        .post("/api/assets")
        .json(&json!({
            "name": "测试资产-s57"
        }))
        .await;
    asset.assert_status_ok();
    // Dedicated assets module returns {"data": {"id": ...}}
    let asset_body: serde_json::Value = asset.json();
    let asset_id = asset_body["data"]["id"].as_str().unwrap().to_string();

    let res = app
        .post("/api/v1/asset_records")
        .json(&json!({
            "asset_id": asset_id,
            "record_type": "maintenance",
            "description": "年度设备维护",
            "amount": 5000.0,
            "record_date": "2026-03-15"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let get: serde_json::Value = app
        .get(&format!("/api/v1/asset_records/{}", id))
        .await
        .json();
    assert_eq!(get["record_type"].as_str().unwrap(), "maintenance");
    assert_eq!(get["asset_id"].as_str().unwrap(), asset_id);

    // Update
    app.put(&format!("/api/v1/asset_records/{}", id))
        .json(&json!({ "amount": 5500.0 }))
        .await
        .assert_status_ok();

    let list = app.get("/api/v1/asset_records").await;
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 1);

    app.delete(&format!("/api/v1/asset_records/{}", id)).await;
    assert_eq!(
        app.get(&format!("/api/v1/asset_records/{}", id))
            .await
            .status_code(),
        StatusCode::NOT_FOUND
    );
}

// ══════════════ Utilities (水电费) ══════════════

#[tokio::test]
async fn s57_utilities_full_crud() {
    let app = common::TestApp::new().await;

    // required: ["location_name", "utility_type", "period"]
    let res = app
        .post("/api/v1/utilities")
        .json(&json!({
            "location_name": "总部办公室",
            "utility_type": "electricity",
            "period": "2026-03",
            "amount": 1200.50,
            "status": "unpaid"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let get: serde_json::Value = app.get(&format!("/api/v1/utilities/{}", id)).await.json();
    assert_eq!(get["utility_type"].as_str().unwrap(), "electricity");
    assert_eq!(get["location_name"].as_str().unwrap(), "总部办公室");

    app.put(&format!("/api/v1/utilities/{}", id))
        .json(&json!({ "status": "paid", "amount": 1200.50 }))
        .await
        .assert_status_ok();

    let list = app.get("/api/v1/utilities").await;
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 1);

    app.delete(&format!("/api/v1/utilities/{}", id)).await;
    assert_eq!(
        app.get(&format!("/api/v1/utilities/{}", id))
            .await
            .status_code(),
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn s57_utilities_tenant_isolation() {
    let app_a = common::TestApp::new().await;
    let app_b = app_a.create_second_tenant().await;

    app_a
        .post("/api/v1/utilities")
        .json(&json!({
            "location_name": "A的办公室",
            "utility_type": "water",
            "period": "2026-02",
            "amount": 300.0
        }))
        .await
        .assert_status_ok();

    let b_list = app_b.get("/api/v1/utilities").await;
    assert_eq!(
        b_list.json::<serde_json::Value>()["count"]
            .as_i64()
            .unwrap(),
        0
    );
}

// ══════════════ Exchange Rates (汇率) ══════════════

#[tokio::test]
async fn s57_exchange_rates_full_crud() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/exchange_rates")
        .json(&json!({
            "from_currency": "USD",
            "to_currency": "CNY",
            "rate": 7.25,
            "effective_date": "2026-03-20",
            "source": "manual"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let get: serde_json::Value = app
        .get(&format!("/api/v1/exchange_rates/{}", id))
        .await
        .json();
    assert_eq!(get["from_currency"].as_str().unwrap(), "USD");
    assert_eq!(get["to_currency"].as_str().unwrap(), "CNY");

    app.put(&format!("/api/v1/exchange_rates/{}", id))
        .json(&json!({ "rate": 7.28 }))
        .await
        .assert_status_ok();

    let updated: serde_json::Value = app
        .get(&format!("/api/v1/exchange_rates/{}", id))
        .await
        .json();
    assert_eq!(updated["rate"].as_f64().unwrap(), 7.28);

    let list = app.get("/api/v1/exchange_rates").await;
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 1);

    app.delete(&format!("/api/v1/exchange_rates/{}", id)).await;
    assert_eq!(
        app.get(&format!("/api/v1/exchange_rates/{}", id))
            .await
            .status_code(),
        StatusCode::NOT_FOUND
    );
}
