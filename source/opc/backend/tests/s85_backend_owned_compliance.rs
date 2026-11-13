use crate::common;

use chrono::{Duration, Utc};
use serde_json::Value;

fn date_after(days: i64) -> String {
    (Utc::now().date_naive() + Duration::days(days))
        .format("%Y-%m-%d")
        .to_string()
}

async fn seed_insurance(
    app: &common::TestApp,
    id: &str,
    premium: f64,
    status: &str,
    end_date: String,
) {
    sqlx::query(
        r#"
        INSERT INTO insurance_policies (
            id, company_id, policy_name, insurance_type, premium_amount,
            premium_frequency, start_date, end_date, status
        )
        VALUES (?, ?, 'Policy', 'health', ?, 'monthly', '2026-01-01', ?, ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(premium)
    .bind(end_date)
    .bind(status)
    .execute(&app.pool)
    .await
    .expect("seed insurance");
}

async fn seed_pension(app: &common::TestApp, id: &str, employer: f64, employee: f64, status: &str) {
    sqlx::query(
        r#"
        INSERT INTO pension_plans (
            id, company_id, plan_name, plan_type, country,
            contribution_employer, contribution_employee, contribution_frequency, status
        )
        VALUES (?, ?, 'Pension', 'national_pension', 'KR', ?, ?, 'monthly', ?)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(employer)
    .bind(employee)
    .bind(status)
    .execute(&app.pool)
    .await
    .expect("seed pension");
}

#[tokio::test]
async fn s85_compliance_overview_calculates_status_and_monthly_cost_on_backend() {
    let app = common::TestApp::new().await;

    seed_insurance(&app, "active-ins", 100.0, "active", date_after(30)).await;
    seed_insurance(&app, "expired-by-date", 300.0, "active", date_after(-1)).await;
    seed_insurance(&app, "cancelled-ins", 500.0, "cancelled", date_after(30)).await;
    seed_pension(&app, "active-pen", 200.0, 50.0, "active").await;
    seed_pension(&app, "suspended-pen", 900.0, 90.0, "suspended").await;

    let res = app.get("/api/v1/compliance/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["active_insurance_count"].as_i64().unwrap(), 1);
    assert_eq!(
        body["stats"]["expired_insurance_count"].as_i64().unwrap(),
        1
    );
    assert_eq!(body["stats"]["active_pension_count"].as_i64().unwrap(), 1);
    assert_eq!(
        body["stats"]["monthly_insurance_cost"].as_f64().unwrap(),
        100.0
    );
    assert_eq!(
        body["stats"]["monthly_pension_cost"].as_f64().unwrap(),
        250.0
    );
    assert_eq!(
        body["stats"]["monthly_compliance_cost"].as_f64().unwrap(),
        350.0
    );
    assert_eq!(body["insurance_policies"].as_array().unwrap().len(), 3);
    assert_eq!(body["pension_plans"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn s85_compliance_overview_returns_display_fields() {
    let app = common::TestApp::new().await;

    seed_insurance(&app, "display-ins", 1200.0, "active", date_after(30)).await;
    sqlx::query(
        r#"
        UPDATE insurance_policies
        SET coverage_amount = 1000000, deductible = 500, provider = 'Display Insurance',
            country = 'CN', premium_frequency = 'yearly'
        WHERE id = 'display-ins'
        "#,
    )
    .execute(&app.pool)
    .await
    .expect("update insurance display row");
    seed_pension(&app, "display-pen", 200.0, 50.0, "active").await;
    sqlx::query(
        r#"
        UPDATE pension_plans
        SET current_value = 8888, country = 'KR', plan_type = 'national_pension'
        WHERE id = 'display-pen'
        "#,
    )
    .execute(&app.pool)
    .await
    .expect("update pension display row");

    let res = app.get("/api/v1/compliance/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["monthly_compliance_cost_label"], "¥350");
    assert_eq!(body["stats"]["monthly_insurance_cost_label"], "¥100");
    assert_eq!(body["stats"]["monthly_pension_cost_label"], "¥250");
    assert_eq!(body["insurance_create_action"]["method"], "POST");
    assert_eq!(
        body["insurance_create_action"]["endpoint"],
        "/api/v1/insurance_policies"
    );
    assert_eq!(body["insurance_create_action"]["label"], "新增保险单");
    assert_eq!(
        body["insurance_create_action"]["success_message"],
        "保险单已创建"
    );
    assert_eq!(body["pension_create_action"]["method"], "POST");
    assert_eq!(
        body["pension_create_action"]["endpoint"],
        "/api/v1/pension_plans"
    );
    assert_eq!(body["pension_create_action"]["label"], "新增年金计划");
    assert_eq!(
        body["pension_create_action"]["success_message"],
        "年金计划已创建"
    );

    let insurance = body["insurance_policies"].as_array().unwrap()[0].clone();
    assert_eq!(insurance["insurance_type_label"], "健康险");
    assert_eq!(insurance["premium_amount_label"], "¥1,200");
    assert_eq!(insurance["coverage_amount_label"], "¥1,000,000");
    assert_eq!(insurance["deductible_label"], "¥500");
    assert!(
        insurance["coverage_period_label"]
            .as_str()
            .unwrap()
            .contains("~")
    );
    assert_eq!(insurance["status_badge"]["label"], "有效");
    assert_eq!(insurance["country_label"], "中国");
    assert_eq!(insurance["update_action"]["method"], "PUT");
    assert_eq!(
        insurance["update_action"]["endpoint"],
        "/api/v1/insurance_policies/display-ins"
    );
    assert_eq!(insurance["update_action"]["label"], "保存修改");
    assert_eq!(
        insurance["update_action"]["success_message"],
        "保险单已更新"
    );
    assert_eq!(insurance["can_delete"], true);
    assert_eq!(insurance["delete_action"]["method"], "DELETE");
    assert_eq!(
        insurance["delete_action"]["endpoint"],
        "/api/v1/insurance_policies/display-ins"
    );
    assert_eq!(
        insurance["delete_action"]["confirm_message"],
        "确定要删除保险单「Policy」吗？"
    );
    assert_eq!(insurance["delete_action"]["success_title"], "删除成功");
    assert_eq!(
        insurance["delete_action"]["success_message"],
        "保险单已删除"
    );

    let pension = body["pension_plans"].as_array().unwrap()[0].clone();
    assert_eq!(pension["plan_type_label"], "国民年金");
    assert_eq!(pension["country_label"], "韩国");
    assert_eq!(pension["contribution_employer_label"], "¥200");
    assert_eq!(pension["contribution_employee_label"], "¥50");
    assert_eq!(pension["current_value_label"], "¥8,888");
    assert_eq!(pension["status_badge"]["label"], "有效");
    assert_eq!(pension["update_action"]["method"], "PUT");
    assert_eq!(
        pension["update_action"]["endpoint"],
        "/api/v1/pension_plans/display-pen"
    );
    assert_eq!(pension["update_action"]["label"], "保存修改");
    assert_eq!(
        pension["update_action"]["success_message"],
        "年金计划已更新"
    );
    assert_eq!(pension["can_delete"], true);
    assert_eq!(pension["delete_action"]["method"], "DELETE");
    assert_eq!(
        pension["delete_action"]["endpoint"],
        "/api/v1/pension_plans/display-pen"
    );
    assert_eq!(
        pension["delete_action"]["confirm_message"],
        "确定要删除年金计划「Pension」吗？"
    );
    assert_eq!(pension["delete_action"]["success_title"], "删除成功");
    assert_eq!(
        pension["delete_action"]["success_message"],
        "年金计划已删除"
    );
}

#[tokio::test]
async fn s85_compliance_overview_is_company_scoped() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;

    seed_insurance(&app, "owned-ins", 100.0, "active", date_after(30)).await;
    seed_insurance(&other, "other-ins", 900.0, "active", date_after(30)).await;
    seed_pension(&app, "owned-pen", 20.0, 30.0, "active").await;
    seed_pension(&other, "other-pen", 800.0, 100.0, "active").await;

    let res = app.get("/api/v1/compliance/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["active_insurance_count"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["active_pension_count"].as_i64().unwrap(), 1);
    assert_eq!(
        body["stats"]["monthly_compliance_cost"].as_f64().unwrap(),
        150.0
    );
}
