use crate::common;

use serde_json::Value;

async fn seed_bank_account(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO bank_accounts (
            id, company_id, bank_name, account_name, account_number
        )
        VALUES ('bank-needle', ?, 'Backend Bank', 'Main Account', '6222')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed bank account");
}

async fn seed_payment(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO payments (
            id, company_id, payment_no, payment_type, amount,
            payer, payee, payment_date, status
        )
        VALUES (
            'payment-needle', ?, 'PAY-001', 'expense', 800.0,
            'Backend Payer', 'Backend Payee', '2026-05-09', 'paid'
        )
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed payment");
}

async fn seed_matched_payment(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO payments (
            id, company_id, payment_no, payment_type, amount,
            payer, payee, payment_date, status
        )
        VALUES (
            'payment-matched', ?, 'PAY-MATCHED', 'expense', 800.0,
            'Backend Payer', 'Backend Payee', '2026-05-09', 'paid'
        )
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed matched payment");
}

#[tokio::test]
async fn s105_reconciliation_overview_returns_backend_owned_sections_stats_and_labels() {
    let app = common::TestApp::new().await;
    seed_bank_account(&app).await;
    seed_payment(&app).await;
    seed_matched_payment(&app).await;

    sqlx::query(
        r#"
        INSERT INTO bank_transactions (
            id, company_id, account_id, transaction_type, amount,
            balance_after, counterparty, description, transaction_date
        )
        VALUES (
            'tx-unmatched', ?, 'bank-needle', 'credit', 1200.0,
            1200.0, 'Backend Counterparty', 'Unmatched transfer', '2026-05-09 10:00:00'
        )
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed unmatched transaction");

    sqlx::query(
        r#"
        INSERT INTO bank_transactions (
            id, company_id, account_id, transaction_type, amount,
            balance_after, counterparty, description, transaction_date, payment_id
        )
        VALUES (
            'tx-matched', ?, 'bank-needle', 'debit', 800.0,
            400.0, 'Backend Payee', 'Matched payment', '2026-05-09 11:00:00', 'payment-matched'
        )
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed matched transaction");

    let res = app.get("/api/v1/reconciliation/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"][0]["label"], "待匹配项目");
    assert_eq!(body["stats"][0]["value"], 2);
    assert_eq!(
        body["stats"][0]["class_name"],
        "bg-yellow-500/10 text-yellow-600"
    );

    let bank_section = body["unmatched_sections"]
        .as_array()
        .unwrap()
        .iter()
        .find(|section| section["key"] == "bank_transactions")
        .expect("bank transaction section");
    assert_eq!(bank_section["title"], "未匹配银行流水");
    let bank_row = &bank_section["rows"].as_array().unwrap()[0];
    assert!(
        bank_row["cells"]
            .as_array()
            .unwrap()
            .iter()
            .any(|cell| cell["value"] == "¥1,200")
    );
    assert!(
        bank_row["cells"]
            .as_array()
            .unwrap()
            .iter()
            .any(|cell| cell["badge"]["label"] == "收入")
    );

    let payment_section = body["unmatched_sections"]
        .as_array()
        .unwrap()
        .iter()
        .find(|section| section["key"] == "payments")
        .expect("payment section");
    assert_eq!(payment_section["rows"][0]["cells"][1]["value"], "PAY-001");
    assert!(
        payment_section["rows"][0]["cells"]
            .as_array()
            .unwrap()
            .iter()
            .any(|cell| cell["badge"]["label"] == "付款")
    );

    let matched_row = &body["matched_section"]["rows"].as_array().unwrap()[0];
    assert!(
        matched_row["cells"]
            .as_array()
            .unwrap()
            .iter()
            .any(|cell| cell["badge"]["label"] == "已匹配")
    );
}
