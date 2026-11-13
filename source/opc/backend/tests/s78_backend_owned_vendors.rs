use crate::common;

use serde_json::Value;

async fn seed_vendor(app: &common::TestApp, id: &str, name: &str, vendor_type: &str, status: &str) {
    sqlx::query(
        r#"
        INSERT INTO vendors (id, company_id, name, vendor_type, status)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
    .bind(vendor_type)
    .bind(status)
    .execute(&app.pool)
    .await
    .expect("seed vendor");
}

#[tokio::test]
async fn s78_vendor_overview_calculates_counts_on_backend() {
    let app = common::TestApp::new().await;

    seed_vendor(&app, "vendor-supplier", "供应商 A", "supplier", "active").await;
    seed_vendor(&app, "vendor-customer", "客户 B", "customer", "inactive").await;
    seed_vendor(&app, "vendor-both", "双向 C", "both", "active").await;

    let res = app.get("/api/v1/vendors/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total"].as_i64().unwrap(), 3);
    assert_eq!(body["stats"]["suppliers"].as_i64().unwrap(), 2);
    assert_eq!(body["stats"]["customers"].as_i64().unwrap(), 2);
    assert_eq!(body["stats"]["active"].as_i64().unwrap(), 2);
    assert_eq!(body["create_action"]["method"], "POST");
    assert_eq!(body["create_action"]["endpoint"], "/api/v1/vendors");
    assert_eq!(body["create_action"]["label"], "新增友商");
    assert_eq!(body["create_action"]["success_title"], "创建成功");
    assert_eq!(
        body["create_action"]["success_message"],
        "友商资料已成功创建"
    );

    let vendors = body["vendors"].as_array().unwrap();
    assert_eq!(vendors.len(), 3);
    assert!(vendors.iter().any(|vendor| vendor["id"] == "vendor-both"));
    let supplier = vendors
        .iter()
        .find(|vendor| vendor["id"] == "vendor-supplier")
        .expect("supplier vendor");
    assert_eq!(supplier["update_action"]["method"], "PUT");
    assert_eq!(
        supplier["update_action"]["endpoint"],
        "/api/v1/vendors/vendor-supplier"
    );
    assert_eq!(supplier["update_action"]["label"], "保存修改");
    assert_eq!(supplier["update_action"]["success_title"], "更新成功");
    assert_eq!(
        supplier["update_action"]["success_message"],
        "友商资料已成功更新"
    );
    assert_eq!(supplier["can_delete"].as_bool().unwrap(), true);
    assert_eq!(supplier["delete_action"]["method"], "DELETE");
    assert_eq!(
        supplier["delete_action"]["endpoint"],
        "/api/vendors/vendor-supplier"
    );
    assert_eq!(
        supplier["delete_action"]["confirm_message"],
        "确定要删除友商「供应商 A」吗？"
    );
    assert_eq!(supplier["delete_action"]["success_title"], "删除成功");
    assert_eq!(
        supplier["delete_action"]["success_message"],
        "友商资料已成功删除"
    );
}

#[tokio::test]
async fn s78_vendor_overview_is_company_scoped() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_vendor(&app, "owned", "本公司友商", "supplier", "active").await;
    seed_vendor(&other, "other", "其他公司友商", "both", "active").await;

    let res = app.get("/api/v1/vendors/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["suppliers"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["customers"].as_i64().unwrap(), 0);
    assert_eq!(body["stats"]["active"].as_i64().unwrap(), 1);

    let vendors = body["vendors"].as_array().unwrap();
    assert_eq!(vendors.len(), 1);
    assert_eq!(vendors[0]["id"], "owned");
}
