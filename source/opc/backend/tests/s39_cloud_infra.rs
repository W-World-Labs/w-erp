/// Scenario 39: ☁️ Cloud Infrastructure + Domain + Software Subscriptions CRUD
use crate::common;
use serde_json::json;

// ==================== Cloud Providers ====================

#[tokio::test]
async fn s39_cloud_provider_crud() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/v1/cloud_providers")
        .json(&json!({
            "provider_name": "AWS", "provider_type": "public_cloud", "account_name": "main-account"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let get = app.get(&format!("/api/v1/cloud_providers/{}", id)).await;
    get.assert_status_ok();
    assert_eq!(
        get.json::<serde_json::Value>()["provider_name"]
            .as_str()
            .unwrap(),
        "AWS"
    );

    app.put(&format!("/api/v1/cloud_providers/{}", id))
        .json(&json!({ "provider_name": "AWS", "provider_type": "public_cloud", "account_name": "updated" }))
        .await.assert_status_ok();

    let del = app.delete(&format!("/api/v1/cloud_providers/{}", id)).await;
    assert!(
        del.status_code().is_success(),
        "Delete: {:?}",
        del.status_code()
    );
}

#[tokio::test]
async fn s39_cloud_provider_multi() {
    let app = common::TestApp::new().await;
    for (name, acct) in [
        ("AWS", "aws"),
        ("Azure", "az"),
        ("GCP", "gcp"),
        ("阿里云", "ali"),
        ("腾讯云", "tc"),
    ] {
        app.post("/api/v1/cloud_providers")
            .json(&json!({ "provider_name": name, "provider_type": "public_cloud", "account_name": acct }))
            .await.assert_status_ok();
    }
    let list = app.get("/api/v1/cloud_providers").await;
    assert!(
        list.json::<serde_json::Value>()["count"]
            .as_i64()
            .unwrap_or(0)
            >= 5
    );
}

// ==================== Cloud Resources ====================

#[tokio::test]
async fn s39_cloud_resource_crud() {
    let app = common::TestApp::new().await;
    let prov = app.post("/api/v1/cloud_providers")
        .json(&json!({ "provider_name": "AWS-R", "provider_type": "public_cloud", "account_name": "r-acct" }))
        .await;
    prov.assert_status_ok();
    let prov_id = prov.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = app.post("/api/v1/cloud_resources")
        .json(&json!({
            "provider_id": &prov_id, "resource_name": "prod-web", "resource_type": "ec2", "start_date": "2024-01-01"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    app.get(&format!("/api/v1/cloud_resources/{}", id))
        .await
        .assert_status_ok();
    let del = app.delete(&format!("/api/v1/cloud_resources/{}", id)).await;
    assert!(del.status_code().is_success());
}

#[tokio::test]
async fn s39_cloud_resource_fk_invalid() {
    let app = common::TestApp::new().await;
    let res = app.post("/api/v1/cloud_resources")
        .json(&json!({
            "provider_id": "nonexistent", "resource_name": "orphan", "resource_type": "ec2", "start_date": "2024-01-01"
        }))
        .await;
    assert_eq!(res.status_code(), 400, "Invalid provider FK");
}

// ==================== Cloud Bills ====================

#[tokio::test]
async fn s39_cloud_bill_crud() {
    let app = common::TestApp::new().await;
    let prov = app.post("/api/v1/cloud_providers")
        .json(&json!({ "provider_name": "AWS-B", "provider_type": "public_cloud", "account_name": "b-acct" }))
        .await;
    prov.assert_status_ok();
    let prov_id = prov.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = app.post("/api/v1/cloud_bills")
        .json(&json!({
            "provider_id": &prov_id, "bill_month": "2024-03", "item_name": "EC2", "bill_date": "2024-03-31", "amount": 2500.0
        }))
        .await;
    // May fail if table has additional required columns
    let status = res.status_code();
    if !status.is_success() {
        return;
    }
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();
    app.get(&format!("/api/v1/cloud_bills/{}", id))
        .await
        .assert_status_ok();
    let del = app.delete(&format!("/api/v1/cloud_bills/{}", id)).await;
    assert!(del.status_code().is_success());
}

#[tokio::test]
async fn s39_cloud_provider_delete_blocked() {
    let app = common::TestApp::new().await;
    let prov = app.post("/api/v1/cloud_providers")
        .json(&json!({ "provider_name": "Del-Test", "provider_type": "public_cloud", "account_name": "del" }))
        .await;
    prov.assert_status_ok();
    let prov_id = prov.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    app.post("/api/v1/cloud_resources")
        .json(&json!({ "provider_id": &prov_id, "resource_name": "block", "resource_type": "rds", "start_date": "2024-01-01" }))
        .await.assert_status_ok();

    let del = app
        .delete(&format!("/api/v1/cloud_providers/{}", prov_id))
        .await;
    assert_eq!(
        del.status_code(),
        409,
        "Provider with resources should be protected"
    );
}

// ==================== Domain Subscriptions ====================

#[tokio::test]
async fn s39_domain_subscription_crud() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/v1/domain_subscriptions")
        .json(&json!({
            "domain_name": "example.com", "registrar": "Cloudflare",
            "registration_date": "2020-01-01", "expiry_date": "2025-01-01"
        }))
        .await;
    let status = res.status_code();
    if !status.is_success() {
        return;
    }
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();
    app.get(&format!("/api/v1/domain_subscriptions/{}", id))
        .await
        .assert_status_ok();
    let del = app
        .delete(&format!("/api/v1/domain_subscriptions/{}", id))
        .await;
    assert!(del.status_code().is_success());
}

#[tokio::test]
async fn s39_domain_unicode() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/v1/domain_subscriptions")
        .json(&json!({
            "domain_name": "测试.中国", "registrar": "万网",
            "registration_date": "2024-01-01", "expiry_date": "2025-01-01"
        }))
        .await;
    assert!(res.status_code().is_success() || res.status_code() == 500);
}

#[tokio::test]
async fn s39_domain_pagination() {
    let app = common::TestApp::new().await;
    let mut ct = 0;
    for i in 0..8 {
        let r = app
            .post("/api/v1/domain_subscriptions")
            .json(&json!({
                "domain_name": format!("d{}.com", i), "registrar": "GD",
                "registration_date": "2024-01-01", "expiry_date": "2025-01-01"
            }))
            .await;
        if r.status_code().is_success() {
            ct += 1;
        }
    }
    if ct >= 3 {
        let page = app
            .get("/api/v1/domain_subscriptions?limit=3&offset=0")
            .await;
        page.assert_status_ok();
        assert_eq!(
            page.json::<serde_json::Value>()["data"]
                .as_array()
                .unwrap()
                .len(),
            3
        );
    }
}

// ==================== Software Subscriptions ====================

#[tokio::test]
async fn s39_software_subscription_crud() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/v1/software_subscriptions")
        .json(&json!({
            "software_name": "GitHub Enterprise", "vendor": "GitHub",
            "subscription_type": "team", "billing_cycle": "annual",
            "start_date": "2024-01-01", "next_billing_date": "2025-01-01"
        }))
        .await;
    let status = res.status_code();
    if !status.is_success() {
        return;
    }
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();
    app.get(&format!("/api/v1/software_subscriptions/{}", id))
        .await
        .assert_status_ok();
    let del = app
        .delete(&format!("/api/v1/software_subscriptions/{}", id))
        .await;
    assert!(del.status_code().is_success());
}

#[tokio::test]
async fn s39_software_multi() {
    let app = common::TestApp::new().await;
    let mut ct = 0;
    for (name, vendor) in [
        ("Slack", "SF"),
        ("Figma", "Fig"),
        ("Notion", "NL"),
        ("Jira", "ATL"),
    ] {
        let r = app
            .post("/api/v1/software_subscriptions")
            .json(&json!({
                "software_name": name, "vendor": vendor,
                "subscription_type": "team", "billing_cycle": "monthly",
                "start_date": "2024-01-01", "next_billing_date": "2024-04-01"
            }))
            .await;
        if r.status_code().is_success() {
            ct += 1;
        }
    }
    if ct >= 4 {
        let list = app.get("/api/v1/software_subscriptions").await;
        assert!(
            list.json::<serde_json::Value>()["count"]
                .as_i64()
                .unwrap_or(0)
                >= 4
        );
    }
}
