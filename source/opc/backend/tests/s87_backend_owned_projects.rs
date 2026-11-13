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

fn assert_f64(actual: &Value, expected: f64) {
    let actual = actual.as_f64().expect("number");
    assert!(
        (actual - expected).abs() < 0.001,
        "expected {expected}, got {actual}"
    );
}

#[tokio::test]
async fn s87_projects_overview_calculates_stats_filters_and_rd_cost_groups_on_backend() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_project(
        &app,
        "project-alpha",
        "Alpha Automation",
        "ongoing",
        1000.0,
        2,
    )
    .await;
    seed_project(
        &app,
        "project-beta",
        "Beta Platform",
        "completed",
        2000.0,
        3,
    )
    .await;
    seed_project(
        &other,
        "project-other",
        "Other Tenant",
        "ongoing",
        9000.0,
        9,
    )
    .await;

    seed_rd_cost(
        &app,
        "cost-alpha-personnel",
        "project-alpha",
        "personnel",
        100.0,
        "2026-05-02",
    )
    .await;
    seed_rd_cost(
        &app,
        "cost-alpha-material",
        "project-alpha",
        "material",
        50.0,
        "2026-05-01",
    )
    .await;
    seed_rd_cost(
        &app,
        "cost-beta-equipment",
        "project-beta",
        "equipment",
        30.0,
        "2026-04-30",
    )
    .await;
    seed_rd_cost(
        &other,
        "cost-other",
        "project-other",
        "personnel",
        999.0,
        "2026-05-03",
    )
    .await;

    let res = app
        .get("/api/v1/projects/overview?search=alpha&status=ongoing")
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total_projects"].as_i64().unwrap(), 2);
    assert_eq!(body["stats"]["ongoing_count"].as_i64().unwrap(), 1);
    assert_f64(&body["stats"]["total_budget"], 3000.0);
    assert_eq!(body["stats"]["total_members"].as_i64().unwrap(), 5);
    assert_f64(&body["stats"]["total_rd_cost"], 180.0);

    let projects = body["projects"].as_array().unwrap();
    assert_eq!(projects.len(), 1);
    let project = &projects[0];
    assert_eq!(project["id"], "project-alpha");
    assert_eq!(project["project_no"], "project-alpha");
    assert_f64(&project["rd_cost_summary"]["total"], 150.0);
    assert_f64(&project["rd_cost_summary"]["by_type"]["personnel"], 100.0);
    assert_f64(&project["rd_cost_summary"]["by_type"]["material"], 50.0);

    let items = project["rd_cost_summary"]["items"].as_array().unwrap();
    assert_eq!(items[0]["id"], "cost-alpha-personnel");
    assert_eq!(items[0]["cost_type_label"], "人员费用");

    let pie_data = project["rd_cost_summary"]["pie_data"].as_array().unwrap();
    assert!(pie_data.iter().any(|row| row["name"] == "人员费用"));
    assert!(pie_data.iter().any(|row| row["name"] == "材料费用"));
}
