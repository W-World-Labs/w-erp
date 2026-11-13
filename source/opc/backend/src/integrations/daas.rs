//! DaaS document-fusion integration (Phase 1).
//!
//! ERP stays the system-of-record. This module mirrors uploaded files into the
//! DaaS (Data Asset Platform) as governed document assets and records the DaaS
//! reference back on the `files` row. It is the ONLY place ERP talks to DaaS.
//!
//! Design invariants:
//!   * **Additive & failure-isolated** — the user's upload is committed and the
//!     200 returned *before* the mirror runs. A DaaS outage never affects ERP.
//!   * **Local FS stays canonical** for serving; DaaS holds the governed copy.
//!   * **Token forwarding** — calls carry the logged-in user's W-ID access token
//!     so DaaS scopes data per real user + workspace. Background work (admin
//!     backfill) falls back to the configured service token.
//!   * **Optional** — when `DAAS_ENABLED` is unset/false, [`DaasConfig::from_env`]
//!     returns `None` and ERP behaves exactly as before (rows stay `pending`).

use sqlx::SqlitePool;

/// Document extensions DaaS accepts — mirror of the allowlist in the DaaS
/// `upload_document` handler. Anything else is recorded `skipped` (not failed),
/// so backfill never churns on it. Image receipts (png/jpg) take a different
/// DaaS path that is out of scope for Phase 1.
const SUPPORTED_EXTS: &[&str] = &[
    "pdf", "md", "txt", "docx", "docm", "pptx", "pptm", "xlsx", "xlsm", "json", "html", "htm",
    "epub", "tex", "latex",
];

/// Runtime configuration for the DaaS integration. Built from env once at
/// startup; absence of `DAAS_ENABLED`/`DAAS_BASE_URL` disables the feature.
#[derive(Clone, Debug)]
pub struct DaasConfig {
    pub base_url: String,
    /// Shared service token used for background work without a live user token.
    pub service_token: Option<String>,
    /// Optional `knowledge_engine` to set on workspaces ERP creates.
    pub default_knowledge_engine: Option<String>,
}

impl DaasConfig {
    /// Read config from env. Returns `None` (feature disabled) unless
    /// `DAAS_ENABLED` is truthy AND `DAAS_BASE_URL` is set.
    pub fn from_env() -> Option<Self> {
        if !enabled_from_str(std::env::var("DAAS_ENABLED").ok().as_deref()) {
            return None;
        }
        let base_url = std::env::var("DAAS_BASE_URL")
            .ok()
            .map(|v| v.trim().trim_end_matches('/').to_string())
            .filter(|v| !v.is_empty())?;
        let service_token = std::env::var("DAAS_SERVICE_TOKEN")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());
        let default_knowledge_engine = std::env::var("DAAS_DEFAULT_KNOWLEDGE_ENGINE")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());
        Some(Self {
            base_url,
            service_token,
            default_knowledge_engine,
        })
    }
}

/// Async DaaS HTTP client. Uses ERP's own `reqwest` (async) — deliberately NOT
/// the `daas-sdk` crate, which is blocking and lacks a document-upload method.
pub struct DaasClient {
    cfg: DaasConfig,
    http: reqwest::Client,
}

/// Whether a `DAAS_ENABLED` value should turn the feature on. Accepts the usual
/// truthy spellings; absent/empty/anything else → disabled.
fn enabled_from_str(value: Option<&str>) -> bool {
    value
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

/// True when DaaS would accept a document with this name (by extension).
pub fn is_supported_document(file_name: &str) -> bool {
    let ext = std::path::Path::new(file_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    SUPPORTED_EXTS.contains(&ext.as_str())
}

impl DaasClient {
    pub fn new(cfg: DaasConfig) -> Self {
        Self {
            cfg,
            // Shared fleet client (w-http): standard 30s timeout + obscura UA + pool.
            // Replaces a bare `reqwest::Client::new()`; the request/auth shape below
            // is unchanged (w-http returns a plain `reqwest::Client`).
            http: w_http::client(),
        }
    }

    /// Attach auth: prefer the forwarded user bearer, else the service token.
    /// If neither is present, send no header — a DaaS instance running without
    /// `DAAS_SERVICE_TOKEN` treats that as a (dev-mode) service caller.
    fn auth(&self, req: reqwest::RequestBuilder, bearer: Option<&str>) -> reqwest::RequestBuilder {
        match bearer
            .map(str::to_string)
            .or_else(|| self.cfg.service_token.clone())
        {
            Some(token) if !token.is_empty() => req.bearer_auth(token),
            _ => req,
        }
    }

    /// Resolve (and lazily create) the DaaS workspace/project for an ERP company.
    /// A DaaS `workspace_id` IS its `project_id` (1:1). The mapping is cached in
    /// `daas_company_workspace`; a stale mapping (DaaS 404) is recreated.
    pub async fn ensure_workspace(
        &self,
        pool: &SqlitePool,
        company_id: &str,
        bearer: Option<&str>,
    ) -> Result<String, String> {
        // 1. Reuse an existing mapping if the workspace still exists.
        if let Ok(Some((ws,))) = sqlx::query_as::<_, (String,)>(
            "SELECT daas_workspace_id FROM daas_company_workspace WHERE company_id = ?",
        )
        .bind(company_id)
        .fetch_optional(pool)
        .await
        {
            let url = format!("{}/api/daas/v1/workspaces/{}", self.cfg.base_url, ws);
            match self.auth(self.http.get(&url), bearer).send().await {
                Ok(resp) if resp.status().is_success() => return Ok(ws),
                Ok(_) => { /* stale (404/401) — fall through and recreate */ }
                Err(e) => return Err(format!("GET workspace failed: {e}")),
            }
        }

        // 2. Create a new workspace named after the ERP company.
        let company_name =
            sqlx::query_as::<_, (String,)>("SELECT name FROM companies WHERE id = ?")
                .bind(company_id)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten()
                .map(|(n,)| n)
                .filter(|n| !n.trim().is_empty())
                .unwrap_or_else(|| format!("OPC {company_id}"));

        let url = format!("{}/api/daas/v1/workspaces", self.cfg.base_url);
        let mut body = serde_json::json!({
            "name": company_name,
            "description": format!("OPC company {company_id}"),
        });
        if let Some(engine) = &self.cfg.default_knowledge_engine {
            body["knowledge_engine"] = serde_json::json!(engine);
        }
        let resp = self
            .auth(self.http.post(&url).json(&body), bearer)
            .send()
            .await
            .map_err(|e| format!("create workspace failed: {e}"))?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!(
                "create workspace HTTP {}: {}",
                status.as_u16(),
                text
            ));
        }
        let parsed: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| format!("workspace response parse: {e}"))?;
        let ws = parsed
            .get("workspace_id")
            .and_then(|v| v.as_str())
            .ok_or("workspace_id missing in response")?
            .to_string();

        // 3. Persist the mapping (idempotent).
        sqlx::query(
            "INSERT OR REPLACE INTO daas_company_workspace (company_id, daas_workspace_id) VALUES (?, ?)",
        )
        .bind(company_id)
        .bind(&ws)
        .execute(pool)
        .await
        .map_err(|e| format!("persist mapping failed: {e}"))?;
        Ok(ws)
    }

    /// Upload one document into a DaaS workspace (parsing deferred — Phase 2).
    /// Returns the `document_asset_id`.
    pub async fn upload_document(
        &self,
        workspace_id: &str,
        file_name: &str,
        content_type: &str,
        bytes: Vec<u8>,
        bearer: Option<&str>,
    ) -> Result<String, String> {
        let url = format!(
            "{}/api/daas/v1/workspaces/{}/documents?auto_parse=false",
            self.cfg.base_url, workspace_id
        );
        let part = reqwest::multipart::Part::bytes(bytes)
            .file_name(file_name.to_string())
            .mime_str(content_type)
            .map_err(|e| format!("bad mime {content_type}: {e}"))?;
        let form = reqwest::multipart::Form::new().part("file", part);
        let resp = self
            .auth(self.http.post(&url).multipart(form), bearer)
            .send()
            .await
            .map_err(|e| format!("upload failed: {e}"))?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!("upload HTTP {}: {}", status.as_u16(), text));
        }
        // Response: { "results": [ { "document_asset_id", ... } ], "errors": [...] }
        let parsed: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| format!("upload response parse: {e}"))?;
        if let Some(id) = parsed
            .get("results")
            .and_then(|r| r.as_array())
            .and_then(|a| a.first())
            .and_then(|item| item.get("document_asset_id"))
            .and_then(|v| v.as_str())
        {
            return Ok(id.to_string());
        }
        let errors = parsed
            .get("errors")
            .and_then(|e| e.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            })
            .unwrap_or_default();
        Err(if errors.is_empty() {
            "upload returned no results".to_string()
        } else {
            format!("upload rejected: {errors}")
        })
    }

    /// Mirror a single file end-to-end and record the outcome on its `files`
    /// row. Best-effort: every path updates `daas_sync_status`, never panics the
    /// caller. Returns the resulting status for counting.
    pub async fn mirror_file(
        &self,
        pool: &SqlitePool,
        file_id: &str,
        company_id: &str,
        file_name: &str,
        content_type: &str,
        bytes: Vec<u8>,
        bearer: Option<String>,
    ) -> &'static str {
        if !is_supported_document(file_name) {
            let _ = mark(pool, file_id, "skipped", None, None, None).await;
            return "skipped";
        }
        let bearer_ref = bearer.as_deref();
        let ws = match self.ensure_workspace(pool, company_id, bearer_ref).await {
            Ok(ws) => ws,
            Err(e) => {
                let _ = mark(pool, file_id, "failed", None, None, Some(&e)).await;
                return "failed";
            }
        };
        match self
            .upload_document(&ws, file_name, content_type, bytes, bearer_ref)
            .await
        {
            Ok(doc_id) => {
                let _ = mark(pool, file_id, "synced", Some(&ws), Some(&doc_id), None).await;
                "synced"
            }
            Err(e) => {
                let _ = mark(pool, file_id, "failed", Some(&ws), None, Some(&e)).await;
                "failed"
            }
        }
    }

    /// Re-mirror a company's `pending`/`failed` files (admin-triggered backfill).
    /// Reads bytes from the local upload dir. Returns (synced, failed, skipped).
    pub async fn backfill_company(
        &self,
        pool: &SqlitePool,
        company_id: &str,
        bearer: Option<String>,
        uploads_root: &str,
    ) -> (usize, usize, usize) {
        let rows: Vec<(String, String, String, String)> = sqlx::query_as(
            "SELECT id, filename, original_name, content_type FROM files \
             WHERE company_id = ? AND daas_sync_status IN ('pending', 'failed') \
             ORDER BY created_at ASC LIMIT 500",
        )
        .bind(company_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let (mut synced, mut failed, mut skipped) = (0usize, 0usize, 0usize);
        for (id, filename, original_name, content_type) in rows {
            let path = format!("{uploads_root}/{company_id}/{filename}");
            let bytes = match tokio::fs::read(&path).await {
                Ok(b) => b,
                Err(e) => {
                    let _ = mark(
                        pool,
                        &id,
                        "failed",
                        None,
                        None,
                        Some(&format!("read {path}: {e}")),
                    )
                    .await;
                    failed += 1;
                    continue;
                }
            };
            match self
                .mirror_file(
                    pool,
                    &id,
                    company_id,
                    &original_name,
                    &content_type,
                    bytes,
                    bearer.clone(),
                )
                .await
            {
                "synced" => synced += 1,
                "skipped" => skipped += 1,
                _ => failed += 1,
            }
        }
        (synced, failed, skipped)
    }
}

/// Update the DaaS mirror columns on a `files` row. `ws`/`doc_id` are kept
/// (COALESCE) when not supplied; `daas_synced_at` is stamped only on success.
async fn mark(
    pool: &SqlitePool,
    file_id: &str,
    status: &str,
    ws: Option<&str>,
    doc_id: Option<&str>,
    err: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE files SET \
            daas_sync_status = ?, \
            daas_workspace_id = COALESCE(?, daas_workspace_id), \
            daas_document_asset_id = COALESCE(?, daas_document_asset_id), \
            daas_synced_at = CASE WHEN ? = 'synced' THEN datetime('now') ELSE daas_synced_at END, \
            daas_sync_error = ? \
         WHERE id = ?",
    )
    .bind(status)
    .bind(ws)
    .bind(doc_id)
    .bind(status)
    .bind(err)
    .bind(file_id)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_document_matches_daas_allowlist() {
        assert!(is_supported_document("contract.pdf"));
        assert!(is_supported_document("Report.DOCX"));
        assert!(is_supported_document("notes.md"));
        // Image receipts are out of scope for Phase 1 → skipped, not synced.
        assert!(!is_supported_document("receipt.png"));
        assert!(!is_supported_document("scan.jpg"));
        assert!(!is_supported_document("noext"));
    }

    #[test]
    fn enabled_flag_parsing() {
        for on in ["1", "true", "TRUE", "yes", "on", " on "] {
            assert!(enabled_from_str(Some(on)), "{on:?} should enable");
        }
        for off in [None, Some(""), Some("0"), Some("false"), Some("nope")] {
            assert!(!enabled_from_str(off), "{off:?} should not enable");
        }
    }

    fn cfg(service_token: Option<&str>) -> DaasConfig {
        DaasConfig {
            base_url: "http://daas.local".to_string(),
            service_token: service_token.map(str::to_string),
            default_knowledge_engine: None,
        }
    }

    /// The DaaS client now builds its `reqwest::Client` via `w_http::client()`.
    /// This proves (a) construction succeeds (the shared kit returns a usable
    /// `reqwest::Client`) and (b) the auth/bearer attachment is byte-for-byte
    /// unchanged from the hand-rolled version: forwarded user bearer wins, else
    /// the service token, else no `Authorization` header at all.
    #[test]
    fn auth_header_shape_preserved() {
        // 1. Forwarded user bearer takes precedence over the service token.
        let client = DaasClient::new(cfg(Some("svc-token")));
        let req = client
            .auth(client.http.get("http://daas.local/x"), Some("user-token"))
            .build()
            .expect("request builds");
        assert_eq!(
            req.headers()
                .get("authorization")
                .map(|v| v.to_str().unwrap()),
            Some("Bearer user-token"),
        );

        // 2. No forwarded bearer → fall back to the configured service token.
        let req = client
            .auth(client.http.get("http://daas.local/x"), None)
            .build()
            .expect("request builds");
        assert_eq!(
            req.headers()
                .get("authorization")
                .map(|v| v.to_str().unwrap()),
            Some("Bearer svc-token"),
        );

        // 3. Neither bearer nor service token → no Authorization header.
        let client = DaasClient::new(cfg(None));
        let req = client
            .auth(client.http.get("http://daas.local/x"), None)
            .build()
            .expect("request builds");
        assert!(req.headers().get("authorization").is_none());
    }
}
