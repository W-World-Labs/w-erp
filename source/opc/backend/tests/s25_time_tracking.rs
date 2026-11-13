/// Scenario 25: ⏱️ Time Tracking (时间追踪)
/// Tests time entries, project/client linking, billable tracking
use crate::common;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn s25_time_entry_crud() {
    let app = common::TestApp::new().await;

    // Create a time entry
    let res = app
        .post("/api/v1/time_entries")
        .json(&json!({
            "description": "前端页面开发",
            "date": "2024-03-15",
            "duration_minutes": 120,
            "is_billable": 1,
            "hourly_rate": 200,
            "amount": 400,
            "status": "logged"
        }))
        .await;
    res.assert_status_ok();
    let entry_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Get
    let get_res = app.get(&format!("/api/v1/time_entries/{}", entry_id)).await;
    let entry = get_res.json::<serde_json::Value>();
    assert_eq!(entry["description"].as_str().unwrap(), "前端页面开发");
    assert_eq!(entry["duration_minutes"].as_i64().unwrap(), 120);

    // Update
    app.put(&format!("/api/v1/time_entries/{}", entry_id))
        .json(&json!({ "duration_minutes": 150, "amount": 500 }))
        .await
        .assert_status_ok();

    // List
    let list_res = app.get("/api/v1/time_entries").await;
    list_res.assert_status_ok();
    assert!(
        list_res.json::<serde_json::Value>()["count"]
            .as_i64()
            .unwrap()
            >= 1
    );
}

#[tokio::test]
async fn s25_time_entry_with_project_and_client() {
    let app = common::TestApp::new().await;

    // Create project
    let proj_res = app
        .post("/api/v1/projects")
        .json(&json!({ "name": "ERP开发", "project_type": "development", "status": "active" }))
        .await;
    proj_res.assert_status_ok();
    let project_id = proj_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create client
    let client_res = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "客户A" }))
        .await;
    client_res.assert_status_ok();
    let client_id = client_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create time entry with FK references
    let res = app
        .post("/api/v1/time_entries")
        .json(&json!({
            "description": "需求分析",
            "date": "2024-03-16",
            "project_id": project_id,
            "client_id": client_id,
            "duration_minutes": 60,
            "is_billable": 1,
            "hourly_rate": 300
        }))
        .await;
    res.assert_status_ok();
    let entry_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Verify FK references via GET
    let get_res = app.get(&format!("/api/v1/time_entries/{}", entry_id)).await;
    let entry = get_res.json::<serde_json::Value>();
    assert_eq!(entry["project_id"].as_str().unwrap(), project_id);
    assert_eq!(entry["client_id"].as_str().unwrap(), client_id);
}

#[tokio::test]
async fn s25_billable_vs_nonbillable() {
    let app = common::TestApp::new().await;

    // Billable entry
    app.post("/api/v1/time_entries")
        .json(&json!({
            "description": "客户会议",
            "date": "2024-03-17",
            "duration_minutes": 60,
            "is_billable": 1
        }))
        .await
        .assert_status_ok();

    // Non-billable entry
    app.post("/api/v1/time_entries")
        .json(&json!({
            "description": "内部培训",
            "date": "2024-03-17",
            "duration_minutes": 90,
            "is_billable": 0
        }))
        .await
        .assert_status_ok();

    let list_res = app.get("/api/v1/time_entries").await;
    assert!(
        list_res.json::<serde_json::Value>()["count"]
            .as_i64()
            .unwrap()
            >= 2
    );
}

#[tokio::test]
async fn s25_time_entry_delete() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/time_entries")
        .json(&json!({
            "description": "要删除的工时",
            "date": "2024-03-20",
            "duration_minutes": 30,
            "is_billable": 0
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let del = app.delete(&format!("/api/v1/time_entries/{}", id)).await;
    assert!(
        del.status_code().is_success(),
        "Time entry delete should succeed"
    );

    // Verify deleted
    let get = app.get(&format!("/api/v1/time_entries/{}", id)).await;
    assert_eq!(get.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn s25_zero_duration_boundary() {
    let app = common::TestApp::new().await;

    // 0 minutes should be valid (e.g., canceled entry)
    let res = app
        .post("/api/v1/time_entries")
        .json(&json!({
            "description": "零时长登记",
            "date": "2024-03-20",
            "duration_minutes": 0,
            "is_billable": 0,
            "amount": 0.0
        }))
        .await;
    res.assert_status_ok();
    let data: serde_json::Value = res.json();
    // Zero duration should be accepted and stored
    assert_eq!(data["duration_minutes"].as_i64().unwrap_or(0), 0);
}
