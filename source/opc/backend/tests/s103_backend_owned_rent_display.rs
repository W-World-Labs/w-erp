use crate::common;

use chrono::Utc;
use serde_json::Value;

async fn seed_rent(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO rent_records (
            id, company_id, location_name, property_type, address, area,
            monthly_rent, contract_start_date, contract_end_date,
            next_payment_date, status
        )
        VALUES (
            'rent-needle', ?, 'Needle HQ', 'office', 'Needle Road', 88.5,
            1200.0, '2026-01-01', '2026-12-31', '2026-06-01', 'active'
        )
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed rent");
}

async fn seed_utility(app: &common::TestApp) {
    let period = Utc::now().format("%Y-%m").to_string();
    sqlx::query(
        r#"
        INSERT INTO utilities (
            id, company_id, location_name, utility_type, period,
            usage, unit_price, amount, due_date, status
        )
        VALUES (
            'utility-needle', ?, 'Needle HQ', 'electricity', ?,
            30.0, 1.25, 37.5, '2026-06-20', 'pending'
        )
        "#,
    )
    .bind(&app.company_id)
    .bind(period)
    .execute(&app.pool)
    .await
    .expect("seed utility");
}

#[tokio::test]
async fn s103_rent_overview_returns_filtered_rows_labels_badges_and_permissions() {
    let app = common::TestApp::new().await;
    seed_rent(&app).await;
    seed_utility(&app).await;

    let res = app
        .get("/api/v1/rent/overview?rent_search=needle&utility_search=electric")
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["rent_stats"]["total_label"], "¥1,200");
    assert_eq!(body["rent_stats"]["area_label"], "88.5㎡");
    assert_eq!(body["utility_stats"]["total_label"], "¥37.50");
    assert_eq!(body["permissions"]["can_delete"], true);
    assert_eq!(body["permissions"]["can_create"], true);
    assert_eq!(body["permissions"]["can_update"], true);
    assert_eq!(body["rent_create_action"]["method"], "POST");
    assert_eq!(body["rent_create_action"]["endpoint"], "/api/v1/rent");
    assert_eq!(body["rent_create_action"]["label"], "添加场地");
    assert_eq!(body["rent_create_action"]["success_title"], "创建成功");
    assert_eq!(body["rent_create_action"]["success_message"], "场地已添加");
    assert_eq!(body["utility_create_action"]["method"], "POST");
    assert_eq!(
        body["utility_create_action"]["endpoint"],
        "/api/v1/utilities"
    );
    assert_eq!(body["utility_create_action"]["label"], "录入账单");
    assert_eq!(body["utility_create_action"]["success_title"], "创建成功");
    assert_eq!(
        body["utility_create_action"]["success_message"],
        "账单已添加"
    );

    let rents = body["rents"].as_array().unwrap();
    assert_eq!(rents.len(), 1);
    let rent = &rents[0];
    assert_eq!(rent["location_name"], "Needle HQ");
    assert_eq!(rent["monthly_rent_label"], "¥1,200");
    assert_eq!(rent["area_label"], "88.5㎡");
    assert_eq!(rent["contract_period_label"], "2026-01-01 至 2026-12-31");
    assert_eq!(rent["next_payment_label"], "2026-06-01");
    assert_eq!(rent["status_badge"]["label"], "活跃");
    assert_eq!(rent["update_action"]["method"], "PUT");
    assert_eq!(
        rent["update_action"]["endpoint"],
        "/api/v1/rent/rent-needle"
    );
    assert_eq!(rent["update_action"]["label"], "保存修改");
    assert_eq!(rent["update_action"]["success_title"], "更新成功");
    assert_eq!(rent["update_action"]["success_message"], "场地信息已更新");
    assert_eq!(rent["can_delete"], true);
    assert_eq!(
        rent["delete_action"]["endpoint"],
        "/api/v1/rent/rent-needle"
    );
    assert_eq!(rent["delete_action"]["success_message"], "场地已删除");
    assert_eq!(
        rent["delete_action"]["confirm_message"],
        "此操作无法撤销。确定要删除这个场地吗？"
    );

    let utilities = body["utilities"].as_array().unwrap();
    assert_eq!(utilities.len(), 1);
    let utility = &utilities[0];
    assert_eq!(utility["utility_type_icon"], "Zap");
    assert_eq!(utility["utility_type_label"], "电费");
    assert_eq!(utility["usage_label"], "30");
    assert_eq!(utility["unit_price_label"], "¥1.25");
    assert_eq!(utility["amount_label"], "¥37.50");
    assert_eq!(utility["due_date_label"], "2026-06-20");
    assert_eq!(utility["status_badge"]["label"], "待支付");
    assert_eq!(utility["update_action"]["method"], "PUT");
    assert_eq!(
        utility["update_action"]["endpoint"],
        "/api/v1/utilities/utility-needle"
    );
    assert_eq!(utility["update_action"]["label"], "保存修改");
    assert_eq!(utility["update_action"]["success_title"], "更新成功");
    assert_eq!(utility["update_action"]["success_message"], "账单已更新");
    assert_eq!(utility["can_mark_paid"], true);
    assert_eq!(utility["mark_paid_action"]["method"], "POST");
    assert_eq!(
        utility["mark_paid_action"]["endpoint"],
        "/api/v1/utilities/utility-needle/mark_paid"
    );
    assert_eq!(utility["mark_paid_action"]["label"], "标记已支付");
    assert_eq!(
        utility["mark_paid_action"]["success_message"],
        "账单已标记为已支付"
    );
    assert_eq!(utility["can_delete"], true);
    assert_eq!(
        utility["delete_action"]["endpoint"],
        "/api/v1/utilities/utility-needle"
    );
    assert_eq!(utility["delete_action"]["success_message"], "账单已删除");
    assert_eq!(
        utility["delete_action"]["confirm_message"],
        "此操作无法撤销。确定要删除这条账单吗？"
    );

    assert!(
        body["rent_trend"]
            .as_array()
            .unwrap()
            .iter()
            .any(|point| point["total_label"] == "¥1,200")
    );
    assert!(
        body["utility_trend"]
            .as_array()
            .unwrap()
            .iter()
            .any(|point| point["electricity_label"] == "¥37.50")
    );
}

#[tokio::test]
async fn s103_rent_form_options_are_backend_owned() {
    let app = common::TestApp::new().await;

    let res = app.get("/api/v1/rent/form_options").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["default_values"]["payment_cycle"], "monthly");
    assert_eq!(body["default_values"]["utility_type"], "electricity");
    assert_eq!(body["payment_cycle_options"][0]["value"], "monthly");
    assert_eq!(body["payment_cycle_options"][0]["label"], "月付");
    assert_eq!(body["payment_cycle_options"][2]["value"], "yearly");
    assert_eq!(body["utility_type_options"][0]["value"], "electricity");
    assert_eq!(body["utility_type_options"][0]["label"], "电费");
}

#[tokio::test]
async fn s103_utility_mark_paid_is_backend_owned() {
    let app = common::TestApp::new().await;
    seed_utility(&app).await;

    let res = app
        .post("/api/v1/utilities/utility-needle/mark_paid")
        .json(&serde_json::json!({}))
        .await;
    res.assert_status_ok();
    let utility = res.json::<Value>();
    assert_eq!(utility["status"], "paid");
    assert_eq!(utility["status_badge"]["label"], "已支付");
    assert_eq!(utility["can_mark_paid"], false);
    assert!(utility["mark_paid_action"].is_null());

    let overview = app.get("/api/v1/rent/overview").await;
    overview.assert_status_ok();
    let overview = overview.json::<Value>();
    assert_eq!(overview["utility_stats"]["total_label"], "¥0");
}
