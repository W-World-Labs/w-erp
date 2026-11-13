use crate::common;

use serde_json::Value;

async fn seed_account(
    app: &common::TestApp,
    id: &str,
    bank_name: &str,
    account_name: &str,
    account_type: &str,
    balance: f64,
    status: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO bank_accounts (
            id, company_id, bank_name, account_name, account_number,
            account_type, balance, currency, status
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, 'CNY', ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(bank_name)
    .bind(account_name)
    .bind(format!("{id}-number"))
    .bind(account_type)
    .bind(balance)
    .bind(status)
    .execute(&app.pool)
    .await
    .expect("seed bank account");
}

async fn seed_transaction(
    app: &common::TestApp,
    id: &str,
    account_id: &str,
    transaction_type: &str,
    amount: f64,
) {
    sqlx::query(
        r#"
        INSERT INTO bank_transactions (
            id, company_id, account_id, transaction_type, amount, transaction_date
        )
        VALUES (?, ?, ?, ?, ?, '2026-05-01')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(account_id)
    .bind(transaction_type)
    .bind(amount)
    .execute(&app.pool)
    .await
    .expect("seed bank transaction");
}

fn assert_f64(actual: &Value, expected: f64) {
    let actual = actual.as_f64().expect("number");
    assert!(
        (actual - expected).abs() < 0.001,
        "expected {expected}, got {actual}"
    );
}

#[tokio::test]
async fn s91_bank_accounts_overview_calculates_balances_transactions_and_labels_on_backend() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_account(
        &app,
        "bank-active",
        "招商银行",
        "基本户",
        "checking",
        1_000.0,
        "active",
    )
    .await;
    seed_account(
        &app,
        "bank-inactive",
        "建设银行",
        "定期户",
        "savings",
        500.0,
        "inactive",
    )
    .await;
    seed_account(
        &other,
        "bank-other",
        "其他银行",
        "其他租户户",
        "checking",
        999_999.0,
        "active",
    )
    .await;

    seed_transaction(&app, "tx-income", "bank-active", "credit", 300.0).await;
    seed_transaction(&app, "tx-expense", "bank-active", "debit", 120.0).await;
    seed_transaction(&app, "tx-income-2", "bank-inactive", "income", 50.0).await;
    seed_transaction(&other, "tx-other", "bank-other", "credit", 999.0).await;

    let res = app.get("/api/bank/accounts/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total_accounts"].as_i64().unwrap(), 2);
    assert_eq!(body["stats"]["active_accounts"].as_i64().unwrap(), 1);
    assert_f64(&body["stats"]["total_balance"], 1_500.0);
    assert_eq!(body["stats"]["total_balance_label"], "¥1,500");

    let accounts = body["accounts"].as_array().unwrap();
    assert_eq!(accounts.len(), 2);
    assert!(accounts.iter().all(|account| account["id"] != "bank-other"));

    let active = accounts
        .iter()
        .find(|account| account["id"] == "bank-active")
        .expect("active bank account");
    assert_eq!(active["account_type_label"], "活期");
    assert_eq!(active["status_badge"]["label"], "活跃");
    assert_eq!(active["transaction_stats"]["count"].as_i64().unwrap(), 2);
    assert_f64(&active["transaction_stats"]["income"], 300.0);
    assert_f64(&active["transaction_stats"]["expense"], 120.0);
    assert_eq!(active["transaction_stats"]["income_label"], "¥300");
}
