use crate::common;

use chrono::Utc;
use serde_json::{Value, json};

async fn seed_employee(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO employees (id, company_id, name, department, position, status)
        VALUES ('salary-emp-needle', ?, 'Needle Employee', 'Backend Dept', 'Backend Role', 'active')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed employee");
}

async fn seed_salary(app: &common::TestApp) {
    // Cost-analysis stats reflect the CURRENT calendar month, so seed the salary
    // record in the current period (matching s77) rather than a fixed date that
    // goes stale once the real clock rolls past it.
    let period = Utc::now().format("%Y-%m").to_string();
    sqlx::query(
        r#"
        INSERT INTO salary_records (
            id, company_id, employee_id, period, base_salary, bonus, deduction,
            social_security, social_security_company, housing_fund, housing_fund_company,
            net_salary, status
        )
        VALUES (
            'salary-needle', ?, 'salary-emp-needle', ?, 1000.0, 200.0, 50.0,
            30.0, 60.0, 20.0, 40.0, 1100.0, 'pending'
        )
        "#,
    )
    .bind(&app.company_id)
    .bind(&period)
    .execute(&app.pool)
    .await
    .expect("seed salary");
}

async fn seed_insurance_configs(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO social_insurance_configs (
            id, company_id, city, social_security_base_min, social_security_base_max,
            housing_fund_base_min, housing_fund_base_max,
            pension_personal_rate, pension_company_rate, medical_personal_rate,
            medical_company_rate, unemployment_personal_rate, unemployment_company_rate,
            work_injury_company_rate, maternity_company_rate,
            housing_fund_personal_rate, housing_fund_company_rate, notes, created_at
        )
        VALUES (
            'cfg-old', ?, 'Old City', 4000.0, 9000.0, 2000.0, 7000.0,
            7.0, 13.0, 2.0, 9.0, 0.5, 0.5, 0.2, 0.0, 10.0, 10.0,
            'old config', '2026-01-01 00:00:00'
        ),
        (
            'cfg-current', ?, 'Hangzhou Current', 5000.0, 10000.0, 3000.0, 8000.0,
            8.0, 14.0, 2.0, 9.5, 0.5, 0.5, 0.2, 0.0, 12.0, 12.0,
            'current config', '2026-05-01 00:00:00'
        )
        "#,
    )
    .bind(&app.company_id)
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed insurance configs");
}

#[tokio::test]
async fn s107_salary_overview_returns_filtered_rows_labels_badges_actions_and_permissions() {
    let app = common::TestApp::new().await;
    seed_employee(&app).await;
    seed_salary(&app).await;
    seed_insurance_configs(&app).await;

    let res = app.get("/api/v1/salary/overview?search=needle").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total_label"], "¥1,200");
    assert_eq!(body["stats"]["average_label"], "¥1,200");
    assert_eq!(body["cost_analysis"]["stats"]["total_cost_label"], "¥1,200");
    assert_eq!(
        body["cost_analysis"]["stats"]["employee_cost_share_label"],
        "91.7%"
    );
    assert_eq!(body["permissions"]["can_manage"], true);
    assert_eq!(body["create_action"]["method"], "POST");
    assert_eq!(body["create_action"]["endpoint"], "/api/v1/salary");
    assert_eq!(body["create_action"]["label"], "录入工资");
    assert_eq!(body["create_action"]["success_title"], "创建成功");
    assert_eq!(body["create_action"]["success_message"], "工资记录已添加");
    assert_eq!(body["insurance_config_create_action"]["method"], "POST");
    assert_eq!(
        body["insurance_config_create_action"]["endpoint"],
        "/api/v1/social_insurance_config"
    );
    assert_eq!(
        body["insurance_config_create_action"]["label"],
        "保存社保配置"
    );
    assert_eq!(
        body["insurance_config_create_action"]["success_message"],
        "社保公积金配置已保存"
    );
    assert_eq!(body["empty_message"], "未找到匹配的记录");
    assert_eq!(body["current_insurance_config"]["id"], "cfg-current");
    assert_eq!(body["current_insurance_config"]["city"], "Hangzhou Current");
    assert_eq!(
        body["current_insurance_config"]["update_action"]["method"],
        "PUT"
    );
    assert_eq!(
        body["current_insurance_config"]["update_action"]["endpoint"],
        "/api/v1/social_insurance_config/cfg-current"
    );
    assert_eq!(
        body["current_insurance_config"]["update_action"]["label"],
        "保存社保配置"
    );
    assert_eq!(
        body["current_insurance_config"]["update_action"]["success_message"],
        "社保公积金配置已保存"
    );
    assert_eq!(
        body["current_insurance_config"]["social_security_base_min_input"],
        "5000.00"
    );
    assert_eq!(
        body["current_insurance_config"]["housing_fund_company_rate_input"],
        "12.00"
    );

    let salary = &body["salaries"].as_array().unwrap()[0];
    assert_eq!(salary["employee_name"], "Needle Employee");
    assert_eq!(salary["base_salary_label"], "¥1,000");
    assert_eq!(salary["bonus_label"], "+¥200");
    assert_eq!(salary["deduction_label"], "-¥50");
    assert_eq!(salary["social_security_label"], "¥30");
    assert_eq!(salary["social_security_company_label"], "¥60");
    assert_eq!(salary["housing_fund_label"], "¥20");
    assert_eq!(salary["housing_fund_company_label"], "¥40");
    assert_eq!(salary["actual_salary_label"], "¥1,100");
    assert_eq!(salary["status_badge"]["label"], "待发放");
    assert_eq!(salary["update_action"]["method"], "PUT");
    assert_eq!(
        salary["update_action"]["endpoint"],
        "/api/v1/salary/salary-needle"
    );
    assert_eq!(salary["update_action"]["label"], "保存修改");
    assert_eq!(salary["update_action"]["success_title"], "更新成功");
    assert_eq!(salary["update_action"]["success_message"], "工资记录已更新");
    assert_eq!(salary["can_mark_paid"], true);
    assert_eq!(salary["mark_paid_action"]["method"], "POST");
    assert_eq!(
        salary["mark_paid_action"]["endpoint"],
        "/api/v1/salary/salary-needle/mark_paid"
    );
    assert_eq!(salary["mark_paid_action"]["label"], "标记已发放");
    assert_eq!(
        salary["mark_paid_action"]["success_message"],
        "工资已标记为已发放"
    );
    assert_eq!(salary["can_delete"], true);
    assert_eq!(salary["delete_action"]["method"], "DELETE");
    assert_eq!(
        salary["delete_action"]["endpoint"],
        "/api/v1/salary/salary-needle"
    );
    let period = Utc::now().format("%Y-%m").to_string();
    assert_eq!(
        salary["delete_action"]["confirm_message"],
        format!("确定要删除工资记录「Needle Employee {period}」吗？")
    );
    assert_eq!(salary["delete_action"]["success_title"], "删除成功");
    assert_eq!(salary["delete_action"]["success_message"], "工资记录已删除");

    assert!(
        body["monthly_trend"]
            .as_array()
            .unwrap()
            .iter()
            .any(|point| point["total_label"] == "¥1,200")
    );
}

#[tokio::test]
async fn s107_salary_update_paid_sets_payment_date_on_backend() {
    let app = common::TestApp::new().await;
    seed_employee(&app).await;
    seed_salary(&app).await;

    let res = app
        .put("/api/v1/salary/salary-needle")
        .json(&json!({ "status": "paid" }))
        .await;
    res.assert_status_ok();

    let get = app.get("/api/v1/salary/salary-needle").await;
    get.assert_status_ok();
    let salary = get.json::<Value>();
    assert_eq!(salary["status"], "paid");
    assert_eq!(
        salary["payment_date"],
        Utc::now().format("%Y-%m-%d").to_string()
    );
}

#[tokio::test]
async fn s107_salary_mark_paid_is_backend_owned() {
    let app = common::TestApp::new().await;
    seed_employee(&app).await;
    seed_salary(&app).await;

    let res = app
        .post("/api/v1/salary/salary-needle/mark_paid")
        .json(&json!({}))
        .await;
    res.assert_status_ok();
    let salary = res.json::<Value>();
    assert_eq!(salary["status"], "paid");
    assert_eq!(
        salary["payment_date"],
        Utc::now().format("%Y-%m-%d").to_string()
    );
    assert_eq!(salary["status_badge"]["label"], "已发放");
    assert_eq!(salary["can_mark_paid"], false);
    assert!(salary["mark_paid_action"].is_null());
}

#[tokio::test]
async fn s107_salary_delete_requires_backend_permission() {
    let app = common::TestApp::new().await;
    seed_employee(&app).await;
    seed_salary(&app).await;
    sqlx::query("UPDATE company_members SET role = 'member' WHERE user_id = ? AND company_id = ?")
        .bind(&app.user_id)
        .bind(&app.company_id)
        .execute(&app.pool)
        .await
        .expect("downgrade member");

    let delete = app.delete("/api/v1/salary/salary-needle").await;
    delete.assert_status_forbidden();
}
