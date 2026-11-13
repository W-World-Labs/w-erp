use crate::common;

use serde_json::Value;

async fn seed_recurring_invoice(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO recurring_invoices (
            id, company_id, title, amount, frequency, start_date,
            next_invoice_date, status, total_generated
        )
        VALUES (
            'recurring-needle', ?, 'Needle Subscription', 1234.5, 'quarterly',
            '2026-01-01', '2026-03-01', 'active', 3
        )
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed recurring invoice");
}

async fn seed_invoice(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO invoices (
            id, company_id, invoice_number, invoice_type, amount, status
        )
        VALUES ('invoice-needle', ?, 'INV-NEEDLE', 'output', 100.0, 'pending')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed invoice");
}

async fn seed_reminder(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO invoice_reminders (
            id, company_id, invoice_id, reminder_type, scheduled_date,
            status, channel, subject
        )
        VALUES (
            'reminder-needle', ?, 'invoice-needle', 'overdue_7',
            '2026-02-15', 'failed', 'sms', '逾期提醒'
        )
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed reminder");
}

#[tokio::test]
async fn s112_recurring_overview_returns_display_labels_badges_and_actions() {
    let app = common::TestApp::new().await;
    seed_recurring_invoice(&app).await;
    seed_invoice(&app).await;
    seed_reminder(&app).await;

    let res = app.get("/api/v1/recurring_invoices/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total_monthly_label"], "¥411.50");
    assert_eq!(body["permissions"]["can_manage"], true);
    assert_eq!(body["create_action"]["method"], "POST");
    assert_eq!(
        body["create_action"]["endpoint"],
        "/api/v1/recurring_invoices"
    );
    assert_eq!(body["create_action"]["label"], "新建周期发票");
    assert_eq!(body["create_action"]["success_message"], "周期发票已创建");
    assert_eq!(body["reminder_create_action"]["method"], "POST");
    assert_eq!(
        body["reminder_create_action"]["endpoint"],
        "/api/v1/invoice_reminders"
    );
    assert_eq!(body["reminder_create_action"]["label"], "新建催款提醒");
    assert_eq!(
        body["reminder_create_action"]["success_message"],
        "催款提醒已创建"
    );
    assert_eq!(body["empty_messages"]["recurring"], "暂无周期发票");
    assert_eq!(body["empty_messages"]["reminders"], "暂无催款提醒");

    let recurring = &body["recurring"][0];
    assert_eq!(recurring["amount_label"], "¥1,234.50");
    assert_eq!(recurring["frequency_label"], "每季度");
    assert_eq!(recurring["next_invoice_date_label"], "2026-03-01");
    assert_eq!(recurring["status_badge"]["label"], "运行中");
    assert_eq!(recurring["toggle_action"]["next_status"], "paused");
    assert_eq!(recurring["toggle_action"]["icon"], "Pause");
    assert_eq!(recurring["toggle_action"]["method"], "PUT");
    assert_eq!(
        recurring["toggle_action"]["endpoint"],
        "/api/v1/recurring_invoices/recurring-needle"
    );
    assert_eq!(
        recurring["toggle_action"]["default_values"]["status"],
        "paused"
    );
    assert_eq!(recurring["total_generated_label"], "3 次");
    assert_eq!(recurring["update_action"]["method"], "PUT");
    assert_eq!(
        recurring["update_action"]["endpoint"],
        "/api/v1/recurring_invoices/recurring-needle"
    );
    assert_eq!(recurring["update_action"]["label"], "保存修改");
    assert_eq!(
        recurring["update_action"]["success_message"],
        "周期发票已更新"
    );
    assert_eq!(recurring["can_delete"], true);
    assert_eq!(recurring["delete_action"]["method"], "DELETE");
    assert_eq!(
        recurring["delete_action"]["endpoint"],
        "/api/v1/recurring_invoices/recurring-needle"
    );
    assert_eq!(
        recurring["delete_action"]["confirm_message"],
        "确定要删除周期发票「Needle Subscription」吗？"
    );
    assert_eq!(recurring["delete_action"]["success_title"], "删除成功");
    assert_eq!(
        recurring["delete_action"]["success_message"],
        "周期发票已删除"
    );

    let reminder = &body["reminders"][0];
    assert_eq!(reminder["invoice_display"], "INV-NEEDLE");
    assert_eq!(reminder["reminder_type_label"], "逾期7天");
    assert_eq!(reminder["scheduled_date_label"], "2026-02-15");
    assert_eq!(reminder["sent_at_label"], "-");
    assert_eq!(reminder["status_badge"]["label"], "失败");
    assert_eq!(reminder["channel_label"], "短信");
    assert_eq!(reminder["can_delete"], true);
    assert_eq!(reminder["delete_action"]["method"], "DELETE");
    assert_eq!(
        reminder["delete_action"]["endpoint"],
        "/api/v1/invoice_reminders/reminder-needle"
    );
    assert_eq!(
        reminder["delete_action"]["confirm_message"],
        "确定要删除催款提醒「INV-NEEDLE」吗？"
    );
    assert_eq!(reminder["delete_action"]["success_title"], "删除成功");
    assert_eq!(
        reminder["delete_action"]["success_message"],
        "催款提醒已删除"
    );
}

#[tokio::test]
async fn s112_recurring_form_options_are_backend_owned() {
    let app = common::TestApp::new().await;

    let res = app.get("/api/v1/recurring_invoices/form_options").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["default_values"]["frequency"], "monthly");
    assert_eq!(body["default_values"]["payment_terms_days"], 30);
    assert_eq!(body["default_values"]["reminder_type"], "before_due");
    assert_eq!(body["frequency_options"][0]["value"], "weekly");
    assert_eq!(body["frequency_options"][1]["label"], "每月");
    assert_eq!(body["reminder_type_options"][3]["value"], "overdue_7");
    assert_eq!(body["reminder_type_options"][3]["label"], "逾期7天");
}

#[tokio::test]
async fn s112_recurring_deletes_require_backend_permission() {
    let app = common::TestApp::new().await;
    seed_recurring_invoice(&app).await;
    seed_invoice(&app).await;
    seed_reminder(&app).await;

    sqlx::query("UPDATE company_members SET role = 'member' WHERE user_id = ? AND company_id = ?")
        .bind(&app.user_id)
        .bind(&app.company_id)
        .execute(&app.pool)
        .await
        .expect("downgrade member");

    let recurring_delete = app
        .delete("/api/v1/recurring_invoices/recurring-needle")
        .await;
    recurring_delete.assert_status_forbidden();

    let reminder_delete = app
        .delete("/api/v1/invoice_reminders/reminder-needle")
        .await;
    reminder_delete.assert_status_forbidden();
}
