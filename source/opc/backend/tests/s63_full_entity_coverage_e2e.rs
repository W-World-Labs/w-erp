/// Scenario 63: 🔄 E2E Full Entity Coverage — CRM, HR, Cloud, KPI, Tax, Config
/// Tests CRUD lifecycle for every module category via real HTTP.
/// Column names & NOT NULL constraints verified against schema.rs (2026-03-21)
use crate::common;
use reqwest::Client;
use serde_json::{Value, json};

async fn start_real_server() -> (String, String, sqlx::SqlitePool) {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("db");
    opc::db::run_migrations(&pool).await.expect("migrations");

    let user_id = common::seed::create_user(&pool, "full@test-dev.example.invalid", "Full Cov").await;
    let company_id = common::seed::create_company(&pool, &user_id, "Full Corp", "CNY").await;
    let session = common::seed::create_session(&pool, &user_id, &company_id).await;

    let app = opc::build_app(pool.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    (
        format!("http://{}", addr),
        format!("Bearer {}", session),
        pool,
    )
}

fn client() -> Client {
    Client::new()
}

/// Generic CRUD lifecycle: create → get → update → list → delete → 404
async fn crud_lifecycle(
    base: &str,
    auth: &str,
    path: &str,
    create_body: Value,
    update_body: Value,
    label: &str,
) {
    let c = client();
    let url = format!("{}{}", base, path);

    let resp = c
        .post(&url)
        .header("Authorization", auth)
        .json(&create_body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "{}: create failed", label);
    let body: Value = resp.json().await.unwrap();
    let id = body["id"].as_str().unwrap().to_string();

    let resp = c
        .get(format!("{}/{}", url, id))
        .header("Authorization", auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "{}: get failed", label);

    let resp = c
        .put(format!("{}/{}", url, id))
        .header("Authorization", auth)
        .json(&update_body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "{}: update failed", label);

    let resp = c
        .get(format!("{}?limit=10", url))
        .header("Authorization", auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "{}: list failed", label);
    let list: Value = resp.json().await.unwrap();
    assert!(
        list["count"].as_i64().unwrap_or(0) >= 1
            || list["data"].as_array().map(|a| a.len()).unwrap_or(0) >= 1,
        "{}: list empty",
        label
    );

    let resp = c
        .delete(format!("{}/{}", url, id))
        .header("Authorization", auth)
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "{}: delete failed ({})",
        label,
        resp.status()
    );

    let resp = c
        .get(format!("{}/{}", url, id))
        .header("Authorization", auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404, "{}: should 404 after delete", label);
}

// ══════════════ CRM (5) ══════════════

#[tokio::test]
async fn s63_crm_clients() {
    let (b, a, _) = start_real_server().await;
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/clients",
        json!({"name": "E2E客户"}),
        json!({"name": "E2E改"}),
        "clients",
    )
    .await;
}

#[tokio::test]
async fn s63_crm_contacts() {
    let (b, a, _) = start_real_server().await;
    let c = client();
    let cid = c
        .post(format!("{}/api/v1/clients", b))
        .header("Authorization", &a)
        .json(&json!({"name": "联系人客户"}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/contacts",
        json!({"name": "张三", "client_id": cid}),
        json!({"name": "张三改"}),
        "contacts",
    )
    .await;
}

#[tokio::test]
async fn s63_crm_deals() {
    let (b, a, _) = start_real_server().await;
    let c = client();
    let cid = c
        .post(format!("{}/api/v1/clients", b))
        .header("Authorization", &a)
        .json(&json!({"name": "交易客户"}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/deals",
        json!({"title": "交易", "client_id": cid, "amount": 100000, "stage": "prospecting"}),
        json!({"stage": "negotiation"}),
        "deals",
    )
    .await;
}

#[tokio::test]
async fn s63_crm_quotes() {
    let (b, a, _) = start_real_server().await;
    let c = client();
    let cid = c
        .post(format!("{}/api/v1/clients", b))
        .header("Authorization", &a)
        .json(&json!({"name": "报价客户"}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/quotes",
        json!({"quote_number": "Q-001", "client_id": cid, "total": 50000}),
        json!({"status": "accepted"}),
        "quotes",
    )
    .await;
}

#[tokio::test]
async fn s63_crm_interactions() {
    let (b, a, _) = start_real_server().await;
    let c = client();
    let cid = c
        .post(format!("{}/api/v1/clients", b))
        .header("Authorization", &a)
        .json(&json!({"name": "互动客户"}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    // interaction_date NOT NULL
    crud_lifecycle(&b, &a, "/api/v1/interactions",
        json!({"client_id": cid, "interaction_type": "meeting", "subject": "洽谈", "interaction_date": "2026-03-21"}),
        json!({"outcome": "positive"}), "interactions").await;
}

// ══════════════ HR (5) ══════════════

#[tokio::test]
async fn s63_hr_salary_records() {
    let (b, a, _) = start_real_server().await;
    let c = client();
    // Route is /salary (not /salary_records)
    let eid = c
        .post(format!("{}/api/employees", b))
        .header("Authorization", &a)
        .json(&json!({"name": "薪资员工"}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    crud_lifecycle(&b, &a, "/api/v1/salary",
        json!({"employee_id": eid, "employee_name": "薪资员工", "period": "2026-03", "base_salary": 15000}),
        json!({"bonus": 3000}), "salary_records").await;
}

#[tokio::test]
async fn s63_hr_time_entries() {
    let (b, a, _) = start_real_server().await;
    crud_lifecycle(&b, &a, "/api/v1/time_entries",
        json!({"description": "开发", "duration_minutes": 480, "date": "2026-03-21", "is_billable": true}),
        json!({"duration_minutes": 390}), "time_entries").await;
}

#[tokio::test]
async fn s63_hr_insurance() {
    let (b, a, _) = start_real_server().await;
    // insurance_type, start_date, end_date NOT NULL
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/insurance_policies",
        json!({"policy_name": "E2E保险", "insurance_type": "health", "provider": "平安",
               "premium_amount": 5000, "start_date": "2026-01-01", "end_date": "2026-12-31"}),
        json!({"premium_amount": 5500}),
        "insurance_policies",
    )
    .await;
}

#[tokio::test]
async fn s63_hr_pension() {
    let (b, a, _) = start_real_server().await;
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/pension_plans",
        json!({"plan_name": "年金", "plan_type": "defined_contribution",
               "contribution_employer": 800, "contribution_employee": 400}),
        json!({"contribution_employer": 1000}),
        "pension_plans",
    )
    .await;
}

#[tokio::test]
async fn s63_hr_social_insurance() {
    let (b, a, _) = start_real_server().await;
    // Route is /social_insurance_config (singular)
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/social_insurance_config",
        json!({"city": "深圳"}),
        json!({"city": "广州"}),
        "social_insurance_config",
    )
    .await;
}

// ══════════════ Cloud & IT (4) ══════════════

#[tokio::test]
async fn s63_cloud_providers() {
    let (b, a, _) = start_real_server().await;
    // account_name NOT NULL
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/cloud_providers",
        json!({"provider_name": "AWS", "provider_type": "cloud", "account_name": "main"}),
        json!({"provider_name": "AWS改"}),
        "cloud_providers",
    )
    .await;
}

#[tokio::test]
async fn s63_cloud_resources() {
    let (b, a, _) = start_real_server().await;
    // start_date NOT NULL
    crud_lifecycle(&b, &a, "/api/v1/cloud_resources",
        json!({"resource_name": "EC2", "resource_type": "compute", "monthly_cost": 200, "start_date": "2026-01-01"}),
        json!({"monthly_cost": 250}), "cloud_resources").await;
}

#[tokio::test]
async fn s63_domain_subscriptions() {
    let (b, a, _) = start_real_server().await;
    // registration_date, expiry_date NOT NULL
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/domain_subscriptions",
        json!({"domain_name": "test.com", "registrar": "CF", "registration_date": "2025-01-01",
               "expiry_date": "2026-01-01", "annual_fee": 100}),
        json!({"annual_fee": 120}),
        "domain_subscriptions",
    )
    .await;
}

#[tokio::test]
async fn s63_software_subscriptions() {
    let (b, a, _) = start_real_server().await;
    // subscription_type, billing_cycle, start_date, next_billing_date NOT NULL
    crud_lifecycle(&b, &a, "/api/v1/software_subscriptions",
        json!({"software_name": "GitHub", "vendor": "GitHub", "subscription_type": "team",
               "billing_cycle": "monthly", "start_date": "2026-01-01", "next_billing_date": "2026-04-01",
               "total_price": 40}),
        json!({"total_price": 44}), "software_subscriptions").await;
}

// ══════════════ KPI (2) ══════════════

#[tokio::test]
async fn s63_kpi_configs() {
    let (b, a, _) = start_real_server().await;
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/kpi_configs",
        json!({"name": "月营收", "target_value": 100000, "unit": "CNY"}),
        json!({"target_value": 150000}),
        "kpi_configs",
    )
    .await;
}

#[tokio::test]
async fn s63_kpi_records() {
    let (b, a, _) = start_real_server().await;
    let c = client();
    // kpi_id NOT NULL — create kpi first
    let kid = c
        .post(format!("{}/api/v1/kpi_configs", b))
        .header("Authorization", &a)
        .json(&json!({"name": "KPI-rec", "target_value": 100}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/kpi_records",
        json!({"kpi_id": kid, "value": 80, "period": "2026-03"}),
        json!({"value": 95}),
        "kpi_records",
    )
    .await;
}

// ══════════════ Tax & Finance (3) ══════════════

#[tokio::test]
async fn s63_tax_records() {
    let (b, a, _) = start_real_server().await;
    // Route is /tax_declarations (not /tax_records)
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/tax_declarations",
        json!({"tax_type": "VAT", "period": "2026-03", "tax_amount": 13000, "status": "pending"}),
        json!({"status": "filed"}),
        "tax_declarations",
    )
    .await;
}

#[tokio::test]
async fn s63_tax_jurisdictions() {
    let (b, a, _) = start_real_server().await;
    // code, country_code, jurisdiction_type NOT NULL
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/tax_jurisdictions",
        json!({"name": "E2E区域", "code": "CN-GD", "country_code": "CN",
               "jurisdiction_type": "province", "vat_rate": 13.0}),
        json!({"vat_rate": 9.0}),
        "tax_jurisdictions",
    )
    .await;
}

#[tokio::test]
async fn s63_recurring_invoices() {
    let (b, a, _) = start_real_server().await;
    // title, frequency, start_date, next_invoice_date NOT NULL
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/recurring_invoices",
        json!({"title": "月费", "frequency": "monthly", "amount": 5000,
               "start_date": "2026-01-01", "next_invoice_date": "2026-04-01"}),
        json!({"amount": 6000}),
        "recurring_invoices",
    )
    .await;
}

// ══════════════ Config (3) ══════════════

#[tokio::test]
async fn s63_company_info() {
    let (b, a, _) = start_real_server().await;
    // company_name NOT NULL
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/company_info",
        json!({"company_name": "E2E有限公司"}),
        json!({"company_name": "E2E科技"}),
        "company_info",
    )
    .await;
}

#[tokio::test]
async fn s63_branches() {
    let (b, a, _) = start_real_server().await;
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/branches",
        json!({"branch_name": "E2E分部", "branch_address": "深圳南山"}),
        json!({"branch_address": "深圳福田"}),
        "branches",
    )
    .await;
}

#[tokio::test]
async fn s63_vendors() {
    let (b, a, _) = start_real_server().await;
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/vendors",
        json!({"name": "供应商", "contact_email": "v@test-dev.example.invalid"}),
        json!({"name": "供应商改"}),
        "vendors",
    )
    .await;
}

// ══════════════ Materials (2) ══════════════

#[tokio::test]
async fn s63_applications() {
    let (b, a, _) = start_real_server().await;
    // application_no, title, type NOT NULL
    crud_lifecycle(&b, &a, "/api/v1/applications",
        json!({"application_no": "APP-001", "title": "高新申报", "type": "national", "status": "draft"}),
        json!({"status": "submitted"}), "applications").await;
}

#[tokio::test]
async fn s63_material_library() {
    let (b, a, _) = start_real_server().await;
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/material_library",
        json!({"name": "E2E材料", "category": "financial"}),
        json!({"name": "E2E材料改"}),
        "material_library",
    )
    .await;
}

// ══════════════ Data & R&D (2) ══════════════

#[tokio::test]
async fn s63_data_asset_meta() {
    let (b, a, _) = start_real_server().await;
    let c = client();
    // asset_id FK → assets(id), must create asset first
    let aid = c
        .post(format!("{}/api/assets", b))
        .header("Authorization", &a)
        .json(&json!({"name": "DA资产"}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/data_asset_meta",
        json!({"asset_id": aid, "data_type": "software", "version": "1.0"}),
        json!({"version": "2.0"}),
        "data_asset_meta",
    )
    .await;
}

#[tokio::test]
async fn s63_rd_costs() {
    let (b, a, _) = start_real_server().await;
    let c = client();
    // project_id NOT NULL — create project first
    let pid = c
        .post(format!("{}/api/v1/projects", b))
        .header("Authorization", &a)
        .json(&json!({"name": "RD项目", "status": "active"}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    crud_lifecycle(&b, &a, "/api/v1/rd_costs",
        json!({"project_id": pid, "cost_type": "personnel", "amount": 80000, "description": "研发投入"}),
        json!({"amount": 95000}), "rd_costs").await;
}

// ══════════════ Additional (5) ══════════════

#[tokio::test]
async fn s63_cloud_bills() {
    let (b, a, _) = start_real_server().await;
    // Handler now correctly requires ["billing_period"]
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/cloud_bills",
        json!({"billing_period": "2026-03", "total_amount": 5000}),
        json!({"total_amount": 5500}),
        "cloud_bills",
    )
    .await;
}

#[tokio::test]
async fn s63_milestones() {
    let (b, a, _) = start_real_server().await;
    // generic_update now safely handles tables without updated_at
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/milestones",
        json!({"title": "里程碑"}),
        json!({"description": "已完成"}),
        "milestones",
    )
    .await;
}

#[tokio::test]
async fn s63_projects() {
    let (b, a, _) = start_real_server().await;
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/projects",
        json!({"name": "项目", "status": "active"}),
        json!({"status": "completed"}),
        "projects",
    )
    .await;
}

#[tokio::test]
async fn s63_payments() {
    let (b, a, _) = start_real_server().await;
    // payment_type NOT NULL (not payment_method)
    crud_lifecycle(
        &b,
        &a,
        "/api/v1/payments",
        json!({"amount": 10000, "payment_type": "bank_transfer", "payment_date": "2026-03-21"}),
        json!({"status": "confirmed"}),
        "payments",
    )
    .await;
}

#[tokio::test]
async fn s63_error_logs() {
    let (b, a, _) = start_real_server().await;
    let c = client();
    let url = format!("{}/api/v1/error_logs", b);
    // error_logs: required fields are error_type + error_message (NOT "message")
    let resp = c.post(&url).header("Authorization", &a)
        .json(&json!({"error_type": "frontend", "error_message": "E2E test error", "stack_trace": "at test.ts:1"}))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200, "error_logs: create");
    let id = resp.json::<Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = c
        .get(format!("{}/{}", url, id))
        .header("Authorization", &a)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "error_logs: get");

    let resp = c
        .get(format!("{}?limit=10", url))
        .header("Authorization", &a)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "error_logs: list");

    let resp = c
        .delete(format!("{}/{}", url, id))
        .header("Authorization", &a)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "error_logs: delete");
}
