/// Scenario 48: 🏯 Asian Market Compliance Engine
/// Tests JP consumption tax, CN VAT, KR e-invoice, compliance check, and input credits.
use crate::common;
use serde_json::json;

// ==================== JP Consumption Tax — Standard 10% ====================

#[tokio::test]
async fn s48_jp_tax_standard() {
    let app = common::TestApp::new().await;
    let res = app
        .get("/api/v1/tax/calculate?amount=10000&country=JP")
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["country"], "JP");
    assert_eq!(body["tax_rate"], 0.10);
    assert_eq!(body["tax_amount"], 1000.0);
    assert_eq!(body["total_with_tax"], 11000.0);
    assert_eq!(body["qis_required"], true);
}

// ==================== JP Consumption Tax — Reduced 8% (food) ====================

#[tokio::test]
async fn s48_jp_tax_reduced_food() {
    let app = common::TestApp::new().await;
    let res = app
        .get("/api/v1/tax/calculate?amount=5000&country=JP&category=food")
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["tax_rate"], 0.08);
    assert_eq!(body["tax_amount"], 400.0);
    assert_eq!(body["total_with_tax"], 5400.0);
    assert!(body["rate_name"].as_str().unwrap().contains("軽減"));
}

// ==================== KR VAT — 10% ====================

#[tokio::test]
async fn s48_kr_vat_standard() {
    let app = common::TestApp::new().await;
    let res = app
        .get("/api/v1/tax/calculate?amount=1000000&country=KR")
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["country"], "KR");
    assert_eq!(body["tax_rate"], 0.10);
    assert_eq!(body["tax_amount"], 100000.0);
    assert_eq!(body["e_tax_invoice_required"], true);
    assert_eq!(body["submission_deadline_hours"], 24);
}

// ==================== CN VAT — General Taxpayer Service 6% ====================

#[tokio::test]
async fn s48_cn_vat_general_service() {
    let app = common::TestApp::new().await;
    let res = app
        .get("/api/v1/tax/calculate?amount=100000&country=CN&category=service")
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["country"], "CN");
    assert_eq!(body["tax_rate"], 0.06);
    assert_eq!(body["tax_amount"], 6000.0);
    assert_eq!(body["invoice_type"], "增值税专用发票");
    assert_eq!(body["input_credit_eligible"], true);
}

// ==================== CN VAT — General Taxpayer Goods 13% ====================

#[tokio::test]
async fn s48_cn_vat_general_goods() {
    let app = common::TestApp::new().await;
    let res = app
        .get("/api/v1/tax/calculate?amount=50000&country=CN&category=goods")
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["tax_rate"], 0.13);
    assert_eq!(body["tax_amount"], 6500.0);
}

// ==================== CN VAT — Small Taxpayer 3% ====================

#[tokio::test]
async fn s48_cn_vat_small_taxpayer() {
    let app = common::TestApp::new().await;
    let res = app
        .get("/api/v1/tax/calculate?amount=100000&country=CN&taxpayer_type=small")
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["tax_rate"], 0.03);
    assert_eq!(body["tax_amount"], 3000.0);
    assert_eq!(body["invoice_type"], "增值税普通发票");
    assert_eq!(body["input_credit_eligible"], false);
}

// ==================== Tax Calc — Company Country Auto-detect ====================

#[tokio::test]
async fn s48_tax_auto_detect_country() {
    let app = common::TestApp::new().await;
    // Default company country is CN
    let res = app.get("/api/v1/tax/calculate?amount=10000").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    // Should auto-detect CN
    assert_eq!(body["country"], "CN");
    assert!(body["tax_amount"].as_f64().unwrap() > 0.0);
}

// ==================== Invoice Requirements — JP QIS ====================

#[tokio::test]
async fn s48_invoice_requirements_jp() {
    let app = common::TestApp::new().await;
    let res = app.get("/api/v1/tax/invoice-requirements?country=JP").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["country"], "JP");
    assert!(body["system"].as_str().unwrap().contains("QIS"));
    let fields = body["required_fields"].as_array().unwrap();
    assert!(fields.len() >= 5);
    // Must include registration_number
    assert!(fields.iter().any(|f| f["field"] == "registration_number"));
    assert_eq!(body["bilingual_support"], true);
}

// ==================== Invoice Requirements — KR NTS ====================

#[tokio::test]
async fn s48_invoice_requirements_kr() {
    let app = common::TestApp::new().await;
    let res = app.get("/api/v1/tax/invoice-requirements?country=KR").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["country"], "KR");
    assert!(body["submission"]["deadline_hours"].as_i64().unwrap() == 24);
    assert_eq!(body["self_billing"]["supported"], true);
}

// ==================== Invoice Requirements — CN VAT ====================

#[tokio::test]
async fn s48_invoice_requirements_cn() {
    let app = common::TestApp::new().await;
    let res = app.get("/api/v1/tax/invoice-requirements?country=CN").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["country"], "CN");
    let types = body["invoice_types"].as_array().unwrap();
    assert!(types.len() >= 2);
    // Must include 专票 and 普票
    assert!(
        types
            .iter()
            .any(|t| t["type"].as_str().unwrap().contains("专用"))
    );
    assert!(
        types
            .iter()
            .any(|t| t["type"].as_str().unwrap().contains("普通"))
    );
    assert!(
        body["taxpayer_types"]["small"]["label"]
            .as_str()
            .unwrap()
            .contains("小规模")
    );
}

// ==================== Compliance Check — Clean State ====================

#[tokio::test]
async fn s48_compliance_check_clean() {
    let app = common::TestApp::new().await;
    let res = app.get("/api/v1/tax/compliance-check").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert!(body["compliance_score"]["total"].as_i64().unwrap() >= 90);
    assert_eq!(body["overdue"]["count"], 0);
    assert!(body["company_country"].is_string());
    assert!(body["alerts"].is_array());
}

// ==================== Compliance Check — With Overdue ====================

#[tokio::test]
async fn s48_compliance_check_with_overdue() {
    let app = common::TestApp::new().await;

    // Create overdue tax record
    app.post("/api/v1/tax")
        .json(&json!({
            "tax_type": "vat",
            "period": "2025-12",
            "due_date": "2026-01-15",
            "taxable_amount": 100000.0,
            "tax_amount": 6000.0,
            "status": "pending"
        }))
        .await
        .assert_status_ok();

    let res = app.get("/api/v1/tax/compliance-check").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert!(body["overdue"]["count"].as_i64().unwrap() >= 1);
    // Score should drop
    assert!(body["compliance_score"]["total"].as_i64().unwrap() < 100);
}

// ==================== Input Credit Summary — Empty ====================

#[tokio::test]
async fn s48_input_credit_empty() {
    let app = common::TestApp::new().await;
    let res = app.get("/api/v1/tax/input-credit").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["output_vat"]["total_tax"], 0.0);
    assert_eq!(body["input_vat"]["total_tax"], 0.0);
    assert_eq!(body["net_vat"]["status"], "zero");
    assert!(body["notes"].is_string());
}

// ==================== Input Credit — With Data ====================

#[tokio::test]
async fn s48_input_credit_with_data() {
    let app = common::TestApp::new().await;

    // Create receivable invoice with tax (output VAT / 销项)
    app.post("/api/invoices")
        .json(&json!({
            "invoice_number": "OUT-001",
            "amount": 100000.0,
            "tax_amount": 6000.0,
            "invoice_type": "receivable",
            "supplier": "Customer A"
        }))
        .await
        .assert_status_ok();

    // Create payable invoice with tax (input VAT / 进项)
    app.post("/api/invoices")
        .json(&json!({
            "invoice_number": "IN-001",
            "amount": 50000.0,
            "tax_amount": 3000.0,
            "invoice_type": "payable",
            "supplier": "Supplier B"
        }))
        .await
        .assert_status_ok();

    let res = app.get("/api/v1/tax/input-credit").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();

    assert_eq!(body["output_vat"]["total_tax"], 6000.0);
    assert_eq!(body["input_vat"]["total_tax"], 3000.0);
    assert_eq!(body["net_vat"]["amount"], 3000.0);
    assert_eq!(body["net_vat"]["status"], "payable"); // 6000 - 3000 = 3000 payable
}
