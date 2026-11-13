/// Scenario 47: 💱 Multi-Currency & Reconciliation Engine
/// Tests currency conversion, multi-currency balances, exchange rate seeding, and bank reconciliation.
use crate::common;
use serde_json::json;

// ==================== Currency Conversion — Same Currency ====================

#[tokio::test]
async fn s47_convert_same_currency() {
    let app = common::TestApp::new().await;
    let res = app
        .get("/api/v1/currency/convert?from=CNY&to=CNY&amount=1000")
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["converted"], 1000.0);
    assert_eq!(body["rate"], 1.0);
    assert_eq!(body["source"], "identity");
}

// ==================== Currency Conversion — Built-in Fallback ====================

#[tokio::test]
async fn s47_convert_builtin_fallback() {
    let app = common::TestApp::new().await;
    let res = app
        .get("/api/v1/currency/convert?from=USD&to=CNY&amount=100")
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["converted"], 725.0); // 100 * 7.25
    assert_eq!(body["source"], "builtin_fallback");
}

// ==================== Currency Conversion — JPY to KRW ====================

#[tokio::test]
async fn s47_convert_jpy_krw() {
    let app = common::TestApp::new().await;
    let res = app
        .get("/api/v1/currency/convert?from=JPY&to=KRW&amount=10000")
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    let converted = body["converted"].as_f64().unwrap();
    // 10000 JPY → ~91700 KRW (10000 * 9.17)
    assert!(
        converted > 90000.0 && converted < 93000.0,
        "JPY→KRW should be ~91700: {}",
        converted
    );
}

// ==================== Currency Conversion — From Exchange Rates DB ====================

#[tokio::test]
async fn s47_convert_from_db() {
    let app = common::TestApp::new().await;

    // Seed a custom rate
    app.post("/api/v1/exchange_rates")
        .json(&json!({
            "from_currency": "USD",
            "to_currency": "CNY",
            "rate": 7.30,
            "effective_date": "2026-03-01"
        }))
        .await
        .assert_status_ok();

    let res = app
        .get("/api/v1/currency/convert?from=USD&to=CNY&amount=100&date=2026-03-06")
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["converted"], 730.0); // uses DB rate 7.30
    assert_ne!(body["source"], "builtin_fallback"); // should NOT be fallback
}

// ==================== Currency Conversion — Reverse Rate ====================

#[tokio::test]
async fn s47_convert_reverse_rate() {
    let app = common::TestApp::new().await;

    // Only seed USD→EUR rate
    app.post("/api/v1/exchange_rates")
        .json(&json!({
            "from_currency": "USD",
            "to_currency": "EUR",
            "rate": 0.92,
            "effective_date": "2026-03-01"
        }))
        .await
        .assert_status_ok();

    // Now ask for EUR→USD (reverse)
    let res = app
        .get("/api/v1/currency/convert?from=EUR&to=USD&amount=100&date=2026-03-06")
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    let converted = body["converted"].as_f64().unwrap();
    assert!(
        converted > 108.0 && converted < 110.0,
        "EUR→USD should be ~108.70: {}",
        converted
    );
    assert_eq!(body["source"], "reverse_calculation");
}

// ==================== Currency Conversion — Unknown Pair ====================

#[tokio::test]
async fn s47_convert_unknown_pair() {
    let app = common::TestApp::new().await;
    let res = app
        .get("/api/v1/currency/convert?from=BTC&to=DOGE&amount=1")
        .await;
    assert_eq!(res.status_code(), 404);
}

// ==================== Multi-Currency Balances — Empty ====================

#[tokio::test]
async fn s47_balances_empty() {
    let app = common::TestApp::new().await;
    let res = app.get("/api/v1/currency/balances").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert!(body["base_currency"].is_string());
    assert_eq!(body["currency_count"], 0);
}

// ==================== Multi-Currency Balances — With Data ====================

#[tokio::test]
async fn s47_balances_with_multi_currency_data() {
    let app = common::TestApp::new().await;

    // Create CNY income
    app.post("/api/v1/payments")
        .json(&json!({ "status": "completed",  "description": "CN project", "amount": 50000.0, "payment_type": "income", "currency": "CNY", "payment_date": "2026-03-01" }))
        .await.assert_status_ok();

    // Create JPY income
    app.post("/api/v1/payments")
        .json(&json!({ "status": "completed",  "description": "JP project", "amount": 1500000.0, "payment_type": "income", "currency": "JPY", "payment_date": "2026-03-01" }))
        .await.assert_status_ok();

    // Create USD expense
    app.post("/api/v1/payments")
        .json(&json!({ "status": "completed",  "description": "US SaaS", "amount": 99.0, "payment_type": "expense", "currency": "USD", "payment_date": "2026-03-01" }))
        .await.assert_status_ok();

    let res = app.get("/api/v1/currency/balances").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();

    let currencies = body["currencies"].as_array().unwrap();
    assert!(currencies.len() >= 2, "should have at least 2 currencies");

    // Check JPY income exists
    let jpy = currencies.iter().find(|c| c["currency"] == "JPY");
    assert!(jpy.is_some(), "should have JPY");
    assert_eq!(jpy.unwrap()["income"], 1500000.0);
}

// ==================== Exchange Rate Seeding ====================

#[tokio::test]
async fn s47_seed_exchange_rates() {
    let app = common::TestApp::new().await;

    // First seed → should insert all
    let res = app
        .post("/api/v1/exchange_rates/seed")
        .json(&json!({}))
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["total_pairs"], 12);
    assert!(
        body["inserted"].as_i64().unwrap() > 0,
        "first seed should insert"
    );
    assert_eq!(body["source"], "builtin_2026_defaults");

    // Second seed same day → should skip all
    let res2 = app
        .post("/api/v1/exchange_rates/seed")
        .json(&json!({}))
        .await;
    res2.assert_status_ok();
    let body2: serde_json::Value = res2.json();
    assert_eq!(body2["inserted"], 0, "second seed should skip all");
    assert_eq!(body2["skipped"], 12);
}

// ==================== Bank Reconciliation — Empty ====================

#[tokio::test]
async fn s47_reconcile_empty() {
    let app = common::TestApp::new().await;
    let res = app.post("/api/v1/bank/reconcile").json(&json!({})).await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["match_count"], 0);
    assert_eq!(body["unmatched_transactions"], 0);
}

// ==================== Bank Reconciliation — With Matches ====================

#[tokio::test]
async fn s47_reconcile_with_matches() {
    let app = common::TestApp::new().await;

    // Create a bank account first
    let ba = app.post("/api/bank/accounts")
        .json(&json!({ "account_name": "主账户", "bank_name": "工商银行", "account_number": "1234567890", "account_type": "checking", "currency": "CNY" }))
        .await;
    ba.assert_status_ok();
    let ba_id = ba.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create a pending receivable invoice
    let inv = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-RECON-001",
            "amount": 25000.0,
            "invoice_type": "receivable",
            "supplier": "Acme客户"
        }))
        .await;
    inv.assert_status_ok();

    // Create a bank transaction that matches (same amount + invoice number in reference)
    app.post("/api/bank/transactions")
        .json(&json!({
            "account_id": &ba_id,
            "amount": 25000.0,
            "transaction_type": "deposit",
            "transaction_date": "2026-03-05",
            "description": "Payment INV-RECON-001",
            "counterparty": "Acme客户"
        }))
        .await
        .assert_status_ok();

    let res = app.post("/api/v1/bank/reconcile").json(&json!({})).await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();

    assert_eq!(body["match_count"], 1, "should find 1 match");

    let matches = body["proposed_matches"].as_array().unwrap();
    let m = &matches[0];
    assert_eq!(m["invoice_number"], "INV-RECON-001");
    assert_eq!(m["transaction_amount"], 25000.0);

    // High confidence: exact amount (50) + ref match (40) + name match (10) = 100
    let confidence = m["confidence"].as_f64().unwrap();
    assert!(
        confidence >= 80.0,
        "should be high confidence: {}",
        confidence
    );
}
