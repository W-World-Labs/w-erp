use crate::common;
use chrono::Duration;
use opc::app::{
    collections::run_collection_cycle, notifications::dispatch_digest_round,
    runtime::run_billing_cycle,
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

fn spawn_webhook_mock() -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind webhook mock");
    let address = listener.local_addr().expect("mock address");
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    std::thread::spawn(move || {
        while let Ok((mut stream, _)) = listener.accept() {
            let mut buffer = [0_u8; 16 * 1024];
            let read = stream.read(&mut buffer).unwrap_or(0);
            captured
                .lock()
                .expect("capture lock")
                .push(String::from_utf8_lossy(&buffer[..read]).to_string());
            let response = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}";
            let _ = stream.write_all(response.as_bytes());
        }
    });
    (format!("http://{address}/hook"), requests)
}

async fn seed_client(app: &common::TestApp, id: &str, name: &str) {
    sqlx::query(
        "INSERT INTO clients (id, company_id, name, client_type, email) \
         VALUES (?, ?, ?, 'active', 'finance@example.com')",
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
    .execute(&app.pool)
    .await
    .expect("seed client");
}

async fn seed_overdue_invoice(
    app: &common::TestApp,
    id: &str,
    number: &str,
    client_id: &str,
    days_overdue: i64,
    status: &str,
) {
    let due_date = (chrono::Local::now().date_naive() - Duration::days(days_overdue))
        .format("%Y-%m-%d")
        .to_string();
    sqlx::query(
        r#"
        INSERT INTO invoices (
            id, company_id, invoice_number, invoice_type, amount, status,
            invoice_date, due_date, supplier, client_id, created_by
        ) VALUES (?, ?, ?, 'receivable', 1234.56, ?, date(?, '-30 days'), ?, '收款客户', ?, ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(number)
    .bind(status)
    .bind(&due_date)
    .bind(&due_date)
    .bind(client_id)
    .bind(&app.user_id)
    .execute(&app.pool)
    .await
    .expect("seed invoice");
}

#[tokio::test]
async fn billing_cycle_sends_all_four_levels_once_and_exposes_history() {
    let (url, requests) = spawn_webhook_mock();
    let app = common::TestApp::new().await;
    seed_client(&app, "collection-client", "收款客户").await;
    sqlx::query(
        "INSERT INTO notification_channels \
         (id, company_id, name, channel_url, mode, enabled) \
         VALUES ('collection-instant', ?, '飞书催收群', ?, 'instant', 1)",
    )
    .bind(&app.company_id)
    .bind(&url)
    .execute(&app.pool)
    .await
    .expect("seed channel");

    for (index, days) in [1_i64, 7, 30, 60].into_iter().enumerate() {
        seed_overdue_invoice(
            &app,
            &format!("collection-invoice-{index}"),
            &format!("INV-COLLECT-{index}"),
            "collection-client",
            days,
            "pending",
        )
        .await;
    }

    run_billing_cycle(&app.pool).await.expect("billing cycle");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert_eq!(requests.lock().expect("requests").len(), 4);

    let levels: Vec<(String,)> =
        sqlx::query_as("SELECT level_code FROM collection_deliveries ORDER BY threshold_days")
            .fetch_all(&app.pool)
            .await
            .expect("delivery levels");
    assert_eq!(
        levels.into_iter().map(|row| row.0).collect::<Vec<_>>(),
        vec!["friendly", "formal", "final", "bad_debt"]
    );

    // Durable idempotency: an hourly retry does not touch the webhook again.
    run_billing_cycle(&app.pool)
        .await
        .expect("second billing cycle");
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert_eq!(requests.lock().expect("requests").len(), 4);
    let delivery_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM collection_deliveries")
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(delivery_count.0, 4);

    let history = app
        .get("/api/v1/collections/history?invoice_id=collection-invoice-3")
        .await
        .json::<Value>();
    assert_eq!(history["total"], 1);
    assert_eq!(history["history"][0]["level"], "bad_debt");
    assert_eq!(history["history"][0]["status"], "sent");

    let aging = app.get("/api/v1/ar/aging").await.json::<Value>();
    let invoice = aging["bucket_sections"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|bucket| bucket["invoices"].as_array().unwrap())
        .find(|invoice| invoice["id"] == "collection-invoice-3")
        .expect("invoice in aging");
    assert_eq!(invoice["collection_history_count"], 1);
    assert_eq!(invoice["latest_collection"]["status_label"], "已送达");

    let audit = app
        .get("/api/v1/operation_logs/overview?table=collection_deliveries")
        .await
        .json::<Value>();
    assert_eq!(audit["logs"].as_array().unwrap().len(), 4);
    assert_eq!(audit["logs"][0]["table_label"], "自动催收");

    let clients = app.get("/api/v1/clients/overview").await.json::<Value>();
    let credit = &clients["clients"][0]["credit_profile"];
    assert!(credit["score"].as_i64().unwrap() < 100);
    assert_eq!(credit["collection_event_count"], 4);
}

#[tokio::test]
async fn disabled_switch_produces_zero_delivery_and_zero_history() {
    let (url, requests) = spawn_webhook_mock();
    let app = common::TestApp::new().await;
    seed_client(&app, "disabled-client", "收款客户").await;
    sqlx::query(
        "INSERT INTO notification_channels \
         (id, company_id, name, channel_url, mode, enabled) \
         VALUES ('disabled-channel', ?, '催收群', ?, 'instant', 1)",
    )
    .bind(&app.company_id)
    .bind(&url)
    .execute(&app.pool)
    .await
    .unwrap();
    let response = app
        .put("/api/v1/settings/collections")
        .json(&json!({
            "enabled": false,
            "friendly_days": 1,
            "formal_days": 7,
            "final_days": 30,
            "bad_debt_days": 60
        }))
        .await;
    assert!(response.status_code().is_success());
    seed_overdue_invoice(
        &app,
        "disabled-invoice",
        "INV-DISABLED",
        "disabled-client",
        30,
        "pending",
    )
    .await;

    run_billing_cycle(&app.pool).await.expect("billing cycle");
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert!(requests.lock().expect("requests").is_empty());
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM collection_deliveries")
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(count.0, 0);
}

#[tokio::test]
async fn digest_channel_queues_then_delivers_through_existing_dispatcher() {
    let (url, requests) = spawn_webhook_mock();
    let app = common::TestApp::new().await;
    seed_client(&app, "digest-client", "收款客户").await;
    sqlx::query(
        "INSERT INTO notification_channels \
         (id, company_id, name, channel_url, mode, enabled) \
         VALUES ('collection-digest', ?, '每日摘要', ?, 'digest', 1)",
    )
    .bind(&app.company_id)
    .bind(&url)
    .execute(&app.pool)
    .await
    .unwrap();
    seed_overdue_invoice(
        &app,
        "digest-invoice",
        "INV-DIGEST",
        "digest-client",
        7,
        "overdue",
    )
    .await;

    let summary = run_collection_cycle(&app.pool).await.unwrap();
    assert_eq!(summary.queued, 1);
    assert_eq!(summary.sent, 0);
    assert!(requests.lock().expect("requests").is_empty());

    let failures = Arc::new(Mutex::new(HashMap::new()));
    assert_eq!(
        dispatch_digest_round(&app.pool, &failures).await.unwrap(),
        1
    );
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let requests = requests.lock().expect("requests");
    assert_eq!(requests.len(), 1);
    assert!(requests[0].contains("INV-DIGEST"));
    drop(requests);
    let status: (String,) = sqlx::query_as(
        "SELECT status FROM collection_deliveries WHERE invoice_id = 'digest-invoice'",
    )
    .fetch_one(&app.pool)
    .await
    .unwrap();
    assert_eq!(status.0, "sent");
}

#[tokio::test]
async fn disabling_after_digest_queue_prevents_the_pending_touch() {
    let (url, requests) = spawn_webhook_mock();
    let app = common::TestApp::new().await;
    seed_client(&app, "queued-disabled-client", "收款客户").await;
    sqlx::query(
        "INSERT INTO notification_channels \
         (id, company_id, name, channel_url, mode, enabled) \
         VALUES ('queued-disabled-digest', ?, '每日摘要', ?, 'digest', 1)",
    )
    .bind(&app.company_id)
    .bind(&url)
    .execute(&app.pool)
    .await
    .unwrap();
    seed_overdue_invoice(
        &app,
        "queued-disabled-invoice",
        "INV-QUEUED-DISABLED",
        "queued-disabled-client",
        7,
        "overdue",
    )
    .await;
    run_collection_cycle(&app.pool).await.unwrap();
    app.put("/api/v1/settings/collections")
        .json(&json!({
            "enabled": false,
            "friendly_days": 1,
            "formal_days": 7,
            "final_days": 30,
            "bad_debt_days": 60
        }))
        .await;

    let failures = Arc::new(Mutex::new(HashMap::new()));
    assert_eq!(
        dispatch_digest_round(&app.pool, &failures).await.unwrap(),
        0
    );
    assert!(requests.lock().expect("requests").is_empty());
    let status: (String,) = sqlx::query_as(
        "SELECT status FROM collection_deliveries WHERE invoice_id = 'queued-disabled-invoice'",
    )
    .fetch_one(&app.pool)
    .await
    .unwrap();
    assert_eq!(status.0, "pending");
}
