/// Scenario 4: 🏦 银行账户与资金流
/// 开户(初始余额) → 存入 → 取出 → 验证余额一致
///  → 验证交易记录 → 并发安全
use crate::common;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn s04_bank_account_lifecycle() {
    let app = common::TestApp::new().await;

    // Step 1: Create bank account
    let res = app
        .post("/api/v1/bank/accounts")
        .json(&json!({
            "bank_name": "中国银行",
            "account_name": "对公账户",
            "account_number": "0002001234567890",
            "account_type": "checking",
            "currency": "CNY",
            "balance": 100000.0
        }))
        .await;
    res.assert_status_ok();
    let acct: serde_json::Value = res.json();
    let acct_id = acct["id"].as_str().expect("should have id");

    // Step 2: Get by ID
    let get_res = app.get(&format!("/api/v1/bank/accounts/{}", acct_id)).await;
    get_res.assert_status_ok();

    // Step 3: Create deposit transaction (typed extractor requires transaction_date)
    let tx_res = app
        .post("/api/v1/bank/transactions")
        .json(&json!({
            "account_id": acct_id,
            "transaction_type": "deposit",
            "amount": 50000.0,
            "transaction_date": "2026-03-01",
            "description": "客户付款"
        }))
        .await;
    tx_res.assert_status_ok();

    // Step 4: Create withdrawal
    let wd_res = app
        .post("/api/v1/bank/transactions")
        .json(&json!({
            "account_id": acct_id,
            "transaction_type": "withdrawal",
            "amount": 20000.0,
            "transaction_date": "2026-03-01",
            "description": "供应商付款"
        }))
        .await;
    wd_res.assert_status_ok();

    // Step 5: List transactions
    let list_res = app.get("/api/v1/bank/transactions").await;
    list_res.assert_status_ok();
    let body: serde_json::Value = list_res.json();
    assert!(
        body["count"].as_i64().unwrap() >= 2,
        "should have at least 2 transactions"
    );
}

#[tokio::test]
async fn s04_bank_transaction_updates_balance_chain() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/bank/accounts")
        .json(&json!({
            "bank_name": "链路银行",
            "account_name": "流水户",
            "account_number": "CHAIN-001",
            "balance": 1000.0
        }))
        .await;
    res.assert_status_ok();
    let acct_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let deposit = app
        .post("/api/v1/bank/transactions")
        .json(&json!({
            "account_id": &acct_id,
            "transaction_type": "deposit",
            "amount": 250.0,
            "transaction_date": "2026-03-02"
        }))
        .await;
    deposit.assert_status_ok();
    assert_eq!(deposit.json::<serde_json::Value>()["balance_after"], 1250.0);

    let withdrawal = app
        .post("/api/v1/bank/transactions")
        .json(&json!({
            "account_id": &acct_id,
            "transaction_type": "withdrawal",
            "amount": 50.0,
            "transaction_date": "2026-03-03"
        }))
        .await;
    withdrawal.assert_status_ok();
    assert_eq!(
        withdrawal.json::<serde_json::Value>()["balance_after"],
        1200.0
    );

    let account = app
        .get(&format!("/api/v1/bank/accounts/{}", acct_id))
        .await
        .json::<serde_json::Value>();
    assert_eq!(account["balance"], 1200.0);
}

#[tokio::test]
async fn s04_bank_negative_amount_rejected() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/bank/accounts")
        .json(&json!({
            "bank_name": "Bad Bank",
            "account_name": "Test",
            "account_number": "0000",
            "balance": -500.0
        }))
        .await;
    res.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn s04_bank_invalid_transaction_type_rejected() {
    let app = common::TestApp::new().await;

    let acct_res = app
        .post("/api/v1/bank/accounts")
        .json(&json!({
            "bank_name": "Guard Bank",
            "account_name": "Boundary",
            "account_number": "BOUND-001",
            "balance": 800.0
        }))
        .await;
    acct_res.assert_status_ok();
    let acct_id = acct_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let tx_res = app
        .post("/api/v1/bank/transactions")
        .json(&json!({
            "account_id": &acct_id,
            "transaction_type": "refund",
            "amount": 100.0,
            "transaction_date": "2026-03-04"
        }))
        .await;
    tx_res.assert_status(StatusCode::BAD_REQUEST);

    let account = app
        .get(&format!("/api/v1/bank/accounts/{}", acct_id))
        .await
        .json::<serde_json::Value>();
    assert_eq!(account["balance"], 800.0);
}

#[tokio::test]
async fn s04_bank_tenant_isolation() {
    let app_a = common::TestApp::new().await;
    let app_b = app_a.create_second_tenant().await;

    app_a
        .post("/api/v1/bank/accounts")
        .json(&json!({
            "bank_name": "A银行",
            "account_name": "A户",
            "account_number": "111",
            "balance": 10000.0
        }))
        .await
        .assert_status_ok();

    let res = app_b.get("/api/v1/bank/accounts").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["count"], 0, "tenant B should see 0 of A's accounts");
}

#[tokio::test]
async fn s04_bank_pagination() {
    let app = common::TestApp::new().await;

    for i in 1..=3 {
        app.post("/api/v1/bank/accounts")
            .json(&json!({
                "bank_name": format!("Bank {}", i),
                "account_name": format!("Acct {}", i),
                "account_number": format!("NUM-{}", i),
                "balance": 1000.0 * (i as f64)
            }))
            .await
            .assert_status_ok();
    }

    let res = app.get("/api/v1/bank/accounts?page=1&page_size=2").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["count"], 3);
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn s04_bank_account_delete() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/bank/accounts")
        .json(&json!({
            "bank_name": "删除测试银行",
            "account_name": "测试户",
            "account_number": "DEL-001",
            "balance": 0.0
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let del = app.delete(&format!("/api/v1/bank/accounts/{}", id)).await;
    assert!(
        del.status_code().is_success(),
        "Bank account delete should succeed"
    );

    let get = app.get(&format!("/api/v1/bank/accounts/{}", id)).await;
    assert_eq!(get.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn s04_bank_zero_balance() {
    let app = common::TestApp::new().await;

    // Zero balance should be valid (new account)
    let res = app
        .post("/api/v1/bank/accounts")
        .json(&json!({
            "bank_name": "零余额银行",
            "account_name": "新户",
            "account_number": "ZERO-001",
            "balance": 0.0
        }))
        .await;
    res.assert_status_ok();
}
