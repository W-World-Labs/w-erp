use crate::common;

use chrono::{Duration, Utc};
use serde_json::{Value, json};

async fn seed_company_info(app: &common::TestApp) {
    sqlx::query(
        "INSERT INTO company_info (id, company_id, company_name) VALUES (?, ?, 'Onboarding Co')",
    )
    .bind(format!("onboarding-company-{}", app.company_id))
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed company info");
}

async fn seed_employee(app: &common::TestApp) {
    common::seed::create_employee(&app.pool, &app.company_id, "Onboarding Employee", 1000.0).await;
}

async fn seed_contract(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO contracts (id, company_id, contract_name)
        VALUES ('onboarding-contract', ?, 'Onboarding Contract')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed contract");
}

async fn seed_client(app: &common::TestApp, id: &str, name: &str) {
    sqlx::query(
        r#"
        INSERT INTO clients (id, company_id, name, client_type)
        VALUES (?, ?, ?, 'active')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
    .execute(&app.pool)
    .await
    .expect("seed client");
}

async fn seed_quote(app: &common::TestApp, id: &str, client_id: &str, invoice_id: Option<&str>) {
    sqlx::query(
        r#"
        INSERT INTO quotes (
            id, company_id, client_id, quote_number, title, status, total, invoice_id
        )
        VALUES (?, ?, ?, 'QT-GOLD-001', 'Golden path quote', 'invoiced', 1200.0, ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(client_id)
    .bind(invoice_id)
    .execute(&app.pool)
    .await
    .expect("seed quote");
}

async fn seed_invoice(app: &common::TestApp, id: &str, status: &str) {
    sqlx::query(
        r#"
        INSERT INTO invoices (
            id, company_id, invoice_number, invoice_type, amount, status, supplier, due_date
        )
        VALUES (?, ?, 'INV-GOLD-001', 'receivable', 1200.0, ?, 'Golden Client', '2026-06-30')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(status)
    .execute(&app.pool)
    .await
    .expect("seed invoice");
}

async fn seed_payment(app: &common::TestApp, id: &str, invoice_id: &str) {
    sqlx::query(
        r#"
        INSERT INTO payments (
            id, company_id, payment_no, payment_type, amount, status, payer, invoice_id, payment_date
        )
        VALUES (?, ?, 'PAY-GOLD-001', 'income', 1200.0, 'completed', 'Golden Client', ?, '2026-06-17')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(invoice_id)
    .execute(&app.pool)
    .await
    .expect("seed payment");
}

async fn seed_operating_backbone(app: &common::TestApp) {
    seed_operating_backbone_with_options(app, true, true).await;
}

async fn seed_operating_backbone_with_options(
    app: &common::TestApp,
    connected_payroll: bool,
    linked_reminder: bool,
) {
    let today = Utc::now().date_naive();
    let future = (today + Duration::days(90)).format("%Y-%m-%d").to_string();
    let current_month = today.format("%Y-%m").to_string();
    let project_id = "onboarding-op-project";
    let contract_id = "onboarding-op-contract";
    let active_employee_id = "onboarding-op-employee";
    let inactive_employee_id = "onboarding-op-inactive-employee";
    let salary_employee_id = if connected_payroll {
        active_employee_id
    } else {
        inactive_employee_id
    };

    sqlx::query(
        r#"
        INSERT INTO projects (id, company_id, name, status, progress, budget)
        VALUES (?, ?, 'Operating Backbone Project', 'active', 25, 5000)
        "#,
    )
    .bind(project_id)
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed project");

    sqlx::query(
        r#"
        INSERT INTO contracts (id, company_id, contract_name, contract_type, status, amount, end_date, project_id)
        VALUES (?, ?, 'Operating Backbone Contract', 'service', 'active', 5000, ?, ?)
        "#,
    )
    .bind(contract_id)
    .bind(&app.company_id)
    .bind(&future)
    .bind(project_id)
    .execute(&app.pool)
    .await
    .expect("seed contract");

    sqlx::query(
        r#"
        INSERT INTO employees (id, company_id, name, department, position, status, base_salary)
        VALUES (?, ?, 'Operating Employee', 'Ops', 'Manager', 'active', 1000)
        "#,
    )
    .bind(active_employee_id)
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed active employee");

    if !connected_payroll {
        sqlx::query(
            r#"
            INSERT INTO employees (id, company_id, name, department, position, status, base_salary)
            VALUES (?, ?, 'Inactive Payroll Employee', 'Ops', 'Analyst', 'inactive', 1000)
            "#,
        )
        .bind(inactive_employee_id)
        .bind(&app.company_id)
        .execute(&app.pool)
        .await
        .expect("seed inactive employee");
    }

    sqlx::query(
        r#"
        INSERT INTO salary_records (
            id, company_id, employee_id, employee_name, period, base_salary, net_salary, status
        )
        VALUES (?, ?, ?, 'Operating Employee', ?, 1000, 1000, 'pending')
        "#,
    )
    .bind("onboarding-op-salary")
    .bind(&app.company_id)
    .bind(salary_employee_id)
    .bind(&current_month)
    .execute(&app.pool)
    .await
    .expect("seed salary");

    sqlx::query(
        r#"
        INSERT INTO assets (id, company_id, name, asset_type, status, current_value, expiry_date, contract_id)
        VALUES (?, ?, 'Operating Asset', 'intangible', 'active', 8000, ?, ?)
        "#,
    )
    .bind("onboarding-op-asset")
    .bind(&app.company_id)
    .bind(&future)
    .bind(contract_id)
    .execute(&app.pool)
    .await
    .expect("seed asset");

    sqlx::query(
        r#"
        INSERT INTO licenses (id, company_id, license_name, license_type, status, expiry_date)
        VALUES (?, ?, 'Operating License', 'business_license', 'valid', ?)
        "#,
    )
    .bind("onboarding-op-license")
    .bind(&app.company_id)
    .bind(&future)
    .execute(&app.pool)
    .await
    .expect("seed license");

    sqlx::query(
        r#"
        INSERT INTO rent_records (
            id, company_id, location_name, property_type, monthly_rent, status, contract_end_date, contract_id
        )
        VALUES (?, ?, 'Operating Office', 'office', 3000, 'active', ?, ?)
        "#,
    )
    .bind("onboarding-op-rent")
    .bind(&app.company_id)
    .bind(&future)
    .bind(contract_id)
    .execute(&app.pool)
    .await
    .expect("seed rent");

    sqlx::query(
        r#"
        INSERT INTO budgets (id, company_id, category, budget_type, period, budget_amount, actual_amount, status, project_id)
        VALUES (?, ?, 'operations', 'monthly', ?, 10000, 1000, 'active', ?)
        "#,
    )
    .bind("onboarding-op-budget")
    .bind(&app.company_id)
    .bind(&current_month)
    .bind(project_id)
    .execute(&app.pool)
    .await
    .expect("seed budget");

    let (entity_type, entity_id): (Option<&str>, Option<&str>) = if linked_reminder {
        (Some("contracts"), Some(contract_id))
    } else {
        (None, None)
    };
    sqlx::query(
        r#"
        INSERT INTO reminders (
            id, company_id, title, description, reminder_type, priority, due_date,
            status, is_completed, entity_type, entity_id
        )
        VALUES (?, ?, 'Review operating contract', 'Renewal check', 'contract', 'medium', ?, 'pending', 0, ?, ?)
        "#,
    )
    .bind("onboarding-op-reminder")
    .bind(&app.company_id)
    .bind(&future)
    .bind(entity_type)
    .bind(entity_id)
    .execute(&app.pool)
    .await
    .expect("seed reminder");
}

fn assert_backbone_step_incomplete(body: &Value, step_id: &str) {
    let backbone = &body["summary"]["operating_backbone"];

    assert_eq!(backbone["completed"].as_bool(), Some(false));
    assert_eq!(backbone["current_step_id"], step_id);
    let step = backbone["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|step| step["id"] == step_id)
        .expect("backbone step");
    assert_eq!(step["completed"].as_bool(), Some(false));
}

#[tokio::test]
async fn s122_onboarding_progress_returns_modules_steps_and_display_labels() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_company_info(&app).await;
    seed_employee(&app).await;
    seed_contract(&app).await;
    seed_company_info(&other).await;

    let res = app.get("/api/v1/onboarding/progress").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    let summary = &body["summary"];
    assert_eq!(summary["completed_count"].as_i64().unwrap(), 3);
    assert_eq!(summary["total_count"].as_i64().unwrap(), 15);
    assert_eq!(summary["total_records"].as_i64().unwrap(), 3);
    assert_eq!(summary["progress_rate"].as_i64().unwrap(), 20);
    assert_eq!(summary["progress_rate_label"], "20%");
    assert_eq!(summary["completed_label"], "已完成 3/15 个模块");
    assert_eq!(summary["total_records_label"], "共录入 3 条数据");
    // Activation = Tier 1 value path; only `company` is seeded among the six.
    assert_eq!(summary["activation"]["total_count"].as_i64().unwrap(), 6);
    assert_eq!(
        summary["activation"]["completed_count"].as_i64().unwrap(),
        1
    );
    assert_eq!(summary["demo_company"].as_bool(), Some(false));
    assert!(summary["cta"].is_null());

    let modules = body["modules"].as_array().unwrap();
    assert_eq!(modules.len(), 15);
    let company = modules
        .iter()
        .find(|module| module["id"] == "company")
        .expect("company module");
    assert_eq!(company["completed"].as_bool(), Some(true));
    assert_eq!(company["count"].as_i64().unwrap(), 1);
    assert_eq!(company["count_label"], "1 条数据");
    assert_eq!(company["status_badge"]["label"], "已完成");
    assert_eq!(company["tier"].as_i64().unwrap(), 1);

    let branches = modules
        .iter()
        .find(|module| module["id"] == "branches")
        .expect("branches module");
    assert_eq!(branches["completed"].as_bool(), Some(false));
    assert_eq!(branches["status_badge"]["label"], "待完善");

    let tutorial_steps = body["tutorial_steps"].as_array().unwrap();
    assert_eq!(tutorial_steps.len(), 15);
    assert!(tutorial_steps.iter().any(|step| step["id"] == "contracts"));
    assert_eq!(tutorial_steps[0]["id"], "company");
    assert_eq!(tutorial_steps[0]["completed"].as_bool(), Some(true));
    assert_eq!(tutorial_steps[0]["action_label"], "完善公司信息");
    assert_eq!(tutorial_steps[0]["section_title"], "激活 · 跑通获客到收款");

    assert!(body["incomplete_tip"]["title"].as_str().is_some());
    assert!(body["completed_tip"]["title"].as_str().is_some());
    assert!(
        body["incomplete_tip"]["description"]
            .as_str()
            .unwrap()
            .contains("工作台")
    );
}

#[tokio::test]
async fn s122_onboarding_progress_returns_operating_backbone_contract_when_empty() {
    let app = common::TestApp::new().await;

    let res = app.get("/api/v1/onboarding/progress").await;
    res.assert_status_ok();
    let body = res.json::<Value>();
    let backbone = &body["summary"]["operating_backbone"];

    assert_eq!(backbone["id"], "operating_backbone");
    assert_eq!(backbone["title"], "运营骨架");
    assert_eq!(backbone["completed"].as_bool(), Some(false));
    assert_eq!(backbone["current_step_id"], "contracts");
    assert_eq!(backbone["completed_count"].as_i64(), Some(0));
    assert_eq!(backbone["total_count"].as_i64(), Some(9));
    assert_eq!(backbone["rate"].as_i64(), Some(0));
    assert_eq!(backbone["rate_label"], "0%");
    assert_eq!(backbone["status_badge"]["label"], "待完善");
    assert_eq!(backbone["next_action"]["route"], "/contracts");

    let steps = backbone["steps"].as_array().expect("backbone steps");
    assert_eq!(steps.len(), 9);
    let ids: Vec<&str> = steps
        .iter()
        .filter_map(|step| step["id"].as_str())
        .collect();
    assert_eq!(
        ids,
        vec![
            "contracts",
            "projects",
            "employees",
            "payroll",
            "assets",
            "licenses",
            "rent",
            "budgets",
            "reminders"
        ]
    );
}

#[tokio::test]
async fn s122_onboarding_progress_marks_operating_backbone_complete() {
    let app = common::TestApp::new().await;
    seed_operating_backbone(&app).await;

    let res = app.get("/api/v1/onboarding/progress").await;
    res.assert_status_ok();
    let body = res.json::<Value>();
    let backbone = &body["summary"]["operating_backbone"];

    assert_eq!(backbone["completed"].as_bool(), Some(true));
    assert_eq!(backbone["current_step_id"], "complete");
    assert_eq!(backbone["completed_count"].as_i64(), Some(9));
    assert_eq!(backbone["total_count"].as_i64(), Some(9));
    assert_eq!(backbone["rate"].as_i64(), Some(100));
    assert_eq!(backbone["rate_label"], "100%");
    assert_eq!(backbone["next_action"]["route"], "/reminders");
    assert_eq!(backbone["status_badge"]["label"], "已完成");
    assert!(backbone["steps"].as_array().unwrap().iter().all(|step| {
        step["completed"].as_bool() == Some(true) && step["evidence_label"].as_str().is_some()
    }));
}

#[tokio::test]
async fn s122_onboarding_progress_rejects_disconnected_operating_payroll() {
    let app = common::TestApp::new().await;
    seed_operating_backbone_with_options(&app, false, true).await;

    let res = app.get("/api/v1/onboarding/progress").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_backbone_step_incomplete(&body, "payroll");
}

#[tokio::test]
async fn s122_onboarding_progress_rejects_uncontracted_operating_project() {
    let app = common::TestApp::new().await;
    seed_operating_backbone(&app).await;

    sqlx::query("UPDATE contracts SET project_id = NULL WHERE id = 'onboarding-op-contract'")
        .execute(&app.pool)
        .await
        .expect("disconnect contract from project");

    let res = app.get("/api/v1/onboarding/progress").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_backbone_step_incomplete(&body, "projects");
}

#[tokio::test]
async fn s122_onboarding_progress_rejects_uncontracted_operating_assets() {
    let app = common::TestApp::new().await;
    seed_operating_backbone(&app).await;

    sqlx::query("UPDATE assets SET contract_id = NULL WHERE id = 'onboarding-op-asset'")
        .execute(&app.pool)
        .await
        .expect("disconnect asset from contract");

    let res = app.get("/api/v1/onboarding/progress").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_backbone_step_incomplete(&body, "assets");
}

#[tokio::test]
async fn s122_onboarding_progress_rejects_uncontracted_operating_rent() {
    let app = common::TestApp::new().await;
    seed_operating_backbone(&app).await;

    sqlx::query("UPDATE rent_records SET contract_id = NULL WHERE id = 'onboarding-op-rent'")
        .execute(&app.pool)
        .await
        .expect("disconnect rent from contract");

    let res = app.get("/api/v1/onboarding/progress").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_backbone_step_incomplete(&body, "rent");
}

#[tokio::test]
async fn s122_onboarding_progress_rejects_unprojected_operating_budget() {
    let app = common::TestApp::new().await;
    seed_operating_backbone(&app).await;

    sqlx::query("UPDATE budgets SET project_id = NULL WHERE id = 'onboarding-op-budget'")
        .execute(&app.pool)
        .await
        .expect("disconnect budget from project");

    let res = app.get("/api/v1/onboarding/progress").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_backbone_step_incomplete(&body, "budgets");
}

#[tokio::test]
async fn s122_onboarding_progress_rejects_reminders_with_missing_operating_target() {
    let app = common::TestApp::new().await;
    seed_operating_backbone(&app).await;

    sqlx::query(
        "UPDATE reminders SET entity_id = 'missing-contract' WHERE id = 'onboarding-op-reminder'",
    )
    .execute(&app.pool)
    .await
    .expect("point reminder at a missing operating target");

    let res = app.get("/api/v1/onboarding/progress").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_backbone_step_incomplete(&body, "reminders");
}

#[tokio::test]
async fn s122_onboarding_progress_rejects_unlinked_operating_reminders() {
    let app = common::TestApp::new().await;
    seed_operating_backbone_with_options(&app, true, false).await;

    let res = app.get("/api/v1/onboarding/progress").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_backbone_step_incomplete(&body, "reminders");
}

#[tokio::test]
async fn s122_onboarding_progress_marks_connected_paid_invoice_loop_complete() {
    let app = common::TestApp::new().await;

    seed_company_info(&app).await;
    seed_client(&app, "gold-client", "Golden Client").await;
    seed_quote(&app, "gold-quote", "gold-client", Some("gold-invoice")).await;
    seed_invoice(&app, "gold-invoice", "paid").await;
    seed_payment(&app, "gold-payment", "gold-invoice").await;

    let res = app.get("/api/v1/onboarding/progress").await;
    res.assert_status_ok();
    let body = res.json::<Value>();
    let loop_state = &body["summary"]["activation_loop"];

    assert_eq!(loop_state["id"], "first_paid_invoice");
    assert_eq!(loop_state["completed"].as_bool(), Some(true));
    assert_eq!(loop_state["current_step_id"], "complete");
    assert_eq!(loop_state["completed_count"].as_i64(), Some(7));
    assert_eq!(loop_state["total_count"].as_i64(), Some(7));
    assert_eq!(loop_state["rate"].as_i64(), Some(100));
    assert_eq!(loop_state["rate_label"], "100%");
    assert_eq!(loop_state["status_badge"]["label"], "已完成");
    assert_eq!(loop_state["next_action"]["route"], "/payments");

    let steps = loop_state["steps"].as_array().expect("activation steps");
    assert_eq!(steps.len(), 7);
    assert!(steps.iter().any(|step| {
        step["id"] == "invoice_pdf"
            && step["completed"].as_bool() == Some(true)
            && step["route"] == "/invoices"
    }));

    let ar = app.get("/api/v1/ar/aging").await;
    ar.assert_status_ok();
    let ar_body = ar.json::<Value>();
    let ar_json = serde_json::to_string(&ar_body).expect("serialize ar");
    assert!(
        !ar_json.contains("INV-GOLD-001"),
        "paid golden-path invoice must not remain in AR aging"
    );
}

#[tokio::test]
async fn s122_onboarding_progress_rejects_disconnected_paid_invoice_rows() {
    let app = common::TestApp::new().await;

    seed_company_info(&app).await;
    seed_client(&app, "loose-client", "Loose Client").await;
    seed_quote(&app, "loose-quote", "loose-client", None).await;
    seed_invoice(&app, "loose-invoice", "paid").await;
    seed_payment(&app, "loose-payment", "loose-invoice").await;

    let res = app.get("/api/v1/onboarding/progress").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["summary"]["activation"]["completed_count"], 5);
    let loop_state = &body["summary"]["activation_loop"];
    assert_eq!(loop_state["completed"].as_bool(), Some(false));
    assert_eq!(loop_state["current_step_id"], "invoice");
    assert_eq!(loop_state["completed_count"].as_i64(), Some(3));
    assert_eq!(loop_state["total_count"].as_i64(), Some(7));
    assert_eq!(loop_state["next_action"]["route"], "/quotes");
}

#[tokio::test]
async fn s122_onboarding_api_golden_path_completes_first_paid_invoice_loop() {
    let app = common::TestApp::new().await;

    app.post("/api/v1/company_info")
        .json(&json!({
            "company_name": "API Golden Co",
            "legal_representative": "API Founder",
            "taxpayer_type": "small_scale"
        }))
        .await
        .assert_status_ok();

    let client = app
        .post("/api/v1/clients")
        .json(&json!({
            "name": "API Golden Client",
            "client_type": "corporate"
        }))
        .await;
    client.assert_status_ok();
    let client_id = client.json::<Value>()["id"].as_str().unwrap().to_string();

    let quote = app
        .post("/api/v1/quotes")
        .json(&json!({
            "client_id": &client_id,
            "quote_number": "QT-API-GOLD-001",
            "title": "API Golden Path Quote",
            "status": "accepted",
            "subtotal": 1800.0,
            "tax_amount": 180.0,
            "total": 1980.0
        }))
        .await;
    quote.assert_status_ok();
    let quote_id = quote.json::<Value>()["id"].as_str().unwrap().to_string();

    let conversion = app
        .post(&format!("/api/v1/quotes/{quote_id}/convert-to-invoice"))
        .json(&json!({}))
        .await;
    conversion.assert_status_ok();
    let invoice_id = conversion.json::<Value>()["invoice_id"]
        .as_str()
        .unwrap()
        .to_string();

    let pdf = app.get(&format!("/api/v1/invoices/{invoice_id}/pdf")).await;
    pdf.assert_status_ok();
    let pdf_body = pdf.as_bytes();
    assert!(pdf_body.starts_with(b"%PDF"), "invoice PDF must download");

    let payment = app
        .post(&format!("/api/invoices/{invoice_id}/record-payment"))
        .json(&json!({}))
        .await;
    payment.assert_status_ok();
    assert_eq!(payment.json::<Value>()["invoice_id"], invoice_id);

    let invoice = app.get(&format!("/api/invoices/{invoice_id}")).await;
    invoice.assert_status_ok();
    assert_eq!(invoice.json::<Value>()["status"], "paid");

    let progress = app.get("/api/v1/onboarding/progress").await;
    progress.assert_status_ok();
    let progress_body = progress.json::<Value>();
    let loop_state = &progress_body["summary"]["activation_loop"];
    assert_eq!(loop_state["completed"].as_bool(), Some(true));
    assert_eq!(loop_state["current_step_id"], "complete");
    assert_eq!(loop_state["completed_count"].as_i64(), Some(7));

    let ar = app.get("/api/v1/ar/aging").await;
    ar.assert_status_ok();
    let ar_body = ar.json::<Value>();
    let ar_json = serde_json::to_string(&ar_body).expect("serialize ar");
    assert!(
        !ar_json.contains("QT-API-GOLD-001"),
        "paid golden-path invoice must not remain in AR aging"
    );
}

#[tokio::test]
async fn s122_founder_activation_requires_settled_income() {
    let app = common::TestApp::new().await;
    seed_client(&app, "decision-client", "Decision Client").await;
    seed_quote(&app, "decision-quote", "decision-client", Some("decision-invoice")).await;
    seed_invoice(&app, "decision-invoice", "paid").await;
    seed_payment(&app, "decision-payment", "decision-invoice").await;
    for (status, payment_type, expected) in [
        ("pending", "income", false), ("cancelled", "income", false),
        ("completed", "expense", false), ("completed", "income", true), ("paid", "income", true),
    ] {
        sqlx::query("UPDATE payments SET status = ?, payment_type = ? WHERE id = 'decision-payment'")
            .bind(status).bind(payment_type).execute(&app.pool).await.unwrap();
        let body = app.get("/api/v1/onboarding/progress").await.json::<Value>();
        let payment_module = body["modules"].as_array().unwrap().iter().find(|m| m["id"] == "payments").unwrap();
        assert_eq!(payment_module["completed"], expected, "{status}/{payment_type}");
        let payment_step = body["summary"]["activation_loop"]["steps"].as_array().unwrap().iter().find(|m| m["id"] == "payment").unwrap();
        assert_eq!(payment_step["completed"], expected, "{status}/{payment_type}");
    }
}
