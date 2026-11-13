use crate::common;

use chrono::Utc;
use serde_json::{Value, json};

#[tokio::test]
async fn s102_budget_overview_returns_display_labels_badges_and_permissions() {
    let app = common::TestApp::new().await;
    let period = Utc::now().format("%Y-%m").to_string();

    let employee_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO employees (id, company_id, name, status) VALUES (?, ?, 'Budget Display Employee', 'active')",
    )
    .bind(&employee_id)
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO salary_records (id, company_id, employee_id, period, base_salary, bonus)
         VALUES (?, ?, ?, ?, 400, 100)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .bind(&employee_id)
    .bind(&period)
    .execute(&app.pool)
    .await
    .unwrap();

    app.post("/api/budgets")
        .json(&json!({
            "category": "salary",
            "budget_type": "monthly",
            "period": period,
            "budget_amount": 400.0
        }))
        .await
        .assert_status_ok();

    let res = app
        .get(&format!(
            "/api/budgets/overview?budget_type=monthly&period={}",
            period
        ))
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total_budget_label"], "¥400");
    assert_eq!(body["stats"]["total_actual_label"], "¥500");
    assert_eq!(body["stats"]["saved_amount_label"], "¥0");
    assert_eq!(body["stats"]["utilization_label"], "125%");
    assert_eq!(body["permissions"]["can_delete"], true);
    assert_eq!(body["permissions"]["can_create"], true);
    assert_eq!(body["permissions"]["can_update"], true);
    assert_eq!(body["create_action"]["method"], "POST");
    assert_eq!(body["create_action"]["endpoint"], "/api/budgets");
    assert_eq!(body["create_action"]["label"], "新建预算");
    assert_eq!(body["create_action"]["success_title"], "创建成功");
    assert_eq!(body["create_action"]["success_message"], "预算已创建");
    assert!(
        body["budget_type_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "quarterly" && option["label"] == "季度预算")
    );

    let form_options = app.get("/api/budgets/form_options").await;
    form_options.assert_status_ok();
    let form_options = form_options.json::<Value>();
    assert_eq!(form_options["default_values"]["budget_type"], "monthly");
    assert_eq!(form_options["default_values"]["category"], "salary");
    assert!(
        form_options["budget_type_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "quarterly" && option["label"] == "季度预算")
    );
    assert!(
        form_options["category_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "utility" && option["label"] == "水电支出")
    );

    let budget = &body["budgets"].as_array().unwrap()[0];
    assert_eq!(budget["category_label"], "工资支出");
    assert_eq!(budget["category_icon"], "Users");
    assert_eq!(budget["category_class"], "text-primary");
    assert_eq!(budget["budget_amount_label"], "¥400");
    assert_eq!(budget["actual_amount_label"], "¥500");
    assert_eq!(budget["usage_label"], "125% 已使用");
    assert_eq!(budget["remaining_label"], "超支 ¥100");
    assert_eq!(budget["progress_value"].as_f64().unwrap(), 100.0);
    assert_eq!(budget["progress_class"], "[&>div]:bg-destructive");
    assert_eq!(budget["status_badge"]["label"], "超支");
    assert_eq!(budget["status_badge"]["variant"], "destructive");
    assert_eq!(budget["status_badge"]["icon"], "AlertTriangle");
    let budget_id = budget["id"].as_str().unwrap();
    assert_eq!(budget["update_action"]["method"], "PUT");
    assert_eq!(
        budget["update_action"]["endpoint"],
        format!("/api/budgets/{budget_id}")
    );
    assert_eq!(budget["update_action"]["label"], "保存修改");
    assert_eq!(budget["update_action"]["success_title"], "更新成功");
    assert_eq!(budget["update_action"]["success_message"], "预算已更新");
    assert_eq!(budget["can_delete"], true);
    assert_eq!(budget["delete_action"]["method"], "DELETE");
    assert_eq!(
        budget["delete_action"]["endpoint"],
        format!("/api/budgets/{budget_id}")
    );
    assert_eq!(
        budget["delete_action"]["confirm_message"],
        format!("确定要删除预算「工资支出 {period}」吗？")
    );
    assert_eq!(budget["delete_action"]["success_title"], "删除成功");
    assert_eq!(budget["delete_action"]["success_message"], "预算已删除");

    let trend = body["trend"].as_array().unwrap();
    assert!(trend.iter().any(|point| {
        point["period"] == period
            && point["period_label"].as_str().unwrap().ends_with('月')
            && point["budget_label"] == "¥400"
            && point["actual_label"] == "¥500"
    }));
}
