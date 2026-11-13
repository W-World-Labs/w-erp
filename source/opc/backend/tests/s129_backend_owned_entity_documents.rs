/// Scenario 129: entity document lists are backend-owned display contracts.
use crate::common;

use serde_json::Value;

#[tokio::test]
async fn s129_employee_and_project_documents_overview_return_backend_display_contracts() {
    let app = common::TestApp::new().await;
    let employee_id =
        common::seed::create_employee(&app.pool, &app.company_id, "Document Owner", 10000.0).await;
    let project_id = "project-doc-owner";

    sqlx::query(
        "INSERT INTO projects (id, company_id, name, status) VALUES (?, ?, '文档项目', 'active')",
    )
    .bind(project_id)
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();

    seed_document_tables(&app).await;
    sqlx::query(
        "INSERT INTO employee_documents (id, company_id, employee_id, document_type, file_name, file_path, description, upload_date)
         VALUES ('employee-doc-display', ?, ?, 'id_card', 'id-card.png', 'file-employee', '身份证正面', '2026-01-02T03:04:05Z')",
    )
    .bind(&app.company_id)
    .bind(&employee_id)
    .execute(&app.pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO project_documents (id, company_id, project_id, document_type, file_name, file_path, description, upload_date)
         VALUES ('project-doc-display', ?, ?, 'budget', 'budget.xlsx', 'file-project', '预算表', '2026-02-03T04:05:06Z')",
    )
    .bind(&app.company_id)
    .bind(project_id)
    .execute(&app.pool)
    .await
    .unwrap();

    let employee = app
        .get(&format!(
            "/api/v1/employee_documents/overview?employee_id={employee_id}"
        ))
        .await;
    employee.assert_status_ok();
    let employee_body: Value = employee.json();
    let employee_doc = &employee_body["documents"][0];
    assert_eq!(employee_doc["document_type_label"], "身份证");
    assert_eq!(employee_doc["upload_date_label"], "2026-01-02");
    assert_eq!(employee_doc["file_icon"], "ImageIcon");
    assert!(
        employee_doc["file_icon_class_name"]
            .as_str()
            .unwrap()
            .contains("blue")
    );
    assert_eq!(
        employee_doc["delete_action"]["endpoint"],
        "/api/employee_documents/employee-doc-display"
    );
    assert_eq!(
        employee_doc["delete_action"]["confirm_message"],
        "确定要删除员工文档「id-card.png」吗？此操作无法撤销。"
    );
    assert_eq!(
        employee_doc["delete_action"]["success_message"],
        "员工文档已删除"
    );
    assert!(
        employee_body["document_type_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "id_card")
    );

    let project = app
        .get("/api/v1/project_documents/overview?project_id=project-doc-owner")
        .await;
    project.assert_status_ok();
    let project_body: Value = project.json();
    let project_doc = &project_body["documents"][0];
    assert_eq!(project_doc["document_type_label"], "预算文件");
    assert_eq!(project_doc["upload_date_label"], "2026-02-03");
    assert_eq!(project_doc["file_icon"], "FileText");
    assert!(
        project_doc["file_icon_class_name"]
            .as_str()
            .unwrap()
            .contains("green")
    );
    assert_eq!(
        project_doc["delete_action"]["endpoint"],
        "/api/project_documents/project-doc-display"
    );
    assert_eq!(
        project_doc["delete_action"]["confirm_message"],
        "确定要删除项目文档「budget.xlsx」吗？此操作无法撤销。"
    );
    assert_eq!(
        project_doc["delete_action"]["success_message"],
        "项目文档已删除"
    );
    assert!(
        project_body["document_type_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "budget")
    );
}

async fn seed_document_tables(app: &common::TestApp) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS employee_documents (
            id TEXT PRIMARY KEY,
            company_id TEXT NOT NULL,
            employee_id TEXT NOT NULL,
            document_type TEXT NOT NULL,
            file_name TEXT NOT NULL,
            file_path TEXT NOT NULL,
            description TEXT,
            upload_date TEXT DEFAULT CURRENT_TIMESTAMP,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&app.pool)
    .await
    .unwrap();
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS project_documents (
            id TEXT PRIMARY KEY,
            company_id TEXT NOT NULL,
            project_id TEXT NOT NULL,
            document_type TEXT NOT NULL,
            file_name TEXT NOT NULL,
            file_path TEXT NOT NULL,
            description TEXT,
            upload_date TEXT DEFAULT CURRENT_TIMESTAMP,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&app.pool)
    .await
    .unwrap();
}
