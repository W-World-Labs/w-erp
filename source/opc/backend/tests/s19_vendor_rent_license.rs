/// Scenario 19: 🏪 Vendor, Rent, and License CRUD (友商/租赁/证照)
/// Tests generic CRUD operations on tables that lacked dedicated tests
use crate::common;
use serde_json::json;

// ==================== Vendors ====================

#[tokio::test]
async fn s19_vendor_full_lifecycle() {
    let app = common::TestApp::new().await;

    // Create
    let res = app
        .post("/api/v1/vendors")
        .json(&json!({
            "name": "深圳某某科技",
            "short_name": "某某科技",
            "unified_code": "91440300MA1234567X",
            "contact_person": "张三",
            "contact_phone": "00000138000",
            "vendor_type": "supplier",
            "status": "active"
        }))
        .await;
    res.assert_status_ok();
    let vendor_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Read
    let get_res = app.get(&format!("/api/v1/vendors/{}", vendor_id)).await;
    get_res.assert_status_ok();
    let vendor = get_res.json::<serde_json::Value>();
    assert_eq!(vendor["name"].as_str().unwrap(), "深圳某某科技");
    assert_eq!(
        vendor["unified_code"].as_str().unwrap(),
        "91440300MA1234567X"
    );

    // Update
    let put_res = app
        .put(&format!("/api/v1/vendors/{}", vendor_id))
        .json(&json!({ "name": "深圳某某科技有限公司", "status": "inactive" }))
        .await;
    put_res.assert_status_ok();

    // Verify update
    let verify = app.get(&format!("/api/v1/vendors/{}", vendor_id)).await;
    let updated = verify.json::<serde_json::Value>();
    assert_eq!(updated["name"].as_str().unwrap(), "深圳某某科技有限公司");
    assert_eq!(updated["status"].as_str().unwrap(), "inactive");

    // Delete
    let del_res = app.delete(&format!("/api/v1/vendors/{}", vendor_id)).await;
    assert!(del_res.status_code().is_success());
}

#[tokio::test]
async fn s19_vendor_tenant_isolation() {
    let app = common::TestApp::new().await;
    let app_b = app.create_second_tenant().await;

    // A creates vendor
    let res = app
        .post("/api/v1/vendors")
        .json(&json!({ "name": "PrivateVendor", "vendor_type": "supplier" }))
        .await;
    res.assert_status_ok();
    let vid = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // B cannot see it
    let b_res = app_b.get(&format!("/api/v1/vendors/{}", vid)).await;
    assert_eq!(b_res.status_code(), 404);
}

// ==================== Rent Records ====================

#[tokio::test]
async fn s19_rent_record_lifecycle() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/rents")
        .json(&json!({
            "location_name": "创业园A栋301",
            "property_type": "office",
            "address": "深圳市南山区",
            "monthly_rent": 8000.0,
            "deposit": 24000.0,
            "contract_start_date": "2024-01-01",
            "contract_end_date": "2025-12-31",
            "status": "active"
        }))
        .await;
    res.assert_status_ok();
    let rent_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // List
    let list_res = app.get("/api/v1/rents").await;
    list_res.assert_status_ok();
    let list = list_res.json::<serde_json::Value>();
    assert!(list["count"].as_i64().unwrap() >= 1);

    // Update rent amount
    let put_res = app
        .put(&format!("/api/v1/rents/{}", rent_id))
        .json(&json!({ "monthly_rent": 8500.0 }))
        .await;
    put_res.assert_status_ok();

    // Delete
    let del_res = app.delete(&format!("/api/v1/rents/{}", rent_id)).await;
    assert!(del_res.status_code().is_success());
}

// ==================== Licenses ====================

#[tokio::test]
async fn s19_license_lifecycle() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/licenses")
        .json(&json!({
            "license_name": "营业执照",
            "license_type": "business_license",
            "license_number": "91440300MA1234567X",
            "issuing_authority": "深圳市市场监督管理局",
            "issue_date": "2024-01-15",
            "expiry_date": "2054-01-15",
            "status": "valid"
        }))
        .await;
    res.assert_status_ok();
    let lic_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Verify read
    let get_res = app.get(&format!("/api/v1/licenses/{}", lic_id)).await;
    get_res.assert_status_ok();
    let lic = get_res.json::<serde_json::Value>();
    assert_eq!(lic["license_name"].as_str().unwrap(), "营业执照");
    assert_eq!(lic["license_type"].as_str().unwrap(), "business_license");

    // Delete
    let del_res = app.delete(&format!("/api/v1/licenses/{}", lic_id)).await;
    assert!(del_res.status_code().is_success());
}

// ==================== Reminders ====================

#[tokio::test]
async fn s19_reminder_lifecycle() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/reminders")
        .json(&json!({
            "title": "域名续费提醒",
            "description": "example.com 将于2025-03-01到期",
            "reminder_type": "domain_expiry",
            "priority": "high",
            "due_date": "2025-02-15",
            "status": "pending"
        }))
        .await;
    res.assert_status_ok();
    let rem_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Mark as completed
    let put_res = app
        .put(&format!("/api/v1/reminders/{}", rem_id))
        .json(&json!({ "status": "completed", "is_completed": 1 }))
        .await;
    put_res.assert_status_ok();

    // Verify
    let get_res = app.get(&format!("/api/v1/reminders/{}", rem_id)).await;
    let rem = get_res.json::<serde_json::Value>();
    assert_eq!(rem["status"].as_str().unwrap(), "completed");
}

// ==================== Utilities (水电费) ====================

#[tokio::test]
async fn s19_utility_lifecycle() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/utilities")
        .json(&json!({
            "location_name": "A栋办公室",
            "utility_type": "electricity",
            "period": "2024-03",
            "usage": 350.0,
            "unit_price": 0.85,
            "amount": 297.5,
            "status": "unpaid"
        }))
        .await;
    res.assert_status_ok();
    let util_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Pay it
    let put_res = app
        .put(&format!("/api/v1/utilities/{}", util_id))
        .json(&json!({ "status": "paid" }))
        .await;
    put_res.assert_status_ok();

    // Delete
    let del_res = app.delete(&format!("/api/v1/utilities/{}", util_id)).await;
    assert!(del_res.status_code().is_success());
}

// ==================== Exchange Rates ====================

#[tokio::test]
async fn s19_exchange_rate_crud() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/exchange_rates")
        .json(&json!({
            "from_currency": "USD",
            "to_currency": "CNY",
            "rate": 7.25,
            "effective_date": "2024-03-01",
            "source": "manual"
        }))
        .await;
    res.assert_status_ok();
    let rate_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Update rate
    let put_res = app
        .put(&format!("/api/v1/exchange_rates/{}", rate_id))
        .json(&json!({ "rate": 7.28 }))
        .await;
    put_res.assert_status_ok();

    // Verify
    let get_res = app
        .get(&format!("/api/v1/exchange_rates/{}", rate_id))
        .await;
    let rate = get_res.json::<serde_json::Value>();
    assert_eq!(rate["rate"].as_f64().unwrap(), 7.28);
}
