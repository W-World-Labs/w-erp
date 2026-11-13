/// Scenario 118: backend-owned policy display metadata for dashboard and detail.
use crate::common;

use serde_json::Value;

#[tokio::test]
async fn s118_policy_list_detail_match_and_applications_return_display_fields() {
    let app = common::TestApp::new().await;
    seed_policy_fixture(&app).await;

    let list = app
        .get("/api/v1/policies?category=government_grant&country=CN&search=测试")
        .await;
    list.assert_status_ok();
    let list_body: Value = list.json();
    assert_eq!(list_body["count"], 1);
    assert!(list_body["category_options"].as_array().unwrap().len() >= 2);
    assert!(list_body["country_options"].as_array().unwrap().len() >= 2);

    let listed = &list_body["data"][0];
    assert_eq!(listed["category_badge"]["label"], "政府无偿资助");
    assert_eq!(listed["country_badge"]["label"], "🇨🇳 中国");
    assert_eq!(listed["amount_max_label"], "CNY 1,000,000");
    assert_eq!(listed["apply_action"]["method"], "POST");
    assert_eq!(
        listed["apply_action"]["endpoint"],
        "/api/v1/policies/applications"
    );
    assert_eq!(
        listed["apply_action"]["default_values"]["policy_id"],
        "policy-display"
    );

    let detail = app.get("/api/v1/policies/policy-display").await;
    detail.assert_status_ok();
    let policy: Value = detail.json();
    assert_eq!(policy["amount_range_label"], "CNY 100,000 - CNY 1,000,000");
    assert_eq!(policy["equity_required_label"], "5%");
    assert_eq!(
        policy["eligibility_items"].as_array().unwrap()[0]["text"],
        "注册在中国"
    );
    assert_eq!(
        policy["benefit_items"].as_array().unwrap()[0]["text"],
        "现金补贴"
    );
    assert_eq!(policy["apply_action"]["method"], "POST");
    assert_eq!(
        policy["apply_action"]["default_values"]["policy_id"],
        "policy-display"
    );
    assert!(policy["match_profile_items"].as_array().unwrap().len() >= 4);

    let matched = app.get("/api/v1/policies/match").await;
    matched.assert_status_ok();
    let matched_body: Value = matched.json();
    let first_match = &matched_body["data"][0];
    assert!(
        first_match["match_score_label"]
            .as_str()
            .unwrap()
            .ends_with('分')
    );
    assert_eq!(first_match["rank_badge"]["label"], "最优匹配");
    assert_eq!(first_match["rank_badge"]["visible"], true);
    assert_eq!(
        first_match["card_class_name"],
        "border-primary/50 shadow-md relative overflow-hidden"
    );
    assert_eq!(first_match["policy"]["amount_max_label"], "CNY 1,000,000");
    assert_eq!(first_match["country"], "CN");

    let applications = app.get("/api/v1/policies/applications/overview").await;
    applications.assert_status_ok();
    let applications_body: Value = applications.json();
    let application = &applications_body["applications"][0];
    assert_eq!(application["policy_name"], "测试创新补贴");
    assert_eq!(application["status_badge"]["label"], "材料准备中");
    assert!(
        application["updated_at_label"]
            .as_str()
            .unwrap()
            .contains('-')
    );

    let dashboard = app
        .get("/api/v1/policies/dashboard/overview?category=government_grant&country=CN&search=测试")
        .await;
    dashboard.assert_status_ok();
    let dashboard_body: Value = dashboard.json();
    assert_eq!(dashboard_body["policies"].as_array().unwrap().len(), 1);
    assert_eq!(dashboard_body["policy_count"], 1);
    assert!(dashboard_body["category_options"].as_array().unwrap().len() >= 2);
    assert!(dashboard_body["country_options"].as_array().unwrap().len() >= 2);
    assert_eq!(
        dashboard_body["matched_policies"][0]["match_score_label"]
            .as_str()
            .unwrap()
            .ends_with('分'),
        true
    );
    assert_eq!(
        dashboard_body["matched_policies"][0]["rank_badge"]["visible"],
        true
    );
    assert_eq!(
        dashboard_body["matched_policies"][0]["card_class_name"],
        "border-primary/50 shadow-md relative overflow-hidden"
    );
    assert_eq!(
        dashboard_body["applications"][0]["status_badge"]["label"],
        "材料准备中"
    );

    app.post("/api/v1/policies/applications")
        .json(&serde_json::json!({
            "policy_id": "policy-display",
            "status": "approved",
            "notes": "frontend supplied note",
            "result_date": "2026-01-01"
        }))
        .await
        .assert_status_success();
    let refreshed = app.get("/api/v1/policies/applications/overview").await;
    refreshed.assert_status_ok();
    let refreshed_body: Value = refreshed.json();
    let created = refreshed_body["applications"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| {
            item["policy_id"] == "policy-display"
                && item["notes"]
                    .as_str()
                    .is_some_and(|notes| notes.contains("政策大厅"))
        })
        .expect("backend-created policy tracking application");
    assert_eq!(created["status"], "preparing");
    assert_ne!(created["notes"], "frontend supplied note");
    assert!(created["result_date"].is_null());
}

async fn seed_policy_fixture(app: &common::TestApp) {
    sqlx::query(
        r#"
        INSERT INTO policies (
            id, name, name_en, country, region, category, provider, description,
            eligibility, benefits, amount_min, amount_max, currency, equity_required,
            deadline, cycle, url, match_industries, match_min_employees,
            match_max_employees, match_min_revenue, match_max_revenue,
            match_countries, match_company_age_max, match_founder_education,
            match_founder_is_overseas_returnee, is_active
        )
        VALUES (
            'policy-display', '测试创新补贴', 'Innovation Grant', 'CN', '深圳',
            'government_grant', '科技局', '面向科技企业的测试补贴',
            '1. 注册在中国\n2. 拥有研发项目',
            '1. 现金补贴\n2. 专家辅导',
            100000, 1000000, 'CNY', 0.05, '2026-12-31', '年度申报',
            'https://example.com/policy', '["technology","ai"]', 1, 10,
            10000, 2000000, '["CN"]', 36, '["masters"]', 1, 1
        )
        "#,
    )
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query("UPDATE companies SET country = 'CN' WHERE id = ?")
        .bind(&app.company_id)
        .execute(&app.pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO company_info (id, company_id, company_name, registration_date, founder_education, founder_is_overseas_returnee)
         VALUES (?, ?, 'Policy Test Co', '2025-01-01', 'masters', 1)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO employees (id, company_id, name, status) VALUES (?, ?, 'Policy Employee', 'active')",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO invoices (id, company_id, invoice_number, invoice_type, amount, status)
         VALUES (?, ?, 'POLICY-REV', 'income', 500000, 'paid')",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO policy_applications (id, company_id, policy_id, status, notes)
         VALUES ('policy-app-display', ?, 'policy-display', 'preparing', '准备材料')",
    )
    .bind(&app.company_id)
    .execute(&app.pool)
    .await
    .unwrap();
}
