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
) {
    sqlx::query(
        r#"
        INSERT INTO invoices (
            id, company_id, invoice_number, invoice_type, amount, tax_amount, status
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(invoice_number)
    .bind(invoice_type)
    .bind(amount)
    .bind(tax_amount)
    .bind(status)
    .execute(&app.pool)
    .await
    .expect("seed invoice");
}

#[tokio::test]
async fn s80_invoice_overview_calculates_direction_stats_on_backend() {
    let app = common::TestApp::new().await;

    seed_invoice(
        &app,
        "output",
        "OUT-001",
        Some("output"),
        100.0,
        10.0,
        "sent",
    )
    .await;
    seed_invoice(
        &app,
        "income",
        "INC-001",
        Some("income"),
        200.0,
        20.0,
        "approved",
    )
    .await;
    seed_invoice(&app, "legacy", "LEG-001", None, 300.0, 30.0, "pending").await;
    seed_invoice(&app, "input", "IN-001", Some("input"), 80.0, 8.0, "pending").await;
    seed_invoice(
        &app,
        "expense",
        "EXP-001",
        Some("expense"),
        40.0,
        4.0,
        "paid",
    )
    .await;

    let res = app.get("/api/v1/invoices/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["output_stats"]["total"].as_i64().unwrap(), 3);
    assert_eq!(body["output_stats"]["pending"].as_i64().unwrap(), 1);
    assert_eq!(body["output_stats"]["sent"].as_i64().unwrap(), 2);
    assert_eq!(
        body["output_stats"]["total_amount"].as_f64().unwrap(),
        600.0
    );
    assert_eq!(body["output_stats"]["total_tax"].as_f64().unwrap(), 60.0);

    assert_eq!(body["input_stats"]["total"].as_i64().unwrap(), 2);
    assert_eq!(body["input_stats"]["pending"].as_i64().unwrap(), 1);
    assert_eq!(body["input_stats"]["paid"].as_i64().unwrap(), 1);
    assert_eq!(body["input_stats"]["total_amount"].as_f64().unwrap(), 120.0);
    assert_eq!(body["input_stats"]["total_tax"].as_f64().unwrap(), 12.0);

    assert_eq!(body["output_invoices"].as_array().unwrap().len(), 3);
    assert_eq!(body["input_invoices"].as_array().unwrap().len(), 2);
    assert_eq!(body["invoices"].as_array().unwrap().len(), 5);
}

#[tokio::test]
async fn s80_invoice_overview_is_company_scoped() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_invoice(
        &app,
        "owned",
        "OWN-001",
        Some("output"),
        100.0,
        10.0,
        "sent",
    )
    .await;
    seed_invoice(
        &other,
        "other",
        "OTHER-001",
        Some("output"),
        900.0,
        90.0,
        "sent",
    )
    .await;

    let res = app.get("/api/v1/invoices/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["output_stats"]["total"].as_i64().unwrap(), 1);
    assert_eq!(
        body["output_stats"]["total_amount"].as_f64().unwrap(),
        100.0
    );
    assert_eq!(body["invoices"].as_array().unwrap().len(), 1);
    assert_eq!(body["invoices"][0]["id"], "owned");
}
