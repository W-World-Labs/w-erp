use crate::common;
use axum::http::StatusCode;
use serde_json::{Value, json};

#[tokio::test]
async fn s141_default_nav_prefs_are_empty() {
    let app = common::TestApp::new().await;
    let res = app.get("/api/v1/settings/nav").await;
    res.assert_status_ok();
    let body = res.json::<Value>();
    assert_eq!(body["version"], 1);
    assert_eq!(body["hidden_ids"], json!([]));
    assert_eq!(body["category_order"], json!([]));
    assert_eq!(body["item_order"], json!({}));
    assert_eq!(body["locked_ids"], json!(["dashboard", "settings"]));
    assert!(body["updated_at"].is_null());
}

#[tokio::test]
async fn s141_put_roundtrip_strips_locked_and_isolates_tenants() {
    let app = common::TestApp::new().await;
    let saved = app
        .put("/api/v1/settings/nav")
        .json(&json!({
            "version": 1,
            "hidden_ids": ["invoices", "dashboard", "settings", "NOPE"],
            "category_order": ["business", "overview"],
            "item_order": { "finance": ["payments", "invoices"] }
        }))
        .await;
    saved.assert_status_ok();
    let body = saved.json::<Value>();
    assert_eq!(body["hidden_ids"], json!(["invoices"]));
    assert_eq!(body["category_order"], json!(["business", "overview"]));
    assert_eq!(
        body["item_order"]["finance"],
        json!(["payments", "invoices"])
    );
    assert!(body["updated_at"].as_str().is_some());

    let fetched = app.get("/api/v1/settings/nav").await.json::<Value>();
    assert_eq!(fetched["hidden_ids"], json!(["invoices"]));

    let other = app.create_second_tenant().await;
    let other_get = other.get("/api/v1/settings/nav").await.json::<Value>();
    assert_eq!(other_get["hidden_ids"], json!([]));

    let other_put = other
        .put("/api/v1/settings/nav")
        .json(&json!({ "hidden_ids": ["crm"] }))
        .await;
    other_put.assert_status_ok();
    let mine = app.get("/api/v1/settings/nav").await.json::<Value>();
    assert_eq!(mine["hidden_ids"], json!(["invoices"]));
}

#[tokio::test]
async fn s141_rejects_non_object_and_scopes_by_company() {
    let app = common::TestApp::new().await;
    let bad = app
        .put("/api/v1/settings/nav")
        .json(&json!(["invoices"]))
        .await;
    assert_eq!(bad.status_code(), StatusCode::BAD_REQUEST);

    app.put("/api/v1/settings/nav")
        .json(&json!({ "hidden_ids": ["quotes"] }))
        .await
        .assert_status_ok();

    let (_company_id, token) = app.create_second_company("AltCo", "USD").await;
    let other_company = app
        .server
        .get("/api/v1/settings/nav")
        .authorization_bearer(&token)
        .await
        .json::<Value>();
    assert_eq!(other_company["hidden_ids"], json!([]));
}
