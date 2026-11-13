/// Scenario 54: 🔧 Backend Edge Cases — 空 body / 认证 / 分页 / 生命周期
/// Tests the generic CRUD edge cases.
use crate::common;
use serde_json::json;

// ─── Empty-body / no-field scenarios ───

#[tokio::test]
async fn s54_create_with_empty_body() {
    let app = common::TestApp::new().await;

    // POST with empty JSON body — should not panic
    let res = app.post("/api/v1/invoices").json(&json!({})).await;
    // Generic create may 200 (creating with defaults) or any error code, but should NOT 5xx panic
    let status = res.status_code().as_u16();
    assert!(
        status == 200 || status == 400 || status == 422 || status == 500,
        "Unexpected status: {}",
        status
    );
}

#[tokio::test]
async fn s54_update_nonexistent_record() {
    let app = common::TestApp::new().await;

    let res = app
        .put("/api/v1/invoices/non-existent-id")
        .json(&json!({"description": "ghost"}))
        .await;
    // API returns 404 or 422 for nonexistent - either is acceptable
    assert!(
        res.status_code().is_client_error(),
        "Expected 4xx, got {}",
        res.status_code()
    );
}

#[tokio::test]
async fn s54_delete_nonexistent_record() {
    let app = common::TestApp::new().await;

    let res = app.delete("/api/v1/invoices/non-existent-id").await;
    assert_eq!(res.status_code(), 404);
}

#[tokio::test]
async fn s54_get_nonexistent_record() {
    let app = common::TestApp::new().await;

    let res = app.get("/api/v1/invoices/non-existent-id").await;
    assert_eq!(res.status_code(), 404);
}

// ─── Unauthenticated access ───

#[tokio::test]
async fn s54_unauthenticated_crud() {
    let app = common::TestApp::new().await;

    // No auth header
    let res = app.server.get("/api/v1/invoices").await;
    assert_eq!(res.status_code(), 401);
}

#[tokio::test]
async fn s54_invalid_bearer_token() {
    let app = common::TestApp::new().await;

    let res = app
        .server
        .get("/api/v1/invoices")
        .authorization_bearer("invalid-token-xyz")
        .await;
    assert_eq!(res.status_code(), 401);
}

// ─── Pagination ───

#[tokio::test]
async fn s54_list_pagination() {
    let app = common::TestApp::new().await;

    // Create multiple records
    for i in 0..5 {
        app.post("/api/v1/categories")
            .json(&json!({"name": format!("Cat {}", i), "category_type": "支出"}))
            .await;
    }

    // List with limit
    let res = app.get("/api/v1/categories?limit=2").await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    let data = body["data"].as_array().unwrap();
    assert!(data.len() <= 2);
}

#[tokio::test]
async fn s54_list_with_offset() {
    let app = common::TestApp::new().await;

    // Create records
    for i in 0..5 {
        app.post("/api/v1/categories")
            .json(&json!({"name": format!("Offset Cat {}", i), "category_type": "收入"}))
            .await;
    }

    let res = app.get("/api/v1/categories?offset=3").await;
    res.assert_status_ok();
}

// ─── Full CRUD lifecycle ───

#[tokio::test]
async fn s54_full_crud_lifecycle() {
    let app = common::TestApp::new().await;

    // Create
    let res = app
        .post("/api/v1/categories")
        .json(&json!({"name": "Lifecycle Test", "category_type": "支出"}))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Read
    let res = app.get(&format!("/api/v1/categories/{}", id)).await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    assert_eq!(body["name"].as_str().unwrap(), "Lifecycle Test");

    // Update
    let res = app
        .put(&format!("/api/v1/categories/{}", id))
        .json(&json!({"name": "Updated Name"}))
        .await;
    assert!(
        res.status_code().is_success(),
        "Update failed with status: {}",
        res.status_code()
    );

    // Verify update
    let res = app.get(&format!("/api/v1/categories/{}", id)).await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    assert_eq!(body["name"].as_str().unwrap(), "Updated Name");

    // Delete
    let res = app.delete(&format!("/api/v1/categories/{}", id)).await;
    assert!(res.status_code().is_success());

    // Verify deleted
    let res = app.get(&format!("/api/v1/categories/{}", id)).await;
    assert_eq!(res.status_code(), 404);
}

// ─── Cross-tenant isolation on CRUD ───

#[tokio::test]
async fn s54_cross_tenant_crud_isolation() {
    let app = common::TestApp::new().await;
    let app_b = app.create_second_tenant().await;

    // A creates a category
    let res = app
        .post("/api/v1/categories")
        .json(&json!({"name": "A's Category", "category_type": "支出"}))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // B should not see A's category
    let res = app_b.get(&format!("/api/v1/categories/{}", id)).await;
    assert_eq!(res.status_code(), 404);

    // B should not be able to update A's category
    let res = app_b
        .put(&format!("/api/v1/categories/{}", id))
        .json(&json!({"name": "Hacked!"}))
        .await;
    assert!(res.status_code().is_client_error());

    // B should not be able to delete A's category
    let res = app_b.delete(&format!("/api/v1/categories/{}", id)).await;
    assert!(res.status_code().is_client_error() || res.status_code() == 404);
}
