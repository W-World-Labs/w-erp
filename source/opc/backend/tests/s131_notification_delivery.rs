// #11 reminder notification delivery via w-notify / webhook.
use crate::common;
use axum::http::StatusCode;
use opc::app::notifications::{dispatch_instant_round, format_digest_message};
use serde_json::json;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

fn spawn_webhook_mock() -> (String, Arc<Mutex<usize>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let hits = Arc::new(Mutex::new(0usize));
    let hits_c = hits.clone();
    thread::spawn(move || {
        for _ in 0..8 {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 8192];
                let _ = stream.read(&mut buf);
                *hits_c.lock().unwrap() += 1;
                let body = "{}";
                let resp = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
                let _ = stream.write_all(resp.as_bytes());
            }
        }
    });
    (format!("http://{addr}/hook"), hits)
}

#[tokio::test]
async fn instant_dispatch_posts_webhook_and_sets_notified_at() {
    let (url, hits) = spawn_webhook_mock();
    let app = common::TestApp::new().await;
    let company_id = &app.company_id;
    sqlx::query("INSERT INTO notification_channels (id, company_id, name, channel_url, mode, enabled) VALUES ('ch1', ?, 'Test', ?, 'instant', 1)")
        .bind(company_id).bind(&url).execute(&app.pool).await.unwrap();
    sqlx::query("INSERT INTO reminders (id, company_id, title, description, status, due_date) VALUES ('r1', ?, '合同即将到期: X', '[contract_expiry]c1', 'pending', date('now'))")
        .bind(company_id).execute(&app.pool).await.unwrap();
    let failures = Arc::new(Mutex::new(HashMap::new()));
    let sent = dispatch_instant_round(&app.pool, &failures).await.unwrap();
    assert!(sent >= 1);
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert!(*hits.lock().unwrap() >= 1);
    let notified: (Option<String>,) = sqlx::query_as("SELECT notified_at FROM reminders WHERE id = 'r1'").fetch_one(&app.pool).await.unwrap();
    assert!(notified.0.is_some());
}

#[tokio::test]
async fn failed_send_leaves_notified_at_null() {
    let app = common::TestApp::new().await;
    sqlx::query("INSERT INTO notification_channels (id, company_id, name, channel_url, mode, enabled) VALUES ('ch-fail', ?, 'Fail', 'http://127.0.0.1:9/hook', 'instant', 1)")
        .bind(&app.company_id).execute(&app.pool).await.unwrap();
    sqlx::query("INSERT INTO reminders (id, company_id, title, status, due_date) VALUES ('r-fail', ?, '测试提醒', 'pending', date('now'))")
        .bind(&app.company_id).execute(&app.pool).await.unwrap();
    let failures = Arc::new(Mutex::new(HashMap::new()));
    let _ = dispatch_instant_round(&app.pool, &failures).await;
    let notified: (Option<String>,) = sqlx::query_as("SELECT notified_at FROM reminders WHERE id = 'r-fail'").fetch_one(&app.pool).await.unwrap();
    assert!(notified.0.is_none());
}

#[tokio::test]
async fn notification_channel_crud_and_test_endpoint() {
    let (url, hits) = spawn_webhook_mock();
    let app = common::TestApp::new().await;
    let create = app.post("/api/v1/notification_channels").json(&json!({"name":"飞书","channel_url":url,"mode":"digest","enabled":true})).await;
    assert_eq!(create.status_code(), StatusCode::CREATED);
    let id = create.json::<serde_json::Value>()["id"].as_str().unwrap().to_string();
    let test = app.post(&format!("/api/v1/settings/notifications/{id}/test")).json(&json!({})).await;
    assert_eq!(test.status_code(), StatusCode::OK);
    assert_eq!(test.json::<serde_json::Value>()["ok"], true);
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert!(*hits.lock().unwrap() >= 1);
    assert_eq!(app.delete(&format!("/api/v1/notification_channels/{id}")).await.status_code(), StatusCode::NO_CONTENT);
}

#[test]
fn digest_message_assembles_from_fixture() {
    let items = vec![
        ("1".into(), "合同即将到期: A".into(), Some("[contract_expiry]a".into()), Some("2026-08-05".into())),
        ("2".into(), "证照即将到期: B".into(), None, Some("2026-08-06".into())),
    ];
    let body = format_digest_message(&items, 2);
    assert!(body.contains("今日待处理提醒 2 条"));
    assert!(body.contains("逾期发票 2 张"));
    let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8987".to_owned());
    let reminders_url = format!("{}/reminders", app_url.trim_end_matches('/'));
    assert!(body.contains(reminders_url.as_str()));
}
