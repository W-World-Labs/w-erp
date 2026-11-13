/// Scenario 12: 🤝 采购与履约闭环 (Procurement Cycle Lifecycle)
/// 验证 供应商入库 -> 电子合同签订 -> 请款/付款流转 -> 最终归档闭环
use crate::common;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn s12_procurement_lifecycle() {
    let app = common::TestApp::new().await;

    // 1. 供应商侧入库 (Vendor)
    let vendor_res = app
        .post("/api/v1/vendors")
        .json(&json!({
            "name": "Design Studio LLC",
            "vendor_type": "service",
            "contact_person": "Jane Doe",
            "contact_phone": "+1-555-0192",
            "status": "active"
        }))
        .await;
    vendor_res.assert_status_ok();
    let vendor_record: serde_json::Value = vendor_res.json();
    let _vendor_id = vendor_record["id"].as_str().unwrap(); // Keep if needed for relations

    // 2. 与该供应商签订设计服务合同 (Contracts)
    let contract_res = app
        .post("/api/v1/contracts")
        .json(&json!({
            "contract_name": "Q3 2026 UI/UX Design Refresh",
            "contract_type": "service",
            "party_a": "OPC DevRoot",
            "party_b": "Design Studio LLC",
            "counterparty": "Design Studio LLC",
            "amount": 25000.0,
            "start_date": "2026-07-01",
            "end_date": "2026-09-30",
            "status": "active"
        }))
        .await;
    contract_res.assert_status_ok();
    let contract_record: serde_json::Value = contract_res.json();
    let contract_id = contract_record["id"].as_str().unwrap();

    // 3. 为该合同创建首付款 (Payments)
    let payment_res = app
        .post("/api/v1/payments")
        .json(&json!({
            "payment_type": "首期预付款 (Advancement)",
            "amount": 10000.0,
            "payment_date": "2026-07-05",
            "payer": "OPC DevRoot",
            "payee": "Design Studio LLC",
            "status": "pending",
            "category": "研发外包"
        }))
        .await;
    payment_res.assert_status_ok();
    let payment_id = payment_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 4. 将付款完成，代表真实发生转账
    app.put(&format!("/api/v1/payments/{}", payment_id))
        .json(&json!({
            "status": "completed",
            "description": "已汇款"
        }))
        .await
        .assert_status_ok();

    // 5. 合同履约完成，进行归档
    app.put(&format!("/api/v1/contracts/{}", contract_id))
        .json(&json!({
            "status": "completed"
        }))
        .await
        .assert_status_ok();

    let fetched_contract: serde_json::Value = app
        .get(&format!("/api/v1/contracts/{}", contract_id))
        .await
        .json();
    assert_eq!(fetched_contract["status"].as_str().unwrap(), "completed");
}

#[tokio::test]
async fn s12_procurement_tenant_isolation() {
    let app_a = common::TestApp::new().await;
    let app_b = app_a.create_second_tenant().await;

    // Tenant A creates Vendor
    app_a
        .post("/api/v1/vendors")
        .json(&json!({
            "name": "Acme Corp",
            "vendor_type": "supplier",
            "status": "active"
        }))
        .await
        .assert_status_ok();

    // Tenant B lists Vendors
    let b_list = app_b.get("/api/v1/vendors").await;
    b_list.assert_status_ok();
    assert_eq!(
        b_list.json::<serde_json::Value>()["count"]
            .as_i64()
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn s12_vendor_delete() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/vendors")
        .json(&json!({
            "name": "Delete Me LLC",
            "vendor_type": "supplier",
            "status": "active"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let del = app.delete(&format!("/api/v1/vendors/{}", id)).await;
    assert!(del.status_code().is_success());

    let list = app.get("/api/v1/vendors").await;
    assert_eq!(
        list.json::<serde_json::Value>()["count"].as_i64().unwrap(),
        0
    );
}

#[tokio::test]
async fn s12_payment_delete() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/payments")
        .json(&json!({
            "payment_type": "测试付款",
            "amount": 100.0,
            "payment_date": "2026-07-01",
            "payer": "Test",
            "payee": "Test Vendor",
            "status": "pending"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let del = app.delete(&format!("/api/v1/payments/{}", id)).await;
    assert!(del.status_code().is_success());
}
