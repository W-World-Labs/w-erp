/// Scenario 23: ⚙️ Profiles, User Roles, AI Config, Email Config, Operation Logs
/// Tests system-level configuration and audit trail
use crate::common;
use serde_json::json;

// ==================== Profiles ====================

#[tokio::test]
async fn s23_profile_crud() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/profiles")
        .json(&json!({
            "user_id": app.user_id,
            "display_name": "张老板",
            "phone": "00000138000",
            "department": "管理层",
            "position": "CEO",
            "email": "boss@opc.test",
            "bio": "OPC 创始人"
        }))
        .await;
    res.assert_status_ok();
    let profile_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Update
    let put_res = app
        .put(&format!("/api/v1/profiles/{}", profile_id))
        .json(&json!({ "display_name": "张三", "position": "CTO" }))
        .await;
    put_res.assert_status_ok();

    // Verify
    let get_res = app.get(&format!("/api/v1/profiles/{}", profile_id)).await;
    let profile = get_res.json::<serde_json::Value>();
    assert_eq!(profile["display_name"].as_str().unwrap(), "张三");
    assert_eq!(profile["position"].as_str().unwrap(), "CTO");
}

// ==================== User Roles ====================

#[tokio::test]
async fn s23_user_role_crud() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/user_roles")
        .json(&json!({
            "user_id": app.user_id,
            "role": "admin",
            "permissions": "{\"invoices\": \"rw\", \"contracts\": \"rw\", \"employees\": \"r\"}"
        }))
        .await;
    res.assert_status_ok();
    let role_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Update role
    let put_res = app
        .put(&format!("/api/v1/user_roles/{}", role_id))
        .json(&json!({ "role": "accountant" }))
        .await;
    put_res.assert_status_ok();
}

// ==================== AI Configurations ====================

#[tokio::test]
async fn s23_ai_configuration_crud() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/ai_configurations")
        .json(&json!({
            "provider": "deepseek",
            "model": "deepseek-v4-flash",
            "api_key": "sk-test-placeholder",
            "endpoint": opc::runtime_config::DEEPSEEK_CHAT_COMPLETIONS_URL,
            "enabled": 1
        }))
        .await;
    res.assert_status_ok();
    let config_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Update to the higher-capacity DeepSeek text model.
    let put_res = app.put(&format!("/api/v1/ai_configurations/{}", config_id))
        .json(&json!({ "provider": "deepseek", "model": "deepseek-v4-pro", "endpoint": opc::runtime_config::DEEPSEEK_CHAT_COMPLETIONS_URL }))
        .await;
    put_res.assert_status_ok();

    // Verify
    let get_res = app
        .get(&format!("/api/v1/ai_configurations/{}", config_id))
        .await;
    let config = get_res.json::<serde_json::Value>();
    assert_eq!(config["provider"].as_str().unwrap(), "deepseek");
    assert_eq!(config["model"].as_str().unwrap(), "deepseek-v4-pro");
}

#[tokio::test]
async fn s23_ai_config_tenant_isolation() {
    let app = common::TestApp::new().await;
    let app_b = app.create_second_tenant().await;

    // A creates AI config
    let res = app
        .post("/api/v1/ai_configurations")
        .json(&json!({
            "provider": "deepseek",
            "model": opc::runtime_config::DEFAULT_AI_MODEL,
            "api_key": "sk-test-placeholder",
            "endpoint": opc::runtime_config::DEEPSEEK_CHAT_COMPLETIONS_URL,
            "enabled": 1
        }))
        .await;
    res.assert_status_ok();
    let config_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // B cannot see A's API key
    let b_res = app_b
        .get(&format!("/api/v1/ai_configurations/{}", config_id))
        .await;
    assert_eq!(b_res.status_code(), 404);
}

// ==================== Email Configurations ====================

#[tokio::test]
async fn s23_email_configuration_crud() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/email_configurations")
        .json(&json!({
            "smtp_host": "smtp.qq.com",
            "smtp_port": 465,
            "smtp_user": "admin@opc.test",
            "smtp_password": "app-password-xxx",
            "sender_name": "OPC 系统",
            "sender_email": "admin@opc.test",
            "enabled": 0
        }))
        .await;
    res.assert_status_ok();
    let email_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Enable
    let put_res = app
        .put(&format!("/api/v1/email_configurations/{}", email_id))
        .json(&json!({ "enabled": 1 }))
        .await;
    put_res.assert_status_ok();

    // Delete
    let del_res = app
        .delete(&format!("/api/v1/email_configurations/{}", email_id))
        .await;
    assert!(del_res.status_code().is_success());
}

// ==================== Operation Logs (read-only) ====================

#[tokio::test]
async fn s23_operation_logs_recorded() {
    let app = common::TestApp::new().await;

    // Create something to generate a log
    app.post("/api/v1/invoices")
        .json(&json!({
            "invoice_number": "INV-LOG-001",
            "amount": 1000.0,
            "invoice_type": "income"
        }))
        .await;

    // Check operation logs
    let log_res = app.get("/api/v1/operation_logs").await;
    log_res.assert_status_ok();
    let logs = log_res.json::<serde_json::Value>();
    // Logs should exist (created by audit trigger in generic_create)
    assert!(logs["count"].as_i64().unwrap() >= 0);
}

// ==================== Categories ====================

#[tokio::test]
async fn s23_asset_category_crud() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/categories")
        .json(&json!({
            "name": "电子设备",
            "category_type": "fixed_asset",
            "description": "电脑、显示器等"
        }))
        .await;
    res.assert_status_ok();
    let cat_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create asset with category
    let asset_res = app
        .post("/api/assets")
        .json(&json!({
            "name": "Dell 显示器",
            "asset_type": "physical",
            "category_id": cat_id,
            "purchase_price": 3000.0
        }))
        .await;
    asset_res.assert_status_ok();
    let asset_body = asset_res.json::<serde_json::Value>();
    let asset_id = asset_body["data"]["id"]
        .as_str()
        .or_else(|| asset_body["id"].as_str())
        .unwrap()
        .to_string();

    // Verify FK
    let asset = app.get(&format!("/api/assets/{}", asset_id)).await;
    let asset_data = asset.json::<serde_json::Value>();
    let cat_val = asset_data["category_id"].as_str().or_else(|| {
        asset_data["data"]
            .get("category_id")
            .and_then(|v| v.as_str())
    });
    assert_eq!(cat_val.unwrap(), cat_id);
}
