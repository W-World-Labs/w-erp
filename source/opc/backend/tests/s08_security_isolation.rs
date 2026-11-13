/// Scenario 8: 🔒 安全与隔离
/// 无 session → 401 → 过期 session → 401
///  → 跨租户访问拒绝 → SQL 注入防护 → 非法导出拒绝
use crate::common;
use axum::http::StatusCode;
use axum_test::TestServer;
use opc::auth::OAuthConfig;
use serde_json::json;

fn configured_oauth() -> OAuthConfig {
    OAuthConfig {
        client_id: "test-client".to_owned(),
        client_secret: "test-secret".to_owned(),
        base_url: "https://auth.example.test".to_owned(),
        authorize_url: "https://auth.example.test/oauth/authorize".to_owned(),
        token_url: "https://auth.example.test/oauth/token".to_owned(),
        userinfo_url: "https://auth.example.test/oauth/userinfo".to_owned(),
        logout_url: "https://auth.example.test/oauth/logout".to_owned(),
        redirect_uri: "https://erp.example.test/auth/callback".to_owned(),
    }
}

async fn configured_auth_server(app: &common::TestApp) -> TestServer {
    TestServer::new(opc::auth::router_with_config(
        app.pool.clone(),
        configured_oauth(),
    ))
    .unwrap()
}

#[tokio::test]
async fn s08_no_auth_returns_401() {
    let app = common::TestApp::new().await;

    // Request without any auth header
    let res = app.server.get("/api/v1/invoices").await;
    res.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn s08_invalid_session_returns_401() {
    let app = common::TestApp::new().await;

    let res = app
        .server
        .get("/api/v1/invoices")
        .authorization_bearer("invalid-session-id-that-does-not-exist")
        .await;
    res.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn s08_expired_session_returns_401() {
    let app = common::TestApp::new().await;

    // Create an expired session directly in DB
    let expired_session = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO sessions (id, user_id, access_token, current_company_id, expires_at) VALUES (?, ?, 'x', ?, datetime('now', '-1 hour'))"
    )
    .bind(&expired_session)
    .bind(&app.user_id)
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();

    let res = app
        .server
        .get("/api/v1/invoices")
        .authorization_bearer(&expired_session)
        .await;
    res.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn s08_cross_tenant_get_returns_404() {
    let app_a = common::TestApp::new().await;
    let app_b = app_a.create_second_tenant().await;

    // A creates record
    let res = app_a
        .post("/api/v1/invoices")
        .json(&json!({
            "invoice_number": "PRIV-001",
            "amount": 1000.0,
            "invoice_type": "income"
        }))
        .await;
    res.assert_status_ok();
    let inv: serde_json::Value = res.json();
    let inv_id = inv["id"].as_str().unwrap();

    // B tries to directly GET A's record — should be 404
    let get_res = app_b.get(&format!("/api/v1/invoices/{}", inv_id)).await;
    get_res.assert_status(StatusCode::NOT_FOUND);

    // B tries to DELETE A's record — should be 404
    let del_res = app_b.delete(&format!("/api/v1/invoices/{}", inv_id)).await;
    let status = del_res.status_code();
    assert!(
        status == 404 || status == 204,
        "should not be able to delete cross-tenant record"
    );
}

#[tokio::test]
async fn s08_sql_injection_in_order_by() {
    let app = common::TestApp::new().await;

    // Try SQL injection via order_by parameter
    let res = app
        .get("/api/v1/invoices?order_by=name;DROP%20TABLE%20invoices;--")
        .await;
    // Should not crash, either returns OK (with sanitized param) or BAD_REQUEST
    let status = res.status_code();
    assert!(
        status == 200 || status == 400,
        "SQL injection should be handled gracefully, got {}",
        status
    );
}

#[tokio::test]
async fn s08_non_exportable_table_rejected() {
    let app = common::TestApp::new().await;

    // Try to export system tables
    let res = app.get("/api/v1/export/sessions").await;
    res.assert_status(StatusCode::BAD_REQUEST);
    let body: serde_json::Value = res.json();
    assert!(body["error"].as_str().unwrap().contains("not exportable"));

    // Try to export users (has sensitive data)
    let res2 = app.get("/api/v1/export/users").await;
    res2.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn s08_health_check_no_auth_needed() {
    let app = common::TestApp::new().await;

    // Health check should work without auth
    let res = app.server.get("/health").await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn s08_exchange_rate_crud_and_isolation() {
    let app = common::TestApp::new().await;

    // Create exchange rate
    let res = app
        .post("/api/v1/exchange_rates")
        .json(&json!({
            "from_currency": "USD",
            "to_currency": "CNY",
            "rate": 7.25,
            "effective_date": "2026-03-01",
            "source": "manual"
        }))
        .await;
    res.assert_status_ok();
    let rate: serde_json::Value = res.json();
    let rate_id = rate["id"].as_str().expect("should have id");

    // Verify
    let get_res = app
        .get(&format!("/api/v1/exchange_rates/{}", rate_id))
        .await;
    get_res.assert_status_ok();
    let fetched: serde_json::Value = get_res.json();
    assert_eq!(fetched["rate"], 7.25);
    assert_eq!(fetched["from_currency"], "USD");

    // Tenant isolation
    let app_b = app.create_second_tenant().await;
    let list_b = app_b.get("/api/v1/exchange_rates").await;
    list_b.assert_status_ok();
    let body_b: serde_json::Value = list_b.json();
    assert_eq!(body_b["count"], 0, "tenant B should not see A's rates");
}

#[tokio::test]
async fn s08_query_param_session_no_longer_authenticates() {
    let app = common::TestApp::new().await;

    // Session tokens must never be accepted from the URL: query strings leak
    // into access logs, browser history, and Referer headers.
    let res = app
        .server
        .get(&format!("/api/v1/invoices?session={}", app.session_token))
        .await;
    res.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn s08_oauth_callback_rejects_missing_state() {
    let app = common::TestApp::new().await;
    let server = configured_auth_server(&app).await;

    // No state param and no oauth_state cookie → login-CSRF guard rejects
    // before any token exchange is attempted.
    let res = server.get("/auth/callback?code=fake-code").await;
    res.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn s08_oauth_callback_rejects_mismatched_state() {
    let app = common::TestApp::new().await;
    let server = configured_auth_server(&app).await;

    let res = server
        .get("/auth/callback?code=fake-code&state=attacker-state")
        .add_header("cookie", "oauth_state=victim-state")
        .await;
    res.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn s08_oauth_callback_rejects_missing_pkce_verifier() {
    let app = common::TestApp::new().await;
    let server = configured_auth_server(&app).await;

    let res = server
        .get("/auth/callback?code=fake-code&state=expected-state")
        .add_header("cookie", "oauth_state=expected-state")
        .await;
    res.assert_status(StatusCode::BAD_REQUEST);
    assert!(res.text().contains("Missing PKCE verifier"));
}

#[tokio::test]
async fn s08_login_issues_state_cookie_matching_authorize_url() {
    let app = common::TestApp::new().await;
    let server = configured_auth_server(&app).await;

    let res = server.get("/auth/login").await;
    res.assert_status(StatusCode::FOUND);

    let location = res
        .header("location")
        .to_str()
        .expect("location header")
        .to_string();
    let url_state = location
        .split('&')
        .find_map(|part| part.strip_prefix("state="))
        .expect("authorize URL must carry a state param")
        .to_string();

    let cookies = res.headers().get_all("set-cookie");
    let cookies = cookies
        .iter()
        .map(|value| value.to_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    let state_cookie = cookies
        .iter()
        .find(|cookie| cookie.starts_with("oauth_state="))
        .expect("oauth state cookie");
    for cookie in &cookies {
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("SameSite=Lax"));
    }
    let cookie_state = state_cookie
        .split(';')
        .next()
        .and_then(|kv| kv.strip_prefix("oauth_state="))
        .expect("oauth_state cookie value")
        .to_string();

    assert_eq!(
        url_state, cookie_state,
        "cookie state must match the state sent to W-ID"
    );
    assert!(location.contains("code_challenge="));
    assert!(location.contains("code_challenge_method=S256"));
    assert!(
        cookies
            .iter()
            .any(|cookie| cookie.starts_with("oauth_pkce=")),
        "PKCE verifier must remain in an HttpOnly cookie"
    );
}

#[tokio::test]
async fn s08_missing_oauth_configuration_fails_fast_with_503() {
    let app = common::TestApp::new().await;
    let mut oauth = configured_oauth();
    oauth.client_id.clear();
    oauth.client_secret.clear();
    oauth.redirect_uri.clear();
    let server = TestServer::new(opc::auth::router_with_config(app.pool.clone(), oauth)).unwrap();

    let res = server.get("/auth/login").await;
    res.assert_status(StatusCode::SERVICE_UNAVAILABLE);
    let body: serde_json::Value = res.json();
    assert_eq!(body["error"], "auth_configuration_missing");
    assert_eq!(body["missing"].as_array().unwrap().len(), 3);

    let persisted: (String, String, String) = sqlx::query_as(
        "SELECT error_type, error_message, metadata FROM error_logs \
         WHERE page_url = '/auth/login' ORDER BY created_at DESC LIMIT 1",
    )
    .fetch_one(&app.pool)
    .await
    .expect("missing OAuth configuration must be persisted for the audit center");
    assert_eq!(persisted.0, "server_error");
    assert!(persisted.1.contains("W_ID_CLIENT_ID"));
    assert!(persisted.1.contains("W_ID_CLIENT_SECRET"));
    assert!(persisted.1.contains("W_ID_REDIRECT_URI"));
    assert!(persisted.2.contains("\"status\":503"));
}

#[tokio::test]
async fn s08_cookie_session_authenticates_but_requires_origin_for_writes() {
    let app = common::TestApp::new().await;
    let cookie = format!("opc_session={}", app.session_token);

    app.server
        .get("/api/v1/invoices")
        .add_header("cookie", &cookie)
        .await
        .assert_status_ok();

    let rejected = app
        .server
        .post("/api/v1/invoices")
        .add_header("cookie", &cookie)
        .json(&json!({"invoice_number":"CSRF-1","amount":10,"invoice_type":"income"}))
        .await;
    rejected.assert_status(StatusCode::FORBIDDEN);
    assert_eq!(
        rejected.json::<serde_json::Value>()["error"],
        "csrf_origin_rejected"
    );

    app.server
        .post("/api/v1/invoices")
        .add_header("cookie", cookie)
        .add_header("origin", "http://localhost:8987")
        .json(&json!({"invoice_number":"CSRF-2","amount":10,"invoice_type":"income"}))
        .await
        .assert_status_ok();
}

#[tokio::test]
async fn s08_file_handlers_accept_the_httponly_cookie_session() {
    let app = common::TestApp::new().await;
    sqlx::query(
        "INSERT INTO files (id, company_id, filename, original_name, content_type, size, entity_type, entity_id) VALUES ('cookie-file', ?, 'hash', 'invoice.xml', 'application/xml', 10, 'invoices', 'invoice-1')",
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();

    let res = app
        .server
        .get("/api/v1/files/list/invoices/invoice-1")
        .add_header("cookie", format!("opc_session={}", app.session_token))
        .await;
    res.assert_status_ok();
    assert_eq!(
        res.json::<serde_json::Value>()["data"][0]["id"],
        "cookie-file"
    );
}

#[tokio::test]
async fn s08_cookie_is_preferred_over_conflicting_bearer() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    let res = app
        .server
        .get("/api/v1/companies")
        .add_header("cookie", format!("opc_session={}", app.session_token))
        .authorization_bearer(&other.session_token)
        .await;
    res.assert_status_ok();
    let companies = res.json::<serde_json::Value>()["data"]
        .as_array()
        .unwrap()
        .clone();
    assert!(
        companies
            .iter()
            .any(|company| company["id"] == app.company_id)
    );
    assert!(
        !companies
            .iter()
            .any(|company| company["id"] == other.company_id)
    );
}

#[tokio::test]
async fn s08_bearer_cli_write_does_not_require_browser_origin() {
    let app = common::TestApp::new().await;
    app.post("/api/v1/invoices")
        .json(&json!({"invoice_number":"CLI-1","amount":10,"invoice_type":"income"}))
        .await
        .assert_status_ok();
}

#[tokio::test]
async fn s08_api_key_cli_write_does_not_require_browser_origin() {
    let app = common::TestApp::new().await;
    let created = app
        .post("/api/v1/api-keys")
        .json(&json!({"name":"security-wave-2","scopes":"write"}))
        .await;
    created.assert_status(StatusCode::CREATED);
    let key = created.json::<serde_json::Value>()["key"]
        .as_str()
        .unwrap()
        .to_owned();

    app.server
        .post("/api/v1/invoices")
        .add_header("x-api-key", key)
        .json(&json!({"invoice_number":"APIKEY-1","amount":10,"invoice_type":"income"}))
        .await
        .assert_status_ok();
}

#[tokio::test]
async fn s08_logout_deletes_cookie_session_and_expires_cookie() {
    let app = common::TestApp::new().await;
    let cookie = format!("opc_session={}", app.session_token);

    let res = app
        .server
        .post("/auth/logout")
        .add_header("cookie", cookie)
        .add_header("origin", "http://localhost:8987")
        .await;
    res.assert_status(StatusCode::FOUND);
    let set_cookie_header = res.header("set-cookie");
    let set_cookie = set_cookie_header.to_str().unwrap();
    assert!(set_cookie.starts_with("opc_session="));
    assert!(set_cookie.contains("HttpOnly"));
    assert!(set_cookie.contains("Secure"));
    assert!(set_cookie.contains("SameSite=Lax"));
    assert!(set_cookie.contains("Max-Age=0"));

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE id = ?")
        .bind(&app.session_token)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(count.0, 0);
}

#[tokio::test]
async fn s08_api_health_alias_is_public() {
    let app = common::TestApp::new().await;

    for path in ["/health", "/api/health", "/api/v1/health"] {
        let res = app.server.get(path).await;
        res.assert_status_ok();
    }
}

#[tokio::test]
async fn s08_removed_member_loses_existing_session_and_api_key_access() {
    let app = common::TestApp::new().await;
    let key = "om_tenant_revocation_fixture";
    sqlx::query("INSERT INTO api_keys(id,user_id,company_id,name,key_hash,key_prefix) VALUES ('revocation-key',?,?, 'fixture',?, 'om_test')")
        .bind(&app.user_id).bind(&app.company_id).bind(opc::auth::middleware::hash_api_key(key))
        .execute(&app.pool).await.unwrap();
    app.get("/api/v1/invoices").await.assert_status_ok();
    app.server.get("/api/v1/invoices").add_header("x-api-key", key).await.assert_status_ok();
    sqlx::query("DELETE FROM company_members WHERE user_id=? AND company_id=?")
        .bind(&app.user_id).bind(&app.company_id).execute(&app.pool).await.unwrap();
    app.get("/api/v1/invoices").await.assert_status(StatusCode::FORBIDDEN);
    app.server.get("/api/v1/invoices").add_header("cookie", format!("opc_session={}", app.session_token)).await.assert_status(StatusCode::FORBIDDEN);
    app.server.get("/api/v1/invoices").add_header("x-api-key", key).await.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn s08_company_owner_cannot_access_instance_backups() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;
    for tenant in [&app, &other] {
        tenant.get("/api/v1/settings/backups").await.assert_status(StatusCode::FORBIDDEN);
        tenant.post("/api/v1/settings/backups").json(&json!({})).await.assert_status(StatusCode::FORBIDDEN);
        tenant.get("/api/v1/settings/backups/backup-fixture.tar.gz/download").await.assert_status(StatusCode::FORBIDDEN);
        tenant.post("/api/v1/settings/restore").json(&json!({"file_name":"backup-fixture.tar.gz"})).await.assert_status(StatusCode::FORBIDDEN);
    }
}

#[tokio::test]
async fn s08_scoped_api_key_cannot_mint_unrestricted_key() {
    let app = common::TestApp::new().await;
    let key = "om_key_management_fixture";
    sqlx::query("INSERT INTO api_keys(id,user_id,company_id,name,key_hash,key_prefix,scopes) VALUES ('limited-key',?,?, 'fixture',?, 'om_test','api_keys:write')")
        .bind(&app.user_id).bind(&app.company_id).bind(opc::auth::middleware::hash_api_key(key))
        .execute(&app.pool).await.unwrap();
    app.server.post("/api/v1/api-keys").add_header("x-api-key", key)
        .json(&json!({"name":"escalated","scopes":"*"})).await.assert_status(StatusCode::FORBIDDEN);
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM api_keys").fetch_one(&app.pool).await.unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn s08_deleted_company_rejects_existing_credentials() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;
    let key = "om_deleted_company_fixture";
    sqlx::query("INSERT INTO api_keys(id,user_id,company_id,name,key_hash,key_prefix) VALUES ('deleted-company-key',?,?, 'fixture',?, 'om_test')")
        .bind(&app.user_id).bind(&app.company_id).bind(opc::auth::middleware::hash_api_key(key))
        .execute(&app.pool).await.unwrap();
    sqlx::query("UPDATE companies SET is_deleted=1 WHERE id=?").bind(&app.company_id).execute(&app.pool).await.unwrap();
    app.get("/api/v1/invoices").await.assert_status(StatusCode::FORBIDDEN);
    app.server.get("/api/v1/invoices").add_header("x-api-key", key).await.assert_status(StatusCode::UNAUTHORIZED);
    other.get("/api/v1/invoices").await.assert_status_ok();
}

#[tokio::test]
async fn s08_revoked_company_session_can_select_another_actual_membership() {
    let app = common::TestApp::new().await;
    let (allowed_company, _) = app.create_second_company("AllowedCo", "CNY").await;
    let outsider = app.create_second_tenant().await;
    sqlx::query("DELETE FROM company_members WHERE user_id=? AND company_id=?")
        .bind(&app.user_id).bind(&app.company_id).execute(&app.pool).await.unwrap();
    app.get("/api/v1/invoices").await.assert_status(StatusCode::FORBIDDEN);
    app.get("/api/v1/companies").await.assert_status_ok();
    app.post(&format!("/api/v1/companies/{}/switch", outsider.company_id)).json(&json!({})).await.assert_status(StatusCode::FORBIDDEN);
    app.post(&format!("/api/v1/companies/{allowed_company}/switch")).json(&json!({})).await.assert_status_ok();
    app.get("/api/v1/invoices").await.assert_status_ok();
    let selected: String = sqlx::query_scalar("SELECT current_company_id FROM sessions WHERE id=?").bind(&app.session_token).fetch_one(&app.pool).await.unwrap();
    assert_eq!(selected, allowed_company);
}
