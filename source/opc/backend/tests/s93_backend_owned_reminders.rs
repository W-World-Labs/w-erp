use crate::common;

use chrono::{Duration, Utc};
use serde_json::Value;

struct ReminderSeed<'a> {
    id: &'a str,
    title: &'a str,
    status: &'a str,
    reminder_type: &'a str,
    priority: &'a str,
    remind_date: String,
}

async fn seed_reminder(app: &common::TestApp, seed: ReminderSeed<'_>) {
    sqlx::query(
        r#"
        INSERT INTO reminders (
            id, company_id, title, description, reminder_type,
            priority, remind_date, status
        )
        VALUES (?, ?, ?, '提醒说明', ?, ?, ?, ?)
        "#,
    )
    .bind(seed.id)
    .bind(&app.company_id)
    .bind(seed.title)
    .bind(seed.reminder_type)
    .bind(seed.priority)
    .bind(seed.remind_date)
    .bind(seed.status)
    .execute(&app.pool)
    .await
    .expect("seed reminder");
}

async fn seed_expiring_asset(app: &common::TestApp, id: &str, name: &str, expiry_date: String) {
    sqlx::query(
        r#"
        INSERT INTO assets (id, company_id, name, asset_type, status, expiry_date)
        VALUES (?, ?, ?, 'virtual', 'active', ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
    .bind(expiry_date)
    .execute(&app.pool)
    .await
    .expect("seed expiring asset");
}

async fn seed_expiring_contract(app: &common::TestApp, id: &str, name: &str, end_date: String) {
    sqlx::query(
        r#"
        INSERT INTO contracts (id, company_id, contract_name, status, end_date)
        VALUES (?, ?, ?, 'active', ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
    .bind(end_date)
    .execute(&app.pool)
    .await
    .expect("seed expiring contract");
}

async fn seed_expiring_license(app: &common::TestApp, id: &str, name: &str, expiry_date: String) {
    sqlx::query(
        r#"
        INSERT INTO licenses (id, company_id, license_name, license_type, status, expiry_date)
        VALUES (?, ?, ?, 'business_license', 'valid', ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
    .bind(expiry_date)
    .execute(&app.pool)
    .await
    .expect("seed expiring license");
}

async fn seed_expiring_rent(
    app: &common::TestApp,
    id: &str,
    name: &str,
    contract_end_date: String,
) {
    sqlx::query(
        r#"
        INSERT INTO rent_records (
            id, company_id, location_name, property_type, monthly_rent, status, contract_end_date
        )
        VALUES (?, ?, ?, 'office', 3000, 'active', ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
    .bind(contract_end_date)
    .execute(&app.pool)
    .await
    .expect("seed expiring rent");
}

#[tokio::test]
async fn s93_reminders_overview_groups_dates_and_badges_on_backend() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;
    let today = Utc::now().date_naive();

    seed_reminder(
        &app,
        ReminderSeed {
            id: "reminder-overdue",
            title: "逾期合同",
            status: "pending",
            reminder_type: "contract",
            priority: "urgent",
            remind_date: (today - Duration::days(1)).to_string(),
        },
    )
    .await;
    seed_reminder(
        &app,
        ReminderSeed {
            id: "reminder-upcoming",
            title: "即将到期证照",
            status: "pending",
            reminder_type: "license",
            priority: "high",
            remind_date: (today + Duration::days(1)).to_string(),
        },
    )
    .await;
    seed_reminder(
        &app,
        ReminderSeed {
            id: "reminder-completed",
            title: "已完成提醒",
            status: "completed",
            reminder_type: "asset",
            priority: "normal",
            remind_date: today.to_string(),
        },
    )
    .await;
    seed_reminder(
        &app,
        ReminderSeed {
            id: "reminder-dismissed",
            title: "已忽略提醒",
            status: "dismissed",
            reminder_type: "invoice",
            priority: "low",
            remind_date: today.to_string(),
        },
    )
    .await;
    seed_reminder(
        &other,
        ReminderSeed {
            id: "reminder-other",
            title: "其他租户提醒",
            status: "pending",
            reminder_type: "contract",
            priority: "urgent",
            remind_date: (today - Duration::days(1)).to_string(),
        },
    )
    .await;

    let res = app.get("/api/v1/reminders/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["pending"].as_i64().unwrap(), 2);
    assert_eq!(body["stats"]["completed"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["dismissed"].as_i64().unwrap(), 1);
    assert_eq!(body["pending_reminders"].as_array().unwrap().len(), 2);
    assert_eq!(body["history_reminders"].as_array().unwrap().len(), 2);

    let overdue = body["pending_reminders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|reminder| reminder["id"] == "reminder-overdue")
        .expect("overdue reminder");
    assert_eq!(overdue["date_state"], "overdue");
    assert_eq!(overdue["date_badge"]["label"], "已逾期");
    assert_eq!(overdue["status_badge"]["label"], "待处理");
    assert_eq!(overdue["priority_badge"]["label"], "紧急");
    assert_eq!(overdue["type_label"], "合同");
    assert_eq!(overdue["complete_action"]["method"], "POST");
    assert_eq!(
        overdue["complete_action"]["endpoint"],
        "/api/v1/reminders/reminder-overdue/complete"
    );
    assert_eq!(overdue["complete_action"]["label"], "完成");
    assert_eq!(
        overdue["complete_action"]["success_message"],
        "提醒已标记完成"
    );
    assert_eq!(overdue["dismiss_action"]["method"], "POST");
    assert_eq!(
        overdue["dismiss_action"]["endpoint"],
        "/api/v1/reminders/reminder-overdue/dismiss"
    );
    assert_eq!(overdue["dismiss_action"]["label"], "忽略");
    assert_eq!(overdue["dismiss_action"]["success_message"], "提醒已忽略");

    let upcoming = body["pending_reminders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|reminder| reminder["id"] == "reminder-upcoming")
        .expect("upcoming reminder");
    assert_eq!(upcoming["date_state"], "upcoming");
    assert_eq!(upcoming["date_badge"]["label"], "即将到期");

    assert!(
        body["pending_reminders"]
            .as_array()
            .unwrap()
            .iter()
            .all(|reminder| reminder["id"] != "reminder-other")
    );
}

#[tokio::test]
async fn s93_reminder_actions_are_backend_owned() {
    let app = common::TestApp::new().await;
    let today = Utc::now().date_naive().to_string();

    seed_reminder(
        &app,
        ReminderSeed {
            id: "reminder-complete-action",
            title: "完成动作",
            status: "pending",
            reminder_type: "contract",
            priority: "normal",
            remind_date: today.clone(),
        },
    )
    .await;
    seed_reminder(
        &app,
        ReminderSeed {
            id: "reminder-dismiss-action",
            title: "忽略动作",
            status: "pending",
            reminder_type: "license",
            priority: "normal",
            remind_date: today,
        },
    )
    .await;

    let complete = app
        .post("/api/v1/reminders/reminder-complete-action/complete")
        .json(&serde_json::json!({}))
        .await;
    complete.assert_status_ok();
    let complete_body = complete.json::<Value>();
    assert_eq!(complete_body["status"], "completed");
    assert_eq!(complete_body["is_completed"], true);
    assert_eq!(complete_body["status_badge"]["label"], "已完成");
    assert!(complete_body["complete_action"].is_null());
    assert!(complete_body["dismiss_action"].is_null());

    let dismiss = app
        .post("/api/v1/reminders/reminder-dismiss-action/dismiss")
        .json(&serde_json::json!({}))
        .await;
    dismiss.assert_status_ok();
    let dismiss_body = dismiss.json::<Value>();
    assert_eq!(dismiss_body["status"], "dismissed");
    assert_eq!(dismiss_body["is_completed"], false);
    assert_eq!(dismiss_body["status_badge"]["label"], "已忽略");
    assert!(dismiss_body["complete_action"].is_null());
    assert!(dismiss_body["dismiss_action"].is_null());
}

#[tokio::test]
async fn s93_asset_expiry_check_creates_reminders_on_backend() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;
    let soon = (Utc::now().date_naive() + Duration::days(10)).to_string();

    seed_expiring_asset(&app, "asset-expiring", "Backend 域名", soon.clone()).await;
    seed_expiring_asset(&other, "asset-other-expiring", "其他租户域名", soon).await;

    let res = app
        .post("/api/v1/reminders/check_asset_expiry")
        .json(&serde_json::json!({}))
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["reminders_created"].as_i64().unwrap(), 1);
    assert_eq!(
        body["created_reminders"].as_array().unwrap()[0]["entity_id"],
        "asset-expiring"
    );
    assert_eq!(
        body["created_reminders"].as_array().unwrap()[0]["type_label"],
        "资产到期"
    );

    let overview = app.get("/api/v1/reminders/overview").await;
    overview.assert_status_ok();
    let overview = overview.json::<Value>();
    assert!(
        overview["pending_reminders"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reminder| {
                reminder["entity_id"] == "asset-expiring"
                    && reminder["title"] == "资产即将到期：Backend 域名"
            })
    );
    assert!(
        overview["pending_reminders"]
            .as_array()
            .unwrap()
            .iter()
            .all(|reminder| reminder["entity_id"] != "asset-other-expiring")
    );
}

#[tokio::test]
async fn s93_operating_deadline_check_creates_linked_reminders_without_duplicates() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;
    let soon = (Utc::now().date_naive() + Duration::days(10)).to_string();

    seed_expiring_contract(
        &app,
        "contract-expiring",
        "Backend Service Contract",
        soon.clone(),
    )
    .await;
    seed_expiring_license(
        &app,
        "license-expiring",
        "Backend Business License",
        soon.clone(),
    )
    .await;
    seed_expiring_rent(&app, "rent-expiring", "Backend Office", soon.clone()).await;
    seed_expiring_asset(
        &app,
        "asset-operating-expiring",
        "Backend Trademark",
        soon.clone(),
    )
    .await;
    seed_expiring_contract(&other, "contract-other-expiring", "Other Contract", soon).await;

    let res = app
        .post("/api/v1/reminders/check_operating_deadlines")
        .json(&serde_json::json!({}))
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["reminders_created"].as_i64().unwrap(), 4);
    assert_eq!(
        body["created_reminders"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|reminder| reminder["entity_type"] == "contracts")
            .count(),
        1
    );
    assert_eq!(
        body["created_reminders"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|reminder| reminder["entity_type"] == "licenses")
            .count(),
        1
    );
    assert_eq!(
        body["created_reminders"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|reminder| reminder["entity_type"] == "rent_records")
            .count(),
        1
    );
    assert_eq!(
        body["created_reminders"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|reminder| reminder["entity_type"] == "assets")
            .count(),
        1
    );

    let second = app
        .post("/api/v1/reminders/check_operating_deadlines")
        .json(&serde_json::json!({}))
        .await;
    second.assert_status_ok();
    assert_eq!(
        second.json::<Value>()["reminders_created"].as_i64(),
        Some(0)
    );

    let overview = app.get("/api/v1/reminders/overview").await;
    overview.assert_status_ok();
    let overview = overview.json::<Value>();
    let pending = overview["pending_reminders"].as_array().unwrap();
    assert_eq!(pending.len(), 4);
    assert!(pending.iter().any(|reminder| {
        reminder["entity_type"] == "contracts"
            && reminder["entity_id"] == "contract-expiring"
            && reminder["type_label"] == "合同"
    }));
    assert!(pending.iter().any(|reminder| {
        reminder["entity_type"] == "licenses"
            && reminder["entity_id"] == "license-expiring"
            && reminder["type_label"] == "证照"
    }));
    assert!(pending.iter().any(|reminder| {
        reminder["entity_type"] == "rent_records"
            && reminder["entity_id"] == "rent-expiring"
            && reminder["type_label"] == "租赁"
    }));
    assert!(pending.iter().any(|reminder| {
        reminder["entity_type"] == "assets"
            && reminder["entity_id"] == "asset-operating-expiring"
            && reminder["type_label"] == "资产到期"
    }));
    assert!(
        pending
            .iter()
            .all(|reminder| reminder["entity_id"] != "contract-other-expiring")
    );
    assert_eq!(
        overview["check_operating_deadlines_action"]["endpoint"],
        "/api/v1/reminders/check_operating_deadlines"
    );
}
