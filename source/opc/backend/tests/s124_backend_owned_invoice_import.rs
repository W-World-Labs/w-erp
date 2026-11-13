use crate::common;

use base64::{Engine as _, engine::general_purpose};
use serde_json::{Value, json};

async fn seed_existing_invoice(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO invoices (id, company_id, invoice_number, invoice_type, amount, status)
        VALUES ('existing-import-invoice', ?, 'DUP-001', 'input', 88, 'pending')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed existing invoice");
}

#[tokio::test]
async fn s124_invoice_import_preview_returns_backend_owned_rows_stats_and_badges() {
    let app = common::TestApp::new().await;
    seed_existing_invoice(&app).await;

    let csv = "\
invoice_number,title,vendor_name,amount,tax_amount,invoice_date,description,invoice_type
DUP-001,重复发票,供应商甲,99,9,2026-05-09,重复行,input
OK-001,办公设备,供应商乙,100.5,13,2026-05-10,有效行,input
BAD-001,错误金额,供应商丙,-1,0,2026-05-11,错误行,input
";

    let res = app
        .post("/api/v1/imports/invoices/preview")
        .json(&json!({
            "filename": "invoices.csv",
            "file_base64": encode(csv),
            "import_format": "csv"
        }))
        .await;
    res.assert_status_ok();

    let body = res.json::<Value>();
    assert_eq!(body["stats"]["total_count"].as_i64().unwrap(), 3);
    assert_eq!(body["stats"]["valid_count"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["invalid_count"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["duplicate_count"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["valid_label"], "1 条有效数据");
    assert_eq!(body["can_import"].as_bool(), Some(true));
    assert_eq!(body["import_button_label"], "导入 1 条有效数据");

    let rows = body["rows"].as_array().unwrap();
    assert_eq!(rows[0]["status"], "duplicate");
    assert_eq!(rows[0]["status_badge"]["label"], "重复");
    assert_eq!(rows[1]["status"], "valid");
    assert_eq!(rows[1]["amount_label"], "¥100.50");
    assert_eq!(rows[1]["tax_amount_label"], "¥13");
    assert_eq!(rows[1]["status_badge"]["label"], "有效");
    assert_eq!(rows[2]["status"], "invalid");
    assert!(rows[2]["error"].as_str().unwrap().contains("金额无效"));

    let list = app.get("/api/v1/invoices").await;
    list.assert_status_ok();
    assert_eq!(list.json::<Value>()["count"].as_i64().unwrap(), 1);
}

#[tokio::test]
async fn s124_invoice_import_commit_imports_only_backend_valid_rows() {
    let app = common::TestApp::new().await;
    seed_existing_invoice(&app).await;

    let csv = "\
invoice_number,title,vendor_name,amount,tax_amount,invoice_date,description,invoice_type
DUP-001,重复发票,供应商甲,99,9,2026-05-09,重复行,input
OK-001,办公设备,供应商乙,100.5,13,2026-05-10,有效行,input
BAD-001,错误金额,供应商丙,-1,0,2026-05-11,错误行,input
";

    let res = app
        .post("/api/v1/imports/invoices/commit")
        .json(&json!({
            "filename": "invoices.csv",
            "file_base64": encode(csv),
            "import_format": "csv"
        }))
        .await;
    res.assert_status_ok();

    let body = res.json::<Value>();
    assert_eq!(body["stats"]["valid_count"].as_i64().unwrap(), 1);
    assert_eq!(body["success_count"].as_i64().unwrap(), 1);
    assert_eq!(body["failed_count"].as_i64().unwrap(), 2);
    assert_eq!(body["success_message"], "成功导入 1 条发票，跳过 2 条");

    let list = app.get("/api/v1/invoices").await;
    list.assert_status_ok();
    let list_body = list.json::<Value>();
    assert_eq!(list_body["count"].as_i64().unwrap(), 2);
    assert!(
        list_body["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["invoice_number"] == "OK-001")
    );
}

#[tokio::test]
async fn s124_invoice_import_preview_parses_xml_on_backend() {
    let app = common::TestApp::new().await;
    let xml = r#"
<Invoice>
  <Fpdm>044001</Fpdm>
  <Fphm>9001</Fphm>
  <Kprq>2026-05-12</Kprq>
  <Xfmc>XML供应商</Xfmc>
  <Hjje>300.25</Hjje>
  <Hjse>39.03</Hjse>
  <Slv>13</Slv>
  <Spxx>XML服务费</Spxx>
  <Bz>后端解析XML</Bz>
</Invoice>
"#;

    let res = app
        .post("/api/v1/imports/invoices/preview")
        .json(&json!({
            "filename": "invoice.xml",
            "file_base64": encode(xml),
            "import_format": "xml"
        }))
        .await;
    res.assert_status_ok();

    let body = res.json::<Value>();
    assert_eq!(body["stats"]["valid_count"].as_i64().unwrap(), 1);
    let row = &body["rows"].as_array().unwrap()[0];
    assert_eq!(row["invoice_number"], "044001-9001");
    assert_eq!(row["supplier"], "XML供应商");
    assert_eq!(row["amount_label"], "¥300.25");
    assert_eq!(row["status_badge"]["label"], "有效");
}

fn encode(input: &str) -> String {
    general_purpose::STANDARD.encode(input.as_bytes())
}
