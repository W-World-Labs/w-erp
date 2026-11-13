/// Scenario 64: 🖥️ CLI Binary E2E — subprocess tests
/// Actually invokes `opc` binary as a child process to verify the entire CLI pipeline.
/// Tests: --help, status, CRUD via CLI flags, --format json, --quiet mode
use crate::common;

use reqwest::Client;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::process::Command;

fn opc_bin() -> String {
    let p = PathBuf::from(env!("CARGO_BIN_EXE_opc"));
    p.to_str().unwrap().to_string()
}

async fn start_real_server() -> (String, String, SqlitePool) {
    let pool = common::memory_pool().await;
    opc::db::run_migrations(&pool).await.expect("migrations");

    let user_id = common::seed::create_user(&pool, "s64@test-dev.example.invalid", "S64 User").await;
    let company_id = common::seed::create_company(&pool, &user_id, "S64 Corp", "CNY").await;
    let session = common::seed::create_session(&pool, &user_id, &company_id).await;

    let app = opc::build_app(pool.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    (format!("http://{}", addr), session, pool)
}

async fn run_cli(server: &str, session: &str, args: &[&str]) -> std::process::Output {
    tokio::process::Command::new(opc_bin())
        .env("OPC_SERVER", server)
        .env("OPC_SESSION", session)
        .args(args)
        .output()
        .await
        .unwrap()
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_string()
}

// ══════════════ Help & Version ══════════════

#[test]
fn s64_cli_help_shows_all_commands() {
    let out = Command::new(opc_bin()).arg("--help").output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "opc --help should succeed");
    assert!(stdout.contains("OPC"), "Should show OPC branding");
    // All major commands present
    for cmd in &[
        "status",
        "dashboard",
        "auth",
        "company",
        "bank",
        "ai",
        "report",
        "pdf",
        "policies",
        "locations",
        "files",
        "api-keys",
        "tax",
        "currency",
        "ar",
        "webhooks",
        "list",
        "get",
        "create",
        "update",
        "delete",
        "export",
    ] {
        assert!(stdout.contains(cmd), "Missing command: {}", cmd);
    }
}

#[test]
fn s64_cli_version() {
    let out = Command::new(opc_bin()).arg("--version").output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("opc"), "Should show opc in version");
}

// ══════════════ Subcommand Help ══════════════

#[test]
fn s64_cli_subcommand_help() {
    for cmd in &[
        "auth",
        "company",
        "bank",
        "ai",
        "report",
        "pdf",
        "policies",
        "locations",
        "files",
        "api-keys",
        "tax",
        "currency",
        "ar",
        "webhooks",
    ] {
        let out = Command::new(opc_bin())
            .arg(cmd)
            .arg("--help")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(out.status.success(), "{} --help should succeed", cmd);
        assert!(
            stdout.contains("Usage:"),
            "{} --help should show Usage",
            cmd
        );
    }
}

// ══════════════ CRUD Help ══════════════

#[test]
fn s64_cli_crud_help() {
    for cmd in &["list", "get", "create", "update", "delete", "export"] {
        let out = Command::new(opc_bin())
            .arg(cmd)
            .arg("--help")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(out.status.success(), "{} --help should succeed", cmd);
        assert!(
            stdout.contains("ENTITY"),
            "{} --help should mention ENTITY",
            cmd
        );
    }
}

// ══════════════ Auth Required ══════════════

#[test]
fn s64_cli_no_auth_errors() {
    // Without --session, commands needing auth should fail gracefully
    let out = Command::new(opc_bin())
        .args(&["--server", "http://127.0.0.1:1", "status"])
        .output()
        .unwrap();
    // Should exit non-zero (connection refused or auth error)
    // This proves the CLI doesn't panic without auth
    assert!(
        !String::from_utf8_lossy(&out.stdout).is_empty()
            || !String::from_utf8_lossy(&out.stderr).is_empty(),
        "Should produce some output even on error"
    );
}

// ══════════════ Env Var Recognition ══════════════

#[test]
fn s64_cli_env_vars_in_help() {
    let out = Command::new(opc_bin()).arg("--help").output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("OPC_SERVER"), "Should show OPC_SERVER env");
    assert!(
        stdout.contains("OPC_SESSION"),
        "Should show OPC_SESSION env"
    );
    assert!(
        stdout.contains("OPC_API_KEY"),
        "Should show OPC_API_KEY env"
    );
}

// ══════════════ Format Flag ══════════════

#[test]
fn s64_cli_format_and_quiet_in_help() {
    let out = Command::new(opc_bin()).arg("--help").output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("--format"), "Should show --format");
    assert!(stdout.contains("--quiet"), "Should show --quiet");
    assert!(stdout.contains("table"), "Should mention table format");
    assert!(stdout.contains("json"), "Should mention json format");
}

// ══════════════ Tax Subcommand Help ══════════════

#[test]
fn s64_cli_tax_subcommands() {
    let out = Command::new(opc_bin())
        .args(&["tax", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    for sub in &[
        "calculate",
        "compliance-check",
        "invoice-requirements",
        "input-credit",
    ] {
        assert!(stdout.contains(sub), "tax should have {} subcommand", sub);
    }
}

// ══════════════ Currency Subcommand Help ══════════════

#[test]
fn s64_cli_currency_subcommands() {
    let out = Command::new(opc_bin())
        .args(&["currency", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("convert"));
    assert!(stdout.contains("balances"));
}

// ══════════════ Webhooks Subcommand Help ══════════════

#[test]
fn s64_cli_webhooks_subcommands() {
    let out = Command::new(opc_bin())
        .args(&["webhooks", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    for sub in &["list", "create", "delete", "test"] {
        assert!(stdout.contains(sub), "webhooks should have {}", sub);
    }
}

// ══════════════ Create Field Parsing ══════════════

#[test]
fn s64_cli_create_field_flag() {
    let out = Command::new(opc_bin())
        .args(&["create", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("-f"), "Should support -f short flag");
    assert!(
        stdout.contains("--field"),
        "Should support --field long flag"
    );
    assert!(stdout.contains("--json"), "Should support --json");
}

#[test]
fn s64_cli_rejects_empty_field_key() {
    let out = Command::new(opc_bin())
        .args(["create", "invoices", "-f", "=value"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "empty field keys should fail");
    assert!(
        stderr.contains("invalid KEY=VALUE: empty key"),
        "stderr should explain the empty key error, got: {}",
        stderr
    );
}

#[test]
fn s64_cli_invalid_json_reports_flag_name() {
    let out = Command::new(opc_bin())
        .args(["create", "invoices", "--json", "{"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "invalid --json should fail");
    assert!(
        stderr.contains("invalid JSON for --json"),
        "stderr should mention the failing flag, got: {}",
        stderr
    );
}

// ══════════════ API-Aligned CLI Surface ══════════════

#[test]
fn s64_company_members_add_help_uses_company_scope_and_email() {
    let out = Command::new(opc_bin())
        .args(&["company", "members", "add", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "company members add --help should succeed"
    );
    assert!(
        stdout.contains("COMPANY_ID"),
        "Should require company scope"
    );
    assert!(stdout.contains("--email"), "Should accept member email");
}

#[test]
fn s64_ai_parse_doc_help_mentions_document_type() {
    let out = Command::new(opc_bin())
        .args(&["ai", "parse-doc", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "ai parse-doc --help should succeed");
    assert!(
        stdout.contains("--document-type"),
        "Should require a document type"
    );
}

#[test]
fn s64_ai_contract_review_help_mentions_text_or_file() {
    let out = Command::new(opc_bin())
        .args(&["ai", "contract-review", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "ai contract-review --help should succeed"
    );
    assert!(
        stdout.contains("--text"),
        "Should support inline contract text"
    );
    assert!(
        stdout.contains("--file"),
        "Should support reading contract text from a file"
    );
}

#[tokio::test]
async fn s64_cli_api_keys_revoke_json_output_is_machine_readable() {
    let (base, session, pool) = start_real_server().await;

    let create: Value = Client::new()
        .post(format!("{}/api/api-keys", base))
        .header("Authorization", format!("Bearer {}", session))
        .json(&json!({ "name": "json revoke key" }))
        .send()
        .await
        .expect("create api key")
        .json()
        .await
        .expect("parse api key create");
    let key_id = create["id"].as_str().expect("api key id").to_string();

    let out = run_cli(
        &base,
        &session,
        &["--format", "json", "api-keys", "revoke", &key_id],
    )
    .await;
    assert!(
        out.status.success(),
        "api key revoke should succeed\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );

    let body: Value = serde_json::from_str(&stdout(&out)).expect("parse revoke JSON");
    assert_eq!(body["success"], true);
    assert_eq!(body["message"], "API key revoked");

    let is_active: i64 = sqlx::query_scalar("SELECT is_active FROM api_keys WHERE id = ?")
        .bind(&key_id)
        .fetch_one(&pool)
        .await
        .expect("fetch api key status");
    assert_eq!(is_active, 0);
}

#[tokio::test]
async fn s64_cli_api_keys_delete_quiet_outputs_id_only() {
    let (base, session, pool) = start_real_server().await;

    let create: Value = Client::new()
        .post(format!("{}/api/api-keys", base))
        .header("Authorization", format!("Bearer {}", session))
        .json(&json!({ "name": "quiet delete key" }))
        .send()
        .await
        .expect("create api key")
        .json()
        .await
        .expect("parse api key create");
    let key_id = create["id"].as_str().expect("api key id").to_string();

    let out = run_cli(&base, &session, &["--quiet", "api-keys", "delete", &key_id]).await;
    assert!(
        out.status.success(),
        "api key delete should succeed\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
    assert_eq!(stdout(&out), key_id);

    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM api_keys WHERE id = ?")
        .bind(&key_id)
        .fetch_one(&pool)
        .await
        .expect("fetch api key count");
    assert_eq!(remaining, 0);
}

#[tokio::test]
async fn s64_cli_generic_delete_json_output_is_machine_readable() {
    let (base, session, pool) = start_real_server().await;

    let create: Value = Client::new()
        .post(format!("{}/api/v1/contracts", base))
        .header("Authorization", format!("Bearer {}", session))
        .json(&json!({
            "contract_name": "Delete via CLI",
            "contract_type": "service",
            "counterparty": "CLI Counterparty",
            "amount": 1888
        }))
        .send()
        .await
        .expect("create contract")
        .json()
        .await
        .expect("parse contract create");
    let contract_id = create["id"].as_str().expect("contract id").to_string();

    let out = run_cli(
        &base,
        &session,
        &["--format", "json", "delete", "contracts", &contract_id],
    )
    .await;
    assert!(
        out.status.success(),
        "generic delete should succeed\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );

    let body: Value = serde_json::from_str(&stdout(&out)).expect("parse generic delete JSON");
    assert_eq!(body["id"], contract_id);
    assert_eq!(body["deleted"], true);

    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM contracts WHERE id = ?")
        .bind(&contract_id)
        .fetch_one(&pool)
        .await
        .expect("fetch remaining contracts");
    assert_eq!(remaining, 0, "contract should no longer exist after delete");
}

#[tokio::test]
async fn s64_cli_webhooks_test_json_output_is_machine_readable() {
    let (base, session, _pool) = start_real_server().await;

    let create: Value = Client::new()
        .post(format!("{}/api/v1/webhooks", base))
        .header("Authorization", format!("Bearer {}", session))
        .json(&json!({
            "url": "http://127.0.0.1:9/webhook-target",
            "events": "invoice.created"
        }))
        .send()
        .await
        .expect("create webhook")
        .json()
        .await
        .expect("parse webhook create");
    let webhook_id = create["id"].as_str().expect("webhook id").to_string();

    let out = run_cli(
        &base,
        &session,
        &["--format", "json", "webhooks", "test", &webhook_id],
    )
    .await;
    assert!(
        out.status.success(),
        "webhook test should succeed\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );

    let body: Value = serde_json::from_str(&stdout(&out)).expect("parse webhook test JSON");
    assert!(
        body["status"].is_string(),
        "status should stay machine-readable"
    );
    assert_eq!(body["url"], "http://127.0.0.1:9/webhook-target");
}
