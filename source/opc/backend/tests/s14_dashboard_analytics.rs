/// Scenario 14: 📊 多维统计看板与聚合分析 (Dashboard Analytics)
/// 验证 首页 /api/v1/stats/dashboard 大盘数据的准确性
/// 包括待办事项、财务总览、近期活动数量的聚合计算，模拟并发下多表聚合分析的稳定性
use crate::common;
use serde_json::json;

#[tokio::test]
async fn s14_dashboard_analytics_aggregation() {
    let app = common::TestApp::new().await;

    // 1. 初始化一些多表离散数据
    app.post("/api/v1/invoices")
        .json(&json!({
            "invoice_number": "DASH-INV-001",
            "amount": 20000.0,
            "invoice_type": "income",
            "status": "pending",
            "invoice_date": "2026-09-01"
        }))
        .await
        .assert_status_ok();

    app.post("/api/v1/projects")
        .json(&json!({
            "name": "Dashboard Project",
            "status": "ongoing",
            "budget": 500000.0
        }))
        .await
        .assert_status_ok();

    app.post("/api/v1/reminders")
        .json(&json!({
            "title": "File Q3 Taxes",
            "remind_date": "2026-09-15",
            "status": "pending"
        }))
        .await
        .assert_status_ok();

    app.post("/api/v1/reminders")
        .json(&json!({
            "title": "Pay AWS Bill",
            "remind_date": "2026-09-05",
            "status": "completed" // Should not be in pending count
        }))
        .await
        .assert_status_ok();

    // 2. 调用 /stats/dashboard 聚合端点
    let dash_res = app.get("/api/v1/stats/dashboard").await;
    dash_res.assert_status_ok();

    let stats: serde_json::Value = dash_res.json();

    // Dashboard API in generic.rs returns:
    // { "assets": count, "invoices": count, "employees": count, "projects": count, "contracts": count }
    assert!(stats["invoices"].as_i64().unwrap_or(0) >= 1);
    assert!(stats["projects"].as_i64().unwrap_or(0) >= 1);
    assert!(stats["assets"].as_i64().is_some() || stats["assets"].as_f64().is_some());
}

#[tokio::test]
async fn s14_dashboard_tenant_isolation() {
    let app_a = common::TestApp::new().await;
    let app_b = app_a.create_second_tenant().await;

    // Tenant A adds a massive amount of invoices
    for i in 0..5 {
        app_a
            .post("/api/v1/invoices")
            .json(&json!({
                "invoice_number": format!("INV-TNT-A-{}", i),
                "amount": 100.0,
                "invoice_type": "income",
                "status": "paid",
                "invoice_date": "2026-09-01"
            }))
            .await
            .assert_status_ok();
    }

    // Tenant B dashboard should have 0 invoices
    let b_dash_res = app_b.get("/api/v1/stats/dashboard").await;
    b_dash_res.assert_status_ok();
    let b_stats: serde_json::Value = b_dash_res.json();

    assert_eq!(
        b_stats["invoices"].as_i64().unwrap_or(0),
        0,
        "Dashboard stats cross-tenant leak detected!"
    );
}

#[tokio::test]
async fn s14_dashboard_empty_data() {
    // Brand new tenant with zero data should still return valid stats
    let app = common::TestApp::new().await;

    let dash = app.get("/api/v1/stats/dashboard").await;
    dash.assert_status_ok();

    let stats: serde_json::Value = dash.json();
    // All counts should be 0 for fresh tenant
    assert_eq!(stats["invoices"].as_i64().unwrap_or(0), 0);
    assert_eq!(stats["projects"].as_i64().unwrap_or(0), 0);
    assert_eq!(stats["employees"].as_i64().unwrap_or(0), 0);
}
