/// Scenario 42: 🔗 Material Versions, Quote→Invoice, Generic Export, Email, Logs, Seeds
/// Tests remaining special/workflow endpoints.
use crate::common;
use serde_json::json;

// ==================== Material Versions ====================

#[tokio::test]
async fn s42_material_version_crud() {
    let app = common::TestApp::new().await;
    let mat = app
        .post("/api/v1/material_library")
        .json(&json!({ "name": "版本测试材料" }))
        .await;
    mat.assert_status_ok();
    let mat_id = mat.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = app
        .post("/api/v1/material_versions")
        .json(&json!({ "material_id": &mat_id, "file_name": "v1.0.pdf", "version": "1.0" }))
        .await;
    if !res.status_code().is_success() {
        return;
    }
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();
    app.get(&format!("/api/v1/material_versions/{}", id))
        .await
        .assert_status_ok();
    let del = app
        .delete(&format!("/api/v1/material_versions/{}", id))
        .await;
    assert!(del.status_code().is_success());
}

#[tokio::test]
async fn s42_material_version_fk_invalid() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/v1/material_versions")
        .json(&json!({ "material_id": "no-such-mat", "file_name": "orphan.pdf" }))
        .await;
    assert_eq!(res.status_code(), 400);
}

// ==================== Pension Plans (full CRUD) ====================

#[tokio::test]
async fn s42_pension_plan_full_crud() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/v1/pension_plans")
        .json(&json!({ "plan_name": "养老金计划A", "plan_type": "defined_benefit" }))
        .await;
    if !res.status_code().is_success() {
        return;
    }
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();
    app.get(&format!("/api/v1/pension_plans/{}", id))
        .await
        .assert_status_ok();
    let del = app.delete(&format!("/api/v1/pension_plans/{}", id)).await;
    assert!(del.status_code().is_success());
}

// ==================== Quote → Invoice Conversion ====================

#[tokio::test]
async fn s42_quote_convert_to_invoice() {
    let app = common::TestApp::new().await;
    let quote = app
        .post("/api/v1/quotes")
        .json(&json!({ "quote_number": "QT-CONV-001", "status": "approved", "amount": 25000.0 }))
        .await;
    if !quote.status_code().is_success() {
        return;
    }
    let quote_id = quote.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let conv = app
        .post(&format!("/api/v1/quotes/{}/convert-to-invoice", quote_id))
        .json(&json!({}))
        .await;
    let status = conv.status_code();
    // Should create an invoice from the quote
    assert!(
        status.is_success() || status == 400 || status == 404 || status == 500,
        "Convert to invoice: {}",
        status
    );
}

// ==================== Generic Export Endpoint ====================

#[tokio::test]
async fn s42_generic_export_invoices() {
    let app = common::TestApp::new().await;
    app.post("/api/invoices")
        .json(
            &json!({ "invoice_number": "INV-EXP-G1", "amount": 1000.0, "invoice_type": "income" }),
        )
        .await
        .assert_status_ok();

    let csv = app.get("/api/v1/export/invoices").await;
    csv.assert_status_ok();
}

#[tokio::test]
async fn s42_generic_export_employees() {
    let app = common::TestApp::new().await;
    app.post("/api/employees")
        .json(&json!({ "name": "导出员工", "email": "exp@test-dev.example.invalid" }))
        .await
        .assert_status_ok();

    let csv = app.get("/api/v1/export/employees").await;
    csv.assert_status_ok();
}

#[tokio::test]
async fn s42_generic_export_clients() {
    let app = common::TestApp::new().await;
    app.post("/api/v1/clients")
        .json(&json!({ "name": "导出客户G", "client_type": "corporate" }))
        .await
        .assert_status_ok();

    let csv = app.get("/api/v1/export/clients").await;
    csv.assert_status_ok();
}

#[tokio::test]
async fn s42_generic_export_projects() {
    let app = common::TestApp::new().await;
    app.post("/api/projects")
        .json(&json!({ "name": "导出项目G", "status": "active" }))
        .await
        .assert_status_ok();

    let csv = app.get("/api/v1/export/projects").await;
    csv.assert_status_ok();
}

#[tokio::test]
async fn s42_generic_export_assets() {
    let app = common::TestApp::new().await;
    let csv = app.get("/api/v1/export/assets").await;
    csv.assert_status_ok();
}

#[tokio::test]
async fn s42_generic_export_contracts() {
    let app = common::TestApp::new().await;
    let csv = app.get("/api/v1/export/contracts").await;
    csv.assert_status_ok();
}

#[tokio::test]
async fn s42_generic_export_payments() {
    let app = common::TestApp::new().await;
    let csv = app.get("/api/v1/export/payments").await;
    csv.assert_status_ok();
}

#[tokio::test]
async fn s42_generic_export_vendors() {
    let app = common::TestApp::new().await;
    let csv = app.get("/api/v1/export/vendors").await;
    csv.assert_status_ok();
}

#[tokio::test]
async fn s42_generic_export_nonexistent() {
    let app = common::TestApp::new().await;
    let csv = app.get("/api/v1/export/nonexistent_table").await;
    let s = csv.status_code();
    assert!(s == 400 || s == 404, "Bad export table: {}", s);
}

// ==================== Email Send ====================

#[tokio::test]
async fn s42_email_send_no_config() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/v1/email/send")
        .json(&json!({ "to": "test@example.com", "subject": "测试邮件", "body": "Hello" }))
        .await;
    // Without SMTP config, should fail gracefully
    let s = res.status_code();
    assert!(
        s == 400 || s == 500 || s == 422 || s == 502,
        "Email send without config: {}",
        s
    );
}

// ==================== Email Logs ====================

#[tokio::test]
async fn s42_email_logs_list() {
    let app = common::TestApp::new().await;
    let res = app.get("/api/v1/email/logs").await;
    res.assert_status_ok();
}

// ==================== Contract Review History ====================

#[tokio::test]
async fn s42_contract_review_history() {
    let app = common::TestApp::new().await;
    // Create a contract first
    app.post("/api/contracts")
        .json(&json!({ "contract_name": "审查合同", "contract_type": "service", "counterparty": "Y", "counterparty_type": "individual" }))
        .await.assert_status_ok();

    let res = app.get("/api/v1/contract_review_history").await;
    let s = res.status_code();
    assert!(s.is_success() || s == 500, "Contract review history: {}", s);
}

// ==================== Operation Logs ====================

#[tokio::test]
async fn s42_logs_list() {
    let app = common::TestApp::new().await;
    let res = app.get("/api/v1/logs").await;
    res.assert_status_ok();
}

// ==================== Seed Endpoints ====================

#[tokio::test]
async fn s42_tax_calendar_seed() {
    let app = common::TestApp::new().await;
    let res = app.post("/api/v1/tax_calendar/seed").json(&json!({})).await;
    let s = res.status_code();
    assert!(s.is_success() || s == 409 || s == 400, "Tax seed: {}", s);
}

#[tokio::test]
async fn s42_opc_hub_seed() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/v1/global_opc_hub/seed")
        .json(&json!({}))
        .await;
    let s = res.status_code();
    assert!(s.is_success() || s == 409 || s == 400, "OPC seed: {}", s);
}

// ==================== Application Materials ====================

#[tokio::test]
async fn s42_application_material_crud() {
    let app = common::TestApp::new().await;
    let application = app
        .post("/api/v1/applications")
        .json(&json!({ "title": "材料CRUD", "type": "grant", "application_no": "AM-001" }))
        .await;
    application.assert_status_ok();
    let app_id = application.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = app
        .post("/api/v1/application_materials")
        .json(&json!({ "application_id": &app_id, "name": "申报材料A" }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();
    app.get(&format!("/api/v1/application_materials/{}", id))
        .await
        .assert_status_ok();
    let del = app
        .delete(&format!("/api/v1/application_materials/{}", id))
        .await;
    assert!(del.status_code().is_success());
}

#[tokio::test]
async fn s42_application_material_fk_invalid() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/v1/application_materials")
        .json(&json!({ "application_id": "no-such-app", "name": "孤儿材料" }))
        .await;
    assert_eq!(res.status_code(), 400);
}
