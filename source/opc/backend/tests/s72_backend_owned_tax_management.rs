/// Scenario 72: backend-owned tax management summary.
/// Declaration counts, taxpayer info, and invoice tax totals are calculated server-side.
use crate::common;

use serde_json::{Value, json};

#[tokio::test]
async fn s72_tax_management_summary_calculates_counts_and_tax_totals() {
    let app = common::TestApp::new().await;
    seed_tax_summary_fixture(&app).await;

    let res = app.get("/api/v1/tax/management-summary").await;
    res.assert_status_ok();

    let body: Value = res.json();
    assert_eq!(body["taxpayer"]["taxpayer_type"], "general");
    assert_eq!(body["taxpayer"]["vat_rate"].as_f64().unwrap(), 13.0);
    assert_eq!(body["declaration_counts"]["pending"].as_i64().unwrap(), 1);
    assert_eq!(body["declaration_counts"]["declared"].as_i64().unwrap(), 1);
    assert_eq!(body["declaration_counts"]["overdue"].as_i64().unwrap(), 1);
    assert_eq!(
        body["invoice_tax"]["deductible_total"].as_f64().unwrap(),
        130.0
    );
    assert_eq!(
        body["invoice_tax"]["output_tax_due"].as_f64().unwrap(),
        260.0
    );
}

#[tokio::test]
async fn s72_tax_management_summary_is_company_scoped() {
    let app = common::TestApp::new().await;
    seed_tax_summary_fixture(&app).await;
    let rival = app.create_second_tenant().await;

    let res = rival.get("/api/v1/tax/management-summary").await;
    res.assert_status_ok();

    let body: Value = res.json();
    assert_eq!(body["declaration_counts"]["total"].as_i64().unwrap(), 0);
    assert_eq!(
        body["invoice_tax"]["deductible_total"].as_f64().unwrap(),
        0.0
    );
    assert_eq!(body["invoice_tax"]["output_tax_due"].as_f64().unwrap(), 0.0);
}

async fn seed_tax_summary_fixture(app: &common::TestApp) {
    app.post("/api/v1/company_info")
        .json(&json!({
            "company_name": "Tax Summary Co",
            "taxpayer_type": "general",
            "vat_rate": 13.0,
            "invoice_limit": 100000.0,
            "tax_registration_number": "TAX-123",
            "tax_authority": "Local Tax Bureau"
        }))
        .await
        .assert_status_ok();

    for status in ["pending", "declared", "overdue"] {
        app.post("/api/v1/tax_declarations")
            .json(&json!({
                "tax_type": "vat",
                "period": "2026-05",
                "declaration_type": "vat",
                "declaration_period": "monthly",
                "status": status
            }))
            .await
            .assert_status_ok();
    }

    app.post("/api/invoices")
        .json(&json!({
            "invoice_number": "TAX-IN-001",
            "invoice_type": "input",
            "amount": 1000.0,
            "tax_amount": 130.0,
            "status": "paid"
        }))
        .await
        .assert_status_ok();

    app.post("/api/invoices")
        .json(&json!({
            "invoice_number": "TAX-OUT-001",
            "invoice_type": "output",
            "amount": 2000.0,
            "tax_amount": 260.0,
            "status": "paid"
        }))
        .await
        .assert_status_ok();
}
