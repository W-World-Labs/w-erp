/// S17: API Key Management & Authentication
/// Tests the complete API key lifecycle: create → auth → revoke → reject
use crate::common;
use axum::http::{HeaderName, HeaderValue};
use common::TestApp;
use serde_json::json;

fn api_key_header(key: &str) -> (HeaderName, HeaderValue) {
    (
        HeaderName::from_static("x-api-key"),
        HeaderValue::from_str(key).unwrap(),
    )
}

#[tokio::test]
async fn s17_create_api_key_returns_key_once() {
    let app = TestApp::new().await;
    let res = app
        .post("/api/v1/api-keys")
        .json(&json!({ "name": "CI Bot", "scopes": "read,write" }))
        .await;
    assert_eq!(res.status_code(), 201);
    let body: serde_json::Value = res.json();
    assert!(body["key"].as_str().unwrap().starts_with("om_"));
    assert_eq!(body["name"], "CI Bot");
    assert_eq!(body["scopes"], "read,write");
    assert!(body["key_prefix"].as_str().unwrap().starts_with("om_"));
}

#[tokio::test]
async fn s17_list_api_keys_shows_prefix_not_key() {
    let app = TestApp::new().await;
    app.post("/api/v1/api-keys")
        .json(&json!({ "name": "Test Key" }))
        .await;
    let res = app.get("/api/v1/api-keys").await;
    assert_eq!(res.status_code(), 200);
    let body: serde_json::Value = res.json();
    assert_eq!(body["total"], 1);
    let key_info = &body["keys"][0];
    assert!(key_info["key_prefix"].as_str().unwrap().starts_with("om_"));
    assert!(key_info.get("key").is_none());
}

#[tokio::test]
async fn s17_api_key_authenticates_requests() {
    let app = TestApp::new().await;
    let res = app
        .post("/api/v1/api-keys")
        .json(&json!({ "name": "Agent Key" }))
        .await;
    let key = res.json::<serde_json::Value>()["key"]
        .as_str()
        .unwrap()
        .to_string();

    let (name, value) = api_key_header(&key);
    let res = app
        .server
        .get("/api/v1/assets")
        .add_header(name, value)
        .await;
    assert_eq!(res.status_code(), 200);
}

#[tokio::test]
async fn s17_revoked_key_rejected() {
    let app = TestApp::new().await;
    let res = app
        .post("/api/v1/api-keys")
        .json(&json!({ "name": "Temp Key" }))
        .await;
    let body: serde_json::Value = res.json();
    let key = body["key"].as_str().unwrap().to_string();
    let key_id = body["id"].as_str().unwrap().to_string();

    // Verify it works
    let (name, value) = api_key_header(&key);
    let res = app
        .server
        .get("/api/v1/assets")
        .add_header(name, value)
        .await;
    assert_eq!(res.status_code(), 200);

    // Revoke
    app.post(&format!("/api/v1/api-keys/{}/revoke", key_id))
        .await;

    // Should fail now
    let (name, value) = api_key_header(&key);
    let res = app
        .server
        .get("/api/v1/assets")
        .add_header(name, value)
        .await;
    assert_eq!(res.status_code(), 401);
}

#[tokio::test]
async fn s17_invalid_api_key_rejected() {
    let app = TestApp::new().await;
    let (name, value) = api_key_header("om_invalid_key_here");
    let res = app
        .server
        .get("/api/v1/assets")
        .add_header(name, value)
        .await;
    assert_eq!(res.status_code(), 401);
}

#[tokio::test]
async fn s17_delete_key_permanently() {
    let app = TestApp::new().await;
    let res = app
        .post("/api/v1/api-keys")
        .json(&json!({ "name": "Delete Me" }))
        .await;
    let body: serde_json::Value = res.json();
    let key_id = body["id"].as_str().unwrap().to_string();

    let del = app.delete(&format!("/api/v1/api-keys/{}", key_id)).await;
    assert_eq!(del.status_code(), 200);

    let list = app.get("/api/v1/api-keys").await;
    let body: serde_json::Value = list.json();
    assert_eq!(body["total"], 0);
}

#[tokio::test]
async fn s17_api_keys_are_tenant_scoped() {
    let app_a = TestApp::new().await;
    let app_b = app_a.create_second_tenant().await;

    let res = app_a
        .post("/api/v1/api-keys")
        .json(&json!({ "name": "Tenant A Key" }))
        .await;
    assert_eq!(res.status_code(), 201);

    let a_list = app_a.get("/api/v1/api-keys").await;
    assert_eq!(a_list.status_code(), 200);
    assert_eq!(
        a_list.json::<serde_json::Value>()["total"]
            .as_i64()
            .unwrap(),
        1
    );

    let b_list = app_b.get("/api/v1/api-keys").await;
    assert_eq!(b_list.status_code(), 200);
    assert_eq!(
        b_list.json::<serde_json::Value>()["total"]
            .as_i64()
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn s17_expired_api_key_is_rejected() {
    let app = TestApp::new().await;
    let res = app
        .post("/api/v1/api-keys")
        .json(&json!({ "name": "Short Lived", "expires_in_days": 1 }))
        .await;
    assert_eq!(res.status_code(), 201);

    let body: serde_json::Value = res.json();
    let key = body["key"].as_str().unwrap().to_string();
    let key_id = body["id"].as_str().unwrap().to_string();

    sqlx::query("UPDATE api_keys SET expires_at = datetime('now', '-1 day') WHERE id = ?")
        .bind(&key_id)
        .execute(&app.pool)
        .await
        .unwrap();

    let (name, value) = api_key_header(&key);
    let res = app
        .server
        .get("/api/v1/assets")
        .add_header(name, value)
        .await;
    assert_eq!(res.status_code(), 401);
}
