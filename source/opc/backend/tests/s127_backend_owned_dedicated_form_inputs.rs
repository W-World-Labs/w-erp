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

fn current_period() -> String {
    chrono::Utc::now().format("%Y-%m").to_string()
}

#[tokio::test]
async fn s127_dedicated_forms_normalize_numbers_dates_and_derived_values_on_backend() {
    let app = common::TestApp::new().await;

    let employee = app
        .post("/api/employees")
        .json(&json!({
            "name": "Backend Employee",
            "base_salary": "¥12,345.67",
            "social_security_base": "¥10,000.00",
            "housing_fund_base": "¥8,000.00",
            "hire_date": "2026-05-09T00:00:00.000Z",
            "contract_start_date": "2026-05-09T00:00:00.000Z",
            "contract_end_date": "2027-05-08T00:00:00.000Z"
        }))
        .await;
    employee.assert_status_ok();
    let employee = employee.json::<Value>();
    assert_f64(&employee["base_salary"], 12345.67);
    assert_f64(&employee["social_security_base"], 10000.0);
    assert_f64(&employee["housing_fund_base"], 8000.0);
    assert_eq!(employee["hire_date"], "2026-05-09");
    assert_eq!(employee["contract_start_date"], "2026-05-09");
    assert_eq!(employee["contract_end_date"], "2027-05-08");

    let asset = app
        .post("/api/assets")
        .json(&json!({
            "name": "Backend Asset",
            "asset_type": "physical",
            "purchase_price": "¥12,000.00",
            "current_value": "¥11,500.00",
            "useful_life": "12",
            "salvage_rate": "5",
            "purchase_date": "2026-05-09T00:00:00.000Z",
            "expiry_date": "2027-05-08T00:00:00.000Z"
        }))
        .await;
    asset.assert_status_ok();
    let asset = asset.json::<Value>()["data"].clone();
    assert_f64(&asset["purchase_price"], 12000.0);
    assert_f64(&asset["current_value"], 11500.0);
    assert_eq!(asset["useful_life"].as_i64().unwrap(), 12);
    assert_f64(&asset["salvage_rate"], 0.05);
    assert_f64(&asset["monthly_depreciation"], 950.0);
    assert_eq!(asset["purchase_date"], "2026-05-09");
    assert_eq!(asset["expiry_date"], "2027-05-08");

    let budget = app
        .post("/api/budgets")
        .json(&json!({
            "category": "office",
            "budget_type": "monthly",
            "period": "2026-05",
            "budget_amount": "¥88,888.88"
        }))
        .await;
    budget.assert_status_ok();
    let budget = budget.json::<Value>();
    let budget_id = budget["id"].as_str().unwrap().to_string();
    assert_f64(&budget["budget_amount"], 88888.88);

    let updated = app
        .put(&format!("/api/budgets/{budget_id}"))
        .json(&json!({
            "budget_amount": "¥99,999.99"
        }))
        .await;
    updated.assert_status_ok();
    let updated = updated.json::<Value>();
    assert_f64(&updated["budget_amount"], 99999.99);
}

#[tokio::test]
async fn s127_dedicated_forms_default_start_dates_on_backend_when_frontend_omits_them() {
    let app = common::TestApp::new().await;

    let contract = app
        .post("/api/contracts")
        .json(&json!({
            "contract_name": "Backend Default Contract",
            "amount": "¥12,000.00",
            "end_date": "2027-05-08T00:00:00.000Z"
        }))
        .await;
    contract.assert_status_ok();
    let contract_id = contract.json::<Value>()["id"].as_str().unwrap().to_string();

    let contract = app.get(&format!("/api/contracts/{contract_id}")).await;
    contract.assert_status_ok();
    let contract = contract.json::<Value>();
    assert_f64(&contract["amount"], 12000.0);
    assert_eq!(contract["start_date"], today());
    assert_eq!(contract["end_date"], "2027-05-08");

    let project = app
        .post("/api/projects")
        .json(&json!({
            "name": "Backend Default Project",
            "budget": "¥88,000.00",
            "end_date": "2027-05-08T00:00:00.000Z"
        }))
        .await;
    project.assert_status_ok();
    let project_id = project.json::<Value>()["id"].as_str().unwrap().to_string();

    let project = app.get(&format!("/api/projects/{project_id}")).await;
    project.assert_status_ok();
    let project = project.json::<Value>();
    assert_f64(&project["budget"], 88000.0);
    assert_eq!(project["start_date"], today());
    assert_eq!(project["end_date"], "2027-05-08");

    let rd_cost = app
        .post("/api/rd_costs")
        .json(&json!({
            "project_id": project_id,
            "cost_type": "material",
            "amount": "¥1,200.00"
        }))
        .await;
    rd_cost.assert_status_ok();
    let rd_cost_id = rd_cost.json::<Value>()["id"].as_str().unwrap().to_string();

    let rd_cost = app.get(&format!("/api/rd_costs/{rd_cost_id}")).await;
    rd_cost.assert_status_ok();
    let rd_cost = rd_cost.json::<Value>();
    assert_eq!(rd_cost["cost_date"], today());
    assert_f64(&rd_cost["amount"], 1200.0);

    let rent = app
        .post("/api/rent")
        .json(&json!({
            "location_name": "Backend Default Rent",
            "monthly_rent": "¥8,800.00",
            "contract_end_date": "2027-05-08T00:00:00.000Z",
            "next_payment_date": "2026-06-09T00:00:00.000Z"
        }))
        .await;
    rent.assert_status_ok();
    let rent_id = rent.json::<Value>()["id"].as_str().unwrap().to_string();

    let rent = app.get(&format!("/api/rent/{rent_id}")).await;
    rent.assert_status_ok();
    let rent = rent.json::<Value>();
    assert_f64(&rent["monthly_rent"], 8800.0);
    assert_eq!(rent["contract_start_date"], today());
    assert_eq!(rent["contract_end_date"], "2027-05-08");
    assert_eq!(rent["next_payment_date"], "2026-06-09");

    let utility = app
        .post("/api/v1/utilities")
        .json(&json!({
            "location_name": "Backend Default Utility",
            "utility_type": "electricity",
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
    assert_eq!(utility["period"], current_period());
    assert_f64(&utility["amount"], 150.625);
    assert_eq!(utility["due_date"], "2026-06-01");

    let employee = app
        .post("/api/employees")
        .json(&json!({
            "name": "Backend Default Salary Employee",
            "base_salary": "¥12,000.00"
        }))
        .await;
    employee.assert_status_ok();
    let employee_id = employee.json::<Value>()["id"].as_str().unwrap().to_string();

    let salary = app
        .post("/api/salary")
        .json(&json!({
            "employee_id": employee_id,
            "base_salary": "¥12,000.00"
        }))
        .await;
    salary.assert_status_ok();
    let salary_id = salary.json::<Value>()["id"].as_str().unwrap().to_string();

    let salary = app.get(&format!("/api/salary/{salary_id}")).await;
    salary.assert_status_ok();
    let salary = salary.json::<Value>();
    assert_eq!(salary["period"], current_period());
    assert_f64(&salary["base_salary"], 12000.0);

    let client = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "Backend Default Interaction Client" }))
        .await;
    client.assert_status_ok();
    let client_id = client.json::<Value>()["id"].as_str().unwrap().to_string();

    let interaction = app
        .post("/api/v1/interactions")
        .json(&json!({
            "client_id": client_id,
            "interaction_type": "call",
            "subject": "Backend default date"
        }))
        .await;
    interaction.assert_status_ok();
    let interaction_id = interaction.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let interaction = app
        .get(&format!("/api/v1/interactions/{interaction_id}"))
        .await;
    interaction.assert_status_ok();
    let interaction = interaction.json::<Value>();
    assert_eq!(interaction["interaction_date"], today());

    let cloud_bill = app
        .post("/api/v1/cloud_bills")
        .json(&json!({
            "total_amount": "¥5,000.00"
        }))
        .await;
    cloud_bill.assert_status_ok();
    let cloud_bill_id = cloud_bill.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let cloud_bill = app
        .get(&format!("/api/v1/cloud_bills/{cloud_bill_id}"))
        .await;
    cloud_bill.assert_status_ok();
    let cloud_bill = cloud_bill.json::<Value>();
    assert_eq!(cloud_bill["billing_period"], current_period());
    assert_f64(&cloud_bill["total_amount"], 5000.0);

    let cloud_resource = app
        .post("/api/v1/cloud_resources")
        .json(&json!({
            "resource_name": "Backend Default Resource",
            "resource_type": "compute",
            "monthly_cost": "¥300.00"
        }))
        .await;
    cloud_resource.assert_status_ok();
    let cloud_resource_id = cloud_resource.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let cloud_resource = app
        .get(&format!("/api/v1/cloud_resources/{cloud_resource_id}"))
        .await;
    cloud_resource.assert_status_ok();
    let cloud_resource = cloud_resource.json::<Value>();
    assert_eq!(cloud_resource["start_date"], today());
    assert_f64(&cloud_resource["monthly_cost"], 300.0);

    let software = app
        .post("/api/v1/software_subscriptions")
        .json(&json!({
            "software_name": "Backend Default Software",
            "vendor": "Backend Vendor",
            "subscription_type": "annual",
            "billing_cycle": "annually",
            "next_billing_date": "2027-05-08T00:00:00.000Z",
            "license_count": "3",
            "total_price": "¥9,000.00"
        }))
        .await;
    software.assert_status_ok();
    let software_id = software.json::<Value>()["id"].as_str().unwrap().to_string();

    let software = app
        .get(&format!("/api/v1/software_subscriptions/{software_id}"))
        .await;
    software.assert_status_ok();
    let software = software.json::<Value>();
    assert_eq!(software["start_date"], today());
    assert_eq!(software["next_billing_date"], "2027-05-08");
    assert_eq!(software["license_count"].as_i64().unwrap(), 3);
    assert_f64(&software["total_price"], 9000.0);
}
