use crate::common;

use chrono::{Duration, Utc};
use serde_json::Value;

async fn seed_payment(
    app: &common::TestApp,
    id: &str,
    amount: f64,
    invoice_deadline: Option<String>,
    invoice_id: Option<&str>,
) {
    sqlx::query(
        r#"
        INSERT INTO payments (
            id, company_id, payment_no, payment_date, payment_type,
            payment_method, amount, payer, payee, invoice_required,
            invoice_id, invoice_deadline, status
        )
        VALUES (
            ?, ?, ?, '2026-01-02', 'expense', 'bank_transfer_b2b',
            ?, 'Needle Payer', 'Needle Payee', 1, ?, ?, 'completed'
        )
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(format!("PAY-{id}"))
    .bind(amount)
    .bind(invoice_id)
    .bind(invoice_deadline)
    .execute(&app.pool)
    .await
    .expect("seed payment");
}

#[tokio::test]
async fn s114_payments_overview_returns_invoice_reminder_stats_and_rows() {
    let app = common::TestApp::new().await;
    let past = (Utc::now() - Duration::days(2))
        .format("%Y-%m-%d")
        .to_string();
    let urgent = (Utc::now() + Duration::days(2))
        .format("%Y-%m-%d")
        .to_string();

    seed_payment(&app, "overdue", 100.0, Some(past.clone()), None).await;
    seed_payment(&app, "urgent", 200.0, Some(urgent), None).await;
    seed_payment(&app, "received", 999.0, None, Some("invoice-1")).await;

    let res = app.get("/api/payments/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    let stats = &body["invoice_reminder_stats"];
    assert_eq!(stats["total"], 2);
    assert_eq!(stats["overdue"], 1);
    assert_eq!(stats["urgent"], 1);
    assert_eq!(stats["total_amount_label"], "¥300");

    let rows = body["pending_invoice_payments"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|row| row["id"] != "received"));

    let overdue = rows.iter().find(|row| row["id"] == "overdue").unwrap();
    assert_eq!(overdue["amount_label"], "¥100");
    assert_eq!(overdue["payment_date_label"], "2026-01-02");
    assert_eq!(overdue["payment_method_label"], "公对公");
    assert_eq!(overdue["invoice_deadline_label"], past);
    assert_eq!(overdue["urgency_badge"]["variant"], "destructive");
    assert!(
        overdue["urgency_badge"]["label"]
            .as_str()
            .unwrap()
            .contains("已逾期")
    );
}
