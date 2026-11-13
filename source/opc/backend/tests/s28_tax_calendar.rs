/// Scenario 28: 🌍 Global Tax Calendar (全球税务日历)
/// Tests tax jurisdictions, tax deadlines, seed data, multi-country support
use crate::common;
use serde_json::json;

// ==================== Tax Jurisdictions ====================

#[tokio::test]
async fn s28_jurisdiction_crud() {
    let app = common::TestApp::new().await;

    // Create China jurisdiction
    let res = app
        .post("/api/v1/tax_jurisdictions")
        .json(&json!({
            "code": "CN",
            "name": "China",
            "name_local": "中国",
            "country_code": "CN",
            "jurisdiction_type": "country",
            "currency": "CNY",
            "corporate_tax_rate": 25.0,
            "personal_tax_rate": 45.0,
            "vat_rate": 13.0
        }))
        .await;
    res.assert_status_ok();
    let jid = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Get
    let get_res = app.get(&format!("/api/v1/tax_jurisdictions/{}", jid)).await;
    let j = get_res.json::<serde_json::Value>();
    assert_eq!(j["code"].as_str().unwrap(), "CN");
    assert_eq!(j["jurisdiction_type"].as_str().unwrap(), "country");

    // Update
    app.put(&format!("/api/v1/tax_jurisdictions/{}", jid))
        .json(&json!({ "corporate_tax_rate": 20.0 }))
        .await
        .assert_status_ok();

    let updated = app.get(&format!("/api/v1/tax_jurisdictions/{}", jid)).await;
    assert_eq!(
        updated.json::<serde_json::Value>()["corporate_tax_rate"]
            .as_f64()
            .unwrap(),
        20.0
    );

    // List
    let list = app.get("/api/v1/tax_jurisdictions").await;
    list.assert_status_ok();
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn s28_us_state_jurisdictions() {
    let app = common::TestApp::new().await;

    // Create multiple US states
    let states = vec![
        ("US-CA", "California", 8.84, Some(13.3), Some(7.25), true),
        ("US-TX", "Texas", 0.0, None, Some(6.25), false),
        ("US-FL", "Florida", 5.5, None, Some(6.0), false),
    ];

    for (code, name, corp_rate, pers_rate, sales_rate, has_income) in &states {
        let mut body = json!({
            "code": code,
            "name": name,
            "country_code": "US",
            "region_code": &code[3..],
            "jurisdiction_type": "state",
            "currency": "USD",
            "corporate_tax_rate": corp_rate,
            "has_state_income_tax": if *has_income { 1 } else { 0 }
        });
        if let Some(pr) = pers_rate {
            body["personal_tax_rate"] = json!(pr);
        }
        if let Some(sr) = sales_rate {
            body["sales_tax_rate"] = json!(sr);
        }
        app.post("/api/v1/tax_jurisdictions")
            .json(&body)
            .await
            .assert_status_ok();
    }

    let list = app.get("/api/v1/tax_jurisdictions").await;
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 3);
}

// ==================== Tax Deadlines ====================

#[tokio::test]
async fn s28_deadline_crud() {
    let app = common::TestApp::new().await;

    // Create a tax deadline
    let res = app
        .post("/api/v1/tax_deadlines")
        .json(&json!({
            "jurisdiction_code": "CN",
            "tax_type": "vat",
            "title": "VAT Monthly Filing",
            "title_local": "增值税月度申报",
            "deadline_month": 1,
            "deadline_day": 15,
            "recurrence": "monthly",
            "applies_to": "all",
            "description": "每月15日前申报上月增值税",
            "filing_url": "https://etax.chinatax.gov.cn"
        }))
        .await;
    res.assert_status_ok();
    let did = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Get
    let get_res = app.get(&format!("/api/v1/tax_deadlines/{}", did)).await;
    let d = get_res.json::<serde_json::Value>();
    assert_eq!(d["jurisdiction_code"].as_str().unwrap(), "CN");
    assert_eq!(d["tax_type"].as_str().unwrap(), "vat");
    assert_eq!(d["deadline_month"].as_i64().unwrap(), 1);
    assert_eq!(d["deadline_day"].as_i64().unwrap(), 15);

    // Update
    app.put(&format!("/api/v1/tax_deadlines/{}", did))
        .json(&json!({ "is_active": 0 }))
        .await
        .assert_status_ok();

    // List
    let list = app.get("/api/v1/tax_deadlines").await;
    list.assert_status_ok();
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn s28_multi_country_deadlines() {
    let app = common::TestApp::new().await;

    // Create deadlines from 4 countries
    let deadlines = vec![
        ("CN", "corporate_income", "CIT Annual Filing", 5, 31),
        ("KR", "vat", "VAT Q1 Filing", 7, 25),
        ("US", "personal_income", "Form 1040 Due", 4, 15),
        ("JP", "personal_income", "確定申告", 3, 15),
        ("US-CA", "personal_income", "CA Form 540", 4, 15),
    ];

    for (code, tax_type, title, month, day) in &deadlines {
        app.post("/api/v1/tax_deadlines")
            .json(&json!({
                "jurisdiction_code": code,
                "tax_type": tax_type,
                "title": title,
                "deadline_month": month,
                "deadline_day": day
            }))
            .await
            .assert_status_ok();
    }

    let list = app.get("/api/v1/tax_deadlines").await;
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 5);
}

#[tokio::test]
async fn s28_quarterly_estimated_payments() {
    let app = common::TestApp::new().await;

    // US estimated tax quarterly deadlines
    let quarters = vec![
        (1, 4, 15, "Q1 Estimated"),
        (2, 6, 15, "Q2 Estimated"),
        (3, 9, 15, "Q3 Estimated"),
        (4, 1, 15, "Q4 Estimated"),
    ];

    for (q, month, day, title) in &quarters {
        app.post("/api/v1/tax_deadlines")
            .json(&json!({
                "jurisdiction_code": "US",
                "tax_type": "estimated",
                "title": title,
                "deadline_month": month,
                "deadline_day": day,
                "recurrence": "quarterly",
                "quarter": q,
                "is_estimated_payment": 1,
                "applies_to": "all"
            }))
            .await
            .assert_status_ok();
    }

    let list = app.get("/api/v1/tax_deadlines").await;
    assert!(list.json::<serde_json::Value>()["count"].as_i64().unwrap() >= 4);
}

#[tokio::test]
async fn s28_seed_tax_data() {
    let app = common::TestApp::new().await;

    // Call seed endpoint
    let res = app.post("/api/v1/tax_calendar/seed").json(&json!({})).await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();

    // Should have seeded jurisdictions and deadlines
    assert!(body["success"].as_bool().unwrap());
    assert!(body["jurisdictions_seeded"].as_i64().unwrap() >= 12); // 4 countries + 8 states
    assert!(body["deadlines_seeded"].as_i64().unwrap() >= 40); // 42 deadlines

    // Verify data was actually created
    let j_list = app.get("/api/v1/tax_jurisdictions?limit=100").await;
    assert!(
        j_list.json::<serde_json::Value>()["count"]
            .as_i64()
            .unwrap()
            >= 12
    );

    let d_list = app.get("/api/v1/tax_deadlines?limit=200").await;
    assert!(
        d_list.json::<serde_json::Value>()["count"]
            .as_i64()
            .unwrap()
            >= 40
    );
}

#[tokio::test]
async fn s28_tenant_isolation() {
    let app = common::TestApp::new().await;

    app.post("/api/v1/tax_jurisdictions")
        .json(&json!({
            "code": "TEST",
            "name": "Test Country",
            "country_code": "XX",
            "jurisdiction_type": "country"
        }))
        .await
        .assert_status_ok();

    app.post("/api/v1/tax_deadlines")
        .json(&json!({
            "jurisdiction_code": "TEST",
            "tax_type": "corporate_income",
            "title": "Test Deadline",
            "deadline_month": 3,
            "deadline_day": 15
        }))
        .await
        .assert_status_ok();

    // Verify company_id
    let j_list = app.get("/api/v1/tax_jurisdictions").await;
    if let Some(items) = j_list.json::<serde_json::Value>()["data"].as_array() {
        for item in items {
            assert!(item["company_id"].as_str().is_some());
        }
    }

    let d_list = app.get("/api/v1/tax_deadlines").await;
    if let Some(items) = d_list.json::<serde_json::Value>()["data"].as_array() {
        for item in items {
            assert!(item["company_id"].as_str().is_some());
        }
    }
}
