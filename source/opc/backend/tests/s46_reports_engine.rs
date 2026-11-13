/// Scenario 46: 📊 Reports Engine — Backend Data Layer
/// Tests server-side report endpoints for financial health, cash flow, profit analysis, deals.
use crate::common;
use serde_json::json;

// ==================== Financial Health — Empty State ====================

#[tokio::test]
async fn s46_financial_health_empty() {
    let app = common::TestApp::new().await;
    let res = app.get("/api/v1/reports/financial-health").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert!(body["health_score"]["total"].is_number());
    assert!(body["health_score"]["level"].is_string());
    assert!(body["metrics"]["asset_liability_ratio"].is_number());
    assert!(body["metrics"]["current_ratio"].is_number());
    assert!(body["metrics"]["profit_margin"].is_number());
    assert!(body["metrics"]["operating_cash_flow"].is_number());
}

// ==================== Financial Health — With Data ====================

#[tokio::test]
async fn s46_financial_health_with_data() {
    let app = common::TestApp::new().await;

    // Create an asset
    app.post("/api/v1/assets")
        .json(&json!({ "name": "Server", "purchase_price": 100000.0, "current_value": 80000.0 }))
        .await
        .assert_status_ok();

    // Create income payments
    app.post("/api/v1/payments")
        .json(&json!({ "status": "completed",  "description": "Project payment", "amount": 50000.0, "payment_type": "income", "payment_date": "2026-03-01" }))
        .await.assert_status_ok();

    // Create expense
    app.post("/api/v1/payments")
        .json(&json!({ "status": "completed",  "description": "Office rent", "amount": 10000.0, "payment_type": "expense", "payment_date": "2026-03-01" }))
        .await.assert_status_ok();

    let res = app.get("/api/v1/reports/financial-health").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();

    // Should have positive profit margin (50k income - 10k expense = 80% margin)
    let profit_margin = body["metrics"]["profit_margin"].as_f64().unwrap();
    assert!(
        profit_margin > 0.0,
        "profit margin should be positive: {}",
        profit_margin
    );

    // Cash flow should be 40000
    let cash_flow = body["metrics"]["operating_cash_flow"].as_f64().unwrap();
    assert_eq!(cash_flow, 40000.0);

    // Total assets should be 80000
    let total_assets = body["raw"]["total_assets"].as_f64().unwrap();
    assert_eq!(total_assets, 80000.0);

    // Health level should exist
    let level = body["health_score"]["level"].as_str().unwrap();
    assert!(["excellent", "good", "fair", "poor"].contains(&level));

    let recommendations = body["recommendations"].as_array().unwrap();
    assert_eq!(
        recommendations.len(),
        4,
        "backend should prepare financial recommendations"
    );
    assert!(recommendations.iter().any(|rec| {
        rec["type"] == "success" && rec["message"].as_str().unwrap_or("").contains("经营现金流")
    }));

    assert_eq!(body["charts"]["radar"].as_array().unwrap().len(), 4);
    assert_eq!(body["charts"]["bar"].as_array().unwrap().len(), 3);
}

// ==================== Cash Flow Forecast — Default 3 months ====================

#[tokio::test]
async fn s46_cashflow_forecast_default() {
    let app = common::TestApp::new().await;
    let res = app.get("/api/v1/reports/cashflow-forecast").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();

    assert_eq!(body["forecast_months"], 3);
    let forecast = body["forecast"].as_array().unwrap();
    assert_eq!(forecast.len(), 3);

    // Each month should have breakdown
    for m in forecast {
        assert!(m["month"].is_string());
        assert!(m["cash_inflow"].is_number());
        assert!(m["cash_outflow"].is_number());
        assert!(m["net_cash_flow"].is_number());
        assert!(m["breakdown"].is_object());
    }

    // Summary should exist
    assert!(body["summary"]["total_inflow"].is_number());
    assert!(body["summary"]["avg_monthly_net"].is_number());
}

#[tokio::test]
async fn s46_cashflow_forecast_returns_backend_display_fields() {
    let app = common::TestApp::new().await;
    app.post("/api/v1/rent")
        .json(&json!({ "location_name": "Office Display", "monthly_rent": 5000.0, "status": "active" }))
        .await
        .assert_status_ok();

    let res = app.get("/api/v1/reports/cashflow-forecast?months=3").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();

    assert!(
        body["summary"]["total_inflow_label"]
            .as_str()
            .unwrap()
            .starts_with("¥")
    );
    assert!(
        body["summary"]["total_outflow_label"]
            .as_str()
            .unwrap()
            .starts_with("¥")
    );
    assert!(
        body["summary"]["net_cash_flow_label"]
            .as_str()
            .unwrap()
            .starts_with("¥")
    );
    assert!(
        body["summary"]["avg_monthly_net_label"]
            .as_str()
            .unwrap()
            .starts_with("¥")
    );
    assert!(body["summary"]["net_cash_flow_class"].is_string());
    assert_eq!(body["month_options"][0]["value"], "3");
    assert_eq!(body["month_options"][0]["label"], "未来3个月");
    assert_eq!(body["month_options"][1]["value"], "6");
    assert_eq!(body["chart_points"].as_array().unwrap().len(), 3);

    let row = &body["forecast"].as_array().unwrap()[0];
    assert!(row["month_label"].as_str().unwrap().contains("年"));
    assert!(row["month_short_label"].as_str().unwrap().ends_with("月"));
    assert!(row["receivables_label"].as_str().unwrap().starts_with("¥"));
    assert!(
        row["contract_income_label"]
            .as_str()
            .unwrap()
            .starts_with("¥")
    );
    assert!(row["salary_label"].as_str().unwrap().starts_with("¥"));
    assert!(row["rent_label"].as_str().unwrap().starts_with("¥"));
    assert!(row["subscription_label"].as_str().unwrap().starts_with("¥"));
    assert!(row["payables_label"].as_str().unwrap().starts_with("¥"));
    assert!(row["cash_inflow_label"].as_str().unwrap().starts_with("¥"));
    assert!(row["cash_outflow_label"].as_str().unwrap().starts_with("¥"));
    assert!(
        row["net_cash_flow_label"]
            .as_str()
            .unwrap()
            .starts_with("¥")
    );
    assert!(row["net_cash_flow_class"].is_string());
}

// ==================== Cash Flow Forecast — Custom months ====================

#[tokio::test]
async fn s46_cashflow_forecast_6_months() {
    let app = common::TestApp::new().await;
    let res = app.get("/api/v1/reports/cashflow-forecast?months=6").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["forecast_months"], 6);
    assert_eq!(body["forecast"].as_array().unwrap().len(), 6);
}

// ==================== Cash Flow — With Baselines ====================

#[tokio::test]
async fn s46_cashflow_with_rent_and_salary() {
    let app = common::TestApp::new().await;

    // Create rent
    app.post("/api/v1/rent")
        .json(&json!({ "location_name": "Office A", "monthly_rent": 5000.0, "status": "active" }))
        .await
        .assert_status_ok();

    let res = app.get("/api/v1/reports/cashflow-forecast").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();

    let monthly_rent = body["baselines"]["monthly_rent"].as_f64().unwrap();
    assert_eq!(monthly_rent, 5000.0, "rent baseline should be 5000");

    // Each forecast month should show rent in outflow
    for m in body["forecast"].as_array().unwrap() {
        let rent = m["breakdown"]["rent"].as_f64().unwrap();
        assert_eq!(rent, 5000.0);
    }
}

// ==================== Profit Analysis — Empty ====================

#[tokio::test]
async fn s46_profit_analysis_empty() {
    let app = common::TestApp::new().await;
    let res = app.get("/api/v1/reports/profit-analysis").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["total_revenue"], 0.0);
    assert_eq!(body["client_count"], 0);
    assert_eq!(body["project_count"], 0);
    assert!(body["by_client"].as_array().unwrap().is_empty());
}

// ==================== Profit Analysis — With Revenue ====================

#[tokio::test]
async fn s46_profit_analysis_with_invoices() {
    let app = common::TestApp::new().await;

    // Create receivable invoices via dedicated invoices module
    app.post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-RPT-001",
            "amount": 20000.0,
            "invoice_type": "receivable",
            "supplier": "Client Alpha"
        }))
        .await
        .assert_status_ok();

    app.post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-RPT-002",
            "amount": 35000.0,
            "invoice_type": "receivable",
            "supplier": "Client Alpha"
        }))
        .await
        .assert_status_ok();

    app.post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-RPT-003",
            "amount": 15000.0,
            "invoice_type": "receivable",
            "supplier": "Client Beta"
        }))
        .await
        .assert_status_ok();

    let res = app.get("/api/v1/reports/profit-analysis").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();

    let total = body["total_revenue"].as_f64().unwrap();
    assert_eq!(total, 70000.0, "total revenue should be 70k");
    assert_eq!(body["client_count"], 2);

    // Client Alpha should have 55k
    let clients = body["by_client"].as_array().unwrap();
    let alpha = clients
        .iter()
        .find(|c| c["client"] == "Client Alpha")
        .unwrap();
    assert_eq!(alpha["revenue"], 55000.0);
    assert_eq!(alpha["invoice_count"], 2);
}

// ==================== Deals Conversion — Empty ====================

#[tokio::test]
async fn s46_deals_conversion_empty() {
    let app = common::TestApp::new().await;
    let res = app.get("/api/v1/reports/deals-conversion").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["pipeline"]["total_deals"], 0);
    assert_eq!(body["conversion"]["win_rate"], 0.0);
    assert!(body["top_lost_reasons"].as_array().unwrap().is_empty());
}

// ==================== Deals Conversion — With Pipeline ====================

#[tokio::test]
async fn s46_deals_conversion_pipeline() {
    let app = common::TestApp::new().await;

    // Create client first
    let cl = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "Deal客户", "client_type": "corporate" }))
        .await;
    cl.assert_status_ok();
    let cl_id = cl.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Won deals
    app.post("/api/v1/deals")
        .json(
            &json!({ "title": "Deal A", "client_id": &cl_id, "stage": "won", "amount": 100000.0 }),
        )
        .await
        .assert_status_ok();
    app.post("/api/v1/deals")
        .json(&json!({ "title": "Deal B", "client_id": &cl_id, "stage": "won", "amount": 50000.0 }))
        .await
        .assert_status_ok();

    // Lost deals
    app.post("/api/v1/deals")
        .json(&json!({ "title": "Deal C", "client_id": &cl_id, "stage": "lost", "amount": 30000.0, "lost_reason": "价格太高" }))
        .await.assert_status_ok();

    // Pipeline deals
    app.post("/api/v1/deals")
        .json(&json!({ "title": "Deal D", "client_id": &cl_id, "stage": "proposal", "amount": 80000.0 }))
        .await.assert_status_ok();

    let res = app.get("/api/v1/reports/deals-conversion").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();

    assert_eq!(body["pipeline"]["total_deals"], 4);
    assert_eq!(body["conversion"]["won"]["count"], 2);
    assert_eq!(body["conversion"]["won"]["amount"], 150000.0);
    assert_eq!(body["conversion"]["lost"]["count"], 1);

    // Win rate should be 2/3 ≈ 66.7%
    let win_rate = body["conversion"]["win_rate"].as_f64().unwrap();
    assert!(
        (win_rate - 66.7).abs() < 0.1,
        "win rate should be ~66.7%: {}",
        win_rate
    );

    // Avg deal size
    assert_eq!(body["conversion"]["won"]["avg_deal_size"], 75000.0);

    // Lost reasons
    let reasons = body["top_lost_reasons"].as_array().unwrap();
    assert!(!reasons.is_empty());
    assert_eq!(reasons[0]["reason"], "价格太高");
}

#[tokio::test]
async fn s46_only_completed_and_legacy_paid_payments_affect_balances() {
    use serde_json::Value;
    let app = common::TestApp::new().await;
    sqlx::query("INSERT INTO invoices(id,company_id,invoice_number,invoice_type,amount,status,currency,created_by) VALUES ('status-invoice',?,'STATUS-1','receivable',1000,'pending','CNY',?)")
        .bind(&app.company_id).bind(&app.user_id).execute(&app.pool).await.unwrap();
    for (status, amount) in [
        (Some("completed"), 100.0),
        (Some("paid"), 50.0),
        (Some("pending"), 200.0),
        (Some("cancelled"), 300.0),
        (Some("unknown"), 400.0),
        (None, 500.0),
    ] {
        for kind in ["income", "expense"] {
            sqlx::query("INSERT INTO payments(id,company_id,payment_type,amount,status,currency,payment_date,invoice_id) VALUES (?,?,?,?,?,'CNY','2026-03-01',?)")
                .bind(uuid::Uuid::new_v4().to_string()).bind(&app.company_id).bind(kind)
                .bind(amount).bind(status).bind(if kind == "income" { Some("status-invoice") } else { None })
                .execute(&app.pool).await.unwrap();
        }
    }
    let health = app
        .get("/api/v1/reports/financial-health")
        .await
        .json::<Value>();
    assert_eq!(health["raw"]["income"], 150.0);
    assert_eq!(health["raw"]["expense"], 150.0);
    let overview = app.get("/api/payments/overview").await.json::<Value>();
    assert_eq!(overview["stats"]["total"], 12);
    assert_eq!(overview["payments"].as_array().unwrap().len(), 12);
    assert_eq!(overview["stats"]["income"], 150.0);
    assert_eq!(overview["stats"]["expense"], 150.0);
    let invoices = app.get("/api/v1/invoices/overview").await.json::<Value>();
    assert_eq!(invoices["invoices"][0]["paid_total"], 150.0);
    let balances = app.get("/api/v1/currency/balances").await.json::<Value>();
    let cny = balances["currencies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["currency"] == "CNY")
        .unwrap();
    assert_eq!(cny["income"], 150.0);
    assert_eq!(cny["expenses"], 150.0);
    assert_eq!(cny["receivables"], 850.0);
    let monthly = app
        .post("/api/v1/reports/monthly-close?period=2026-03")
        .json(&json!({}))
        .await
        .json::<Value>();
    assert_eq!(monthly["income"], 150.0);
    assert_eq!(monthly["expense"], 150.0);
}
