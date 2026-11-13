use crate::common;

use serde_json::Value;

async fn seed_company_info(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO company_info (
            id, company_id, company_name, legal_representative,
            unified_social_credit_code, registration_date, registered_capital,
            founder_education, founder_is_expat, founder_is_overseas_returnee
        )
        VALUES (
            'company-info-needle', ?, 'Backend Corp', 'Backend CEO',
            'USCC-001', '2026-05-09', 1000000.0,
            'masters', 1, 1
        )
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed company info");
}

async fn seed_branch(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO branches (
            id, company_id, branch_name, branch_type, legal_representative,
            branch_address, registration_date, status
        )
        VALUES (
            'branch-needle', ?, 'Backend Branch', 'subsidiary', 'Branch CEO',
            'Branch Road', '2026-06-01', 'active'
        )
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed branch");
}

async fn seed_milestone(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO milestones (
            id, company_id, title, description, milestone_type, milestone_date
        )
        VALUES (
            'milestone-needle', ?, 'Backend Milestone', 'Backend owned timeline',
            'milestone', '2026-07-01'
        )
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed milestone");
}

#[tokio::test]
async fn s106_company_info_overview_returns_display_labels_timeline_and_permissions() {
    let app = common::TestApp::new().await;
    seed_company_info(&app).await;
    seed_branch(&app).await;
    seed_milestone(&app).await;

    let res = app
        .get("/api/v1/company_info/overview?timeline_type=milestone")
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["company"]["company_name"], "Backend Corp");
    assert_eq!(body["company"]["record_badge"]["label"], "已录入");
    assert_eq!(body["company"]["registration_date_label"], "2026-05-09");
    assert_eq!(body["company"]["registered_capital_label"], "¥1,000,000");
    assert_eq!(body["company_create_action"]["method"], "POST");
    assert_eq!(
        body["company_create_action"]["endpoint"],
        "/api/v1/company_info"
    );
    assert_eq!(body["company_create_action"]["label"], "添加公司信息");
    assert_eq!(
        body["company_create_action"]["success_message"],
        "公司信息添加成功"
    );
    assert_eq!(body["company"]["update_action"]["method"], "PUT");
    assert_eq!(
        body["company"]["update_action"]["endpoint"],
        "/api/v1/company_info/company-info-needle"
    );
    assert_eq!(body["company"]["update_action"]["label"], "保存信息");
    assert_eq!(
        body["company"]["update_action"]["success_message"],
        "公司信息更新成功"
    );
    assert_eq!(body["branch_create_action"]["method"], "POST");
    assert_eq!(body["branch_create_action"]["endpoint"], "/api/v1/branches");
    assert_eq!(body["branch_create_action"]["label"], "添加分支机构");
    assert_eq!(
        body["branch_create_action"]["success_message"],
        "分支机构添加成功"
    );
    assert_eq!(body["milestone_create_action"]["method"], "POST");
    assert_eq!(
        body["milestone_create_action"]["endpoint"],
        "/api/v1/milestones"
    );
    assert_eq!(body["milestone_create_action"]["label"], "添加里程碑");
    assert_eq!(
        body["milestone_create_action"]["success_message"],
        "里程碑创建成功"
    );
    assert!(
        body["company"]["founder_badges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|badge| badge["label"] == "硕士 (Master)")
    );
    assert_eq!(body["permissions"]["can_delete_branch"], true);

    let company_form_options = app.get("/api/v1/company_info/form_options").await;
    company_form_options.assert_status_ok();
    let company_form_options = company_form_options.json::<Value>();
    assert_eq!(
        company_form_options["default_values"]["taxpayer_type"],
        "small_scale"
    );
    assert_eq!(company_form_options["default_values"]["vat_rate"], "3");
    assert!(
        company_form_options["taxpayer_type_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "general" && option["label"] == "一般纳税人")
    );
    assert!(
        company_form_options["founder_education_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "masters" && option["label"] == "硕士 (Master's)")
    );

    let branch = &body["branches"].as_array().unwrap()[0];
    assert_eq!(branch["branch_name"], "Backend Branch");
    assert_eq!(branch["branch_type_badge"]["label"], "子公司");
    assert_eq!(branch["status_badge"]["label"], "运营中");
    assert_eq!(branch["can_delete"], true);
    assert_eq!(branch["can_activate"], false);
    assert_eq!(branch["can_mark_preparing"], true);
    assert_eq!(branch["can_suspend"], true);
    assert_eq!(branch["can_close"], true);
    assert_eq!(branch["update_action"]["method"], "PUT");
    assert_eq!(
        branch["update_action"]["endpoint"],
        "/api/v1/branches/branch-needle"
    );
    assert_eq!(branch["update_action"]["label"], "保存修改");
    assert_eq!(
        branch["update_action"]["success_message"],
        "分支机构更新成功"
    );
    assert!(branch["activate_action"].is_null());
    assert_eq!(branch["mark_preparing_action"]["method"], "POST");
    assert_eq!(
        branch["mark_preparing_action"]["endpoint"],
        "/api/v1/branches/branch-needle/mark_preparing"
    );
    assert_eq!(branch["mark_preparing_action"]["label"], "标记筹建中");
    assert_eq!(
        branch["mark_preparing_action"]["success_message"],
        "分支机构已标记为筹建中"
    );
    assert_eq!(
        branch["suspend_action"]["endpoint"],
        "/api/v1/branches/branch-needle/suspend"
    );
    assert_eq!(
        branch["close_action"]["endpoint"],
        "/api/v1/branches/branch-needle/close"
    );
    assert_eq!(
        branch["delete_action"]["endpoint"],
        "/api/v1/branches/branch-needle"
    );
    assert_eq!(
        branch["delete_action"]["confirm_message"],
        "确定要删除分支机构「Backend Branch」吗？此操作无法撤销。"
    );
    assert_eq!(branch["delete_action"]["success_message"], "分支机构已删除");
    assert_eq!(body["parent_branch_options"][0]["value"], "branch-needle");
    assert_eq!(body["parent_branch_options"][0]["label"], "Backend Branch");
    assert!(
        branch["parent_branch_options"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let form_options = app
        .get("/api/v1/branches/form_options?exclude_id=branch-needle")
        .await;
    form_options.assert_status_ok();
    let form_options = form_options.json::<Value>();
    assert_eq!(form_options["default_values"]["branch_type"], "branch");
    assert!(form_options["default_values"]["status"].is_null());
    assert!(form_options["status_options"].is_null());
    assert_eq!(form_options["default_values"]["parent_branch_id"], "none");
    assert!(
        form_options["branch_type_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "subsidiary" && option["label"] == "子公司")
    );
    assert_eq!(form_options["parent_branch_options"][0]["value"], "none");
    assert_eq!(
        form_options["parent_branch_options"][0]["label"],
        "无上级机构"
    );
    assert_eq!(
        form_options["parent_branch_options"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let events = body["timeline"]["events"].as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["title"], "Backend Milestone");
    assert_eq!(events[0]["date_label"], "2026-07-01");
    assert_eq!(events[0]["type"], "milestone");
    assert_eq!(events[0]["record_id"], "milestone-needle");
    assert_eq!(events[0]["update_action"]["method"], "PUT");
    assert_eq!(
        events[0]["update_action"]["endpoint"],
        "/api/v1/milestones/milestone-needle"
    );
    assert_eq!(
        events[0]["update_action"]["success_message"],
        "里程碑更新成功"
    );
}

#[tokio::test]
async fn s106_branch_lifecycle_status_is_backend_owned() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    let create = app
        .post("/api/v1/branches")
        .json(&serde_json::json!({
            "branch_name": "Backend Owned Branch",
            "branch_type": "branch",
            "branch_address": "Original",
            "employee_count": "3",
            "status": "closed"
        }))
        .await;
    create.assert_status_ok();
    let branch_id = create.json::<Value>()["id"].as_str().unwrap().to_string();

    let created = app.get(&format!("/api/v1/branches/{branch_id}")).await;
    created.assert_status_ok();
    let created_body = created.json::<Value>();
    assert_eq!(created_body["status"], "active");

    let update = app
        .put(&format!("/api/v1/branches/{branch_id}"))
        .json(&serde_json::json!({
            "branch_address": "Updated",
            "status": "closed"
        }))
        .await;
    update.assert_status_ok();

    let updated = app.get(&format!("/api/v1/branches/{branch_id}")).await;
    updated.assert_status_ok();
    let updated_body = updated.json::<Value>();
    assert_eq!(updated_body["branch_address"], "Updated");
    assert_eq!(updated_body["status"], "active");

    let preparing = app
        .post(&format!("/api/v1/branches/{branch_id}/mark_preparing"))
        .json(&serde_json::json!({}))
        .await;
    preparing.assert_status_ok();
    let preparing_body = preparing.json::<Value>();
    assert_eq!(preparing_body["status"], "preparing");
    assert_eq!(preparing_body["status_badge"]["label"], "筹建中");
    assert_eq!(preparing_body["can_activate"], true);
    assert_eq!(preparing_body["can_suspend"], false);
    assert_eq!(
        preparing_body["activate_action"]["endpoint"],
        format!("/api/v1/branches/{branch_id}/activate")
    );
    assert!(preparing_body["suspend_action"].is_null());

    let activated = app
        .post(&format!("/api/v1/branches/{branch_id}/activate"))
        .json(&serde_json::json!({}))
        .await;
    activated.assert_status_ok();
    let activated_body = activated.json::<Value>();
    assert_eq!(activated_body["status"], "active");
    assert_eq!(activated_body["status_badge"]["label"], "运营中");
    assert_eq!(activated_body["can_activate"], false);
    assert!(activated_body["activate_action"].is_null());

    let suspended = app
        .post(&format!("/api/v1/branches/{branch_id}/suspend"))
        .json(&serde_json::json!({}))
        .await;
    suspended.assert_status_ok();
    let suspended_body = suspended.json::<Value>();
    assert_eq!(suspended_body["status"], "suspended");
    assert_eq!(suspended_body["status_badge"]["label"], "暂停营业");
    assert_eq!(suspended_body["can_activate"], true);
    assert_eq!(
        suspended_body["activate_action"]["endpoint"],
        format!("/api/v1/branches/{branch_id}/activate")
    );

    let closed = app
        .post(&format!("/api/v1/branches/{branch_id}/close"))
        .json(&serde_json::json!({}))
        .await;
    closed.assert_status_ok();
    let closed_body = closed.json::<Value>();
    assert_eq!(closed_body["status"], "closed");
    assert_eq!(closed_body["status_badge"]["label"], "已关闭");
    assert_eq!(closed_body["can_close"], false);
    assert_eq!(closed_body["can_activate"], false);
    assert!(closed_body["close_action"].is_null());
    assert!(closed_body["activate_action"].is_null());

    let other_get = other.get(&format!("/api/v1/branches/{branch_id}")).await;
    other_get.assert_status_not_found();

    let other_activate = other
        .post(&format!("/api/v1/branches/{branch_id}/activate"))
        .json(&serde_json::json!({}))
        .await;
    other_activate.assert_status_not_found();
}

#[tokio::test]
async fn s106_milestone_form_options_are_backend_owned() {
    let app = common::TestApp::new().await;

    let res = app.get("/api/v1/milestones/form_options").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["default_values"]["importance"], "medium");
    assert_eq!(body["milestone_type_options"][0]["value"], "funding");
    assert_eq!(body["milestone_type_options"][0]["label"], "融资");
    assert_eq!(body["importance_options"][1]["value"], "medium");
    assert_eq!(body["importance_options"][1]["label"], "中");
}

#[tokio::test]
async fn s106_branch_delete_requires_backend_permission() {
    let app = common::TestApp::new().await;
    seed_branch(&app).await;
    sqlx::query("UPDATE company_members SET role = 'member' WHERE user_id = ? AND company_id = ?")
        .bind(&app.user_id)
        .bind(&app.company_id)
        .execute(&app.pool)
        .await
        .expect("downgrade member");

    let delete = app.delete("/api/v1/branches/branch-needle").await;
    delete.assert_status_forbidden();
}
