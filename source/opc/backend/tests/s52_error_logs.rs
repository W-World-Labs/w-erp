/// Scenario 52: 🛡️ Error Logs (前端错误自动上报)
/// Tests the error reporting pipeline CRUD
use crate::common;
use serde_json::json;

#[tokio::test]
async fn s52_error_log_create_and_list() {
    let app = common::TestApp::new().await;

    // Create an error log entry (simulating frontend error report)
    let res = app.post("/api/v1/error_logs")
        .json(&json!({
            "error_type": "render_crash",
            "error_message": "Cannot read property 'toLocaleString' of null",
            "stack_trace": "TypeError: Cannot read property...\n    at ReconciliationCenter.tsx:267",
            "component_stack": "\n    at ReconciliationCenter\n    at PageErrorBoundary",
            "page_url": "http://localhost:8987/reconciliation",
            "user_agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)"
        }))
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    let id = body["id"].as_str().unwrap().to_string();

    // List error logs
    let list_res = app.get("/api/v1/error_logs").await;
    list_res.assert_status_ok();
    let list = list_res.json::<serde_json::Value>();
    assert!(list["count"].as_i64().unwrap() >= 1);

    // Get by ID — verify fields
    let get_res = app.get(&format!("/api/v1/error_logs/{}", id)).await;
    get_res.assert_status_ok();
    let detail = get_res.json::<serde_json::Value>();
    assert_eq!(detail["error_type"].as_str().unwrap(), "render_crash");
    assert_eq!(
        detail["error_message"].as_str().unwrap(),
        "Cannot read property 'toLocaleString' of null"
    );
    assert!(
        detail["page_url"]
            .as_str()
            .unwrap()
            .contains("reconciliation")
    );

    // Delete
    let del = app.delete(&format!("/api/v1/error_logs/{}", id)).await;
    assert!(del.status_code().is_success());
}

#[tokio::test]
async fn s52_error_log_api_error_type() {
    let app = common::TestApp::new().await;

    let res = app.post("/api/v1/error_logs")
        .json(&json!({
            "error_type": "api_error",
            "error_message": "HTTP 500: Internal Server Error",
            "metadata": "{\"endpoint\":\"/api/v1/tax_calendar/seed\",\"method\":\"POST\",\"status\":500}"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Verify via GET
    let get_res = app.get(&format!("/api/v1/error_logs/{}", id)).await;
    get_res.assert_status_ok();
    let detail = get_res.json::<serde_json::Value>();
    assert_eq!(detail["error_type"].as_str().unwrap(), "api_error");
}

#[tokio::test]
async fn s52_error_log_list_returns_backend_display_fields_and_filtering() {
    let app = common::TestApp::new().await;

    app.post("/api/v1/error_logs")
        .json(&json!({
            "error_type": "api_error",
            "error_message": "HTTP 500: Internal Server Error",
            "page_url": "http://localhost:8987/settings?tab=logs",
            "metadata": "{\"endpoint\":\"/api/v1/error_logs\",\"method\":\"POST\",\"status\":500}"
        }))
        .await
        .assert_status_ok();
    app.post("/api/v1/error_logs")
        .json(&json!({
            "error_type": "render_crash",
            "error_message": "render failed"
        }))
        .await
        .assert_status_ok();

    let list = app.get("/api/v1/error_logs?error_type=api_error").await;
    list.assert_status_ok();
    let body = list.json::<serde_json::Value>();
    assert_eq!(body["count"].as_i64().unwrap(), 1);
    assert_eq!(body["type_counts"]["api_error"].as_i64().unwrap(), 1);
    assert_eq!(body["type_counts"]["render_crash"].as_i64().unwrap(), 1);
    assert_eq!(body["type_options"][0]["value"], "all");
    assert_eq!(body["type_options"][0]["label"], "全部 (2)");
    assert!(
        body["type_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "api_error" && option["label"] == "API 错误 (1)")
    );

    let rows = body["data"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row["error_type"], "api_error");
    assert_eq!(row["error_type_label"], "API 错误");
    assert_eq!(row["error_type_badge"]["icon"], "Globe");
    assert_eq!(row["error_type_badge"]["variant"], "default");
    assert_eq!(row["page_path"], "/settings");
    assert!(row["created_at_label"].as_str().unwrap().len() >= 10);
    assert_eq!(row["metadata_fields"]["endpoint"], "/api/v1/error_logs");
    assert_eq!(row["metadata_fields"]["method"], "POST");
    assert_eq!(row["metadata_fields"]["status"], "500");
    let delete_action = &row["delete_action"];
    assert_eq!(
        delete_action["endpoint"],
        format!("/api/v1/error_logs/{}", row["id"].as_str().unwrap())
    );
    assert_eq!(delete_action["success_message"], "错误日志已删除");
    assert!(
        delete_action["confirm_message"]
            .as_str()
            .unwrap()
            .contains("API 错误")
    );
}

#[tokio::test]
async fn s52_error_log_requires_fields() {
    let app = common::TestApp::new().await;

    // Missing error_message should fail
    let res = app
        .post("/api/v1/error_logs")
        .json(&json!({
            "error_type": "render_crash"
        }))
        .await;
    res.assert_status(axum::http::StatusCode::BAD_REQUEST);

    // Missing error_type should fail
    let res2 = app
        .post("/api/v1/error_logs")
        .json(&json!({
            "error_message": "something broke"
        }))
        .await;
    res2.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn s52_error_log_tenant_isolation() {
    let app = common::TestApp::new().await;
    let app_b = app.create_second_tenant().await;

    // A creates error log
    let res = app
        .post("/api/v1/error_logs")
        .json(&json!({
            "error_type": "unhandled_exception",
            "error_message": "tenant A crash"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // B should not see it
    let b_res = app_b.get(&format!("/api/v1/error_logs/{}", id)).await;
    assert_eq!(b_res.status_code(), 404);
}
