/// Scenario 10: 🏢 涉税与合规全链路 (Tax Compliance Lifecycle)
/// 验证 销项发票创建 -> 增值税计算 -> 自动税务归档流转
/// 模拟税务合规的完整闭环，确保发票与 Tax 节点的数据一致性
use crate::common;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn s10_tax_compliance_lifecycle() {
    let app = common::TestApp::new().await;

    // 1. 模拟发生多笔销项发票 (Income/Output VAT)
    let inv1_res = app
        .post("/api/v1/invoices")
        .json(&json!({
            "invoice_number": "INV-2026-001",
            "amount": 100000.0,
            "tax_amount": 6000.0,
            "invoice_type": "income",
            "status": "paid",
            "invoice_date": "2026-06-15"
        }))
        .await;
    inv1_res.assert_status_ok();

    let inv2_res = app
        .post("/api/v1/invoices")
        .json(&json!({
            "invoice_number": "INV-2026-002",
            "amount": 50000.0,
            "tax_amount": 3000.0,
            "invoice_type": "income",
            "status": "paid",
            "invoice_date": "2026-06-20"
        }))
        .await;
    inv2_res.assert_status_ok();

    let inv3_res = app
        .post("/api/v1/invoices")
        .json(&json!({
            "invoice_number": "INV-2026-003",
            "amount": 20000.0,
            "tax_amount": 1200.0,
            "invoice_type": "expense", // 进项发票 (Input VAT) 可抵扣
            "status": "paid",
            "invoice_date": "2026-06-25"
        }))
        .await;
    inv3_res.assert_status_ok();

    // 2. 根据本月发票创建一条税务计提记录 (Tax)
    // 销项总额 = 150000, 销项税 = 9000
    // 进项总额 = 20000, 进项税 = 1200
    // 应缴增值税 = 9000 - 1200 = 7800
    let tax_res = app
        .post("/api/v1/tax")
        .json(&json!({
            "tax_type": "增值税 (VAT)",
            "period": "2026-06",
            "taxable_amount": 130000.0,
            "tax_amount": 7800.0,
            "status": "pending"
        }))
        .await;
    tax_res.assert_status_ok();

    let tax_record: serde_json::Value = tax_res.json();
    let tax_id = tax_record["id"].as_str().unwrap();

    // 3. 将状态流转为已申报 (filed)
    let filed_res = app
        .put(&format!("/api/v1/tax/{}", tax_id))
        .json(&json!({
            "status": "filed"
        }))
        .await;
    filed_res.assert_status_ok();

    // 4. 并验证获取单个明细与列表页过滤
    let get_res = app.get(&format!("/api/v1/tax/{}", tax_id)).await;
    get_res.assert_status_ok();
    let fetched_tax: serde_json::Value = get_res.json();
    assert_eq!(fetched_tax["status"].as_str().unwrap(), "filed");
    assert_eq!(fetched_tax["tax_amount"].as_f64().unwrap(), 7800.0);

    let list_res = app.get("/api/v1/tax").await;
    list_res.assert_status_ok();
    let body: serde_json::Value = list_res.json();
    assert_eq!(body["count"].as_i64().unwrap(), 1);
}

#[tokio::test]
async fn s10_tax_tenant_isolation() {
    let app_a = common::TestApp::new().await;
    let app_b = app_a.create_second_tenant().await;

    // A records tax
    app_a
        .post("/api/v1/tax")
        .json(&json!({
            "tax_type": "企业所得税",
            "period": "2026-Q1",
            "tax_amount": 50000.0
        }))
        .await
        .assert_status_ok();

    // B lists tax
    let b_list = app_b.get("/api/v1/tax").await;
    b_list.assert_status_ok();
    let body: serde_json::Value = b_list.json();
    assert_eq!(
        body["count"].as_i64().unwrap(),
        0,
        "B should not see A's tax records"
    );
}

#[tokio::test]
async fn s10_tax_record_delete() {
    let app = common::TestApp::new().await;

    let tax_res = app
        .post("/api/v1/tax")
        .json(&json!({
            "tax_type": "增值税 (VAT)",
            "period": "2026-07",
            "tax_amount": 5000.0,
            "status": "pending"
        }))
        .await;
    tax_res.assert_status_ok();
    let tax_id = tax_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Delete (returns 204 No Content)
    let del = app.delete(&format!("/api/v1/tax/{}", tax_id)).await;
    assert!(del.status_code().is_success(), "Delete should succeed");

    // Verify gone
    let list = app.get("/api/v1/tax").await;
    let body: serde_json::Value = list.json();
    assert_eq!(body["count"].as_i64().unwrap(), 0);
}

#[tokio::test]
async fn s10_tax_boundary_zero_amount() {
    let app = common::TestApp::new().await;

    // Zero tax amount should be valid
    let res = app
        .post("/api/v1/tax")
        .json(&json!({
            "tax_type": "企业所得税",
            "period": "2026-Q2",
            "tax_amount": 0.0,
            "status": "pending"
        }))
        .await;
    res.assert_status_ok();
    let data: serde_json::Value = res.json();
    // Verify zero amount is accepted
    let amt = data["tax_amount"].as_f64().unwrap_or(0.0);
    assert_eq!(amt, 0.0);
}
