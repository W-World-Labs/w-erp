use crate::common;

use serde_json::Value;

async fn seed_user(app: &common::TestApp, id: &str, email: &str, name: &str) {
    sqlx::query("INSERT INTO users (id, email, name) VALUES (?, ?, ?)")
        .bind(id)
        .bind(email)
        .bind(name)
        .execute(&app.pool)
        .await
        .expect("seed user");
}

async fn seed_member(app: &common::TestApp, id: &str, user_id: &str, role: &str) {
    sqlx::query("INSERT INTO company_members (id, user_id, company_id, role) VALUES (?, ?, ?, ?)")
        .bind(id)
        .bind(user_id)
        .bind(&app.company_id)
        .bind(role)
        .execute(&app.pool)
        .await
        .expect("seed company member");
}

#[tokio::test]
async fn s95_company_members_overview_filters_stats_roles_and_permissions_on_backend() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_user(&app, "admin-user", "admin.member@test-com.example.invalid", "Admin Member").await;
    seed_user(&app, "member-user", "member@test-com.example.invalid", "Normal Member").await;
    seed_user(&app, "auditor-user", "auditor@test-com.example.invalid", "Audit Member").await;
    seed_member(&app, "member-admin", "admin-user", "admin").await;
    seed_member(&app, "member-normal", "member-user", "member").await;
    seed_member(&app, "member-auditor", "auditor-user", "auditor").await;

    seed_user(&other, "other-user", "admin.other@test-com.example.invalid", "Other Admin").await;
    seed_member(&other, "other-admin", "other-user", "admin").await;

    let res = app
        .get(&format!(
            "/api/companies/{}/members/overview?search=admin",
            app.company_id
        ))
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total"].as_i64().unwrap(), 4);
    assert_eq!(body["stats"]["owners"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["admins"].as_i64().unwrap(), 1);
    assert_eq!(body["company"]["id"], app.company_id);
    assert_eq!(body["company"]["current_user_role"], "owner");
    assert_eq!(body["company"]["is_admin"], true);

    let members = body["members"].as_array().unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0]["id"], "member-admin");
    assert_eq!(members[0]["role_badge"]["label"], "管理员");
    assert_eq!(members[0]["can_edit_role"], true);
    assert_eq!(members[0]["can_remove"], true);
    assert!(
        members
            .iter()
            .all(|member| member["email"] != "admin.other@test-com.example.invalid")
    );

    assert!(
        body["assignable_roles"]
            .as_array()
            .unwrap()
            .iter()
            .any(|role| { role["value"] == "tax_advisor" && role["label"] == "税务顾问" })
    );
}

#[tokio::test]
async fn s95_company_list_returns_backend_owned_role_badges() {
    let app = common::TestApp::new().await;

    let res = app.get("/api/companies").await;
    res.assert_status_ok();
    let body = res.json::<Value>();
    let companies = body["data"].as_array().unwrap();
    let company = companies
        .iter()
        .find(|company| company["id"] == app.company_id)
        .expect("current company in list");

    assert_eq!(company["role"], "owner");
    assert_eq!(company["role_badge"]["label"], "所有者");
    assert_eq!(company["role_badge"]["variant"], "default");
    assert_eq!(company["role_badge"]["icon"], "Crown");
}
