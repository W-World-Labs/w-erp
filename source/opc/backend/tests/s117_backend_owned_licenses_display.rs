/// Scenario 117: backend-owned license display metadata and delete permissions.
use crate::common;

use axum::http::StatusCode;
use chrono::{Duration, Utc};
use serde_json::Value;

#[tokio::test]
async fn s117_licenses_overview_returns_display_labels_and_permissions() {
    let app = common::TestApp::new().await;
    seed_license(&app).await;

    let res = app.get("/api/v1/licenses/overview").await;
    res.assert_status_ok();

    let body: Value = res.json();
    assert_eq!(body["stats"]["total"], 1);
    assert_eq!(body["permissions"]["can_manage"], true);
    assert_eq!(body["create_action"]["method"], "POST");
    assert_eq!(body["create_action"]["endpoint"], "/api/v1/licenses");
    assert_eq!(body["create_action"]["label"], "添加证照");
    assert_eq!(body["create_action"]["success_message"], "证照已添加");
    assert_eq!(body["empty_message"], "还没有证照记录");

    let license = &body["licenses"][0];
    assert_eq!(license["license_type_label"], "营业执照");
    assert_eq!(license["status_badge"]["label"], "即将到期");
    assert_eq!(license["issue_date_label"], "2026-01-01");
    assert!(license["expiry_date_label"].as_str().unwrap().len() >= 10);
    assert!(
        license["expiry_state"]["label"]
            .as_str()
            .unwrap()
            .contains("天")
    );
    assert_eq!(license["can_edit"], true);
    assert_eq!(license["update_action"]["method"], "PUT");
    assert_eq!(
        license["update_action"]["endpoint"],
        "/api/v1/licenses/license-needle"
    );
    assert_eq!(license["update_action"]["label"], "保存证照");
    assert_eq!(license["update_action"]["success_message"], "证照已更新");
    assert_eq!(license["can_delete"], true);
    assert_eq!(license["delete_action"]["method"], "DELETE");
    assert_eq!(
        license["delete_action"]["endpoint"],
        "/api/v1/licenses/license-needle"
    );
    assert_eq!(
        license["delete_action"]["confirm_message"],
        "确定要删除证照「企业营业执照」吗？"
    );
    assert_eq!(license["delete_action"]["success_title"], "删除成功");
    assert_eq!(license["delete_action"]["success_message"], "证照已删除");
}

#[tokio::test]
async fn s117_license_form_options_are_backend_owned() {
    let app = common::TestApp::new().await;

    let res = app.get("/api/v1/licenses/form_options").await;
    res.assert_status_ok();
    let body: Value = res.json();

    assert_eq!(body["default_values"]["license_type"], "business_license");
    assert_eq!(body["default_values"]["status"], "valid");
    assert_eq!(body["license_type_options"][0]["label"], "营业执照");
    assert_eq!(body["status_options"][0]["value"], "valid");
    assert_eq!(body["status_options"][1]["label"], "即将到期");
}

#[tokio::test]
async fn s117_license_delete_requires_backend_permission() {
    let app = common::TestApp::new().await;
    seed_license(&app).await;

    sqlx::query("UPDATE company_members SET role = 'member' WHERE user_id = ? AND company_id = ?")
        .bind(&app.user_id)
        .bind(&app.company_id)
        .execute(&app.pool)
        .await
        .expect("downgrade member");

    let res = app.delete("/api/v1/licenses/license-needle").await;
    res.assert_status(StatusCode::FORBIDDEN);
}

async fn seed_license(app: &common::TestApp) {
    let expiry = (Utc::now().date_naive() + Duration::days(8))
        .format("%Y-%m-%d")
        .to_string();

    sqlx::query(
        "INSERT INTO licenses (id, company_id, license_name, license_type, license_number, issuing_authority, issue_date, expiry_date, file_path, status)
         VALUES ('license-needle', ?, '企业营业执照', 'business_license', '91440300MA1234567X', '市场监督管理局', '2026-01-01', ?, '/uploads/license.pdf', 'valid')",
    )
    .bind(&app.company_id)
    .bind(&expiry)
    .execute(&app.pool)
    .await
    .unwrap();
}
