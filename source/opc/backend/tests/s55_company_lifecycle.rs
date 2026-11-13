/// Scenario 55: 🏢 Company Lifecycle (公司生命周期)
/// Tests: company CRUD, soft-delete, switch-company, RBAC enforcement
/// Covers: companies.rs — create/get/update/delete/switch + member RBAC gates
use crate::common;
use axum::http::StatusCode;
use serde_json::json;

// ── Create ──

#[tokio::test]
async fn s55_create_company() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/companies")
        .json(&json!({
            "name": "新公司Alpha",
            "legal_name": "Alpha有限公司",
            "tax_id": "91110000MA0TEST01"
        }))
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert!(body["id"].is_string(), "Should return company id");
    assert_eq!(body["name"].as_str().unwrap(), "新公司Alpha");
}

#[tokio::test]
async fn s55_create_second_company_not_default() {
    let app = common::TestApp::new().await;

    // First company already exists (created by TestApp). Create second.
    let res = app
        .post("/api/companies")
        .json(&json!({ "name": "SecondCo" }))
        .await;
    res.assert_status_ok();
    let new_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // List companies — the new one should NOT be default
    let list = app.get("/api/companies").await;
    list.assert_status_ok();
    let companies = list.json::<serde_json::Value>()["data"]
        .as_array()
        .unwrap()
        .clone();

    let new_co = companies.iter().find(|c| c["id"].as_str() == Some(&new_id));
    assert!(new_co.is_some(), "New company should appear in list");
    // The first company was created with is_default=1 in seed; second should be 0
    assert_eq!(new_co.unwrap()["is_default"].as_bool(), Some(false));
}

// ── Get / Detail ──

#[tokio::test]
async fn s55_get_company_detail() {
    let app = common::TestApp::new().await;

    // Create company with full details
    let res = app
        .post("/api/companies")
        .json(&json!({
            "name": "DetailCo",
            "legal_name": "Detail法律名",
            "tax_id": "TAX-DETAIL-001",
            "address": "上海市浦东新区"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Get company detail
    let detail = app.get(&format!("/api/companies/{}", id)).await;
    detail.assert_status_ok();
    let d: serde_json::Value = detail.json();
    assert_eq!(d["name"].as_str().unwrap(), "DetailCo");
    assert_eq!(d["legal_name"].as_str().unwrap(), "Detail法律名");
    assert_eq!(d["tax_id"].as_str().unwrap(), "TAX-DETAIL-001");
    assert_eq!(d["address"].as_str().unwrap(), "上海市浦东新区");
}

// ── Update ──

#[tokio::test]
async fn s55_update_company() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/companies")
        .json(&json!({ "name": "OldName" }))
        .await;
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Update
    let update_res = app
        .put(&format!("/api/companies/{}", id))
        .json(&json!({ "name": "NewName", "address": "北京市朝阳区" }))
        .await;
    update_res.assert_status_ok();

    // Verify
    let d: serde_json::Value = app.get(&format!("/api/companies/{}", id)).await.json();
    assert_eq!(d["name"].as_str().unwrap(), "NewName");
    assert_eq!(d["address"].as_str().unwrap(), "北京市朝阳区");
}

#[tokio::test]
async fn s55_member_cannot_update_company() {
    let app = common::TestApp::new().await;

    // Create a company
    let res = app
        .post("/api/companies")
        .json(&json!({ "name": "ProtectedCo" }))
        .await;
    let co_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Add a plain member
    let user_b = common::seed::create_user(&app.pool, "member55@test-com.example.invalid", "Member55").await;
    app.post(&format!("/api/companies/{}/members", co_id))
        .json(&json!({ "email": "member55@test-com.example.invalid", "role": "member" }))
        .await;

    let session_b = common::seed::create_session(&app.pool, &user_b, &co_id).await;

    // Member tries to update company — should be 403
    let res = app
        .server
        .put(&format!("/api/companies/{}", co_id))
        .authorization_bearer(&session_b)
        .json(&json!({ "name": "HackedName" }))
        .await;
    assert_eq!(res.status_code(), 403);
}

// ── Soft Delete ──

#[tokio::test]
async fn s55_soft_delete_company() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/companies")
        .json(&json!({ "name": "ToDeleteCo" }))
        .await;
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Delete
    let del = app.delete(&format!("/api/companies/{}", id)).await;
    assert_eq!(del.status_code(), StatusCode::NO_CONTENT);

    // Should no longer appear in list
    let list = app.get("/api/companies").await;
    let companies = list.json::<serde_json::Value>()["data"]
        .as_array()
        .unwrap()
        .clone();
    let found = companies.iter().find(|c| c["id"].as_str() == Some(&id));
    assert!(found.is_none(), "Deleted company should not appear in list");
}

#[tokio::test]
async fn s55_deleted_company_get_returns_error() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/companies")
        .json(&json!({ "name": "WillBeGone" }))
        .await;
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Soft delete
    app.delete(&format!("/api/companies/{}", id)).await;

    // GET should fail (403 because verify_membership checks is_deleted)
    let get = app.get(&format!("/api/companies/{}", id)).await;
    assert!(
        get.status_code() == StatusCode::FORBIDDEN || get.status_code() == StatusCode::NOT_FOUND,
        "Deleted company GET should return 403 or 404, got: {}",
        get.status_code()
    );
}

#[tokio::test]
async fn s55_only_owner_can_delete() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/companies")
        .json(&json!({ "name": "OnlyOwnerDelete" }))
        .await;
    let co_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Add admin member
    let user_b = common::seed::create_user(&app.pool, "admin55@test-com.example.invalid", "Admin55").await;
    app.post(&format!("/api/companies/{}/members", co_id))
        .json(&json!({ "email": "admin55@test-com.example.invalid", "role": "admin" }))
        .await;

    let session_b = common::seed::create_session(&app.pool, &user_b, &co_id).await;

    // Admin tries to delete — should be 403
    let del = app
        .server
        .delete(&format!("/api/companies/{}", co_id))
        .authorization_bearer(&session_b)
        .await;
    assert_eq!(
        del.status_code(),
        403,
        "Admin should not be able to delete, only owner"
    );
}

// ── Switch Company ──

#[tokio::test]
async fn s55_switch_company() {
    let app = common::TestApp::new().await;

    // Create second company
    let res = app
        .post("/api/companies")
        .json(&json!({ "name": "SwitchTarget" }))
        .await;
    let new_co_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Switch
    let switch = app
        .post(&format!("/api/companies/{}/switch", new_co_id))
        .await;
    switch.assert_status_ok();
    let body: serde_json::Value = switch.json();
    assert_eq!(body["current_company_id"].as_str().unwrap(), new_co_id);
}

#[tokio::test]
async fn s55_list_marks_current_session_company() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/companies")
        .json(&json!({ "name": "CurrentMarkerTarget" }))
        .await;
    res.assert_status_ok();
    let new_co_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    app.post(&format!("/api/companies/{}/switch", new_co_id))
        .await
        .assert_status_ok();

    let list = app.get("/api/companies").await;
    list.assert_status_ok();
    let body: serde_json::Value = list.json();
    let companies = body["data"].as_array().unwrap();
    let switched = companies
        .iter()
        .find(|company| company["id"].as_str() == Some(new_co_id.as_str()))
        .expect("switched company should be listed");
    assert_eq!(switched["is_current"].as_bool(), Some(true));
    let original = companies
        .iter()
        .find(|company| company["id"].as_str() == Some(app.company_id.as_str()))
        .expect("original company should be listed");
    assert_eq!(original["is_current"].as_bool(), Some(false));
}

#[tokio::test]
async fn s55_switch_to_non_member_company() {
    let app_a = common::TestApp::new().await;
    let app_b = app_a.create_second_tenant().await;

    // User A tries to switch to B's company — should fail
    let switch = app_a
        .post(&format!("/api/companies/{}/switch", app_b.company_id))
        .await;
    assert_eq!(
        switch.status_code(),
        403,
        "Should not switch to non-member company"
    );
}

// ── Get Member by ID ──

#[tokio::test]
async fn s55_get_member_by_id() {
    let app = common::TestApp::new().await;
    let _user_b = common::seed::create_user(&app.pool, "getmember55@test-com.example.invalid", "GetMe").await;

    // Add member
    let add_res = app
        .post(&format!("/api/companies/{}/members", app.company_id))
        .json(&json!({ "email": "getmember55@test-com.example.invalid", "role": "accountant" }))
        .await;
    add_res.assert_status_ok();
    let member_id = add_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Get member by ID
    let get_res = app
        .get(&format!(
            "/api/companies/{}/members/{}",
            app.company_id, member_id
        ))
        .await;
    get_res.assert_status_ok();
    let m: serde_json::Value = get_res.json();
    assert_eq!(m["role"].as_str().unwrap(), "accountant");
    assert_eq!(m["email"].as_str().unwrap(), "getmember55@test-com.example.invalid");
}

// ── Duplicate Member ──

#[tokio::test]
async fn s55_add_nonexistent_user_fails() {
    let app = common::TestApp::new().await;

    // Try to add a user that doesn't exist
    let res = app
        .post(&format!("/api/companies/{}/members", app.company_id))
        .json(&json!({ "email": "ghost@nowhere-com.example.invalid", "role": "member" }))
        .await;
    assert_eq!(
        res.status_code(),
        StatusCode::NOT_FOUND,
        "Should fail for non-existent email"
    );
}

#[tokio::test]
async fn s55_add_existing_member_returns_conflict_without_changing_role() {
    let app = common::TestApp::new().await;
    let invited_user_id =
        common::seed::create_user(&app.pool, "duplicate55@test-com.example.invalid", "Duplicate 55").await;

    let first = app
        .post(&format!("/api/companies/{}/members", app.company_id))
        .json(&json!({ "email": "duplicate55@test-com.example.invalid", "role": "member" }))
        .await;
    first.assert_status_ok();

    let duplicate = app
        .post(&format!("/api/companies/{}/members", app.company_id))
        .json(&json!({ "email": "duplicate55@test-com.example.invalid", "role": "admin" }))
        .await;
    assert_eq!(duplicate.status_code(), StatusCode::CONFLICT);
    let body = duplicate.json::<serde_json::Value>();
    assert!(
        body["error"].as_str().unwrap_or("").contains("already"),
        "unexpected error body: {body}"
    );

    let role: String =
        sqlx::query_scalar("SELECT role FROM company_members WHERE company_id = ? AND user_id = ?")
            .bind(&app.company_id)
            .bind(&invited_user_id)
            .fetch_one(&app.pool)
            .await
            .expect("member role");
    assert_eq!(role, "member");
}
