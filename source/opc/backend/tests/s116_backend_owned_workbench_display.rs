/// Scenario 116: backend-owned dashboard workbench display metadata.
/// The dashboard receives display-ready labels, badges, limited attention lists, and chart labels.
use crate::common;

use chrono::{Duration, Utc};
use serde_json::Value;

#[tokio::test]
async fn s116_workbench_summary_returns_display_ready_dashboard_fields() {
    let app = common::TestApp::new().await;
    seed_dashboard_fixture(&app).await;

    let res = app.get("/api/v1/workbench/summary").await;
    res.assert_status_ok();

    let body: Value = res.json();
    assert!(
        body["generated_date_label"]
            .as_str()
            .unwrap()
            .contains('年')
    );

    let stats = &body["stats"];
    assert_eq!(stats["monthly_expense_label"], "¥2,200.00");
    assert_eq!(stats["pending_invoices_label"], "1 张待审发票");
    assert_eq!(stats["expiring_assets_label"], "1 项即将到期");

    let attention_items = body["attention_items"].as_array().unwrap();
    assert!(!attention_items.is_empty());
    assert!(attention_items.len() <= 5);
    let first_item = &attention_items[0];
    assert!(first_item["priority_badge"]["label"].as_str().is_some());
    assert!(first_item["days_left_label"].as_str().is_some());
    assert!(first_item["type_icon"].as_str().is_some());

    assert_eq!(
        body["today_tasks_count"],
        body["today_tasks"].as_array().unwrap().len() as i64
    );

    let stage_stats = &body["stage_stats"];
    assert_eq!(stage_stats["total_value_label"], "¥8,000.00");
    assert_eq!(stage_stats["total_salary_label"], "¥1,000.00");
    assert_eq!(stage_stats["project_execution_rate_label"], "50%");
    assert_eq!(stage_stats["contract_activity_rate_label"], "100%");

    let trend = body["monthly_expense_trend"].as_array().unwrap();
    let latest = trend.last().unwrap();
    assert_eq!(latest["salary_label"], "¥1,000.00");
    assert_eq!(latest["invoice_label"], "¥1,200.00");
    assert_eq!(latest["total_label"], "¥2,200.00");

    let cards = body["overview_cards"].as_array().unwrap();
    assert_eq!(cards[0]["key"], "overdue_receivables");
    assert_eq!(cards[1]["key"], "month_cash_net");
    assert_eq!(cards[2]["key"], "monthly_expense");
    assert_eq!(cards[2]["value_label"], "¥2,200.00");
    let actions = body["quick_actions"].as_array().unwrap();
    assert!(actions.len() >= 5);
    assert_eq!(actions[0]["label"], "应收追款");
    assert_eq!(actions[0]["path"], "/payments");
    assert!(body["guidance"]["empty_urgent_title"].as_str().is_some());
    assert!(body["guidance"]["kind"].as_str().is_some());
    assert!(
        body["survival_items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"]
                .as_str()
                .unwrap_or_default()
                .starts_with("payable:"))
    );
    assert_eq!(body["survival_title"], "企业风险");
    assert!(
        attention_items
            .iter()
            .all(|item| !item["id"].as_str().unwrap_or_default().starts_with("payable:"))
    );
}

async fn seed_dashboard_fixture(app: &common::TestApp) {
    let today = Utc::now().date_naive();
    let today_str = today.format("%Y-%m-%d").to_string();
    let due_soon = (today + Duration::days(1)).format("%Y-%m-%d").to_string();
    let current_month = today.format("%Y-%m").to_string();

    sqlx::query(
        "INSERT INTO invoices (id, company_id, invoice_number, invoice_type, amount, tax_amount, status, invoice_date, due_date, supplier, description)
         VALUES (?, ?, 'WB-PENDING', 'expense', 1200, 0, 'pending', ?, ?, 'Workbench Vendor', NULL)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .bind(&today_str)
    .bind(&due_soon)
    .execute(&app.pool)
    .await
    .unwrap();

    let employee_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO employees (id, company_id, name, status) VALUES (?, ?, 'Workbench Employee', 'active')",
    )
    .bind(&employee_id)
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO salary_records (id, company_id, employee_id, period, base_salary, bonus)
         VALUES (?, ?, ?, ?, 800, 200)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .bind(&employee_id)
    .bind(&current_month)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO contracts (id, company_id, contract_name, counterparty, amount, start_date, end_date, status)
         VALUES (?, ?, 'Workbench Contract', 'Counterparty', 12000, ?, ?, 'active')",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .bind(&today_str)
    .bind(&due_soon)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO assets (id, company_id, name, asset_type, status, current_value, expiry_date)
         VALUES (?, ?, 'Workbench Trademark', 'intangible', 'active', 8000, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .bind(&due_soon)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO projects (id, company_id, name, status) VALUES (?, ?, 'Workbench Running', 'ongoing')",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO projects (id, company_id, name, status) VALUES (?, ?, 'Workbench Done', 'completed')",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();
}
