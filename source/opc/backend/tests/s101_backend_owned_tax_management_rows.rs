use crate::common;

use serde_json::{Value, json};

#[tokio::test]
async fn s101_tax_management_summary_returns_declaration_rows_labels_and_permissions() {
    let app = common::TestApp::new().await;

    app.post("/api/v1/company_info")
        .json(&json!({
            "company_name": "Tax Rows Co",
            "taxpayer_type": "general",
            "vat_rate": 13.0,
            "invoice_limit": 100000.0,
            "tax_registration_number": "TAX-ROWS",
            "tax_authority": "Local Tax Bureau"
        }))
        .await
        .assert_status_ok();

    app.post("/api/v1/tax_declarations")
        .json(&json!({
            "tax_type": "vat",
            "period": "2026-05",
            "declaration_type": "vat",
            "declaration_period": "monthly",
            "period_start": "2026-05-01",
            "period_end": "2026-05-31",
            "declaration_deadline": "2026-06-15",
            "tax_amount": 1234.5,
            "status": "pending"
        }))
        .await
        .assert_status_ok();

    app.post("/api/v1/tax_declarations")
        .json(&json!({
            "tax_type": "income_tax",
            "period": "2026-Q2",
            "declaration_type": "income_tax",
            "declaration_period": "quarterly",
            "declaration_date": "2026-07-10",
            "declared_amount": 5000.0,
            "tax_amount": 800.0,
            "status": "declared"
        }))
        .await
        .assert_status_ok();

    let res = app.get("/api/v1/tax/management-summary").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["taxpayer"]["taxpayer_type_label"], "一般纳税人");
    assert_eq!(body["taxpayer"]["vat_rate_label"], "13%");
    assert_eq!(body["taxpayer"]["invoice_limit_label"], "¥100000");
    assert_eq!(body["invoice_tax"]["deductible_total_label"], "¥0");
    assert_eq!(body["invoice_tax"]["output_tax_due_label"], "¥0");
    assert_eq!(body["permissions"]["can_delete"], true);
    assert_eq!(body["declaration_create_action"]["method"], "POST");
    assert_eq!(
        body["declaration_create_action"]["endpoint"],
        "/api/v1/tax_declarations"
    );
    assert_eq!(body["declaration_create_action"]["label"], "添加税务申报");
    assert_eq!(
        body["declaration_create_action"]["success_message"],
        "税务申报添加成功"
    );

    let form_options = app.get("/api/v1/tax_declarations/form_options").await;
    form_options.assert_status_ok();
    let form_options = form_options.json::<Value>();
    assert_eq!(form_options["default_values"]["declaration_type"], "vat");
    assert_eq!(
        form_options["default_values"]["declaration_period"],
        "monthly"
    );
    assert_eq!(form_options["declaration_type_options"][0]["value"], "");
    assert_eq!(
        form_options["declaration_type_options"][0]["label"],
        "请选择"
    );
    assert_eq!(form_options["declaration_period_options"][0]["value"], "");
    assert_eq!(
        form_options["declaration_period_options"][0]["label"],
        "请选择"
    );
    assert!(
        form_options["declaration_type_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "income_tax" && option["label"] == "企业所得税")
    );
    assert!(
        form_options["declaration_period_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "quarterly" && option["label"] == "按季度")
    );

    let inferred = app
        .post("/api/v1/tax_declarations")
        .json(&json!({
            "declaration_type": "vat",
            "declaration_period": "monthly",
            "period_start": "2026-08-01",
            "period_end": "2026-08-31",
            "declaration_deadline": "2026-09-15",
            "declaration_date": "2026-09-01"
        }))
        .await;
    inferred.assert_status_ok();
    let inferred_id = inferred.json::<Value>()["id"].as_str().unwrap().to_string();
    let inferred = app
        .get(&format!("/api/v1/tax_declarations/{inferred_id}"))
        .await;
    inferred.assert_status_ok();
    let inferred = inferred.json::<Value>();
    assert_eq!(inferred["tax_type"], "vat");
    assert_eq!(inferred["period"], "2026-08");
    assert_eq!(inferred["due_date"], "2026-09-15");
    assert_eq!(inferred["status"], "declared");

    let declarations = body["declarations"].as_array().unwrap();
    assert_eq!(declarations.len(), 2);
    let pending = body["pending_declarations"].as_array().unwrap();
    assert_eq!(pending.len(), 1);
    let row = &pending[0];
    assert_eq!(row["declaration_type_label"], "增值税");
    assert_eq!(row["declaration_period_label"], "按月");
    assert_eq!(row["period_label"], "2026-05-01 ~ 2026-05-31");
    assert_eq!(row["deadline_label"], "2026-06-15");
    assert_eq!(row["tax_amount_label"], "¥1234.5");
    assert_eq!(row["status_badge"]["label"], "待申报");
    assert_eq!(row["status_badge"]["variant"], "secondary");
    assert_eq!(row["can_delete"], true);
    assert_eq!(row["update_action"]["method"], "PUT");
    assert_eq!(
        row["update_action"]["endpoint"],
        format!("/api/v1/tax_declarations/{}", row["id"].as_str().unwrap())
    );
    assert_eq!(row["update_action"]["label"], "保存税务申报");
    assert_eq!(row["update_action"]["success_message"], "税务申报更新成功");
    assert_eq!(
        row["delete_action"]["endpoint"],
        format!("/api/v1/tax_declarations/{}", row["id"].as_str().unwrap())
    );
    assert_eq!(
        row["delete_action"]["confirm_message"],
        "确定要删除税务申报「增值税 2026-05-01 ~ 2026-05-31」吗？此操作无法撤销。"
    );
    assert_eq!(row["delete_action"]["success_message"], "税务申报已删除");

    let declared = body["declared_declarations"].as_array().unwrap();
    assert_eq!(declared.len(), 1);
    assert_eq!(declared[0]["declaration_date_label"], "2026-07-10");
    assert_eq!(declared[0]["declared_amount_label"], "¥5000");
}
