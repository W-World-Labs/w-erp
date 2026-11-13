use crate::common;

use serde_json::Value;

async fn seed_application(app: &common::TestApp, id: &str, title: &str, created_at: &str) {
    sqlx::query(
        r#"
        INSERT INTO applications (id, company_id, application_no, title, type, created_at, updated_at)
        VALUES (?, ?, ?, ?, 'subsidy', ?, ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(format!("APP-{id}"))
    .bind(title)
    .bind(created_at)
    .bind(created_at)
    .execute(&app.pool)
    .await
    .expect("seed application");
}

async fn seed_contract(app: &common::TestApp, id: &str, name: &str) {
    sqlx::query(
        r#"
        INSERT INTO contracts (id, company_id, contract_name, contract_number, created_at, updated_at)
        VALUES (?, ?, ?, ?, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
    .bind(format!("CT-{id}"))
    .execute(&app.pool)
    .await
    .expect("seed contract");
}

async fn seed_employee(app: &common::TestApp, id: &str, name: &str) {
    sqlx::query(
        r#"
        INSERT INTO employees (id, company_id, name, full_name, created_at, updated_at)
        VALUES (?, ?, ?, ?, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
    .bind(name)
    .execute(&app.pool)
    .await
    .expect("seed employee");
}

async fn seed_budget(app: &common::TestApp, id: &str, category: &str, period: &str) {
    sqlx::query(
        r#"
        INSERT INTO budgets (id, company_id, category, budget_type, period, created_at, updated_at)
        VALUES (?, ?, ?, 'monthly', ?, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(category)
    .bind(period)
    .execute(&app.pool)
    .await
    .expect("seed budget");
}

async fn seed_project(app: &common::TestApp, id: &str, name: &str) {
    sqlx::query(
        r#"
        INSERT INTO projects (id, company_id, name, created_at, updated_at)
        VALUES (?, ?, ?, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
    .execute(&app.pool)
    .await
    .expect("seed project");
}

#[tokio::test]
async fn s119_audit_center_overview_returns_modules_quick_items_and_labels() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_application(&app, "app-old", "旧申报", "2026-01-01T00:00:00Z").await;
    seed_application(&app, "app-alpha", "Alpha 申报", "2026-01-04T00:00:00Z").await;
    seed_application(&app, "app-beta", "Beta 申报", "2026-01-03T00:00:00Z").await;
    seed_application(&app, "app-gamma", "Gamma 申报", "2026-01-02T00:00:00Z").await;
    seed_application(&other, "app-other", "其他租户申报", "2026-01-05T00:00:00Z").await;
    seed_contract(&app, "contract-alpha", "Alpha 合同").await;
    seed_employee(&app, "employee-alpha", "Alpha 员工").await;
    seed_budget(&app, "budget-alpha", "研发", "2026-05").await;
    seed_project(&app, "project-alpha", "Alpha 项目").await;

    let res = app.get("/api/v1/audit_center/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total_auditable_count"].as_i64().unwrap(), 8);
    assert_eq!(body["stats"]["total_auditable_label"], "8 个对象");
    assert_eq!(body["stats"]["module_count"].as_i64().unwrap(), 5);
    assert_eq!(body["stats"]["module_count_label"], "5");
    assert_eq!(body["stats"]["analysis_label"], "24/7");

    let modules = body["modules"].as_array().unwrap();
    assert_eq!(modules.len(), 5);

    let applications = modules
        .iter()
        .find(|module| module["audit_type"] == "application")
        .expect("application module");
    assert_eq!(applications["count"].as_i64().unwrap(), 4);
    assert_eq!(applications["count_label"], "4");
    assert_eq!(applications["quick_title"], "快速审计（最近3项）：");
    assert_eq!(applications["quick_items"].as_array().unwrap().len(), 3);
    assert_eq!(applications["quick_items"][0]["id"], "app-alpha");
    assert_eq!(applications["quick_items"][0]["title"], "Alpha 申报");
    assert_eq!(
        applications["quick_items"][0]["audit_title"],
        "申报项目：Alpha 申报"
    );
    assert_eq!(
        applications["quick_items"][0]["audit_request"]["audit_type"],
        "application"
    );
    assert_eq!(
        applications["quick_items"][0]["audit_request"]["target_id"],
        "app-alpha"
    );
    assert!(
        applications["quick_items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["id"] != "app-other")
    );

    let budgets = modules
        .iter()
        .find(|module| module["audit_type"] == "budget")
        .expect("budget module");
    assert_eq!(budgets["quick_items"][0]["title"], "研发 (2026-05)");
    assert_eq!(
        budgets["quick_items"][0]["audit_title"],
        "预算分析：研发 (2026-05)"
    );

    let capabilities = body["capability_sections"].as_array().unwrap();
    assert_eq!(capabilities.len(), 5);
    assert!(
        capabilities
            .iter()
            .any(|section| section["title"] == "项目健康审计")
    );
}
