use crate::common;

use chrono::Utc;
use serde_json::Value;

async fn seed_log(
    app: &common::TestApp,
    id: &str,
    action: &str,
    entity_type: &str,
    entity_id: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO operation_logs (
            id, company_id, user_id, action, entity_type, entity_id, details
        )
        VALUES (?, ?, ?, ?, ?, ?, '{"source":"test"}')
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(&app.user_id)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .execute(&app.pool)
    .await
    .expect("seed operation log");
}

#[tokio::test]
async fn s94_operation_logs_overview_filters_stats_labels_and_export_on_backend() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_log(&app, "log-create", "create", "contracts", "contract-123456").await;
    seed_log(&app, "log-update", "update", "invoices", "invoice-123456").await;
    seed_log(&app, "log-delete", "delete", "payments", "payment-123456").await;
    seed_log(&app, "log-view", "view", "contracts", "contract-view").await;
    seed_log(&other, "log-other", "create", "contracts", "other-contract").await;

    let res = app
        .get("/api/v1/operation_logs/overview?search=contracts&type=CREATE&table=contracts")
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total"].as_i64().unwrap(), 4);
    assert_eq!(body["stats"]["create"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["update"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["delete"].as_i64().unwrap(), 1);

    let logs = body["logs"].as_array().unwrap();
    assert_eq!(logs.len(), 1);
    let log = &logs[0];
    assert_eq!(log["id"], "log-create");
    assert_eq!(log["operation_type"], "CREATE");
    assert_eq!(log["operation_badge"]["label"], "新建");
    assert_eq!(log["table_label"], "合同");
    assert_eq!(log["record_id_short"], "contract...");

    assert!(
        body["table_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|table| { table["value"] == "contracts" && table["label"] == "合同" })
    );
    assert!(
        body["type_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|kind| { kind["value"] == "DELETE" && kind["label"] == "删除" })
    );
    assert_eq!(body["permissions"]["can_view"], true);
    assert_eq!(body["permissions"]["can_export"], true);

    let today = Utc::now().format("%Y%m%d").to_string();
    assert_eq!(
        body["export"]["filename"].as_str().unwrap(),
        format!("operation_logs_{today}.csv")
    );
    let csv = body["export"]["csv"].as_str().unwrap();
    assert!(csv.contains("合同"));
    assert!(!csv.contains("invoice-123456"));
    assert!(!csv.contains("other-contract"));
}

#[tokio::test]
async fn s94_operation_logs_overview_permissions_are_backend_owned() {
    let app = common::TestApp::new().await;
    seed_log(&app, "log-hidden", "create", "contracts", "contract-hidden").await;

    sqlx::query("UPDATE company_members SET role = 'member' WHERE user_id = ? AND company_id = ?")
        .bind(&app.user_id)
        .bind(&app.company_id)
        .execute(&app.pool)
        .await
        .expect("downgrade member");

    let res = app.get("/api/v1/operation_logs/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["permissions"]["can_view"], false);
    assert_eq!(body["permissions"]["can_export"], false);
    assert_eq!(body["logs"].as_array().unwrap().len(), 0);
    assert_eq!(body["stats"]["total"], 0);
}
