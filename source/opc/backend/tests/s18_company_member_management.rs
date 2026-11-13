/// Scenario 18: 👥 Company Member Management (公司成员管理)
/// Tests: add member, list members, update role, remove member, permission gates
use crate::common;
use serde_json::json;

#[tokio::test]
async fn s18_add_member_requires_admin() {
    let app = common::TestApp::new().await;
    // Create second user directly via DB
    let user_b = common::seed::create_user(&app.pool, "newbie@test-com.example.invalid", "New User").await;

    // Owner should be able to add member
    let res = app
        .post(&format!("/api/companies/{}/members", app.company_id))
        .json(&json!({ "email": "newbie@test-com.example.invalid", "role": "member" }))
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn s18_list_members_returns_all() {
    let app = common::TestApp::new().await;
    // Owner is already a member
    let res = app
        .get(&format!("/api/companies/{}/members", app.company_id))
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    let members = body["data"].as_array().unwrap();
    assert!(members.len() >= 1, "Should have at least the owner");
    // Owner should have role 'owner'
    let owner = members.iter().find(|m| m["role"].as_str() == Some("owner"));
    assert!(owner.is_some(), "Owner should be in members list");
}

#[tokio::test]
async fn s18_update_member_role() {
    let app = common::TestApp::new().await;
    let _user_b = common::seed::create_user(&app.pool, "member@test-com.example.invalid", "Member User").await;

    // Add member
    let add_res = app
        .post(&format!("/api/companies/{}/members", app.company_id))
        .json(&json!({ "email": "member@test-com.example.invalid", "role": "member" }))
        .await;
    add_res.assert_status_ok();
    let member_id = add_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Update role to admin
    let update_res = app
        .put(&format!(
            "/api/companies/{}/members/{}",
            app.company_id, member_id
        ))
        .json(&json!({ "role": "admin" }))
        .await;
    update_res.assert_status_ok();

    // Verify role changed
    let list_res = app
        .get(&format!("/api/companies/{}/members", app.company_id))
        .await;
    let members = list_res.json::<serde_json::Value>()["data"]
        .as_array()
        .unwrap()
        .clone();
    let updated = members
        .iter()
        .find(|m| m["id"].as_str() == Some(&member_id))
        .unwrap();
    assert_eq!(updated["role"].as_str().unwrap(), "admin");
}

#[tokio::test]
async fn s18_remove_member() {
    let app = common::TestApp::new().await;
    let _user_b = common::seed::create_user(&app.pool, "temp@test-com.example.invalid", "Temp User").await;

    // Add then remove
    let add_res = app
        .post(&format!("/api/companies/{}/members", app.company_id))
        .json(&json!({ "email": "temp@test-com.example.invalid", "role": "member" }))
        .await;
    add_res.assert_status_ok();
    let member_id = add_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let del_res = app
        .delete(&format!(
            "/api/companies/{}/members/{}",
            app.company_id, member_id
        ))
        .await;
    assert!(del_res.status_code().is_success());

    // Verify removed
    let list_res = app
        .get(&format!("/api/companies/{}/members", app.company_id))
        .await;
    let members = list_res.json::<serde_json::Value>()["data"]
        .as_array()
        .unwrap()
        .clone();
    let found = members
        .iter()
        .find(|m| m["id"].as_str() == Some(&member_id));
    assert!(found.is_none(), "Member should be removed");
}

#[tokio::test]
async fn s18_non_admin_cannot_add_member() {
    let app = common::TestApp::new().await;
    // Create a user B who is a plain member
    let _user_b = common::seed::create_user(&app.pool, "plain@test-com.example.invalid", "Plain User").await;
    app.post(&format!("/api/companies/{}/members", app.company_id))
        .json(&json!({ "email": "plain@test-com.example.invalid", "role": "member" }))
        .await;

    // Create user C to be invited by B
    let _user_c = common::seed::create_user(&app.pool, "c@test-com.example.invalid", "User C").await;

    // Login as B (create session for B)
    let session_b = common::seed::create_session(&app.pool, &_user_b, &app.company_id).await;

    // B tries to add C — should be forbidden
    let res = app
        .server
        .post(&format!("/api/companies/{}/members", app.company_id))
        .authorization_bearer(&session_b)
        .json(&json!({ "email": "c@test-com.example.invalid", "role": "member" }))
        .await;
    assert_eq!(res.status_code(), 403);
}

#[tokio::test]
async fn s18_added_member_sees_company() {
    let app = common::TestApp::new().await;
    let user_b = common::seed::create_user(&app.pool, "viewer@test-com.example.invalid", "Viewer").await;

    // Add B as member
    app.post(&format!("/api/companies/{}/members", app.company_id))
        .json(&json!({ "email": "viewer@test-com.example.invalid", "role": "accountant" }))
        .await;

    // B logs in: create session for B with a different company initially
    let session_b = common::seed::create_session(&app.pool, &user_b, &app.company_id).await;

    // B lists companies — should see the owner's company
    let res = app
        .server
        .get("/api/companies")
        .authorization_bearer(&session_b)
        .await;
    res.assert_status_ok();
    let companies = res.json::<serde_json::Value>()["data"]
        .as_array()
        .unwrap()
        .clone();
    let found = companies
        .iter()
        .find(|c| c["id"].as_str() == Some(&app.company_id));
    assert!(found.is_some(), "Member B should see owner's company");
    assert_eq!(found.unwrap()["role"].as_str().unwrap(), "accountant");
}

#[tokio::test]
async fn s18_add_member_rejects_role_outside_whitelist() {
    let app = common::TestApp::new().await;
    let _user_b =
        common::seed::create_user(&app.pool, "invalid-role@test-com.example.invalid", "Invalid Role").await;

    let res = app
        .post(&format!("/api/companies/{}/members", app.company_id))
        .json(&json!({ "email": "invalid-role@test-com.example.invalid", "role": "super-admin" }))
        .await;

    assert_eq!(res.status_code(), 400);
    let body = res.json::<serde_json::Value>();
    assert!(
        body["error"]
            .as_str()
            .unwrap_or("")
            .contains("invalid role"),
        "unexpected error body: {body}"
    );
}
