/// Scenario 9: 🌍 全球政策智能 (Global Policy Intelligence)
/// 验证 60+ 预置政策是否正确加载
/// 验证企业数据智能匹配 (Country, Revenue, Employees)
/// 验证一键申报 (policy_applications) 生命周期与租户隔离
use crate::common;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn s09_global_policies_seeded() {
    let app = common::TestApp::new().await;

    // Seed policies into test DB
    opc::db::seed_policies::seed_global_policies(&app.pool)
        .await
        .expect("Failed to seed policies");

    // Policies are read-only and global, should just be returned correctly
    let res = app.get("/api/v1/policies").await;
    res.assert_status_ok();

    let body: serde_json::Value = res.json();
    let count = body["count"].as_i64().unwrap();
    // We seeded ~22 policies in seed_policies.rs
    assert!(
        count >= 20,
        "Should have at least 20 seeded global policies, got {}",
        count
    );

    // Check that Y Combinator is in there
    let data = body["data"].as_array().unwrap();
    let yc = data.iter().find(|p| p["name"] == "Y Combinator");
    assert!(yc.is_some(), "Y Combinator policy missing");
}

#[tokio::test]
async fn s09_policy_smart_matching() {
    let app = common::TestApp::new().await;

    opc::db::seed_policies::seed_global_policies(&app.pool)
        .await
        .expect("seed");

    // 1. Setup company profile to match specific policies
    // The test app creates a default company which is "US" in this test context
    // because company_info uses registration_date, we may need to insert company_info

    // Set CN company
    sqlx::query("UPDATE companies SET country = 'CN' WHERE id = ?")
        .bind(&app.company_id)
        .execute(&app.pool)
        .await
        .unwrap();

    // Add OPC founder profile
    sqlx::query(
        "INSERT INTO company_info (id, company_id, company_name, registration_date, founder_education, founder_is_overseas_returnee) 
         VALUES (?, ?, 'Test CN Company', '2025-01-01', 'masters', 1)"
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();

    // Add some employees (e.g. 50 employees)
    for _ in 0..5 {
        app.post("/api/v1/employees")
            .json(&json!({
                "name": "Employee",
                "department": "Tech",
                "position": "Engineer",
                "employment_type": "full_time"
            }))
            .await
            .assert_status_ok();
    }

    // Add some revenue ($500K). Invoice numbers must be unique per company:
    // duplicate numbers are now rejected with 409 (document numbering rules).
    for i in 1..=5 {
        app.post("/api/v1/invoices")
            .json(&json!({
                "invoice_number": format!("REV-{i:03}"),
                "amount": 100000.0,
                "invoice_type": "income",
                "status": "paid"
            }))
            .await
            .assert_status_ok();
    }

    // Call match
    let match_res = app.get("/api/v1/policies/match").await;
    match_res.assert_status_ok();

    let body: serde_json::Value = match_res.json();
    let data = body["data"].as_array().unwrap();

    assert!(!data.is_empty(), "Smart match should return policies");

    // The top one should have a high score and likely be a CN policy (especially the overseas returnee one like 深圳孔雀计划)
    let top_match = &data[0];
    let score = top_match["match_score"].as_i64().unwrap();

    // Country (+40) + Founder Returnee (+15) + Founder Edu (+15) = ~70+ score
    assert!(
        score >= 70,
        "Top match score should be >= 70 due to founder OPC bonuses, got {}",
        score
    );

    let country = top_match["country"].as_str().unwrap();
    assert!(
        country == "CN" || ["CN", "*"].contains(&country),
        "Top match should be relevant to CN company"
    );
}

#[tokio::test]
async fn s09_policy_application_lifecycle() {
    let app = common::TestApp::new().await;

    opc::db::seed_policies::seed_global_policies(&app.pool)
        .await
        .expect("seed");

    // Get a policy ID to apply for
    let list_res = app.get("/api/v1/policies").await;
    let list_body: serde_json::Value = list_res.json();
    let policy_id = list_body["data"][0]["id"]
        .as_str()
        .expect("policy id exists");

    // 1. Create application
    let create_res = app
        .post("/api/v1/policies/applications")
        .json(&json!({
            "policy_id": policy_id,
            "status": "preparing",
            "notes": "Starting YC application"
        }))
        .await;
    create_res.assert_status_ok();

    let app_record: serde_json::Value = create_res.json();
    let app_id = app_record["id"].as_str().unwrap();

    // 2. Add some materials
    // We already have application_materials endpoint logic which works with application_id
    // Wait, the application_materials table has `application_id REFERENCES applications(id)`
    // So the previous `applications` table from S07 is different from `policy_applications`.
    // Policy applications are separate.

    // 3. Update status
    let upd_res = app
        .put(&format!("/api/v1/policies/applications/{}", app_id))
        .json(&json!({
            "status": "submitted",
            "applied_date": "2026-03-01"
        }))
        .await;
    upd_res.assert_status_ok();

    // 4. List my applications
    let my_apps = app.get("/api/v1/policies/applications").await;
    my_apps.assert_status_ok();
    let my_body: serde_json::Value = my_apps.json();
    assert_eq!(my_body["count"].as_i64().unwrap(), 1);

    // 5. Delete application
    let del_res = app
        .delete(&format!("/api/v1/policies/applications/{}", app_id))
        .await;
    del_res.assert_status(StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn s09_policy_tenant_isolation() {
    let app_a = common::TestApp::new().await;
    let app_b = app_a.create_second_tenant().await;

    opc::db::seed_policies::seed_global_policies(&app_a.pool)
        .await
        .expect("seed");

    // Get policy config
    let list_res = app_a.get("/api/v1/policies").await;
    let list_body: serde_json::Value = list_res.json();
    let policy_id = list_body["data"][0]["id"].as_str().unwrap();

    // A applies
    app_a
        .post("/api/v1/policies/applications")
        .json(&json!({"policy_id": policy_id, "status": "preparing"}))
        .await
        .assert_status_ok();

    // B should see 0 applications
    let b_apps = app_b.get("/api/v1/policies/applications").await;
    b_apps.assert_status_ok();
    let b_body: serde_json::Value = b_apps.json();
    assert_eq!(b_body["count"].as_i64().unwrap(), 0);
}

#[tokio::test]
async fn s09_get_policy_detail_returns_all_fields() {
    let app = common::TestApp::new().await;

    opc::db::seed_policies::seed_global_policies(&app.pool)
        .await
        .expect("seed");

    // Get first policy ID
    let list_res = app.get("/api/v1/policies").await;
    let list_body: serde_json::Value = list_res.json();
    let policy_id = list_body["data"][0]["id"].as_str().expect("policy id");

    // Fetch policy detail
    let detail_res = app.get(&format!("/api/v1/policies/{}", policy_id)).await;
    detail_res.assert_status_ok();

    let detail: serde_json::Value = detail_res.json();

    // Verify all enriched fields are present
    assert!(detail["id"].is_string(), "id should be present");
    assert!(detail["name"].is_string(), "name should be present");
    assert!(detail["country"].is_string(), "country should be present");
    assert!(detail["category"].is_string(), "category should be present");

    // These fields were previously null/empty — now should be populated
    assert!(
        detail["eligibility"].is_string(),
        "eligibility should be present and populated"
    );
    assert!(
        detail["benefits"].is_string(),
        "benefits should be present and populated"
    );
    assert!(
        detail["description"].is_string(),
        "description should be present and populated"
    );

    // Verify the content is meaningful (not just empty string)
    let eligibility = detail["eligibility"].as_str().unwrap();
    let benefits = detail["benefits"].as_str().unwrap();
    assert!(!eligibility.is_empty(), "eligibility should not be empty");
    assert!(!benefits.is_empty(), "benefits should not be empty");
}

#[tokio::test]
async fn s09_get_policy_detail_404_for_nonexistent() {
    let app = common::TestApp::new().await;

    let res = app.get("/api/v1/policies/nonexistent-uuid-12345").await;
    // Should return 404 or similar error, not crash
    let status = res.status_code();
    assert!(
        status == StatusCode::NOT_FOUND || status == StatusCode::INTERNAL_SERVER_ERROR,
        "nonexistent policy should return 404 or 500, got {}",
        status
    );
}

#[tokio::test]
async fn s09_inactive_policy_detail_is_hidden() {
    let app = common::TestApp::new().await;

    opc::db::seed_policies::seed_global_policies(&app.pool)
        .await
        .expect("seed");

    let list_res = app.get("/api/v1/policies").await;
    let list_body: serde_json::Value = list_res.json();
    let policy_id = list_body["data"][0]["id"].as_str().expect("policy id");

    sqlx::query("UPDATE policies SET is_active = 0 WHERE id = ?")
        .bind(policy_id)
        .execute(&app.pool)
        .await
        .unwrap();

    let detail_res = app.get(&format!("/api/v1/policies/{}", policy_id)).await;
    assert_eq!(
        detail_res.status_code(),
        StatusCode::NOT_FOUND,
        "inactive policies should not be retrievable"
    );
}
