/// Scenario 21: 📄 Material Library, Templates & Versions (材料库/模板/版本)
/// Tests the application material management pipeline
use crate::common;
use serde_json::json;

#[tokio::test]
async fn s21_material_library_crud() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/material_library")
        .json(&json!({
            "name": "企业营业执照副本",
            "category": "证照类",
            "file_path": "/uploads/license.pdf",
            "file_size": "2.1MB",
            "used_count": 0
        }))
        .await;
    res.assert_status_ok();
    let mat_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Update used count
    let put_res = app
        .put(&format!("/api/v1/material_library/{}", mat_id))
        .json(&json!({ "used_count": 3 }))
        .await;
    put_res.assert_status_ok();

    // Verify
    let get_res = app
        .get(&format!("/api/v1/material_library/{}", mat_id))
        .await;
    let mat = get_res.json::<serde_json::Value>();
    assert_eq!(mat["name"].as_str().unwrap(), "企业营业执照副本");
    assert_eq!(mat["used_count"].as_i64().unwrap(), 3);

    // Delete
    let del_res = app
        .delete(&format!("/api/v1/material_library/{}", mat_id))
        .await;
    assert!(del_res.status_code().is_success());
}

#[tokio::test]
async fn s21_material_template_crud() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/material_templates")
        .json(&json!({
            "template_type": "high_tech",
            "template_name": "高新技术企业认定材料清单",
            "template_category": "资质认定",
            "material_name": "知识产权清单",
            "required": 1,
            "description": "近三年获得的知识产权",
            "display_order": 1
        }))
        .await;
    res.assert_status_ok();
    let tpl_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Verify
    let get_res = app
        .get(&format!("/api/v1/material_templates/{}", tpl_id))
        .await;
    let tpl = get_res.json::<serde_json::Value>();
    assert_eq!(
        tpl["template_name"].as_str().unwrap(),
        "高新技术企业认定材料清单"
    );
    assert_eq!(tpl["required"].as_i64().unwrap(), 1);
}

#[tokio::test]
async fn s21_material_version_lifecycle() {
    let app = common::TestApp::new().await;

    // Create a material first
    let mat_res = app
        .post("/api/v1/material_library")
        .json(&json!({ "name": "公司简介", "category": "基本材料" }))
        .await;
    mat_res.assert_status_ok();
    let mat_id = mat_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create version 1
    let v1_res = app
        .post("/api/v1/material_versions")
        .json(&json!({
            "material_id": mat_id,
            "version_number": 1,
            "file_path": "/uploads/intro_v1.pdf",
            "file_name": "公司简介_v1.pdf",
            "file_size": "1.2MB",
            "description": "初始版本",
            "is_current": 1
        }))
        .await;
    v1_res.assert_status_ok();
    let v1_id = v1_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create version 2
    let v2_res = app
        .post("/api/v1/material_versions")
        .json(&json!({
            "material_id": mat_id,
            "version_number": 2,
            "file_path": "/uploads/intro_v2.pdf",
            "file_name": "公司简介_v2.pdf",
            "file_size": "1.5MB",
            "description": "更新版本",
            "change_notes": "更新了业务范围",
            "is_current": 1
        }))
        .await;
    v2_res.assert_status_ok();

    // Mark v1 as non-current
    app.put(&format!("/api/v1/material_versions/{}", v1_id))
        .json(&json!({ "is_current": 0 }))
        .await;

    // List all versions
    let list_res = app.get("/api/v1/material_versions").await;
    list_res.assert_status_ok();
    let list = list_res.json::<serde_json::Value>();
    assert!(list["count"].as_i64().unwrap() >= 2);
}

#[tokio::test]
async fn s21_material_tenant_isolation() {
    let app = common::TestApp::new().await;
    let app_b = app.create_second_tenant().await;

    // A creates material
    let res = app
        .post("/api/v1/material_library")
        .json(&json!({ "name": "机密文件", "category": "内部" }))
        .await;
    res.assert_status_ok();
    let mat_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // B cannot access
    let b_res = app_b
        .get(&format!("/api/v1/material_library/{}", mat_id))
        .await;
    assert_eq!(b_res.status_code(), 404);
}

#[tokio::test]
async fn s21_material_template_delete() {
    let app = common::TestApp::new().await;

    let res = app
        .post("/api/v1/material_templates")
        .json(&json!({
            "template_type": "sbir",
            "template_name": "SBIR申报材料模板",
            "template_category": "资质认定",
            "material_name": "项目计划书",
            "required": 1,
            "display_order": 1
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let del = app
        .delete(&format!("/api/v1/material_templates/{}", id))
        .await;
    assert!(del.status_code().is_success());

    let list = app.get("/api/v1/material_templates").await;
    assert_eq!(
        list.json::<serde_json::Value>()["count"].as_i64().unwrap(),
        0
    );
}

#[tokio::test]
async fn s21_material_zero_used_count() {
    let app = common::TestApp::new().await;

    // Zero used_count should be valid
    let res = app
        .post("/api/v1/material_library")
        .json(&json!({
            "name": "未使用材料",
            "category": "测试",
            "used_count": 0
        }))
        .await;
    res.assert_status_ok();
    let data: serde_json::Value = res.json();
    assert_eq!(data["used_count"].as_i64().unwrap_or(0), 0);
}
