/// Test infrastructure for OPC E2E scenarios
/// Each test gets its own in-memory SQLite database with full app routing.
use axum_test::TestServer;
use opc::{build_app, db};
use serde_json::{Value, json};
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{str::FromStr, time::Duration};

/// SQLx gives each `sqlite::memory:` pool a uniquely named shared-cache
/// database. A busy timeout is still required because request handlers and
/// background audit writes can briefly contend for SQLite's single writer.
pub async fn memory_pool() -> SqlitePool {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")
        .expect("valid in-memory SQLite URL")
        .busy_timeout(Duration::from_secs(30));
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .expect("Failed to create in-memory database")
}

pub mod seed;

/// Global lock serializing tests that mutate process-wide env vars (AI /
/// document parser backends). Env mutation is process-global, so without this
/// lock parallel tests observe each other's configuration.
pub fn env_guard() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Complete test environment with authenticated session
pub struct TestApp {
    pub server: TestServer,
    pub pool: SqlitePool,
    pub user_id: String,
    pub company_id: String,
    pub session_token: String,
}

impl TestApp {
    /// Create a fresh test environment:
    /// - In-memory SQLite with full schema
    /// - Seeded user + company + session
    /// - Full app router with auth middleware
    pub async fn new() -> Self {
        Self::with_company("TestCo", "test@opc.test", "CNY").await
    }

    /// Create test env with custom company settings
    pub async fn with_company(company_name: &str, email: &str, currency: &str) -> Self {
        let pool = memory_pool().await;

        // Run full schema + migrations
        db::run_migrations(&pool)
            .await
            .expect("Failed to run migrations");

        // Seed test data
        let user_id = seed::create_user(&pool, email, "Test User").await;
        let company_id = seed::create_company(&pool, &user_id, company_name, currency).await;
        let session_token = seed::create_session(&pool, &user_id, &company_id).await;

        // Build full app
        let app = build_app(pool.clone());
        let server = TestServer::new(app).expect("Failed to create test server");

        Self {
            server,
            pool,
            user_id,
            company_id,
            session_token,
        }
    }

    /// Create a second tenant (different user + company) sharing the same server
    pub async fn create_second_tenant(&self) -> TestApp {
        let user_id = seed::create_user(&self.pool, "other@rival-com.example.invalid", "Rival User").await;
        let company_id = seed::create_company(&self.pool, &user_id, "RivalCo", "USD").await;
        let session_token = seed::create_session(&self.pool, &user_id, &company_id).await;

        let app = build_app(self.pool.clone());
        let server = TestServer::new(app).expect("Failed to create test server");

        TestApp {
            server,
            pool: self.pool.clone(),
            user_id,
            company_id,
            session_token,
        }
    }

    /// Create a second company for the same user (multi-company OPC scenario)
    pub async fn create_second_company(&self, name: &str, currency: &str) -> (String, String) {
        let company_id = seed::create_company(&self.pool, &self.user_id, name, currency).await;
        let session_token = seed::create_session(&self.pool, &self.user_id, &company_id).await;
        (company_id, session_token)
    }

    // ─── Authenticated request helpers ───

    pub fn get(&self, url: &str) -> axum_test::TestRequest {
        self.server
            .get(url)
            .authorization_bearer(&self.session_token)
    }

    pub fn post(&self, url: &str) -> axum_test::TestRequest {
        self.server
            .post(url)
            .authorization_bearer(&self.session_token)
    }

    pub fn put(&self, url: &str) -> axum_test::TestRequest {
        self.server
            .put(url)
            .authorization_bearer(&self.session_token)
    }

    pub fn delete(&self, url: &str) -> axum_test::TestRequest {
        self.server
            .delete(url)
            .authorization_bearer(&self.session_token)
    }
}
