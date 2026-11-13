use crate::common;

use serde_json::{Value, json};

fn assert_f64(actual: &Value, expected: f64) {
    let actual = actual.as_f64().expect("number");
    assert!(
        (actual - expected).abs() < 0.001,
        "expected {expected}, got {actual}"
    );
}

fn today() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

#[tokio::test]
async fn s125_generic_forms_normalize_numeric_dates_and_derived_amounts_on_backend() {
    let app = common::TestApp::new().await;

    let payment = app
        .post("/api/v1/payments")
        .json(&json!({
            "payment_no": "PAY-BACKEND-001",
            "payment_type": "expense",
            "payment_method": "bank_transfer_b2b",
            "payer": "甲方",
            "payee": "乙方",
            "amount": "¥1,234.50",
            "payment_date": "2026-05-09T00:00:00.000Z",
            "invoice_deadline": "2026-05-31T00:00:00.000Z"
        }))
        .await;
    payment.assert_status_ok();
    let payment_id = payment.json::<Value>()["id"].as_str().unwrap().to_string();

    let payment = app.get(&format!("/api/v1/payments/{payment_id}")).await;
    payment.assert_status_ok();
    let payment = payment.json::<Value>();
    assert_f64(&payment["amount"], 1234.50);
    assert_eq!(payment["payment_date"], "2026-05-09");
    assert_eq!(payment["invoice_deadline"], "2026-05-31");

    let rent = app
        .post("/api/v1/rent")
        .json(&json!({
            "location_name": "Backend Rent HQ",
            "address": "Backend Road",
            "area": "88.5",
            "monthly_rent": "¥8,000.00",
            "contract_start_date": "2026-05-09T00:00:00.000Z",
            "contract_end_date": "2027-05-08T00:00:00.000Z",
            "next_payment_date": "2026-06-09T00:00:00.000Z"
        }))
        .await;
    rent.assert_status_ok();
    let rent_id = rent.json::<Value>()["id"].as_str().unwrap().to_string();

    let rent = app.get(&format!("/api/v1/rent/{rent_id}")).await;
    rent.assert_status_ok();
    let rent = rent.json::<Value>();
    assert_f64(&rent["area"], 88.5);
    assert_f64(&rent["monthly_rent"], 8000.0);
    assert_eq!(rent["contract_start_date"], "2026-05-09");
    assert_eq!(rent["contract_end_date"], "2027-05-08");
    assert_eq!(rent["next_payment_date"], "2026-06-09");

    let utility = app
        .post("/api/v1/utilities")
        .json(&json!({
            "location_name": "Backend Rent HQ",
            "utility_type": "electricity",
            "period": "2026-05",
            "usage": "120.5",
            "unit_price": "1.25",
            "due_date": "2026-06-01T00:00:00.000Z"
        }))
        .await;
    utility.assert_status_ok();
    let utility_id = utility.json::<Value>()["id"].as_str().unwrap().to_string();

    let utility = app.get(&format!("/api/v1/utilities/{utility_id}")).await;
    utility.assert_status_ok();
    let utility = utility.json::<Value>();
    assert_f64(&utility["usage"], 120.5);
    assert_f64(&utility["unit_price"], 1.25);
    assert_f64(&utility["amount"], 150.625);
    assert_eq!(utility["due_date"], "2026-06-01");

    let project = app
        .post("/api/v1/projects")
        .json(&json!({
            "name": "Backend Numeric Project",
            "project_type": "rd",
            "leader_name": "Owner",
            "budget": "¥50,000.00",
            "team_members": "8",
            "start_date": "2026-05-09T00:00:00.000Z",
            "end_date": "2026-12-31T00:00:00.000Z"
        }))
        .await;
    project.assert_status_ok();
    let project_id = project.json::<Value>()["id"].as_str().unwrap().to_string();

    let project = app.get(&format!("/api/v1/projects/{project_id}")).await;
    project.assert_status_ok();
    let project = project.json::<Value>();
    assert_f64(&project["budget"], 50000.0);
    assert_eq!(project["team_members"].as_i64().unwrap(), 8);
    assert_eq!(project["start_date"], "2026-05-09");
    assert_eq!(project["end_date"], "2026-12-31");

    sqlx::query("INSERT INTO clients (id, company_id, name) VALUES ('client-form-inputs', ?, 'Backend Client')")
        .bind(&app.company_id)
        .execute(&app.pool)
        .await
        .expect("seed client");

    let deal = app
        .post("/api/v1/deals")
        .json(&json!({
            "client_id": "client-form-inputs",
            "title": "Backend Deal",
            "amount": "¥9,999.99",
            "probability": "75",
            "expected_close_date": "2026-07-01T00:00:00.000Z"
        }))
        .await;
    deal.assert_status_ok();
    let deal_id = deal.json::<Value>()["id"].as_str().unwrap().to_string();

    let deal = app.get(&format!("/api/v1/deals/{deal_id}")).await;
    deal.assert_status_ok();
    let deal = deal.json::<Value>();
    assert_f64(&deal["amount"], 9999.99);
    assert_eq!(deal["probability"].as_i64().unwrap(), 75);
    assert_eq!(deal["expected_close_date"], "2026-07-01");

    let quote = app
        .post("/api/v1/quotes")
        .json(&json!({
            "client_id": "client-form-inputs",
            "quote_number": "QT-BACKEND-FORM-001",
            "subtotal": "¥10,000.00",
            "tax_rate": "13",
            "tax_amount": "¥1,300.00",
            "discount": "¥100.00",
            "total": "¥11,200.00",
            "valid_until": "2026-08-01T00:00:00.000Z"
        }))
        .await;
    quote.assert_status_ok();
    let quote_id = quote.json::<Value>()["id"].as_str().unwrap().to_string();

    let quote = app.get(&format!("/api/v1/quotes/{quote_id}")).await;
    quote.assert_status_ok();
    let quote = quote.json::<Value>();
    assert_f64(&quote["subtotal"], 10000.0);
    assert_f64(&quote["tax_rate"], 13.0);
    assert_f64(&quote["tax_amount"], 1300.0);
    assert_f64(&quote["discount"], 100.0);
    assert_f64(&quote["total"], 11200.0);
    assert_eq!(quote["valid_until"], "2026-08-01");

    let recurring = app
        .post("/api/v1/recurring_invoices")
        .json(&json!({
            "title": "Backend Recurring",
            "amount": "¥2,000.00",
            "tax_rate": "6.5",
            "frequency": "monthly",
            "start_date": "2026-05-09T00:00:00.000Z",
            "day_of_month": "9",
            "payment_terms_days": "45"
        }))
        .await;
    recurring.assert_status_ok();
    let recurring_id = recurring.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let recurring = app
        .get(&format!("/api/v1/recurring_invoices/{recurring_id}"))
        .await;
    recurring.assert_status_ok();
    let recurring = recurring.json::<Value>();
    assert_f64(&recurring["amount"], 2000.0);
    assert_f64(&recurring["tax_rate"], 6.5);
    assert_eq!(recurring["day_of_month"].as_i64().unwrap(), 9);
    assert_eq!(recurring["payment_terms_days"].as_i64().unwrap(), 45);
    assert_eq!(recurring["start_date"], "2026-05-09");
    assert_eq!(recurring["next_invoice_date"], "2026-05-09");

    let kpi = app
        .post("/api/v1/kpi_configs")
        .json(&json!({
            "name": "Backend KPI",
            "target_value": "¥100,000.00",
            "warning_threshold": "80,000",
            "danger_threshold": "50,000"
        }))
        .await;
    kpi.assert_status_ok();
    let kpi_id = kpi.json::<Value>()["id"].as_str().unwrap().to_string();

    let kpi = app.get(&format!("/api/v1/kpi_configs/{kpi_id}")).await;
    kpi.assert_status_ok();
    let kpi = kpi.json::<Value>();
    assert_f64(&kpi["target_value"], 100000.0);
    assert_f64(&kpi["warning_threshold"], 80000.0);
    assert_f64(&kpi["danger_threshold"], 50000.0);

    let company = app
        .post("/api/v1/company_info")
        .json(&json!({
            "company_name": "Backend Company",
            "registered_capital": "¥1,000,000.00",
            "vat_rate": "13",
            "invoice_limit": "¥50,000.00",
            "cash_flow_threshold": "¥88,888.00",
            "registration_date": "2026-05-09T00:00:00.000Z"
        }))
        .await;
    company.assert_status_ok();
    let company_id = company.json::<Value>()["id"].as_str().unwrap().to_string();

    let company = app.get(&format!("/api/v1/company_info/{company_id}")).await;
    company.assert_status_ok();
    let company = company.json::<Value>();
    assert_f64(&company["registered_capital"], 1_000_000.0);
    assert_f64(&company["vat_rate"], 13.0);
    assert_f64(&company["invoice_limit"], 50_000.0);
    assert_f64(&company["cash_flow_threshold"], 88_888.0);
    assert_eq!(company["registration_date"], "2026-05-09");

    let branch = app
        .post("/api/v1/branches")
        .json(&json!({
            "branch_name": "Backend Branch",
            "registered_capital": "¥123,456.78",
            "employee_count": "12",
            "registration_date": "2026-05-09T00:00:00.000Z"
        }))
        .await;
    branch.assert_status_ok();
    let branch_id = branch.json::<Value>()["id"].as_str().unwrap().to_string();

    let branch = app.get(&format!("/api/v1/branches/{branch_id}")).await;
    branch.assert_status_ok();
    let branch = branch.json::<Value>();
    assert_eq!(branch["registered_capital"], "123456.78");
    assert_eq!(branch["employee_count"].as_i64().unwrap(), 12);
    assert_eq!(branch["registration_date"], "2026-05-09");

    let rd_cost = app
        .post("/api/v1/rd_costs")
        .json(&json!({
            "project_id": project_id,
            "cost_type": "personnel",
            "amount": "¥6,789.01",
            "cost_date": "2026-05-09T00:00:00.000Z"
        }))
        .await;
    rd_cost.assert_status_ok();
    let rd_cost_id = rd_cost.json::<Value>()["id"].as_str().unwrap().to_string();

    let rd_cost = app.get(&format!("/api/v1/rd_costs/{rd_cost_id}")).await;
    rd_cost.assert_status_ok();
    let rd_cost = rd_cost.json::<Value>();
    assert_f64(&rd_cost["amount"], 6789.01);
    assert_eq!(rd_cost["cost_date"], "2026-05-09");

    let tax = app
        .post("/api/v1/tax_declarations")
        .json(&json!({
            "tax_type": "vat",
            "period": "2026-05",
            "declared_amount": "¥20,000.00",
            "tax_amount": "¥2,600.00",
            "declaration_date": "2026-06-01T00:00:00.000Z"
        }))
        .await;
    tax.assert_status_ok();
    let tax_id = tax.json::<Value>()["id"].as_str().unwrap().to_string();

    let tax = app.get(&format!("/api/v1/tax_declarations/{tax_id}")).await;
    tax.assert_status_ok();
    let tax = tax.json::<Value>();
    assert_f64(&tax["declared_amount"], 20_000.0);
    assert_f64(&tax["tax_amount"], 2600.0);
    assert_eq!(tax["declaration_date"], "2026-06-01");

    let social_config = app
        .post("/api/v1/social_insurance_config")
        .json(&json!({
            "city": "Backend City",
            "social_security_base_min": "¥5,000.00",
            "social_security_base_max": "¥30,000.00",
            "housing_fund_base_min": "¥5,000.00",
            "housing_fund_base_max": "¥30,000.00",
            "pension_personal_rate": "8",
            "housing_fund_company_rate": "12"
        }))
        .await;
    social_config.assert_status_ok();
    let social_config_id = social_config.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let social_config = app
        .get(&format!(
            "/api/v1/social_insurance_config/{social_config_id}"
        ))
        .await;
    social_config.assert_status_ok();
    let social_config = social_config.json::<Value>();
    assert_f64(&social_config["social_security_base_min"], 5000.0);
    assert_f64(&social_config["social_security_base_max"], 30000.0);
    assert_f64(&social_config["housing_fund_base_min"], 5000.0);
    assert_f64(&social_config["housing_fund_base_max"], 30000.0);
    assert_f64(&social_config["pension_personal_rate"], 8.0);
    assert_f64(&social_config["housing_fund_company_rate"], 12.0);

    let insurance = app
        .post("/api/v1/insurance_policies")
        .json(&json!({
            "policy_name": "Backend Insurance",
            "insurance_type": "health",
            "premium_amount": "¥1,200.00",
            "coverage_amount": "¥1,000,000.00",
            "deductible": "¥500.00",
            "start_date": "2026-05-09T00:00:00.000Z",
            "end_date": "2027-05-08T00:00:00.000Z",
            "renewal_date": "2027-04-08T00:00:00.000Z"
        }))
        .await;
    insurance.assert_status_ok();
    let insurance_id = insurance.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let insurance = app
        .get(&format!("/api/v1/insurance_policies/{insurance_id}"))
        .await;
    insurance.assert_status_ok();
    let insurance = insurance.json::<Value>();
    assert_f64(&insurance["premium_amount"], 1200.0);
    assert_f64(&insurance["coverage_amount"], 1_000_000.0);
    assert_f64(&insurance["deductible"], 500.0);
    assert_eq!(insurance["start_date"], "2026-05-09");
    assert_eq!(insurance["end_date"], "2027-05-08");
    assert_eq!(insurance["renewal_date"], "2027-04-08");

    let pension = app
        .post("/api/v1/pension_plans")
        .json(&json!({
            "plan_name": "Backend Pension",
            "plan_type": "401k",
            "contribution_employer": "¥800.00",
            "contribution_employee": "¥600.00",
            "current_value": "¥12,345.67",
            "vesting_period_months": "36"
        }))
        .await;
    pension.assert_status_ok();
    let pension_id = pension.json::<Value>()["id"].as_str().unwrap().to_string();

    let pension = app
        .get(&format!("/api/v1/pension_plans/{pension_id}"))
        .await;
    pension.assert_status_ok();
    let pension = pension.json::<Value>();
    assert_f64(&pension["contribution_employer"], 800.0);
    assert_f64(&pension["contribution_employee"], 600.0);
    assert_f64(&pension["current_value"], 12345.67);
    assert_eq!(pension["vesting_period_months"].as_i64().unwrap(), 36);
}

#[tokio::test]
async fn s125_payment_create_defaults_payment_date_on_backend_when_frontend_omits_it() {
    let app = common::TestApp::new().await;

    let payment = app
        .post("/api/v1/payments")
        .json(&json!({
            "payment_no": "PAY-BACKEND-DATE-DEFAULT",
            "payment_type": "expense",
            "payment_method": "bank_transfer_b2b",
            "payer": "Backend Payer",
            "payee": "Backend Payee",
            "amount": "¥888.88"
        }))
        .await;
    payment.assert_status_ok();
    let payment_id = payment.json::<Value>()["id"].as_str().unwrap().to_string();

    let payment = app.get(&format!("/api/v1/payments/{payment_id}")).await;
    payment.assert_status_ok();
    let payment = payment.json::<Value>();

    assert_f64(&payment["amount"], 888.88);
    assert_eq!(payment["payment_date"], today());
}
