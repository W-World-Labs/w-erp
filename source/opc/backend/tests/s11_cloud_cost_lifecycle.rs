/// Scenario 11: ☁️ 云成本与 SaaS 订阅生命周期 (Cloud Cost Lifecycle)
/// 验证 云厂商创建 -> 云资源挂载 -> 软件/域名订阅 -> 产生定期扣费提醒与关联
use crate::common;
use serde_json::json;

#[tokio::test]
async fn s11_cloud_cost_and_saas_lifecycle() {
    let app = common::TestApp::new().await;

    // 1. 创建云厂商 (AWS)
    let provider_res = app
        .post("/api/v1/cloud_providers")
        .json(&json!({
            "provider_name": "AWS",
            "provider_type": "IaaS",
            "account_name": "OPC DevRoot",
            "account_id": "us-east-1-xxxxx"
        }))
        .await;
    provider_res.assert_status_ok();
    let provider_record: serde_json::Value = provider_res.json();
    let provider_id = provider_record["id"].as_str().unwrap();

    // 2. 挂载云上的 ECS (EC2)
    let ecs_res = app
        .post("/api/v1/cloud_resources")
        .json(&json!({
            "provider_id": provider_id,
            "resource_name": "Web Proxy Server",
            "resource_type": "EC2",
            "specification": "t3.medium",
            "monthly_cost": 35.5,
            "region": "us-east-1",
            "start_date": "2026-06-01",
            "status": "running"
        }))
        .await;
    ecs_res.assert_status_ok();
    let ecs_id = ecs_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 3. 购买 SaaS 订阅 (Notion)
    let saas_res = app
        .post("/api/v1/software_subscriptions")
        .json(&json!({
            "software_name": "Notion Team",
            "vendor": "Notion Labs",
            "subscription_type": "SaaS",
            "billing_cycle": "monthly",
            "start_date": "2026-06-01",
            "next_billing_date": "2026-07-01",
            "total_price": 240.0,
            "status": "active"
        }))
        .await;
    saas_res.assert_status_ok();

    // 4. 购买域名 (Domains)
    let domain_res = app
        .post("/api/v1/domain_subscriptions")
        .json(&json!({
            "domain_name": "opc.example.test",
            "registrar": "Cloudflare",
            "registration_date": "2026-06-15",
            "expiry_date": "2027-06-15",
            "annual_fee": 9.15,
            "status": "active"
        }))
        .await;
    domain_res.assert_status_ok();

    // 5. 验证各类总览 API 和资源聚合统计
    let list_res = app.get("/api/v1/cloud_resources").await;
    list_res.assert_status_ok();
    let body: serde_json::Value = list_res.json();
    assert_eq!(body["count"].as_i64().unwrap(), 1);

    // 资料更新不再直接改状态，生命周期必须走后端动作
    app.put(&format!("/api/v1/cloud_resources/{}", ecs_id))
        .json(&json!({
            "status": "stopped",
            "monthly_cost": 0.0
        }))
        .await
        .assert_status_ok();

    let fetched_ecs: serde_json::Value = app
        .get(&format!("/api/v1/cloud_resources/{}", ecs_id))
        .await
        .json();
    assert_eq!(fetched_ecs["status"].as_str().unwrap(), "running");
    assert_eq!(fetched_ecs["monthly_cost"].as_f64().unwrap(), 0.0);

    app.post(&format!("/api/v1/cloud_resources/{}/stop", ecs_id))
        .json(&json!({}))
        .await
        .assert_status_ok();

    let fetched_ecs: serde_json::Value = app
        .get(&format!("/api/v1/cloud_resources/{}", ecs_id))
        .await
        .json();
    assert_eq!(fetched_ecs["status"].as_str().unwrap(), "stopped");
}

#[tokio::test]
async fn s11_cloud_tenant_isolation() {
    let app_a = common::TestApp::new().await;
    let app_b = app_a.create_second_tenant().await;

    // Tenant A creates SaaS
    app_a
        .post("/api/v1/software_subscriptions")
        .json(&json!({
            "software_name": "Figma",
            "vendor": "Figma Inc",
            "subscription_type": "SaaS",
            "billing_cycle": "monthly",
            "start_date": "2026-06-01",
            "next_billing_date": "2026-07-01"
        }))
        .await
        .assert_status_ok();

    // Tenant B cannot see
    let b_res = app_b.get("/api/v1/software_subscriptions").await;
    b_res.assert_status_ok();
    assert_eq!(
        b_res.json::<serde_json::Value>()["count"].as_i64().unwrap(),
        0
    );
}

#[tokio::test]
async fn s11_cloud_resource_delete() {
    let app = common::TestApp::new().await;

    // Create provider
    let prov = app
        .post("/api/v1/cloud_providers")
        .json(&json!({
            "provider_name": "GCP",
            "provider_type": "IaaS",
            "account_name": "Test Account"
        }))
        .await;
    prov.assert_status_ok();
    let prov_id = prov.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create resource
    let res = app
        .post("/api/v1/cloud_resources")
        .json(&json!({
            "provider_id": prov_id,
            "resource_name": "Test VM",
            "resource_type": "VM",
            "monthly_cost": 10.0,
            "status": "running",
            "start_date": "2026-01-01",
            "region": "us-east-1"
        }))
        .await;
    res.assert_status_ok();
    let res_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Delete
    app.delete(&format!("/api/v1/cloud_resources/{}", res_id))
        .await;

    let list = app.get("/api/v1/cloud_resources").await;
    assert_eq!(
        list.json::<serde_json::Value>()["count"].as_i64().unwrap(),
        0
    );
}

#[tokio::test]
async fn s11_domain_subscription_delete() {
    let app = common::TestApp::new().await;

    let dom = app
        .post("/api/v1/domain_subscriptions")
        .json(&json!({
            "domain_name": "test-delete.com",
            "registrar": "Namecheap",
            "registration_date": "2026-01-01",
            "expiry_date": "2027-01-01",
            "annual_fee": 12.0,
            "status": "active"
        }))
        .await;
    dom.assert_status_ok();
    let dom_id = dom.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    app.delete(&format!("/api/v1/domain_subscriptions/{}", dom_id))
        .await;

    let list = app.get("/api/v1/domain_subscriptions").await;
    assert_eq!(
        list.json::<serde_json::Value>()["count"].as_i64().unwrap(),
        0
    );
}
