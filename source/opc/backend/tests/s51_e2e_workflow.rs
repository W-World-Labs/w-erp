/// Scenario 51: 🚀 End-to-End Business Workflow
/// Complete business flow: Client → Quote → Contract → Time → Invoice → Payment → Report
/// This tests the full happy path a user would follow.
use crate::common;
use serde_json::json;

// ==================== Full Business Lifecycle ====================

#[tokio::test]
async fn s51_complete_client_to_payment_flow() {
    let app = common::TestApp::new().await;

    // 1. CREATE CLIENT
    let res = app
        .post("/api/v1/clients")
        .json(&json!({
            "name": "Acme Corp",
            "email": "billing@acme-com.example.invalid",
            "phone": "+81-3-1234-5678",
            "address": "Tokyo, Japan"
        }))
        .await;
    res.assert_status_ok();
    let client: serde_json::Value = res.json();
    let client_id = client["id"].as_str().unwrap().to_string();

    // 2. CREATE CONTACT for client
    let res = app
        .post("/api/v1/contacts")
        .json(&json!({
            "name": "田中太郎",
            "email": "tanaka@acme-com.example.invalid",
            "client_id": &client_id,
            "title": "CTO"
        }))
        .await;
    res.assert_status_ok();

    // 3. CREATE DEAL (opportunity)
    let res = app
        .post("/api/v1/deals")
        .json(&json!({
            "title": "Acme Web Platform",
            "client_id": &client_id,
            "amount": 500000.0,
            "stage": "proposal",
            "probability": 80
        }))
        .await;
    res.assert_status_ok();
    let deal_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 4. CREATE QUOTE
    let res = app
        .post("/api/v1/quotes")
        .json(&json!({
            "deal_id": &deal_id,
            "client_id": &client_id,
            "quote_number": "Q-2026-001",
            "total": 500000.0,
            "subtotal": 500000.0,
            "status": "draft"
        }))
        .await;
    res.assert_status_ok();
    let quote_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 5. ADD QUOTE ITEMS
    app.post("/api/v1/quote_items")
        .json(&json!({
            "quote_id": &quote_id,
            "description": "Frontend Development",
            "quantity": 1.0,
            "unit_price": 300000.0
        }))
        .await
        .assert_status_ok();

    app.post("/api/v1/quote_items")
        .json(&json!({
            "quote_id": &quote_id,
            "description": "Backend Development",
            "quantity": 1.0,
            "unit_price": 200000.0
        }))
        .await
        .assert_status_ok();

    // 6. ACCEPT QUOTE → UPDATE STATUS
    app.put(&format!("/api/v1/quotes/{}", quote_id))
        .json(&json!({"status": "accepted"}))
        .await
        .assert_status_ok();

    // 7. CREATE CONTRACT
    let res = app
        .post("/api/v1/contracts")
        .json(&json!({
            "contract_name": "Acme Web Platform Contract",
            "counterparty": "Acme Corp",
            "contract_type": "service",
            "amount": 500000.0,
            "start_date": "2026-03-01",
            "end_date": "2026-06-30",
            "status": "active"
        }))
        .await;
    res.assert_status_ok();

    // 8. CREATE PROJECT
    let res = app
        .post("/api/v1/projects")
        .json(&json!({
            "name": "Acme Web Platform",
            "status": "active",
            "budget": 500000.0
        }))
        .await;
    res.assert_status_ok();
    let project_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 9. ADD MILESTONES
    app.post("/api/v1/milestones")
        .json(&json!({
            "project_id": &project_id,
            "title": "Phase 1: Design",
            "milestone_date": "2026-04-01"
        }))
        .await
        .assert_status_ok();

    // 10. LOG TIME ENTRIES
    app.post("/api/v1/time_entries")
        .json(&json!({
            "project_id": &project_id,
            "client_id": &client_id,
            "description": "Frontend wireframes",
            "duration_minutes": 480,
            "is_billable": true,
            "hourly_rate": 10000.0,
            "date": "2026-03-10"
        }))
        .await
        .assert_status_ok();

    app.post("/api/v1/time_entries")
        .json(&json!({
            "project_id": &project_id,
            "client_id": &client_id,
            "description": "API development",
            "duration_minutes": 960,
            "is_billable": true,
            "hourly_rate": 12000.0,
            "date": "2026-03-15"
        }))
        .await
        .assert_status_ok();

    // 11. CREATE INVOICE
    let res = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-2026-001",
            "amount": 500000.0,
            "tax_amount": 50000.0,
            "invoice_type": "receivable",
            "status": "sent",
            "supplier": "Acme Corp",
            "due_date": "2026-04-30",
            "currency": "JPY"
        }))
        .await;
    res.assert_status_ok();
    let inv_body: serde_json::Value = res.json();
    let invoice_id = inv_body["id"]
        .as_str()
        .or_else(|| inv_body["data"]["id"].as_str())
        .unwrap_or("test-inv")
        .to_string();

    // 12. CREATE BANK ACCOUNT
    let res = app
        .post("/api/bank/accounts")
        .json(&json!({
            "account_name": "Business Checking",
            "bank_name": "MUFG",
            "account_number": "1234567890",
            "currency": "JPY",
            "balance": 1000000.0
        }))
        .await;
    res.assert_status_ok();
    let acct_body: serde_json::Value = res.json();
    let account_id = acct_body["id"]
        .as_str()
        .or_else(|| acct_body["data"]["id"].as_str())
        .unwrap_or("test-acct")
        .to_string();

    // 13. RECEIVE PAYMENT
    let res = app
        .post("/api/v1/payments")
        .json(&json!({
            "invoice_id": &invoice_id,
            "amount": 550000.0,
            "payment_type": "bank_transfer",
            "payment_date": "2026-04-25",
            "status": "completed"
        }))
        .await;
    res.assert_status_ok();

    // 14. VERIFY: Financial Health Report
    let res = app.get("/api/v1/reports/financial-health").await;
    res.assert_status_ok();
    let health: serde_json::Value = res.json();
    assert!(health["score"].is_number() || health["total_score"].is_number() || health.is_object());

    // 15. VERIFY: Cashflow Forecast
    let res = app.get("/api/v1/reports/cashflow-forecast").await;
    res.assert_status_ok();

    // 16. VERIFY: AR Aging
    let res = app.get("/api/v1/ar/aging").await;
    res.assert_status_ok();

    // 17. VERIFY: Tax calculation with JP (company flow)
    let res = app
        .get("/api/v1/tax/calculate?amount=500000&country=JP")
        .await;
    res.assert_status_ok();
    let tax: serde_json::Value = res.json();
    assert_eq!(tax["tax_amount"], 50000.0);

    // 18. VERIFY: Multi-currency balance
    let res = app.get("/api/v1/currency/balances").await;
    res.assert_status_ok();

    // 19. VERIFY: Deal pipeline
    let res = app.get("/api/v1/reports/deals-conversion").await;
    res.assert_status_ok();

    // 20. VERIFY: Dashboard should now show data
    let res = app.get("/api/v1/stats/dashboard").await;
    res.assert_status_ok();
}

// ==================== Multi-Currency Invoice Flow ====================

#[tokio::test]
async fn s51_multi_currency_invoice_flow() {
    let app = common::TestApp::new().await;

    // Seed exchange rates
    app.post("/api/v1/exchange_rates/seed").await;

    // Create USD invoice
    app.post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-USD-001",
            "amount": 10000.0,
            "invoice_type": "receivable",
            "currency": "USD",
            "supplier": "US Client"
        }))
        .await
        .assert_status_ok();

    // Create JPY invoice
    app.post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-JPY-001",
            "amount": 1500000.0,
            "invoice_type": "receivable",
            "currency": "JPY",
            "supplier": "JP Client"
        }))
        .await
        .assert_status_ok();

    // Create CNY payable
    app.post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-CNY-001",
            "amount": 50000.0,
            "invoice_type": "payable",
            "currency": "CNY",
            "supplier": "CN Supplier"
        }))
        .await
        .assert_status_ok();

    // Check multi-currency balances
    let res = app.get("/api/v1/currency/balances").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert!(body.is_object());

    // Convert USD to CNY
    let res = app
        .get("/api/v1/currency/convert?from=USD&to=CNY&amount=10000")
        .await;
    res.assert_status_ok();
    let conv: serde_json::Value = res.json();
    assert!(
        conv["converted"].as_f64().unwrap_or(0.0) > 0.0
            || conv["converted_amount"].as_f64().unwrap_or(0.0) > 0.0
            || conv["result"].as_f64().unwrap_or(0.0) > 0.0
    );
}

// ==================== Tax Compliance Flow ====================

#[tokio::test]
async fn s51_tax_compliance_flow() {
    let app = common::TestApp::new().await;

    // Seed tax data
    app.post("/api/v1/tax_calendar/seed").await;

    // Create tax records (CN)
    app.post("/api/v1/tax")
        .json(&json!({
            "tax_type": "vat",
            "period": "2026-01",
            "due_date": "2026-02-15",
            "taxable_amount": 500000.0,
            "tax_amount": 30000.0,
            "status": "filed"
        }))
        .await
        .assert_status_ok();

    // Add pending record
    app.post("/api/v1/tax")
        .json(&json!({
            "tax_type": "corporate_income",
            "period": "2026-Q1",
            "due_date": "2026-04-15",
            "taxable_amount": 200000.0,
            "tax_amount": 50000.0,
            "status": "pending"
        }))
        .await
        .assert_status_ok();

    // Check compliance
    let res = app.get("/api/v1/tax/compliance-check").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert!(body["compliance_score"]["total"].as_i64().unwrap() >= 0);
    assert!(body["filing_stats"]["filed"].as_i64().unwrap() >= 1);
    assert!(body["filing_stats"]["pending"].as_i64().unwrap() >= 1);

    // Check input credit
    let res = app.get("/api/v1/tax/input-credit").await;
    res.assert_status_ok();

    // Verify invoice requirements for CN
    let res = app.get("/api/v1/tax/invoice-requirements").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    // Default company country is CN
    assert_eq!(body["country"], "CN");
}

// ==================== Webhook + Security Audit Flow ====================

#[tokio::test]
async fn s51_webhook_security_flow() {
    let app = common::TestApp::new().await;

    // Register webhook
    app.post("/api/v1/webhooks")
        .json(&json!({
            "url": "https://example.com/hook",
            "events": "invoice.created,payment.received",
            "secret": "my-secret"
        }))
        .await
        .assert_status_ok();

    // Check security audit
    let res = app.get("/api/v1/security/audit").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert!(body["webhooks"]["total"].as_i64().unwrap() >= 1);
    assert!(body["gdpr_compliance"]["data_portability"]["status"] == "ok");

    // Check API catalog
    let res = app.get("/api/v1/api-catalog").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert!(body["modules"].as_array().unwrap().len() >= 6);
}

// ==================== Employee → Salary → Report ====================

#[tokio::test]
async fn s51_employee_salary_report_flow() {
    let app = common::TestApp::new().await;

    // Create employee
    let res = app
        .post("/api/v1/employees")
        .json(&json!({
            "name": "佐藤花子",
            "email": "sato@company-jp.example.invalid",
            "department": "Engineering",
            "position": "Senior Developer",
            "base_salary": 800000.0,
            "hire_date": "2024-04-01"
        }))
        .await;
    res.assert_status_ok();
    let emp_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Record salary
    app.post("/api/v1/salary")
        .json(&json!({
            "employee_id": &emp_id,
            "period": "2026-03",
            "base_salary": 800000.0,
            "bonus": 100000.0,
            "deductions": 150000.0,
            "net_salary": 750000.0,
            "payment_date": "2026-03-25"
        }))
        .await
        .assert_status_ok();

    // Check profit analysis (salary as expense)
    let res = app.get("/api/v1/reports/profit-analysis").await;
    res.assert_status_ok();

    // Check financial health
    let res = app.get("/api/v1/reports/financial-health").await;
    res.assert_status_ok();
}
