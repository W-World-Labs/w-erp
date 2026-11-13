/// Scenario 132: backend-owned agent and MCP gateway.
/// External agents should discover, inspect, and execute scoped business tools
/// without relying on frontend UI automation.
use crate::common;

use axum::http::{HeaderName, HeaderValue, StatusCode};
use base64::{Engine as _, engine::general_purpose};
use common::TestApp;
use serde_json::{Value, json};
use std::collections::HashSet;

fn api_key_header(key: &str) -> (HeaderName, HeaderValue) {
    (
        HeaderName::from_static("x-api-key"),
        HeaderValue::from_str(key).unwrap(),
    )
}

#[tokio::test]
async fn s132_agent_tools_are_discoverable_with_schema_scope_and_risk() {
    let app = TestApp::new().await;

    let res = app.get("/api/v1/agent/tools").await;
    res.assert_status_ok();
    let body = res.json::<Value>();
    let tools = body["tools"].as_array().expect("tools array");
    let names: Vec<&str> = tools
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect();

    assert!(names.contains(&"assets.list"));
    assert!(names.contains(&"branches.create"));
    assert!(names.contains(&"payments.approve"));
    assert!(names.contains(&"files.upload"));
    assert!(names.contains(&"onboarding.progress"));
    assert!(names.contains(&"onboarding.autofill"));

    let domains: HashSet<&str> = tools
        .iter()
        .filter_map(|tool| tool["domain"].as_str())
        .collect();
    for domain in [
        "assets",
        "branches",
        "payments",
        "files",
        "onboarding",
        "invoices",
        "contracts",
        "projects",
        "vendors",
        "clients",
        "employees",
        "budgets",
        "tax",
        "audit",
        "settings",
    ] {
        assert!(domains.contains(domain), "missing agent domain {domain}");
    }

    let branch_tool = tools
        .iter()
        .find(|tool| tool["name"] == "branches.create")
        .expect("branches.create tool");
    assert_eq!(branch_tool["required_scope"], "branches:write");
    assert_eq!(branch_tool["risk_level"], "medium");
    assert_eq!(branch_tool["input_schema"]["type"], "object");

    let schema = app.get("/api/v1/agent/tools/branches.create/schema").await;
    schema.assert_status_ok();
    let body = schema.json::<Value>();
    assert_eq!(body["tool"]["name"], "branches.create");
    assert_eq!(body["permissions"]["required_scope"], "branches:write");
    assert_eq!(body["risk_level"], "medium");
    assert_eq!(
        body["input_schema"]["required"].as_array().unwrap(),
        &vec![json!("branch_name")]
    );
}

#[tokio::test]
async fn s132_agent_execute_runs_tool_and_writes_audit_log() {
    let app = TestApp::new().await;

    let res = app
        .post("/api/v1/agent/execute")
        .json(&json!({
            "tool_name": "branches.create",
            "agent_name": "hermas-test",
            "arguments": {
                "branch_name": "Agent Branch",
                "branch_type": "branch",
                "contact_person": "Agent Operator"
            }
        }))
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();
    assert_eq!(body["tool_name"], "branches.create");
    assert_eq!(body["status"], "success");
    let branch_id = body["result"]["id"].as_str().expect("created branch id");

    let branch_name: String =
        sqlx::query_scalar("SELECT branch_name FROM branches WHERE id = ? AND company_id = ?")
            .bind(branch_id)
            .bind(&app.company_id)
            .fetch_one(&app.pool)
            .await
            .expect("created branch row");
    assert_eq!(branch_name, "Agent Branch");

    let audit: (String, String, String, Option<String>) = sqlx::query_as(
        "SELECT agent_name, tool_name, status, api_key_id FROM agent_audit_logs WHERE company_id = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&app.company_id)
    .fetch_one(&app.pool)
    .await
    .expect("agent audit row");

    assert_eq!(audit.0, "hermas-test");
    assert_eq!(audit.1, "branches.create");
    assert_eq!(audit.2, "success");
    assert!(audit.3.is_none());
}

#[tokio::test]
async fn s132_agent_can_upload_files_without_frontend_multipart() {
    let app = TestApp::new().await;
    let content = b"agent-generated purchase evidence";

    let res = app
        .post("/api/v1/agent/execute")
        .json(&json!({
            "tool_name": "files.upload",
            "agent_name": "openclaw-file-agent",
            "arguments": {
                "original_name": "evidence.txt",
                "content_type": "text/plain",
                "content_base64": general_purpose::STANDARD.encode(content),
                "entity_type": "branches",
                "entity_id": app.company_id,
                "description": "uploaded through agent gateway"
            }
        }))
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();
    assert_eq!(body["tool_name"], "files.upload");
    assert_eq!(body["status"], "success");
    assert_eq!(body["result"]["original_name"], "evidence.txt");
    assert_eq!(body["result"]["size"], content.len() as i64);

    let file_row: (String, i64, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT original_name, size, entity_type, description FROM files WHERE company_id = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&app.company_id)
    .fetch_one(&app.pool)
    .await
    .expect("uploaded file row");

    assert_eq!(file_row.0, "evidence.txt");
    assert_eq!(file_row.1, content.len() as i64);
    assert_eq!(file_row.2.as_deref(), Some("branches"));
    assert_eq!(
        file_row.3.as_deref(),
        Some("uploaded through agent gateway")
    );
}

#[tokio::test]
async fn s132_agent_can_complete_onboarding_modules_from_backend() {
    let app = TestApp::new().await;

    let before = app.get("/api/v1/onboarding/progress").await;
    before.assert_status_ok();
    let before = before.json::<Value>();
    assert!(before["summary"]["completed_count"].as_i64().unwrap() < 15);

    let res = app
        .post("/api/v1/agent/execute")
        .json(&json!({
            "tool_name": "onboarding.autofill",
            "agent_name": "hermas-onboarding-agent",
            "arguments": {
                "mode": "minimal_verified_defaults"
            }
        }))
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();
    assert_eq!(body["tool_name"], "onboarding.autofill");
    assert_eq!(body["status"], "success");
    assert_eq!(body["result"]["summary"]["completed_count"], 15);
    assert_eq!(
        body["result"]["summary"]["completed_label"],
        "已完成 15/15 个模块"
    );

    let after = app.get("/api/v1/onboarding/progress").await;
    after.assert_status_ok();
    let after = after.json::<Value>();
    assert_eq!(after["summary"]["all_completed"], true);
    assert_eq!(after["summary"]["completed_count"], 15);
}

#[tokio::test]
async fn s132_api_key_scopes_gate_http_and_agent_execution() {
    let app = TestApp::new().await;

    let key_res = app
        .post("/api/v1/api-keys")
        .json(&json!({ "name": "Read Only Agent", "scopes": "assets:read" }))
        .await;
    key_res.assert_status(StatusCode::CREATED);
    let key = key_res.json::<Value>()["key"].as_str().unwrap().to_string();

    let (name, value) = api_key_header(&key);
    let read_res = app
        .server
        .get("/api/v1/assets")
        .add_header(name, value)
        .await;
    read_res.assert_status_ok();

    let (name, value) = api_key_header(&key);
    let write_res = app
        .server
        .post("/api/v1/branches")
        .add_header(name, value)
        .json(&json!({ "branch_name": "Blocked Branch" }))
        .await;
    write_res.assert_status(StatusCode::FORBIDDEN);

    let (name, value) = api_key_header(&key);
    let agent_res = app
        .server
        .post("/api/v1/agent/execute")
        .add_header(name, value)
        .json(&json!({
            "tool_name": "branches.create",
            "agent_name": "openclaw-test",
            "arguments": { "branch_name": "Blocked Agent Branch" }
        }))
        .await;
    agent_res.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn s132_mcp_lists_and_calls_the_same_backend_tools() {
    let app = TestApp::new().await;

    let list = app
        .post("/api/v1/mcp")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        }))
        .await;
    list.assert_status_ok();
    let body = list.json::<Value>();
    assert_eq!(body["jsonrpc"], "2.0");
    assert_eq!(body["id"], 1);
    let tools = body["result"]["tools"].as_array().expect("mcp tools");
    assert!(tools.iter().any(|tool| tool["name"] == "branches.create"));
    assert!(tools.iter().any(|tool| tool["name"] == "files.upload"));
    assert!(
        tools
            .iter()
            .any(|tool| tool["name"] == "onboarding.autofill")
    );
    assert!(tools.iter().all(|tool| tool["inputSchema"].is_object()));

    let call = app
        .post("/api/v1/mcp")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "branches.create",
                "arguments": {
                    "branch_name": "MCP Branch",
                    "branch_type": "branch"
                }
            }
        }))
        .await;
    call.assert_status_ok();
    let body = call.json::<Value>();
    assert_eq!(body["jsonrpc"], "2.0");
    assert_eq!(body["id"], 2);
    assert_eq!(body["result"]["isError"], false);
    assert_eq!(
        body["result"]["structuredContent"]["tool_name"],
        "branches.create"
    );
    assert_eq!(body["result"]["structuredContent"]["status"], "success");
    assert!(
        body["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("branches.create")
    );
}
