use crate::common;

use serde_json::Value;

async fn seed_employee(
    app: &common::TestApp,
    id: &str,
    name: &str,
    status: &str,
    employment_type: &str,
    base_salary: f64,
) {
    sqlx::query(
        r#"
        INSERT INTO employees (
            id, company_id, name, full_name, employee_number, status,
            employment_type, base_salary, hire_date
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, '2026-01-02')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(name)
    .bind(name)
    .bind(format!("NO-{id}"))
    .bind(status)
    .bind(employment_type)
    .bind(base_salary)
    .execute(&app.pool)
    .await
    .expect("seed employee");
}

#[tokio::test]
async fn s98_employees_overview_formats_labels_and_permissions_on_backend() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_employee(
        &app,
        "emp-active",
        "Active Employee",
        "active",
        "full_time",
        12_345.0,
    )
    .await;
    seed_employee(
        &app,
        "emp-resigned",
        "Resigned Employee",
        "resigned",
        "part_time",
        0.0,
    )
    .await;
    seed_employee(
        &other,
        "emp-other",
        "Other Employee",
        "active",
        "contract",
        99_999.0,
    )
    .await;

    let res = app.get("/api/employees/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total"].as_i64().unwrap(), 2);
    assert_eq!(body["stats"]["active"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["resigned"].as_i64().unwrap(), 1);
    assert_eq!(body["permissions"]["can_delete"], true);
    assert_eq!(body["create_action"]["method"], "POST");
    assert_eq!(body["create_action"]["endpoint"], "/api/employees");
    assert_eq!(body["create_action"]["label"], "添加员工");
    assert_eq!(body["create_action"]["success_message"], "员工添加成功");

    let employees = body["employees"].as_array().unwrap();
    assert_eq!(employees.len(), 2);
    let active = employees
        .iter()
        .find(|employee| employee["id"] == "emp-active")
        .unwrap();
    assert_eq!(active["display_name"], "Active Employee");
    assert_eq!(active["status_badge"]["label"], "在职");
    assert_eq!(active["employment_type_badge"]["label"], "全职");
    assert_eq!(active["base_salary_label"], "¥12,345");
    assert_eq!(active["hire_date_label"], "2026-01-02");
    assert_eq!(active["can_delete"], true);
    assert_eq!(active["update_action"]["method"], "PUT");
    assert_eq!(
        active["update_action"]["endpoint"],
        "/api/employees/emp-active"
    );
    assert_eq!(active["update_action"]["label"], "保存员工");
    assert_eq!(active["update_action"]["success_message"], "员工更新成功");
    assert_eq!(active["delete_action"]["method"], "DELETE");
    assert_eq!(
        active["delete_action"]["endpoint"],
        "/api/employees/emp-active"
    );
    assert_eq!(
        active["delete_action"]["confirm_message"],
        "确定要删除员工「Active Employee」吗？如果该员工有关联的薪资或社保记录，删除将被阻止。"
    );
    assert_eq!(active["delete_action"]["success_title"], "删除成功");
    assert_eq!(active["delete_action"]["success_message"], "员工已删除");
    assert!(
        employees
            .iter()
            .all(|employee| employee["id"] != "emp-other")
    );

    let docs = app
        .get("/api/employee_documents/overview?employee_id=emp-active")
        .await;
    docs.assert_status_ok();
    let docs = docs.json::<Value>();
    assert_eq!(docs["upload_action"]["method"], "POST");
    assert_eq!(docs["upload_action"]["endpoint"], "/api/employee_documents");
    assert_eq!(docs["upload_action"]["label"], "上传员工文档");
    assert_eq!(docs["upload_action"]["success_message"], "员工文档上传成功");
    assert_eq!(
        docs["upload_action"]["default_values"]["employee_id"],
        "emp-active"
    );

    let form_options = app.get("/api/employees/form_options").await;
    form_options.assert_status_ok();
    let form_options = form_options.json::<Value>();
    assert_eq!(form_options["default_values"]["status"], "active");
    assert_eq!(
        form_options["default_values"]["employment_type"],
        "full_time"
    );
    assert!(
        form_options["status_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "resigned" && option["label"] == "离职")
    );
    assert!(
        form_options["employment_type_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "contract" && option["label"] == "合同工")
    );
}
