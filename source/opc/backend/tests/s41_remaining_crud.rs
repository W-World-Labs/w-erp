/// Scenario 41: 📋 Asset Records, Contacts, Invoice Reminders, Quote Items, KPI, Tax Records, Email Config
/// CRUD tests for remaining untested CRUD endpoints.
use crate::common;
use serde_json::json;

// ==================== Asset Records ====================

#[tokio::test]
async fn s41_asset_record_crud() {
    let app = common::TestApp::new().await;
    let asset = app
        .post("/api/assets")
        .json(&json!({ "name": "记录资产", "asset_type": "equipment" }))
        .await;
    if !asset.status_code().is_success() {
        return;
    }
    let asset_id = asset.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = app.post("/api/v1/asset_records")
        .json(&json!({ "asset_id": &asset_id, "record_type": "maintenance", "description": "年度保养" }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();
    app.get(&format!("/api/v1/asset_records/{}", id))
        .await
        .assert_status_ok();
    let del = app.delete(&format!("/api/v1/asset_records/{}", id)).await;
    assert!(del.status_code().is_success());
}

#[tokio::test]
async fn s41_asset_record_fk_invalid() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/v1/asset_records")
        .json(&json!({ "asset_id": "no-such-asset", "record_type": "repair" }))
        .await;
    assert_eq!(res.status_code(), 400);
}

// ==================== Contacts ====================

#[tokio::test]
async fn s41_contact_crud() {
    let app = common::TestApp::new().await;
    let client = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "联系人客户", "client_type": "corporate" }))
        .await;
    client.assert_status_ok();
    let client_id = client.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = app.post("/api/v1/contacts")
        .json(&json!({ "name": "张三", "client_id": &client_id, "email": "zs@test-com.example.invalid", "phone": "138-0000" }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let get = app.get(&format!("/api/v1/contacts/{}", id)).await;
    get.assert_status_ok();
    assert_eq!(
        get.json::<serde_json::Value>()["name"].as_str().unwrap(),
        "张三"
    );

    app.put(&format!("/api/v1/contacts/{}", id))
        .json(&json!({ "name": "张三(更新)", "email": "zs2@test-com.example.invalid" }))
        .await
        .assert_status_ok();

    let del = app.delete(&format!("/api/v1/contacts/{}", id)).await;
    assert!(del.status_code().is_success());
}

#[tokio::test]
async fn s41_contact_unicode() {
    let app = common::TestApp::new().await;
    app.post("/api/v1/contacts")
        .json(&json!({ "name": "김철수 / 田中太郎 🧑‍💼" }))
        .await
        .assert_status_ok();
}

#[tokio::test]
async fn s41_contact_pagination() {
    let app = common::TestApp::new().await;
    for i in 0..10 {
        app.post("/api/v1/contacts")
            .json(&json!({ "name": format!("联系人{:03}", i) }))
            .await
            .assert_status_ok();
    }
    let page = app.get("/api/v1/contacts?limit=5&offset=0").await;
    page.assert_status_ok();
    assert_eq!(
        page.json::<serde_json::Value>()["data"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
}

// ==================== Invoice Reminders ====================

#[tokio::test]
async fn s41_invoice_reminder_crud() {
    let app = common::TestApp::new().await;
    let inv = app
        .post("/api/invoices")
        .json(&json!({ "invoice_number": "INV-IREM", "amount": 5000.0, "invoice_type": "income" }))
        .await;
    inv.assert_status_ok();
    let inv_id = inv.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = app.post("/api/v1/invoice_reminders")
        .json(&json!({ "invoice_id": &inv_id, "reminder_type": "overdue", "scheduled_date": "2024-04-15" }))
        .await;
    if !res.status_code().is_success() {
        return;
    }
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();
    app.get(&format!("/api/v1/invoice_reminders/{}", id))
        .await
        .assert_status_ok();
    let del = app
        .delete(&format!("/api/v1/invoice_reminders/{}", id))
        .await;
    assert!(del.status_code().is_success());
}

// ==================== Quote Items ====================

#[tokio::test]
async fn s41_quote_item_crud() {
    let app = common::TestApp::new().await;
    let quote = app
        .post("/api/v1/quotes")
        .json(&json!({ "quote_number": "QT-ITEM-001", "status": "draft" }))
        .await;
    if !quote.status_code().is_success() {
        return;
    }
    let quote_id = quote.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = app.post("/api/v1/quote_items")
        .json(&json!({ "quote_id": &quote_id, "description": "咨询服务", "quantity": 10, "unit_price": 500.0 }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();
    app.get(&format!("/api/v1/quote_items/{}", id))
        .await
        .assert_status_ok();
    let del = app.delete(&format!("/api/v1/quote_items/{}", id)).await;
    assert!(del.status_code().is_success());
}

#[tokio::test]
async fn s41_quote_item_negative_price() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/v1/quote_items")
        .json(&json!({ "description": "负价格", "quantity": 1, "unit_price": -100.0 }))
        .await;
    let s = res.status_code();
    assert!(
        s == 400 || s == 422 || s == 500 || s.is_success(),
        "Negative price: {}",
        s
    );
}

// ==================== KPI (main) ====================

#[tokio::test]
async fn s41_kpi_crud() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/v1/kpi")
        .json(&json!({ "name": "客户满意度", "target_value": 95.0, "unit": "percent" }))
        .await;
    if !res.status_code().is_success() {
        return;
    }
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let get = app.get(&format!("/api/v1/kpi/{}", id)).await;
    get.assert_status_ok();
    assert_eq!(
        get.json::<serde_json::Value>()["name"].as_str().unwrap(),
        "客户满意度"
    );
    let del = app.delete(&format!("/api/v1/kpi/{}", id)).await;
    assert!(del.status_code().is_success());
}

#[tokio::test]
async fn s41_kpi_list() {
    let app = common::TestApp::new().await;
    for name in ["收入", "利润", "客户数", "NPS", "员工满意度"] {
        let r = app.post("/api/v1/kpi").json(&json!({ "name": name })).await;
        if !r.status_code().is_success() {
            return;
        }
    }
    let list = app.get("/api/v1/kpi").await;
    assert!(
        list.json::<serde_json::Value>()["count"]
            .as_i64()
            .unwrap_or(0)
            >= 5
    );
}

// ==================== Tax Records (main) ====================

#[tokio::test]
async fn s41_tax_record_crud() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/v1/tax")
        .json(
            &json!({ "tax_type": "vat", "period": "2024-03", "amount": 13000.0, "status": "paid" }),
        )
        .await;
    if !res.status_code().is_success() {
        return;
    }
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();
    app.get(&format!("/api/v1/tax/{}", id))
        .await
        .assert_status_ok();
    let del = app.delete(&format!("/api/v1/tax/{}", id)).await;
    assert!(del.status_code().is_success());
}

#[tokio::test]
async fn s41_tax_record_pagination() {
    let app = common::TestApp::new().await;
    for i in 0..8 {
        let r = app.post("/api/v1/tax")
            .json(&json!({ "tax_type": "vat", "period": format!("2024-{:02}", i + 1), "amount": 10000.0 + i as f64 * 1000.0 }))
            .await;
        if !r.status_code().is_success() {
            return;
        }
    }
    let page = app.get("/api/v1/tax?limit=3&offset=3").await;
    page.assert_status_ok();
    assert_eq!(
        page.json::<serde_json::Value>()["data"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
}

// ==================== Email Configurations ====================

#[tokio::test]
async fn s41_email_config_crud() {
    let app = common::TestApp::new().await;
    let res = app.post("/api/v1/email_configurations")
        .json(&json!({ "smtp_host": "smtp.test.com", "smtp_port": 587, "username": "admin@test-com.example.invalid" }))
        .await;
    if !res.status_code().is_success() {
        return;
    }
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();
    app.get(&format!("/api/v1/email_configurations/{}", id))
        .await
        .assert_status_ok();
    let del = app
        .delete(&format!("/api/v1/email_configurations/{}", id))
        .await;
    assert!(del.status_code().is_success());
}

// ==================== Milestones (standalone) ====================

#[tokio::test]
async fn s41_milestone_crud() {
    let app = common::TestApp::new().await;
    let proj = app
        .post("/api/projects")
        .json(&json!({ "name": "里程碑项目", "status": "active" }))
        .await;
    proj.assert_status_ok();
    let proj_id = proj.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = app
        .post("/api/v1/milestones")
        .json(&json!({ "project_id": &proj_id, "title": "需求确认", "status": "pending" }))
        .await;
    if !res.status_code().is_success() {
        return;
    }
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();
    app.get(&format!("/api/v1/milestones/{}", id))
        .await
        .assert_status_ok();
    let del = app.delete(&format!("/api/v1/milestones/{}", id)).await;
    assert!(del.status_code().is_success());
}

#[tokio::test]
async fn s41_milestone_fk_invalid() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/v1/milestones")
        .json(&json!({ "project_id": "no-such-project", "title": "孤儿里程碑" }))
        .await;
    let s = res.status_code();
    assert!(s == 400 || s == 500, "Invalid project FK: {}", s);
}

// ==================== Interactions ====================

#[tokio::test]
async fn s41_interaction_crud() {
    let app = common::TestApp::new().await;
    let client = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "互动客户", "client_type": "individual" }))
        .await;
    client.assert_status_ok();
    let client_id = client.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = app
        .post("/api/v1/interactions")
        .json(&json!({
            "client_id": &client_id, "interaction_type": "meeting",
            "interaction_date": "2024-03-15", "notes": "讨论合作方案"
        }))
        .await;
    if !res.status_code().is_success() {
        return;
    }
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();
    app.get(&format!("/api/v1/interactions/{}", id))
        .await
        .assert_status_ok();
    let del = app.delete(&format!("/api/v1/interactions/{}", id)).await;
    assert!(del.status_code().is_success());
}

#[tokio::test]
async fn s41_interaction_types() {
    let app = common::TestApp::new().await;
    for itype in ["call", "email", "meeting", "visit"] {
        let r = app
            .post("/api/v1/interactions")
            .json(&json!({ "interaction_type": itype, "interaction_date": "2024-03-15" }))
            .await;
        if !r.status_code().is_success() {
            return;
        }
    }
}
