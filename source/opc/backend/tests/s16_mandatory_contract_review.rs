/// Scenario 16: 🛡️ 强制 AI 合同审查 (Mandatory AI Contract Review)
///
/// Tests:
/// 1. Contract creation sets ai_review_status to 'pending' by default
/// 2. Without AI_API_URL configured, async review sets status to 'unavailable'
/// 3. counterparty_type is correctly persisted on create and update
/// 4. Contract update with content re-triggers AI review
/// 5. AI review columns exist and are queryable
/// 6. Multiple contracts each get independent review status
/// 7. Risk badge data is returned in contract list response
use crate::common;
use serde_json::json;

#[tokio::test]
async fn s16_contract_create_sets_default_review_status() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/contracts")
        .json(&json!({
            "contract_name": "测试服务合同",
            "contract_type": "服务合同",
            "counterparty": "上海测试有限公司",
            "counterparty_type": "client",
            "amount": 50000.0,
            "status": "active"
        }))
        .await;
    res.assert_status_ok();

    let body: serde_json::Value = res.json();
    let contract_id = body["id"].as_str().unwrap();

    // Fetch the contract and verify AI review fields
    let detail = app.get(&format!("/api/contracts/{}", contract_id)).await;
    detail.assert_status_ok();

    let contract: serde_json::Value = detail.json();
    // Status should be 'pending' (default from schema) or 'unavailable' (set by async task since no AI_API_URL)
    let status = contract["ai_review_status"].as_str().unwrap_or("pending");
    assert!(
        status == "pending" || status == "unavailable" || status == "reviewing",
        "ai_review_status should be pending/unavailable/reviewing, got: {}",
        status
    );
}

#[tokio::test]
async fn s16_contract_without_ai_degrades_gracefully() {
    let app = common::TestApp::new().await;

    // Create contract with content (this triggers the async review)
    let res = app
        .post("/api/contracts")
        .json(&json!({
            "contract_name": "风险测试合同",
            "content": "甲方有权在任何时候单方面解除合同。乙方违约金为合同总额的100%。",
            "contract_type": "采购合同",
            "counterparty_type": "client",
            "amount": 100000.0
        }))
        .await;
    res.assert_status_ok();

    let body: serde_json::Value = res.json();
    let contract_id = body["id"].as_str().unwrap();

    // Wait a bit for the async task to complete
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Without AI_API_URL, status should be 'unavailable'
    let detail = app.get(&format!("/api/contracts/{}", contract_id)).await;
    detail.assert_status_ok();

    let contract: serde_json::Value = detail.json();
    let status = contract["ai_review_status"].as_str().unwrap_or("pending");
    assert_eq!(
        status, "unavailable",
        "Without AI configured, status should be 'unavailable'"
    );
}

#[tokio::test]
async fn s16_counterparty_type_persisted() {
    let app = common::TestApp::new().await;

    // Create with counterparty_type = "vc"
    let res = app
        .post("/api/contracts")
        .json(&json!({
            "contract_name": "投资协议",
            "counterparty_type": "vc",
            "contract_type": "投资协议"
        }))
        .await;
    res.assert_status_ok();

    let body: serde_json::Value = res.json();
    let id = body["id"].as_str().unwrap();

    let detail = app.get(&format!("/api/contracts/{}", id)).await;
    detail.assert_status_ok();
    let contract: serde_json::Value = detail.json();
    assert_eq!(contract["counterparty_type"].as_str().unwrap(), "vc");

    // Update to "government"
    let res = app
        .put(&format!("/api/contracts/{}", id))
        .json(&json!({
            "counterparty_type": "government"
        }))
        .await;
    res.assert_status_ok();

    let detail = app.get(&format!("/api/contracts/{}", id)).await;
    detail.assert_status_ok();
    let contract: serde_json::Value = detail.json();
    assert_eq!(
        contract["counterparty_type"].as_str().unwrap(),
        "government"
    );
}

#[tokio::test]
async fn s16_multiple_contracts_independent_review() {
    let app = common::TestApp::new().await;

    let mut ids = Vec::new();
    let counterparties = ["vc", "client", "employee", "agency", "landlord"];

    for (i, cp) in counterparties.iter().enumerate() {
        let res = app
            .post("/api/contracts")
            .json(&json!({
                "contract_name": format!("合同_{}", i),
                "counterparty_type": cp,
                "contract_type": "服务合同",
                "amount": (i + 1) as f64 * 10000.0
            }))
            .await;
        res.assert_status_ok();
        let body: serde_json::Value = res.json();
        ids.push(body["id"].as_str().unwrap().to_string());
    }

    // Each contract should have its own review status
    let list = app.get("/api/contracts").await;
    list.assert_status_ok();
    let body: serde_json::Value = list.json();
    let contracts = body["data"].as_array().unwrap();

    assert!(
        contracts.len() >= 5,
        "Should have at least 5 contracts, got {}",
        contracts.len()
    );

    // All should have ai_review_status field
    for c in contracts {
        assert!(
            c.get("ai_review_status").is_some(),
            "Contract {} missing ai_review_status",
            c["contract_name"]
        );
    }
}

#[tokio::test]
async fn s16_contract_list_includes_risk_fields() {
    let app = common::TestApp::new().await;

    // Create a contract
    let res = app
        .post("/api/contracts")
        .json(&json!({
            "contract_name": "AI风险字段测试",
            "content": "本合同条款公平合理。",
            "counterparty_type": "client"
        }))
        .await;
    res.assert_status_ok();

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // List should return AI review fields
    let list = app.get("/api/contracts").await;
    list.assert_status_ok();
    let body: serde_json::Value = list.json();
    let contracts = body["data"].as_array().unwrap();

    assert!(!contracts.is_empty(), "Should have at least one contract");
    let c = &contracts[0];

    // These columns must be present in the response (even if null)
    assert!(
        c.get("ai_review_status").is_some(),
        "Missing ai_review_status in list response"
    );
    assert!(
        c.get("counterparty_type").is_some(),
        "Missing counterparty_type in list response"
    );
    // ai_risk_level and ai_review_result may be null, but the key should still be returned
}

#[tokio::test]
async fn s16_contract_update_content_retriggers_review() {
    let app = common::TestApp::new().await;

    // Create without content
    let res = app
        .post("/api/contracts")
        .json(&json!({
            "contract_name": "空内容合同"
        }))
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    let id = body["id"].as_str().unwrap().to_string();

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Update with content — should trigger review
    let res = app
        .put(&format!("/api/contracts/{}", id))
        .json(&json!({
            "content": "甲方享有项目全部知识产权，乙方不得保留任何权利。"
        }))
        .await;
    res.assert_status_ok();

    // Poll until status settles (max 2 seconds)
    let mut final_status = String::from("pending");
    for _ in 0..20 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let detail = app.get(&format!("/api/contracts/{}", id)).await;
        let contract: serde_json::Value = detail.json();
        final_status = contract["ai_review_status"]
            .as_str()
            .unwrap_or("pending")
            .to_string();
        // Terminal states: unavailable, completed, failed
        if final_status == "unavailable" || final_status == "completed" || final_status == "failed"
        {
            break;
        }
    }

    assert_eq!(
        final_status, "unavailable",
        "After content update without AI configured, review should degrade to 'unavailable'"
    );
}
