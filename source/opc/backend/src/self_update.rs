//! Optional self-update from an operator-configured trusted manifest.
//! Downloads are applied on the next restart. Configure SELF_UPDATE_MANIFEST_URL
//! and SELF_UPDATE_AUTO_APPLY=1 to enable automatic replacement.

use serde::Deserialize;
use std::path::Path;
use std::time::Duration;
use tracing::{info, warn};
use w_crypto::sha256_hex;

const FETCH_TIMEOUT_SECS: u64 = 15;
const DOWNLOAD_TIMEOUT_SECS: u64 = 300;
const MIN_BINARY_BYTES: usize = 1024;
const MAX_BINARY_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Debug, Deserialize)]
struct Manifest {
    version: String,
    targets: Targets,
}

#[derive(Debug, Deserialize)]
struct Targets {
    #[serde(rename = "linux-x86_64")]
    linux_x86_64: Target,
}

#[derive(Debug, Deserialize)]
struct Target {
    url: String,
    #[serde(default)]
    sha256: Option<String>,
}

/// Spawn a background self-update check. Logs only; with auto-apply it downloads
/// and atomically replaces the on-disk binary (effective on next restart).
/// Never blocks startup or panics. Disable per-host with `SELF_UPDATE_AUTO_APPLY=0`.
pub fn spawn(current_version: &'static str) {
    let auto_apply = std::env::var("SELF_UPDATE_AUTO_APPLY")
        .map(|v| {
            !matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "0" | "false" | "off" | "no"
            )
        })
        .unwrap_or(false);
    let Some(manifest_url) = std::env::var("SELF_UPDATE_MANIFEST_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
    else {
        return;
    };
    let manifest_urls = vec![manifest_url];

    tokio::spawn(async move {
        let mut last_error = None;
        for manifest_url in &manifest_urls {
            match check_and_apply(current_version, manifest_url, auto_apply).await {
                Ok(Some(v)) if auto_apply => {
                    info!(
                        "🆕 self-update: v{current_version} → v{v} downloaded — restart to apply"
                    );
                    return;
                }
                Ok(Some(v)) => {
                    info!("🆕 self-update available: v{current_version} → v{v} (auto-apply off)");
                    return;
                }
                Ok(None) => {
                    info!("✅ self-update: running latest v{current_version}");
                    return;
                }
                Err(error) => last_error = Some(error),
            }
        }
        if let Some(error) = last_error {
            warn!("self-update check failed (non-fatal): {error}");
        }
    });
}

/// Returns `Ok(Some(version))` if the manifest advertises a newer version
/// (and, when `auto_apply`, after the new binary has been installed).
async fn check_and_apply(
    current: &str,
    manifest_url: &str,
    auto_apply: bool,
) -> Result<Option<String>, String> {
    let manifest = fetch_manifest(manifest_url).await?;
    if !is_newer(current, &manifest.version) {
        return Ok(None);
    }
    if auto_apply {
        let exe = std::env::current_exe().map_err(|e| format!("cannot find current exe: {e}"))?;
        let bytes = download(&manifest.targets.linux_x86_64.url).await?;
        verify_and_install(
            &bytes,
            manifest.targets.linux_x86_64.sha256.as_deref(),
            &exe,
        )?;
    }
    Ok(Some(manifest.version))
}

async fn fetch_manifest(url: &str) -> Result<Manifest, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(FETCH_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("http client: {e}"))?;
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("fetch manifest {url}: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("manifest {url} returned {}", resp.status()));
    }
    resp.json::<Manifest>()
        .await
        .map_err(|e| format!("invalid manifest JSON: {e}"))
}

async fn download(url: &str) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(DOWNLOAD_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("http client: {e}"))?;
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("download {url}: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("download {url} returned {}", resp.status()));
    }
    if let Some(len) = resp.content_length() {
        if len > MAX_BINARY_BYTES {
            return Err(format!("binary too large: {len} bytes"));
        }
    }
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("read download: {e}"))?;
    Ok(bytes.to_vec())
}

/// Verify the SHA-256 (when the manifest provides one) and atomically replace
/// `dest`: write `dest.new`, mark executable, rename over `dest`. The previous
/// binary is kept as `dest.bak`. `dest` is left untouched on any failure.
fn verify_and_install(bytes: &[u8], expected_sha: Option<&str>, dest: &Path) -> Result<(), String> {
    if bytes.len() < MIN_BINARY_BYTES {
        return Err(format!("downloaded file too small: {} bytes", bytes.len()));
    }
    if let Some(expected) = expected_sha {
        let actual = sha256_hex(bytes);
        if !actual.eq_ignore_ascii_case(expected.trim()) {
            return Err(format!(
                "sha256 mismatch: expected {expected}, got {actual}"
            ));
        }
    }

    let new_path = dest.with_extension("new");
    std::fs::write(&new_path, bytes).map_err(|e| format!("write {}: {e}", new_path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&new_path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("chmod: {e}"))?;
    }
    if dest.exists() {
        let _ = std::fs::rename(dest, dest.with_extension("bak"));
    }
    std::fs::rename(&new_path, dest).map_err(|e| {
        // best-effort rollback
        let _ = std::fs::rename(dest.with_extension("bak"), dest);
        format!("replace binary: {e}")
    })?;
    Ok(())
}

/// `true` iff `candidate` is a strictly higher semver-ish version than `current`.
fn is_newer(current: &str, candidate: &str) -> bool {
    match (parse_ver(candidate), parse_ver(current)) {
        (Some(c), Some(cur)) => c > cur,
        _ => false,
    }
}

/// Parse `vX.Y.Z` / `X.Y.Z[-pre]` into a comparable tuple. Missing patch = 0.
fn parse_ver(v: &str) -> Option<(u64, u64, u64)> {
    let v = v.trim().trim_start_matches('v');
    let mut parts = v.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts
        .next()
        .unwrap_or("0")
        .split(['-', '+'])
        .next()?
        .parse()
        .ok()?;
    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_version_detected() {
        assert!(is_newer("0.2.0", "0.2.3"));
        assert!(is_newer("0.2.3", "0.3.0"));
        assert!(is_newer("0.2.3", "1.0.0"));
        assert!(is_newer("0.2.3", "v0.2.4")); // tolerates a leading v
    }

    #[test]
    fn same_or_older_is_not_newer() {
        assert!(!is_newer("0.2.3", "0.2.3"));
        assert!(!is_newer("0.2.3", "0.2.2"));
        assert!(!is_newer("0.2.3", "0.1.9"));
    }

    #[test]
    fn unparseable_versions_never_update() {
        assert!(!is_newer("0.2.3", "garbage"));
        assert!(!is_newer("nope", "0.2.4"));
    }

    #[test]
    fn manifest_parses_ci_format() {
        let json = r#"{"version":"0.2.4","targets":{"linux-x86_64":{"url":"https://releases.example.invalid/w-erp/opc-server-linux-amd64","sha256":"abc","size":123}}}"#;
        let m: Manifest = serde_json::from_str(json).unwrap();
        assert_eq!(m.version, "0.2.4");
        assert_eq!(
            m.targets.linux_x86_64.url,
            "https://releases.example.invalid/w-erp/opc-server-linux-amd64"
        );
        assert_eq!(m.targets.linux_x86_64.sha256.as_deref(), Some("abc"));
    }

    #[test]
    fn install_replaces_dest_when_sha_matches() {
        let dir = std::env::temp_dir().join(format!("opc-selfupd-ok-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let dest = dir.join("opc-server");
        std::fs::write(&dest, b"OLD BINARY").unwrap();

        let new_bytes = vec![7u8; 2048];
        let sha = sha256_hex(&new_bytes);
        verify_and_install(&new_bytes, Some(&sha), &dest).expect("install");

        assert_eq!(
            std::fs::read(&dest).unwrap(),
            new_bytes,
            "dest must hold the new binary"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn install_rejects_sha_mismatch_and_keeps_dest() {
        let dir = std::env::temp_dir().join(format!("opc-selfupd-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let dest = dir.join("opc-server");
        std::fs::write(&dest, b"OLD BINARY KEEP ME").unwrap();

        let new_bytes = vec![9u8; 2048];
        let err = verify_and_install(&new_bytes, Some("deadbeef"), &dest).unwrap_err();
        assert!(err.contains("sha256 mismatch"), "got: {err}");
        assert_eq!(
            std::fs::read(&dest).unwrap(),
            b"OLD BINARY KEEP ME",
            "dest must be untouched on sha mismatch"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn install_rejects_truncated_download() {
        let dir = std::env::temp_dir().join(format!("opc-selfupd-tiny-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let dest = dir.join("opc-server");
        let err = verify_and_install(b"too small", None, &dest).unwrap_err();
        assert!(err.contains("too small"), "got: {err}");
        std::fs::remove_dir_all(&dir).ok();
    }
}
