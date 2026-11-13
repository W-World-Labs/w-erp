/// Scenario 61: 🔒 E2E Multi-Tenant Isolation via Real HTTP
/// Validates that data created by tenant A is invisible to tenant B
/// through the full HTTP stack (middleware → auth → DB filtering).
use crate::common;
use reqwest::Client;
use serde_json::{Value, json};

/// Start a server with TWO tenants on the SAME database
async fn start_multi_tenant_server() -> (String, String, String, sqlx::SqlitePool) {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("db");
    opc::db::run_migrations(&pool).await.expect("migrations");

    // Tenant A
    let uid_a = common::seed::create_user(&pool, "alpha@corp-dev.example.invalid", "Alpha User").await;
    let cid_a = common::seed::create_company(&pool, &uid_a, "AlphaCorp", "CNY").await;
    let session_a = common::seed::create_session(&pool, &uid_a, &cid_a).await;

    // Tenant B (different user, different company)
    let uid_b = common::seed::create_user(&pool, "beta@rival-dev.example.invalid", "Beta User").await;
    let cid_b = common::seed::create_company(&pool, &uid_b, "BetaCorp", "USD").await;
    let session_b = common::seed::create_session(&pool, &uid_b, &cid_b).await;

    let app = opc::build_app(pool.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let base = format!("http://{}", addr);
    (
        base,
        format!("Bearer {}", session_a),
        format!("Bearer {}", session_b),
        pool,
    )
}

fn c() -> Client {
    Client::new()
}

// ══════════════ Contracts Isolation ══════════════

#[tokio::test]
async fn s61_contracts_tenant_isolation() {
    let (base, auth_a, auth_b, _pool) = start_multi_tenant_server().await;

    // A creates a contract
    let resp = c()
        .post(format!("{}/api/v1/contracts", base))
        .header("Authorization", &auth_a)
        .json(&json!({ "contract_name": "Alpha机密合同", "amount": 999999.0 }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let id = resp.json::<Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // B cannot see A's contracts
    let resp = c()
        .get(format!("{}/api/v1/contracts", base))
        .header("Authorization", &auth_b)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["count"]
            .as_i64()
            .unwrap(),
        0
    );

    // B cannot access A's contract by ID
    let resp = c()
        .get(format!("{}/api/v1/contracts/{}", base, id))
        .header("Authorization", &auth_b)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);

    // B cannot update A's contract
    let resp = c()
        .put(format!("{}/api/v1/contracts/{}", base, id))
        .header("Authorization", &auth_b)
        .json(&json!({ "contract_name": "被篡改了" }))
        .send()
        .await
        .unwrap();
    assert_ne!(resp.status(), 200);

    // B cannot delete A's contract
    let resp = c()
        .delete(format!("{}/api/v1/contracts/{}", base, id))
        .header("Authorization", &auth_b)
        .send()
        .await
        .unwrap();
    assert_ne!(resp.status().as_u16(), 200);
    assert_ne!(resp.status().as_u16(), 204);
}

// ══════════════ Invoices Isolation ══════════════

#[tokio::test]
async fn s61_invoices_tenant_isolation() {
    let (base, auth_a, auth_b, _pool) = start_multi_tenant_server().await;

    c().post(format!("{}/api/invoices", base))
        .header("Authorization", &auth_a)
        .json(&json!({ "invoice_number": "ALPHA-001", "amount": 100000.0 }))
        .send()
        .await
        .unwrap();

    // B sees nothing
    let resp = c()
        .get(format!("{}/api/invoices", base))
        .header("Authorization", &auth_b)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.json::<Value>().await.unwrap()["count"]
            .as_i64()
            .unwrap(),
        0
    );
}

// ══════════════ Employees Isolation ══════════════

#[tokio::test]
async fn s61_employees_tenant_isolation() {
    let (base, auth_a, auth_b, _pool) = start_multi_tenant_server().await;

    c().post(format!("{}/api/employees", base))
        .header("Authorization", &auth_a)
        .json(&json!({ "name": "Alpha员工" }))
        .send()
        .await
        .unwrap();

    let resp = c()
        .get(format!("{}/api/employees", base))
        .header("Authorization", &auth_b)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.json::<Value>().await.unwrap()["count"]
            .as_i64()
            .unwrap(),
        0
    );
}

// ══════════════ Bank Accounts Isolation ══════════════

#[tokio::test]
async fn s61_bank_tenant_isolation() {
    let (base, auth_a, auth_b, _pool) = start_multi_tenant_server().await;

    c().post(format!("{}/api/bank/accounts", base))
        .header("Authorization", &auth_a)
        .json(&json!({
            "bank_name": "Alpha银行",
            "account_name": "Alpha账户",
            "account_number": "ALPHA-9999",
            "balance": 1000000.0
        }))
        .send()
        .await
        .unwrap();

    // B sees no accounts
    let resp = c()
        .get(format!("{}/api/bank/accounts", base))
        .header("Authorization", &auth_b)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.json::<Value>().await.unwrap()["count"]
            .as_i64()
            .unwrap(),
        0
    );
}

// ══════════════ Cross-Tenant ID Injection ══════════════

#[tokio::test]
async fn s61_cross_tenant_id_injection() {
    let (base, auth_a, auth_b, _pool) = start_multi_tenant_server().await;

    // A creates a project
    let resp = c()
        .post(format!("{}/api/v1/projects", base))
        .header("Authorization", &auth_a)
        .json(&json!({ "name": "Alpha绝密项目" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let proj_id = resp.json::<Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // B tries to get A's project → 404
    let resp = c()
        .get(format!("{}/api/v1/projects/{}", base, proj_id))
        .header("Authorization", &auth_b)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);

    // B tries to update A's project → should fail
    let resp = c()
        .put(format!("{}/api/v1/projects/{}", base, proj_id))
        .header("Authorization", &auth_b)
        .json(&json!({ "name": "被篡改项目" }))
        .send()
        .await
        .unwrap();
    assert_ne!(resp.status(), 200);

    // B tries to delete A's project → should fail
    let resp = c()
        .delete(format!("{}/api/v1/projects/{}", base, proj_id))
        .header("Authorization", &auth_b)
        .send()
        .await
        .unwrap();
    assert_ne!(resp.status().as_u16(), 200);
    assert_ne!(resp.status().as_u16(), 204);

    // A can still access their own project
    let resp = c()
        .get(format!("{}/api/v1/projects/{}", base, proj_id))
        .header("Authorization", &auth_a)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.json::<Value>().await.unwrap()["name"], "Alpha绝密项目");
}

// ══════════════ Bidirectional Isolation ══════════════

#[tokio::test]
async fn s61_bidirectional_isolation() {
    let (base, auth_a, auth_b, _pool) = start_multi_tenant_server().await;

    // Both create reminders
    c().post(format!("{}/api/v1/reminders", base))
        .header("Authorization", &auth_a)
        .json(&json!({ "title": "Alpha提醒" }))
        .send()
        .await
        .unwrap();

    c().post(format!("{}/api/v1/reminders", base))
        .header("Authorization", &auth_b)
        .json(&json!({ "title": "Beta提醒" }))
        .send()
        .await
        .unwrap();

    // Each sees only their own
    let a_list = c()
        .get(format!("{}/api/v1/reminders", base))
        .header("Authorization", &auth_a)
        .send()
        .await
        .unwrap();
    let a_data: Value = a_list.json().await.unwrap();
    assert_eq!(a_data["count"].as_i64().unwrap(), 1);
    assert_eq!(a_data["data"][0]["title"], "Alpha提醒");

    let b_list = c()
        .get(format!("{}/api/v1/reminders", base))
        .header("Authorization", &auth_b)
        .send()
        .await
        .unwrap();
    let b_data: Value = b_list.json().await.unwrap();
    assert_eq!(b_data["count"].as_i64().unwrap(), 1);
    assert_eq!(b_data["data"][0]["title"], "Beta提醒");
}
