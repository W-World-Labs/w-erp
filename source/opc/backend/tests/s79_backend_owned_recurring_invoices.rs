use crate::common;

use serde_json::Value;

async fn seed_recurring_invoice(
    app: &common::TestApp,
    id: &str,
    title: &str,
    amount: f64,
    frequency: &str,
    status: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO recurring_invoices (
            id, company_id, title, amount, frequency, start_date, next_invoice_date, status
        )
        VALUES (?, ?, ?, ?, ?, '2026-01-01', '2026-02-01', ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(title)
    .bind(amount)
    .bind(frequency)
    .bind(status)
    .execute(&app.pool)
    .await
    .expect("seed recurring invoice");
}

async fn seed_invoice(app: &common::TestApp, id: &str, number: &str) {
    sqlx::query(
        r#"
        INSERT INTO invoices (id, company_id, invoice_number, invoice_type, amount, status)
        VALUES (?, ?, ?, 'output', 100.0, 'pending')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(number)
    .execute(&app.pool)
    .await
    .expect("seed invoice");
}

async fn seed_reminder(app: &common::TestApp, id: &str, invoice_id: &str, status: &str) {
    sqlx::query(
        r#"
        INSERT INTO invoice_reminders (
            id, company_id, invoice_id, reminder_type, scheduled_date, status, subject
        )
        VALUES (?, ?, ?, 'before_due', '2026-02-01', ?, '催款提醒')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(invoice_id)
    .bind(status)
    .execute(&app.pool)
    .await
    .expect("seed reminder");
}

#[tokio::test]
async fn s79_recurring_overview_calculates_stats_on_backend() {
    let app = common::TestApp::new().await;

    seed_recurring_invoice(&app, "monthly", "月度服务", 1200.0, "monthly", "active").await;
    seed_recurring_invoice(&app, "quarterly", "季度服务", 3000.0, "quarterly", "active").await;
    seed_recurring_invoice(&app, "yearly", "年度服务", 12000.0, "yearly", "active").await;
    seed_recurring_invoice(&app, "paused", "暂停服务", 9000.0, "monthly", "paused").await;
    seed_recurring_invoice(&app, "weekly", "周度服务", 100.0, "weekly", "active").await;

    seed_invoice(&app, "invoice-a", "INV-A").await;
    seed_invoice(&app, "invoice-b", "INV-B").await;
    seed_reminder(&app, "reminder-pending", "invoice-a", "pending").await;
    seed_reminder(&app, "reminder-sent", "invoice-b", "sent").await;

    let res = app.get("/api/v1/recurring_invoices/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["active_recurring"].as_i64().unwrap(), 4);
    assert_eq!(body["stats"]["pending_reminders"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["sent_reminders"].as_i64().unwrap(), 1);

    let monthly = body["stats"]["total_monthly"].as_f64().unwrap();
    assert!((monthly - 3633.0).abs() < 0.001, "monthly total: {monthly}");
    assert_eq!(body["recurring"].as_array().unwrap().len(), 5);
    assert_eq!(body["reminders"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn s79_recurring_overview_is_company_scoped() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_recurring_invoice(&app, "owned", "本公司周期发票", 500.0, "monthly", "active").await;
    seed_recurring_invoice(
        &other,
        "other",
        "其他公司周期发票",
        9000.0,
        "monthly",
        "active",
    )
    .await;

    seed_invoice(&app, "owned-invoice", "OWN").await;
    seed_invoice(&other, "other-invoice", "OTHER").await;
    seed_reminder(&app, "owned-reminder", "owned-invoice", "pending").await;
    seed_reminder(&other, "other-reminder", "other-invoice", "pending").await;

    let res = app.get("/api/v1/recurring_invoices/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["active_recurring"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["total_monthly"].as_f64().unwrap(), 500.0);
    assert_eq!(body["stats"]["pending_reminders"].as_i64().unwrap(), 1);
    assert_eq!(body["recurring"].as_array().unwrap().len(), 1);
    assert_eq!(body["recurring"][0]["id"], "owned");
    assert_eq!(body["reminders"].as_array().unwrap().len(), 1);
    assert_eq!(body["reminders"][0]["id"], "owned-reminder");
}
