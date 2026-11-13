use crate::common;

use serde_json::{Value, json};

#[tokio::test]
async fn s128_profiles_returns_current_user_profile_with_display_labels() {
    let app = common::TestApp::new().await;

    let res = app.get("/api/profiles").await;
    res.assert_status_ok();
    let body = res.json::<Value>();
    let profile = &body["data"][0];

    assert_eq!(profile["user_id"], app.user_id);
    assert_eq!(profile["email"], "test@opc.test");
    assert_eq!(profile["full_name"], "Test User");
    assert!(profile["created_at_label"].as_str().unwrap().len() >= 10);
    assert!(profile["updated_at_label"].as_str().unwrap().len() >= 10);
    assert_eq!(profile["update_action"]["method"], "PUT");
    assert_eq!(
        profile["update_action"]["endpoint"],
        format!("/api/profiles/{}", profile["id"].as_str().unwrap())
    );
    let current = app.get("/api/profiles/current").await;
    current.assert_status_ok();
    let current = current.json::<Value>();
    assert_eq!(current["id"], profile["id"]);
    assert_eq!(current["full_name"], "Test User");
    assert_eq!(current["update_action"]["method"], "PUT");

    let profile_id = profile["id"].as_str().expect("profile id");
    let update = app
        .put(&format!("/api/profiles/{profile_id}"))
        .json(&json!({
            "display_name": "Backend Profile",
            "email": "backend-profile@opc.test",
            "avatar_url": "/api/files/backend-avatar-id"
        }))
        .await;
    update.assert_status_ok();

    let res = app.get("/api/profiles").await;
    res.assert_status_ok();
    let body = res.json::<Value>();
    let profile = &body["data"][0];

    assert_eq!(profile["full_name"], "Backend Profile");
    assert_eq!(profile["email"], "backend-profile@opc.test");
    assert_eq!(profile["avatar_url"], "/api/files/backend-avatar-id");
}
