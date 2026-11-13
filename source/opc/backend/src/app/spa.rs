//! OPC SPA embedded-static fallback.
//!
//! Delegates to the shared `w-web-kit` `spa` module — the faithful superset of
//! the per-project `spa.rs` copies. The only things that stay local are the
//! `#[derive(RustEmbed)]` struct (its `#[folder]` is manifest-relative) and the
//! per-consumer [`SpaConfig`] (guard list + `index.html` shell + strict-asset
//! 404 + Cache-Control hardening).

use std::sync::LazyLock;

use axum::{http::Uri, response::Response};
use rust_embed::RustEmbed;
use w_web_kit::{SpaConfig, spa_response};

/// Embedded frontend assets built by Vite into `../frontend/dist`.
/// Path is relative to this crate's manifest directory.
#[derive(RustEmbed)]
#[folder = "../frontend/dist"]
struct SpaAssets;

/// Per-consumer SPA config. OPC guards the real server-route prefixes
/// (`api` and `health` exact+prefix; `auth/`; and the path-traversal-looking
/// `static/ etc/ proc/ sys/ var/ tmp/ root/ home/ opt/ dev/` prefixes) —
/// so `/health` and `/health/...` 404 while the CLIENT route
/// `/health-records` still falls through to the SPA (a bare guard entry
/// matches the exact segment or `<entry>/...`, never a same-prefixed sibling
/// like `health-records`). Uses `index.html` as the shell, with strict-asset
/// 404 (an extensioned miss must NOT be answered with HTML) and the
/// immutable-`assets/` / no-cache-shell `Cache-Control` policy.
static SPA: LazyLock<SpaConfig> = LazyLock::new(|| {
    SpaConfig::with_guards([
        "api", "auth/", "health", "static/", "etc/", "proc/", "sys/", "var/", "tmp/", "root/",
        "home/", "opt/", "dev/",
    ])
    .strict_assets(true)
    .cache_control(true)
});

/// Embedded-static SPA fallback: guarded server routes 404; real assets are
/// served mime-guessed (with `Cache-Control`); an extensioned miss 404s; unknown
/// client-side routes fall back to `index.html`. Wired as the axum router
/// `.fallback`, so it only handles requests not matched by an explicit
/// API/auth/health route.
pub async fn spa_handler(uri: Uri) -> Response {
    spa_response::<SpaAssets>(&uri, &SPA)
}

/// Retained for the `cache_control_for_branches` regression test — documents the
/// project's original 3-branch cache policy. The live handler now uses
/// `w-web-kit`'s Cache-Control policy (immutable `assets/`, no-cache otherwise).
#[allow(dead_code)]
fn cache_control_for(lookup: &str) -> &'static str {
    if lookup == "index.html" {
        "no-cache"
    } else if lookup.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=3600"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Uri, header};

    async fn status_and_ct(uri_str: &str) -> (u16, String) {
        let uri: Uri = uri_str.parse().unwrap();
        let response = spa_handler(uri).await;
        let status = response.status().as_u16();
        let ct = response
            .headers()
            .get(header::CONTENT_TYPE)
            .map(|v| v.to_str().unwrap_or("").to_string())
            .unwrap_or_default();
        (status, ct)
    }

    async fn cache_header(uri_str: &str) -> String {
        let uri: Uri = uri_str.parse().unwrap();
        let response = spa_handler(uri).await;
        response
            .headers()
            .get(header::CACHE_CONTROL)
            .map(|v| v.to_str().unwrap_or("").to_string())
            .unwrap_or_default()
    }

    #[tokio::test]
    async fn root_returns_html() {
        let (status, ct) = status_and_ct("/").await;
        assert_eq!(status, 200, "root should return 200");
        assert!(
            ct.contains("text/html"),
            "root content-type should be text/html, got: {ct}"
        );
    }

    #[tokio::test]
    async fn api_path_returns_404() {
        let (status, _ct) = status_and_ct("/api/anything").await;
        assert_eq!(status, 404, "api path should return 404 from spa_handler");
    }

    #[tokio::test]
    async fn api_root_returns_404() {
        let (status, _ct) = status_and_ct("/api").await;
        assert_eq!(status, 404, "api root should return 404 from spa_handler");
    }

    #[tokio::test]
    async fn auth_path_returns_404() {
        let (status, _ct) = status_and_ct("/auth/login").await;
        assert_eq!(status, 404, "auth path should return 404 from spa_handler");
    }

    #[tokio::test]
    async fn health_path_returns_404() {
        let (status, _ct) = status_and_ct("/health").await;
        assert_eq!(
            status, 404,
            "health path should return 404 from spa_handler"
        );
    }

    #[tokio::test]
    async fn unknown_spa_route_returns_index_html() {
        let (status, ct) = status_and_ct("/some/client/route").await;
        assert_eq!(
            status, 200,
            "unknown SPA route should fall back to index.html"
        );
        assert!(
            ct.contains("text/html"),
            "SPA fallback should be text/html, got: {ct}"
        );
    }

    #[tokio::test]
    async fn normalized_files_path_traversal_target_returns_404() {
        let (status, _ct) = status_and_ct("/etc/passwd").await;
        assert_eq!(
            status, 404,
            "normalized traversal targets should not return SPA HTML"
        );
    }

    #[test]
    fn cache_control_for_branches() {
        assert_eq!(cache_control_for("index.html"), "no-cache");
        assert_eq!(
            cache_control_for("assets/index-abc123.js"),
            "public, max-age=31536000, immutable"
        );
        assert_eq!(cache_control_for("robots.txt"), "public, max-age=3600");
    }

    #[tokio::test]
    async fn index_html_is_served_no_cache() {
        // index.html must never be cached, so a redeploy's new bundle hashes are
        // picked up immediately (the whole point of the asset-skew fix).
        assert!(
            cache_header("/").await.contains("no-cache"),
            "index.html (served for /) must be no-cache"
        );
    }

    #[tokio::test]
    async fn client_route_prefixed_health_still_falls_back_to_index() {
        // The /health guard must match the route exactly — a future client route
        // like /health-records must still get the SPA, not a bare 404.
        let (status, ct) = status_and_ct("/health-records").await;
        assert_eq!(
            status, 200,
            "a client route merely prefixed 'health' must fall back to index.html"
        );
        assert!(
            ct.contains("text/html"),
            "should be the SPA HTML, got: {ct}"
        );
    }

    #[tokio::test]
    async fn missing_hashed_asset_returns_404_not_html() {
        // A request for a hashed bundle that is NOT embedded (e.g. a stale
        // index.html across a redeploy referencing an old hash) must 404 — it
        // must NOT fall back to index.html, or the browser throws a module
        // MIME error ("expected JavaScript, got text/html") and the app breaks.
        let (status, ct) = status_and_ct("/assets/index-doesnotexist123.js").await;
        assert_eq!(
            status, 404,
            "missing JS asset must 404, not fall back to index.html"
        );
        assert!(
            !ct.contains("text/html"),
            "missing asset must not be served as HTML, got: {ct}"
        );
    }
}
