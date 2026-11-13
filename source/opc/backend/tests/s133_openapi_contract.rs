use crate::common;

#[tokio::test]
async fn s133_openapi_entrypoints_are_public_and_cover_registered_routes() {
    let app = common::TestApp::new().await;

    for entrypoint in ["/openapi.json", "/api/openapi.json"] {
        let response = app.server.get(entrypoint).await;
        response.assert_status_ok();
        let spec: serde_json::Value = response.json();

        assert_eq!(spec["openapi"], "3.1.0", "{entrypoint} must be OpenAPI 3.1");
        assert_eq!(spec["info"]["title"], "OPC API");
        let paths = spec["paths"].as_object().expect("paths must be an object");

        for path in [
            "/openapi.json",
            "/api/openapi.json",
            "/health",
            "/auth/login",
            "/api/v1/companies",
            "/api/v1/companies/{id}/members",
            "/api/v1/invoices",
            "/api/v1/reports/financial-health",
            "/api/companies",
            "/api/invoices",
            "/api/files/static/{path}",
            "/api/v1/mcp",
        ] {
            assert!(paths.contains_key(path), "missing OpenAPI path {path}");
        }

        assert!(paths["/api/v1/companies"]["get"].is_object());
        assert!(paths["/api/v1/companies"]["post"].is_object());
        assert!(paths["/api/v1/invoices/{id}"]["put"].is_object());
        assert!(paths["/api/files/static/{path}"]["get"].is_object());
        assert!(
            paths.len() >= 220,
            "expected full route coverage, got {} paths",
            paths.len()
        );
    }
}

#[tokio::test]
async fn s133_openapi_declares_tenant_backup_denial_and_session_only_key_creation() {
    let app = common::TestApp::new().await;
    let response = app.server.get("/openapi.json").await;
    response.assert_status_ok();
    let spec: serde_json::Value = response.json();
    for prefix in ["/api", "/api/v1"] {
        for (path, method) in [("/settings/backups", "get"), ("/settings/backups", "post"), ("/settings/backups/{file_name}/download", "get"), ("/settings/restore", "post")] {
            let operation = &spec["paths"][format!("{prefix}{path}")][method];
            assert!(operation["responses"]["403"].is_object());
            assert!(operation["responses"]["200"].is_null());
        }
        assert_eq!(spec["paths"][format!("{prefix}/api-keys")]["post"]["security"], serde_json::json!([{ "SessionBearer": [] }]));
    }
}

#[tokio::test]
async fn s133_openapi_declares_model_selection_and_review_failures() {
    let app = common::TestApp::new().await;
    let spec: serde_json::Value = app.server.get("/openapi.json").await.json();
    let models = &spec["paths"]["/api/ai/models"]["get"];
    assert!(models["security"].is_array());
    assert!(models["responses"]["502"].is_object());
    assert!(models["responses"]["200"]["content"]["application/json"]["schema"]["properties"]["models"].is_object());
    let chat = &spec["paths"]["/api/ai/chat"]["post"];
    assert!(chat["requestBody"]["content"]["application/json"]["schema"]["properties"]["model"].is_object());
    for endpoint in ["chat", "contract-review", "parse-document"] {
        assert!(spec["paths"][format!("/api/ai/{endpoint}")]["post"]["responses"]["502"].is_object());
    }
}
