use crate::common;

use serde_json::Value;

#[tokio::test]
async fn s121_system_status_returns_display_ready_operational_metadata() {
    let app = common::TestApp::new().await;

    let res = app.get("/api/v1/system/status").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert!(body["health"]["status_label"].as_str().is_some());
    assert!(body["health"]["status_badge"]["label"].as_str().is_some());
    assert!(body["health"]["database_label"].as_str().is_some());
    assert!(body["health"]["database_badge"]["label"].as_str().is_some());
    assert!(body["health"]["timestamp_label"].as_str().is_some());

    assert!(body["database"]["file_size_label"].as_str().is_some());
    assert!(body["database"]["total_rows_label"].as_str().is_some());
    assert!(body["database"]["table_count_label"].as_str().is_some());
    assert!(body["database"]["summary_label"].as_str().is_some());
    assert!(body["database"]["health_percent"].as_i64().is_some());
    assert!(body["database"]["health_percent_label"].as_str().is_some());

    let tables = body["database"]["tables"].as_array().unwrap();
    assert!(!tables.is_empty());
    assert!(tables[0]["count_label"].as_str().is_some());
    assert!(tables[0]["status_badge"]["label"].as_str().is_some());

    assert!(body["storage"]["data_files_label"].as_str().is_some());
    assert!(body["api"]["summary_label"].as_str().is_some());
    assert!(body["api"]["health_percent_label"].as_str().is_some());
    let endpoints = body["api"]["endpoints"].as_array().unwrap();
    assert!(!endpoints.is_empty());
    assert!(endpoints[0]["status_badge"]["label"].as_str().is_some());
}
