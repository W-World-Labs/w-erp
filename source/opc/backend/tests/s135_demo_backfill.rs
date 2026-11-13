use crate::common;

use opc::db;
use sqlx::SqlitePool;

async fn fresh_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("create in-memory database");
    db::run_migrations(&pool).await.expect("run migrations");
    pool
}

async fn demo_company_count_for(pool: &SqlitePool, user_id: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM company_members cm
        JOIN companies c ON c.id = cm.company_id
        WHERE cm.user_id = ?
          AND c.is_demo = 1
          AND COALESCE(c.is_deleted, 0) = 0
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .expect("count demo companies")
}

#[tokio::test]
async fn s135_new_user_demo_company_is_flagged_is_demo() {
    let pool = fresh_pool().await;
    let user_id = common::seed::create_user(&pool, "newdemo@opc.test", "New User").await;
    let _ = opc::api::companies::ensure_default_company(&pool, &user_id, "newdemo@opc.test")
        .await
        .expect("ensure default company");

    assert_eq!(
        demo_company_count_for(&pool, &user_id).await,
        1,
        "a new user's seeded demo company must be flagged is_demo = 1"
    );
}

#[tokio::test]
async fn s135_old_user_with_real_company_is_backfilled_on_login() {
    let pool = fresh_pool().await;
    // Simulate a PRE-Demo-feature user: already owns a real (non-demo) company
    // and has never received a demo company.
    let user_id = common::seed::create_user(&pool, "old@opc.test", "Old User").await;
    let real_company = common::seed::create_company(&pool, &user_id, "Real Co", "CNY").await;
    assert_eq!(
        demo_company_count_for(&pool, &user_id).await,
        0,
        "old user starts with no demo company"
    );

    // Their next login runs ensure_default_company again.
    let returned =
        opc::api::companies::ensure_default_company(&pool, &user_id, "old@opc.test")
            .await
            .expect("ensure default company");

    assert_eq!(
        returned, real_company,
        "login must keep the old user's real company as current — not switch them to the demo"
    );
    assert_eq!(
        demo_company_count_for(&pool, &user_id).await,
        1,
        "old user must be backfilled with a demo company on login"
    );
}

#[tokio::test]
async fn s135_migration_does_not_flag_a_real_company_sharing_the_demo_name() {
    let pool = fresh_pool().await;
    // A real (non-demo) company that happens to share the demo display name but
    // has its own real credentials. The is_demo flag-backfill migration must NOT
    // reclassify it as a demo (a user could rename their company to this name).
    sqlx::query(
        "INSERT INTO companies (id, name, unified_social_credit_code) VALUES ('real-1', ?, '91440300REALCORP')",
    )
    .bind(opc::db::demo_seed::DEMO_COMPANY_NAME)
    .execute(&pool)
    .await
    .expect("insert real company");

    // Re-run migrations (idempotent) to trigger the is_demo flag-backfill.
    db::run_migrations(&pool).await.expect("re-run migrations");

    let is_demo: i64 = sqlx::query_scalar("SELECT is_demo FROM companies WHERE id = 'real-1'")
        .fetch_one(&pool)
        .await
        .expect("read is_demo");
    assert_eq!(
        is_demo, 0,
        "a real company sharing the demo NAME must not be flagged is_demo"
    );
}

#[tokio::test]
async fn s135_migration_flags_legacy_demo_company_by_fingerprint() {
    let pool = fresh_pool().await;
    // A legacy demo company created before is_demo existed: it carries the demo's
    // fake unified social credit code. The migration must backfill its flag.
    sqlx::query(
        "INSERT INTO companies (id, name, unified_social_credit_code, is_demo) VALUES ('legacy-demo', ?, '91440300DEMOYUSHU', 0)",
    )
    .bind(opc::db::demo_seed::DEMO_COMPANY_NAME)
    .execute(&pool)
    .await
    .expect("insert legacy demo company");

    db::run_migrations(&pool).await.expect("re-run migrations");

    let is_demo: i64 = sqlx::query_scalar("SELECT is_demo FROM companies WHERE id = 'legacy-demo'")
        .fetch_one(&pool)
        .await
        .expect("read is_demo");
    assert_eq!(
        is_demo, 1,
        "a legacy demo company (matched by fingerprint) must be backfilled to is_demo = 1"
    );
}

#[tokio::test]
async fn s135_migration_adds_missing_company_columns_on_legacy_db() {
    // A real prod DB created before the i18n + is_demo columns were added: the
    // companies table is missing country/timezone/locale/base_currency/is_demo.
    // INIT_SQL's CREATE TABLE IF NOT EXISTS will NOT add columns to it, so the
    // demo seed (which INSERTs those columns) would fail unless run_migrations
    // ALTER-adds them. This guards the headline old-user backfill on legacy DBs.
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("create db");
    sqlx::query(
        "CREATE TABLE companies (\
            id TEXT PRIMARY KEY, name TEXT NOT NULL, legal_name TEXT, tax_id TEXT, \
            registration_number TEXT, address TEXT, city TEXT, province TEXT, postal_code TEXT, \
            phone TEXT, email TEXT, website TEXT, industry TEXT, founded_date TEXT, description TEXT, \
            logo_url TEXT, legal_representative TEXT, unified_social_credit_code TEXT, \
            registered_capital REAL, created_at TEXT, updated_at TEXT)",
    )
    .execute(&pool)
    .await
    .expect("create legacy companies table");

    db::run_migrations(&pool).await.expect("migrations on legacy db");

    // The demo seed INSERTs country/timezone/locale/base_currency/is_demo — it
    // must succeed now that the migration backfilled the missing columns.
    let user_id = common::seed::create_user(&pool, "legacy@opc.test", "Legacy").await;
    opc::api::companies::ensure_default_company(&pool, &user_id, "legacy@opc.test")
        .await
        .expect("demo seed must succeed after legacy-db column migration");
    assert_eq!(
        demo_company_count_for(&pool, &user_id).await,
        1,
        "legacy-DB user must still receive a demo company"
    );
}

#[tokio::test]
async fn s135_backfill_all_users_is_idempotent() {
    let pool = fresh_pool().await;
    let u1 = common::seed::create_user(&pool, "a@opc.test", "A").await;
    let _ = common::seed::create_company(&pool, &u1, "A Co", "CNY").await;
    let u2 = common::seed::create_user(&pool, "b@opc.test", "B").await;
    let _ = common::seed::create_company(&pool, &u2, "B Co", "CNY").await;

    let first = db::demo_seed::backfill_demo_for_all_users(&pool)
        .await
        .expect("backfill first pass");
    assert_eq!(first, 2, "first backfill seeds a demo company for both users");

    let second = db::demo_seed::backfill_demo_for_all_users(&pool)
        .await
        .expect("backfill second pass");
    assert_eq!(second, 0, "second backfill is a no-op (idempotent)");

    assert_eq!(demo_company_count_for(&pool, &u1).await, 1);
    assert_eq!(demo_company_count_for(&pool, &u2).await, 1);
}
