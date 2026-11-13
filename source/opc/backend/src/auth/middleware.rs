use axum::{
    extract::{FromRequestParts, State},
    http::{HeaderMap, Method, StatusCode, request::Parts},
    response::Response,
};
use sqlx::SqlitePool;

pub const SESSION_COOKIE: &str = "opc_session";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CredentialSource {
    Cookie,
    Bearer,
    ApiKey,
}

/// Authenticated user extracted from request
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub session_id: String,
    pub company_id: String,
    pub api_key_id: Option<String>,
    pub api_key_scopes: Vec<String>,
    /// Raw W-ID access token from the session, forwarded to DaaS (Phase 1
    /// document fusion). `None` for API-key callers, which have no live W-ID
    /// token and must reach DaaS via service mode instead.
    pub wid_access_token: Option<String>,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let pool = parts
            .extensions
            .get::<SqlitePool>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // Browser cookies are authoritative when present. This prevents a
        // confused-deputy request from silently switching identity through an
        // injected Authorization/API-key header. Header credentials remain
        // available to CLI and integrations when no browser cookie is sent.
        if let Some(session_id) = extract_cookie_session_id(&parts.headers) {
            return validate_session(
                pool,
                session_id,
                company_selection_request(
                    &parts.method,
                    parts
                        .extensions
                        .get::<axum::extract::OriginalUri>()
                        .map_or(parts.uri.path(), |uri| uri.0.path()),
                ),
            )
            .await;
        }

        if let Some(api_key) = extract_api_key(&parts.headers)
            && let Some(auth) = validate_api_key(pool, &api_key).await
        {
            return Ok(auth);
        }

        // Fall back to session auth
        let session_id = extract_session_id(&parts.headers).ok_or(StatusCode::UNAUTHORIZED)?;

        validate_session(
            pool,
            session_id,
            company_selection_request(
                &parts.method,
                parts
                    .extensions
                    .get::<axum::extract::OriginalUri>()
                    .map_or(parts.uri.path(), |uri| uri.0.path()),
            ),
        )
        .await
    }
}

/// Extract API key from X-API-Key header
fn extract_api_key(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

fn company_selection_request(method: &Method, path: &str) -> bool {
    let path = path
        .strip_prefix("/api/v1")
        .or_else(|| path.strip_prefix("/api"))
        .unwrap_or(path);
    if method == Method::GET && path == "/companies" {
        return true;
    }
    method == Method::POST
        && path
            .strip_prefix("/companies/")
            .and_then(|p| p.strip_suffix("/switch"))
            .is_some_and(|id| !id.is_empty() && !id.contains('/'))
}

async fn validate_session(
    pool: &SqlitePool,
    session_id: String,
    allow_company_selection: bool,
) -> Result<AuthUser, StatusCode> {
    let session_data: Option<(String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT user_id, current_company_id, access_token FROM sessions WHERE id = ? AND (expires_at IS NULL OR expires_at > datetime('now'))"
    )
    .bind(&session_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (uid, company_id, access_token) = session_data.ok_or(StatusCode::UNAUTHORIZED)?;
    let mut company_id = company_id.unwrap_or_default();
    // An existing session is not proof of continued tenant membership.
    // Empty scope remains available for first-login company onboarding.
    if !company_id.is_empty() && !active_company_member(pool, &uid, &company_id).await? {
        if !allow_company_selection {
            return Err(StatusCode::FORBIDDEN);
        }
        // Company listing uses user identity; switching separately authorizes
        // the target company. Never carry the revoked scope into either route.
        company_id.clear();
    }
    Ok(AuthUser {
        user_id: uid,
        session_id,
        company_id,
        api_key_id: None,
        api_key_scopes: Vec::new(),
        wid_access_token: access_token,
    })
}

async fn active_company_member(
    pool: &SqlitePool,
    user_id: &str,
    company_id: &str,
) -> Result<bool, StatusCode> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM company_members cm JOIN companies c ON c.id=cm.company_id WHERE cm.user_id=? AND cm.company_id=? AND COALESCE(c.is_deleted,0)=0)")
        .bind(user_id).bind(company_id).fetch_one(pool).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Validate API key by hashing and looking up in database
async fn validate_api_key(pool: &SqlitePool, raw_key: &str) -> Option<AuthUser> {
    let key_hash = w_crypto::sha256_hex_str(raw_key);

    let result: Option<(String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, user_id, company_id, scopes FROM api_keys WHERE key_hash = ? AND is_active = 1 AND (expires_at IS NULL OR expires_at > datetime('now'))"
    )
    .bind(&key_hash)
    .fetch_optional(pool)
    .await
    .ok()?;

    let (key_id, user_id, company_id, scopes) = result?;

    if !active_company_member(pool, &user_id, &company_id)
        .await
        .ok()?
    {
        return None;
    }

    // Update last_used_at (fire and forget)
    let _ = sqlx::query("UPDATE api_keys SET last_used_at = datetime('now') WHERE id = ?")
        .bind(&key_id)
        .execute(pool)
        .await;

    Some(AuthUser {
        user_id,
        session_id: format!("apikey:{}", key_id),
        company_id,
        api_key_id: Some(key_id),
        api_key_scopes: parse_api_key_scopes(scopes.as_deref().unwrap_or("*")),
        wid_access_token: None,
    })
}

pub fn parse_api_key_scopes(scopes: &str) -> Vec<String> {
    scopes
        .split(|ch: char| ch == ',' || ch.is_ascii_whitespace())
        .map(|scope| scope.trim().to_ascii_lowercase())
        .filter(|scope| !scope.is_empty())
        .collect()
}

fn is_read_method(method: &Method) -> bool {
    matches!(method, &Method::GET | &Method::HEAD | &Method::OPTIONS)
}

fn request_scope_resource(path: &str) -> Option<String> {
    let trimmed = path.trim_start_matches('/');
    let api_path = trimmed
        .strip_prefix("api/v1/")
        .or_else(|| trimmed.strip_prefix("api/"))
        .unwrap_or(trimmed);

    let mut segments = api_path.split('/').filter(|segment| !segment.is_empty());
    let first = segments.next()?;

    if matches!(first, "agent" | "mcp") {
        return None;
    }

    let resource = match first {
        "api-keys" => "api_keys".to_string(),
        "company-info" => "company_info".to_string(),
        "bank" => segments.next().unwrap_or("bank").replace('-', "_"),
        other => other.replace('-', "_"),
    };

    if resource.is_empty() {
        None
    } else {
        Some(resource)
    }
}

fn api_key_scopes_allow_http(scopes: &[String], method: &Method, path: &str) -> bool {
    if scopes.iter().any(|scope| scope == "*") {
        return true;
    }

    let is_read = is_read_method(method);
    if is_read && scopes.iter().any(|scope| scope == "read") {
        return true;
    }
    if !is_read && scopes.iter().any(|scope| scope == "write") {
        return true;
    }

    let Some(resource) = request_scope_resource(path) else {
        return true;
    };

    scopes.iter().any(|scope| {
        scope == &resource
            || (is_read && scope == &format!("{resource}:read"))
            || (!is_read && scope == &format!("{resource}:write"))
    })
}

fn read_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get_all(axum::http::header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .find_map(|pair| {
            let (key, value) = pair.trim().split_once('=')?;
            (key == name && !value.is_empty()).then(|| value.to_owned())
        })
}

pub(crate) fn extract_cookie_session_id(headers: &HeaderMap) -> Option<String> {
    read_cookie(headers, SESSION_COOKIE)
}

/// Session credentials are accepted from the HttpOnly browser cookie first,
/// then explicit headers for CLI compatibility. The former `?session=`
/// query-param channel was removed: URLs leak into access logs, browser
/// history, and Referer headers, so they must never carry session tokens.
pub(crate) fn extract_session_id(headers: &HeaderMap) -> Option<String> {
    if let Some(session) = extract_cookie_session_id(headers) {
        return Some(session);
    }

    // Try Authorization: Bearer <session_id>
    if let Some(auth) = headers.get("authorization")
        && let Ok(auth_str) = auth.to_str()
        && let Some(token) = auth_str.strip_prefix("Bearer ")
    {
        return Some(token.to_string());
    }

    // Try X-Session-Id header
    if let Some(session) = headers.get("x-session-id")
        && let Ok(s) = session.to_str()
    {
        return Some(s.to_string());
    }

    None
}

fn credential_source(headers: &HeaderMap) -> Option<CredentialSource> {
    if extract_cookie_session_id(headers).is_some() {
        Some(CredentialSource::Cookie)
    } else if extract_api_key(headers).is_some() {
        Some(CredentialSource::ApiKey)
    } else if extract_session_id(headers).is_some() {
        Some(CredentialSource::Bearer)
    } else {
        None
    }
}

fn normalized_origin(value: &str) -> Option<String> {
    let url = reqwest::Url::parse(value.trim()).ok()?;
    if url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
        return None;
    }
    Some(url.origin().ascii_serialization())
}

fn allowed_browser_origins() -> Vec<String> {
    let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8987".to_owned());
    std::iter::once(app_url)
        .chain(
            std::env::var("ALLOWED_ORIGINS")
                .unwrap_or_default()
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned),
        )
        .filter_map(|value| normalized_origin(&value))
        .collect()
}

pub(crate) fn cookie_write_origin_allowed(method: &Method, headers: &HeaderMap) -> bool {
    if is_read_method(method) || credential_source(headers) != Some(CredentialSource::Cookie) {
        return true;
    }

    let Some(origin) = headers
        .get(axum::http::header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .and_then(normalized_origin)
    else {
        return false;
    };

    allowed_browser_origins()
        .iter()
        .any(|allowed| allowed == &origin)
}

/// Middleware to inject pool into request extensions
pub async fn inject_pool(
    State(pool): State<SqlitePool>,
    mut request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    request.extensions_mut().insert(pool);
    next.run(request).await
}

/// Middleware to require authentication for all routes
/// Supports: HttpOnly cookie sessions (preferred), Bearer/X-Session-Id CLI
/// sessions, and API Key auth (X-API-Key). Cookie-authenticated writes require
/// an exact allowed Origin; explicit CLI credentials are not subject to browser
/// CSRF checks.
pub async fn require_auth(
    State(pool): State<SqlitePool>,
    mut request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Response {
    // Public within the API prefix: GET /health (mirrors the root /health so
    // monitors probing /api/health get a real probe instead of 404). This
    // router is nested under /api and /api/v1, so the path here is stripped.
    if request.method() == Method::GET && request.uri().path() == "/health" {
        request.extensions_mut().insert(pool);
        return next.run(request).await;
    }

    let headers = request.headers().clone();

    // 1. Browser cookie session is authoritative when present.
    if let Some(session_id) = extract_cookie_session_id(&headers) {
        if !cookie_write_origin_allowed(request.method(), &headers) {
            return Response::builder()
                .status(StatusCode::FORBIDDEN)
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .body(axum::body::Body::from(
                    r#"{"error":"csrf_origin_rejected","message":"Cookie-authenticated writes require an allowed Origin"}"#,
                ))
                .unwrap();
        }

        return match validate_session(
            &pool,
            session_id,
            company_selection_request(
                request.method(),
                request
                    .extensions()
                    .get::<axum::extract::OriginalUri>()
                    .map_or(request.uri().path(), |uri| uri.0.path()),
            ),
        )
        .await
        {
            Ok(auth_user) => {
                request.extensions_mut().insert(pool);
                request.extensions_mut().insert(auth_user);
                next.run(request).await
            }
            Err(status) => Response::builder()
                .status(status)
                .body(axum::body::Body::from("Unauthorized: Invalid session"))
                .unwrap(),
        };
    }

    // 2. Try API Key auth (X-API-Key header)
    if let Some(api_key) = extract_api_key(&headers)
        && let Some(auth_user) = validate_api_key(&pool, &api_key).await
    {
        if !api_key_scopes_allow_http(
            &auth_user.api_key_scopes,
            request.method(),
            request.uri().path(),
        ) {
            return Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(axum::body::Body::from("Forbidden: API key scope denied"))
                .unwrap();
        }
        request.extensions_mut().insert(pool);
        request.extensions_mut().insert(auth_user);
        return next.run(request).await;
    }
    // Invalid API key → fall through to session auth

    // 3. Try explicit session auth for CLI compatibility.
    let session_id = match extract_session_id(&headers) {
        Some(id) => id,
        None => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(axum::body::Body::from(
                    "Unauthorized: No session or API key provided",
                ))
                .unwrap();
        }
    };

    match validate_session(
        &pool,
        session_id,
        company_selection_request(
            request.method(),
            request
                .extensions()
                .get::<axum::extract::OriginalUri>()
                .map_or(request.uri().path(), |uri| uri.0.path()),
        ),
    )
    .await
    {
        Ok(auth_user) => {
            request.extensions_mut().insert(pool);
            request.extensions_mut().insert(auth_user);
            next.run(request).await
        }
        Err(status) => Response::builder()
            .status(status)
            .body(axum::body::Body::from("Unauthorized: Invalid session"))
            .unwrap(),
    }
}

/// Generate a new API key string in format: om_xxxxxxxxxxxx (32 random hex chars)
pub fn generate_api_key() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    let random_bytes: [u8; 24] = rng.random();
    format!("om_{}", hex::encode(random_bytes))
}

/// Hash an API key for storage
pub fn hash_api_key(key: &str) -> String {
    w_crypto::sha256_hex_str(key)
}
