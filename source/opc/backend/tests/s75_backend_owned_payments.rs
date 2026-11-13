use crate::common;

use chrono::{Duration, Utc};
use serde_json::Value;

async fn seed_payment(
    app: &common::TestApp,
    id: &str,
    payment_type: &str,
    amount: f64,
    invoice_required: bool,
    invoice_id: Option<&str>,
    invoice_deadline: Option<String>,
) {
    sqlx::query(
        r#"
        INSERT INTO payments (
            id, company_id, payment_no, payment_type, amount,
            invoice_required, invoice_id, invoice_deadline, status
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'completed')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(format!("PAY-{}", id))
    .bind(payment_type)
    .bind(amount)
    .bind(if invoice_required { 1 } else { 0 })
    .bind(invoice_id)
    .bind(invoice_deadline)
    .execute(&app.pool)
    .await
    .expect("seed payment");
}

#[tokio::test]
async fn s75_payments_overview_calculates_totals_and_invoice_status_on_backend() {
    let app = common::TestApp::new().await;
    let past = (Utc::now() - Duration::days(2))
        .format("%Y-%m-%d")
        .to_string();
    let future = (Utc::now() + Duration::days(7))
        .format("%Y-%m-%d")
        .to_string();

    seed_payment(
        &app,
        "expense-overdue",
        "expense",
        100.0,
        true,
        None,
        Some(past),
    )
    .await;
    seed_payment(
        &app,
        "expense-received",
        "支出",
        50.0,
        true,
        Some("invoice-1"),
        Some(future),
    )
    .await;
    seed_payment(&app, "income", "income", 250.0, false, None, None).await;

    let res = app.get("/api/payments/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total"].as_i64().unwrap(), 3);
    assert_eq!(body["stats"]["expense"].as_f64().unwrap(), 150.0);
    assert_eq!(body["stats"]["income"].as_f64().unwrap(), 250.0);
    assert_eq!(body["stats"]["pending_invoice"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["overdue_invoice"].as_i64().unwrap(), 1);
    assert_eq!(body["create_action"]["method"], "POST");
    assert_eq!(body["create_action"]["endpoint"], "/api/payments");
    assert_eq!(body["create_action"]["label"], "新增支付");
    assert_eq!(body["create_action"]["success_message"], "支付记录已创建");

    let payments = body["payments"].as_array().unwrap();
    let overdue = payments
        .iter()
        .find(|payment| payment["id"] == "expense-overdue")
        .expect("overdue payment");
    assert_eq!(overdue["invoice_received"], false);
    assert_eq!(overdue["invoice_status"], "overdue");
    assert_eq!(overdue["can_delete"].as_bool().unwrap(), true);
    assert_eq!(overdue["delete_action"]["method"], "DELETE");
    assert_eq!(
        overdue["delete_action"]["endpoint"],
        "/api/payments/expense-overdue"
    );
    assert_eq!(
        overdue["delete_action"]["confirm_message"],
        "确定要删除支付记录「PAY-expense-overdue」吗？"
    );
    assert_eq!(overdue["delete_action"]["success_title"], "删除成功");
    assert_eq!(
        overdue["delete_action"]["success_message"],
        "支付记录已删除"
    );
    assert_eq!(overdue["update_action"]["method"], "PUT");
    assert_eq!(
        overdue["update_action"]["endpoint"],
        "/api/payments/expense-overdue"
    );
    assert_eq!(overdue["update_action"]["label"], "保存修改");
    assert_eq!(
        overdue["update_action"]["success_message"],
        "支付记录已更新"
    );

    let received = payments
        .iter()
        .find(|payment| payment["id"] == "expense-received")
        .expect("received payment");
    assert_eq!(received["invoice_received"], true);
    assert_eq!(received["invoice_status"], "received");
    assert_eq!(body["payment_type_options"][0]["value"], "all");
    assert_eq!(body["payment_type_options"][0]["label"], "全部");
    assert_eq!(body["payment_method_options"][0]["value"], "all");
    assert_eq!(body["payment_method_options"][0]["label"], "全部方式");
    assert_eq!(body["invoice_status_options"][0]["value"], "all");
    assert_eq!(body["invoice_status_options"][0]["label"], "全部发票");
}

#[tokio::test]
async fn s75_payment_form_options_are_backend_owned() {
    let app = common::TestApp::new().await;

    let res = app.get("/api/payments/form_options").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["default_values"]["payment_type"], "expense");
    assert_eq!(body["default_values"]["status"], "completed");
    assert_eq!(body["payment_type_options"][0]["value"], "expense");
    assert_eq!(
        body["payment_method_options"][0]["value"],
        "bank_transfer_b2b"
    );
    assert_eq!(body["category_options"][0]["value"], "工资支出");
    assert_eq!(body["status_options"][0]["label"], "已完成");
}

#[tokio::test]
async fn s75_payments_overview_is_company_scoped() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_payment(&app, "owned", "income", 100.0, false, None, None).await;
    seed_payment(&other, "other", "income", 900.0, false, None, None).await;

    let res = app.get("/api/payments/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["income"].as_f64().unwrap(), 100.0);
}
