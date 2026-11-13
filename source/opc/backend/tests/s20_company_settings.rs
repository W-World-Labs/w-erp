/// Scenario 20: 🏢 Company Settings & Profile Management
/// Tests: company CRUD, company info, social insurance config, KPI config/records
use crate::common;
use serde_json::json;

// ==================== Company CRUD ====================

#[tokio::test]
async fn s20_create_and_switch_company() {
    let app = common::TestApp::new().await;

    // Create second company
    let res = app
        .post("/api/companies")
        .json(&json!({ "name": "海外子公司", "legal_name": "Overseas Co Ltd" }))
        .await;
    res.assert_status_ok();
    let company2_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Switch to it
    let switch_res = app
        .post(&format!("/api/companies/{}/switch", company2_id))
        .await;
    switch_res.assert_status_ok();
    let switched = switch_res.json::<serde_json::Value>();
    assert_eq!(
        switched["current_company_id"].as_str().unwrap(),
        company2_id
    );

    // List companies — should have 2
    let list_res = app.get("/api/companies").await;
    let companies = list_res.json::<serde_json::Value>()["data"]
        .as_array()
        .unwrap()
        .clone();
    assert!(companies.len() >= 2);
}

// ==================== Company Info ====================

#[tokio::test]
async fn s20_company_info_crud() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/company_info")
        .json(&json!({
            "company_name": "一人科技",
            "legal_representative": "张三",
            "unified_social_credit_code": "91440300MA1234567X",
            "taxpayer_type": "small_scale",
            "vat_rate": 3,
            "registered_capital": 100000.0
        }))
        .await;
    res.assert_status_ok();
    let info_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Update
    let put_res = app
        .put(&format!("/api/v1/company_info/{}", info_id))
        .json(&json!({ "taxpayer_type": "general", "vat_rate": 13 }))
        .await;
    put_res.assert_status_ok();

    // Verify
    let get_res = app.get(&format!("/api/v1/company_info/{}", info_id)).await;
    let info = get_res.json::<serde_json::Value>();
    assert_eq!(info["taxpayer_type"].as_str().unwrap(), "general");
}

// ==================== Social Insurance Config ====================

#[tokio::test]
async fn s20_social_insurance_config() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/social_insurance_config")
        .json(&json!({
            "city": "深圳",
            "social_security_base_min": 2360.0,
            "social_security_base_max": 33891.0,
            "pension_personal_rate": 0.08,
            "pension_company_rate": 0.14,
            "medical_personal_rate": 0.02,
            "medical_company_rate": 0.052,
            "housing_fund_personal_rate": 0.05,
            "housing_fund_company_rate": 0.05
        }))
        .await;
    res.assert_status_ok();
    let config_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Verify
    let get_res = app
        .get(&format!("/api/v1/social_insurance_config/{}", config_id))
        .await;
    let config = get_res.json::<serde_json::Value>();
    assert_eq!(config["city"].as_str().unwrap(), "深圳");
    assert_eq!(config["pension_personal_rate"].as_f64().unwrap(), 0.08);
}

// ==================== KPI Config → Records ====================

#[tokio::test]
async fn s20_kpi_config_and_records() {
    let app = common::TestApp::new().await;

    // Create KPI config
    let config_res = app
        .post("/api/v1/kpi_configs")
        .json(&json!({
            "name": "月收入",
            "category": "revenue",
            "unit": "CNY",
            "target_value": 100000.0,
            "warning_threshold": 50000.0,
            "danger_threshold": 20000.0,
            "is_active": 1
        }))
        .await;
    config_res.assert_status_ok();
    let kpi_id = config_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create KPI record
    let record_res = app
        .post("/api/v1/kpi_records")
        .json(&json!({
            "kpi_id": kpi_id,
            "value": 85000.0,
            "period": "2024-03",
            "notes": "三月收入"
        }))
        .await;
    record_res.assert_status_ok();

    // List records
    let list_res = app.get("/api/v1/kpi_records").await;
    list_res.assert_status_ok();
    let records = list_res.json::<serde_json::Value>();
    assert!(records["count"].as_i64().unwrap() >= 1);
}

// ==================== Branches ====================

#[tokio::test]
async fn s20_branch_management() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/branches")
        .json(&json!({
            "branch_name": "上海分公司",
            "branch_type": "subsidiary",
            "branch_address": "上海市浦东新区",
            "contact_person": "李四",
            "employee_count": 5,
            "status": "active"
        }))
        .await;
    res.assert_status_ok();
    let branch_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Verify
    let get_res = app.get(&format!("/api/v1/branches/{}", branch_id)).await;
    let branch = get_res.json::<serde_json::Value>();
    assert_eq!(branch["branch_name"].as_str().unwrap(), "上海分公司");
}

// ==================== Milestones ====================

#[tokio::test]
async fn s20_milestone_crud() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/milestones")
        .json(&json!({
            "title": "产品上线",
            "milestone_type": "product_launch",
            "importance": "high",
            "milestone_date": "2024-06-01",
            "description": "V1.0正式发布"
        }))
        .await;
    res.assert_status_ok();
    let ms_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Delete
    let del_res = app.delete(&format!("/api/v1/milestones/{}", ms_id)).await;
    assert!(del_res.status_code().is_success());
}

// ==================== Domain Subscriptions ====================

#[tokio::test]
async fn s20_domain_subscription() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/domain_subscriptions")
        .json(&json!({
            "domain_name": "opc.test",
            "registrar": "Cloudflare",
            "registration_date": "2024-01-01",
            "expiry_date": "2025-01-01",
            "annual_fee": 98.0,
            "auto_renewal": 1,
            "status": "active"
        }))
        .await;
    res.assert_status_ok();
    let dom_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Verify
    let get_res = app
        .get(&format!("/api/v1/domain_subscriptions/{}", dom_id))
        .await;
    let dom = get_res.json::<serde_json::Value>();
    assert_eq!(dom["domain_name"].as_str().unwrap(), "opc.test");
    assert_eq!(dom["registrar"].as_str().unwrap(), "Cloudflare");
}

// ==================== Software Subscriptions ====================

#[tokio::test]
async fn s20_software_subscription() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/software_subscriptions")
        .json(&json!({
            "software_name": "GitHub Team",
            "vendor": "GitHub Inc.",
            "subscription_type": "team",
            "plan_name": "Team",
            "license_count": 5,
            "total_price": 25.0,
            "billing_cycle": "monthly",
            "start_date": "2024-01-01",
            "next_billing_date": "2024-04-01",
            "auto_renewal": 1,
            "status": "active"
        }))
        .await;
    res.assert_status_ok();
    let sw_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Update
    let put_res = app
        .put(&format!("/api/v1/software_subscriptions/{}", sw_id))
        .json(&json!({ "license_count": 10, "total_price": 50.0 }))
        .await;
    put_res.assert_status_ok();
}
