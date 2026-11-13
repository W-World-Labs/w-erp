use super::error::{ApiError, api_err};
use crate::auth::AuthUser;
use crate::db::models::BankAccount;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{Row, SqlitePool};

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/accounts", get(list_accounts).post(create_account))
        .route("/accounts/overview", get(accounts_overview))
        .route(
            "/accounts/{id}",
            get(get_account).put(update_account).delete(delete_account),
        )
        .route("/transactions", post(create_transaction))
        .with_state(pool)
}

#[derive(Deserialize)]
pub struct CreateAccountRequest {
    pub bank_name: String,
    pub account_name: String,
    pub account_number: String,
    pub account_type: Option<String>,
    pub balance: Option<f64>,
    pub currency: Option<String>,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateTransactionRequest {
    pub account_id: String,
    pub transaction_type: String, // "deposit" | "withdrawal" | "transfer"
    pub amount: f64,
    pub counterparty: Option<String>,
    pub description: Option<String>,
    pub transaction_date: String,
}

#[derive(Serialize)]
pub struct AccountsResponse {
    pub data: Vec<BankAccount>,
    pub count: i64,
}

async fn list_accounts(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(q): Query<super::generic::ListQuery>,
) -> Result<Json<AccountsResponse>, ApiError> {
    let limit = q.limit.or(q.page_size).unwrap_or(100).min(1000).max(1);
    let offset = q
        .offset
        .or_else(|| q.page.map(|p| (p.max(1) - 1) * limit))
        .unwrap_or(0)
        .max(0);
    let accounts: Vec<BankAccount> = sqlx::query_as(
        "SELECT * FROM bank_accounts WHERE company_id = ? ORDER BY bank_name LIMIT ? OFFSET ?",
    )
    .bind(&user.company_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM bank_accounts WHERE company_id = ?")
            .bind(&user.company_id)
            .fetch_one(&pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(Json(AccountsResponse {
        data: accounts,
        count,
    }))
}

async fn accounts_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<Value>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT
            ba.id,
            ba.bank_name,
            ba.account_name,
            ba.account_number,
            ba.account_type,
            ba.balance,
            ba.currency,
            ba.status,
            ba.notes,
            ba.created_at,
            ba.updated_at,
            COUNT(bt.id) AS transaction_count,
            COALESCE(SUM(CASE
                WHEN bt.transaction_type IN ('credit', 'deposit', 'income') THEN COALESCE(bt.amount, 0.0)
                ELSE 0.0
            END), 0.0) AS transaction_income,
            COALESCE(SUM(CASE
                WHEN bt.id IS NOT NULL AND bt.transaction_type NOT IN ('credit', 'deposit', 'income') THEN COALESCE(bt.amount, 0.0)
                ELSE 0.0
            END), 0.0) AS transaction_expense
        FROM bank_accounts ba
        LEFT JOIN bank_transactions bt ON bt.account_id = ba.id AND bt.company_id = ba.company_id
        WHERE ba.company_id = ?
        GROUP BY ba.id
        ORDER BY ba.bank_name ASC, ba.account_name ASC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let mut total_balance = 0.0;
    let mut active_accounts = 0_i64;
    let mut accounts = Vec::new();

    for row in rows {
        let balance = number(&row, "balance");
        let status = text(&row, "status").unwrap_or_else(|| "active".to_string());
        let account_type = text(&row, "account_type").unwrap_or_else(|| "checking".to_string());
        let income = number(&row, "transaction_income");
        let expense = number(&row, "transaction_expense");

        total_balance += balance;
        if status == "active" {
            active_accounts += 1;
        }

        accounts.push(json!({
            "id": text(&row, "id"),
            "bank_name": text(&row, "bank_name"),
            "account_name": text(&row, "account_name"),
            "account_number": text(&row, "account_number"),
            "account_type": account_type.clone(),
            "account_type_label": bank_account_type_label(&account_type),
            "balance": round_money(balance),
            "balance_label": format_cny(balance),
            "currency": text(&row, "currency").unwrap_or_else(|| "CNY".to_string()),
            "status": status.clone(),
            "status_badge": bank_account_status_badge(&status),
            "notes": text(&row, "notes"),
            "created_at": text(&row, "created_at"),
            "updated_at": text(&row, "updated_at"),
            "transaction_stats": {
                "count": integer(&row, "transaction_count"),
                "income": round_money(income),
                "expense": round_money(expense),
                "income_label": format_cny(income),
                "expense_label": format_cny(expense),
            },
        }));
    }

    Ok(Json(json!({
        "stats": {
            "total_balance": round_money(total_balance),
            "total_balance_label": format_cny(total_balance),
            "active_accounts": active_accounts,
            "total_accounts": accounts.len() as i64,
        },
        "accounts": accounts,
    })))
}

async fn get_account(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<BankAccount>, ApiError> {
    let account: Option<BankAccount> =
        sqlx::query_as("SELECT * FROM bank_accounts WHERE id = ? AND company_id = ?")
            .bind(&id)
            .bind(&user.company_id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    account
        .map(Json)
        .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "account not found"))
}

async fn create_account(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Json(req): Json<CreateAccountRequest>,
) -> Result<Json<BankAccount>, ApiError> {
    // R2-3: Validate non-negative initial balance
    if let Some(bal) = req.balance
        && bal < 0.0
    {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "balance must be non-negative",
        ));
    }

    let id = uuid::Uuid::new_v4().to_string();

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    sqlx::query(
        "INSERT INTO bank_accounts (id, company_id, bank_name, account_name, account_number, account_type, balance, currency, notes) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&user.company_id)
    .bind(&req.bank_name)
    .bind(&req.account_name)
    .bind(&req.account_number)
    .bind(&req.account_type)
    .bind(req.balance)
    .bind(&req.currency)
    .bind(&req.notes)
    .execute(&mut *tx)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    // Audit log
    let log_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO operation_logs (id, company_id, user_id, action, entity_type, entity_id) VALUES (?, ?, ?, 'create', 'bank_accounts', ?)")
        .bind(&log_id).bind(&user.company_id).bind(&user.user_id).bind(&id)
        .execute(&mut *tx).await.ok();

    tx.commit()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    // R2-1: Re-fetch with company_id scoping
    let account: BankAccount =
        sqlx::query_as("SELECT * FROM bank_accounts WHERE id = ? AND company_id = ?")
            .bind(&id)
            .bind(&user.company_id)
            .fetch_one(&pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(Json(account))
}

async fn update_account(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<CreateAccountRequest>,
) -> Result<Json<BankAccount>, ApiError> {
    // F3: Balance removed from direct update — only changeable via transactions
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let result = sqlx::query(
        "UPDATE bank_accounts SET 
            bank_name = ?, account_name = ?, account_number = ?,
            account_type = COALESCE(?, account_type), 
            currency = COALESCE(?, currency), 
            notes = COALESCE(?, notes),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ? AND company_id = ?",
    )
    .bind(&req.bank_name)
    .bind(&req.account_name)
    .bind(&req.account_number)
    .bind(&req.account_type)
    .bind(&req.currency)
    .bind(&req.notes)
    .bind(&id)
    .bind(&user.company_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    if result.rows_affected() == 0 {
        return Err(api_err(StatusCode::NOT_FOUND, "account not found"));
    }

    // Audit log
    let log_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO operation_logs (id, company_id, user_id, action, entity_type, entity_id) VALUES (?, ?, ?, 'update', 'bank_accounts', ?)")
        .bind(&log_id).bind(&user.company_id).bind(&user.user_id).bind(&id)
        .execute(&mut *tx).await.ok();

    tx.commit()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    // R2-1: Re-fetch with company_id scoping
    let account: BankAccount =
        sqlx::query_as("SELECT * FROM bank_accounts WHERE id = ? AND company_id = ?")
            .bind(&id)
            .bind(&user.company_id)
            .fetch_one(&pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(Json(account))
}

/// F6: Create a bank transaction and atomically update account balance
async fn create_transaction(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Json(req): Json<CreateTransactionRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // F5: Validate amount is positive
    if req.amount <= 0.0 {
        return Err(api_err(StatusCode::BAD_REQUEST, "amount must be positive"));
    }

    // R2-5: Use IMMEDIATE transaction to prevent TOCTOU race on balance
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    sqlx::query("BEGIN IMMEDIATE").execute(&mut *tx).await.ok(); // Upgrade to write lock

    // Verify account exists and belongs to this company
    let account: Option<BankAccount> =
        sqlx::query_as("SELECT * FROM bank_accounts WHERE id = ? AND company_id = ?")
            .bind(&req.account_id)
            .bind(&user.company_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let account = account.ok_or_else(|| api_err(StatusCode::NOT_FOUND, "account not found"))?;
    let current_balance = account.balance.unwrap_or(0.0);

    // Calculate new balance based on transaction type
    let delta = match req.transaction_type.as_str() {
        "deposit" | "income" => req.amount,
        "withdrawal" | "expense" => {
            if current_balance < req.amount {
                return Err(api_err(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "insufficient funds",
                ));
            }
            -req.amount
        }
        _ => return Err(api_err(StatusCode::BAD_REQUEST, "invalid transaction_type")),
    };

    let new_balance = current_balance + delta;

    // Insert transaction record
    let txn_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO bank_transactions (id, company_id, account_id, transaction_type, amount, balance_after, counterparty, description, transaction_date) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&txn_id)
    .bind(&user.company_id)
    .bind(&req.account_id)
    .bind(&req.transaction_type)
    .bind(req.amount)
    .bind(new_balance)
    .bind(&req.counterparty)
    .bind(&req.description)
    .bind(&req.transaction_date)
    .execute(&mut *tx)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    // Atomically update account balance
    sqlx::query("UPDATE bank_accounts SET balance = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? AND company_id = ?")
        .bind(new_balance)
        .bind(&req.account_id)
        .bind(&user.company_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    // Audit log
    let log_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO operation_logs (id, company_id, user_id, action, entity_type, entity_id, details) VALUES (?, ?, ?, 'create', 'bank_transactions', ?, ?)")
        .bind(&log_id).bind(&user.company_id).bind(&user.user_id).bind(&txn_id)
        .bind(format!("{}:{} amount={}", req.transaction_type, req.account_id, req.amount))
        .execute(&mut *tx).await.ok();

    tx.commit()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(Json(serde_json::json!({
        "id": txn_id,
        "balance_after": new_balance
    })))
}

async fn delete_account(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    if let Ok((n,)) = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM bank_transactions WHERE account_id = ? AND company_id = ?",
    )
    .bind(&id)
    .bind(&user.company_id)
    .fetch_one(&pool)
    .await
        && n > 0
    {
        return Err(api_err(
            StatusCode::CONFLICT,
            "cannot delete: has dependent transactions",
        ));
    }

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let result = sqlx::query("DELETE FROM bank_accounts WHERE id = ? AND company_id = ?")
        .bind(&id)
        .bind(&user.company_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    if result.rows_affected() == 0 {
        return Err(api_err(StatusCode::NOT_FOUND, "account not found"));
    }

    // Audit log
    let log_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO operation_logs (id, company_id, user_id, action, entity_type, entity_id) VALUES (?, ?, ?, 'delete', 'bank_accounts', ?)")
        .bind(&log_id).bind(&user.company_id).bind(&user.user_id).bind(&id)
        .execute(&mut *tx).await.ok();

    tx.commit()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(StatusCode::NO_CONTENT)
}

fn bank_account_type_label(account_type: &str) -> &'static str {
    match account_type {
        "checking" | "current" => "活期",
        "savings" | "fixed" | "deposit" => "定期",
        _ => "其他",
    }
}

fn bank_account_status_badge(status: &str) -> Value {
    match status {
        "active" => json!({
            "label": "活跃",
            "variant": "default",
        }),
        _ => json!({
            "label": "停用",
            "variant": "secondary",
        }),
    }
}

fn round_money(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn format_cny(value: f64) -> String {
    let rounded = round_money(value);
    let sign = if rounded < 0.0 { "-" } else { "" };
    let absolute = rounded.abs();
    if (absolute.fract()).abs() < 0.005 {
        format!("¥{sign}{}", comma_i64(absolute.round() as i64))
    } else {
        let whole = absolute.trunc() as i64;
        let cents = ((absolute - whole as f64) * 100.0).round() as i64;
        format!("¥{sign}{}.{cents:02}", comma_i64(whole))
    }
}

fn comma_i64(value: i64) -> String {
    let digits = value.to_string();
    let mut output = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, ch) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            output.push(',');
        }
        output.push(ch);
    }
    output.chars().rev().collect()
}

fn text(row: &sqlx::sqlite::SqliteRow, column: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(column).ok().flatten()
}

fn number(row: &sqlx::sqlite::SqliteRow, column: &str) -> f64 {
    row.try_get::<Option<f64>, _>(column)
        .ok()
        .flatten()
        .unwrap_or(0.0)
}

fn integer(row: &sqlx::sqlite::SqliteRow, column: &str) -> i64 {
    row.try_get::<i64, _>(column).unwrap_or(0)
}
