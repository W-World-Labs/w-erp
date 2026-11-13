use crate::common;

use serde_json::Value;

async fn seed_project(
    app: &common::TestApp,
    id: &str,
    name: &str,
    status: &str,
    budget: f64,
    team_members: i64,
) {
    sqlx::query(
        r#"
        INSERT INTO projects (
            id, company_id, name, project_type, leader_name, status,
            budget, progress, team_members
        )
        VALUES (?, ?, ?, 'rd', 'Lead', ?, ?, 35, ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
    .bind(status)
    .bind(budget)
    .bind(team_members)
    .execute(&app.pool)
    .await
    .expect("seed project");
}

#[tokio::test]
async fn s108_project_form_options_are_backend_owned() {
    let app = common::TestApp::new().await;

    let res = app.get("/api/v1/projects/form_options").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["default_values"]["project_type"], "");
    assert_eq!(body["project_type_options"][0]["value"], "tech");
    assert_eq!(body["project_type_options"][0]["label"], "技术研发");
    assert_eq!(body["project_type_options"][4]["value"], "innovation");
}

async fn seed_rd_cost(
    app: &common::TestApp,
    id: &str,
    project_id: &str,
    cost_type: &str,
    amount: f64,
    cost_date: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO rd_costs (
            id, company_id, project_id, cost_type, amount, description, cost_date
        )
        VALUES (?, ?, ?, ?, ?, 'R&D cost', ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(project_id)
    .bind(cost_type)
    .bind(amount)
    .bind(cost_date)
    .execute(&app.pool)
    .await
    .expect("seed rd cost");
}

async fn seed_employee(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO employees (id, company_id, name, position, status)
        VALUES ('employee-option', ?, 'Needle Employee', 'Backend Role', 'active')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed employee");
}

async fn seed_assets(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO assets (id, company_id, name, asset_type, serial_number, status)
        VALUES
            ('asset-option', ?, 'Needle Device', 'physical', 'SN-1', 'active'),
            ('asset-virtual', ?, 'Virtual Asset', 'virtual', 'VA-1', 'active')
        "#,
    )
    .bind(&app.company_id)
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed assets");
}

#[tokio::test]
async fn s108_projects_overview_returns_display_labels_badges_options_and_permissions() {
    let app = common::TestApp::new().await;

    seed_project(
        &app,
        "project-alpha",
        "Alpha Automation",
        "ongoing",
        10000.0,
        2,
    )
    .await;
    seed_project(
        &app,
        "project-beta",
        "Beta Platform",
        "completed",
        20000.0,
        3,
    )
    .await;
    seed_rd_cost(
        &app,
        "cost-alpha-personnel",
        "project-alpha",
        "personnel",
        10000.0,
        "2026-05-02",
    )
    .await;
    seed_rd_cost(
        &app,
        "cost-alpha-material",
        "project-alpha",
        "material",
        5000.5,
        "2026-05-01",
    )
    .await;
    seed_employee(&app).await;
    seed_assets(&app).await;

    let res = app
        .get("/api/v1/projects/overview?search=alpha&status=ongoing")
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total_budget_label"], "¥3.0万");
    assert_eq!(body["stats"]["total_rd_cost_label"], "¥1.5万");
    assert_eq!(body["permissions"]["can_create"], true);
    assert_eq!(body["permissions"]["can_edit"], true);
    assert_eq!(body["permissions"]["can_delete"], true);
    assert_eq!(body["permissions"]["can_manage_rd_costs"], true);
    assert_eq!(body["create_action"]["method"], "POST");
    assert_eq!(body["create_action"]["endpoint"], "/api/projects");
    assert_eq!(body["create_action"]["label"], "新建项目");
    assert_eq!(body["create_action"]["success_title"], "创建成功");
    assert_eq!(body["create_action"]["success_message"], "项目已成功创建");
    assert_eq!(body["empty_message"], "未找到匹配的项目");
    assert!(
        body["status_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "ongoing" && option["label"] == "进行中")
    );
    assert_eq!(body["status_options"][0]["value"], "all");
    assert_eq!(body["status_options"][0]["label"], "全部状态");
    assert!(
        body["rd_cost_type_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "personnel" && option["label"] == "人员费用")
    );
    assert_eq!(body["employee_options"][0]["value"], "employee-option");
    assert_eq!(
        body["employee_options"][0]["label"],
        "Needle Employee - Backend Role"
    );
    assert_eq!(body["asset_options"][0]["value"], "asset-option");
    assert_eq!(body["asset_options"][0]["label"], "Needle Device (SN-1)");

    let projects = body["projects"].as_array().unwrap();
    assert_eq!(projects.len(), 1);
    let project = &projects[0];
    assert_eq!(project["budget_label"], "¥1.0万");
    assert_eq!(project["progress_label"], "35%");
    assert_eq!(project["status_badge"]["label"], "进行中");
    assert_eq!(project["status_badge"]["variant"], "default");
    assert_eq!(project["status_badge"]["icon"], "TrendingUp");
    assert_eq!(project["can_edit"], true);
    assert_eq!(project["update_action"]["method"], "PUT");
    assert_eq!(
        project["update_action"]["endpoint"],
        "/api/projects/project-alpha"
    );
    assert_eq!(project["update_action"]["label"], "保存修改");
    assert_eq!(project["update_action"]["success_title"], "更新成功");
    assert_eq!(
        project["update_action"]["success_message"],
        "项目已成功更新"
    );
    assert_eq!(project["can_delete"], true);
    assert_eq!(project["can_add_rd_cost"], true);
    assert_eq!(project["rd_cost_create_action"]["method"], "POST");
    assert_eq!(
        project["rd_cost_create_action"]["endpoint"],
        "/api/v1/rd_costs"
    );
    assert_eq!(project["rd_cost_create_action"]["label"], "添加费用");
    assert_eq!(
        project["rd_cost_create_action"]["success_message"],
        "研发费用已添加"
    );
    assert_eq!(
        project["rd_cost_create_action"]["default_values"]["project_id"],
        "project-alpha"
    );
    assert_eq!(project["delete_action"]["method"], "DELETE");
    assert_eq!(
        project["delete_action"]["endpoint"],
        "/api/projects/project-alpha"
    );
    assert_eq!(
        project["delete_action"]["confirm_message"],
        "确定要删除项目「Alpha Automation」吗？"
    );
    assert_eq!(project["delete_action"]["success_title"], "删除成功");
    assert_eq!(
        project["delete_action"]["success_message"],
        "项目已成功删除"
    );

    let summary = &project["rd_cost_summary"];
    assert_eq!(summary["total_label"], "¥15,000.50");
    assert_eq!(summary["total_wan_label"], "¥1.5万");
    let items = summary["items"].as_array().unwrap();
    assert_eq!(items[0]["id"], "cost-alpha-personnel");
    assert_eq!(items[0]["amount_label"], "¥10,000");
    assert_eq!(items[0]["cost_type_badge"]["label"], "人员费用");
    assert_eq!(items[0]["can_edit"], true);
    assert_eq!(items[0]["update_action"]["method"], "PUT");
    assert_eq!(
        items[0]["update_action"]["endpoint"],
        "/api/v1/rd_costs/cost-alpha-personnel"
    );
    assert_eq!(items[0]["update_action"]["label"], "保存费用");
    assert_eq!(
        items[0]["update_action"]["success_message"],
        "研发费用已更新"
    );
    assert_eq!(items[0]["can_delete"], true);
    assert_eq!(items[0]["delete_action"]["method"], "DELETE");
    assert_eq!(
        items[0]["delete_action"]["endpoint"],
        "/api/v1/rd_costs/cost-alpha-personnel"
    );
    assert_eq!(
        items[0]["delete_action"]["confirm_message"],
        "确定要删除研发费用「人员费用 2026-05-02」吗？"
    );
    assert_eq!(items[0]["delete_action"]["success_title"], "删除成功");
    assert_eq!(
        items[0]["delete_action"]["success_message"],
        "研发费用已删除"
    );

    let pie_data = summary["pie_data"].as_array().unwrap();
    assert!(
        pie_data
            .iter()
            .any(|row| row["name"] == "人员费用" && row["amount_label"] == "¥10,000")
    );
}

#[tokio::test]
async fn s108_project_and_rd_cost_delete_require_backend_permission() {
    let app = common::TestApp::new().await;
    seed_project(
        &app,
        "project-delete-guard",
        "Delete Guard",
        "planning",
        1000.0,
        1,
    )
    .await;
    seed_project(
        &app,
        "project-cost-guard",
        "Cost Guard",
        "planning",
        1000.0,
        1,
    )
    .await;
    seed_rd_cost(
        &app,
        "cost-delete-guard",
        "project-cost-guard",
        "other",
        88.0,
        "2026-05-09",
    )
    .await;

    sqlx::query("UPDATE company_members SET role = 'member' WHERE user_id = ? AND company_id = ?")
        .bind(&app.user_id)
        .bind(&app.company_id)
        .execute(&app.pool)
        .await
        .expect("downgrade member");

    let project_delete = app.delete("/api/v1/projects/project-delete-guard").await;
    project_delete.assert_status_forbidden();

    let rd_cost_delete = app.delete("/api/v1/rd_costs/cost-delete-guard").await;
    rd_cost_delete.assert_status_forbidden();
}
