/// Scenario 29: 🛡️ Data Integrity (数据完整性)
/// Tests cascading delete protection, quote→invoice conversion, auto audit logs, FK validation
use crate::common;
use serde_json::json;

// ==================== Cascading Delete Protection ====================

#[tokio::test]
async fn s29_client_delete_blocked_by_contacts() {
    let app = common::TestApp::new().await;

    let client_res = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "测试客户公司", "industry": "technology", "client_type": "active" }))
        .await;
    client_res.assert_status_ok();
    let client_id = client_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let contact_res = app
        .post("/api/v1/contacts")
        .json(&json!({ "client_id": &client_id, "name": "张三", "email": "zhang@test-com.example.invalid" }))
        .await;
    contact_res.assert_status_ok();
    let contact_id = contact_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Delete client → BLOCKED
    assert_eq!(
        app.delete(&format!("/api/v1/clients/{}", client_id))
            .await
            .status_code(),
        409
    );

    // Delete contact first, then client succeeds
    assert_eq!(
        app.delete(&format!("/api/v1/contacts/{}", contact_id))
            .await
            .status_code(),
        204
    );
    assert_eq!(
        app.delete(&format!("/api/v1/clients/{}", client_id))
            .await
            .status_code(),
        204
    );
}

#[tokio::test]
async fn s29_client_delete_blocked_by_deals() {
    let app = common::TestApp::new().await;

    let client_res = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "客户A", "client_type": "active" }))
        .await;
    client_res.assert_status_ok();
    let client_id = client_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    app.post("/api/v1/deals")
        .json(&json!({ "client_id": &client_id, "title": "大单机会", "stage": "proposal", "amount": 500000 }))
        .await.assert_status_ok();

    assert_eq!(
        app.delete(&format!("/api/v1/clients/{}", client_id))
            .await
            .status_code(),
        409
    );
}

#[tokio::test]
async fn s29_client_delete_blocked_by_quotes() {
    let app = common::TestApp::new().await;

    let client_res = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "报价客户B", "client_type": "active" }))
        .await;
    client_res.assert_status_ok();
    let client_id = client_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    app.post("/api/v1/quotes")
        .json(
            &json!({ "client_id": &client_id, "quote_number": "QT-DEL-001", "title": "删除测试" }),
        )
        .await
        .assert_status_ok();

    assert_eq!(
        app.delete(&format!("/api/v1/clients/{}", client_id))
            .await
            .status_code(),
        409
    );
}

#[tokio::test]
async fn s29_quote_delete_blocked_by_items() {
    let app = common::TestApp::new().await;

    let client_res = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "报价客户", "client_type": "active" }))
        .await;
    client_res.assert_status_ok();
    let client_id = client_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let quote_res = app
        .post("/api/v1/quotes")
        .json(
            &json!({ "client_id": &client_id, "quote_number": "QT-2024-001", "title": "网站开发" }),
        )
        .await;
    quote_res.assert_status_ok();
    let quote_id = quote_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    app.post("/api/v1/quote_items")
        .json(&json!({ "quote_id": &quote_id, "description": "前端开发", "quantity": 1, "unit_price": 50000, "amount": 50000 }))
        .await.assert_status_ok();

    assert_eq!(
        app.delete(&format!("/api/v1/quotes/{}", quote_id))
            .await
            .status_code(),
        409
    );
}

#[tokio::test]
async fn s29_deal_delete_blocked_by_quotes() {
    let app = common::TestApp::new().await;

    let client_res = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "交易客户", "client_type": "active" }))
        .await;
    client_res.assert_status_ok();
    let client_id = client_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let deal_res = app
        .post("/api/v1/deals")
        .json(&json!({ "client_id": &client_id, "title": "ERP项目", "amount": 200000 }))
        .await;
    deal_res.assert_status_ok();
    let deal_id = deal_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    app.post("/api/v1/quotes")
        .json(&json!({ "client_id": &client_id, "deal_id": &deal_id, "quote_number": "QT-DEAL-001", "title": "项目报价" }))
        .await.assert_status_ok();

    assert_eq!(
        app.delete(&format!("/api/v1/deals/{}", deal_id))
            .await
            .status_code(),
        409
    );
}

// ==================== Quote → Invoice Conversion ====================

#[tokio::test]
async fn s29_quote_to_invoice_conversion() {
    let app = common::TestApp::new().await;

    let client_res = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "转化测试客户", "client_type": "active" }))
        .await;
    client_res.assert_status_ok();
    let client_id = client_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let quote_res = app
        .post("/api/v1/quotes")
        .json(&json!({
            "client_id": &client_id, "quote_number": "QT-CONV-001", "title": "咨询服务",
            "status": "accepted", "subtotal": 100000, "tax_amount": 6000, "total": 106000
        }))
        .await;
    quote_res.assert_status_ok();
    let quote_id = quote_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let conv = app
        .post(&format!("/api/v1/quotes/{}/convert-to-invoice", quote_id))
        .json(&json!({}))
        .await;
    conv.assert_status_ok();
    let body = conv.json::<serde_json::Value>();
    assert!(body["success"].as_bool().unwrap());
    let invoice_id = body["invoice_id"].as_str().unwrap();

    // Verify invoice
    let inv = app.get(&format!("/api/invoices/{}", invoice_id)).await;
    let inv_data = inv.json::<serde_json::Value>();
    assert_eq!(inv_data["amount"].as_f64().unwrap(), 106000.0);
    assert_eq!(inv_data["status"].as_str().unwrap(), "pending");
    assert!(
        inv_data["description"]
            .as_str()
            .unwrap()
            .contains("QT-CONV-001")
    );

    // Verify quote updated
    let q = app
        .get(&format!("/api/v1/quotes/{}", quote_id))
        .await
        .json::<serde_json::Value>();
    assert_eq!(q["status"].as_str().unwrap(), "invoiced");
    assert_eq!(q["invoice_id"].as_str().unwrap(), invoice_id);
}

#[tokio::test]
async fn s29_double_conversion_blocked() {
    let app = common::TestApp::new().await;

    let client_res = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "重复转化", "client_type": "active" }))
        .await;
    client_res.assert_status_ok();
    let client_id = client_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let quote_res = app.post("/api/v1/quotes")
        .json(&json!({ "client_id": &client_id, "quote_number": "QT-DBL-001", "title": "双重转化", "status": "accepted", "total": 50000 }))
        .await;
    quote_res.assert_status_ok();
    let quote_id = quote_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    app.post(&format!("/api/v1/quotes/{}/convert-to-invoice", quote_id))
        .json(&json!({}))
        .await
        .assert_status_ok();
    assert_eq!(
        app.post(&format!("/api/v1/quotes/{}/convert-to-invoice", quote_id))
            .json(&json!({}))
            .await
            .status_code(),
        409,
        "Double conversion should be blocked"
    );
}

#[tokio::test]
async fn s29_rejected_quote_cannot_convert() {
    let app = common::TestApp::new().await;

    let client_res = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "拒绝测试", "client_type": "active" }))
        .await;
    client_res.assert_status_ok();
    let client_id = client_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let quote_res = app.post("/api/v1/quotes")
        .json(&json!({ "client_id": &client_id, "quote_number": "QT-REJ-001", "title": "拒绝报价", "status": "rejected", "total": 10000 }))
        .await;
    quote_res.assert_status_ok();
    let quote_id = quote_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    assert_eq!(
        app.post(&format!("/api/v1/quotes/{}/convert-to-invoice", quote_id))
            .json(&json!({}))
            .await
            .status_code(),
        400
    );
}

// ==================== Auto Audit Log Verification ====================

#[tokio::test]
async fn s29_auto_audit_log_on_crud() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "审计测试", "client_type": "active" }))
        .await;
    res.assert_status_ok();
    let client_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    app.put(&format!("/api/v1/clients/{}", client_id))
        .json(&json!({ "client_type": "inactive" }))
        .await
        .assert_status_ok();

    let logs = app
        .get("/api/v1/operation_logs?limit=100")
        .await
        .json::<serde_json::Value>();
    let items = logs["data"].as_array().unwrap();

    assert!(
        items.iter().any(|l| l["action"].as_str() == Some("create")
            && l["entity_id"].as_str() == Some(&client_id)),
        "Should have create audit log"
    );
    assert!(
        items.iter().any(|l| l["action"].as_str() == Some("update")
            && l["entity_id"].as_str() == Some(&client_id)),
        "Should have update audit log"
    );

    app.delete(&format!("/api/v1/clients/{}", client_id)).await;
    let logs2 = app
        .get("/api/v1/operation_logs?limit=100")
        .await
        .json::<serde_json::Value>();
    assert!(
        logs2["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|l| l["action"].as_str() == Some("delete")
                && l["entity_id"].as_str() == Some(&client_id)),
        "Should have delete audit log"
    );
}

// ==================== FK Validation ====================

#[tokio::test]
async fn s29_fk_validation_on_new_tables() {
    let app = common::TestApp::new().await;

    assert_eq!(app.post("/api/v1/quotes")
        .json(&json!({ "client_id": "non-existent", "quote_number": "QT-FK-001", "title": "FK测试" }))
        .await.status_code(), 400, "Should reject non-existent client_id FK");

    assert_eq!(
        app.post("/api/v1/contacts")
            .json(&json!({ "client_id": "bogus-id", "name": "FK测试联系人" }))
            .await
            .status_code(),
        400,
        "Should reject non-existent client_id FK for contacts"
    );
}
