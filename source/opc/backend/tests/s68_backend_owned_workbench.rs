/// Scenario 68: backend-owned dashboard workbench and AI insights.
/// The frontend must consume these aggregate endpoints instead of calculating
/// business urgency, statistics, completeness, or insight scores locally.
use crate::common;

use axum::http::StatusCode;
use chrono::{Datelike, Duration, Utc};
use serde_json::Value;

#[tokio::test]
async fn s68_workbench_summary_requires_auth() {
    let app = common::TestApp::new().await;

    let res = app.server.get("/api/v1/workbench/summary").await;

    res.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn s68_empty_workbench_summary_is_backend_owned() {
    let app = common::TestApp::new().await;

    let res = app.get("/api/v1/workbench/summary").await;
    res.assert_status_ok();

    let body: Value = res.json();
    assert!(body["generated_at"].as_str().is_some());
    assert!(body["greeting"].as_str().is_some());
    assert_eq!(body["stats"]["pending_invoices"].as_i64().unwrap(), 0);
    assert_eq!(body["stats"]["active_contracts"].as_i64().unwrap(), 0);
    assert_eq!(body["stats"]["expiring_assets"].as_i64().unwrap(), 0);
    assert_eq!(body["stats"]["ongoing_projects"].as_i64().unwrap(), 0);
    assert_eq!(body["stats"]["employees"].as_i64().unwrap(), 0);
    assert_eq!(body["stats"]["monthly_expense"].as_f64().unwrap(), 0.0);
    assert_eq!(body["urgent_items"].as_array().unwrap().len(), 0);
    assert_eq!(body["survival_items"].as_array().unwrap().len(), 0);
    assert_eq!(body["survival_count"].as_i64().unwrap(), 0);
    assert_eq!(body["application_reminders"].as_array().unwrap().len(), 0);
    assert_eq!(body["asset_expiry_reminders"].as_array().unwrap().len(), 0);
    assert_eq!(body["today_tasks"].as_array().unwrap().len(), 0);
    assert_eq!(body["todos"].as_array().unwrap().len(), 0);
    assert_eq!(body["completion_rate"].as_i64().unwrap(), 0);
    assert_eq!(body["guidance"]["visible"], true);
    assert_eq!(body["guidance"]["kind"], "activation");
    assert_eq!(body["guidance"]["next_route"], "/company-info");
    assert_eq!(body["guidance"]["next_label"], "完善公司信息");
    assert!(
        body["guidance"]["empty_urgent_title"]
            .as_str()
            .unwrap()
            .contains("还没有要催的事")
    );
    assert!(
        !body["guidance"]["empty_urgent_title"]
            .as_str()
            .unwrap()
            .contains("太棒了")
    );
    assert!(!body["guidance"]["steps"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn s68_workbench_summary_returns_operating_control_for_empty_company() {
    let app = common::TestApp::new().await;

    let res = app.get("/api/v1/workbench/summary").await;
    res.assert_status_ok();

    let body: Value = res.json();
    let control = &body["operating_control"];
    assert_eq!(control["id"], "operating_control");
    assert_eq!(control["status"], "blocked");
    assert_eq!(control["tone"], "danger");
    assert_eq!(control["completed"].as_bool(), Some(false));
    assert_eq!(control["blocked_step_id"], "contracts");
    assert_eq!(control["backbone_rate"].as_i64(), Some(0));
    assert!(control["backbone_rate_label"].as_str().is_some());
    assert!(control["status_label"].as_str().is_some());
    assert!(control["summary_label"].as_str().is_some());
    assert!(control["risk_count_label"].as_str().is_some());
    assert!(control["next_action"]["route"].as_str().is_some());
    assert_eq!(
        control["control_items"].as_array().unwrap()[0]["type"],
        "backbone_gap"
    );
    assert_eq!(control["critical_count"].as_i64(), Some(1));
}

#[tokio::test]
async fn s68_workbench_summary_aggregates_operational_data_in_backend() {
    let app = common::TestApp::new().await;
    seed_operational_data(&app).await;

    let res = app.get("/api/v1/workbench/summary").await;
    res.assert_status_ok();

    let body: Value = res.json();
    let stats = &body["stats"];
    assert_eq!(stats["pending_invoices"].as_i64().unwrap(), 1);
    assert_eq!(stats["active_contracts"].as_i64().unwrap(), 1);
    assert_eq!(stats["expiring_assets"].as_i64().unwrap(), 1);
    assert_eq!(stats["ongoing_projects"].as_i64().unwrap(), 1);
    assert_eq!(stats["employees"].as_i64().unwrap(), 1);
    assert_eq!(stats["monthly_expense"].as_f64().unwrap(), 2200.0);
    let header_badges = body["header_badges"].as_array().unwrap();
    assert!(
        header_badges
            .iter()
            .any(|badge| badge["label"] == "1 张待审发票")
    );
    assert!(
        header_badges
            .iter()
            .any(|badge| badge["label"] == "1 项即将到期")
    );
    assert!(
        body["stats"]["month_cash_stressed"].as_bool() == Some(true)
            || body["stats"]["month_cash_net"].as_f64().unwrap() <= 0.0
    );

    let urgent_types: Vec<String> = body["urgent_items"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["type"].as_str().map(str::to_owned))
        .collect();
    assert!(urgent_types.contains(&"invoice".to_string()));
    assert!(urgent_types.contains(&"contract".to_string()));
    assert!(urgent_types.contains(&"asset".to_string()));
    assert!(urgent_types.contains(&"application".to_string()));

    let application_reminders = body["application_reminders"].as_array().unwrap();
    assert_eq!(application_reminders.len(), 1);
    assert!(
        application_reminders
            .iter()
            .all(|item| item["type"].as_str() == Some("application"))
    );
    assert!(
        application_reminders[0]["priority_badge"]["label"]
            .as_str()
            .is_some()
    );
    assert_eq!(
        application_reminders[0]["days_left_label"]
            .as_str()
            .unwrap(),
        "2 天后截止"
    );

    let asset_expiry_reminders = body["asset_expiry_reminders"].as_array().unwrap();
    assert_eq!(asset_expiry_reminders.len(), 1);
    assert!(
        asset_expiry_reminders
            .iter()
            .all(|item| item["type"].as_str() == Some("asset"))
    );
    assert_eq!(
        asset_expiry_reminders[0]["priority_badge"]["label"]
            .as_str()
            .unwrap(),
        "关注"
    );
    assert_eq!(
        asset_expiry_reminders[0]["days_left_label"]
            .as_str()
            .unwrap(),
        "20 天后到期"
    );

    assert!(body["todos"].as_array().unwrap().len() >= 3);
    assert!(!body["ai_suggestions"].as_array().unwrap().is_empty());
    assert_eq!(body["overview_cards"][0]["key"], "overdue_receivables");
    assert_eq!(body["overview_cards"][1]["key"], "month_cash_net");
    assert_eq!(
        body["overview_cards"][2]["class_name"],
        "from-blue-500/10 to-blue-500/5 text-blue-500"
    );
    assert!(
        body["overview_cards"][0]
            .as_object()
            .unwrap()
            .contains_key("trend_variant")
    );
}

#[tokio::test]
async fn s68_workbench_summary_is_company_scoped() {
    let app = common::TestApp::new().await;
    seed_operational_data(&app).await;
    let rival = app.create_second_tenant().await;

    let res = rival.get("/api/v1/workbench/summary").await;
    res.assert_status_ok();

    let body: Value = res.json();
    assert_eq!(body["stats"]["pending_invoices"].as_i64().unwrap(), 0);
    assert_eq!(body["stats"]["active_contracts"].as_i64().unwrap(), 0);
    assert_eq!(body["stats"]["expiring_assets"].as_i64().unwrap(), 0);
    assert_eq!(body["stats"]["ongoing_projects"].as_i64().unwrap(), 0);
    assert_eq!(body["stats"]["employees"].as_i64().unwrap(), 0);
    assert_eq!(body["urgent_items"].as_array().unwrap().len(), 0);
    assert_eq!(body["survival_items"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn s68_ai_insights_are_deterministic_backend_output() {
    let app = common::TestApp::new().await;
    seed_operational_data(&app).await;

    let res = app.get("/api/v1/ai/insights").await;
    res.assert_status_ok();

    let body: Value = res.json();
    assert!(body["generated_at"].as_str().is_some());
    assert!(body["insights"].as_array().unwrap().len() >= 1);
    let first_insight = &body["insights"].as_array().unwrap()[0];
    assert!(first_insight["icon"].as_str().is_some());
    assert!(first_insight["display_icon"].as_str().is_some());
    assert!(first_insight["priority_badge"]["label"].as_str().is_some());
    assert!(first_insight["card_border_class_name"].as_str().is_some());
    assert!(first_insight["action_label"].as_str().is_some());
    assert!(body["completeness"]["items"].as_array().unwrap().len() >= 5);
    assert!(body["completeness"]["overall_score"].as_i64().unwrap() > 0);
    assert!(body["summary"]["total"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn s68_ai_insights_include_operating_control_when_backbone_is_blocked() {
    let app = common::TestApp::new().await;

    let res = app.get("/api/v1/ai/insights").await;
    res.assert_status_ok();

    let body: Value = res.json();
    assert_eq!(body["operating_control"]["status"], "blocked");
    assert!(
        body["insights"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"] == "operating_control:backbone_gap")
    );
}

#[tokio::test]
async fn s68_workbench_operating_control_is_healthy_for_complete_backbone() {
    let app = common::TestApp::new().await;
    seed_complete_operating_backbone(&app).await;

    let res = app.get("/api/v1/workbench/summary").await;
    res.assert_status_ok();

    let body: Value = res.json();
    let control = &body["operating_control"];
    assert_eq!(control["status"], "healthy");
    assert_eq!(control["tone"], "success");
    assert_eq!(control["completed"].as_bool(), Some(true));
    assert_eq!(control["critical_count"].as_i64(), Some(0));
    assert_eq!(control["backbone_rate"].as_i64(), Some(100));
    assert!(
        body["overview_cards"]
            .as_array()
            .unwrap()
            .iter()
            .any(|card| card["key"] == "operating_control")
    );
}

#[tokio::test]
async fn s68_workbench_operating_control_reports_overdue_linked_reminders() {
    let app = common::TestApp::new().await;
    seed_complete_operating_backbone(&app).await;

    let overdue = (Utc::now().date_naive() - Duration::days(2))
        .format("%Y-%m-%d")
        .to_string();
    sqlx::query(
        "INSERT INTO reminders (id, company_id, title, reminder_type, remind_date, status, is_completed, entity_type, entity_id)
         VALUES ('s68-overdue-operating-reminder', ?, '合同复核逾期', 'contract_expiry', ?, 'pending', 0, 'contracts', 's68-op-contract')",
    )
    .bind(&app.company_id)
    .bind(&overdue)
    .execute(&app.pool)
    .await
    .unwrap();

    let res = app.get("/api/v1/workbench/summary").await;
    res.assert_status_ok();

    let body: Value = res.json();
    let control = &body["operating_control"];
    assert_eq!(control["status"], "blocked");
    assert!(control["critical_count"].as_i64().unwrap() >= 1);
    assert!(
        control["control_items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["type"] == "operating_deadline" && item["priority"] == "critical")
    );
}

#[tokio::test]
async fn s68_workbench_surfaces_overdue_receivables_for_collection() {
    let app = common::TestApp::new().await;
    let overdue = (Utc::now().date_naive() - Duration::days(12))
        .format("%Y-%m-%d")
        .to_string();
    sqlx::query(
        "INSERT INTO invoices (id, company_id, invoice_number, invoice_type, amount, status, due_date, supplier)
         VALUES ('s68-ar-overdue', ?, 'AR-LATE-1', 'receivable', 8800, 'overdue', ?, '拖欠客户')",
    )
    .bind(&app.company_id)
    .bind(&overdue)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    assert_eq!(body["stats"]["overdue_receivables"], 1);
    assert_eq!(
        body["stats"]["overdue_receivable_amount_label"],
        "¥8,800.00"
    );
    assert!(
        body["header_badges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|badge| badge["label"]
                .as_str()
                .unwrap_or_default()
                .contains("逾期应收"))
    );
    let overdue_item = body["urgent_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == "receivable:s68-ar-overdue")
        .expect("overdue receivable should be on the workbench");
    assert_eq!(overdue_item["action"], "去追款");
    assert_eq!(overdue_item["path"], "/payments");
    assert_eq!(overdue_item["priority"], "critical");
    assert!(
        body["ai_suggestions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|line| line.as_str().unwrap_or_default().contains("逾期应收"))
    );
}

#[tokio::test]
async fn s68_workbench_flags_month_cash_gap_against_rent_and_salary() {
    let app = common::TestApp::new().await;
    let period = Utc::now().date_naive().format("%Y-%m").to_string();
    sqlx::query(
        "INSERT INTO employees (id, company_id, name, status) VALUES ('s68-cash-emp', ?, 'Cash Emp', 'active')",
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO salary_records (id, company_id, employee_id, employee_name, period, base_salary, bonus, status)
         VALUES ('s68-cash-sal', ?, 's68-cash-emp', 'Cash Emp', ?, 6000, 0, 'pending')",
    )
    .bind(&app.company_id)
    .bind(&period)
    .execute(&app.pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO rent_records (id, company_id, location_name, monthly_rent, status)
         VALUES ('s68-cash-rent', ?, 'Studio', 4000, 'active')",
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    assert_eq!(body["stats"]["month_cash_stressed"], true);
    assert_eq!(body["stats"]["month_cash_outflow_label"], "¥10,000.00");
    assert_eq!(body["stats"]["month_cash_net_label"], "¥-10,000.00");
    let item = body["urgent_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| {
            row["id"]
                .as_str()
                .unwrap_or_default()
                .starts_with("cash-gap:")
        })
        .expect("cash gap should appear on the workbench");
    assert_eq!(item["path"], "/cash-flow-forecast");
    assert_eq!(item["action"], "看现金流");
    assert!(
        body["ai_suggestions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|line| line.as_str().unwrap_or_default().contains("净现金"))
    );
}

#[tokio::test]
async fn s68_workbench_surfaces_upcoming_tax_deadline() {
    let app = common::TestApp::new().await;
    let today = Utc::now().date_naive();
    let due = today + Duration::days(5);
    sqlx::query(
        "INSERT INTO tax_deadlines (
            id, company_id, jurisdiction_code, tax_type, title, title_local,
            deadline_month, deadline_day, recurrence, is_active, penalty_info
         ) VALUES (
            's68-tax-soon', ?, 'CN', 'vat', 'VAT Filing', '增值税申报',
            ?, ?, 'yearly', 1, '逾期可能产生滞纳金。'
         )",
    )
    .bind(&app.company_id)
    .bind(due.month() as i64)
    .bind(due.day() as i64)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    let item = body["urgent_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "tax-deadline:s68-tax-soon")
        .expect("tax deadline should appear on the workbench");
    assert_eq!(item["path"], "/tax-calendar");
    assert_eq!(item["action"], "看日历");
    assert!(item["title"].as_str().unwrap().contains("增值税申报"));
    assert!(
        body["ai_suggestions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|line| line.as_str().unwrap_or_default().contains("税务截止"))
    );
}

#[tokio::test]
async fn s68_workbench_surfaces_pending_mailbox_invoices() {
    let app = common::TestApp::new().await;
    sqlx::query(
        "INSERT INTO invoice_mailboxes (
            id, company_id, name, imap_host, imap_port, imap_username, imap_password, folder, enabled
         ) VALUES ('s68-box', ?, 'Inbox', 'imap.example.test', 993, 'a@b.c', 'secret', 'INBOX', 0)",
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO invoice_mail_imports (
            id, company_id, mailbox_id, message_uid, attachment_index, attachment_name,
            content_sha256, xml_content, status
         ) VALUES (
            's68-mail-1', ?, 's68-box', 1, 0, 'inv.xml', 'hash-s68', x'3c786d6c2f3e', 'pending_confirmation'
         )",
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    let item = body["urgent_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "mailbox-inbox")
        .expect("mailbox queue should appear on the workbench");
    assert_eq!(item["path"], "/settings");
    assert_eq!(item["action"], "去确认");
    assert!(item["title"].as_str().unwrap().contains("1 张发票待确认"));
}

#[tokio::test]
async fn s68_workbench_surfaces_subscription_renewal() {
    let app = common::TestApp::new().await;
    let soon = (Utc::now().date_naive() + Duration::days(4))
        .format("%Y-%m-%d")
        .to_string();
    sqlx::query(
        "INSERT INTO software_subscriptions (
            id, company_id, software_name, vendor, subscription_type, billing_cycle,
            start_date, next_billing_date, total_price, auto_renewal, status
         ) VALUES (
            's68-saas', ?, 'Notion', 'Notion Labs', 'SaaS', 'monthly',
            '2026-01-01', ?, 240, 1, 'active'
         )",
    )
    .bind(&app.company_id)
    .bind(&soon)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    let item = body["urgent_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "subscription:s68-saas")
        .expect("subscription renewal should appear");
    assert_eq!(item["path"], "/cloud-cost-management");
    assert_eq!(item["action"], "去检查");
    assert!(item["title"].as_str().unwrap().contains("Notion"));
}

#[tokio::test]
async fn s68_workbench_surfaces_unbilled_time() {
    let app = common::TestApp::new().await;
    let today = Utc::now().format("%Y-%m-%d").to_string();
    sqlx::query(
        "INSERT INTO time_entries (
            id, company_id, description, date, duration_minutes, is_billable, amount, status
         ) VALUES ('s68-hours', ?, '客户工时', ?, 90, 1, 1500, 'logged')",
    )
    .bind(&app.company_id)
    .bind(&today)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    let item = body["urgent_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "unbilled-time")
        .expect("unbilled time should appear");
    assert_eq!(item["path"], "/time-tracking");
    assert_eq!(item["action"], "去开票");
    assert!(item["title"].as_str().unwrap().contains("1 条工时"));
}

#[tokio::test]
async fn s68_workbench_surfaces_overdue_payables() {
    let app = common::TestApp::new().await;
    let overdue = (Utc::now().date_naive() - Duration::days(6))
        .format("%Y-%m-%d")
        .to_string();
    sqlx::query(
        "INSERT INTO invoices (
            id, company_id, invoice_number, invoice_type, amount, status, due_date, supplier
         ) VALUES (
            's68-ap-overdue', ?, 'AP-LATE-1', 'payable', 3200, 'approved', ?, '印刷厂'
         )",
    )
    .bind(&app.company_id)
    .bind(&overdue)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    let item = body["urgent_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "payable:s68-ap-overdue")
        .expect("overdue payable should appear");
    assert_eq!(item["path"], "/invoices");
    assert_eq!(item["action"], "去付款");
    assert!(item["title"].as_str().unwrap().contains("印刷厂"));
    assert_eq!(item["priority"], "critical");
    assert!(
        body["ai_suggestions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|line| line.as_str().unwrap_or_default().contains("供应商账单"))
    );
}

#[tokio::test]
async fn s68_workbench_ignores_receivable_as_payable() {
    let app = common::TestApp::new().await;
    let overdue = (Utc::now().date_naive() - Duration::days(4))
        .format("%Y-%m-%d")
        .to_string();
    sqlx::query(
        "INSERT INTO invoices (
            id, company_id, invoice_number, invoice_type, amount, status, due_date, supplier
         ) VALUES (
            's68-ar-not-ap', ?, 'AR-NOT-AP', 'receivable', 5000, 'overdue', ?, '客户甲'
         )",
    )
    .bind(&app.company_id)
    .bind(&overdue)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    assert!(
        body["urgent_items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["id"] != "payable:s68-ar-not-ap")
    );
}

#[tokio::test]
async fn s68_workbench_ignores_paid_payables() {
    let app = common::TestApp::new().await;
    let overdue = (Utc::now().date_naive() - Duration::days(3))
        .format("%Y-%m-%d")
        .to_string();
    sqlx::query(
        "INSERT INTO invoices (
            id, company_id, invoice_number, invoice_type, amount, status, due_date, supplier
         ) VALUES (
            's68-ap-paid', ?, 'AP-PAID-1', 'payable', 1800, 'paid', ?, '快递'
         )",
    )
    .bind(&app.company_id)
    .bind(&overdue)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    assert!(
        body["urgent_items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["id"] != "payable:s68-ap-paid")
    );
}

#[tokio::test]
async fn s68_workbench_surfaces_due_rent_payment() {
    let app = common::TestApp::new().await;
    let due = (Utc::now().date_naive() + Duration::days(2))
        .format("%Y-%m-%d")
        .to_string();
    sqlx::query(
        "INSERT INTO rent_records (
            id, company_id, location_name, monthly_rent, next_payment_date, landlord, status
         ) VALUES (
            's68-rent-due', ?, '南山工作室', 6800, ?, '陈房东', 'active'
         )",
    )
    .bind(&app.company_id)
    .bind(&due)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    let item = body["urgent_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "rent-pay:s68-rent-due")
        .expect("due rent should appear");
    assert_eq!(item["path"], "/rent");
    assert_eq!(item["action"], "去付租");
    assert!(item["title"].as_str().unwrap().contains("南山工作室"));
    assert!(
        body["ai_suggestions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|line| line.as_str().unwrap_or_default().contains("房租"))
    );
}

#[tokio::test]
async fn s68_workbench_surfaces_expiring_lease() {
    let app = common::TestApp::new().await;
    let end = (Utc::now().date_naive() + Duration::days(18))
        .format("%Y-%m-%d")
        .to_string();
    sqlx::query(
        "INSERT INTO rent_records (
            id, company_id, location_name, monthly_rent, contract_end_date, landlord, status
         ) VALUES (
            's68-lease-end', ?, '仓库A', 4200, ?, '物业', 'active'
         )",
    )
    .bind(&app.company_id)
    .bind(&end)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    let item = body["urgent_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "rent-lease:s68-lease-end")
        .expect("expiring lease should appear");
    assert_eq!(item["path"], "/rent");
    assert_eq!(item["action"], "去处理");
    assert!(item["title"].as_str().unwrap().contains("仓库A"));
}

#[tokio::test]
async fn s68_workbench_ignores_ended_rent() {
    let app = common::TestApp::new().await;
    let due = (Utc::now().date_naive() + Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();
    sqlx::query(
        "INSERT INTO rent_records (
            id, company_id, location_name, monthly_rent, next_payment_date, status
         ) VALUES (
            's68-rent-ended', ?, '旧办公室', 3000, ?, 'ended'
         )",
    )
    .bind(&app.company_id)
    .bind(&due)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    assert!(
        body["urgent_items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["id"] != "rent-pay:s68-rent-ended")
    );
}

#[tokio::test]
async fn s68_workbench_surfaces_expiring_license_as_survival() {
    let app = common::TestApp::new().await;
    let expiry = (Utc::now().date_naive() + Duration::days(12))
        .format("%Y-%m-%d")
        .to_string();
    sqlx::query(
        "INSERT INTO licenses (
            id, company_id, license_name, issuing_authority, expiry_date, status
         ) VALUES (
            's68-license', ?, '营业执照', '市监局', ?, 'valid'
         )",
    )
    .bind(&app.company_id)
    .bind(&expiry)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    let item = body["survival_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "license:s68-license")
        .expect("license should be on the survival list");
    assert_eq!(item["path"], "/licenses");
    assert_eq!(item["action"], "去处理");
    assert!(item["title"].as_str().unwrap().contains("营业执照"));
    assert_eq!(body["survival_title"], "企业风险");
}

#[tokio::test]
async fn s68_workbench_surfaces_expiring_insurance_as_survival() {
    let app = common::TestApp::new().await;
    let today = Utc::now().date_naive();
    let start = (today - Duration::days(300)).format("%Y-%m-%d").to_string();
    let renew = (today + Duration::days(8)).format("%Y-%m-%d").to_string();
    let end = (today + Duration::days(18)).format("%Y-%m-%d").to_string();
    sqlx::query(
        "INSERT INTO insurance_policies (
            id, company_id, policy_name, insurance_type, provider, premium_amount,
            start_date, end_date, renewal_date, status
         ) VALUES (
            's68-insure', ?, '雇主责任险', 'workers_comp', '平安', 12000,
            ?, ?, ?, 'active'
         )",
    )
    .bind(&app.company_id)
    .bind(&start)
    .bind(&end)
    .bind(&renew)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    let item = body["survival_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "insurance:s68-insure")
        .expect("insurance should be on the survival list");
    assert_eq!(item["path"], "/compliance");
    assert!(item["title"].as_str().unwrap().contains("雇主责任险"));
}

#[tokio::test]
async fn s68_workbench_surfaces_unpaid_last_month_payroll() {
    let app = common::TestApp::new().await;
    let today = Utc::now().date_naive();
    let last = if today.month() == 1 {
        format!("{:04}-12", today.year() - 1)
    } else {
        format!("{:04}-{:02}", today.year(), today.month() - 1)
    };
    sqlx::query(
        "INSERT INTO employees (id, company_id, name, status)
         VALUES ('s68-pay-emp', ?, '待发员工', 'active')",
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO salary_records (
            id, company_id, employee_id, employee_name, period, base_salary, bonus, status
         ) VALUES (
            's68-pay-sal', ?, 's68-pay-emp', '待发员工', ?, 8000, 0, 'pending'
         )",
    )
    .bind(&app.company_id)
    .bind(&last)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    let item = body["survival_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == format!("payroll:{last}"))
        .expect("unpaid last-month payroll should be survival");
    assert_eq!(item["path"], "/salary");
    assert_eq!(item["action"], "去发工资");
    assert_eq!(item["priority"], "critical");
}

#[tokio::test]
async fn s68_workbench_surfaces_bank_cash_shortfall() {
    let app = common::TestApp::new().await;
    sqlx::query(
        "INSERT INTO bank_accounts (
            id, company_id, bank_name, account_name, account_number, balance, status
         ) VALUES (
            's68-bank', ?, '招行', '基本户', '6222', 800, 'active'
         )",
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();
    let due = (Utc::now().date_naive() + Duration::days(3))
        .format("%Y-%m-%d")
        .to_string();
    sqlx::query(
        "INSERT INTO rent_records (
            id, company_id, location_name, monthly_rent, next_payment_date, status
         ) VALUES (
            's68-bank-rent', ?, '办公室', 5000, ?, 'active'
         )",
    )
    .bind(&app.company_id)
    .bind(&due)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    let item = body["survival_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "bank-cash")
        .expect("bank cash shortfall should be survival");
    assert_eq!(item["path"], "/bank-accounts");
    assert_eq!(item["action"], "看账户");
    assert_eq!(item["priority"], "critical");
}

#[tokio::test]
async fn s68_workbench_keeps_unbilled_time_off_survival_list() {
    let app = common::TestApp::new().await;
    let today = Utc::now().format("%Y-%m-%d").to_string();
    sqlx::query(
        "INSERT INTO time_entries (
            id, company_id, description, date, duration_minutes, is_billable, amount, status
         ) VALUES ('s68-hours-ns', ?, '客户工时', ?, 90, 1, 1500, 'logged')",
    )
    .bind(&app.company_id)
    .bind(&today)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    assert!(
        body["urgent_items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["id"] == "unbilled-time")
    );
    assert!(
        body["survival_items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["id"] != "unbilled-time")
    );
}

#[tokio::test]
async fn s68_workbench_surfaces_stale_sent_quote() {
    let app = common::TestApp::new().await;
    let until = (Utc::now().date_naive() + Duration::days(6))
        .format("%Y-%m-%d")
        .to_string();
    sqlx::query(
        "INSERT INTO clients (id, company_id, name) VALUES ('s68-quote-client', ?, '跟进客户')",
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO quotes (
            id, company_id, client_id, quote_number, title, status, total, valid_until
         ) VALUES (
            's68-quote', ?, 's68-quote-client', 'QT-68', '网站改版', 'sent', 18000, ?
         )",
    )
    .bind(&app.company_id)
    .bind(&until)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    let item = body["urgent_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "quote:s68-quote")
        .expect("sent quote should appear");
    assert_eq!(item["path"], "/quotes");
    assert_eq!(item["action"], "去跟进");
    assert!(item["title"].as_str().unwrap().contains("网站改版"));
    assert!(
        body["survival_items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["id"] != "quote:s68-quote")
    );
}

#[tokio::test]
async fn s68_workbench_surfaces_due_recurring_invoice() {
    let app = common::TestApp::new().await;
    let due = (Utc::now().date_naive() + Duration::days(3))
        .format("%Y-%m-%d")
        .to_string();
    sqlx::query(
        "INSERT INTO recurring_invoices (
            id, company_id, title, amount, frequency, start_date, next_invoice_date, status
         ) VALUES (
            's68-retainer', ?, '月度运营托管', 8000, 'monthly', '2026-01-01', ?, 'active'
         )",
    )
    .bind(&app.company_id)
    .bind(&due)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    let item = body["urgent_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "recurring:s68-retainer")
        .expect("due recurring invoice should appear");
    assert_eq!(item["path"], "/recurring-invoices");
    assert_eq!(item["action"], "去开票");
    assert!(item["title"].as_str().unwrap().contains("月度运营托管"));
    assert!(
        body["survival_items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["id"] != "recurring:s68-retainer")
    );
}

#[tokio::test]
async fn s68_workbench_surfaces_deal_past_expected_close() {
    let app = common::TestApp::new().await;
    let close = (Utc::now().date_naive() - Duration::days(4))
        .format("%Y-%m-%d")
        .to_string();
    sqlx::query(
        "INSERT INTO clients (id, company_id, name) VALUES ('s68-deal-client', ?, '谈判客户')",
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO deals (
            id, company_id, client_id, title, stage, amount, expected_close_date
         ) VALUES (
            's68-deal-late', ?, 's68-deal-client', '年度运维', 'negotiation', 26000, ?
         )",
    )
    .bind(&app.company_id)
    .bind(&close)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    let item = body["urgent_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "deal:s68-deal-late")
        .expect("overdue deal should appear");
    assert_eq!(item["path"], "/deals");
    assert_eq!(item["action"], "去跟进");
    assert!(item["title"].as_str().unwrap().contains("年度运维"));
    assert!(
        body["survival_items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["id"] != "deal:s68-deal-late")
    );
    assert!(
        body["ai_suggestions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|line| line.as_str().unwrap_or_default().contains("销售机会"))
    );
}

#[tokio::test]
async fn s68_workbench_surfaces_idle_open_deal() {
    let app = common::TestApp::new().await;
    let touched = (Utc::now().naive_utc() - Duration::days(8))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    sqlx::query(
        "INSERT INTO clients (id, company_id, name) VALUES ('s68-deal-idle-c', ?, '沉默客户')",
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO deals (
            id, company_id, client_id, title, stage, amount, created_at, updated_at
         ) VALUES (
            's68-deal-idle', ?, 's68-deal-idle-c', '官网改版', 'lead', 9000, ?, ?
         )",
    )
    .bind(&app.company_id)
    .bind(&touched)
    .bind(&touched)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    let item = body["urgent_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "deal:s68-deal-idle")
        .expect("idle deal should appear");
    assert!(item["description"].as_str().unwrap().contains("没动"));
}

#[tokio::test]
async fn s68_workbench_ignores_won_deal() {
    let app = common::TestApp::new().await;
    let close = (Utc::now().date_naive() + Duration::days(2))
        .format("%Y-%m-%d")
        .to_string();
    sqlx::query(
        "INSERT INTO clients (id, company_id, name) VALUES ('s68-deal-won-c', ?, '已赢客户')",
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO deals (
            id, company_id, client_id, title, stage, amount, expected_close_date
         ) VALUES (
            's68-deal-won', ?, 's68-deal-won-c', '已成交项目', 'won', 12000, ?
         )",
    )
    .bind(&app.company_id)
    .bind(&close)
    .execute(&app.pool)
    .await
    .unwrap();

    let body: Value = app.get("/api/v1/workbench/summary").await.json();
    assert!(
        body["urgent_items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["id"] != "deal:s68-deal-won")
    );
}

async fn seed_operational_data(app: &common::TestApp) {
    let today = Utc::now().date_naive();
    let due_soon = (today + Duration::days(5)).format("%Y-%m-%d").to_string();
    let due_mid = (today + Duration::days(20)).format("%Y-%m-%d").to_string();
    let app_deadline = (today + Duration::days(2)).format("%Y-%m-%d").to_string();
    let current_month = today.format("%Y-%m").to_string();
    let today_str = today.format("%Y-%m-%d").to_string();

    sqlx::query(
        "INSERT INTO invoices (id, company_id, invoice_number, invoice_type, amount, tax_amount, status, invoice_date, due_date, supplier, description)
         VALUES (?, ?, 'INV-PENDING', 'expense', 1000, 0, 'pending', ?, ?, 'ACME', 'Pending expense')",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .bind(&today_str)
    .bind(&due_soon)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO invoices (id, company_id, invoice_number, invoice_type, amount, tax_amount, status, invoice_date, due_date, supplier, description)
         VALUES (?, ?, 'INV-PAID', 'expense', 500, 0, 'paid', ?, ?, 'Paid Vendor', 'Paid expense')",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .bind(&today_str)
    .bind(&due_mid)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO contracts (id, company_id, contract_name, counterparty, amount, start_date, end_date, status)
         VALUES (?, ?, 'Platform Service', 'Vendor Ltd', 12000, ?, ?, 'active')",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .bind(&today_str)
    .bind(&due_mid)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO assets (id, company_id, name, asset_type, status, current_value, expiry_date)
         VALUES (?, ?, 'Core Trademark', 'intangible', 'active', 8000, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .bind(&due_mid)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO applications (id, company_id, title, type, deadline, status, progress)
         VALUES (?, ?, 'Innovation Grant', 'grant', ?, 'preparing', 30)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .bind(&app_deadline)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO projects (id, company_id, name, status, progress, budget, spent)
         VALUES (?, ?, 'Automation Rollout', 'ongoing', 45, 10000, 3000)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();

    let employee_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO employees (id, company_id, name, department, position, status, base_salary)
         VALUES (?, ?, 'Alice', 'Ops', 'Manager', 'active', 600)",
    )
    .bind(&employee_id)
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO salary_records (id, company_id, employee_id, employee_name, period, base_salary, bonus, status)
         VALUES (?, ?, ?, 'Alice', ?, 600, 100, 'pending')",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .bind(&employee_id)
    .bind(&current_month)
    .execute(&app.pool)
    .await
    .unwrap();
}

async fn seed_complete_operating_backbone(app: &common::TestApp) {
    let today = Utc::now().date_naive();
    let today_str = today.format("%Y-%m-%d").to_string();
    let future = (today + Duration::days(45)).format("%Y-%m-%d").to_string();
    let current_month = today.format("%Y-%m").to_string();

    sqlx::query(
        "INSERT INTO projects (id, company_id, name, status, progress, budget, spent)
         VALUES ('s68-op-project', ?, 'Operating Backbone', 'ongoing', 25, 10000, 1200)",
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO contracts (id, company_id, contract_name, counterparty, amount, start_date, end_date, status, project_id)
         VALUES ('s68-op-contract', ?, 'Operating Contract', 'Client Ltd', 12000, ?, ?, 'active', 's68-op-project')",
    )
    .bind(&app.company_id)
    .bind(&today_str)
    .bind(&future)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO employees (id, company_id, name, department, position, status, base_salary)
         VALUES ('s68-op-employee', ?, 'Ops Employee', 'Ops', 'Manager', 'active', 800)",
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO salary_records (id, company_id, employee_id, employee_name, period, base_salary, bonus, status)
         VALUES ('s68-op-salary', ?, 's68-op-employee', 'Ops Employee', ?, 800, 200, 'paid')",
    )
    .bind(&app.company_id)
    .bind(&current_month)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO assets (id, company_id, name, asset_type, status, current_value, expiry_date, contract_id)
         VALUES ('s68-op-asset', ?, 'Operating Asset', 'intangible', 'active', 5000, ?, 's68-op-contract')",
    )
    .bind(&app.company_id)
    .bind(&future)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO licenses (id, company_id, license_name, license_type, license_number, expiry_date, status)
         VALUES ('s68-op-license', ?, 'Operating License', 'business', 'LIC-68', ?, 'valid')",
    )
    .bind(&app.company_id)
    .bind(&future)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO rent_records (id, company_id, location_name, monthly_rent, contract_start_date, contract_end_date, contract_id, status)
         VALUES ('s68-op-rent', ?, 'Operating Office', 2000, ?, ?, 's68-op-contract', 'active')",
    )
    .bind(&app.company_id)
    .bind(&today_str)
    .bind(&future)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO budgets (id, company_id, category, budget_type, period, budget_amount, actual_amount, status, project_id)
         VALUES ('s68-op-budget', ?, 'Operations', 'monthly', ?, 10000, 1200, 'active', 's68-op-project')",
    )
    .bind(&app.company_id)
    .bind(&current_month)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO reminders (id, company_id, title, reminder_type, status, is_completed, entity_type, entity_id)
         VALUES ('s68-op-reminder', ?, '合同复核', 'contract_expiry', 'pending', 0, 'contracts', 's68-op-contract')",
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();
}
