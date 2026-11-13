/// Scenario 65: 🧪 CLI/API Golden Flow TDD
/// Exercises real `opc` subprocesses against a real HTTP server and verifies
/// persisted state through API/database reads.
use crate::common;

use reqwest::Client;
use serde_json::Value;
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::process::Output;

fn opc_bin() -> String {
    let p = PathBuf::from(env!("CARGO_BIN_EXE_opc"));
    p.to_str().unwrap().to_string()
}

async fn start_real_server() -> (String, String, SqlitePool, String, String) {
    let pool = SqlitePool::connect("sqlite::memory:").await.expect("db");
    opc::db::run_migrations(&pool).await.expect("migrations");

    let user_id = common::seed::create_user(&pool, "golden@test-dev.example.invalid", "Golden User").await;
    let company_id = common::seed::create_company(&pool, &user_id, "Golden Corp", "CNY").await;
    let session = common::seed::create_session(&pool, &user_id, &company_id).await;

    let app = opc::build_app(pool.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    (
        format!("http://{}", addr),
        session,
        pool,
        user_id,
        company_id,
    )
}

async fn cli(server: &str, session: &str, args: &[&str]) -> Output {
    tokio::process::Command::new(opc_bin())
        .env("OPC_SERVER", server)
        .env("OPC_SESSION", session)
        .args(args)
        .output()
        .await
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_string()
}

fn unique_temp_file(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", uuid::Uuid::new_v4(), name))
}

#[tokio::test]
async fn s65_cli_company_switch_changes_followup_write_scope() {
    let (base, session, pool, user_id, original_company_id) = start_real_server().await;
    let second_company_id =
        common::seed::create_company(&pool, &user_id, "Switched Corp", "USD").await;

    let switch_out = cli(&base, &session, &["company", "switch", &second_company_id]).await;
    assert!(
        switch_out.status.success(),
        "company switch should succeed\nstdout:\n{}\nstderr:\n{}",
        stdout(&switch_out),
        stderr(&switch_out)
    );

    let create_out = cli(
        &base,
        &session,
        &[
            "--quiet",
            "create",
            "invoices",
            "-f",
            "invoice_number=CLI-SWITCH-001",
            "-f",
            "amount=1234",
        ],
    )
    .await;
    assert!(
        create_out.status.success(),
        "invoice create after switch should succeed\nstdout:\n{}\nstderr:\n{}",
        stdout(&create_out),
        stderr(&create_out)
    );

    let invoice_id = stdout(&create_out);
    let stored_company_id: String =
        sqlx::query_scalar("SELECT company_id FROM invoices WHERE id = ?")
            .bind(&invoice_id)
            .fetch_one(&pool)
            .await
            .expect("invoice company");

    assert_ne!(original_company_id, second_company_id);
    assert_eq!(stored_company_id, second_company_id);
}

#[tokio::test]
async fn s65_cli_files_upload_list_download_flow() {
    let (base, session, pool, _user_id, company_id) = start_real_server().await;

    let create_invoice_out = cli(
        &base,
        &session,
        &[
            "--quiet",
            "create",
            "invoices",
            "-f",
            "invoice_number=CLI-FILE-001",
            "-f",
            "amount=8800",
        ],
    )
    .await;
    assert!(
        create_invoice_out.status.success(),
        "invoice create should succeed\nstdout:\n{}\nstderr:\n{}",
        stdout(&create_invoice_out),
        stderr(&create_invoice_out)
    );
    let invoice_id = stdout(&create_invoice_out);

    let upload_path = unique_temp_file("cli-upload.txt");
    let download_path = unique_temp_file("cli-download.txt");
    let source_bytes = b"golden flow upload via real opc cli";
    std::fs::write(&upload_path, source_bytes).expect("write upload file");

    let upload_path_arg = upload_path.to_string_lossy().to_string();
    let download_path_arg = download_path.to_string_lossy().to_string();

    let upload_out = cli(
        &base,
        &session,
        &[
            "--quiet",
            "files",
            "upload",
            &upload_path_arg,
            "--entity-type",
            "invoices",
            "--entity-id",
            &invoice_id,
        ],
    )
    .await;
    assert!(
        upload_out.status.success(),
        "files upload should succeed\nstdout:\n{}\nstderr:\n{}",
        stdout(&upload_out),
        stderr(&upload_out)
    );
    let file_id = stdout(&upload_out);

    let list_out = cli(
        &base,
        &session,
        &[
            "--format",
            "json",
            "files",
            "list",
            "--entity-type",
            "invoices",
            "--entity-id",
            &invoice_id,
        ],
    )
    .await;
    assert!(
        list_out.status.success(),
        "files list should succeed\nstdout:\n{}\nstderr:\n{}",
        stdout(&list_out),
        stderr(&list_out)
    );
    let list: Vec<Value> = serde_json::from_str(&stdout(&list_out)).expect("parse files list");
    assert!(
        list.iter().any(|item| item["id"] == file_id),
        "uploaded file should appear in CLI list output"
    );

    let download_out = cli(
        &base,
        &session,
        &[
            "files",
            "download",
            &file_id,
            "--output",
            &download_path_arg,
        ],
    )
    .await;
    assert!(
        download_out.status.success(),
        "files download should succeed\nstdout:\n{}\nstderr:\n{}",
        stdout(&download_out),
        stderr(&download_out)
    );
    let downloaded_bytes = std::fs::read(&download_path).expect("read downloaded file");
    assert_eq!(downloaded_bytes, source_bytes);

    let stored: (String, String, String) =
        sqlx::query_as("SELECT company_id, entity_type, entity_id FROM files WHERE id = ?")
            .bind(&file_id)
            .fetch_one(&pool)
            .await
            .expect("stored file record");
    assert_eq!(stored.0, company_id);
    assert_eq!(stored.1, "invoices");
    assert_eq!(stored.2, invoice_id);

    let _ = std::fs::remove_file(&upload_path);
    let _ = std::fs::remove_file(&download_path);
}

#[tokio::test]
async fn s65_cli_company_members_add_and_list_flow() {
    let (base, session, pool, _user_id, company_id) = start_real_server().await;
    let invited_email = "new.member@test-dev.example.invalid";
    let invited_user_id = common::seed::create_user(&pool, invited_email, "New Member").await;

    let add_out = cli(
        &base,
        &session,
        &[
            "company",
            "members",
            "add",
            &company_id,
            "--email",
            invited_email,
            "--role",
            "admin",
        ],
    )
    .await;
    assert!(
        add_out.status.success(),
        "company members add should succeed\nstdout:\n{}\nstderr:\n{}",
        stdout(&add_out),
        stderr(&add_out)
    );

    let list_out = cli(
        &base,
        &session,
        &[
            "--format",
            "json",
            "company",
            "members",
            "list",
            &company_id,
        ],
    )
    .await;
    assert!(
        list_out.status.success(),
        "company members list should succeed\nstdout:\n{}\nstderr:\n{}",
        stdout(&list_out),
        stderr(&list_out)
    );
    let members: Vec<Value> =
        serde_json::from_str(&stdout(&list_out)).expect("parse company members list");
    assert!(
        members.iter().any(|item| item["email"] == invited_email),
        "newly added member should appear in list output"
    );

    let role: String =
        sqlx::query_scalar("SELECT role FROM company_members WHERE company_id = ? AND user_id = ?")
            .bind(&company_id)
            .bind(&invited_user_id)
            .fetch_one(&pool)
            .await
            .expect("member role");
    assert_eq!(role, "admin");
}

#[tokio::test]
async fn s65_cli_company_switch_json_output_is_machine_readable() {
    let (base, session, pool, user_id, _original_company_id) = start_real_server().await;
    let second_company_id =
        common::seed::create_company(&pool, &user_id, "JSON Switch Corp", "USD").await;

    let out = cli(
        &base,
        &session,
        &["--format", "json", "company", "switch", &second_company_id],
    )
    .await;
    assert!(
        out.status.success(),
        "company switch should succeed\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );

    let body: Value = serde_json::from_str(&stdout(&out)).expect("parse company switch JSON");
    assert_eq!(body["current_company_id"], second_company_id);
}

#[tokio::test]
async fn s65_cli_company_members_add_quiet_outputs_member_id_only() {
    let (base, session, pool, _user_id, company_id) = start_real_server().await;
    let invited_email = "quiet.member@test-dev.example.invalid";
    let invited_user_id = common::seed::create_user(&pool, invited_email, "Quiet Member").await;

    let out = cli(
        &base,
        &session,
        &[
            "--quiet",
            "company",
            "members",
            "add",
            &company_id,
            "--email",
            invited_email,
            "--role",
            "accountant",
        ],
    )
    .await;
    assert!(
        out.status.success(),
        "quiet company members add should succeed\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );

    let member_id = stdout(&out);
    let stored: (String, String) =
        sqlx::query_as("SELECT id, role FROM company_members WHERE company_id = ? AND user_id = ?")
            .bind(&company_id)
            .bind(&invited_user_id)
            .fetch_one(&pool)
            .await
            .expect("member row");
    assert_eq!(member_id, stored.0);
    assert_eq!(stored.1, "accountant");
}

#[tokio::test]
async fn s65_cli_files_upload_json_output_is_machine_readable() {
    let (base, session, pool, _user_id, company_id) = start_real_server().await;

    let create_invoice_out = cli(
        &base,
        &session,
        &[
            "--quiet",
            "create",
            "invoices",
            "-f",
            "invoice_number=CLI-FILE-JSON-001",
            "-f",
            "amount=9900",
        ],
    )
    .await;
    assert!(
        create_invoice_out.status.success(),
        "invoice create should succeed\nstdout:\n{}\nstderr:\n{}",
        stdout(&create_invoice_out),
        stderr(&create_invoice_out)
    );
    let invoice_id = stdout(&create_invoice_out);

    let upload_path = unique_temp_file("cli-upload-json.txt");
    std::fs::write(&upload_path, b"json upload via cli").expect("write upload file");
    let upload_path_arg = upload_path.to_string_lossy().to_string();

    let out = cli(
        &base,
        &session,
        &[
            "--format",
            "json",
            "files",
            "upload",
            &upload_path_arg,
            "--entity-type",
            "invoice",
            "--entity-id",
            &invoice_id,
        ],
    )
    .await;
    assert!(
        out.status.success(),
        "json files upload should succeed\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );

    let body: Value = serde_json::from_str(&stdout(&out)).expect("parse files upload JSON");
    let file_id = body["id"].as_str().expect("file id");
    assert_eq!(body["entity_type"], "invoices");

    let stored: (String, String, String) =
        sqlx::query_as("SELECT company_id, entity_type, entity_id FROM files WHERE id = ?")
            .bind(file_id)
            .fetch_one(&pool)
            .await
            .expect("stored file row");
    assert_eq!(stored.0, company_id);
    assert_eq!(stored.1, "invoices");
    assert_eq!(stored.2, invoice_id);

    let _ = std::fs::remove_file(&upload_path);
}

#[tokio::test]
async fn s65_cli_dashboard_matches_http_after_cli_writes() {
    let (base, session, _pool, _user_id, _company_id) = start_real_server().await;

    let create_income = cli(
        &base,
        &session,
        &[
            "--quiet",
            "create",
            "invoices",
            "-f",
            "invoice_number=CLI-DASH-INC",
            "-f",
            "amount=5000",
            "-f",
            "invoice_type=income",
        ],
    )
    .await;
    assert!(
        create_income.status.success(),
        "income invoice create should succeed\nstdout:\n{}\nstderr:\n{}",
        stdout(&create_income),
        stderr(&create_income)
    );

    let create_expense = cli(
        &base,
        &session,
        &[
            "--quiet",
            "create",
            "invoices",
            "-f",
            "invoice_number=CLI-DASH-EXP",
            "-f",
            "amount=1200",
            "-f",
            "invoice_type=expense",
        ],
    )
    .await;
    assert!(
        create_expense.status.success(),
        "expense invoice create should succeed\nstdout:\n{}\nstderr:\n{}",
        stdout(&create_expense),
        stderr(&create_expense)
    );

    let dashboard_out = cli(&base, &session, &["--format", "json", "dashboard"]).await;
    assert!(
        dashboard_out.status.success(),
        "dashboard command should succeed\nstdout:\n{}\nstderr:\n{}",
        stdout(&dashboard_out),
        stderr(&dashboard_out)
    );
    let dashboard_from_cli: Value =
        serde_json::from_str(&stdout(&dashboard_out)).expect("parse dashboard JSON");

    let api_dashboard: Value = Client::new()
        .get(format!("{}/api/v1/stats/dashboard", base))
        .header("Authorization", format!("Bearer {}", session))
        .send()
        .await
        .expect("dashboard request")
        .json()
        .await
        .expect("dashboard json");

    assert_eq!(
        dashboard_from_cli["total_invoices"],
        api_dashboard["total_invoices"]
    );
}
