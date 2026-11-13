/// Scenario 15: 🌍 全球 OPC 出海税务与生活成本引擎 (Global OPC Tax & Cost Engine)
///
/// Tests:
/// 1. Pre-seeded OPC Hubs are correctly loaded (5 destinations)
/// 2. OPC Tax Calculator returns correct sandbox types for each hub
/// 3. MRR Survival Line and Take-Home percentages are mathematically valid
/// 4. AI Contract Review endpoint is reachable (graceful degradation without AI key)
/// 5. Grassroots startup policies are seeded alongside existing policies
use crate::common;
use serde_json::json;

#[tokio::test]
async fn s15_opc_hubs_seeded() {
    let app = common::TestApp::new().await;

    // Seed the OPC hubs (test runs in-memory, so we need to manually seed)
    opc::db::seed_opc_hubs::seed_global_opc_hubs(&app.pool)
        .await
        .expect("Failed to seed OPC hubs");

    let res = app.get("/api/v1/global_opc_hubs").await;
    res.assert_status_ok();

    let body: serde_json::Value = res.json();
    let count = body["count"].as_i64().unwrap();
    assert_eq!(count, 5, "Should have exactly 5 pre-seeded OPC hubs");

    let data = body["data"].as_array().unwrap();

    // Verify specific hubs exist
    let dubai = data.iter().find(|h| h["city"] == "Dubai");
    assert!(dubai.is_some(), "Dubai hub should be seeded");
    let dubai = dubai.unwrap();
    assert_eq!(
        dubai["foreign_income_exempt"].as_i64().unwrap(),
        1,
        "Dubai should be foreign income exempt"
    );
    assert_eq!(
        dubai["digital_nomad_visa"].as_i64().unwrap(),
        1,
        "Dubai should have digital nomad visa"
    );

    let wyoming = data.iter().find(|h| h["city"] == "Wyoming");
    assert!(wyoming.is_some(), "Wyoming hub should be seeded");
    let wyoming = wyoming.unwrap();
    assert_eq!(
        wyoming["pass_through_allowed"].as_i64().unwrap(),
        1,
        "Wyoming LLC should allow pass-through"
    );

    let tallinn = data.iter().find(|h| h["city"] == "Tallinn");
    assert!(tallinn.is_some(), "Tallinn hub should be seeded");
    let tallinn = tallinn.unwrap();
    assert_eq!(
        tallinn["zero_tax_on_reinvested"].as_i64().unwrap(),
        1,
        "Estonia should have zero tax on reinvested"
    );

    let hk = data.iter().find(|h| h["city"] == "Hong Kong");
    assert!(hk.is_some(), "Hong Kong hub should be seeded");
    let hk = hk.unwrap();
    assert_eq!(
        hk["foreign_income_exempt"].as_i64().unwrap(),
        1,
        "HK should be territorial (foreign exempt)"
    );

    let sg = data.iter().find(|h| h["city"] == "Singapore");
    assert!(sg.is_some(), "Singapore hub should be seeded");
}

#[tokio::test]
async fn s15_opc_hubs_and_calculator_return_backend_display_fields() {
    let app = common::TestApp::new().await;

    opc::db::seed_opc_hubs::seed_global_opc_hubs(&app.pool)
        .await
        .expect("Failed to seed OPC hubs");

    let list = app.get("/api/v1/global_opc_hubs").await;
    list.assert_status_ok();
    let list_body: serde_json::Value = list.json();
    let dubai = list_body["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hub| hub["city"] == "Dubai")
        .expect("Dubai hub");
    assert_eq!(dubai["flag"], "🇦🇪");
    assert!(
        dubai["corporate_tax_rate_label"]
            .as_str()
            .unwrap()
            .ends_with('%')
    );
    assert!(
        dubai["dividend_tax_rate_label"]
            .as_str()
            .unwrap()
            .ends_with('%')
    );
    assert!(dubai["rent_label"].as_str().unwrap().starts_with('$'));
    assert!(dubai["feature_badges"].as_array().unwrap().len() >= 2);

    let calc = app
        .post("/api/v1/global_opc_hubs/calculator")
        .json(&json!({
            "est_annual_revenue": 60000.0,
            "current_base_col_index": 50.0
        }))
        .await;
    calc.assert_status_ok();
    let rows: Vec<serde_json::Value> = calc.json();
    assert_eq!(rows[0]["rank"], 1);
    assert_eq!(rows[0]["is_best"], true);
    assert!(
        rows[0]["net_take_home_label"]
            .as_str()
            .unwrap()
            .starts_with('$')
    );
    assert!(
        rows[0]["take_home_percentage_label"]
            .as_str()
            .unwrap()
            .ends_with('%')
    );
    assert!(
        rows[0]["mrr_survival_line_label"]
            .as_str()
            .unwrap()
            .contains("/月")
    );
    assert!(rows[0]["tax_type_badge"]["label"].is_string());

    for window in rows.windows(2) {
        let left = window[0]["net_take_home"].as_f64().unwrap();
        let right = window[1]["net_take_home"].as_f64().unwrap();
        assert!(left >= right, "calculator results should be backend ranked");
    }
}

#[tokio::test]
async fn s15_opc_tax_calculator_sandbox_types() {
    let app = common::TestApp::new().await;

    opc::db::seed_opc_hubs::seed_global_opc_hubs(&app.pool)
        .await
        .expect("Failed to seed OPC hubs");

    // Calculate with $120K annual revenue, base COL = NYC (100)
    let res = app
        .post("/api/v1/global_opc_hubs/calculator")
        .json(&json!({
            "est_annual_revenue": 120000.0,
            "current_base_col_index": 100.0
        }))
        .await;
    res.assert_status_ok();

    let results: Vec<serde_json::Value> = res.json();
    assert_eq!(results.len(), 5, "Should return results for all 5 hubs");

    // Verify tax sandbox type assignments
    let dubai_r = results.iter().find(|r| r["city"] == "Dubai").unwrap();
    assert!(
        dubai_r["tax_sandbox_type"]
            .as_str()
            .unwrap()
            .contains("Territorial"),
        "Dubai should be Type C (Territorial), got {}",
        dubai_r["tax_sandbox_type"]
    );
    assert_eq!(
        dubai_r["total_tax_paid"].as_f64().unwrap(),
        0.0,
        "Dubai offshore income should be zero tax"
    );

    let wy_r = results.iter().find(|r| r["city"] == "Wyoming").unwrap();
    assert!(
        wy_r["tax_sandbox_type"]
            .as_str()
            .unwrap()
            .contains("Pass-through"),
        "Wyoming should be Type A (Pass-through), got {}",
        wy_r["tax_sandbox_type"]
    );

    let ee_r = results.iter().find(|r| r["city"] == "Tallinn").unwrap();
    assert!(
        ee_r["tax_sandbox_type"]
            .as_str()
            .unwrap()
            .contains("Estonia"),
        "Tallinn should be Type B (Estonia Model), got {}",
        ee_r["tax_sandbox_type"]
    );
}

#[tokio::test]
async fn s15_opc_mrr_survival_line_math() {
    let app = common::TestApp::new().await;

    opc::db::seed_opc_hubs::seed_global_opc_hubs(&app.pool)
        .await
        .expect("Failed to seed OPC hubs");

    let res = app
        .post("/api/v1/global_opc_hubs/calculator")
        .json(&json!({
            "est_annual_revenue": 60000.0,
            "current_base_col_index": 100.0
        }))
        .await;
    res.assert_status_ok();

    let results: Vec<serde_json::Value> = res.json();

    for hub in &results {
        let mrr = hub["mrr_survival_line"].as_f64().unwrap();
        let friction = hub["friction_costs"].as_f64().unwrap();
        let living = hub["living_costs"].as_f64().unwrap();

        // MRR = (friction + living) / 12
        let expected_mrr = (friction + living) / 12.0;
        assert!(
            (mrr - expected_mrr).abs() < 0.01,
            "City {}: MRR should be (friction + living) / 12 = {}, got {}",
            hub["city"],
            expected_mrr,
            mrr
        );

        // Take-home percentage should be between 0 and 100
        let thp = hub["take_home_percentage"].as_f64().unwrap();
        assert!(
            thp <= 100.0,
            "Take-home percentage should not exceed 100%, got {}",
            thp
        );

        // Setup & maintenance ROI should be positive
        let roi = hub["setup_and_maintenance_roi"].as_f64().unwrap();
        assert!(roi >= 0.0, "Setup ROI should be non-negative, got {}", roi);
    }
}

#[tokio::test]
async fn s15_ai_contract_review_graceful_degradation() {
    let app = common::TestApp::new().await;

    // Without AI_API_URL set, the endpoint should return 503 (Service Unavailable)
    // This tests graceful degradation when no AI provider is configured
    let res = app.post("/api/v1/ai/contract-review")
        .json(&json!({
            "contract_text": "甲方有权在任何时候单方面解除合同，乙方不得异议。违约金由乙方承担100%。",
            "contract_type": "服务合同",
            "counterparty_type": "client"
        }))
        .await;

    // Should be 503 since no AI env vars are set in test
    assert_eq!(
        res.status_code(),
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        "Should return 503 when AI service is not configured"
    );
}

#[tokio::test]
async fn s15_grassroots_policies_seeded() {
    let app = common::TestApp::new().await;

    // Seed policies (including new grassroots ones)
    opc::db::seed_policies::seed_global_policies(&app.pool)
        .await
        .expect("Failed to seed policies");

    let res = app.get("/api/v1/policies").await;
    res.assert_status_ok();

    let body: serde_json::Value = res.json();
    let data = body["data"].as_array().unwrap();

    // Check grassroots policies exist
    let student_loan = data.iter().find(|p| {
        p["name"]
            .as_str()
            .map(|n| n.contains("大学生创业"))
            .unwrap_or(false)
    });
    assert!(
        student_loan.is_some(),
        "大学生创业无息贷款 should be seeded"
    );

    let social_security = data.iter().find(|p| {
        p["name"]
            .as_str()
            .map(|n| n.contains("社保"))
            .unwrap_or(false)
    });
    assert!(
        social_security.is_some(),
        "小微企业社保全额补贴 should be seeded"
    );

    let startup_subsidy = data.iter().find(|p| {
        p["name"]
            .as_str()
            .map(|n| n.contains("一次性创业"))
            .unwrap_or(false)
    });
    assert!(
        startup_subsidy.is_some(),
        "初创期一次性创业补贴 should be seeded"
    );

    let housing = data.iter().find(|p| {
        p["name"]
            .as_str()
            .map(|n| n.contains("落户"))
            .unwrap_or(false)
    });
    assert!(housing.is_some(), "高校毕业生落户安居补贴 should be seeded");
}

#[tokio::test]
async fn s15_opc_hub_idempotent_seeding() {
    let app = common::TestApp::new().await;

    // Seed twice — should be idempotent
    opc::db::seed_opc_hubs::seed_global_opc_hubs(&app.pool)
        .await
        .expect("First seed failed");
    opc::db::seed_opc_hubs::seed_global_opc_hubs(&app.pool)
        .await
        .expect("Second seed failed");

    let res = app.get("/api/v1/global_opc_hubs").await;
    res.assert_status_ok();

    let body: serde_json::Value = res.json();
    let count = body["count"].as_i64().unwrap();
    assert_eq!(count, 5, "Double seeding should not create duplicates");
}

#[tokio::test]
async fn s15_inactive_opc_hubs_are_hidden() {
    let app = common::TestApp::new().await;

    opc::db::seed_opc_hubs::seed_global_opc_hubs(&app.pool)
        .await
        .expect("Failed to seed OPC hubs");

    sqlx::query("UPDATE global_opc_hubs SET is_active = 0 WHERE city = 'Dubai'")
        .execute(&app.pool)
        .await
        .unwrap();

    let res = app.get("/api/v1/global_opc_hubs").await;
    res.assert_status_ok();

    let body: serde_json::Value = res.json();
    assert_eq!(body["count"].as_i64().unwrap(), 4);
    let data = body["data"].as_array().unwrap();
    assert!(
        data.iter().all(|hub| hub["city"] != "Dubai"),
        "inactive hubs should not be listed"
    );
}
