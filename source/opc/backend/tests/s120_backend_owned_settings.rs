use crate::common;

use serde_json::Value;

async fn seed_company_info(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO company_info (
            id, company_id, company_name, unified_social_credit_code,
            company_address, contact_phone
        )
        VALUES (
            'company-info-settings', ?, 'Settings Co',
            'USCC-SETTINGS', 'Settings Road', '010-55556666'
        )
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed company info");
}

async fn seed_category(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO asset_categories (id, company_id, name)
        VALUES ('category-settings', ?, '设备')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed category");
}

async fn seed_branch(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO branches (id, company_id, branch_name)
        VALUES ('branch-settings', ?, '上海分部')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed branch");
}

async fn seed_ai_configuration(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO ai_configurations (id, company_id, provider, model, api_key, endpoint, enabled)
        VALUES ('ai-settings', ?, 'DeepSeek', 'deepseek-v4-flash', 'sk-test-placeholder', 'https://api.deepseek.com/v1/chat/completions', 1)
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed ai configuration");
}

async fn seed_email_configuration(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO email_configurations (
            id, company_id, smtp_host, smtp_port, smtp_user, smtp_password,
            sender_name, sender_email, enabled
        )
        VALUES (
            'email-settings', ?, 'smtp.example.com', 587, 'mailer',
            'resend-secret-value', 'Ops Mail', 'ops@example.com', 1
        )
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed email configuration");
}

#[tokio::test]
async fn s120_settings_overview_returns_setup_completion_and_config_display_fields() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_company_info(&app).await;
    common::seed::create_employee(&app.pool, &app.company_id, "Settings Employee", 1000.0).await;
    seed_category(&app).await;
    seed_ai_configuration(&app).await;
    seed_email_configuration(&app).await;
    seed_branch(&other).await;

    let res = app.get("/api/v1/settings/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    let completion = &body["setup_completion"];
    assert_eq!(completion["completed_count"].as_i64().unwrap(), 3);
    assert_eq!(completion["total_count"].as_i64().unwrap(), 4);
    assert_eq!(completion["rate"].as_i64().unwrap(), 75);
    assert_eq!(completion["rate_label"], "75%");
    assert_eq!(completion["summary_label"], "已完成 3 / 4 项基础数据配置");
    assert_eq!(completion["needs_attention"].as_bool(), Some(true));
    assert_eq!(completion["success_alert"]["visible"], false);
    assert_eq!(completion["success_alert"]["title"], "恭喜！");
    assert_eq!(
        completion["success_alert"]["class_name"],
        "border-green-200 bg-green-50/50"
    );
    assert!(
        completion["recommendation_items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item == "配置分支机构，完善组织架构")
    );

    let checks = body["setup_checks"].as_array().unwrap();
    assert_eq!(checks.len(), 4);
    let branches = checks
        .iter()
        .find(|check| check["module"] == "分支机构")
        .expect("branch check");
    assert_eq!(branches["status"].as_bool(), Some(false));
    assert_eq!(branches["status_label"], "待完善");
    assert_eq!(
        branches["card_class_name"],
        "border-orange-200 bg-orange-50/50"
    );
    assert_eq!(branches["status_icon"], "AlertCircle");
    assert_eq!(
        branches["status_icon_class_name"],
        "h-5 w-5 text-orange-600 mt-0.5"
    );
    assert_eq!(branches["status_badge"]["label"], "待完善");
    assert_eq!(
        branches["status_badge"]["class_name"],
        "bg-orange-100 text-orange-700"
    );
    assert_eq!(branches["action_button"]["visible"], true);
    assert_eq!(branches["action_button"]["label"], "前往完善");
    assert_eq!(branches["action_button"]["route"], "/branches");
    assert_eq!(branches["impact_label"], "组织管理");

    assert_eq!(
        body["ai_configuration_create_action"]["endpoint"],
        "/api/v1/ai_configurations"
    );
    assert_eq!(body["ai_configuration_create_action"]["method"], "POST");
    let ai = &body["ai_configurations"][0];
    assert_eq!(ai["id"], "ai-settings");
    assert_eq!(ai["display_name"], "DeepSeek / deepseek-v4-flash");
    assert_eq!(
        ai["endpoint_label"],
        opc::runtime_config::DEEPSEEK_CHAT_COMPLETIONS_URL
    );
    assert_eq!(ai["api_key_label"], "sk-test-pl...");
    assert_eq!(ai["enabled_badge"]["label"], "启用");
    assert_eq!(ai["is_default"], true);
    assert_eq!(
        ai["delete_action"]["endpoint"],
        "/api/v1/ai_configurations/ai-settings"
    );
    assert_eq!(
        ai["update_action"]["endpoint"],
        "/api/v1/ai_configurations/ai-settings"
    );
    assert_eq!(ai["update_action"]["method"], "PUT");
    assert_eq!(
        ai["delete_action"]["confirm_message"],
        "确定要删除这个 AI 配置吗？"
    );
    assert_eq!(ai["delete_action"]["success_message"], "AI 配置已删除");

    assert_eq!(
        body["email_configuration_create_action"]["endpoint"],
        "/api/v1/email_configurations"
    );
    assert_eq!(body["email_configuration_create_action"]["method"], "POST");
    let email = &body["email_configurations"][0];
    assert_eq!(email["id"], "email-settings");
    assert_eq!(email["display_name"], "Ops Mail");
    assert_eq!(email["sender_label"], "Ops Mail <ops@example.com>");
    assert_eq!(email["api_key_label"], "resend-sec...");
    assert_eq!(email["enabled_badge"]["label"], "启用");
    assert_eq!(email["from_email"], "ops@example.com");
    assert_eq!(
        email["delete_action"]["endpoint"],
        "/api/v1/email_configurations/email-settings"
    );
    assert_eq!(
        email["update_action"]["endpoint"],
        "/api/v1/email_configurations/email-settings"
    );
    assert_eq!(email["update_action"]["method"], "PUT");
    assert_eq!(
        email["delete_action"]["confirm_message"],
        "确定要删除这个邮件配置吗？"
    );
    assert_eq!(email["delete_action"]["success_message"], "邮件配置已删除");

    let sections = body["settings_sections"].as_array().unwrap();
    let company_section = sections
        .iter()
        .find(|section| section["id"] == "company-info")
        .expect("company info settings section");
    assert_eq!(company_section["title"], "企业信息");
    assert_eq!(company_section["fields"][0]["value"], "Settings Co");
    assert_eq!(company_section["fields"][1]["value"], "USCC-SETTINGS");
    assert_eq!(company_section["fields"][2]["value"], "Settings Road");
    assert_eq!(company_section["fields"][3]["value"], "010-55556666");

    let department_section = sections
        .iter()
        .find(|section| section["id"] == "department-management")
        .expect("department section");
    assert_eq!(department_section["items"][0]["label"], "Engineering");
    assert_eq!(department_section["items"][0]["action"]["label"], "编辑");

    // The notification section is channel-based since the w-notify delivery
    // work: it lists configured webhook channels instead of local toggles.
    let notification_section = sections
        .iter()
        .find(|section| section["id"] == "notification-settings")
        .expect("notification settings section");
    assert_eq!(notification_section["layout"], "items");
    assert!(
        notification_section["toggles"]
            .as_array()
            .expect("toggles array")
            .is_empty()
    );
    assert_eq!(
        notification_section["empty_message"],
        "尚未配置通知渠道，请在「通知渠道」卡片添加 Webhook"
    );

    let data_section = sections
        .iter()
        .find(|section| section["id"] == "data-management")
        .expect("data management section");
    assert_eq!(data_section["summary_items"], serde_json::json!([]));
    assert_eq!(data_section["actions"], serde_json::json!([]));

    let employee_dependency = app.get("/api/v1/dependencies/employees").await;
    employee_dependency.assert_status_ok();
    let employee_dependency_body = employee_dependency.json::<Value>();
    assert_eq!(employee_dependency_body["exists"], true);
    assert_eq!(employee_dependency_body["count"], 1);
    assert_eq!(employee_dependency_body["action_text"], "前往添加员工");

    let branch_dependency = app.get("/api/v1/dependencies/branches").await;
    branch_dependency.assert_status_ok();
    let branch_dependency_body = branch_dependency.json::<Value>();
    assert_eq!(branch_dependency_body["exists"], false);
    assert_eq!(branch_dependency_body["message"], "建议先添加分支机构信息");
    assert_eq!(branch_dependency_body["navigate_to"], "/branches");
}
