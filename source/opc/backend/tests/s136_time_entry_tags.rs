use crate::common::TestApp;
use serde_json::json;
use sqlx::SqlitePool;

async fn seed_time_entry(pool: &SqlitePool, company_id: &str) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO time_entries (id, company_id, description, date) VALUES (?, ?, 'work', '2026-05-29')",
    )
    .bind(&id)
    .bind(company_id)
    .execute(pool)
    .await
    .expect("seed time_entry");
    id
}

/// The frontend (lib/api/contracts.ts + e2e contract tests) models
/// `time_entry_tags` as a real child-of-time_entries resource at
/// `/api/v1/time_entry_tags`, parallel to quotes -> quote_items. The backend
/// was missing this route, so the path was a true 404 (a frontend<->backend
/// contract gap). This asserts the route now exists and lists for the tenant.
#[tokio::test]
async fn s136_time_entry_tags_route_exists_and_lists() {
    let app = TestApp::new().await;

    let response = app.get("/api/v1/time_entry_tags").await;
    assert_eq!(
        response.status_code(),
        200,
        "GET /api/v1/time_entry_tags must exist (200), not 404"
    );
}

#[tokio::test]
async fn s136_create_tag_with_valid_time_entry_succeeds() {
    let app = TestApp::new().await;
    let entry_id = seed_time_entry(&app.pool, &app.company_id).await;

    let response = app
        .post("/api/v1/time_entry_tags")
        .json(&json!({ "time_entry_id": entry_id, "tag": "billable" }))
        .await;

    assert!(
        response.status_code().is_success(),
        "creating a tag for an in-tenant time entry should succeed, got {}",
        response.status_code()
    );
}

#[tokio::test]
async fn s136_create_tag_with_nonexistent_time_entry_is_rejected() {
    let app = TestApp::new().await;

    let response = app
        .post("/api/v1/time_entry_tags")
        .json(&json!({ "time_entry_id": "does-not-exist", "tag": "x" }))
        .await;

    assert!(
        response.status_code().is_client_error(),
        "a tag referencing a non-existent time_entry must be rejected (4xx), got {}",
        response.status_code()
    );
}

#[tokio::test]
async fn s136_create_tag_referencing_other_tenant_time_entry_is_rejected() {
    let app = TestApp::new().await;
    let other = app.create_second_tenant().await;
    // A time entry that belongs to the OTHER tenant.
    let foreign_entry = seed_time_entry(&other.pool, &other.company_id).await;

    let response = app
        .post("/api/v1/time_entry_tags")
        .json(&json!({ "time_entry_id": foreign_entry, "tag": "leak" }))
        .await;

    assert!(
        response.status_code().is_client_error(),
        "must not let tenant A attach a tag to tenant B's time entry (cross-tenant), got {}",
        response.status_code()
    );
}
