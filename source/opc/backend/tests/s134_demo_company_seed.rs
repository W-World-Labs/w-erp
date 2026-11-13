use crate::common;

use axum_test::TestServer;
use opc::{build_app, db};
use serde_json::Value;
use sqlx::SqlitePool;

async fn fresh_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("create in-memory database");
    db::run_migrations(&pool).await.expect("run migrations");
    pool
}

async fn ensure_company_for_new_user(pool: &SqlitePool, email: &str) -> (String, String) {
    let user_id = common::seed::create_user(pool, email, "Demo User").await;
    let company_id = opc::api::companies::ensure_default_company(pool, &user_id, email)
        .await
        .expect("ensure default company");
    (user_id, company_id)
}

async fn scoped_count(pool: &SqlitePool, table: &str, company_id: &str) -> i64 {
    let sql = format!("SELECT COUNT(*) FROM {table} WHERE company_id = ?");
    sqlx::query_scalar::<_, i64>(&sql)
        .bind(company_id)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|err| panic!("count {table}: {err}"))
}

async fn demo_counts(pool: &SqlitePool, company_id: &str) -> Vec<(&'static str, i64)> {
    let tables = [
        "company_info",
        "branches",
        "rent_records",
        "licenses",
        "employees",
        "assets",
        "invoices",
        "salary_records",
        "projects",
        "budgets",
        "contracts",
        "clients",
        "contacts",
        "deals",
        "quotes",
        "bank_accounts",
        "payments",
        "time_entries",
        "applications",
        "material_library",
        "cloud_bills",
        "software_subscriptions",
        "tax_deadlines",
        "insurance_policies",
    ];

    let mut counts = Vec::new();
    for table in tables {
        counts.push((table, scoped_count(pool, table, company_id).await));
    }
    counts
}

#[tokio::test]
async fn s134_new_user_receives_personal_demo_company_for_tutorial() {
    let pool = fresh_pool().await;
    let (user_id, company_id) = ensure_company_for_new_user(&pool, "first-demo@opc.test").await;

    let (name, legal_name, base_currency, role, is_default): (
        String,
        Option<String>,
        String,
        String,
        i64,
    ) = sqlx::query_as(
        r#"
        SELECT c.name, c.legal_name, c.base_currency, cm.role, cm.is_default
        FROM companies c
        JOIN company_members cm ON cm.company_id = c.id
        WHERE c.id = ? AND cm.user_id = ?
        "#,
    )
    .bind(&company_id)
    .bind(&user_id)
    .fetch_one(&pool)
    .await
    .expect("demo company row");

    assert!(name.contains("Demo"));
    assert!(name.contains("语数科技"));
    assert_eq!(legal_name.as_deref(), Some("虚构示例科技有限公司"));
    assert_eq!(base_currency, "CNY");
    assert_eq!(role, "owner");
    assert_eq!(is_default, 1);

    let empty_company_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM companies WHERE name = 'first-demo的公司'",
    )
    .fetch_one(&pool)
    .await
    .expect("empty company count");
    assert_eq!(empty_company_count, 0);

    let session_token = common::seed::create_session(&pool, &user_id, &company_id).await;
    let server = TestServer::new(build_app(pool.clone())).expect("test server");
    let response = server
        .get("/api/v1/onboarding/progress")
        .authorization_bearer(&session_token)
        .await;
    response.assert_status_ok();
    let progress = response.json::<Value>();

    assert_eq!(progress["summary"]["completed_count"].as_i64(), Some(15));
    assert_eq!(progress["summary"]["total_count"].as_i64(), Some(15));
    assert_eq!(progress["summary"]["all_completed"].as_bool(), Some(true));
    assert_eq!(progress["summary"]["demo_company"].as_bool(), Some(true));
    assert_eq!(progress["summary"]["tutorial_mode"], "demo_walkthrough");
    assert_eq!(
        progress["summary"]["tutorial_storage_key"],
        format!("onboarding-tutorial-dismissed:{company_id}")
    );
    assert_eq!(progress["summary"]["progress_rate_label"], "100%");
    assert!(
        progress["tutorial_steps"]
            .as_array()
            .expect("tutorial steps")
            .iter()
            .all(|step| step["completed"].as_bool() == Some(true))
    );

    for (table, count) in demo_counts(&pool, &company_id).await {
        assert!(count > 0, "{table} should have demo records");
    }
}

#[tokio::test]
async fn s134_demo_company_seed_is_idempotent_for_repeat_login() {
    let pool = fresh_pool().await;
    let user_id = common::seed::create_user(&pool, "repeat-demo@opc.test", "Repeat Demo").await;

    let first_company_id =
        opc::api::companies::ensure_default_company(&pool, &user_id, "repeat-demo@opc.test")
            .await
            .expect("first ensure");
    let first_counts = demo_counts(&pool, &first_company_id).await;

    let second_company_id =
        opc::api::companies::ensure_default_company(&pool, &user_id, "repeat-demo@opc.test")
            .await
            .expect("second ensure");
    let second_counts = demo_counts(&pool, &second_company_id).await;

    assert_eq!(first_company_id, second_company_id);
    assert_eq!(first_counts, second_counts);

    let member_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM company_members WHERE user_id = ? AND company_id = ?",
    )
    .bind(&user_id)
    .bind(&first_company_id)
    .fetch_one(&pool)
    .await
    .expect("member count");
    assert_eq!(member_count, 1);
}

#[tokio::test]
async fn s134_each_user_gets_an_isolated_demo_company_copy() {
    let pool = fresh_pool().await;

    let (alice_user_id, alice_company_id) =
        ensure_company_for_new_user(&pool, "alice-demo@opc.test").await;
    let (bob_user_id, bob_company_id) =
        ensure_company_for_new_user(&pool, "bob-demo@opc.test").await;

    assert_ne!(alice_company_id, bob_company_id);

    let alice_counts = demo_counts(&pool, &alice_company_id).await;
    let bob_counts = demo_counts(&pool, &bob_company_id).await;
    assert_eq!(alice_counts, bob_counts);

    let cross_memberships = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM company_members
        WHERE (user_id = ? AND company_id = ?)
           OR (user_id = ? AND company_id = ?)
        "#,
    )
    .bind(&alice_user_id)
    .bind(&bob_company_id)
    .bind(&bob_user_id)
    .bind(&alice_company_id)
    .fetch_one(&pool)
    .await
    .expect("cross membership count");
    assert_eq!(cross_memberships, 0);
}
