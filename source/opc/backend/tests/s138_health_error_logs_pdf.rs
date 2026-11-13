// #7 / #10 / #12 — health probes, server error logging, PDF fonts, onboarding connectivity.
use crate::common;

use axum::http::StatusCode;
use serde_json::{Value, json};

#[tokio::test]
async fn s138_health_check_reports_db_ok_and_disk() {
    let app = common::TestApp::new().await;

    let res = app.server.get("/health").await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["db"], "ok");
    assert!(body["version"].as_str().is_some());
    assert!(body["uptime_s"].as_u64().is_some() || body["uptime_s"].as_i64().is_some());
    // disk_free_mb may be null on exotic FS but usually present
    assert!(body.get("disk_free_mb").is_some());

    // Write probe must have landed in app_meta
    let probe: Option<String> =
        sqlx::query_scalar("SELECT value FROM app_meta WHERE key = 'health_probe'")
            .fetch_optional(&app.pool)
            .await
            .expect("query app_meta");
    assert!(
        probe.is_some(),
        "health_probe row must be written by /health"
    );
}

#[tokio::test]
async fn s138_health_returns_503_when_db_unreachable() {
    // Build a pool, migrate, then close it so subsequent queries fail.
    let pool = common::memory_pool().await;
    opc::db::run_migrations(&pool).await.expect("migrations");
    let app = opc::build_app(pool.clone());
    let server = axum_test::TestServer::new(app).expect("test server");
    pool.close().await;

    let res = server.get("/health").await;
    assert_eq!(res.status_code(), StatusCode::SERVICE_UNAVAILABLE);
    let body: Value = res.json();
    assert_eq!(body["status"], "error");
    assert_eq!(body["db"], "unreachable");
}

#[tokio::test]
async fn s138_server_error_middleware_writes_error_logs() {
    let app = common::TestApp::new().await;

    // Drop a table so a list endpoint hard-fails with 500.
    sqlx::query("DROP TABLE contracts")
        .execute(&app.pool)
        .await
        .expect("drop contracts");

    let before: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM error_logs WHERE company_id = ? AND error_type = 'server_error'")
            .bind(&app.company_id)
            .fetch_one(&app.pool)
            .await
            .unwrap();

    let res = app.get("/api/v1/contracts").await;
    assert!(
        res.status_code().is_server_error(),
        "expected 5xx after dropping table, got {}",
        res.status_code()
    );

    // Second identical 500 within the throttle window must not double-insert.
    let _ = app.get("/api/v1/contracts").await;

    let after: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM error_logs WHERE company_id = ? AND error_type = 'server_error'")
            .bind(&app.company_id)
            .fetch_one(&app.pool)
            .await
            .unwrap();

    assert_eq!(
        after,
        before + 1,
        "exactly one throttled server_error row expected"
    );

    let row: (String, Option<String>) = sqlx::query_as(
        "SELECT error_message, metadata FROM error_logs \
         WHERE company_id = ? AND error_type = 'server_error' \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&app.company_id)
    .fetch_one(&app.pool)
    .await
    .expect("fetch error log");

    assert!(!row.0.is_empty(), "error_message must be non-empty");
    let meta: Value = serde_json::from_str(row.1.as_deref().unwrap_or("{}")).unwrap();
    assert_eq!(meta["endpoint"], "/api/v1/contracts");
    assert_eq!(meta["method"], "GET");
    assert!(meta["status"].as_u64().unwrap_or(0) >= 500);
}

#[tokio::test]
async fn s138_pdf_font_family_loads_embedded_cjk() {
    let family = opc::pdf_fonts::pdf_font_family().expect("embedded font");
    assert!(family.regular.has_glyph('发'));
    assert!(family.bold.has_glyph('票'));
    assert!(family.regular.has_glyph('A'));
}

#[tokio::test]
async fn s138_invoice_pdf_chinese_content_is_nontrivial() {
    let app = common::TestApp::new().await;

    sqlx::query(
        r#"
        INSERT INTO invoices (
            id, company_id, invoice_number, invoice_type, amount, tax_amount,
            status, supplier, description, invoice_date, due_date
        ) VALUES (
            'pdf-zh-inv', ?, 'INV-中文-001', 'receivable', 1200.0, 72.0,
            'pending', '华为技术有限公司', '软件开发服务与技术支持',
            '2026-06-01', '2026-06-30'
        )
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed chinese invoice");

    // Give company a Chinese name so PDF header is CJK-heavy
    sqlx::query("UPDATE companies SET name = '星河智造科技有限公司' WHERE id = ?")
        .bind(&app.company_id)
        .execute(&app.pool)
        .await
        .ok();

    let res = app.get("/api/v1/invoices/pdf-zh-inv/pdf").await;
    res.assert_status_ok();
    let bytes = res.as_bytes();
    assert!(bytes.starts_with(b"%PDF"), "must be a PDF");
    // Embedded font subset makes Chinese PDFs much larger than blank shells.
    assert!(
        bytes.len() > 50_000,
        "PDF with embedded CJK font should exceed 50KB, got {}",
        bytes.len()
    );
}

#[tokio::test]
async fn s138_onboarding_rejects_disconnected_payment_and_payable_only() {
    let app = common::TestApp::new().await;

    // Payable-only invoice must NOT complete the invoices step.
    sqlx::query(
        r#"
        INSERT INTO invoices (
            id, company_id, invoice_number, invoice_type, amount, status, supplier
        ) VALUES ('iso-payable', ?, 'INV-P-001', 'payable', 500.0, 'pending', 'Vendor')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed payable invoice");

    // Orphan payment with no invoice_id must NOT complete the payments step.
    sqlx::query(
        r#"
        INSERT INTO payments (
            id, company_id, payment_no, payment_type, amount, status, payer, payment_date
        ) VALUES (
            'iso-pay', ?, 'PAY-ISO-001', 'income', 100.0, 'completed', 'Orphan', '2026-06-01'
        )
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed orphan payment");

    let res = app.get("/api/v1/onboarding/progress").await;
    res.assert_status_ok();
    let body = res.json::<Value>();
    let modules = body["modules"].as_array().unwrap();

    let invoices = modules
        .iter()
        .find(|m| m["id"] == "invoices")
        .expect("invoices module");
    assert_eq!(
        invoices["completed"].as_bool(),
        Some(false),
        "payable-only must not complete invoices step"
    );

    let payments = modules
        .iter()
        .find(|m| m["id"] == "payments")
        .expect("payments module");
    assert_eq!(
        payments["completed"].as_bool(),
        Some(false),
        "orphan payment must not complete payments step"
    );
    assert_eq!(payments["route"], "/invoices");

    // Link a receivable invoice + linked payment → both steps complete.
    sqlx::query(
        r#"
        INSERT INTO invoices (
            id, company_id, invoice_number, invoice_type, amount, status, supplier
        ) VALUES ('iso-recv', ?, 'INV-R-001', 'receivable', 800.0, 'pending', 'Client')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed receivable");

    sqlx::query(
        r#"
        INSERT INTO payments (
            id, company_id, payment_no, payment_type, amount, status, payer,
            invoice_id, payment_date
        ) VALUES (
            'iso-linked', ?, 'PAY-LINK-001', 'income', 800.0, 'completed', 'Client',
            'iso-recv', '2026-06-02'
        )
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed linked payment");

    let res2 = app.get("/api/v1/onboarding/progress").await;
    res2.assert_status_ok();
    let body2 = res2.json::<Value>();
    let modules2 = body2["modules"].as_array().unwrap();
    let invoices2 = modules2.iter().find(|m| m["id"] == "invoices").unwrap();
    let payments2 = modules2.iter().find(|m| m["id"] == "payments").unwrap();
    assert_eq!(invoices2["completed"].as_bool(), Some(true));
    assert_eq!(payments2["completed"].as_bool(), Some(true));

    // Tutorial copy must mention invoice linkage.
    let steps = body2["tutorial_steps"].as_array().unwrap();
    let pay_step = steps.iter().find(|s| s["id"] == "payments").unwrap();
    let desc = pay_step["description"].as_str().unwrap_or("");
    assert!(
        desc.contains("关联") || desc.contains("发票"),
        "tutorial must explain payment-invoice link: {desc}"
    );
}

#[tokio::test]
async fn s138_purge_old_error_logs_retains_recent() {
    let app = common::TestApp::new().await;

    sqlx::query(
        r#"
        INSERT INTO error_logs (id, company_id, error_type, error_message, created_at)
        VALUES
          ('old-err', ?, 'server_error', 'ancient', datetime('now', '-120 days')),
          ('new-err', ?, 'server_error', 'fresh', datetime('now'))
        "#,
    )
    .bind(&app.company_id)
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed error logs");

    let deleted = opc::error_log_mw::purge_old_error_logs(&app.pool, 90)
        .await
        .expect("purge");
    assert!(deleted >= 1);

    let remaining: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM error_logs WHERE id IN ('old-err', 'new-err')")
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!(remaining, 1);

    let kept: String = sqlx::query_scalar("SELECT id FROM error_logs WHERE id IN ('old-err', 'new-err')")
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(kept, "new-err");
}

// silence unused import warning if json! not used in all paths
#[allow(dead_code)]
fn _json_helper() -> Value {
    json!({})
}
