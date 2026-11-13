use crate::common;

use chrono::{Datelike, Utc};
use serde_json::Value;

async fn seed_jurisdiction(
    app: &common::TestApp,
    id: &str,
    code: &str,
    country_code: &str,
    jurisdiction_type: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO tax_jurisdictions (
            id, company_id, code, name, country_code, jurisdiction_type, is_active
        )
        VALUES (?, ?, ?, ?, ?, ?, 1)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(code)
    .bind(format!("Jurisdiction {code}"))
    .bind(country_code)
    .bind(jurisdiction_type)
    .execute(&app.pool)
    .await
    .expect("seed jurisdiction");
}

async fn seed_deadline(
    app: &common::TestApp,
    id: &str,
    jurisdiction_code: &str,
    month: i64,
    day: i64,
    title: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO tax_deadlines (
            id, company_id, jurisdiction_code, tax_type, title,
            deadline_month, deadline_day, recurrence, applies_to, is_active
        )
        VALUES (?, ?, ?, 'corporate_income', ?, ?, ?, 'yearly', 'corporation', 1)
        "#,
    )
    .bind(id)
    .bind(&app.company_id)
    .bind(jurisdiction_code)
    .bind(title)
    .bind(month)
    .bind(day)
    .execute(&app.pool)
    .await
    .expect("seed deadline");
}

#[tokio::test]
async fn s84_tax_calendar_overview_filters_and_groups_on_backend() {
    let app = common::TestApp::new().await;

    seed_jurisdiction(&app, "jur-us", "US", "US", "country").await;
    seed_jurisdiction(&app, "jur-us-ca", "US-CA", "US", "state").await;
    sqlx::query(
        "UPDATE tax_jurisdictions
         SET has_state_income_tax = 1,
             corporate_tax_rate = 21,
             personal_tax_rate = 13.3,
             vat_rate = 7.25,
             sales_tax_rate = 10
         WHERE id = 'jur-us-ca'",
    )
    .execute(&app.pool)
    .await
    .expect("mark state income tax");
    seed_jurisdiction(&app, "jur-cn", "CN", "CN", "country").await;
    seed_deadline(&app, "deadline-us", "US", 4, 15, "US Federal").await;
    seed_deadline(&app, "deadline-us-ca", "US-CA", 4, 30, "California").await;
    seed_deadline(&app, "deadline-cn", "CN", 5, 31, "China").await;

    let res = app
        .get("/api/v1/tax_calendar/overview?country=US-states&month=4")
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["country_count"].as_i64().unwrap(), 2);
    assert_eq!(body["seed_action"]["method"], "POST");
    assert_eq!(body["seed_action"]["endpoint"], "/api/v1/tax_calendar/seed");
    assert_eq!(body["stats"]["state_count"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["deadline_count"].as_i64().unwrap(), 3);
    assert_eq!(body["filters"]["filtered_count"].as_i64().unwrap(), 1);
    assert_eq!(
        body["filtered_deadlines"][0]["jurisdiction_code"]
            .as_str()
            .unwrap(),
        "US-CA"
    );
    assert_eq!(body["filtered_deadlines"][0]["country_flag"], "🇺🇸");
    assert_eq!(
        body["filtered_deadlines"][0]["tax_type_label"],
        "企业所得税"
    );
    assert_eq!(
        body["filtered_deadlines"][0]["tax_type_color"],
        "bg-blue-500"
    );
    assert_eq!(body["filtered_deadlines"][0]["recurrence_label"], "每年");
    assert_eq!(body["filtered_deadlines"][0]["applies_to_label"], "企业");
    assert!(
        body["country_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|option| option["value"] == "US-states" && option["label"] == "🇺🇸 美国各州")
    );
    assert_eq!(body["month_options"][0]["value"], "all");
    assert_eq!(body["month_options"][4]["label"], "4月");

    let april = body["calendar_by_month"]
        .as_array()
        .unwrap()
        .iter()
        .find(|month| month["month"] == 4)
        .expect("april bucket");
    assert_eq!(april["label"], "4月");
    assert_eq!(april["count_badge"]["label"], "2");
    assert_eq!(april["visible_events"].as_array().unwrap().len(), 2);
    assert_eq!(april["empty_message"], "无截止日期");
    assert_eq!(april["overflow_label"], "");
    assert_eq!(april["events"].as_array().unwrap().len(), 2);
    assert_eq!(april["events"][0]["tax_type_label"], "企业所得税");
    let current_month = body["filters"]["current_month"].as_i64().unwrap();
    let current_bucket = body["calendar_by_month"]
        .as_array()
        .unwrap()
        .iter()
        .find(|month| month["month"] == current_month)
        .expect("current month bucket");
    assert_eq!(current_bucket["card_class_name"], "border-primary border-2");
    assert_eq!(body["jurisdictions"][0]["country_flag"], "🇨🇳");
    assert_eq!(
        body["jurisdictions"][0]["jurisdiction_type_badge"]["label"],
        "国家"
    );
    let california = body["jurisdictions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|jurisdiction| jurisdiction["code"] == "US-CA")
        .expect("california jurisdiction");
    assert_eq!(california["state_income_tax_badge"]["label"], "有");
    assert_eq!(california["state_income_tax_badge"]["variant"], "default");
    assert_eq!(california["state_income_tax_label"], "有");
    assert_eq!(california["corporate_tax_rate_label"], "21%");
    assert_eq!(california["personal_tax_rate_label"], "13.3%");
    assert_eq!(california["vat_rate_label"], "7.25%");
    assert_eq!(california["sales_tax_rate_label"], "10%");
}

#[tokio::test]
async fn s84_tax_calendar_overview_is_company_scoped() {
    let app = common::TestApp::new().await;
    let other = app.create_second_tenant().await;
    let current_month = i64::from(Utc::now().month());

    seed_jurisdiction(&app, "owned-jur", "US", "US", "country").await;
    seed_jurisdiction(&other, "other-jur", "JP", "JP", "country").await;
    seed_deadline(&app, "owned-deadline", "US", current_month, 15, "Owned").await;
    seed_deadline(&other, "other-deadline", "JP", current_month, 15, "Other").await;

    let res = app.get("/api/v1/tax_calendar/overview").await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    assert_eq!(body["stats"]["country_count"].as_i64().unwrap(), 1);
    assert_eq!(body["stats"]["deadline_count"].as_i64().unwrap(), 1);
    assert_eq!(body["filtered_deadlines"].as_array().unwrap().len(), 1);
    assert_eq!(body["filtered_deadlines"][0]["jurisdiction_code"], "US");
}
