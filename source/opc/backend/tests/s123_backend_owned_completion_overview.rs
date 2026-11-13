use crate::common;

use serde_json::Value;

async fn seed_contract(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO contracts (id, company_id, contract_name)
        VALUES ('completion-contract', ?, 'Completion Contract')
        "#,
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .expect("seed contract");
}

#[tokio::test]
async fn s123_ai_insights_returns_backend_owned_completion_display_collections() {
    let app = common::TestApp::new().await;
    common::seed::create_employee(&app.pool, &app.company_id, "Completion Employee", 1000.0).await;
    seed_contract(&app).await;

    let res = app.get("/api/v1/ai/insights").await;
    res.assert_status_ok();
    let body = res.json::<Value>();
    let completeness = &body["completeness"];

    assert_eq!(completeness["total_count"].as_i64().unwrap(), 10);
    assert_eq!(completeness["completed_count"].as_i64().unwrap(), 3);
    assert_eq!(completeness["incomplete_count"].as_i64().unwrap(), 7);
    assert_eq!(completeness["overall_score"].as_i64().unwrap(), 30);
    assert_eq!(completeness["overall_score_label"], "30%");
    assert_eq!(
        completeness["overall_gradient_class"],
        "from-orange-500 to-red-500"
    );
    assert_eq!(completeness["unlock_title"], "完善数据可解锁");
    assert!(
        completeness["unlock_description"]
            .as_str()
            .unwrap()
            .contains("智能分析报告")
    );
    assert_eq!(completeness["completed_label"], "已完成 3/10 个模块");
    assert_eq!(completeness["incomplete_label"], "还有 7 个模块待完善");
    assert_eq!(completeness["completion_badge_label"], "30%");
    assert_eq!(
        completeness["remaining_overflow_count"].as_i64().unwrap(),
        4
    );
    assert_eq!(
        completeness["remaining_overflow_label"],
        "还有 4 个模块待完善..."
    );
    assert_eq!(completeness["is_complete"].as_bool(), Some(false));
    assert!(completeness["status_title"].as_str().is_some());
    assert!(completeness["status_description"].as_str().is_some());

    let items = completeness["items"].as_array().unwrap();
    let completed_items = completeness["completed_items"].as_array().unwrap();
    let incomplete_items = completeness["incomplete_items"].as_array().unwrap();
    let top_incomplete_items = completeness["top_incomplete_items"].as_array().unwrap();

    assert_eq!(items.len(), 10);
    assert_eq!(completed_items.len(), 3);
    assert_eq!(incomplete_items.len(), 7);
    assert_eq!(top_incomplete_items.len(), 3);
    assert!(
        items
            .iter()
            .all(|item| item["percentage_label"].as_str().is_some())
    );
    assert!(
        completed_items
            .iter()
            .all(|item| item["percentage"].as_i64() == Some(100))
    );
    assert!(
        incomplete_items
            .iter()
            .all(|item| item["percentage"].as_i64() == Some(0))
    );

    assert!(body["critical_insights"].as_array().is_some());
    assert!(body["other_insights"].as_array().is_some());
    assert_eq!(
        body["critical_insights"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["priority"].as_str() == Some("critical")),
        true
    );
    assert_eq!(
        body["other_insights"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["priority"].as_str() != Some("critical")),
        true
    );
}
