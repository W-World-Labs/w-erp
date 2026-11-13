use crate::common;

use serde_json::Value;

async fn seed_payment(
    app: &common::TestApp,
    id: &str,
    payment_no: &str,
    payment_type: &str,
    payment_method: &str,
    amount: f64,
    invoice_required: bool,
    invoice_id: Option<&str>,
) {
    sqlx::query(
        r#"
        INSERT INTO payments (
            id, company_id, payment_no, payment_date, payment_type, payment_method,
            amount, payer, payee, description, invoice_required, invoice_id,
            invoice_deadline, status
        )
        VALUES (?, ?, ?, '2026-01-02', ?, ?, ?, 'Needle Payer', 'Needle Payee',
                'needle payment', ?, ?, '2026-12-31', 'completed')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(payment_no)
    .bind(payment_type)
    .bind(payment_method)
    .bind(amount)
    .bind(if invoice_required { 1 } else { 0 })
    .bind(invoice_id)
    .execute(&app.pool)
    .await
    .expect("seed payment");
}

#[tokio::test]
async fn s99_payments_overview_filters_labels_and_permissions_on_backend() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_payment(
        &app,
        "payment-needle",
        "PAY-NEEDLE",
        "expense",
        "bank_transfer_b2b",
        123.45,
        true,
        None,
    )
    .await;
    seed_payment(
        &app,
        "payment-income",
        "PAY-INCOME",
        "income",
        "personal_advance",
        500.0,
        false,
        None,
    )
    .await;
    seed_payment(
        &other,
        "payment-other",
        "PAY-NEEDLE-OTHER",
        "expense",
        "bank_transfer_b2b",
        999.0,
        true,
        None,
    )
    .await;

    let res = app
        .get("/api/payments/overview?search=needle&payment_type=expense&payment_method=bank_transfer_b2b&invoice_status=pending")
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total"].as_i64().unwrap(), 2);
    assert_eq!(body["stats"]["expense"].as_f64().unwrap(), 123.45);
    assert_eq!(body["stats"]["income"].as_f64().unwrap(), 500.0);
    assert_eq!(body["stats"]["expense_label"], "¥123.45");
    assert_eq!(body["stats"]["income_label"], "¥500");
    assert_eq!(body["permissions"]["can_delete"], true);

    let payments = body["payments"].as_array().unwrap();
    assert_eq!(payments.len(), 1);
    let payment = &payments[0];
    assert_eq!(payment["id"], "payment-needle");
    assert_eq!(payment["payment_date_label"], "2026-01-02");
    assert_eq!(payment["payment_type_badge"]["label"], "支出");
    assert_eq!(payment["payment_method_label"], "公对公");
    assert_eq!(payment["signed_amount_label"], "-¥123.45");
    assert_eq!(payment["amount_class"], "text-red-600");
    assert_eq!(payment["invoice_badge"]["label"], "待收发票");
    assert_eq!(payment["invoice_badge"]["icon"], "AlertCircle");
    assert_eq!(payment["status_badge"]["label"], "已完成");
    assert_eq!(payment["status_badge"]["icon"], "CheckCircle");
    assert_eq!(payment["can_delete"], true);
    assert!(payments.iter().all(|item| item["id"] != "payment-other"));

    assert!(
        body["payment_method_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "personal_advance" && option["label"] == "个人代付")
    );
    assert!(
        body["invoice_status_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "received" && option["label"] == "已收发票")
    );
}
