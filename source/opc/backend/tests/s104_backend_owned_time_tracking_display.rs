use crate::common;

use chrono::Utc;
use serde_json::{Value, json};

async fn seed_project(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO projects (id, company_id, name, status)
        VALUES ('project-needle', ?, 'Backend Project', 'active')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed project");
}

async fn seed_client(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO clients (id, company_id, name, client_type)
        VALUES ('client-needle', ?, 'Backend Client', 'active')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed client");
}

#[tokio::test]
async fn s104_time_overview_returns_entries_options_labels_actions_and_permissions() {
    let app = common::TestApp::new().await;
    seed_project(&app).await;
    seed_client(&app).await;
    let today = Utc::now().format("%Y-%m-%d").to_string();

    sqlx::query(
        r#"
        INSERT INTO time_entries (
            id, company_id, project_id, client_id, description, date,
            duration_minutes, is_billable, hourly_rate, amount, status
        )
        VALUES (
            'time-needle', ?, 'project-needle', 'client-needle', 'Backend owned work', ?,
            90, 1, 200.0, 300.0, 'logged'
        )
        "#,
    )
    .bind(&app.company_id)
    .bind(&today)
    .execute(&app.pool)
    .await
    .expect("seed time entry");

    let res = app.get("/api/v1/time_entries/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["today_minutes_label"], "1h 30m");
    assert_eq!(body["stats"]["billable_amount_label"], "¥300");
    assert_eq!(body["create_action"]["method"], "POST");
    assert_eq!(body["create_action"]["endpoint"], "/api/v1/time_entries");
    assert_eq!(body["create_action"]["label"], "手动记录");
    assert_eq!(body["create_action"]["success_message"], "工时已记录");
    assert_eq!(body["invoice_action"]["can_generate"], true);
    assert_eq!(body["invoice_action"]["label"], "生成发票 (1)");
    assert_eq!(body["invoice_action"]["action"]["method"], "POST");
    assert_eq!(
        body["invoice_action"]["action"]["endpoint"],
        "/api/v1/time_entries/invoice"
    );
    assert_eq!(body["invoice_action"]["action"]["label"], "生成发票 (1)");
    assert_eq!(
        body["invoice_action"]["action"]["success_message"],
        "工时发票已生成"
    );
    assert_eq!(body["active_timer"]["start_action"]["method"], "POST");
    assert_eq!(
        body["active_timer"]["start_action"]["endpoint"],
        "/api/v1/time_entries/timer/start"
    );
    assert!(body["active_timer"]["stop_action"].is_null());
    assert_eq!(body["permissions"]["can_delete"], true);

    let entry = &body["entries"].as_array().unwrap()[0];
    assert_eq!(entry["description"], "Backend owned work");
    assert_eq!(entry["project_name"], "Backend Project");
    assert_eq!(entry["client_name"], "Backend Client");
    assert_eq!(entry["date_label"], today);
    assert_eq!(entry["duration_label"], "1h 30m");
    assert_eq!(entry["billable_badge"]["label"], "可计费");
    assert_eq!(entry["amount_label"], "¥300");
    assert_eq!(entry["status_badge"]["label"], "已记录");
    assert_eq!(entry["can_delete"], true);
    assert_eq!(entry["delete_action"]["method"], "DELETE");
    assert_eq!(
        entry["delete_action"]["endpoint"],
        "/api/v1/time_entries/time-needle"
    );
    assert_eq!(
        entry["delete_action"]["confirm_message"],
        "确定要删除工时记录「Backend owned work」吗？"
    );
    assert_eq!(entry["delete_action"]["success_title"], "删除成功");
    assert_eq!(entry["delete_action"]["success_message"], "工时记录已删除");

    assert!(
        body["project_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["label"] == "Backend Project")
    );
    assert!(
        body["client_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["label"] == "Backend Client")
    );
    assert_eq!(body["billable_options"][0]["value"], "1");
    assert_eq!(body["billable_options"][0]["label"], "可计费");
    assert_eq!(body["billable_options"][1]["value"], "0");
}

#[tokio::test]
async fn s104_time_entry_create_derives_timer_duration_date_and_amount_on_backend() {
    let app = common::TestApp::new().await;

    let create = app
        .post("/api/v1/time_entries")
        .json(&json!({
            "description": "Timer payload",
            "start_time": "2026-05-09T08:00:00Z",
            "end_time": "2026-05-09T09:30:00Z",
            "hourly_rate": 200.0,
            "is_billable": 1
        }))
        .await;
    create.assert_status_ok();
    let id = create.json::<Value>()["id"].as_str().unwrap().to_string();

    let get = app.get(&format!("/api/v1/time_entries/{}", id)).await;
    get.assert_status_ok();
    let entry = get.json::<Value>();

    assert_eq!(entry["date"], "2026-05-09");
    assert_eq!(entry["duration_minutes"], 90);
    assert_eq!(entry["amount"], 300.0);
}

#[tokio::test]
async fn s104_time_overview_marks_invoiced_entries_as_already_billed() {
    let app = common::TestApp::new().await;
    seed_project(&app).await;
    seed_client(&app).await;

    sqlx::query(
        r#"
        INSERT INTO time_entries (
            id, company_id, project_id, client_id, description, date,
            duration_minutes, is_billable, hourly_rate, amount, status, invoice_id
        )
        VALUES (
            'time-invoiced-needle', ?, 'project-needle', 'client-needle',
            'Already invoiced delivery work', '2026-05-24',
            360, 1, 950.0, 5700.0, 'invoiced', 'invoice-needle'
        )
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed invoiced time entry");

    let res = app.get("/api/v1/time_entries/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["unbilled_count"], 0);
    assert_eq!(body["invoice_action"]["can_generate"], false);
    assert!(body["invoice_action"]["action"].is_null());

    let entry = &body["entries"].as_array().unwrap()[0];
    assert_eq!(entry["status"], "invoiced");
    assert_eq!(entry["status_badge"]["label"], "已开票");
    assert_eq!(entry["billable_badge"]["label"], "已开票");
    assert_eq!(entry["billable_badge"]["variant"], "outline");
}

#[tokio::test]
async fn s104_time_timer_start_stop_is_owned_by_backend() {
    let app = common::TestApp::new().await;
    seed_project(&app).await;
    seed_client(&app).await;
    let today = Utc::now().format("%Y-%m-%d").to_string();

    let started = app
        .post("/api/v1/time_entries/timer/start")
        .json(&json!({
            "description": "Backend timer session",
            "project_id": "project-needle",
            "client_id": "client-needle",
            "hourly_rate": "120.00",
            "is_billable": 1
        }))
        .await;
    started.assert_status_ok();
    let started = started.json::<Value>();
    let timer_id = started["active_timer"]["id"].as_str().unwrap().to_string();
    assert_eq!(started["active_timer"]["is_running"], true);
    assert_eq!(
        started["active_timer"]["description"],
        "Backend timer session"
    );
    assert_eq!(started["active_timer"]["elapsed_label"], "0m");

    let duplicate = app
        .post("/api/v1/time_entries/timer/start")
        .json(&json!({ "description": "Second timer" }))
        .await;
    duplicate.assert_status(axum::http::StatusCode::CONFLICT);

    let overview = app.get("/api/v1/time_entries/overview").await;
    overview.assert_status_ok();
    let overview = overview.json::<Value>();
    assert_eq!(overview["active_timer"]["id"], timer_id);
    assert_eq!(overview["active_timer"]["is_running"], true);
    assert_eq!(overview["active_timer"]["project_name"], "Backend Project");
    assert_eq!(overview["active_timer"]["client_name"], "Backend Client");
    assert!(overview["active_timer"]["start_action"].is_null());
    assert_eq!(
        overview["active_timer"]["stop_action"]["endpoint"],
        "/api/v1/time_entries/timer/stop"
    );

    let stopped = app
        .post("/api/v1/time_entries/timer/stop")
        .json(&json!({}))
        .await;
    stopped.assert_status_ok();
    let stopped = stopped.json::<Value>();
    let entry = &stopped["entry"];
    assert_eq!(entry["id"], timer_id);
    assert_eq!(entry["status"], "logged");
    assert_eq!(entry["date"], today);
    assert!(entry["duration_minutes"].as_i64().unwrap() >= 1);
    assert!(entry["amount"].as_f64().unwrap() >= 2.0);

    let overview = app.get("/api/v1/time_entries/overview").await;
    overview.assert_status_ok();
    let overview = overview.json::<Value>();
    assert_eq!(overview["active_timer"]["is_running"], false);
    assert_eq!(
        overview["active_timer"]["start_action"]["endpoint"],
        "/api/v1/time_entries/timer/start"
    );
    assert!(overview["active_timer"]["stop_action"].is_null());
}

#[tokio::test]
async fn s104_time_entry_delete_requires_backend_permission() {
    let app = common::TestApp::new().await;
    let today = Utc::now().format("%Y-%m-%d").to_string();

    sqlx::query(
        r#"
        INSERT INTO time_entries (id, company_id, description, date, duration_minutes, status)
        VALUES ('time-delete-guard', ?, 'Delete guard', ?, 30, 'logged')
        "#,
    )
    .bind(&app.company_id)
    .bind(&today)
    .execute(&app.pool)
    .await
    .expect("seed time entry");
    sqlx::query("UPDATE company_members SET role = 'member' WHERE user_id = ? AND company_id = ?")
        .bind(&app.user_id)
        .bind(&app.company_id)
        .execute(&app.pool)
        .await
        .expect("downgrade member");

    let delete = app.delete("/api/v1/time_entries/time-delete-guard").await;
    delete.assert_status_forbidden();
}
