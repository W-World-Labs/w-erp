use crate::common;

use chrono::{Datelike, Duration, Utc};
use serde_json::{Value, json};

fn date_after(days: i64) -> String {
    (Utc::now().date_naive() + Duration::days(days))
        .format("%Y-%m-%d")
        .to_string()
}

async fn seed_application(
    app: &common::TestApp,
    id: &str,
    application_no: &str,
    title: &str,
    status: &str,
    progress: i64,
    deadline: Option<String>,
) {
    sqlx::query(
        r#"
        INSERT INTO applications (
            id, company_id, application_no, title, type, category, deadline, status, progress
        )
        VALUES (?, ?, ?, ?, '高新技术企业认定', '国家级', ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(application_no)
    .bind(title)
    .bind(deadline)
    .bind(status)
    .bind(progress)
    .execute(&app.pool)
    .await
    .expect("seed application");
}

async fn seed_material(
    app: &common::TestApp,
    id: &str,
    application_id: &str,
    name: &str,
    required: bool,
    status: &str,
    can_reuse: bool,
) {
    sqlx::query(
        r#"
        INSERT INTO application_materials (
            id, company_id, application_id, name, required, status, can_reuse
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(application_id)
    .bind(name)
    .bind(if required { 1 } else { 0 })
    .bind(status)
    .bind(if can_reuse { 1 } else { 0 })
    .execute(&app.pool)
    .await
    .expect("seed material");
}

async fn seed_library_item(app: &common::TestApp, id: &str, used_count: i64) {
    sqlx::query(
        r#"
        INSERT INTO material_library (
            id, company_id, name, category, file_path, file_size, used_count
        )
        VALUES (?, ?, '营业执照.pdf', '通用材料', 'library/license.pdf', '2 MB', ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(used_count)
    .execute(&app.pool)
    .await
    .expect("seed library item");
}

async fn seed_template(
    app: &common::TestApp,
    id: &str,
    template_type: &str,
    material_name: &str,
    required: bool,
    display_order: i64,
) {
    sqlx::query(
        r#"
        INSERT INTO material_templates (
            id, company_id, template_type, template_name, template_category,
            material_name, required, description, display_order
        )
        VALUES (?, ?, ?, ?, '国家级', ?, ?, '后端模板材料', ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(template_type)
    .bind(template_type)
    .bind(material_name)
    .bind(if required { 1 } else { 0 })
    .bind(display_order)
    .execute(&app.pool)
    .await
    .expect("seed template");
}

#[tokio::test]
async fn s83_applications_overview_calculates_stats_deadlines_and_material_counts_on_backend() {
    let app = common::TestApp::new().await;

    seed_application(
        &app,
        "app-urgent",
        "APP-001",
        "紧急申报",
        "preparing",
        40,
        Some(date_after(2)),
    )
    .await;
    seed_application(
        &app,
        "app-approaching",
        "APP-002",
        "临近申报",
        "reviewing",
        80,
        Some(date_after(6)),
    )
    .await;
    seed_application(
        &app,
        "app-approved",
        "APP-003",
        "已过期获批",
        "approved",
        100,
        Some(date_after(-1)),
    )
    .await;
    seed_material(
        &app,
        "mat-a",
        "app-urgent",
        "必需已完成",
        true,
        "completed",
        false,
    )
    .await;
    seed_material(
        &app,
        "mat-b",
        "app-urgent",
        "可复用待准备",
        false,
        "pending",
        true,
    )
    .await;
    seed_library_item(&app, "library-a", 3).await;

    let res = app.get("/api/v1/applications/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total_applications"].as_i64().unwrap(), 3);
    assert_eq!(body["stats"]["active_applications"].as_i64().unwrap(), 2);
    assert_eq!(body["stats"]["approved_applications"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["material_library_items"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["urgent_applications"].as_i64().unwrap(), 1);
    assert_eq!(
        body["stats"]["approaching_applications"].as_i64().unwrap(),
        1
    );

    let rows = body["applications"].as_array().unwrap();
    let urgent = rows
        .iter()
        .find(|row| row["id"] == "app-urgent")
        .expect("urgent application");
    assert_eq!(urgent["deadline_status"], "urgent");
    assert_eq!(urgent["days_left"].as_i64().unwrap(), 2);
    assert_eq!(urgent["total_materials"].as_i64().unwrap(), 2);
    assert_eq!(urgent["completed_materials"].as_i64().unwrap(), 1);
    assert_eq!(urgent["required_materials"].as_i64().unwrap(), 1);
    assert_eq!(urgent["completed_required_materials"].as_i64().unwrap(), 1);
    assert_eq!(urgent["reusable_materials"].as_i64().unwrap(), 1);
    assert_eq!(urgent["pending_materials"].as_i64().unwrap(), 1);

    let approved = rows
        .iter()
        .find(|row| row["id"] == "app-approved")
        .expect("approved application");
    assert_eq!(approved["deadline_status"], "overdue");
}

#[tokio::test]
async fn s83_applications_overview_returns_backend_display_contract() {
    let app = common::TestApp::new().await;

    sqlx::query(
        r#"
        INSERT INTO company_info (
            id, company_id, company_name, registration_date, registered_capital
        )
        VALUES ('company-info-display', ?, '展示公司', ?, 800)
        "#,
    )
    .bind(&app.company_id)
    .bind(date_after(-1200))
    .execute(&app.pool)
    .await
    .expect("seed company info");

    seed_application(
        &app,
        "app-display-urgent",
        "APP-DISPLAY-1",
        "紧急展示申报",
        "preparing",
        45,
        Some(date_after(1)),
    )
    .await;
    seed_application(
        &app,
        "app-display-other",
        "APP-DISPLAY-2",
        "普通申报",
        "approved",
        100,
        Some(date_after(30)),
    )
    .await;
    seed_material(
        &app,
        "mat-display-required",
        "app-display-urgent",
        "营业执照",
        true,
        "completed",
        true,
    )
    .await;
    sqlx::query(
        r#"
        UPDATE application_materials
        SET upload_date = ?, file_size = '2 MB', reuse_source = '材料库营业执照'
        WHERE id = 'mat-display-required'
        "#,
    )
    .bind(Utc::now().to_rfc3339())
    .execute(&app.pool)
    .await
    .expect("update material display fields");
    seed_material(
        &app,
        "mat-display-pending",
        "app-display-urgent",
        "审计报告",
        false,
        "pending",
        false,
    )
    .await;
    seed_library_item(&app, "library-display", 7).await;
    sqlx::query(
        "UPDATE material_library SET upload_date = ?, file_size = '2048 KB' WHERE id = 'library-display'",
    )
    .bind(Utc::now().to_rfc3339())
    .execute(&app.pool)
    .await
    .expect("update library display fields");

    let search = urlencoding::encode("紧急");
    let type_filter = urlencoding::encode("高新技术企业认定");
    let res = app
        .get(&format!(
            "/api/v1/applications/overview?search={search}&type={type_filter}"
        ))
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    let applications = body["applications"].as_array().unwrap();
    assert_eq!(applications.len(), 1);
    let row = &applications[0];
    assert_eq!(row["id"], "app-display-urgent");
    assert_eq!(row["status_badge"]["label"], "准备中");
    assert_eq!(row["deadline_badge"]["label"], "紧急");
    assert_eq!(row["days_left_label"], "明天");
    assert_eq!(row["deadline_label"], date_after(1));
    assert_eq!(row["progress_label"], "45%");
    assert!(
        row["row_class_name"]
            .as_str()
            .unwrap()
            .contains("destructive")
    );
    assert_eq!(row["category_badge"]["label"], "国家级");
    assert_eq!(row["can_delete"], true);
    assert_eq!(row["delete_action"]["method"], "DELETE");
    assert_eq!(
        row["delete_action"]["endpoint"],
        "/api/v1/applications/app-display-urgent"
    );
    assert_eq!(
        row["delete_action"]["confirm_message"],
        "确定要删除申报项目「紧急展示申报」吗？"
    );
    assert_eq!(row["delete_action"]["success_title"], "删除成功");
    assert_eq!(row["delete_action"]["success_message"], "申报项目已删除");
    assert_eq!(body["type_options"].as_array().unwrap()[0]["value"], "all");
    assert!(body["recommended_templates"].as_array().unwrap().len() > 0);
    assert_eq!(
        body["recommendation_context"]["registered_capital_label"],
        "800 万元"
    );
    assert_eq!(
        body["recommendation_context"]["years_in_business_label"],
        "3 年"
    );
    assert_eq!(
        body["recommendation_context"]["recommended_count_label"]
            .as_str()
            .unwrap()
            .ends_with("个适合的项目"),
        true
    );
    assert!(body["template_catalog"][0]["visual_icon"].is_string());
    assert!(body["template_catalog"][0]["visual_color"].is_string());
    assert!(body["template_catalog"][0]["card_class_name"].is_string());
    assert!(body["template_catalog"][0]["recommended_badge"].is_object());
    assert_eq!(
        body["template_catalog"][0]["recommended_badge"]["visible"],
        true
    );
    assert_eq!(
        body["create_from_template_action"]["endpoint"],
        "/api/v1/applications/from_template"
    );
    assert_eq!(body["create_from_template_action"]["method"], "POST");
    assert_eq!(
        body["material_library_create_action"]["endpoint"],
        "/api/v1/material_library"
    );
    assert_eq!(body["material_library_create_action"]["method"], "POST");

    let materials = app
        .get("/api/v1/application_materials/overview?application_id=app-display-urgent")
        .await;
    materials.assert_status_ok();
    let materials_body = materials.json::<Value>();
    assert_eq!(
        materials_body["reuse_action"]["endpoint"],
        "/api/v1/application_materials/reuse"
    );
    assert_eq!(materials_body["reuse_action"]["method"], "POST");
    assert_eq!(materials_body["stats"]["completion_label"], "1/2");
    assert_eq!(
        materials_body["stats"]["completion_progress"].as_i64(),
        Some(50)
    );
    assert_eq!(
        materials_body["materials"][0]["status_badge"]["label"],
        "已完成"
    );
    assert_eq!(
        materials_body["materials"][0]["required_badge"]["label"],
        "必需"
    );
    assert!(
        materials_body["materials"][0]["uploaded_summary"]
            .as_str()
            .unwrap()
            .contains("文件大小")
    );
    assert_eq!(
        materials_body["required_materials"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        materials_body["pending_materials"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let item = &body["material_library_items"][0];
    assert_eq!(item["upload_date_label"].as_str().unwrap().len(), 10);
    assert_eq!(item["used_count_label"], "已使用 7 次");
    assert_eq!(item["file_size_label"], "2048 KB");
    assert_eq!(item["delete_action"]["method"], "DELETE");
    assert_eq!(
        item["delete_action"]["endpoint"],
        "/api/v1/material_library/library-display"
    );
    assert_eq!(
        item["delete_action"]["confirm_message"],
        "确定要删除材料「营业执照.pdf」吗？"
    );
    assert_eq!(item["delete_action"]["success_title"], "删除成功");
    assert_eq!(
        item["delete_action"]["success_message"],
        "材料已从材料库删除"
    );
    assert_eq!(body["material_library"]["count"], 1);
}

#[tokio::test]
async fn s83_create_application_from_template_generates_number_and_materials_on_backend() {
    let app = common::TestApp::new().await;
    let year = Utc::now().year();
    let existing_no = format!("APP{}001", year);

    seed_application(
        &app,
        "existing-app",
        &existing_no,
        "已有申报",
        "preparing",
        0,
        None,
    )
    .await;
    seed_template(&app, "tpl-a", "高新技术企业认定", "知识产权证明", true, 1).await;
    seed_template(
        &app,
        "tpl-b",
        "高新技术企业认定",
        "研发费用核算表",
        false,
        2,
    )
    .await;

    let res = app
        .post("/api/v1/applications/from_template")
        .json(&json!({
            "template_type": "高新技术企业认定",
            "template_name": "高新技术企业认定",
            "category": "国家级"
        }))
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();
    let application_id = body["application"]["id"].as_str().unwrap();

    assert_eq!(
        body["application"]["application_no"],
        format!("APP{}002", year)
    );
    assert_eq!(body["materials_created"].as_i64().unwrap(), 2);

    let materials: (i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COUNT(*),
            SUM(CASE WHEN required = 1 AND status = 'pending' THEN 1 ELSE 0 END)
        FROM application_materials
        WHERE company_id = ? AND application_id = ?
        "#,
    )
    .bind(&app.company_id)
    .bind(application_id)
    .fetch_one(&app.pool)
    .await
    .expect("material count");

    assert_eq!(materials.0, 2);
    assert_eq!(materials.1, 1);
}

#[tokio::test]
async fn s83_reuse_application_material_is_a_backend_workflow() {
    let app = common::TestApp::new().await;

    seed_application(
        &app,
        "app-reuse",
        "APP-REUSE",
        "材料复用申报",
        "preparing",
        0,
        None,
    )
    .await;
    seed_material(
        &app,
        "mat-reuse",
        "app-reuse",
        "营业执照",
        true,
        "pending",
        false,
    )
    .await;
    seed_library_item(&app, "library-reuse", 4).await;

    let res = app
        .post("/api/v1/application_materials/reuse")
        .json(&json!({
            "material_id": "mat-reuse",
            "library_item_id": "library-reuse"
        }))
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["material"]["status"], "completed");
    assert_eq!(body["material"]["can_reuse"].as_i64().unwrap(), 1);
    assert_eq!(body["material"]["reuse_source"], "营业执照.pdf");
    assert_eq!(body["library_item"]["used_count"].as_i64().unwrap(), 5);

    let row: (String, i64, String, String, String, i64) = sqlx::query_as(
        r#"
        SELECT
            application_materials.status,
            application_materials.can_reuse,
            application_materials.reuse_source,
            application_materials.file_path,
            application_materials.file_size,
            material_library.used_count
        FROM application_materials
        JOIN material_library ON material_library.id = ?
        WHERE application_materials.id = ? AND application_materials.company_id = ?
        "#,
    )
    .bind("library-reuse")
    .bind("mat-reuse")
    .bind(&app.company_id)
    .fetch_one(&app.pool)
    .await
    .expect("reuse row");

    assert_eq!(row.0, "completed");
    assert_eq!(row.1, 1);
    assert_eq!(row.2, "营业执照.pdf");
    assert_eq!(row.3, "library/license.pdf");
    assert_eq!(row.4, "2 MB");
    assert_eq!(row.5, 5);
}

#[tokio::test]
async fn s83_application_material_versions_are_backend_owned_workflow() {
    let app = common::TestApp::new().await;

    seed_application(
        &app,
        "app-version",
        "APP-VERSION",
        "版本流申报",
        "preparing",
        0,
        None,
    )
    .await;
    seed_material(
        &app,
        "mat-version",
        "app-version",
        "财务报表",
        true,
        "pending",
        false,
    )
    .await;

    let first = app
        .post("/api/v1/application_materials/mat-version/versions")
        .json(&json!({
            "file_path": "uploads/report-v1.pdf",
            "file_name": "report-v1.pdf",
            "file_size_bytes": 1536,
            "description": "初始版",
            "change_notes": "首次上传"
        }))
        .await;
    first.assert_status_ok();
    let first_body = first.json::<Value>();
    let first_id = first_body["version"]["id"].as_str().unwrap().to_string();

    assert_eq!(first_body["version"]["version_number"].as_i64(), Some(1));
    assert_eq!(first_body["version"]["is_current"].as_i64(), Some(1));
    assert_eq!(first_body["version"]["file_size_label"], "1.5 KB");
    assert_eq!(first_body["version"]["version_badge"]["label"], "版本 1");
    assert_eq!(first_body["version"]["version_badge"]["variant"], "default");
    assert_eq!(first_body["version"]["current_badge"]["label"], "当前版本");
    assert_eq!(first_body["version"]["current_badge"]["variant"], "outline");
    assert_eq!(
        first_body["version"]["current_badge"]["icon"],
        "CheckCircle"
    );
    assert_eq!(first_body["material"]["status"], "completed");
    assert_eq!(first_body["material"]["file_size"], "1.5 KB");
    assert!(
        first_body["material"]["upload_date_label"]
            .as_str()
            .unwrap()
            .len()
            >= 10
    );

    let second = app
        .post("/api/v1/application_materials/mat-version/versions")
        .json(&json!({
            "file_path": "uploads/report-v2.pdf",
            "file_name": "report-v2.pdf",
            "file_size_bytes": 2 * 1024 * 1024,
            "description": "修订版",
            "change_notes": "补充审计说明"
        }))
        .await;
    second.assert_status_ok();
    let second_body = second.json::<Value>();
    let second_id = second_body["version"]["id"].as_str().unwrap().to_string();
    assert_eq!(second_body["version"]["version_number"].as_i64(), Some(2));
    assert_eq!(second_body["version"]["file_size_label"], "2 MB");

    let list = app
        .get("/api/v1/application_materials/mat-version/versions")
        .await;
    list.assert_status_ok();
    let list_body = list.json::<Value>();
    let versions = list_body["versions"].as_array().unwrap();
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0]["id"], second_id);
    assert_eq!(versions[0]["version_badge"]["label"], "版本 2");
    assert_eq!(versions[0]["version_badge"]["variant"], "default");
    assert_eq!(versions[0]["latest_badge"]["label"], "最新");
    assert_eq!(versions[0]["current_badge"]["label"], "当前版本");
    assert_eq!(versions[1]["id"], first_id);
    assert_eq!(versions[1]["version_badge"]["variant"], "secondary");
    assert!(versions[1]["rollback_enabled"].as_bool().unwrap());
    assert_eq!(
        versions[1]["rollback_action"]["endpoint"],
        format!("/api/v1/application_materials/mat-version/versions/{first_id}/rollback")
    );
    assert_eq!(
        versions[1]["rollback_action"]["confirm_message"],
        "确定要回滚到版本 1 吗？当前材料文件将切换到该版本。"
    );
    assert_eq!(
        versions[1]["rollback_action"]["success_message"],
        "已回滚到版本 1"
    );
    assert!(versions[0]["rollback_action"].is_null());
    assert_eq!(list_body["current_version"]["id"], second_id);
    assert_eq!(list_body["version_count_label"], "共 2 个版本");
    assert!(
        list_body["earliest_version_label"]
            .as_str()
            .unwrap()
            .starts_with("最早版本：")
    );

    let rollback = app
        .post(&format!(
            "/api/v1/application_materials/mat-version/versions/{first_id}/rollback"
        ))
        .json(&json!({}))
        .await;
    rollback.assert_status_ok();
    let rollback_body = rollback.json::<Value>();
    assert_eq!(rollback_body["version"]["id"], first_id);
    assert_eq!(
        rollback_body["version"]["current_badge"]["label"],
        "当前版本"
    );
    assert_eq!(
        rollback_body["material"]["file_path"],
        "uploads/report-v1.pdf"
    );
    assert_eq!(rollback_body["material"]["file_size"], "1.5 KB");

    let current_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM material_versions WHERE material_id = ? AND company_id = ? AND is_current = 1",
    )
    .bind("mat-version")
    .bind(&app.company_id)
    .fetch_one(&app.pool)
    .await
    .expect("current version count");
    assert_eq!(current_count.0, 1);
}
