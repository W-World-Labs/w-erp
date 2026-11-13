use crate::common;

use serde_json::Value;

async fn seed_client(app: &common::TestApp) {
    sqlx::query(
        "INSERT INTO clients (id, company_id, name, client_type) VALUES ('client-needle', ?, 'Needle 客户', 'active')",
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed client");
}

async fn seed_deal(app: &common::TestApp, id: &str, stage: &str, amount: f64, probability: i64) {
    sqlx::query(
        r#"
        INSERT INTO deals (
            id, company_id, client_id, title, stage, amount, probability
        )
        VALUES (?, ?, 'client-needle', ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(format!("Deal {id}"))
    .bind(stage)
    .bind(amount)
    .bind(probability)
    .execute(&app.pool)
    .await
    .expect("seed deal");
}

async fn seed_quote(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO quotes (
            id, company_id, client_id, quote_number, title, status, total, valid_until
        )
        VALUES (
            'quote-needle', ?, 'client-needle', 'QT-NEEDLE-001',
            'Needle Quote', 'accepted', 123456.78, '2026-12-31'
        )
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed quote");
}

#[tokio::test]
async fn s113_deals_overview_returns_display_labels_and_permissions() {
    let app = common::TestApp::new().await;
    seed_client(&app).await;
    seed_deal(&app, "deal-lead", "lead", 100000.0, 10).await;
    seed_deal(&app, "deal-won", "won", 50000.0, 100).await;

    let res = app.get("/api/v1/deals/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total_pipeline_label"], "¥100,000");
    assert_eq!(body["stats"]["won_amount_label"], "¥50,000");
    assert_eq!(body["stats"]["avg_probability_label"], "55%");
    assert_eq!(body["permissions"]["can_manage"], true);
    assert_eq!(body["empty_messages"]["pipeline"], "暂无");
    assert_eq!(body["create_action"]["method"], "POST");
    assert_eq!(body["create_action"]["endpoint"], "/api/v1/deals");
    assert_eq!(body["create_action"]["label"], "新增机会");
    assert_eq!(body["create_action"]["success_title"], "创建成功");
    assert_eq!(body["create_action"]["success_message"], "销售机会已创建");

    let lead = body["pipeline_stages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|stage| stage["value"] == "lead")
        .expect("lead stage");
    assert_eq!(lead["amount_label"], "¥100,000");
    assert_eq!(lead["deals"][0]["amount_label"], "¥100,000");
    assert_eq!(lead["deals"][0]["probability_label"], "10%");
    assert_eq!(lead["deals"][0]["can_update_stage"], true);
    assert_eq!(lead["deals"][0]["update_action"]["method"], "PUT");
    assert_eq!(
        lead["deals"][0]["update_action"]["endpoint"],
        "/api/v1/deals/deal-lead"
    );
    let qualified = lead["deals"][0]["next_stages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|stage| stage["value"] == "qualified")
        .expect("qualified next stage");
    assert_eq!(qualified["action"]["method"], "PUT");
    assert_eq!(qualified["action"]["endpoint"], "/api/v1/deals/deal-lead");
    assert_eq!(qualified["action"]["default_values"]["stage"], "qualified");

    assert_eq!(body["won_deals"][0]["amount_label"], "¥50,000");
}

#[tokio::test]
async fn s113_quotes_overview_returns_display_labels_and_permissions() {
    let app = common::TestApp::new().await;
    seed_client(&app).await;
    seed_quote(&app).await;

    let res = app.get("/api/v1/quotes/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["total_amount_label"], "¥123,456.78");
    assert_eq!(body["permissions"]["can_manage"], true);
    assert_eq!(body["empty_message"], "暂无报价单");
    assert_eq!(body["create_action"]["method"], "POST");
    assert_eq!(body["create_action"]["endpoint"], "/api/v1/quotes");
    assert_eq!(body["create_action"]["label"], "新建报价单");
    assert_eq!(body["create_action"]["success_title"], "创建成功");
    assert_eq!(body["create_action"]["success_message"], "报价单已创建");

    let quote = &body["quotes"][0];
    assert_eq!(quote["total_label"], "¥123,456.78");
    assert_eq!(quote["status_badge"]["label"], "已接受");
    assert_eq!(quote["valid_until_label"], "2026-12-31");
    assert_eq!(quote["can_convert_to_invoice"], true);
    assert_eq!(quote["update_action"]["method"], "PUT");
    assert_eq!(
        quote["update_action"]["endpoint"],
        "/api/v1/quotes/quote-needle"
    );
    assert_eq!(quote["update_action"]["label"], "保存修改");
    assert_eq!(quote["update_action"]["success_title"], "更新成功");
    assert_eq!(quote["update_action"]["success_message"], "报价单已更新");
    assert_eq!(quote["can_delete"], true);
    assert_eq!(quote["delete_action"]["method"], "DELETE");
    assert_eq!(
        quote["delete_action"]["endpoint"],
        "/api/v1/quotes/quote-needle"
    );
    assert_eq!(
        quote["delete_action"]["confirm_message"],
        "确定要删除报价单「QT-NEEDLE-001」吗？"
    );
    assert_eq!(quote["delete_action"]["success_title"], "删除成功");
    assert_eq!(quote["delete_action"]["success_message"], "报价单已删除");
}

#[tokio::test]
async fn s113_crm_deletes_require_backend_permission() {
    let app = common::TestApp::new().await;
    seed_client(&app).await;
    seed_deal(&app, "deal-lead", "lead", 100000.0, 10).await;
    seed_quote(&app).await;

    sqlx::query("UPDATE company_members SET role = 'member' WHERE user_id = ? AND company_id = ?")
        .bind(&app.user_id)
        .bind(&app.company_id)
        .execute(&app.pool)
        .await
        .expect("downgrade member");

    let deal_delete = app.delete("/api/v1/deals/deal-lead").await;
    deal_delete.assert_status_forbidden();

    let quote_delete = app.delete("/api/v1/quotes/quote-needle").await;
    quote_delete.assert_status_forbidden();
}
