use crate::common;

use serde_json::Value;

async fn seed_invoice(
    app: &common::TestApp,
    id: &str,
    invoice_number: &str,
    invoice_type: Option<&str>,
    amount: f64,
    tax_amount: f64,
    status: &str,
    supplier: &str,
    created_by: Option<&str>,
) {
    sqlx::query(
        r#"
        INSERT INTO invoices (
            id, company_id, invoice_number, invoice_type, amount, tax_amount,
            status, invoice_date, supplier, description, created_by
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, '2026-05-01', ?, 'Needle invoice', ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(invoice_number)
    .bind(invoice_type)
    .bind(amount)
    .bind(tax_amount)
    .bind(status)
    .bind(supplier)
    .bind(created_by)
    .execute(&app.pool)
    .await
    .expect("seed invoice");
}

#[tokio::test]
async fn s109_invoice_overview_returns_filtered_rows_labels_badges_and_permissions() {
    let app = common::TestApp::new().await;
    seed_invoice(
        &app,
        "invoice-needle",
        "INV-NEEDLE",
        Some("output"),
        1234.5,
        123.45,
        "sent",
        "Needle Customer",
        Some(&app.user_id),
    )
    .await;
    seed_invoice(
        &app,
        "invoice-other",
        "INV-OTHER",
        Some("output"),
        10.0,
        1.0,
        "pending",
        "Other Customer",
        Some(&app.user_id),
    )
    .await;
    seed_invoice(
        &app,
        "invoice-input",
        "IN-OTHER",
        Some("input"),
        88.0,
        8.0,
        "paid",
        "Input Vendor",
        Some(&app.user_id),
    )
    .await;

    let res = app
        .get("/api/v1/invoices/overview?search=needle&status=sent")
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["output_stats"]["total"], 2);
    assert_eq!(body["output_stats"]["total_amount_label"], "¥1,244.50");
    assert_eq!(body["output_stats"]["total_tax_label"], "¥124.45");
    assert_eq!(body["output_stats"]["pending_message"], "共 1 张待审批");
    assert_eq!(body["input_stats"]["total_amount_label"], "¥88");
    assert_eq!(body["permissions"]["can_create"], true);
    assert_eq!(body["permissions"]["can_import"], true);
    assert_eq!(body["create_action"]["method"], "POST");
    assert_eq!(body["create_action"]["endpoint"], "/api/invoices");
    assert_eq!(body["create_action"]["label"], "添加发票");
    assert_eq!(body["create_action"]["success_message"], "发票添加成功");
    assert_eq!(body["empty_messages"]["filtered"], "没有找到匹配的发票");
    assert!(
        body["status_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "sent" && option["label"] == "已开具")
    );
    assert_eq!(body["status_options"][0]["value"], "all");
    assert_eq!(body["status_options"][0]["label"], "全部状态");

    let output = body["output_invoices"].as_array().unwrap();
    assert_eq!(output.len(), 1);
    let invoice = &output[0];
    assert_eq!(invoice["id"], "invoice-needle");
    assert_eq!(invoice["amount_label"], "¥1,234.50");
    assert_eq!(invoice["tax_amount_label"], "¥123.45");
    assert_eq!(invoice["invoice_date_label"], "2026-05-01");
    assert_eq!(invoice["status_badge"]["label"], "已开具");
    assert_eq!(invoice["status_badge"]["variant"], "default");
    assert_eq!(invoice["status_badge"]["icon"], "CheckCircle");
    assert_eq!(invoice["can_edit"], true);
    assert_eq!(invoice["update_action"]["method"], "PUT");
    assert_eq!(
        invoice["update_action"]["endpoint"],
        "/api/invoices/invoice-needle"
    );
    assert_eq!(invoice["update_action"]["label"], "保存发票");
    assert_eq!(invoice["update_action"]["success_message"], "发票更新成功");
    assert_eq!(invoice["can_delete"], true);
    assert_eq!(invoice["delete_action"]["method"], "DELETE");
    assert_eq!(
        invoice["delete_action"]["endpoint"],
        "/api/invoices/invoice-needle"
    );
    assert_eq!(
        invoice["delete_action"]["confirm_message"],
        "确定要删除发票「INV-NEEDLE」吗？相关附件也将被删除。"
    );
    assert_eq!(invoice["delete_action"]["success_title"], "删除成功");
    assert_eq!(invoice["delete_action"]["success_message"], "发票删除成功");
    assert_eq!(invoice["can_mark_paid"], true);
    assert_eq!(invoice["can_mark_sent"], false);
    assert!(invoice["mark_sent_action"].is_null());
    assert_eq!(invoice["mark_paid_action"]["method"], "POST");
    assert_eq!(
        invoice["mark_paid_action"]["endpoint"],
        "/api/invoices/invoice-needle/mark_paid"
    );
    assert_eq!(invoice["mark_paid_action"]["label"], "标记已支付");
    assert_eq!(
        invoice["mark_paid_action"]["success_message"],
        "发票已标记为已支付"
    );

    let options = app.get("/api/v1/invoices/form_options").await;
    options.assert_status_ok();
    let options_body = options.json::<Value>();
    assert_eq!(options_body["default_status"], "pending");
    assert_eq!(options_body["default_invoice_type"], "receivable");
    assert!(
        options_body["status_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "paid" && option["label"] == "已支付")
    );
    assert!(
        options_body["invoice_type_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| {
                option["value"] == "payable" && option["label"] == "进项发票(应付)"
            })
    );
    assert!(
        options_body["invoice_type_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| {
                option["value"] == "receivable" && option["label"] == "销项发票(应收)"
            })
    );
}

#[tokio::test]
async fn s109_invoice_status_transitions_are_backend_owned() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;
    seed_invoice(
        &app,
        "invoice-flow",
        "INV-FLOW",
        Some("output"),
        500.0,
        50.0,
        "pending",
        "Flow Customer",
        Some(&app.user_id),
    )
    .await;
    seed_invoice(
        &other,
        "invoice-flow-other",
        "INV-FLOW-OTHER",
        Some("output"),
        500.0,
        50.0,
        "pending",
        "Other Customer",
        Some(&other.user_id),
    )
    .await;

    let update = app
        .put("/api/v1/invoices/invoice-flow")
        .json(&serde_json::json!({
            "status": "paid",
            "description": "frontend attempted status update"
        }))
        .await;
    update.assert_status_ok();
    let invoice = update.json::<Value>();
    assert_eq!(invoice["status"], "pending");
    assert_eq!(invoice["description"], "frontend attempted status update");

    let sent = app
        .post("/api/v1/invoices/invoice-flow/mark_sent")
        .json(&serde_json::json!({}))
        .await;
    sent.assert_status_ok();
    let sent = sent.json::<Value>();
    assert_eq!(sent["status"], "sent");
    assert_eq!(sent["status_badge"]["label"], "已开具");
    assert_eq!(sent["can_mark_sent"], false);
    assert_eq!(sent["can_mark_paid"], true);
    assert!(sent["mark_sent_action"].is_null());
    assert_eq!(
        sent["mark_paid_action"]["endpoint"],
        format!("/api/invoices/{}/mark_paid", "invoice-flow")
    );

    let paid = app
        .post("/api/v1/invoices/invoice-flow/mark_paid")
        .json(&serde_json::json!({}))
        .await;
    paid.assert_status_ok();
    let paid = paid.json::<Value>();
    assert_eq!(paid["status"], "paid");
    assert_eq!(paid["status_badge"]["label"], "已支付");
    assert_eq!(paid["can_mark_sent"], false);
    assert_eq!(paid["can_mark_paid"], false);
    assert!(paid["mark_sent_action"].is_null());
    assert!(paid["mark_paid_action"].is_null());

    let other_get = other.get("/api/v1/invoices/invoice-flow").await;
    other_get.assert_status_not_found();
}

#[tokio::test]
async fn s109_invoice_delete_requires_backend_permission() {
    let app = common::TestApp::new().await;
    seed_invoice(
        &app,
        "invoice-delete-guard",
        "INV-DELETE-GUARD",
        Some("output"),
        100.0,
        10.0,
        "pending",
        "Delete Guard",
        Some("someone-else"),
    )
    .await;
    sqlx::query("UPDATE company_members SET role = 'member' WHERE user_id = ? AND company_id = ?")
        .bind(&app.user_id)
        .bind(&app.company_id)
        .execute(&app.pool)
        .await
        .expect("downgrade member");

    let delete = app.delete("/api/v1/invoices/invoice-delete-guard").await;
    delete.assert_status_forbidden();
}
