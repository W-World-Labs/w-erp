use crate::common;

use serde_json::Value;

async fn seed_vendor(
    app: &common::TestApp,
    id: &str,
    name: &str,
    short_name: &str,
    unified_code: &str,
    contact_person: &str,
    vendor_type: &str,
    status: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO vendors (
            id, company_id, name, short_name, unified_code, contact_person, vendor_type, status
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
    .bind(short_name)
    .bind(unified_code)
    .bind(contact_person)
    .bind(vendor_type)
    .bind(status)
    .execute(&app.pool)
    .await
    .expect("seed vendor");
}

#[tokio::test]
async fn s96_vendor_overview_filters_and_labels_on_backend() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_vendor(
        &app,
        "vendor-needle",
        "可信云服务",
        "云服",
        "CREDIT-001",
        "Alice",
        "supplier",
        "active",
    )
    .await;
    seed_vendor(
        &app,
        "vendor-inactive",
        "历史云服务",
        "历史",
        "CREDIT-002",
        "Bob",
        "supplier",
        "inactive",
    )
    .await;
    seed_vendor(
        &app,
        "vendor-customer",
        "客户信用公司",
        "客户",
        "CREDIT-003",
        "Carol",
        "customer",
        "active",
    )
    .await;
    seed_vendor(
        &other,
        "vendor-other",
        "其他信用公司",
        "其他",
        "CREDIT-999",
        "Other",
        "supplier",
        "active",
    )
    .await;

    let res = app
        .get("/api/v1/vendors/overview?search=credit&vendor_type=supplier&status=active")
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total"].as_i64().unwrap(), 3);
    assert_eq!(body["stats"]["suppliers"].as_i64().unwrap(), 2);
    assert_eq!(body["stats"]["customers"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["active"].as_i64().unwrap(), 2);

    let vendors = body["vendors"].as_array().unwrap();
    assert_eq!(vendors.len(), 1);
    assert_eq!(vendors[0]["id"], "vendor-needle");
    assert_eq!(vendors[0]["type_badge"]["label"], "供应商");
    assert_eq!(vendors[0]["status_badge"]["label"], "合作中");
    assert_eq!(vendors[0]["status_badge"]["icon"], "CheckCircle");
    assert!(vendors.iter().all(|vendor| vendor["id"] != "vendor-other"));

    assert!(
        body["type_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| { option["value"] == "both" && option["label"] == "供应商/客户" })
    );
    assert_eq!(body["type_options"][0]["value"], "all");
    assert_eq!(body["type_options"][0]["label"], "全部类型");
    assert!(
        body["status_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "inactive" && option["label"] == "已停用")
    );
    assert_eq!(body["status_options"][0]["value"], "all");
    assert_eq!(body["status_options"][0]["label"], "全部状态");

    let form_options = app.get("/api/v1/vendors/form_options").await;
    form_options.assert_status_ok();
    let form_options = form_options.json::<Value>();
    assert_eq!(form_options["default_values"]["vendor_type"], "supplier");
    assert_eq!(form_options["default_values"]["status"], "active");
    assert!(
        form_options["vendor_type_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "both" && option["label"] == "供应商/客户")
    );
    assert!(
        form_options["status_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "inactive" && option["label"] == "已停用")
    );
}
