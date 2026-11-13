use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::env;

pub mod middleware;

pub use middleware::AuthUser;

const DEFAULT_W_ID_SSO_URL: &str = "https://auth.myfastools.com";

#[derive(Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub base_url: String,
    pub authorize_url: String,
    pub token_url: String,
    pub userinfo_url: String,
    pub logout_url: String,
    pub redirect_uri: String,
}

impl OAuthConfig {
    pub fn from_env() -> Self {
        let base_url = w_id_sso_url();

        Self {
            client_id: required_env_value("W_ID_CLIENT_ID"),
            client_secret: required_env_value("W_ID_CLIENT_SECRET"),
            authorize_url: w_id_endpoint_url(&base_url, "/oauth/authorize"),
            token_url: w_id_endpoint_url(&base_url, "/oauth/token"),
            userinfo_url: w_id_endpoint_url(&base_url, "/oauth/userinfo"),
            logout_url: w_id_endpoint_url(&base_url, "/oauth/logout"),
            base_url,
            redirect_uri: required_env_value("W_ID_REDIRECT_URI"),
        }
    }

    fn missing_required_fields(&self) -> Vec<&'static str> {
        [
            ("W_ID_CLIENT_ID", self.client_id.as_str()),
            ("W_ID_CLIENT_SECRET", self.client_secret.as_str()),
            ("W_ID_REDIRECT_URI", self.redirect_uri.as_str()),
        ]
        .into_iter()
        .filter_map(|(name, value)| value.trim().is_empty().then_some(name))
        .collect()
    }
}

fn required_env_value(name: &str) -> String {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .unwrap_or_default()
}

fn env_or_default(name: &str, default_value: &str) -> String {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default_value.to_owned())
}

fn w_id_sso_url() -> String {
    env_or_default("W_ID_SSO_URL", DEFAULT_W_ID_SSO_URL)
        .trim_end_matches('/')
        .to_owned()
}

fn w_id_endpoint_url(base_url: &str, path: &str) -> String {
    format!("{}{}", base_url.trim_end_matches('/'), path)
}

#[derive(Clone)]
pub struct AuthState {
    pub pool: SqlitePool,
    pub oauth: OAuthConfig,
}

impl AuthState {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            oauth: OAuthConfig::from_env(),
        }
    }
}

pub fn router_with_root(pool: SqlitePool) -> Router {
    router_with_config(pool.clone(), AuthState::new(pool).oauth)
}

#[doc(hidden)]
pub fn router_with_config(pool: SqlitePool, oauth: OAuthConfig) -> Router {
    let state = AuthState { pool, oauth };

    Router::new()
        .route("/auth/login", get(login))
        .route("/auth/callback", get(callback))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(get_current_user))
        .with_state(state)
}

// Handle OAuth callback at root path (W-ID SSO redirects here)
async fn root_callback() -> Response {
    // Redirect to frontend
    let app_url = env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8987".to_string());
    found_redirect(&app_url, HeaderMap::new())
}

/// Cookie carrying the OAuth CSRF state between /auth/login and /auth/callback.
/// Scoped to Path=/auth so it is only sent on the auth handshake routes.
const OAUTH_STATE_COOKIE: &str = "oauth_state";
const OAUTH_PKCE_COOKIE: &str = "oauth_pkce";
const SESSION_MAX_AGE_SECONDS: u64 = 7 * 24 * 60 * 60;

fn private_cookie(name: &str, value: &str, path: &str, max_age: u64) -> String {
    format!("{name}={value}; Path={path}; HttpOnly; Secure; SameSite=Lax; Max-Age={max_age}")
}

fn clear_private_cookie(name: &str, path: &str) -> String {
    format!("{name}=; Path={path}; HttpOnly; Secure; SameSite=Lax; Max-Age=0")
}

fn read_cookie(headers: &axum::http::HeaderMap, name: &str) -> Option<String> {
    let cookie_header = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    cookie_header.split(';').find_map(|pair| {
        let (key, value) = pair.trim().split_once('=')?;
        (key == name).then(|| value.to_string())
    })
}

fn found_redirect(location: &str, mut headers: HeaderMap) -> Response {
    headers.insert(
        header::LOCATION,
        HeaderValue::from_str(location).expect("generated redirect URL must be a valid header"),
    );
    (StatusCode::FOUND, headers).into_response()
}

fn complete_login_response(app_url: &str, session_id: &str) -> Response {
    let mut response_headers = HeaderMap::new();
    for cookie in [
        private_cookie(
            middleware::SESSION_COOKIE,
            session_id,
            "/",
            SESSION_MAX_AGE_SECONDS,
        ),
        clear_private_cookie(OAUTH_STATE_COOKIE, "/auth"),
        clear_private_cookie(OAUTH_PKCE_COOKIE, "/auth"),
    ] {
        response_headers.append(
            header::SET_COOKIE,
            HeaderValue::from_str(&cookie).expect("session cookie must be a valid header"),
        );
    }
    found_redirect(app_url.trim_end_matches('/'), response_headers)
}

#[derive(Serialize)]
struct AuthConfigurationError {
    error: &'static str,
    message: &'static str,
    missing: Vec<&'static str>,
}

async fn configuration_error(
    pool: &SqlitePool,
    oauth: &OAuthConfig,
    request_path: &str,
) -> Option<Response> {
    let missing = oauth.missing_required_fields();
    if missing.is_empty() {
        return None;
    }

    tracing::error!(
        event = "auth_configuration_missing",
        missing_fields = ?missing,
        "W-ID OAuth login is unavailable because required configuration is missing"
    );

    // Auth routes sit outside the authenticated API router, so the generic
    // 5xx logging middleware cannot observe this response. Persist the same
    // structured failure explicitly for the audit center while keeping
    // missing field names (never values) as the only configuration detail.
    let message = format!(
        "W-ID OAuth is not configured; missing: {}",
        missing.join(", ")
    );
    crate::error_log_mw::record_server_error(
        pool,
        None,
        None,
        "GET",
        request_path,
        StatusCode::SERVICE_UNAVAILABLE.as_u16(),
        &message,
    )
    .await;

    Some(
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(AuthConfigurationError {
                error: "auth_configuration_missing",
                message: "W-ID OAuth is not configured",
                missing,
            }),
        )
            .into_response(),
    )
}

fn new_pkce_pair() -> (String, String) {
    let mut random = [0_u8; 32];
    rand::rng().fill_bytes(&mut random);
    let verifier = URL_SAFE_NO_PAD.encode(random);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    (verifier, challenge)
}

// Redirect to W-ID SSO login. Random state and the PKCE verifier are stored in
// short-lived HttpOnly cookies and consumed by the callback.
async fn login(State(state): State<AuthState>) -> Response {
    let oauth = &state.oauth;
    if let Some(error) = configuration_error(&state.pool, oauth, "/auth/login").await {
        return error;
    }

    let csrf_state = uuid::Uuid::new_v4().to_string();
    let (pkce_verifier, pkce_challenge) = new_pkce_pair();
    let auth_url = format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope=openid%20profile%20email&state={}&code_challenge={}&code_challenge_method=S256",
        oauth.authorize_url,
        oauth.client_id,
        urlencoding::encode(&oauth.redirect_uri),
        csrf_state,
        pkce_challenge,
    );
    let mut headers = HeaderMap::new();
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&private_cookie(
            OAUTH_STATE_COOKIE,
            &csrf_state,
            "/auth",
            600,
        ))
        .expect("state cookie must be a valid header"),
    );
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&private_cookie(
            OAUTH_PKCE_COOKIE,
            &pkce_verifier,
            "/auth",
            600,
        ))
        .expect("PKCE cookie must be a valid header"),
    );
    found_redirect(&auth_url, headers)
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: Option<String>,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    #[allow(dead_code)]
    token_type: Option<String>,
    #[allow(dead_code)]
    expires_in: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct UserInfoResponse {
    sub: String,
    email: String,
    name: Option<String>,
    picture: Option<String>,
    login: Option<String>,
    #[allow(dead_code)]
    role: Option<String>,
}

// Handle OAuth callback
async fn callback(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
    Query(query): Query<CallbackQuery>,
) -> Result<Response, (StatusCode, String)> {
    let oauth = &state.oauth;
    if let Some(error) = configuration_error(&state.pool, oauth, "/auth/callback").await {
        return Ok(error);
    }

    // Verify the OAuth state against the value issued at /auth/login. Reject
    // before any token exchange: a missing or mismatched state is a login-CSRF
    // attempt (or an expired/replayed handshake).
    let expected_state = read_cookie(&headers, OAUTH_STATE_COOKIE);
    let state_ok = matches!(
        (expected_state.as_deref(), query.state.as_deref()),
        (Some(expected), Some(provided)) if !expected.is_empty() && expected == provided
    );
    if !state_ok {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid OAuth state: restart login at /auth/login".to_string(),
        ));
    }
    let pkce_verifier = read_cookie(&headers, OAUTH_PKCE_COOKIE).ok_or((
        StatusCode::BAD_REQUEST,
        "Missing PKCE verifier: restart login at /auth/login".to_string(),
    ))?;

    let client = reqwest::Client::new();

    // Exchange code for token
    let token_res = client
        .post(&oauth.token_url)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &query.code),
            ("redirect_uri", &oauth.redirect_uri),
            ("client_id", &oauth.client_id),
            ("client_secret", &oauth.client_secret),
            ("code_verifier", &pkce_verifier),
        ])
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                format!("Token request failed: {}", e),
            )
        })?;

    let token: TokenResponse = token_res.json().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("Invalid token response: {}", e),
        )
    })?;

    // Get user info
    let userinfo_res = client
        .get(&oauth.userinfo_url)
        .bearer_auth(&token.access_token)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                format!("UserInfo request failed: {}", e),
            )
        })?;

    let userinfo: UserInfoResponse = userinfo_res.json().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("Invalid userinfo response: {}", e),
        )
    })?;

    // Upsert user in database
    sqlx::query(
        r#"
        INSERT INTO users (id, email, name, avatar_url, github_login, updated_at)
        VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
        ON CONFLICT(id) DO UPDATE SET
            email = excluded.email,
            name = excluded.name,
            avatar_url = excluded.avatar_url,
            github_login = excluded.github_login,
            updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(&userinfo.sub)
    .bind(&userinfo.email)
    .bind(&userinfo.name)
    .bind(&userinfo.picture)
    .bind(&userinfo.login)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    // Ensure user has a default company (auto-create if needed)
    let company_id =
        crate::api::companies::ensure_default_company(&state.pool, &userinfo.sub, &userinfo.email)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Company error: {}", e),
                )
            })?;

    // Store session with current company and expiry
    let session_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO sessions (id, user_id, access_token, current_company_id, expires_at) VALUES (?, ?, ?, ?, datetime('now', '+7 days'))"
    )
    .bind(&session_id)
    .bind(&userinfo.sub)
    .bind(&token.access_token)
    .bind(&company_id)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Session error: {}", e)))?;

    // The browser session is handed off exclusively through an HttpOnly cookie;
    // no token is exposed to JavaScript, URL history, logs, or Referer headers.
    let app_url = env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8987".to_string());
    Ok(complete_login_response(&app_url, &session_id))
}

async fn logout(
    State(state): State<AuthState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    if !middleware::cookie_write_origin_allowed(&Method::POST, &headers) {
        return Err(StatusCode::FORBIDDEN);
    }

    // A2: Delete session from DB
    if let Some(session_id) = extract_session_from_request(&headers) {
        let _ = sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(&session_id)
            .execute(&state.pool)
            .await;
    }

    let redirect_uri = env::var("APP_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());

    // client_id scopes W-ID's post_logout_redirect_uri check to this client's
    // registered origins (W-ID >= 1.3.8 matches by origin, not exact URI).
    let location = format!(
        "{}?client_id={}&post_logout_redirect_uri={}",
        state.oauth.logout_url,
        urlencoding::encode(&state.oauth.client_id),
        urlencoding::encode(&redirect_uri)
    );
    let mut response_headers = HeaderMap::new();
    response_headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&clear_private_cookie(middleware::SESSION_COOKIE, "/"))
            .expect("cleared session cookie must be a valid header"),
    );
    Ok(found_redirect(&location, response_headers))
}

#[derive(Serialize)]
struct CurrentUserResponse {
    id: String,
    email: String,
    name: Option<String>,
    avatar_url: Option<String>,
}

async fn get_current_user(
    State(state): State<AuthState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<CurrentUserResponse>, StatusCode> {
    // Extract session ID from Authorization or X-Session-Id header
    let session_id = extract_session_from_request(&headers).ok_or(StatusCode::UNAUTHORIZED)?;

    // A3: Validate session and check expiry
    let session_row: Option<(String,)> = sqlx::query_as(
        "SELECT user_id FROM sessions WHERE id = ? AND (expires_at IS NULL OR expires_at > datetime('now'))"
    )
    .bind(&session_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_id = session_row.ok_or(StatusCode::UNAUTHORIZED)?.0;

    // Get user info
    let user_row: Option<(String, String, Option<String>, Option<String>)> =
        sqlx::query_as("SELECT id, email, name, avatar_url FROM users WHERE id = ?")
            .bind(&user_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match user_row {
        Some((id, email, name, avatar_url)) => Ok(Json(CurrentUserResponse {
            id,
            email,
            name,
            avatar_url,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// Delegate to the single source of truth in middleware
fn extract_session_from_request(headers: &axum::http::HeaderMap) -> Option<String> {
    middleware::extract_session_id(headers)
}

#[cfg(test)]
mod tests {
    use super::{complete_login_response, new_pkce_pair, w_id_endpoint_url};
    use axum::http::{StatusCode, header};

    #[test]
    fn w_id_endpoint_url_derives_oauth_paths_from_base_url() {
        assert_eq!(
            w_id_endpoint_url("https://auth.myfastools.com", "/oauth/authorize"),
            "https://auth.myfastools.com/oauth/authorize"
        );
        assert_eq!(
            w_id_endpoint_url("https://auth.myfastools.com/", "/oauth/logout"),
            "https://auth.myfastools.com/oauth/logout"
        );
    }

    #[test]
    fn pkce_pair_is_url_safe_and_matches_s256() {
        use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
        use sha2::{Digest, Sha256};

        let (verifier, challenge) = new_pkce_pair();
        assert!(verifier.len() >= 43);
        assert_eq!(
            challenge,
            URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
        );
    }

    #[test]
    fn completed_login_is_302_with_secure_httponly_cookie_and_no_url_token() {
        let response = complete_login_response("https://erp.example.test/", "secret-session");
        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "https://erp.example.test"
        );
        assert!(
            !response.headers()[header::LOCATION]
                .to_str()
                .unwrap()
                .contains("secret-session")
        );
        let cookies = response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|value| value.to_str().unwrap())
            .collect::<Vec<_>>();
        let session = cookies
            .iter()
            .find(|cookie| cookie.starts_with("opc_session=secret-session"))
            .expect("session cookie");
        assert!(session.contains("Path=/"));
        assert!(session.contains("HttpOnly"));
        assert!(session.contains("Secure"));
        assert!(session.contains("SameSite=Lax"));
        assert!(session.contains("Max-Age=604800"));
    }
}
