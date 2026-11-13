use axum::http::StatusCode;
use axum_test::TestServer;

#[tokio::test]
async fn s131_spa_routes_return_index_html_with_success_status() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect sqlite");
    let app = opc::build_app(pool);
    let server = TestServer::new(app).expect("test server");

    let route = server.get("/dashboard").await;
    route.assert_status_ok();
    assert!(route.text().contains("id=\"root\""));

    let unknown_nested_route = server.get("/settings/profile").await;
    unknown_nested_route.assert_status_ok();
    assert!(unknown_nested_route.text().contains("id=\"root\""));
}

#[tokio::test]
async fn s131_api_routes_do_not_fall_through_to_spa_index() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect sqlite");
    let app = opc::build_app(pool);
    let server = TestServer::new(app).expect("test server");

    for path in [
        "/api/v1/ai/insights",
        "/api/v1/onboarding/progress",
        "/api/v1/workbench/summary",
        "/api/ai/chat-context",
        "/api/profiles/current",
        "/api/v1/reminders/overview",
    ] {
        let response = server.get(path).await;
        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED, "{path}");
        assert!(
            !response.text().contains("id=\"root\""),
            "{path} returned the SPA index"
        );
    }

    let missing_api = server.get("/api/v1/not-a-real-route").await;
    assert_eq!(missing_api.status_code(), StatusCode::NOT_FOUND);
    assert!(!missing_api.text().contains("id=\"root\""));
}
