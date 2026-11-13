/// Scenario 5: 💰 预算编制与执行
/// 创建预算 → 查看 → 分页 → 金额验证
///  → CSV 导出 → 删除 → 租户隔离
use crate::common;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn s05_budget_lifecycle() {
    let app = common::TestApp::new().await;

    // Step 1: Create budget (requires category, budget_type, period)
    let res = app
        .post("/api/v1/budgets")
        .json(&json!({
            "category": "运营费用",
            "budget_type": "annual",
            "period": "2026",
            "budget_amount": 500000.0,
            "notes": "年度总预算"
        }))
        .await;
    res.assert_status_ok();
    let budget: serde_json::Value = res.json();
    let id = budget["id"].as_str().expect("should have id");

    // Step 2: Get
    let get_res = app.get(&format!("/api/v1/budgets/{}", id)).await;
    get_res.assert_status_ok();
    let fetched: serde_json::Value = get_res.json();
    assert_eq!(fetched["category"], "运营费用");
    assert_eq!(fetched["budget_type"], "annual");

    // Step 3: Update
    let upd_res = app
        .put(&format!("/api/v1/budgets/{}", id))
        .json(&json!({
            "category": "运营费用",
            "budget_type": "annual",
            "period": "2026",
            "budget_amount": 600000.0,
            "notes": "调增10万"
        }))
        .await;
    upd_res.assert_status_ok();

    // Step 4: Delete
    let del_res = app.delete(&format!("/api/v1/budgets/{}", id)).await;
    del_res.assert_status(StatusCode::NO_CONTENT);

    // Step 5: 404
    app.get(&format!("/api/v1/budgets/{}", id))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn s05_budget_negative_validation() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/budgets")
        .json(&json!({
            "category": "Bad",
            "budget_type": "annual",
            "period": "2026",
            "budget_amount": -100.0
        }))
        .await;
    res.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn s05_budget_pagination() {
    let app = common::TestApp::new().await;

    for i in 1..=4 {
        app.post("/api/v1/budgets")
            .json(&json!({
                "category": format!("Q{}", i),
                "budget_type": "quarterly",
                "period": format!("2026-Q{}", i),
                "budget_amount": 100000.0 * (i as f64)
            }))
            .await
            .assert_status_ok();
    }

    let res = app.get("/api/v1/budgets?page=1&page_size=2").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["count"], 4);
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn s05_budget_csv_export() {
    let app = common::TestApp::new().await;

    app.post("/api/v1/budgets")
        .json(&json!({
            "category": "Export Test",
            "budget_type": "annual",
            "period": "2026",
            "budget_amount": 999999.0
        }))
        .await
        .assert_status_ok();

    let csv_res = app.get("/api/v1/export/budgets").await;
    csv_res.assert_status_ok();
    let csv_text = csv_res.text();
    assert!(csv_text.contains("Export Test"));
    assert!(csv_text.contains("999999"));
}

#[tokio::test]
async fn s05_budget_tenant_isolation() {
    let app_a = common::TestApp::new().await;
    let app_b = app_a.create_second_tenant().await;

    app_a
        .post("/api/v1/budgets")
        .json(&json!({
            "category": "Secret",
            "budget_type": "annual",
            "period": "2026",
            "budget_amount": 1000000.0
        }))
        .await
        .assert_status_ok();

    let res = app_b.get("/api/v1/budgets").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["count"], 0);
}
