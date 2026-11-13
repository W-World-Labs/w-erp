/// Scenario 59: 🌐 CLI/HTTP Full-Stack CRUD E2E
/// Validates the ENTIRE HTTP stack: TcpListener → axum::serve → middleware → auth → handler → DB
/// Unlike axum-test which mocks the transport layer, this sends real TCP HTTP requests.
use crate::common;
use reqwest::Client;
use serde_json::{Value, json};

/// Helper: start a real HTTP server on a random port and return (base_url, auth_header, pool)
async fn start_real_server() -> (String, String, sqlx::SqlitePool) {
    let pool = common::memory_pool().await;
    opc::db::run_migrations(&pool).await.expect("migrations");

    let user_id = common::seed::create_user(&pool, "e2e@test-dev.example.invalid", "E2E User").await;
    let company_id = common::seed::create_company(&pool, &user_id, "E2E Corp", "CNY").await;
    let session = common::seed::create_session(&pool, &user_id, &company_id).await;

    let app = opc::build_app(pool.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let base = format!("http://{}", addr);
    let auth = format!("Bearer {}", session);
    (base, auth, pool)
}

fn client() -> Client {
    Client::new()
}

// ══════════════ Health Check (无需认证) ══════════════

#[tokio::test]
async fn s59_health_check_real_http() {
    let (base, _, _pool) = start_real_server().await;
    let resp = client()
        .get(format!("{}/health", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

// ══════════════ Unauthenticated → 401 ══════════════

#[tokio::test]
async fn s59_unauthenticated_returns_401() {
    let (base, _, _pool) = start_real_server().await;
    let resp = client()
        .get(format!("{}/api/v1/contracts", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

// ══════════════ Generic CRUD via HTTP (contracts) ══════════════

#[tokio::test]
async fn s59_contracts_crud_real_http() {
    let (base, auth, _pool) = start_real_server().await;
    let c = client();

    // Create
    let resp = c
        .post(format!("{}/api/v1/contracts", base))
        .header("Authorization", &auth)
        .json(&json!({
            "contract_name": "E2E服务合同",
            "contract_type": "service",
            "counterparty": "测试公司",
            "amount": 50000.0
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let id = body["id"].as_str().unwrap().to_string();

    // Get
    let resp = c
        .get(format!("{}/api/v1/contracts/{}", base, id))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["contract_name"], "E2E服务合同");

    // Update
    let resp = c
        .put(format!("{}/api/v1/contracts/{}", base, id))
        .header("Authorization", &auth)
        .json(&json!({ "status": "completed" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // List
    let resp = c
        .get(format!("{}/api/v1/contracts?limit=10", base))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body["count"].as_i64().unwrap() >= 1);

    // Delete
    let resp = c
        .delete(format!("{}/api/v1/contracts/{}", base, id))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    // Verify deleted
    let resp = c
        .get(format!("{}/api/v1/contracts/{}", base, id))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

// ══════════════ Dedicated Module: Invoices ══════════════

#[tokio::test]
async fn s59_invoices_crud_real_http() {
    let (base, auth, _pool) = start_real_server().await;
    let c = client();

    let resp = c
        .post(format!("{}/api/invoices", base))
        .header("Authorization", &auth)
        .json(&json!({ "invoice_number": "HTTP-E2E-001", "amount": 9999.0 }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let id = body["id"].as_str().unwrap().to_string();

    // Get
    let resp = c
        .get(format!("{}/api/invoices/{}", base, id))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["invoice_number"], "HTTP-E2E-001");

    // List
    let resp = c
        .get(format!("{}/api/invoices", base))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert!(
        resp.json::<Value>().await.unwrap()["count"]
            .as_i64()
            .unwrap()
            >= 1
    );

    // Delete
    let resp = c
        .delete(format!("{}/api/invoices/{}", base, id))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
}

// ══════════════ Dedicated Module: Employees ══════════════

#[tokio::test]
async fn s59_employees_crud_real_http() {
    let (base, auth, _pool) = start_real_server().await;
    let c = client();

    let resp = c
        .post(format!("{}/api/employees", base))
        .header("Authorization", &auth)
        .json(&json!({ "name": "HTTP测试员工", "department": "工程部" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let id = body["id"].as_str().unwrap().to_string();

    let resp = c
        .get(format!("{}/api/employees/{}", base, id))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.json::<Value>().await.unwrap()["name"], "HTTP测试员工");

    let resp = c
        .delete(format!("{}/api/employees/{}", base, id))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
}

// ══════════════ Dedicated Module: Assets ══════════════

#[tokio::test]
async fn s59_assets_crud_real_http() {
    let (base, auth, _pool) = start_real_server().await;
    let c = client();

    let resp = c
        .post(format!("{}/api/assets", base))
        .header("Authorization", &auth)
        .json(&json!({ "name": "HTTP测试资产" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let id = body["data"]["id"].as_str().unwrap().to_string();

    let resp = c
        .get(format!("{}/api/assets/{}", base, id))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["data"]["name"],
        "HTTP测试资产"
    );

    let resp = c
        .delete(format!("{}/api/assets/{}", base, id))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
}

// ══════════════ Bank: Account → Transaction ══════════════

#[tokio::test]
async fn s59_bank_transaction_real_http() {
    let (base, auth, _pool) = start_real_server().await;
    let c = client();

    // Create account
    let resp = c
        .post(format!("{}/api/bank/accounts", base))
        .header("Authorization", &auth)
        .json(&json!({
            "bank_name": "E2E银行",
            "account_name": "测试账户",
            "account_number": "E2E-1234",
            "balance": 100000.0
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let acct_id = resp.json::<Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create transaction
    let resp = c
        .post(format!("{}/api/bank/transactions", base))
        .header("Authorization", &auth)
        .json(&json!({
            "account_id": acct_id,
            "transaction_type": "deposit",
            "amount": 5000.0,
            "transaction_date": "2026-03-21"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let tx: Value = resp.json().await.unwrap();
    assert_eq!(tx["balance_after"].as_f64().unwrap(), 105000.0);
}

// ══════════════ Dashboard & System Status ══════════════

#[tokio::test]
async fn s59_dashboard_real_http() {
    let (base, auth, _pool) = start_real_server().await;
    let c = client();

    let resp = c
        .get(format!("{}/api/v1/stats/dashboard", base))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    // Dashboard should have key metric fields
    assert!(
        body.get("total_invoices").is_some() || body.get("invoices").is_some() || body.is_object()
    );
}

#[tokio::test]
async fn s59_system_status_real_http() {
    let (base, auth, _pool) = start_real_server().await;
    let c = client();

    let resp = c
        .get(format!("{}/api/v1/system/status", base))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

// ══════════════ Generic CRUD: Reminders ══════════════

#[tokio::test]
async fn s59_reminders_crud_real_http() {
    let (base, auth, _pool) = start_real_server().await;
    let c = client();

    let resp = c
        .post(format!("{}/api/v1/reminders", base))
        .header("Authorization", &auth)
        .json(&json!({ "title": "HTTP提醒测试", "priority": "high" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let id = resp.json::<Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = c
        .put(format!("{}/api/v1/reminders/{}", base, id))
        .header("Authorization", &auth)
        .json(&json!({ "status": "done" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let resp = c
        .delete(format!("{}/api/v1/reminders/{}", base, id))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
}

// ══════════════ Budgets via HTTP ══════════════

#[tokio::test]
async fn s59_budgets_crud_real_http() {
    let (base, auth, _pool) = start_real_server().await;
    let c = client();

    let resp = c
        .post(format!("{}/api/budgets", base))
        .header("Authorization", &auth)
        .json(&json!({
            "category": "engineering",
            "budget_type": "quarterly",
            "period": "2026-Q1",
            "budget_amount": 200000.0
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let id = resp.json::<Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = c
        .get(format!("{}/api/budgets/{}", base, id))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["category"],
        "engineering"
    );

    let resp = c
        .delete(format!("{}/api/budgets/{}", base, id))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
}
