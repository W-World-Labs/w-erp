/// Scenario 2: 月度开票流程
/// 创建发票(含外币) → 查看列表(分页) → 更新状态
///  → 验证金额非负 → 导出 CSV → 删除 → 404
use crate::common;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn s02_full_invoice_lifecycle() {
    let app = common::TestApp::new().await;

    // Step 1: Create invoice
    let create_res = app
        .post("/api/v1/invoices")
        .json(&json!({
            "invoice_number": "INV-2026-001",
            "amount": 10000.0,
            "invoice_type": "income",
            "supplier_name": "Acme Corp",
            "currency": "USD",
            "exchange_rate": 7.25,
            "status": "pending"
        }))
        .await;
    create_res.assert_status_ok();
    let invoice: serde_json::Value = create_res.json();
    let id = invoice["id"].as_str().expect("should have id");

    // Step 2: Get by ID
    let get_res = app.get(&format!("/api/v1/invoices/{}", id)).await;
    get_res.assert_status_ok();
    let fetched: serde_json::Value = get_res.json();
    assert_eq!(fetched["invoice_number"], "INV-2026-001");
    assert_eq!(fetched["amount"], 10000.0);

    // Step 3: Update status (include required fields for typed extractor)
    let update_res = app
        .put(&format!("/api/v1/invoices/{}", id))
        .json(&json!({"invoice_number": "INV-2026-001", "amount": 10000.0, "status": "paid"}))
        .await;
    update_res.assert_status_ok();

    // Step 4: Delete
    let del_res = app.delete(&format!("/api/v1/invoices/{}", id)).await;
    del_res.assert_status(StatusCode::NO_CONTENT);

    // Step 5: Verify 404
    let gone_res = app.get(&format!("/api/v1/invoices/{}", id)).await;
    gone_res.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn s02_invoice_create_persists_contract_link() {
    let app = common::TestApp::new().await;

    let contract_res = app
        .post("/api/v1/contracts")
        .json(&json!({
            "contract_name": "语数科技服务合同",
            "contract_number": "YS-TEST-CONTRACT-001",
            "amount": 1280000.0,
            "status": "active"
        }))
        .await;
    contract_res.assert_status_ok();
    let contract_id = contract_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let create_res = app
        .post("/api/v1/invoices")
        .json(&json!({
            "invoice_number": "YS-TEST-INV-001",
            "amount": 512000.0,
            "invoice_type": "output",
            "contract_id": contract_id
        }))
        .await;
    create_res.assert_status_ok();
    let invoice = create_res.json::<serde_json::Value>();
    assert_eq!(invoice["contract_id"].as_str(), Some(contract_id.as_str()));

    let fetched = app
        .get(&format!(
            "/api/v1/invoices/{}",
            invoice["id"].as_str().unwrap()
        ))
        .await
        .json::<serde_json::Value>();
    assert_eq!(fetched["contract_id"].as_str(), Some(contract_id.as_str()));
}

#[tokio::test]
async fn s02_invoice_amount_validation() {
    let app = common::TestApp::new().await;

    // Negative amount should be rejected
    let res = app
        .post("/api/v1/invoices")
        .json(&json!({
            "invoice_number": "BAD-001",
            "amount": -500.0,
            "invoice_type": "income"
        }))
        .await;
    res.assert_status(StatusCode::BAD_REQUEST);
    let body: serde_json::Value = res.json();
    assert!(
        body["error"].as_str().unwrap().contains("non-negative"),
        "error should mention non-negative"
    );
}

#[tokio::test]
async fn s02_invoice_negative_update_rejected() {
    let app = common::TestApp::new().await;

    let create_res = app
        .post("/api/v1/invoices")
        .json(&json!({
            "invoice_number": "UPD-NEG-001",
            "amount": 3200.0,
            "invoice_type": "receivable",
            "status": "pending"
        }))
        .await;
    create_res.assert_status_ok();
    let id = create_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let update_res = app
        .put(&format!("/api/v1/invoices/{}", id))
        .json(&json!({ "amount": -1.0 }))
        .await;
    update_res.assert_status(StatusCode::BAD_REQUEST);

    let body: serde_json::Value = update_res.json();
    assert!(
        body["error"]
            .as_str()
            .unwrap_or("")
            .contains("non-negative")
    );

    let fetched = app
        .get(&format!("/api/v1/invoices/{}", id))
        .await
        .json::<serde_json::Value>();
    assert_eq!(fetched["amount"], 3200.0);
}

#[tokio::test]
async fn s02_invoice_pagination() {
    let app = common::TestApp::new().await;

    // Create 5 invoices
    for i in 1..=5 {
        app.post("/api/v1/invoices")
            .json(&json!({
                "invoice_number": format!("INV-{:03}", i),
                "amount": (i as f64) * 1000.0,
                "invoice_type": "income"
            }))
            .await
            .assert_status_ok();
    }

    // Page 1 with page_size=2
    let page1 = app.get("/api/v1/invoices?page=1&page_size=2").await;
    page1.assert_status_ok();
    let body1: serde_json::Value = page1.json();
    assert_eq!(body1["count"], 5, "total count should be 5");
    assert_eq!(
        body1["data"].as_array().unwrap().len(),
        2,
        "page should have 2 items"
    );

    // Page 3 with page_size=2 should have 1 item
    let page3 = app.get("/api/v1/invoices?page=3&page_size=2").await;
    page3.assert_status_ok();
    let body3: serde_json::Value = page3.json();
    assert_eq!(
        body3["data"].as_array().unwrap().len(),
        1,
        "last page should have 1 item"
    );
}

#[tokio::test]
async fn s02_invoice_csv_export() {
    let app = common::TestApp::new().await;

    // Create an invoice
    app.post("/api/v1/invoices")
        .json(&json!({
            "invoice_number": "EXP-001",
            "amount": 7777.0,
            "invoice_type": "expense",
            "currency": "EUR"
        }))
        .await
        .assert_status_ok();

    // Export as CSV
    let csv_res = app.get("/api/v1/export/invoices").await;
    csv_res.assert_status_ok();
    let csv_text = csv_res.text();
    assert!(
        csv_text.contains("EXP-001"),
        "CSV should contain invoice number"
    );
    assert!(csv_text.contains("7777"), "CSV should contain amount");
}

#[tokio::test]
async fn s02_invoice_tenant_isolation() {
    let (app_a, app_b) = {
        let app_a = common::TestApp::new().await;
        let app_b = app_a.create_second_tenant().await;
        (app_a, app_b)
    };

    // User A creates invoice
    app_a
        .post("/api/v1/invoices")
        .json(&json!({
            "invoice_number": "A-001",
            "amount": 100.0,
            "invoice_type": "income"
        }))
        .await
        .assert_status_ok();

    // User B should see 0 invoices
    let res = app_b.get("/api/v1/invoices").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["count"], 0, "tenant B should see 0 of A's invoices");
}
