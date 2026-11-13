use crate::common;

use serde_json::Value;

async fn seed_provider(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO cloud_providers (
            id, company_id, provider_name, provider_type, account_name,
            account_id, region, contact_person
        )
        VALUES ('provider-needle', ?, 'Needle Cloud', 'aws', 'Main',
            'acct-1', 'ap-east-1', 'Ops')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed provider");
}

async fn seed_resource(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO cloud_resources (
            id, company_id, provider_id, resource_type, resource_name,
            specification, region, start_date, end_date, monthly_cost, status
        )
        VALUES ('resource-needle', ?, 'provider-needle', 'compute', 'Needle ECS',
            '2c4g', 'ap-east-1', '2026-01-01', '2026-06-01', 123.45, 'running')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed resource");
}

async fn seed_bill(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO cloud_bills (
            id, company_id, provider_id, billing_period, total_amount, status
        )
        VALUES ('bill-needle', ?, 'provider-needle', '2026-05', 88.8, 'pending')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed bill");
}

async fn seed_domain(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO domain_subscriptions (
            id, company_id, domain_name, registrar, registration_date,
            expiry_date, initial_purchase_cost, annual_fee, auto_renewal, status
        )
        VALUES ('domain-needle', ?, 'needle.example', 'Registrar', '2026-01-01',
            '2026-06-01', 20.0, 120.0, 1, 'active')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed domain");
}

async fn seed_software(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO software_subscriptions (
            id, company_id, software_name, vendor, subscription_type, plan_name,
            license_count, total_price, billing_cycle, start_date,
            next_billing_date, auto_renewal, status
        )
        VALUES ('software-needle', ?, 'Needle SaaS', 'Vendor', 'saas', 'Team',
            5, 300.0, 'quarterly', '2026-01-01', '2026-04-01', 0, 'trial')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed software");
}

#[tokio::test]
async fn s111_cloud_cost_overview_returns_rows_labels_badges_and_permissions() {
    let app = common::TestApp::new().await;
    seed_provider(&app).await;
    seed_resource(&app).await;
    seed_bill(&app).await;
    seed_domain(&app).await;
    seed_software(&app).await;

    let res = app.get("/api/v1/cloud_cost/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["monthly_cloud_cost_label"], "¥123.45");
    assert_eq!(body["monthly_domain_cost_label"], "¥10");
    assert_eq!(body["monthly_software_cost_label"], "¥100");
    assert_eq!(body["total_monthly_cost_label"], "¥233.45");
    assert_eq!(body["permissions"]["can_manage"], true);
    assert_eq!(
        body["empty_messages"]["providers"],
        "还没有添加云服务商账户"
    );

    let provider = &body["providers"][0];
    assert_eq!(provider["provider_type_label"], "AWS");
    assert_eq!(provider["status_badge"]["label"], "正常");
    assert_eq!(provider["can_delete"], true);
    assert_eq!(
        provider["delete_action"]["endpoint"],
        "/api/v1/cloud_providers/provider-needle"
    );
    assert_eq!(
        provider["delete_action"]["confirm_message"],
        "确定要删除云服务商「Needle Cloud」吗？相关云资源或账单存在时删除将被后端阻止。"
    );
    assert_eq!(
        provider["delete_action"]["success_message"],
        "云服务商已删除"
    );
    assert_eq!(body["provider_options"][0]["value"], "provider-needle");
    assert_eq!(body["provider_options"][0]["label"], "Needle Cloud");

    let resource = &body["resources"][0];
    assert_eq!(resource["provider_name"], "Needle Cloud");
    assert_eq!(resource["resource_type_label"], "计算");
    assert_eq!(resource["monthly_cost_label"], "¥123.45");
    assert_eq!(resource["end_date_label"], "2026-06-01");
    assert_eq!(resource["status_badge"]["label"], "运行中");
    assert_eq!(resource["can_start"], false);
    assert_eq!(resource["can_stop"], true);
    assert_eq!(resource["can_terminate"], true);
    assert_eq!(resource["can_mark_expired"], true);
    assert!(resource["start_action"].is_null());
    assert_eq!(resource["stop_action"]["method"], "POST");
    assert_eq!(
        resource["stop_action"]["endpoint"],
        "/api/v1/cloud_resources/resource-needle/stop"
    );
    assert_eq!(resource["stop_action"]["label"], "停止");
    assert_eq!(resource["stop_action"]["success_message"], "云资源已停止");
    assert_eq!(
        resource["terminate_action"]["endpoint"],
        "/api/v1/cloud_resources/resource-needle/terminate"
    );
    assert_eq!(
        resource["mark_expired_action"]["endpoint"],
        "/api/v1/cloud_resources/resource-needle/mark_expired"
    );
    assert_eq!(
        resource["delete_action"]["endpoint"],
        "/api/v1/cloud_resources/resource-needle"
    );
    assert_eq!(
        resource["delete_action"]["confirm_message"],
        "确定要删除云资源「Needle ECS」吗？此操作无法撤销。"
    );
    assert_eq!(resource["delete_action"]["success_message"], "云资源已删除");

    let bill = &body["bills"][0];
    assert_eq!(bill["billing_period_label"], "2026-05");
    assert_eq!(bill["total_amount_label"], "¥88.80");
    assert_eq!(bill["status_badge"]["label"], "待支付");
    assert_eq!(body["bill_create_action"]["method"], "POST");
    assert_eq!(
        body["bill_create_action"]["endpoint"],
        "/api/v1/cloud_bills"
    );
    assert_eq!(body["bill_create_action"]["label"], "录入账单");
    assert_eq!(
        body["bill_create_action"]["success_message"],
        "云账单已添加"
    );
    assert_eq!(bill["update_action"]["method"], "PUT");
    assert_eq!(
        bill["update_action"]["endpoint"],
        "/api/v1/cloud_bills/bill-needle"
    );
    assert_eq!(bill["update_action"]["label"], "保存账单");
    assert_eq!(bill["update_action"]["success_message"], "云账单已更新");
    assert_eq!(bill["can_mark_paid"], true);
    assert_eq!(bill["mark_paid_action"]["method"], "POST");
    assert_eq!(
        bill["mark_paid_action"]["endpoint"],
        "/api/v1/cloud_bills/bill-needle/mark_paid"
    );
    assert_eq!(bill["mark_paid_action"]["label"], "标记已支付");
    assert_eq!(
        bill["mark_paid_action"]["success_message"],
        "云账单已标记为已支付"
    );
    assert_eq!(
        bill["delete_action"]["endpoint"],
        "/api/v1/cloud_bills/bill-needle"
    );
    assert_eq!(
        bill["delete_action"]["confirm_message"],
        "确定要删除云账单「Needle Cloud 2026-05」吗？此操作无法撤销。"
    );
    assert_eq!(bill["delete_action"]["success_message"], "云账单已删除");

    let domain = &body["domains"][0];
    assert_eq!(domain["initial_purchase_cost_label"], "¥20");
    assert_eq!(domain["annual_fee_label"], "¥120");
    assert_eq!(domain["auto_renewal_badge"]["icon"], "CheckCircle2");
    assert!(
        domain["expiry_state"]["label"]
            .as_str()
            .unwrap()
            .contains("天")
            || domain["expiry_state"]["label"] == "已过期"
    );
    assert_eq!(domain["can_activate"], false);
    assert_eq!(domain["can_mark_expired"], true);
    assert_eq!(domain["can_mark_pending_transfer"], true);
    assert!(domain["activate_action"].is_null());
    assert_eq!(
        domain["mark_expired_action"]["endpoint"],
        "/api/v1/domain_subscriptions/domain-needle/mark_expired"
    );
    assert_eq!(
        domain["mark_pending_transfer_action"]["endpoint"],
        "/api/v1/domain_subscriptions/domain-needle/mark_pending_transfer"
    );
    assert_eq!(
        domain["delete_action"]["endpoint"],
        "/api/v1/domain_subscriptions/domain-needle"
    );
    assert_eq!(
        domain["delete_action"]["confirm_message"],
        "确定要删除域名「needle.example」吗？此操作无法撤销。"
    );
    assert_eq!(domain["delete_action"]["success_message"], "域名记录已删除");

    let software = &body["software"][0];
    assert_eq!(software["total_price_label"], "¥300");
    assert_eq!(software["billing_cycle_label"], "每季度");
    assert_eq!(software["auto_renewal_badge"]["icon"], "XCircle");
    assert_eq!(software["status_badge"]["label"], "试用期");
    assert_eq!(software["can_activate"], true);
    assert_eq!(software["can_start_trial"], false);
    assert_eq!(software["can_mark_expired"], true);
    assert_eq!(software["can_cancel"], true);
    assert_eq!(
        software["activate_action"]["endpoint"],
        "/api/v1/software_subscriptions/software-needle/activate"
    );
    assert!(software["start_trial_action"].is_null());
    assert_eq!(
        software["mark_expired_action"]["endpoint"],
        "/api/v1/software_subscriptions/software-needle/mark_expired"
    );
    assert_eq!(
        software["cancel_action"]["endpoint"],
        "/api/v1/software_subscriptions/software-needle/cancel"
    );
    assert_eq!(
        software["delete_action"]["endpoint"],
        "/api/v1/software_subscriptions/software-needle"
    );
    assert_eq!(
        software["delete_action"]["confirm_message"],
        "确定要删除软件订阅「Needle SaaS」吗？此操作无法撤销。"
    );
    assert_eq!(
        software["delete_action"]["success_message"],
        "软件订阅已删除"
    );
}

#[tokio::test]
async fn s111_cloud_bill_status_transitions_are_backend_owned() {
    let app = common::TestApp::new().await;
    seed_provider(&app).await;

    let created = app
        .post("/api/v1/cloud_bills")
        .json(&serde_json::json!({
            "provider_id": "provider-needle",
            "billing_period": "2026-06",
            "total_amount": "¥99.90",
            "status": "paid"
        }))
        .await;
    created.assert_status_ok();
    let created_id = created.json::<Value>()["id"].as_str().unwrap().to_string();

    let stored = app.get(&format!("/api/v1/cloud_bills/{created_id}")).await;
    stored.assert_status_ok();
    let stored = stored.json::<Value>();
    assert_eq!(stored["status"], "pending");

    let update = app
        .put(&format!("/api/v1/cloud_bills/{created_id}"))
        .json(&serde_json::json!({
            "status": "paid",
            "total_amount": "¥101.10"
        }))
        .await;
    update.assert_status_ok();

    let stored = app.get(&format!("/api/v1/cloud_bills/{created_id}")).await;
    stored.assert_status_ok();
    let stored = stored.json::<Value>();
    assert_eq!(stored["status"], "pending");

    let marked = app
        .post(&format!("/api/v1/cloud_bills/{created_id}/mark_paid"))
        .json(&serde_json::json!({}))
        .await;
    marked.assert_status_ok();
    let bill = marked.json::<Value>();
    assert_eq!(bill["id"], created_id);
    assert_eq!(bill["status"], "paid");
    assert_eq!(bill["status_badge"]["label"], "已支付");
    assert_eq!(bill["can_mark_paid"], false);
    assert!(bill["mark_paid_action"].is_null());
}

#[tokio::test]
async fn s111_cloud_cost_form_options_are_backend_owned() {
    let app = common::TestApp::new().await;
    seed_provider(&app).await;

    let res = app.get("/api/v1/cloud_cost/form_options").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["provider_options"][0]["value"], "provider-needle");
    assert_eq!(body["provider_options"][0]["label"], "Needle Cloud");
    assert_eq!(body["default_values"]["bill_status"], "pending");
    assert!(body["default_values"]["resource_status"].is_null());
    assert!(body["default_values"]["domain_status"].is_null());
    assert!(body["default_values"]["software_status"].is_null());
    assert_eq!(body["provider_type_options"][0]["value"], "aliyun");
    assert_eq!(body["provider_type_options"][1]["label"], "AWS");
    assert_eq!(body["resource_type_options"][0]["value"], "compute");
    assert!(body["resource_status_options"].is_null());
    assert_eq!(body["bill_status_options"][0]["label"], "待支付");
    assert!(body["domain_status_options"].is_null());
    assert_eq!(
        body["software_subscription_type_options"][0]["value"],
        "monthly"
    );
    assert_eq!(body["billing_cycle_options"][2]["label"], "每年");
    assert!(body["software_status_options"].is_null());
}

#[tokio::test]
async fn s111_cloud_inventory_status_transitions_are_backend_owned() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;
    seed_provider(&app).await;

    let resource = app
        .post("/api/v1/cloud_resources")
        .json(&serde_json::json!({
            "provider_id": "provider-needle",
            "resource_name": "Backend ECS",
            "resource_type": "compute",
            "start_date": "2026-01-01",
            "monthly_cost": "88.00",
            "status": "terminated"
        }))
        .await;
    resource.assert_status_ok();
    let resource_id = resource.json::<Value>()["id"].as_str().unwrap().to_string();
    let stored_resource = app
        .get(&format!("/api/v1/cloud_resources/{resource_id}"))
        .await;
    stored_resource.assert_status_ok();
    assert_eq!(stored_resource.json::<Value>()["status"], "running");

    let resource_update = app
        .put(&format!("/api/v1/cloud_resources/{resource_id}"))
        .json(&serde_json::json!({
            "monthly_cost": "77.00",
            "status": "terminated"
        }))
        .await;
    resource_update.assert_status_ok();
    let stored_resource = app
        .get(&format!("/api/v1/cloud_resources/{resource_id}"))
        .await;
    stored_resource.assert_status_ok();
    let stored_resource_body = stored_resource.json::<Value>();
    assert_eq!(stored_resource_body["monthly_cost"], 77.0);
    assert_eq!(stored_resource_body["status"], "running");

    let stopped = app
        .post(&format!("/api/v1/cloud_resources/{resource_id}/stop"))
        .json(&serde_json::json!({}))
        .await;
    stopped.assert_status_ok();
    let stopped_body = stopped.json::<Value>();
    assert_eq!(stopped_body["status"], "stopped");
    assert_eq!(stopped_body["status_badge"]["label"], "已停止");
    assert_eq!(stopped_body["can_start"], true);
    assert_eq!(
        stopped_body["start_action"]["endpoint"],
        format!("/api/v1/cloud_resources/{resource_id}/start")
    );

    let started = app
        .post(&format!("/api/v1/cloud_resources/{resource_id}/start"))
        .json(&serde_json::json!({}))
        .await;
    started.assert_status_ok();
    assert_eq!(started.json::<Value>()["status"], "running");

    let terminated = app
        .post(&format!("/api/v1/cloud_resources/{resource_id}/terminate"))
        .json(&serde_json::json!({}))
        .await;
    terminated.assert_status_ok();
    let terminated_body = terminated.json::<Value>();
    assert_eq!(terminated_body["status"], "terminated");
    assert_eq!(terminated_body["status_badge"]["label"], "已终止");
    assert_eq!(terminated_body["can_terminate"], false);
    assert!(terminated_body["terminate_action"].is_null());

    let domain = app
        .post("/api/v1/domain_subscriptions")
        .json(&serde_json::json!({
            "domain_name": "backend-owned.example",
            "registrar": "Registrar",
            "registration_date": "2026-01-01",
            "expiry_date": "2027-01-01",
            "annual_fee": "120.00",
            "status": "expired"
        }))
        .await;
    domain.assert_status_ok();
    let domain_id = domain.json::<Value>()["id"].as_str().unwrap().to_string();
    let stored_domain = app
        .get(&format!("/api/v1/domain_subscriptions/{domain_id}"))
        .await;
    stored_domain.assert_status_ok();
    assert_eq!(stored_domain.json::<Value>()["status"], "active");

    let pending_transfer = app
        .post(&format!(
            "/api/v1/domain_subscriptions/{domain_id}/mark_pending_transfer"
        ))
        .json(&serde_json::json!({}))
        .await;
    pending_transfer.assert_status_ok();
    let pending_transfer_body = pending_transfer.json::<Value>();
    assert_eq!(pending_transfer_body["status"], "pending_transfer");
    assert_eq!(pending_transfer_body["status_badge"]["label"], "转移中");
    assert_eq!(pending_transfer_body["can_activate"], true);
    assert_eq!(
        pending_transfer_body["activate_action"]["endpoint"],
        format!("/api/v1/domain_subscriptions/{domain_id}/activate")
    );

    let domain_expired = app
        .post(&format!(
            "/api/v1/domain_subscriptions/{domain_id}/mark_expired"
        ))
        .json(&serde_json::json!({}))
        .await;
    domain_expired.assert_status_ok();
    let domain_expired_body = domain_expired.json::<Value>();
    assert_eq!(domain_expired_body["status"], "expired");
    assert_eq!(domain_expired_body["status_badge"]["label"], "已过期");
    assert_eq!(domain_expired_body["can_mark_expired"], false);
    assert!(domain_expired_body["mark_expired_action"].is_null());

    let software = app
        .post("/api/v1/software_subscriptions")
        .json(&serde_json::json!({
            "software_name": "Backend SaaS",
            "vendor": "Vendor",
            "subscription_type": "saas",
            "billing_cycle": "monthly",
            "start_date": "2026-01-01",
            "next_billing_date": "2026-02-01",
            "total_price": "99.00",
            "status": "cancelled"
        }))
        .await;
    software.assert_status_ok();
    let software_id = software.json::<Value>()["id"].as_str().unwrap().to_string();
    let stored_software = app
        .get(&format!("/api/v1/software_subscriptions/{software_id}"))
        .await;
    stored_software.assert_status_ok();
    assert_eq!(stored_software.json::<Value>()["status"], "active");

    let trial = app
        .post(&format!(
            "/api/v1/software_subscriptions/{software_id}/start_trial"
        ))
        .json(&serde_json::json!({}))
        .await;
    trial.assert_status_ok();
    let trial_body = trial.json::<Value>();
    assert_eq!(trial_body["status"], "trial");
    assert_eq!(trial_body["status_badge"]["label"], "试用期");
    assert_eq!(trial_body["can_activate"], true);
    assert_eq!(
        trial_body["activate_action"]["endpoint"],
        format!("/api/v1/software_subscriptions/{software_id}/activate")
    );

    let cancelled = app
        .post(&format!(
            "/api/v1/software_subscriptions/{software_id}/cancel"
        ))
        .json(&serde_json::json!({}))
        .await;
    cancelled.assert_status_ok();
    let cancelled_body = cancelled.json::<Value>();
    assert_eq!(cancelled_body["status"], "cancelled");
    assert_eq!(cancelled_body["status_badge"]["label"], "已取消");
    assert_eq!(cancelled_body["can_cancel"], false);
    assert_eq!(cancelled_body["can_activate"], false);
    assert!(cancelled_body["cancel_action"].is_null());
    assert!(cancelled_body["activate_action"].is_null());

    let other_start = other
        .post(&format!("/api/v1/cloud_resources/{resource_id}/start"))
        .json(&serde_json::json!({}))
        .await;
    other_start.assert_status_not_found();
}

#[tokio::test]
async fn s111_cloud_cost_delete_requires_backend_permission() {
    let app = common::TestApp::new().await;
    seed_provider(&app).await;
    seed_domain(&app).await;

    sqlx::query("UPDATE company_members SET role = 'member' WHERE user_id = ? AND company_id = ?")
        .bind(&app.user_id)
        .bind(&app.company_id)
        .execute(&app.pool)
        .await
        .expect("downgrade member");

    let provider_delete = app.delete("/api/v1/cloud_providers/provider-needle").await;
    provider_delete.assert_status_forbidden();

    let domain_delete = app
        .delete("/api/v1/domain_subscriptions/domain-needle")
        .await;
    domain_delete.assert_status_forbidden();
}
