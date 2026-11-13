/// Seed data factories for E2E tests
use sqlx::SqlitePool;

/// Create a test user, returns user_id
pub async fn create_user(pool: &SqlitePool, email: &str, name: &str) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO users (id, email, name, created_at, updated_at) VALUES (?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
    )
    .bind(&id)
    .bind(email)
    .bind(name)
    .execute(pool)
    .await
    .expect("Failed to seed user");
    id
}

/// Create a test company with membership, returns company_id
pub async fn create_company(
    pool: &SqlitePool,
    user_id: &str,
    name: &str,
    currency: &str,
) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO companies (id, name, created_at, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
    )
    .bind(&id)
    .bind(name)
    .execute(pool)
    .await
    .expect("Failed to seed company");

    // Set base_currency (migration column)
    let _ = sqlx::query("UPDATE companies SET base_currency = ? WHERE id = ?")
        .bind(currency)
        .bind(&id)
        .execute(pool)
        .await;

    // Add user as owner
    let member_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO company_members (id, user_id, company_id, role, is_default, created_at) VALUES (?, ?, ?, 'owner', 1, CURRENT_TIMESTAMP)"
    )
    .bind(&member_id)
    .bind(user_id)
    .bind(&id)
    .execute(pool)
    .await
    .expect("Failed to seed company member");

    id
}

/// Create a session for testing, returns session_token
pub async fn create_session(pool: &SqlitePool, user_id: &str, company_id: &str) -> String {
    let session_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO sessions (id, user_id, access_token, current_company_id, expires_at) VALUES (?, ?, 'test-token', ?, datetime('now', '+1 day'))"
    )
    .bind(&session_id)
    .bind(user_id)
    .bind(company_id)
    .execute(pool)
    .await
    .expect("Failed to seed session");
    session_id
}

/// Create an employee, returns employee_id
pub async fn create_employee(
    pool: &SqlitePool,
    company_id: &str,
    name: &str,
    salary: f64,
) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO employees (id, company_id, name, position, department, base_salary, status, created_at, updated_at) VALUES (?, ?, ?, 'Engineer', 'Engineering', ?, 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
    )
    .bind(&id)
    .bind(company_id)
    .bind(name)
    .bind(salary)
    .execute(pool)
    .await
    .expect("Failed to seed employee");
    id
}

/// Create an asset, returns asset_id
pub async fn create_asset(pool: &SqlitePool, company_id: &str, name: &str, price: f64) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO assets (id, company_id, name, asset_type, purchase_price, status, created_at, updated_at) VALUES (?, ?, ?, 'physical', ?, 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
    )
    .bind(&id)
    .bind(company_id)
    .bind(name)
    .bind(price)
    .execute(pool)
    .await
    .expect("Failed to seed asset");
    id
}
