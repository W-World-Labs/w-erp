/// Scenario 49: 🛡️ Webhooks, Audit & Security
/// Tests webhook CRUD, audit log export, API catalog, and security audit.
use crate::common;
use serde_json::json;

// ==================== Webhook CRUD ====================

#[tokio::test]
async fn s49_webhook_lifecycle() {
    let app = common::TestApp::new().await;

    // List — empty
    let res = app.get("/api/v1/webhooks").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["count"], 0);

    // Create webhook
    let res = app
        .post("/api/v1/webhooks")
        .json(&json!({
            "url": "https://example.com/hook",
            "events": "invoice.created,payment.received",
            "description": "Test webhook",
            "secret": "s3cret"
        }))
        .await;
    res.assert_status_ok();
    let created: serde_json::Value = res.json();
    assert_eq!(created["url"], "https://example.com/hook");
    assert_eq!(created["events"], "invoice.created,payment.received");
    let wh_id = created["id"].as_str().unwrap().to_string();

    // List — should have 1
    let res = app.get("/api/v1/webhooks").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["count"], 1);

    // Delete
    let res = app.delete(&format!("/api/v1/webhooks/{}", wh_id)).await;
    assert_eq!(res.status_code(), 204);

    // List — empty again
    let res = app.get("/api/v1/webhooks").await;
    res.assert_status_ok();
    assert_eq!(res.json::<serde_json::Value>()["count"], 0);
}

// ==================== Webhook — URL Required ====================

#[tokio::test]
async fn s49_webhook_url_required() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/v1/webhooks")
        .json(&json!({ "events": "*" }))
        .await;
    assert_eq!(res.status_code(), 400);
}

// ==================== Webhook — Test Ping ====================

#[tokio::test]
async fn s49_webhook_test_ping() {
    let app = common::TestApp::new().await;

    // Create
    let res = app
        .post("/api/v1/webhooks")
        .json(&json!({ "url": "https://httpbin.org/post", "events": "*" }))
        .await;
    res.assert_status_ok();
    let wh_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Test ping (may fail due to network, but should return structured response)
    let res = app
        .post(&format!("/api/v1/webhooks/{}/test", wh_id))
        .json(&json!({}))
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert!(body["status"].is_string());
    assert!(body["payload_sent"]["event"] == "webhook.test");
}

// ==================== Webhook — Delete Not Found ====================

#[tokio::test]
async fn s49_webhook_delete_not_found() {
    let app = common::TestApp::new().await;
    let res = app.delete("/api/v1/webhooks/nonexistent-id").await;
    assert_eq!(res.status_code(), 404);
}

// ==================== Audit Export — Empty ====================

#[tokio::test]
async fn s49_audit_export_empty() {
    let app = common::TestApp::new().await;
    let res = app.get("/api/v1/audit/export").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert!(body["data"].is_array());
    assert!(body["filters"]["limit"].as_i64().unwrap() > 0);
}

// ==================== Audit Export — With Filters ====================

#[tokio::test]
async fn s49_audit_export_filtered() {
    let app = common::TestApp::new().await;
    let res = app
        .get("/api/v1/audit/export?entity_type=invoices&action=create&limit=10")
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["filters"]["entity_type"], "invoices");
    assert_eq!(body["filters"]["action"], "create");
    assert_eq!(body["filters"]["limit"], 10);
}

// ==================== API Catalog ====================

#[tokio::test]
async fn s49_api_catalog() {
    let app = common::TestApp::new().await;
    let res = app.get("/api/v1/api-catalog").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["version"], "1.0");
    let modules = body["modules"].as_array().unwrap();
    assert!(modules.len() >= 6, "should have at least 6 modules");
    // Check WP modules exist
    let names: Vec<&str> = modules
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();
    assert!(names.iter().any(|n| n.contains("CRUD")));
    assert!(names.iter().any(|n| n.contains("Reports")));
    assert!(names.iter().any(|n| n.contains("Currency")));
    assert!(names.iter().any(|n| n.contains("Compliance")));
    assert!(names.iter().any(|n| n.contains("Webhooks")));
}

// ==================== Security Audit — Clean State ====================

#[tokio::test]
async fn s49_security_audit_clean() {
    let app = common::TestApp::new().await;
    let res = app.get("/api/v1/security/audit").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert!(body["security_score"]["total"].as_i64().unwrap() >= 0);
    assert!(body["security_score"]["level"].is_string());
    assert!(body["gdpr_compliance"]["data_minimization"] == true);
    assert!(body["gdpr_compliance"]["data_portability"]["status"] == "ok");
    assert!(body["api_keys"]["total"].is_number());
    assert!(body["recommendations"].is_array());
}

// ==================== Webhook — Wildcard Events ====================

#[tokio::test]
async fn s49_webhook_wildcard_events() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/v1/webhooks")
        .json(&json!({ "url": "https://example.com/all", "events": "*" }))
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["events"], "*");
    assert_eq!(body["is_active"], true);
}
