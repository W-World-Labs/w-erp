/// Scenario 3: 员工入职到发薪
/// 录入员工 → 设置薪资 → 创建月度工资单
///  → 验证薪资非负 → 验证员工删除时关联检查
use crate::common;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn s03_employee_full_lifecycle() {
    let app = common::TestApp::new().await;

    // Step 1: Create employee
    let create_res = app
        .post("/api/v1/employees")
        .json(&json!({
            "name": "Zhang Wei",
            "position": "Senior Engineer",
            "department": "Engineering",
            "base_salary": 25000.0,
            "hire_date": "2026-01-15",
            "status": "active"
        }))
        .await;
    create_res.assert_status_ok();
    let emp: serde_json::Value = create_res.json();
    let emp_id = emp["id"].as_str().expect("should have id");

    // Step 2: Get by ID
    let get_res = app.get(&format!("/api/v1/employees/{}", emp_id)).await;
    get_res.assert_status_ok();
    let fetched: serde_json::Value = get_res.json();
    assert_eq!(fetched["name"], "Zhang Wei");
    assert_eq!(fetched["base_salary"], 25000.0);

    // Step 3: Update (include required fields for typed extractor)
    let update_res = app
        .put(&format!("/api/v1/employees/{}", emp_id))
        .json(&json!({"name": "Zhang Wei", "department": "Management", "position": "CTO"}))
        .await;
    update_res.assert_status_ok();

    // Step 4: Delete
    let del_res = app.delete(&format!("/api/v1/employees/{}", emp_id)).await;
    del_res.assert_status(StatusCode::NO_CONTENT);

    // Step 5: 404
    let gone_res = app.get(&format!("/api/v1/employees/{}", emp_id)).await;
    gone_res.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn s03_salary_negative_validation() {
    let app = common::TestApp::new().await;

    // Negative salary should be rejected
    let res = app
        .post("/api/v1/employees")
        .json(&json!({
            "name": "Bad Hire",
            "base_salary": -1000.0,
            "position": "None"
        }))
        .await;
    res.assert_status(StatusCode::BAD_REQUEST);
    let body: serde_json::Value = res.json();
    assert!(
        body["error"].as_str().unwrap().contains("non-negative"),
        "error should mention non-negative"
    );
}

#[tokio::test]
async fn s03_employee_negative_salary_update_rejected() {
    let app = common::TestApp::new().await;

    let create_res = app
        .post("/api/v1/employees")
        .json(&json!({
            "name": "Raise Guard",
            "position": "Engineer",
            "base_salary": 18000.0
        }))
        .await;
    create_res.assert_status_ok();
    let emp_id = create_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let update_res = app
        .put(&format!("/api/v1/employees/{}", emp_id))
        .json(&json!({ "base_salary": -10.0 }))
        .await;
    update_res.assert_status(StatusCode::BAD_REQUEST);

    let body: serde_json::Value = update_res.json();
    assert!(
        body["error"]
            .as_str()
            .unwrap_or("")
            .contains("non-negative")
    );

    let fetched = app
        .get(&format!("/api/v1/employees/{}", emp_id))
        .await
        .json::<serde_json::Value>();
    assert_eq!(fetched["base_salary"], 18000.0);
}

#[tokio::test]
async fn s03_employee_salary_record_flow() {
    let app = common::TestApp::new().await;

    // Step 1: Create employee via seed
    let emp_id =
        common::seed::create_employee(&app.pool, &app.company_id, "Li Ming", 20000.0).await;

    // Step 2: Create salary record for this employee
    let salary_res = app
        .post("/api/v1/salary")
        .json(&json!({
            "employee_id": emp_id,
            "period": "2026-01",
            "base_salary": 20000.0,
            "bonus": 5000.0,
            "deductions": 2000.0,
            "net_salary": 23000.0
        }))
        .await;
    salary_res.assert_status_ok();

    // Step 3: Verify salary shows in list
    let list_res = app.get("/api/v1/salary").await;
    list_res.assert_status_ok();
    let body: serde_json::Value = list_res.json();
    assert_eq!(body["count"], 1);
}

#[tokio::test]
async fn s03_salary_payment_marks_salary_paid_and_updates_reports() {
    let app = common::TestApp::new().await;

    let emp_res = app
        .post("/api/v1/employees")
        .json(&json!({
            "name": "Payroll Flow",
            "position": "Finance",
            "base_salary": 16000.0
        }))
        .await;
    emp_res.assert_status_ok();
    let emp_id = emp_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let salary_res = app
        .post("/api/v1/salary")
        .json(&json!({
            "employee_id": &emp_id,
            "period": "2026-02",
            "base_salary": 15000.0,
            "bonus": 2000.0,
            "deductions": 1000.0,
            "net_salary": 16000.0,
            "status": "unpaid"
        }))
        .await;
    salary_res.assert_status_ok();
    let salary_id = salary_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let pay_res = app
        .post("/api/v1/payments")
        .json(&json!({ "status": "completed",
            "salary_record_id": &salary_id,
            "amount": 16000.0,
            "payment_type": "salary",
            "payment_date": "2026-02-28",
            "payment_method": "bank_transfer"
        }))
        .await;
    pay_res.assert_status_ok();
    let payment_id = pay_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let salary = app
        .get(&format!("/api/v1/salary/{}", salary_id))
        .await
        .json::<serde_json::Value>();
    assert_eq!(salary["status"], "paid");
    assert_eq!(salary["payment_id"], payment_id);

    let report = app
        .get("/api/v1/reports/financial-health")
        .await
        .json::<serde_json::Value>();
    assert_eq!(report["raw"]["total_liabilities"], 0.0);
    assert_eq!(report["metrics"]["operating_cash_flow"], -16000.0);
}

#[tokio::test]
async fn s03_employee_pagination() {
    let app = common::TestApp::new().await;

    // Create 3 employees
    for i in 1..=3 {
        app.post("/api/v1/employees")
            .json(&json!({
                "name": format!("Employee #{}", i),
                "position": "Staff",
                "base_salary": 10000.0 * (i as f64)
            }))
            .await
            .assert_status_ok();
    }

    // Paginate
    let res = app.get("/api/v1/employees?page=1&page_size=2").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["count"], 3, "total should be 3");
    assert_eq!(
        body["data"].as_array().unwrap().len(),
        2,
        "page should have 2"
    );
}

#[tokio::test]
async fn s03_employee_tenant_isolation() {
    let app_a = common::TestApp::new().await;
    let app_b = app_a.create_second_tenant().await;

    // A creates employee
    app_a
        .post("/api/v1/employees")
        .json(&json!({
            "name": "A's Employee",
            "position": "Staff",
            "base_salary": 15000.0
        }))
        .await
        .assert_status_ok();

    // B sees 0
    let res = app_b.get("/api/v1/employees").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["count"], 0, "tenant B should see 0 employees");
}
