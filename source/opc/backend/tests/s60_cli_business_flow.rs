/// Scenario 60: 🔗 E2E Business Flow — Cross-Module Chains
/// Real HTTP tests validating complete business workflows spanning multiple entities.
use crate::common;
use reqwest::Client;
use serde_json::{Value, json};

async fn start_real_server() -> (String, String, sqlx::SqlitePool) {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("db");
    opc::db::run_migrations(&pool).await.expect("migrations");
    let uid = common::seed::create_user(&pool, "flow@test-dev.example.invalid", "Flow User").await;
    let cid = common::seed::create_company(&pool, &uid, "FlowCorp", "CNY").await;
    let session = common::seed::create_session(&pool, &uid, &cid).await;
    let app = opc::build_app(pool.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (
        format!("http://{}", addr),
        format!("Bearer {}", session),
        pool,
    )
}

fn c() -> Client {
    Client::new()
}

// ══════════════ Flow 1: Client → Quote → Invoice ══════════════

#[tokio::test]
async fn s60_client_to_invoice_flow() {
    let (base, auth, _pool) = start_real_server().await;

    // 1. Create client
    let resp = c()
        .post(format!("{}/api/v1/clients", base))
        .header("Authorization", &auth)
        .json(&json!({ "name": "大客户A", "email": "a@client-com.example.invalid" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let client_id = resp.json::<Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 2. Create contact
    let resp = c()
        .post(format!("{}/api/v1/contacts", base))
        .header("Authorization", &auth)
        .json(&json!({ "name": "联系人张三", "email": "zhang@client-com.example.invalid" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // 3. Create quote
    let resp = c()
        .post(format!("{}/api/v1/quotes", base))
        .header("Authorization", &auth)
        .json(&json!({
            "client_id": client_id,
            "quote_number": "Q-E2E-001",
            "total": 100000.0,
            "valid_until": "2026-06-01",
            "status": "draft"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let quote_id = resp.json::<Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 4. Create invoice linked to client
    let resp = c()
        .post(format!("{}/api/invoices", base))
        .header("Authorization", &auth)
        .json(&json!({
            "invoice_number": "INV-E2E-001",
            "amount": 100000.0,
            "invoice_type": "income",
            "supplier": "大客户A",
            "due_date": "2026-04-30"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let inv_id = resp.json::<Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 5. Verify both exist
    let resp = c()
        .get(format!("{}/api/v1/quotes/{}", base, quote_id))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let resp = c()
        .get(format!("{}/api/invoices/{}", base, inv_id))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

// ══════════════ Flow 2: Employee → Salary → Payment ══════════════

#[tokio::test]
async fn s60_employee_payroll_flow() {
    let (base, auth, _pool) = start_real_server().await;

    // 1. Create employee
    let resp = c()
        .post(format!("{}/api/employees", base))
        .header("Authorization", &auth)
        .json(&json!({
            "name": "李四",
            "department": "工程部",
            "position": "高级工程师",
            "base_salary": 30000.0,
            "hire_date": "2025-01-01"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let emp_id = resp.json::<Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 2. Create salary record
    let resp = c()
        .post(format!("{}/api/v1/salary", base))
        .header("Authorization", &auth)
        .json(&json!({
            "employee_id": emp_id,
            "period": "2026-03",
            "base_salary": 30000.0,
            "bonus": 5000.0,
            "net_salary": 28000.0,
            "status": "pending"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let sal_id = resp.json::<Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 3. Create payment for salary
    let resp = c()
        .post(format!("{}/api/v1/payments", base))
        .header("Authorization", &auth)
        .json(&json!({
            "payment_type": "salary",
            "amount": 28000.0,
            "salary_record_id": sal_id,
            "status": "paid",
            "payment_date": "2026-03-25"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // 4. Verify the chain
    let resp = c()
        .get(format!("{}/api/v1/salary/{}", base, sal_id))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

// ══════════════ Flow 3: Project → Contract → Budget ══════════════

#[tokio::test]
async fn s60_project_contract_budget_flow() {
    let (base, auth, _pool) = start_real_server().await;

    // 1. Create project
    let resp = c()
        .post(format!("{}/api/v1/projects", base))
        .header("Authorization", &auth)
        .json(&json!({
            "name": "E2E项目Alpha",
            "status": "active",
            "budget": 500000.0,
            "start_date": "2026-01-01"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let proj_id = resp.json::<Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 2. Create contract for project
    let resp = c()
        .post(format!("{}/api/v1/contracts", base))
        .header("Authorization", &auth)
        .json(&json!({
            "contract_name": "Alpha研发合同",
            "contract_type": "service",
            "project_id": proj_id,
            "amount": 200000.0,
            "counterparty": "外包商B"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // 3. Create budget for project
    let resp = c()
        .post(format!("{}/api/budgets", base))
        .header("Authorization", &auth)
        .json(&json!({
            "category": "development",
            "budget_type": "project",
            "period": "2026",
            "budget_amount": 500000.0,
            "project_id": proj_id
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // 4. Verify project
    let resp = c()
        .get(format!("{}/api/v1/projects/{}", base, proj_id))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.json::<Value>().await.unwrap()["name"], "E2E项目Alpha");
}

// ══════════════ Flow 4: Reports & Dashboard ══════════════

#[tokio::test]
async fn s60_reports_and_dashboard_flow() {
    let (base, auth, _pool) = start_real_server().await;

    // Create some data first
    c().post(format!("{}/api/invoices", base))
        .header("Authorization", &auth)
        .json(&json!({ "invoice_number": "RPT-001", "amount": 50000.0, "invoice_type": "income" }))
        .send()
        .await
        .unwrap();
    c().post(format!("{}/api/invoices", base))
        .header("Authorization", &auth)
        .json(&json!({ "invoice_number": "RPT-002", "amount": 10000.0, "invoice_type": "expense" }))
        .send()
        .await
        .unwrap();

    // Dashboard
    let resp = c()
        .get(format!("{}/api/v1/stats/dashboard", base))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Financial health report
    let resp = c()
        .get(format!("{}/api/v1/reports/financial-health", base))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Cashflow forecast
    let resp = c()
        .get(format!("{}/api/v1/reports/cashflow-forecast", base))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Profit analysis
    let resp = c()
        .get(format!("{}/api/v1/reports/profit-analysis", base))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

// ══════════════ Flow 5: CRM Deal Pipeline ══════════════

#[tokio::test]
async fn s60_crm_deal_pipeline_flow() {
    let (base, auth, _pool) = start_real_server().await;

    // Create client
    let resp = c()
        .post(format!("{}/api/v1/clients", base))
        .header("Authorization", &auth)
        .json(&json!({ "name": "CRM客户", "email": "crm@deal-com.example.invalid" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let client_id = resp.json::<Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create deal
    let resp = c()
        .post(format!("{}/api/v1/deals", base))
        .header("Authorization", &auth)
        .json(&json!({
            "title": "大额交易",
            "client_id": client_id,
            "amount": 1000000.0,
            "stage": "negotiation",
            "probability": 70
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let deal_id = resp.json::<Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create interaction
    let resp = c()
        .post(format!("{}/api/v1/interactions", base))
        .header("Authorization", &auth)
        .json(&json!({
            "client_id": client_id,
            "interaction_type": "meeting",
            "interaction_date": "2026-03-21",
            "subject": "合同谈判"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Update deal stage
    let resp = c()
        .put(format!("{}/api/v1/deals/{}", base, deal_id))
        .header("Authorization", &auth)
        .json(&json!({ "stage": "won", "probability": 100 }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}
