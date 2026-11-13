use crate::common;

#[tokio::test]
async fn s110_financial_health_returns_metric_labels_badges_and_chart_display_fields() {
    let app = common::TestApp::new().await;

    let res = app.get("/api/v1/reports/financial-health").await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();

    assert_eq!(body["metrics"]["asset_liability_ratio_label"], "0.0%");
    assert_eq!(body["metrics"]["current_ratio_label"], "0.00");
    assert_eq!(body["metrics"]["profit_margin_label"], "0.0%");
    assert_eq!(body["metrics"]["operating_cash_flow_label"], "¥0");
    assert_eq!(
        body["metrics"]["operating_cash_flow_trend"]["icon"],
        "TrendingUp"
    );
    assert_eq!(body["metrics"]["profit_margin_trend"]["icon"], "TrendingUp");

    assert_eq!(body["health_score"]["badge"]["label"], "一般");
    assert_eq!(body["health_score"]["badge"]["class_name"], "bg-yellow-500");
    assert_eq!(body["health_score"]["total_label"], "55");
    assert_eq!(body["health_score"]["scale_label"], "/ 100");

    let radar = body["charts"]["radar"].as_array().unwrap();
    assert_eq!(radar[0]["fullMark"], 100);
    assert_eq!(radar[0]["score_label"], "100");

    let bar = body["charts"]["bar"].as_array().unwrap();
    assert_eq!(bar[0]["value_label"], "0.0%");
    assert_eq!(bar[1]["value_label"], "0.00");

    let recommendations = body["recommendations"].as_array().unwrap();
    assert_eq!(recommendations[0]["type"], "success");
    assert_eq!(recommendations[0]["variant"], "default");
    assert_eq!(recommendations[0]["icon"], "CheckCircle2");
    assert_eq!(recommendations[1]["type"], "warning");
    assert_eq!(recommendations[1]["variant"], "destructive");
    assert_eq!(recommendations[1]["icon"], "AlertCircle");
}
