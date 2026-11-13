use super::error::{ApiError, api_err};
use crate::auth::AuthUser;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::SqlitePool;

/// Company management API
/// Handles company CRUD and member management
pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/", get(list_user_companies).post(create_company))
        .route(
            "/{id}",
            get(get_company).put(update_company).delete(delete_company),
        )
        .route("/{id}/members", get(list_members).post(add_member))
        .route("/{id}/members/overview", get(members_overview))
        .route(
            "/{id}/members/{member_id}",
            get(get_member).put(update_member).delete(remove_member),
        )
        // Switching mutates the current session, so this must not be a GET.
        .route("/{id}/switch", post(switch_company))
        .with_state(pool)
}

#[derive(Serialize)]
struct CompanyResponse {
    id: String,
    name: String,
    legal_name: Option<String>,
    tax_id: Option<String>,
    role: String,
    role_badge: RoleBadge,
    is_default: bool,
    is_current: bool,
}

#[derive(Serialize)]
struct MemberResponse {
    id: String,
    user_id: String,
    email: String,
    name: Option<String>,
    role: String,
}

#[derive(Deserialize)]
struct CreateCompanyRequest {
    name: String,
    legal_name: Option<String>,
    tax_id: Option<String>,
    address: Option<String>,
}

#[derive(Deserialize)]
struct AddMemberRequest {
    email: String,
    role: Option<String>,
}

#[derive(Deserialize)]
struct MembersOverviewQuery {
    search: Option<String>,
}

#[derive(Serialize)]
struct RoleBadge {
    value: &'static str,
    label: &'static str,
    icon: &'static str,
    variant: &'static str,
}

fn normalize_member_role(role: &str) -> Option<&'static str> {
    match role.trim().to_ascii_lowercase().as_str() {
        "admin" => Some("admin"),
        "member" => Some("member"),
        "accountant" => Some("accountant"),
        "auditor" => Some("auditor"),
        "tax_advisor" => Some("tax_advisor"),
        _ => None,
    }
}

fn parse_member_role(role: Option<&str>) -> Result<String, ApiError> {
    let raw = role.unwrap_or("member");
    normalize_member_role(raw)
        .map(str::to_string)
        .ok_or_else(|| api_err(StatusCode::BAD_REQUEST, "invalid role"))
}

fn role_badge(role: &str) -> RoleBadge {
    match role {
        "owner" => RoleBadge {
            value: "owner",
            label: "所有者",
            icon: "Crown",
            variant: "default",
        },
        "admin" => RoleBadge {
            value: "admin",
            label: "管理员",
            icon: "Shield",
            variant: "secondary",
        },
        "accountant" => RoleBadge {
            value: "accountant",
            label: "会计",
            icon: "Calculator",
            variant: "outline",
        },
        "auditor" => RoleBadge {
            value: "auditor",
            label: "审计",
            icon: "Eye",
            variant: "outline",
        },
        "tax_advisor" => RoleBadge {
            value: "tax_advisor",
            label: "税务顾问",
            icon: "Landmark",
            variant: "outline",
        },
        _ => RoleBadge {
            value: "member",
            label: "成员",
            icon: "Users",
            variant: "outline",
        },
    }
}

fn assignable_role_options() -> Vec<RoleBadge> {
    ["admin", "member", "accountant", "auditor", "tax_advisor"]
        .into_iter()
        .map(role_badge)
        .collect()
}

fn avatar_fallback(name: Option<&str>, email: &str) -> String {
    let source = name
        .and_then(|value| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then_some(trimmed)
        })
        .unwrap_or(email);

    source
        .chars()
        .next()
        .map(|ch| ch.to_uppercase().collect())
        .unwrap_or_else(|| "?".to_string())
}

/// Verify user is a member of the given company (must not be soft-deleted), return their role
async fn verify_membership(
    pool: &SqlitePool,
    user_id: &str,
    company_id: &str,
) -> Result<String, ApiError> {
    // R3-1: Also check company is not soft-deleted
    let member: Option<(String,)> = sqlx::query_as(
        r#"SELECT cm.role FROM company_members cm
           JOIN companies c ON cm.company_id = c.id
           WHERE cm.user_id = ? AND cm.company_id = ? AND COALESCE(c.is_deleted, 0) = 0"#,
    )
    .bind(user_id)
    .bind(company_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    member
        .map(|m| m.0)
        .ok_or_else(|| api_err(StatusCode::FORBIDDEN, "not a member of this company"))
}

/// Require owner or admin role
fn require_admin(role: &str) -> Result<(), ApiError> {
    if role == "owner" || role == "admin" {
        Ok(())
    } else {
        Err(api_err(
            StatusCode::FORBIDDEN,
            "admin or owner role required",
        ))
    }
}

// List companies the current user belongs to (exclude soft-deleted)
async fn list_user_companies(
    State(pool): State<SqlitePool>,
    user: AuthUser,
) -> Result<Json<Value>, ApiError> {
    let rows: Vec<(String, String, Option<String>, Option<String>, String, i64)> = sqlx::query_as(
        r#"SELECT c.id, c.name, c.legal_name, c.tax_id, cm.role, cm.is_default
           FROM companies c
           JOIN company_members cm ON c.id = cm.company_id
           WHERE cm.user_id = ? AND COALESCE(c.is_deleted, 0) = 0
           ORDER BY CASE WHEN c.id = ? THEN 0 ELSE 1 END, cm.is_default DESC, c.name"#,
    )
    .bind(&user.user_id)
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let companies: Vec<CompanyResponse> = rows
        .into_iter()
        .map(|r| {
            let role = r.4;
            let badge = role_badge(&role);
            let is_current = r.0 == user.company_id;
            CompanyResponse {
                id: r.0,
                name: r.1,
                legal_name: r.2,
                tax_id: r.3,
                role,
                role_badge: badge,
                is_default: r.5 == 1,
                is_current,
            }
        })
        .collect();

    Ok(Json(json!({ "data": companies })))
}

// R3-7: Create company with transaction
async fn create_company(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Json(body): Json<CreateCompanyRequest>,
) -> Result<Json<Value>, ApiError> {
    let company_id = uuid::Uuid::new_v4().to_string();
    let member_id = uuid::Uuid::new_v4().to_string();

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    // Create company
    sqlx::query(
        "INSERT INTO companies (id, name, legal_name, tax_id, address) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&company_id)
    .bind(&body.name)
    .bind(&body.legal_name)
    .bind(&body.tax_id)
    .bind(&body.address)
    .execute(&mut *tx)
    .await
    .map_err(|_| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to create company",
        )
    })?;

    // Check if user has any existing companies (to set is_default)
    let existing: Option<(i64,)> =
        sqlx::query_as("SELECT COUNT(*) FROM company_members WHERE user_id = ?")
            .bind(&user.user_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let is_default = existing.map(|c| c.0 == 0).unwrap_or(true);

    // Add user as owner
    sqlx::query(
        "INSERT INTO company_members (id, user_id, company_id, role, is_default) VALUES (?, ?, ?, 'owner', ?)"
    )
    .bind(&member_id)
    .bind(&user.user_id)
    .bind(&company_id)
    .bind(if is_default { 1 } else { 0 })
    .execute(&mut *tx)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "failed to add owner"))?;

    tx.commit()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(Json(json!({ "id": company_id, "name": body.name })))
}

// Get company details (must be a member)
async fn get_company(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_membership(&pool, &user.user_id, &id).await?;

    let row: Option<(String, String, Option<String>, Option<String>, Option<String>, Option<String>)> =
        sqlx::query_as(
            "SELECT id, name, legal_name, tax_id, address, phone FROM companies WHERE id = ? AND COALESCE(is_deleted, 0) = 0"
        )
        .bind(&id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    match row {
        Some(r) => Ok(Json(json!({
            "id": r.0, "name": r.1, "legal_name": r.2,
            "tax_id": r.3, "address": r.4, "phone": r.5
        }))),
        None => Err(api_err(StatusCode::NOT_FOUND, "company not found")),
    }
}

// R3-2: Update company — handle all fields (owner/admin only)
async fn update_company(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let role = verify_membership(&pool, &user.user_id, &id).await?;
    require_admin(&role)?;

    let obj = body.as_object().ok_or_else(|| {
        api_err(
            StatusCode::BAD_REQUEST,
            "request body must be a JSON object",
        )
    })?;

    // R3-2 + G-6: Update all supported fields including locale/currency
    let result = sqlx::query(
        "UPDATE companies SET
            name = COALESCE(?, name),
            legal_name = COALESCE(?, legal_name),
            tax_id = COALESCE(?, tax_id),
            address = COALESCE(?, address),
            phone = COALESCE(?, phone),
            country = COALESCE(?, country),
            timezone = COALESCE(?, timezone),
            locale = COALESCE(?, locale),
            base_currency = COALESCE(?, base_currency),
            date_format = COALESCE(?, date_format),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ? AND COALESCE(is_deleted, 0) = 0",
    )
    .bind(obj.get("name").and_then(|v| v.as_str()))
    .bind(obj.get("legal_name").and_then(|v| v.as_str()))
    .bind(obj.get("tax_id").and_then(|v| v.as_str()))
    .bind(obj.get("address").and_then(|v| v.as_str()))
    .bind(obj.get("phone").and_then(|v| v.as_str()))
    .bind(obj.get("country").and_then(|v| v.as_str()))
    .bind(obj.get("timezone").and_then(|v| v.as_str()))
    .bind(obj.get("locale").and_then(|v| v.as_str()))
    .bind(obj.get("base_currency").and_then(|v| v.as_str()))
    .bind(obj.get("date_format").and_then(|v| v.as_str()))
    .bind(&id)
    .execute(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    if result.rows_affected() == 0 {
        return Err(api_err(StatusCode::NOT_FOUND, "company not found"));
    }

    Ok(Json(json!({ "id": id })))
}

// R3-1: Soft delete company (owner only)
async fn delete_company(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let role = verify_membership(&pool, &user.user_id, &id).await?;
    if role != "owner" {
        return Err(api_err(
            StatusCode::FORBIDDEN,
            "only the owner can delete a company",
        ));
    }

    // R3-1: Soft delete — mark as deleted instead of removing
    let result = sqlx::query(
        "UPDATE companies SET is_deleted = 1, deleted_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = ? AND COALESCE(is_deleted, 0) = 0"
    )
    .bind(&id)
    .execute(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    if result.rows_affected() == 0 {
        return Err(api_err(
            StatusCode::NOT_FOUND,
            "company not found or already deleted",
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

// List company members (must be a member)
async fn list_members(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(company_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    verify_membership(&pool, &user.user_id, &company_id).await?;

    let rows: Vec<(String, String, String, Option<String>, String)> = sqlx::query_as(
        r#"SELECT cm.id, cm.user_id, u.email, u.name, cm.role
           FROM company_members cm
           JOIN users u ON cm.user_id = u.id
           WHERE cm.company_id = ?"#,
    )
    .bind(&company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let members: Vec<MemberResponse> = rows
        .into_iter()
        .map(|r| MemberResponse {
            id: r.0,
            user_id: r.1,
            email: r.2,
            name: r.3,
            role: r.4,
        })
        .collect();

    Ok(Json(json!({ "data": members })))
}

// Company member overview keeps role labels, permissions, filtering, and stats on the backend.
async fn members_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(company_id): Path<String>,
    Query(query): Query<MembersOverviewQuery>,
) -> Result<Json<Value>, ApiError> {
    let current_user_role = verify_membership(&pool, &user.user_id, &company_id).await?;
    let is_admin = current_user_role == "owner" || current_user_role == "admin";

    let company: Option<(String,)> =
        sqlx::query_as("SELECT name FROM companies WHERE id = ? AND COALESCE(is_deleted, 0) = 0")
            .bind(&company_id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let company_name = company
        .map(|row| row.0)
        .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "company not found"))?;

    let rows: Vec<(String, String, String, Option<String>, String)> = sqlx::query_as(
        r#"SELECT cm.id, cm.user_id, u.email, u.name, cm.role
           FROM company_members cm
           JOIN users u ON cm.user_id = u.id
           WHERE cm.company_id = ?
           ORDER BY CASE cm.role WHEN 'owner' THEN 0 WHEN 'admin' THEN 1 ELSE 2 END, u.email"#,
    )
    .bind(&company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let stats_total = rows.len();
    let stats_owners = rows.iter().filter(|row| row.4 == "owner").count();
    let stats_admins = rows.iter().filter(|row| row.4 == "admin").count();
    let search = query.search.unwrap_or_default().trim().to_lowercase();

    let members: Vec<Value> = rows
        .into_iter()
        .filter_map(|(id, user_id, email, name, member_role)| {
            if !search.is_empty() {
                let name_match = name
                    .as_deref()
                    .map(|value| value.to_lowercase().contains(&search))
                    .unwrap_or(false);
                if !email.to_lowercase().contains(&search) && !name_match {
                    return None;
                }
            }

            let display_name = name
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("未设置昵称")
                .to_string();
            let avatar = avatar_fallback(name.as_deref(), &email);
            let can_manage_member = is_admin && member_role != "owner";
            let badge = role_badge(&member_role);

            Some(json!({
                "id": id,
                "user_id": user_id,
                "email": email,
                "name": name,
                "role": member_role,
                "display_name": display_name,
                "avatar_fallback": avatar,
                "role_badge": badge,
                "can_edit_role": can_manage_member,
                "can_remove": can_manage_member,
            }))
        })
        .collect();

    let empty_message = if search.is_empty() {
        "暂无成员数据"
    } else {
        "未找到匹配的成员"
    };

    Ok(Json(json!({
        "company": {
            "id": company_id,
            "name": company_name,
            "current_user_role": current_user_role,
            "is_admin": is_admin,
        },
        "stats": {
            "total": stats_total,
            "owners": stats_owners,
            "admins": stats_admins,
        },
        "members": members,
        "assignable_roles": assignable_role_options(),
        "empty_message": empty_message,
    })))
}

// Add member to company (owner/admin only)
async fn add_member(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(company_id): Path<String>,
    Json(body): Json<AddMemberRequest>,
) -> Result<Json<Value>, ApiError> {
    let role = verify_membership(&pool, &user.user_id, &company_id).await?;
    require_admin(&role)?;

    // Find user by email
    let target_user: Option<(String,)> = sqlx::query_as("SELECT id FROM users WHERE email = ?")
        .bind(&body.email)
        .fetch_optional(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let target_user_id = target_user
        .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "user not found"))?
        .0;
    let member_role = parse_member_role(body.role.as_deref())?;

    let existing_member: Option<(String,)> =
        sqlx::query_as("SELECT id FROM company_members WHERE user_id = ? AND company_id = ?")
            .bind(&target_user_id)
            .bind(&company_id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    if existing_member.is_some() {
        return Err(api_err(
            StatusCode::CONFLICT,
            "user is already a member of this company",
        ));
    }

    let member_id = uuid::Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO company_members (id, user_id, company_id, role) VALUES (?, ?, ?, ?)")
        .bind(&member_id)
        .bind(&target_user_id)
        .bind(&company_id)
        .bind(&member_role)
        .execute(&pool)
        .await
        .map_err(|err| match err {
            sqlx::Error::Database(db_err)
                if db_err.message().contains("UNIQUE constraint failed") =>
            {
                api_err(
                    StatusCode::CONFLICT,
                    "user is already a member of this company",
                )
            }
            _ => api_err(StatusCode::INTERNAL_SERVER_ERROR, "failed to add member"),
        })?;

    Ok(Json(json!({ "id": member_id })))
}

// Get member (must be a member of the company)
async fn get_member(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path((company_id, member_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    verify_membership(&pool, &user.user_id, &company_id).await?;

    let row: Option<(String, String, String, Option<String>, String)> = sqlx::query_as(
        r#"SELECT cm.id, cm.user_id, u.email, u.name, cm.role
           FROM company_members cm
           JOIN users u ON cm.user_id = u.id
           WHERE cm.id = ? AND cm.company_id = ?"#,
    )
    .bind(&member_id)
    .bind(&company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    match row {
        Some(r) => Ok(Json(json!({
            "id": r.0, "user_id": r.1, "email": r.2, "name": r.3, "role": r.4
        }))),
        None => Err(api_err(StatusCode::NOT_FOUND, "member not found")),
    }
}

// Update member role (owner/admin only)
async fn update_member(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path((company_id, member_id)): Path<(String, String)>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let role = verify_membership(&pool, &user.user_id, &company_id).await?;
    require_admin(&role)?;

    if let Some(new_role) = body.get("role").and_then(|v| v.as_str()) {
        let new_role = parse_member_role(Some(new_role))?;
        sqlx::query("UPDATE company_members SET role = ? WHERE id = ? AND company_id = ?")
            .bind(&new_role)
            .bind(&member_id)
            .bind(&company_id)
            .execute(&pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    }

    Ok(Json(json!({ "id": member_id })))
}

// Remove member (owner/admin only)
async fn remove_member(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path((company_id, member_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let role = verify_membership(&pool, &user.user_id, &company_id).await?;
    require_admin(&role)?;

    sqlx::query("DELETE FROM company_members WHERE id = ? AND company_id = ?")
        .bind(&member_id)
        .bind(&company_id)
        .execute(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(StatusCode::NO_CONTENT)
}

// Switch current company (must be a member of target company)
async fn switch_company(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(company_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // S4: Verify user is a member of the target company (also checks not soft-deleted)
    verify_membership(&pool, &user.user_id, &company_id).await?;

    // Update session with current company
    sqlx::query("UPDATE sessions SET current_company_id = ? WHERE id = ?")
        .bind(&company_id)
        .bind(&user.session_id)
        .execute(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(Json(json!({ "current_company_id": company_id })))
}

// Auto-create default company for new user (called after OAuth)
pub async fn ensure_default_company(
    pool: &SqlitePool,
    user_id: &str,
    user_email: &str,
) -> Result<String, sqlx::Error> {
    // Check if user has any companies (non-deleted). Prefer their real default
    // company over the demo sandbox so a returning user is never silently
    // switched into the demo.
    let existing: Option<(String,)> = sqlx::query_as(
        r#"SELECT cm.company_id FROM company_members cm
           JOIN companies c ON cm.company_id = c.id
           WHERE cm.user_id = ? AND COALESCE(c.is_deleted, 0) = 0
           ORDER BY cm.is_default DESC, c.is_demo ASC
           LIMIT 1"#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if let Some((company_id,)) = existing {
        // Backfill: pre-Demo-feature users already own a company but may never
        // have received the demo sandbox. Seed it now (idempotent) without
        // changing which company is current.
        if !crate::db::demo_seed::user_has_demo_company(pool, user_id).await? {
            crate::db::demo_seed::ensure_demo_company_for_user(pool, user_id, user_email).await?;
        }
        return Ok(company_id);
    }

    crate::db::demo_seed::ensure_demo_company_for_user(pool, user_id, user_email).await
}
