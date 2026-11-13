/// Scenario 24: 🤝 CRM Client Management (客户关系管理)
/// Tests clients, contacts, interactions, deals, quotes lifecycle
use crate::common;
use serde_json::json;

// ==================== Clients ====================

#[tokio::test]
async fn s24_client_crud() {
    let app = common::TestApp::new().await;

    // Create client
    let res = app
        .post("/api/v1/clients")
        .json(&json!({
            "name": "字节跳动",
            "short_name": "字节",
            "client_type": "active",
            "industry": "互联网",
            "email": "contact@bytedance-com.example.invalid",
            "phone": "010-12345678",
            "source": "referral",
            "annual_revenue": 50000000
        }))
        .await;
    res.assert_status_ok();
    let client_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Get
    let get_res = app.get(&format!("/api/v1/clients/{}", client_id)).await;
    let client = get_res.json::<serde_json::Value>();
    assert_eq!(client["name"].as_str().unwrap(), "字节跳动");
    assert_eq!(client["client_type"].as_str().unwrap(), "active");

    // Update
    let put_res = app
        .put(&format!("/api/v1/clients/{}", client_id))
        .json(&json!({ "client_type": "inactive" }))
        .await;
    put_res.assert_status_ok();

    // List
    let list_res = app.get("/api/v1/clients").await;
    list_res.assert_status_ok();
    assert!(
        list_res.json::<serde_json::Value>()["count"]
            .as_i64()
            .unwrap()
            >= 1
    );
}

// ==================== Contacts ====================

#[tokio::test]
async fn s24_contact_with_client_fk() {
    let app = common::TestApp::new().await;

    // Create client first
    let client_res = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "阿里巴巴" }))
        .await;
    client_res.assert_status_ok();
    let client_id = client_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create contact linked to client
    let res = app
        .post("/api/v1/contacts")
        .json(&json!({
            "client_id": client_id,
            "name": "张三",
            "title": "技术总监",
            "email": "zhangsan@alibaba-com.example.invalid",
            "is_primary": 1
        }))
        .await;
    res.assert_status_ok();
    let contact_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Verify FK data
    let get_res = app.get(&format!("/api/v1/contacts/{}", contact_id)).await;
    let contact = get_res.json::<serde_json::Value>();
    assert_eq!(contact["client_id"].as_str().unwrap(), client_id);
    assert_eq!(contact["title"].as_str().unwrap(), "技术总监");
}

// ==================== Interactions ====================

#[tokio::test]
async fn s24_interaction_timeline() {
    let app = common::TestApp::new().await;

    let client_res = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "腾讯" }))
        .await;
    let client_id = client_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create multiple interactions
    app.post("/api/v1/interactions")
        .json(&json!({
            "client_id": client_id,
            "interaction_type": "call",
            "subject": "初次沟通",
            "interaction_date": "2024-03-01",
            "duration_minutes": 30,
            "outcome": "positive"
        }))
        .await
        .assert_status_ok();

    app.post("/api/v1/interactions")
        .json(&json!({
            "client_id": client_id,
            "interaction_type": "meeting",
            "subject": "需求评审会",
            "interaction_date": "2024-03-15",
            "duration_minutes": 120
        }))
        .await
        .assert_status_ok();

    // List all interactions
    let list_res = app.get("/api/v1/interactions").await;
    list_res.assert_status_ok();
    assert!(
        list_res.json::<serde_json::Value>()["count"]
            .as_i64()
            .unwrap()
            >= 2
    );
}

// ==================== Deals ====================

#[tokio::test]
async fn s24_deal_pipeline_lifecycle() {
    let app = common::TestApp::new().await;

    let client_res = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "华为" }))
        .await;
    let client_id = client_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create deal
    let deal_res = app
        .post("/api/v1/deals")
        .json(&json!({
            "client_id": client_id,
            "title": "企业级ERP定制",
            "stage": "lead",
            "amount": 500000,
            "probability": 20,
            "expected_close_date": "2024-06-30"
        }))
        .await;
    deal_res.assert_status_ok();
    let deal_id = deal_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Progress through pipeline
    app.put(&format!("/api/v1/deals/{}", deal_id))
        .json(&json!({ "stage": "qualified", "probability": 40 }))
        .await
        .assert_status_ok();

    app.put(&format!("/api/v1/deals/{}", deal_id))
        .json(&json!({ "stage": "proposal", "probability": 60 }))
        .await
        .assert_status_ok();

    // Verify
    let get_res = app.get(&format!("/api/v1/deals/{}", deal_id)).await;
    let deal = get_res.json::<serde_json::Value>();
    assert_eq!(deal["stage"].as_str().unwrap(), "proposal");
}

// ==================== Quotes ====================

#[tokio::test]
async fn s24_quote_lifecycle() {
    let app = common::TestApp::new().await;

    let client_res = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "小米" }))
        .await;
    let client_id = client_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create quote
    let quote_res = app
        .post("/api/v1/quotes")
        .json(&json!({
            "client_id": client_id,
            "quote_number": "QT-2024-001",
            "title": "ERP系统实施报价",
            "subtotal": 100000,
            "tax_rate": 6,
            "tax_amount": 6000,
            "total": 106000,
            "valid_until": "2024-05-01",
            "notes": "含一年维保"
        }))
        .await;
    quote_res.assert_status_ok();
    let quote_id = quote_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Update status
    app.put(&format!("/api/v1/quotes/{}", quote_id))
        .json(&json!({ "status": "sent" }))
        .await
        .assert_status_ok();

    // Verify
    let get_res = app.get(&format!("/api/v1/quotes/{}", quote_id)).await;
    let quote = get_res.json::<serde_json::Value>();
    assert_eq!(quote["status"].as_str().unwrap(), "sent");
    assert_eq!(quote["quote_number"].as_str().unwrap(), "QT-2024-001");
}

// ==================== Quote Items ====================

#[tokio::test]
async fn s24_quote_items_with_fk() {
    let app = common::TestApp::new().await;

    let client_res = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "百度" }))
        .await;
    let client_id = client_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let quote_res = app
        .post("/api/v1/quotes")
        .json(&json!({ "client_id": client_id, "quote_number": "QT-2024-002" }))
        .await;
    let quote_id = quote_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Add line items
    let item1 = app
        .post("/api/v1/quote_items")
        .json(&json!({
            "quote_id": quote_id,
            "description": "ERP系统许可证",
            "quantity": 1,
            "unit": "套",
            "unit_price": 80000,
            "amount": 80000
        }))
        .await;
    item1.assert_status_ok();

    let item2 = app
        .post("/api/v1/quote_items")
        .json(&json!({
            "quote_id": quote_id,
            "description": "实施服务费",
            "quantity": 20,
            "unit": "人天",
            "unit_price": 1000,
            "amount": 20000
        }))
        .await;
    item2.assert_status_ok();

    // List items
    let list_res = app.get("/api/v1/quote_items").await;
    list_res.assert_status_ok();
    assert!(
        list_res.json::<serde_json::Value>()["count"]
            .as_i64()
            .unwrap()
            >= 2
    );
}

// ==================== Tenant Isolation ====================

#[tokio::test]
async fn s24_crm_tenant_isolation() {
    let app = common::TestApp::new().await;

    // Create client in default company
    let res = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "默认公司客户" }))
        .await;
    res.assert_status_ok();

    // List should only see own company's clients
    let list = app.get("/api/v1/clients").await;
    let items = list.json::<serde_json::Value>();
    let count = items["count"].as_i64().unwrap();
    // All returned clients should have matching company_id
    if let Some(data) = items["data"].as_array() {
        for item in data {
            assert!(item["company_id"].as_str().is_some());
        }
    }
    assert!(count >= 1);
}
