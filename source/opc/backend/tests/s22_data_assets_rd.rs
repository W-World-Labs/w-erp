/// Scenario 22: 🔬 Data Assets, R&D Costs & Asset Records (数据资产/研发费用/资产变动)
/// Tests cross-table references and business logic
use crate::common;
use serde_json::json;

// ==================== Data Asset Meta ====================

#[tokio::test]
async fn s22_data_asset_meta_crud() {
    let app = common::TestApp::new().await;

    // Create a parent asset first
    let asset_res = app
        .post("/api/assets")
        .json(&json!({
            "name": "OPC 系统",
            "asset_type": "data",
            "status": "active",
            "purchase_price": 0
        }))
        .await;
    asset_res.assert_status_ok();
    let asset_id = asset_res.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create data asset meta
    let meta_res = app
        .post("/api/v1/data_asset_meta")
        .json(&json!({
            "asset_id": asset_id,
            "data_type": "software",
            "tech_stack": "Rust/React/SQLite",
            "version": "1.0.0",
            "license_type": "自研"
        }))
        .await;
    meta_res.assert_status_ok();
    let meta_id = meta_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Verify
    let get_res = app
        .get(&format!("/api/v1/data_asset_meta/{}", meta_id))
        .await;
    let meta = get_res.json::<serde_json::Value>();
    assert_eq!(meta["data_type"].as_str().unwrap(), "software");
    assert_eq!(meta["tech_stack"].as_str().unwrap(), "Rust/React/SQLite");
}

// ==================== Asset Records ====================

#[tokio::test]
async fn s22_asset_record_lifecycle() {
    let app = common::TestApp::new().await;

    // Create parent asset
    let asset_res = app
        .post("/api/assets")
        .json(&json!({
            "name": "MacBook Pro",
            "asset_type": "physical",
            "purchase_price": 15000.0,
            "current_value": 15000.0
        }))
        .await;
    asset_res.assert_status_ok();
    let asset_id = asset_res.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create depreciation record
    let rec_res = app
        .post("/api/v1/asset_records")
        .json(&json!({
            "asset_id": asset_id,
            "record_type": "depreciation",
            "description": "2024年3月折旧 - 3月折旧",
            "amount": 625.0,
            "record_date": "2024-03-31"
        }))
        .await;
    rec_res.assert_status_ok();

    // Create transfer record
    let transfer_res = app
        .post("/api/v1/asset_records")
        .json(&json!({
            "asset_id": asset_id,
            "record_type": "transfer",
            "description": "设备转移 - 转移至上海分部",
            "record_date": "2024-04-01"
        }))
        .await;
    transfer_res.assert_status_ok();

    // List records
    let list_res = app.get("/api/v1/asset_records").await;
    list_res.assert_status_ok();
    let list = list_res.json::<serde_json::Value>();
    assert!(list["count"].as_i64().unwrap() >= 2);
}

// ==================== R&D Costs ====================

#[tokio::test]
async fn s22_rd_cost_with_references() {
    let app = common::TestApp::new().await;

    // Create project first
    let proj_res = app
        .post("/api/v1/projects")
        .json(&json!({
            "name": "AI 审查引擎",
            "project_type": "rd",
            "status": "active"
        }))
        .await;
    proj_res.assert_status_ok();
    let project_id = proj_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create employee
    let emp_res = app
        .post("/api/v1/employees")
        .json(&json!({
            "name": "王工",
            "department": "研发部",
            "position": "高级工程师",
            "base_salary": 25000
        }))
        .await;
    emp_res.assert_status_ok();
    let employee_id = emp_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create R&D cost with cross-references
    let rd_res = app
        .post("/api/v1/rd_costs")
        .json(&json!({
            "project_id": project_id,
            "cost_type": "personnel",
            "amount": 25000.0,
            "description": "王工3月研发工资",
            "employee_id": employee_id,
            "cost_date": "2024-03-31"
        }))
        .await;
    rd_res.assert_status_ok();
    let rd_id = rd_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Verify cross-references
    let get_res = app.get(&format!("/api/v1/rd_costs/{}", rd_id)).await;
    let rd = get_res.json::<serde_json::Value>();
    assert_eq!(rd["project_id"].as_str().unwrap(), project_id);
    assert_eq!(rd["employee_id"].as_str().unwrap(), employee_id);
    assert_eq!(rd["cost_type"].as_str().unwrap(), "personnel");
}

#[tokio::test]
async fn s22_rd_cost_invalid_project_rejected() {
    let app = common::TestApp::new().await;

    // Try to create R&D cost with non-existent project
    let res = app
        .post("/api/v1/rd_costs")
        .json(&json!({
            "project_id": "non-existent-project",
            "cost_type": "material",
            "amount": 5000.0
        }))
        .await;
    // Should fail FK validation
    assert_eq!(res.status_code(), 400);
}

// ==================== Tax Declarations ====================

#[tokio::test]
async fn s22_tax_declaration_crud() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/tax_declarations")
        .json(&json!({
            "tax_type": "增值税",
            "period": "2024-Q1",
            "declaration_type": "quarterly",
            "period_start": "2024-01-01",
            "period_end": "2024-03-31",
            "taxable_amount": 150000.0,
            "tax_amount": 4500.0,
            "status": "pending",
            "declaration_deadline": "2024-04-15"
        }))
        .await;
    res.assert_status_ok();
    let tax_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Declare it
    let put_res = app
        .put(&format!("/api/v1/tax_declarations/{}", tax_id))
        .json(&json!({ "status": "declared", "declaration_date": "2024-04-10" }))
        .await;
    put_res.assert_status_ok();
}
