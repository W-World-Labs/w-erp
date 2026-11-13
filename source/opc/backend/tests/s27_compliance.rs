/// Scenario 27: 🛡️ Compliance: Insurance & Pension (保险/年金合规)
/// Tests insurance policies, pension plans, multi-country support, CRUD lifecycle
use crate::common;
use serde_json::json;

// ==================== Insurance Policies ====================

#[tokio::test]
async fn s27_insurance_policy_crud() {
    let app = common::TestApp::new().await;

    // Create
    let res = app
        .post("/api/v1/insurance_policies")
        .json(&json!({
            "policy_name": "企业财产综合险",
            "insurance_type": "property",
            "provider": "中国平安",
            "premium_amount": 12000,
            "premium_frequency": "yearly",
            "coverage_amount": 5000000,
            "deductible": 5000,
            "start_date": "2024-01-01",
            "end_date": "2024-12-31",
            "renewal_date": "2024-12-01",
            "country": "CN",
            "status": "active"
        }))
        .await;
    res.assert_status_ok();
    let policy_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Get
    let get_res = app
        .get(&format!("/api/v1/insurance_policies/{}", policy_id))
        .await;
    let policy = get_res.json::<serde_json::Value>();
    assert_eq!(policy["policy_name"].as_str().unwrap(), "企业财产综合险");
    assert_eq!(policy["insurance_type"].as_str().unwrap(), "property");
    assert_eq!(policy["country"].as_str().unwrap(), "CN");

    // Update
    app.put(&format!("/api/v1/insurance_policies/{}", policy_id))
        .json(&json!({ "status": "expired" }))
        .await
        .assert_status_ok();

    let updated = app
        .get(&format!("/api/v1/insurance_policies/{}", policy_id))
        .await;
    assert_eq!(
        updated.json::<serde_json::Value>()["status"]
            .as_str()
            .unwrap(),
        "expired"
    );

    // List
    let list = app.get("/api/v1/insurance_policies").await;
    list.assert_status_ok();
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn s27_all_insurance_types() {
    let app = common::TestApp::new().await;

    let types = vec![
        ("health", "员工医疗险"),
        ("property", "办公室财产险"),
        ("liability", "第三方责任险"),
        ("workers_comp", "工伤保险"),
        ("professional", "职业责任险"),
        ("cyber", "网络安全险"),
        ("vehicle", "公司车辆险"),
    ];

    for (ins_type, name) in &types {
        let res = app
            .post("/api/v1/insurance_policies")
            .json(&json!({
                "policy_name": name,
                "insurance_type": ins_type,
                "start_date": "2024-01-01",
                "end_date": "2025-01-01"
            }))
            .await;
        res.assert_status_ok();
    }

    let list = app.get("/api/v1/insurance_policies").await;
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 7);
}

// ==================== Pension Plans ====================

#[tokio::test]
async fn s27_pension_plan_crud() {
    let app = common::TestApp::new().await;

    // Create Korean national pension
    let res = app
        .post("/api/v1/pension_plans")
        .json(&json!({
            "plan_name": "국민연금 (国民年金)",
            "plan_type": "national_pension",
            "country": "KR",
            "provider": "국민연금공단",
            "contribution_employer": 225000,
            "contribution_employee": 225000,
            "contribution_frequency": "monthly",
            "status": "active"
        }))
        .await;
    res.assert_status_ok();
    let plan_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Get
    let get_res = app.get(&format!("/api/v1/pension_plans/{}", plan_id)).await;
    let plan = get_res.json::<serde_json::Value>();
    assert_eq!(plan["plan_type"].as_str().unwrap(), "national_pension");
    assert_eq!(plan["country"].as_str().unwrap(), "KR");

    // Update
    app.put(&format!("/api/v1/pension_plans/{}", plan_id))
        .json(&json!({ "current_value": 15000000 }))
        .await
        .assert_status_ok();

    // List
    let list = app.get("/api/v1/pension_plans").await;
    list.assert_status_ok();
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn s27_multi_country_pension_plans() {
    let app = common::TestApp::new().await;

    // Korean plans
    app.post("/api/v1/pension_plans")
        .json(&json!({
            "plan_name": "건강보험 (健康保险)",
            "plan_type": "health_insurance",
            "country": "KR",
            "contribution_employer": 180000,
            "contribution_employee": 180000
        }))
        .await
        .assert_status_ok();

    app.post("/api/v1/pension_plans")
        .json(&json!({
            "plan_name": "고용보험 (雇佣保险)",
            "plan_type": "employment_insurance",
            "country": "KR",
            "contribution_employer": 50000,
            "contribution_employee": 45000
        }))
        .await
        .assert_status_ok();

    // US 401(k)
    app.post("/api/v1/pension_plans")
        .json(&json!({
            "plan_name": "Company 401(k) Plan",
            "plan_type": "401k",
            "country": "US",
            "provider": "Fidelity",
            "contribution_employer": 500,
            "contribution_employee": 1000,
            "vesting_period_months": 36,
            "current_value": 50000
        }))
        .await
        .assert_status_ok();

    // US IRA
    app.post("/api/v1/pension_plans")
        .json(&json!({
            "plan_name": "Roth IRA",
            "plan_type": "ira",
            "country": "US",
            "provider": "Vanguard",
            "contribution_employee": 583,
            "current_value": 25000
        }))
        .await
        .assert_status_ok();

    // Chinese social security
    app.post("/api/v1/pension_plans")
        .json(&json!({
            "plan_name": "城镇职工基本养老保险",
            "plan_type": "national_pension",
            "country": "CN",
            "contribution_employer": 2400,
            "contribution_employee": 1200
        }))
        .await
        .assert_status_ok();

    let list = app.get("/api/v1/pension_plans").await;
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 5);
}

#[tokio::test]
async fn s27_tenant_isolation() {
    let app = common::TestApp::new().await;

    // Create one of each
    app.post("/api/v1/insurance_policies")
        .json(&json!({
            "policy_name": "隔离测试保单",
            "insurance_type": "health",
            "start_date": "2024-01-01",
            "end_date": "2025-01-01"
        }))
        .await
        .assert_status_ok();

    app.post("/api/v1/pension_plans")
        .json(&json!({
            "plan_name": "隔离测试年金",
            "plan_type": "private_pension"
        }))
        .await
        .assert_status_ok();

    // Verify company_id
    let ins_list = app.get("/api/v1/insurance_policies").await;
    if let Some(items) = ins_list.json::<serde_json::Value>()["data"].as_array() {
        for item in items {
            assert!(item["company_id"].as_str().is_some());
        }
    }

    let pen_list = app.get("/api/v1/pension_plans").await;
    if let Some(items) = pen_list.json::<serde_json::Value>()["data"].as_array() {
        for item in items {
            assert!(item["company_id"].as_str().is_some());
        }
    }
}
