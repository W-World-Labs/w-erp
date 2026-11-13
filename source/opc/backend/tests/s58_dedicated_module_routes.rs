/// Scenario 58: 🔌 Dedicated Module Route Coverage (专用模块路由覆盖)
/// Tests: bank transactions, invoices, assets, employees, budgets (dedicated modules)
///        + policies, global_opc_hubs, files static
use crate::common;
use axum::http::StatusCode;
use serde_json::json;

// ══════════════ Bank Module (银行模块) ══════════════

#[tokio::test]
async fn s58_bank_create_transaction() {
    let app = common::TestApp::new().await;

    // CreateAccountRequest requires: bank_name, account_name, account_number
    let acct = app
        .post("/api/bank/accounts")
        .json(&json!({
            "bank_name": "工商银行",
            "account_name": "公司基本户",
            "account_number": "0002000000001234",
            "account_type": "checking",
            "currency": "CNY",
            "balance": 100000.0
        }))
        .await;
    acct.assert_status_ok();
    let acct_id = acct.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // CreateTransactionRequest requires: account_id, transaction_type, amount, transaction_date
    let tx = app
        .post("/api/bank/transactions")
        .json(&json!({
            "account_id": acct_id,
            "transaction_type": "deposit",
            "amount": 5000.0,
            "description": "客户回款",
            "transaction_date": "2026-03-20"
        }))
        .await;
    tx.assert_status_ok();
    let body: serde_json::Value = tx.json();
    assert_eq!(body["balance_after"].as_f64().unwrap(), 105000.0);
}

#[tokio::test]
async fn s58_bank_transaction_alias_get_consistency() {
    let app = common::TestApp::new().await;

    let acct = app
        .post("/api/bank/accounts")
        .json(&json!({
            "bank_name": "建设银行",
            "account_name": "别名测试户",
            "account_number": "9553300000000001",
            "account_type": "checking",
            "currency": "CNY",
            "balance": 8000.0
        }))
        .await;
    acct.assert_status_ok();
    let acct_id = acct.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let tx = app
        .post("/api/bank/transactions")
        .json(&json!({
            "account_id": acct_id,
            "transaction_type": "deposit",
            "amount": 1200.0,
            "description": "别名一致性入账",
            "transaction_date": "2026-03-21"
        }))
        .await;
    tx.assert_status_ok();
    let tx_id = tx.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let slash_alias = app
        .get(&format!("/api/v1/bank/transactions/{}", tx_id))
        .await;
    slash_alias.assert_status_ok();
    let slash_body: serde_json::Value = slash_alias.json();
    assert_eq!(slash_body["id"], tx_id);
    assert_eq!(slash_body["transaction_type"], "deposit");

    let flat_alias = app
        .get(&format!("/api/v1/bank_transactions/{}", tx_id))
        .await;
    flat_alias.assert_status_ok();
    let flat_body: serde_json::Value = flat_alias.json();
    assert_eq!(flat_body["id"], tx_id);
    assert_eq!(flat_body["amount"], 1200.0);
}

// ══════════════ Invoices Module (发票模块) ══════════════

#[tokio::test]
async fn s58_invoices_full_crud() {
    let app = common::TestApp::new().await;

    // CreateInvoiceRequest requires: invoice_number, amount
    let res = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-2026-001",
            "amount": 15000.0,
            "due_date": "2026-04-20"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["invoice_number"]
        .as_str()
        .map(|_| {
            res.json::<serde_json::Value>()["id"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .unwrap_or_else(|| {
            res.json::<serde_json::Value>()["id"]
                .as_str()
                .unwrap()
                .to_string()
        });
    // Re-fetch ID
    let cre: serde_json::Value = res.json();
    let id = cre["id"].as_str().unwrap().to_string();

    // Get
    let get: serde_json::Value = app.get(&format!("/api/invoices/{}", id)).await.json();
    assert_eq!(get["invoice_number"].as_str().unwrap(), "INV-2026-001");

    // Update — uses same CreateInvoiceRequest struct, so all required fields needed
    app.put(&format!("/api/invoices/{}", id))
        .json(&json!({
            "invoice_number": "INV-2026-001",
            "amount": 16000.0,
            "due_date": "2026-04-25"
        }))
        .await
        .assert_status_ok();

    // List
    let list = app.get("/api/invoices").await;
    list.assert_status_ok();
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 1);

    // Delete
    let del = app.delete(&format!("/api/invoices/{}", id)).await;
    assert_eq!(del.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn s58_invoices_tenant_isolation() {
    let app_a = common::TestApp::new().await;
    let app_b = app_a.create_second_tenant().await;

    app_a
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "ISO-A-001",
            "amount": 1000.0
        }))
        .await
        .assert_status_ok();

    let b_list = app_b.get("/api/invoices").await;
    assert_eq!(
        b_list.json::<serde_json::Value>()["count"]
            .as_i64()
            .unwrap(),
        0
    );
}

// ══════════════ Assets Module (资产模块) ══════════════

#[tokio::test]
async fn s58_assets_full_crud() {
    let app = common::TestApp::new().await;

    // CreateAssetRequest requires: name (rest optional)
    let res = app
        .post("/api/assets")
        .json(&json!({
            "name": "MacBook Pro 16",
            "asset_type": "physical",
            "purchase_price": 25999.0,
            "purchase_date": "2026-01-15"
        }))
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    let id = body["data"]["id"].as_str().unwrap().to_string();

    let get_res = app.get(&format!("/api/assets/{}", id)).await;
    let get: serde_json::Value = get_res.json();
    assert_eq!(get["data"]["name"].as_str().unwrap(), "MacBook Pro 16");

    // Update — uses same CreateAssetRequest struct
    app.put(&format!("/api/assets/{}", id))
        .json(&json!({ "name": "MacBook Pro 16 (已报废)" }))
        .await
        .assert_status_ok();

    let list = app.get("/api/assets").await;
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 1);

    app.delete(&format!("/api/assets/{}", id)).await;
}

#[tokio::test]
async fn s58_assets_tenant_isolation() {
    let app_a = common::TestApp::new().await;
    let app_b = app_a.create_second_tenant().await;

    app_a
        .post("/api/assets")
        .json(&json!({
            "name": "隔离测试资产"
        }))
        .await
        .assert_status_ok();

    let b_list = app_b.get("/api/assets").await;
    assert_eq!(
        b_list.json::<serde_json::Value>()["count"]
            .as_i64()
            .unwrap(),
        0
    );
}

// ══════════════ Employees Module (员工模块) ══════════════

#[tokio::test]
async fn s58_employees_full_crud() {
    let app = common::TestApp::new().await;

    // EmployeeRequest requires: name (rest optional)
    let res = app
        .post("/api/employees")
        .json(&json!({
            "name": "张三",
            "position": "工程师",
            "department": "研发部",
            "base_salary": 20000.0,
            "status": "active",
            "hire_date": "2026-01-01"
        }))
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    let id = body["id"].as_str().unwrap().to_string();

    let get: serde_json::Value = app.get(&format!("/api/employees/{}", id)).await.json();
    assert_eq!(get["name"].as_str().unwrap(), "张三");
    assert_eq!(get["department"].as_str().unwrap(), "研发部");

    // Update — uses same EmployeeRequest struct
    app.put(&format!("/api/employees/{}", id))
        .json(&json!({
            "name": "张三",
            "position": "高级工程师",
            "base_salary": 25000.0
        }))
        .await
        .assert_status_ok();

    let list = app.get("/api/employees").await;
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 1);

    app.delete(&format!("/api/employees/{}", id)).await;
}

#[tokio::test]
async fn s58_employees_tenant_isolation() {
    let app_a = common::TestApp::new().await;
    let app_b = app_a.create_second_tenant().await;

    app_a
        .post("/api/employees")
        .json(&json!({
            "name": "隔离员工"
        }))
        .await
        .assert_status_ok();

    let b_list = app_b.get("/api/employees").await;
    assert_eq!(
        b_list.json::<serde_json::Value>()["count"]
            .as_i64()
            .unwrap(),
        0
    );
}

// ══════════════ Budgets Module (预算模块) ══════════════

#[tokio::test]
async fn s58_budgets_full_crud() {
    let app = common::TestApp::new().await;

    // CreateBudgetRequest requires: category, budget_type, period
    let res = app
        .post("/api/budgets")
        .json(&json!({
            "category": "development",
            "budget_type": "quarterly",
            "period": "2026-Q1",
            "budget_amount": 500000.0,
            "notes": "Q1研发预算"
        }))
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    let id = body["id"].as_str().unwrap().to_string();

    let get: serde_json::Value = app.get(&format!("/api/budgets/{}", id)).await.json();
    assert_eq!(get["category"].as_str().unwrap(), "development");
    assert_eq!(get["budget_type"].as_str().unwrap(), "quarterly");

    // Update — uses same CreateBudgetRequest struct
    app.put(&format!("/api/budgets/{}", id))
        .json(&json!({
            "category": "development",
            "budget_type": "quarterly",
            "period": "2026-Q1",
            "budget_amount": 550000.0
        }))
        .await
        .assert_status_ok();

    let list = app.get("/api/budgets").await;
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 1);

    app.delete(&format!("/api/budgets/{}", id)).await;
}

// ══════════════ Policies (政策全局模块) ══════════════

#[tokio::test]
async fn s58_policy_list_and_detail() {
    let app = common::TestApp::new().await;

    // List policies (global, seeded)
    let list = app.get("/api/policies").await;
    list.assert_status_ok();
    let body: serde_json::Value = list.json();
    assert!(body["data"].is_array());
}

#[tokio::test]
async fn s58_policy_applications_crud() {
    let app = common::TestApp::new().await;

    // policy_applications required: ["policy_id"] with FK to policies table
    // We need a valid policy_id from the seeded policies
    let policies = app.get("/api/policies").await;
    let pol_data: serde_json::Value = policies.json();
    let policy_arr = pol_data["data"].as_array();

    // Skip if no policies are seeded (foreign key constraint)
    let policy_id = match policy_arr
        .and_then(|arr| arr.first())
        .and_then(|p| p["id"].as_str())
    {
        Some(id) => id.to_string(),
        None => {
            // No policies seeded — verify list returns 0 and skip create
            let apps = app.get("/api/policies/applications").await;
            apps.assert_status_ok();
            assert_eq!(
                apps.json::<serde_json::Value>()["count"].as_i64().unwrap(),
                0
            );
            return;
        }
    };

    let res = app
        .post("/api/policies/applications")
        .json(&json!({
            "policy_id": policy_id,
            "status": "preparing",
            "applied_date": "2026-03-01",
            "notes": "准备申报材料"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let get: serde_json::Value = app
        .get(&format!("/api/policies/applications/{}", id))
        .await
        .json();
    assert_eq!(get["policy_id"].as_str().unwrap(), policy_id);

    app.put(&format!("/api/policies/applications/{}", id))
        .json(&json!({ "status": "submitted" }))
        .await
        .assert_status_ok();

    app.delete(&format!("/api/policies/applications/{}", id))
        .await;
}

// ══════════════ Global OPC Hubs ══════════════

#[tokio::test]
async fn s58_global_opc_hubs_list() {
    let app = common::TestApp::new().await;

    let list = app.get("/api/global_opc_hubs").await;
    list.assert_status_ok();
    let body: serde_json::Value = list.json();
    assert!(body["data"].is_array());
}

// ══════════════ Files Static Serve ══════════════

#[tokio::test]
async fn s58_files_static_404() {
    let app = common::TestApp::new().await;

    let res = app.get("/api/files/static/nonexistent/path/file.txt").await;
    // Should return 404 for non-existent static file
    assert_eq!(res.status_code(), StatusCode::NOT_FOUND);
}
