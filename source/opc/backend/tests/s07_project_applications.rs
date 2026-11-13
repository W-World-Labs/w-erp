/// Scenario 7: 📋 项目申报全流程
/// 创建项目 → 创建申报 → 添加材料 → 更新进度
///  → 分页 → 级联删除检查 → 租户隔离
///
/// Note: 后续将扩展全球创业政策搜集、智能推荐申报功能
use crate::common;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn s07_application_full_flow() {
    let app = common::TestApp::new().await;

    // Step 1: Create application (generic handler, requires title, type, application_no)
    let app_res = app
        .post("/api/v1/applications")
        .json(&json!({
            "title": "2026年高企认定申报",
            "type": "高新技术企业",
            "application_no": "GQ-2026-001",
            "category": "政府补贴",
            "deadline": "2026-06-30",
            "status": "draft",
            "progress": 10,
            "notes": "需要准备知识产权和研发费用材料"
        }))
        .await;
    app_res.assert_status_ok();
    let application: serde_json::Value = app_res.json();
    let app_id = application["id"].as_str().expect("application id");

    // Step 2: Add application material
    let mat_res = app
        .post("/api/v1/application_materials")
        .json(&json!({
            "application_id": app_id,
            "name": "企业营业执照副本",
            "required": 1,
            "status": "completed",
            "description": "最新年检版本"
        }))
        .await;
    mat_res.assert_status_ok();

    // Step 3: Add another material
    app.post("/api/v1/application_materials")
        .json(&json!({
            "application_id": app_id,
            "name": "知识产权证书",
            "required": 1,
            "status": "pending"
        }))
        .await
        .assert_status_ok();

    // Step 4: Update application progress
    let upd_res = app
        .put(&format!("/api/v1/applications/{}", app_id))
        .json(&json!({
            "progress": 50,
            "status": "in_progress"
        }))
        .await;
    upd_res.assert_status_ok();

    // Step 5: Get application — verify update
    let get_res = app.get(&format!("/api/v1/applications/{}", app_id)).await;
    get_res.assert_status_ok();
    let fetched: serde_json::Value = get_res.json();
    assert_eq!(fetched["progress"], 50);
    assert_eq!(fetched["status"], "in_progress");

    // Step 6: List materials
    let mats = app.get("/api/v1/application_materials").await;
    mats.assert_status_ok();
    let mat_body: serde_json::Value = mats.json();
    assert!(
        mat_body["count"].as_i64().unwrap() >= 2,
        "should have 2 materials"
    );

    // Step 7: Try delete application — should fail due to materials
    let del_res = app
        .delete(&format!("/api/v1/applications/{}", app_id))
        .await;
    del_res.assert_status(StatusCode::CONFLICT);
    let err: serde_json::Value = del_res.json();
    assert!(
        err["error"].as_str().unwrap().contains("申报材料"),
        "should mention dependent materials"
    );
}

#[tokio::test]
async fn s07_application_required_fields() {
    let app = common::TestApp::new().await;

    // Missing required fields (title, type, application_no)
    let res = app
        .post("/api/v1/applications")
        .json(&json!({"notes": "no required fields"}))
        .await;
    res.assert_status(StatusCode::BAD_REQUEST);
    let body: serde_json::Value = res.json();
    assert!(
        body["error"].as_str().unwrap().contains("required"),
        "should mention missing required field"
    );
}

#[tokio::test]
async fn s07_application_pagination() {
    let app = common::TestApp::new().await;

    for i in 1..=4 {
        app.post("/api/v1/applications")
            .json(&json!({
                "title": format!("申报 #{}", i),
                "type": "补贴",
                "application_no": format!("APP-{:03}", i)
            }))
            .await
            .assert_status_ok();
    }

    let res = app.get("/api/v1/applications?page=1&page_size=2").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["count"], 4);
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn s07_application_tenant_isolation() {
    let app_a = common::TestApp::new().await;
    let app_b = app_a.create_second_tenant().await;

    app_a
        .post("/api/v1/applications")
        .json(&json!({
            "title": "A的申报",
            "type": "税收优惠",
            "application_no": "PRIV-001"
        }))
        .await
        .assert_status_ok();

    let res = app_b.get("/api/v1/applications").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["count"], 0, "tenant B sees 0 of A's applications");
}

#[tokio::test]
async fn s07_project_crud_and_milestones() {
    let app = common::TestApp::new().await;

    // Create project (generic handler, only name required)
    let proj_res = app
        .post("/api/v1/projects")
        .json(&json!({
            "name": "产品研发 v2"
        }))
        .await;
    proj_res.assert_status_ok();
    let proj: serde_json::Value = proj_res.json();
    let proj_id = proj["id"].as_str().unwrap();

    // Create milestone (requires title, not name; no status column in milestones)
    let ms_res = app
        .post("/api/v1/milestones")
        .json(&json!({
            "project_id": proj_id,
            "title": "Alpha 发布",
            "milestone_date": "2026-03-31",
            "importance": "high"
        }))
        .await;
    ms_res.assert_status_ok();

    // List projects
    let list = app.get("/api/v1/projects").await;
    list.assert_status_ok();
    let body: serde_json::Value = list.json();
    assert_eq!(body["count"], 1);
}

#[tokio::test]
async fn s07_application_not_in_export_whitelist() {
    let app = common::TestApp::new().await;

    // applications is not in the CSV export whitelist
    let csv_res = app.get("/api/v1/export/applications").await;
    csv_res.assert_status(StatusCode::BAD_REQUEST);
    let body: serde_json::Value = csv_res.json();
    assert!(body["error"].as_str().unwrap().contains("not exportable"));
}
