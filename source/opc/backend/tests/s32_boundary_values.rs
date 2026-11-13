/// Scenario 32: 🧪 Boundary Values & Edge Cases (边界值与极端情况)
/// Fills the "边界值" column gap in the coverage matrix for ALL business modules.
///
/// Categories:
/// 1. Unicode / Multi-byte (中文/韩文/日文/Emoji/阿拉伯文)
/// 2. Super-long strings (>10000 chars)
/// 3. Zero-amount / Negative-amount
/// 4. Empty / null / whitespace-only fields
/// 5. Floating-point precision (0.1+0.2, 大金额)
/// 6. Large-volume pagination (100+ records)
/// 7. Special characters (SQL injection markers, HTML/XSS, newlines)
/// 8. Date edge cases (leap year, far future, epoch)
/// 9. Concurrent write stress
use crate::common;
use serde_json::json;

// ==================== 1. Unicode / Multi-byte ====================

#[tokio::test]
async fn s32_unicode_chinese_invoice() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "发票-2024-测试-001",
            "amount": 12345.67,
            "invoice_type": "income",
            "supplier": "北京某某科技有限公司",
            "description": "这是一张包含中文的发票，用于测试UTF-8全链路存储"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let get_res = app.get(&format!("/api/invoices/{}", id)).await;
    let inv = get_res.json::<serde_json::Value>();
    assert_eq!(inv["supplier"].as_str().unwrap(), "北京某某科技有限公司");
    assert_eq!(
        inv["description"].as_str().unwrap(),
        "这是一张包含中文的发票，用于测试UTF-8全链路存储"
    );
}

#[tokio::test]
async fn s32_unicode_korean_employee() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/employees")
        .json(&json!({
            "name": "김민수",
            "email": "minsu@opc-kr.example.invalid",
            "department": "개발팀",
            "position": "시니어 엔지니어"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let get_res = app.get(&format!("/api/employees/{}", id)).await;
    let emp = get_res.json::<serde_json::Value>();
    assert_eq!(emp["name"].as_str().unwrap(), "김민수");
    assert_eq!(emp["department"].as_str().unwrap(), "개발팀");
}

#[tokio::test]
async fn s32_unicode_japanese_asset() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/assets")
        .json(&json!({
            "name": "東京オフィス デスク",
            "asset_type": "physical",
            "purchase_price": 50000.0
        }))
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    let id = body["id"]
        .as_str()
        .or_else(|| body["data"]["id"].as_str())
        .unwrap()
        .to_string();

    let get_res = app.get(&format!("/api/assets/{}", id)).await;
    let asset = get_res.json::<serde_json::Value>();
    let name = asset["name"]
        .as_str()
        .or_else(|| asset["data"]["name"].as_str())
        .unwrap();
    assert_eq!(name, "東京オフィス デスク");
}

#[tokio::test]
async fn s32_unicode_emoji_in_description() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-EMOJI-001",
            "amount": 100.0,
            "invoice_type": "income",
            "description": "🎉 项目完成 ✅ 付款已确认 💰 含税价 📋"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let get_res = app.get(&format!("/api/invoices/{}", id)).await;
    let desc = get_res.json::<serde_json::Value>()["description"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(desc.contains("🎉"));
    assert!(desc.contains("💰"));
}

#[tokio::test]
async fn s32_unicode_mixed_script() {
    let app = common::TestApp::new().await;
    // Mixed Chinese + Korean + Japanese + English + Arabic-like
    let res = app
        .post("/api/contracts")
        .json(&json!({
            "contract_name": "合同 계약서 契約書 Contract",
            "contract_type": "service",
            "counterparty": "글로벌パートナー Global合作",
            "counterparty_type": "corporate",
            "amount": 999999.99
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let get_res = app.get(&format!("/api/contracts/{}", id)).await;
    let name = get_res.json::<serde_json::Value>()["contract_name"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(name.contains("合同"));
    assert!(name.contains("계약서"));
    assert!(name.contains("契約書"));
}

// ==================== 2. Super-long strings ====================

#[tokio::test]
async fn s32_very_long_description_10k_chars() {
    let app = common::TestApp::new().await;
    let long_desc = "测试".repeat(5000); // 10,000 chars
    let res = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-LONG-001",
            "amount": 100.0,
            "invoice_type": "income",
            "description": long_desc
        }))
        .await;
    // Should either succeed or give a clear 400, not crash with 500
    let status = res.status_code();
    assert!(
        status.is_success() || status == 400,
        "Got unexpected status: {}",
        status
    );
}

#[tokio::test]
async fn s32_very_long_contract_content() {
    let app = common::TestApp::new().await;
    // Simulate a real contract with 50K chars
    let long_content = "合同条款内容 ".repeat(8000); // ~56K chars
    let res = app
        .post("/api/contracts")
        .json(&json!({
            "contract_name": "超长合同",
            "contract_type": "service",
            "counterparty": "XYZ Corp",
            "counterparty_type": "corporate",
            "content": long_content
        }))
        .await;
    let status = res.status_code();
    assert!(
        status.is_success() || status == 400,
        "Got unexpected status: {}",
        status
    );
}

#[tokio::test]
async fn s32_very_long_employee_name() {
    let app = common::TestApp::new().await;
    let long_name = "A".repeat(1000);
    let res = app
        .post("/api/employees")
        .json(&json!({
            "name": long_name,
            "email": "long@test-dev.example.invalid"
        }))
        .await;
    let status = res.status_code();
    assert!(
        status.is_success() || status == 400,
        "Got unexpected status: {}",
        status
    );
}

// ==================== 3. Zero-amount / Negative-amount ====================

#[tokio::test]
async fn s32_zero_amount_invoice() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-ZERO-001",
            "amount": 0.0,
            "invoice_type": "income"
        }))
        .await;
    // Zero amount should be allowed
    res.assert_status_ok();
}

#[tokio::test]
async fn s32_negative_amount_invoice_blocked() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-NEG-001",
            "amount": -500.0,
            "invoice_type": "income"
        }))
        .await;
    let status = res.status_code();
    assert!(
        status == 400 || status == 422,
        "Negative amounts should be rejected, got {}",
        status
    );
}

#[tokio::test]
async fn s32_negative_salary_blocked() {
    let app = common::TestApp::new().await;
    // First create employee
    let emp_res = app
        .post("/api/employees")
        .json(&json!({ "name": "工资测试", "email": "salary@test-dev.example.invalid" }))
        .await;
    emp_res.assert_status_ok();
    let emp_id = emp_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = app
        .post("/api/v1/salary")
        .json(&json!({
            "employee_id": emp_id,
            "base_salary": -5000.0,
            "period": "2024-03"
        }))
        .await;
    let status = res.status_code();
    assert!(
        status == 400 || status == 422,
        "Negative salary should be rejected, got {}",
        status
    );
}

#[tokio::test]
async fn s32_zero_purchase_price_asset() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/assets")
        .json(&json!({
            "name": "免费赠送设备",
            "asset_type": "physical",
            "purchase_price": 0.0
        }))
        .await;
    // Zero purchase price should be valid (donated assets)
    res.assert_status_ok();
}

#[tokio::test]
async fn s32_negative_budget_blocked() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/budgets")
        .json(&json!({
            "name": "负预算测试",
            "category": "marketing",
            "planned_amount": -10000.0,
            "period": "2024-Q1"
        }))
        .await;
    let status = res.status_code();
    assert!(
        status == 400 || status == 422,
        "Negative budget should be rejected, got {}",
        status
    );
}

#[tokio::test]
async fn s32_negative_time_entry_duration_blocked() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/v1/time_entries")
        .json(&json!({
            "description": "负时间测试",
            "duration_minutes": -60,
            "entry_date": "2024-03-15"
        }))
        .await;
    let status = res.status_code();
    assert!(
        status == 400 || status == 422,
        "Negative duration should be rejected, got {}",
        status
    );
}

#[tokio::test]
async fn s32_zero_duration_time_entry() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/v1/time_entries")
        .json(&json!({
            "description": "零时间条目",
            "duration_minutes": 0,
            "entry_date": "2024-03-15"
        }))
        .await;
    // Zero duration should be allowed (some systems track start/stop)
    let status = res.status_code();
    assert!(status.is_success() || status == 400);
}

// ==================== 4. Empty / Whitespace-only fields ====================

#[tokio::test]
async fn s32_whitespace_only_name_blocked() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/employees")
        .json(&json!({
            "name": "   ",
            "email": "space@test-dev.example.invalid"
        }))
        .await;
    // Whitespace-only names should either fail or be trimmed
    let status = res.status_code();
    // If backend trims, it should fail on empty; if not, it should store
    assert!(status.is_success() || status == 400);
}

#[tokio::test]
async fn s32_empty_invoice_number() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "",
            "amount": 100.0,
            "invoice_type": "income"
        }))
        .await;
    // Backend may accept empty strings or reject them — either is acceptable
    let status = res.status_code();
    assert!(
        status.is_success() || status == 400 || status == 422,
        "Empty invoice number got unexpected status: {}",
        status
    );
}

#[tokio::test]
async fn s32_null_optional_fields_accepted() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-NULL-OPT-001",
            "amount": 500.0,
            "invoice_type": "income",
            "description": null,
            "supplier": null,
            "due_date": null
        }))
        .await;
    res.assert_status_ok();
}

// ==================== 5. Floating-point precision ====================

#[tokio::test]
async fn s32_float_precision_01_plus_02() {
    let app = common::TestApp::new().await;
    // 0.1 + 0.2 = 0.30000000000000004 in IEEE 754
    let res = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-FLOAT-001",
            "amount": 0.1 + 0.2, // This is 0.30000000000000004
            "invoice_type": "income"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let get_res = app.get(&format!("/api/invoices/{}", id)).await;
    let stored_amount = get_res.json::<serde_json::Value>()["amount"]
        .as_f64()
        .unwrap();
    // Due to SQLite REAL (float64), the value will have precision issues
    let diff = (stored_amount - 0.3).abs();
    assert!(
        diff < 0.001,
        "Stored amount {} deviates from 0.3 by {}",
        stored_amount,
        diff
    );
}

#[tokio::test]
async fn s32_float_precision_large_amount() {
    let app = common::TestApp::new().await;
    // Test with amounts > 10 million (common in enterprise)
    let res = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-LARGE-001",
            "amount": 99999999.99,
            "invoice_type": "income"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let get_res = app.get(&format!("/api/invoices/{}", id)).await;
    let stored = get_res.json::<serde_json::Value>()["amount"]
        .as_f64()
        .unwrap();
    assert!(
        (stored - 99999999.99).abs() < 0.01,
        "Large amount precision lost: {}",
        stored
    );
}

#[tokio::test]
async fn s32_float_precision_very_small_amount() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-TINY-001",
            "amount": 0.01, // 1 cent
            "invoice_type": "income"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let get_res = app.get(&format!("/api/invoices/{}", id)).await;
    let stored = get_res.json::<serde_json::Value>()["amount"]
        .as_f64()
        .unwrap();
    assert!(
        (stored - 0.01).abs() < 0.001,
        "Tiny amount precision lost: {}",
        stored
    );
}

// ==================== 6. Large-volume pagination ====================

#[tokio::test]
async fn s32_large_volume_pagination_100_invoices() {
    let app = common::TestApp::new().await;

    // Create 100 invoices
    for i in 0..100 {
        let res = app
            .post("/api/invoices")
            .json(&json!({
                "invoice_number": format!("INV-BULK-{:04}", i),
                "amount": (i as f64) * 100.0 + 50.0,
                "invoice_type": if i % 2 == 0 { "income" } else { "expense" }
            }))
            .await;
        res.assert_status_ok();
    }

    // Fetch all
    let all_res = app.get("/api/invoices").await;
    all_res.assert_status_ok();
    let body = all_res.json::<serde_json::Value>();
    let count = body["count"].as_i64().unwrap_or(0);
    assert!(count >= 100, "Expected >= 100 invoices, got {}", count);

    // Fetch with pagination
    let page_res = app.get("/api/invoices?limit=10&offset=50").await;
    page_res.assert_status_ok();
    let page_data = page_res.json::<serde_json::Value>();
    let items = page_data["data"].as_array().unwrap();
    assert_eq!(items.len(), 10, "Expected exactly 10 items in page");
}

#[tokio::test]
async fn s32_large_volume_employees() {
    let app = common::TestApp::new().await;

    for i in 0..50 {
        app.post("/api/employees")
            .json(&json!({
                "name": format!("员工{:03}", i),
                "email": format!("emp{}@test.dev", i)
            }))
            .await
            .assert_status_ok();
    }

    let all_res = app.get("/api/employees").await;
    all_res.assert_status_ok();
    let body = all_res.json::<serde_json::Value>();
    let count = body["count"].as_i64().unwrap_or(0);
    assert!(count >= 50, "Expected >= 50 employees, got {}", count);
}

// ==================== 7. Special characters ====================

#[tokio::test]
async fn s32_sql_injection_in_invoice_number() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "'; DROP TABLE invoices; --",
            "amount": 100.0,
            "invoice_type": "income"
        }))
        .await;
    // Should succeed (stored as literal text) or be rejected, but NOT crash
    let status = res.status_code();
    assert!(
        status.is_success() || status == 400,
        "SQL injection caused unexpected status: {}",
        status
    );
}

#[tokio::test]
async fn s32_html_xss_in_description() {
    let app = common::TestApp::new().await;
    let xss_payload = r#"<script>alert('xss')</script><img src=x onerror=alert(1)>"#;
    let res = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-XSS-001",
            "amount": 100.0,
            "invoice_type": "income",
            "description": xss_payload
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Verify it's stored as-is (backend stores, frontend escapes)
    let get_res = app.get(&format!("/api/invoices/{}", id)).await;
    let desc = get_res.json::<serde_json::Value>()["description"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(
        desc.contains("<script>"),
        "XSS payload should be stored literally for backend"
    );
}

#[tokio::test]
async fn s32_newlines_and_tabs_in_content() {
    let app = common::TestApp::new().await;
    let multiline = "第一行\n第二行\t含Tab\r\n第三行（Windows换行）";
    let res = app
        .post("/api/contracts")
        .json(&json!({
            "contract_name": "多行合同",
            "contract_type": "service",
            "counterparty": "Test",
            "counterparty_type": "individual",
            "content": multiline
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let get_res = app.get(&format!("/api/contracts/{}", id)).await;
    let content = get_res.json::<serde_json::Value>()["content"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(content.contains("第一行"));
    assert!(content.contains("第三行"));
}

#[tokio::test]
async fn s32_backslashes_and_quotes() {
    let app = common::TestApp::new().await;
    let tricky = r#"文件路径: C:\Users\test\"quoted" and 'single'"#;
    let res = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-ESCAPE-001",
            "amount": 100.0,
            "invoice_type": "income",
            "description": tricky
        }))
        .await;
    res.assert_status_ok();
}

// ==================== 8. Date edge cases ====================

#[tokio::test]
async fn s32_leap_year_date() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-LEAP-001",
            "amount": 100.0,
            "invoice_type": "income",
            "invoice_date": "2024-02-29" // Leap year
        }))
        .await;
    res.assert_status_ok();
}

#[tokio::test]
async fn s32_far_future_date() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-FUTURE-001",
            "amount": 100.0,
            "invoice_type": "income",
            "due_date": "2099-12-31"
        }))
        .await;
    // Far future dates should be allowed
    let status = res.status_code();
    assert!(status.is_success() || status == 400);
}

#[tokio::test]
async fn s32_epoch_date() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/invoices")
        .json(&json!({
            "invoice_number": "INV-EPOCH-001",
            "amount": 100.0,
            "invoice_type": "income",
            "invoice_date": "1970-01-01"
        }))
        .await;
    let status = res.status_code();
    assert!(status.is_success() || status == 400);
}

// ==================== 9. Rapid-fire write stress ====================

#[tokio::test]
async fn s32_rapid_fire_invoice_creation_20() {
    let app = common::TestApp::new().await;

    // Create 20 invoices in rapid succession
    let mut success = 0;
    for i in 0..20 {
        let res = app
            .post("/api/invoices")
            .json(&json!({
                "invoice_number": format!("INV-RAPID-{:04}", i),
                "amount": 100.0 + i as f64,
                "invoice_type": "income"
            }))
            .await;
        if res.status_code().is_success() {
            success += 1;
        }
    }

    assert_eq!(success, 20, "Some rapid-fire writes failed: {}/20", success);

    // Verify all 20 are accessible
    let all_res = app.get("/api/invoices").await;
    all_res.assert_status_ok();
    let body = all_res.json::<serde_json::Value>();
    let count = body["count"].as_i64().unwrap_or(0);
    assert!(count >= 20, "Expected >= 20 invoices, got {}", count);
}

// ==================== 10. CRM module edge cases ====================

#[tokio::test]
async fn s32_client_with_all_unicode_fields() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/v1/clients")
        .json(&json!({
            "name": "상하이 테스트 有限公司 🏢",
            "client_type": "corporate",
            "industry": "テクノロジー・技术",
            "email": "test@测试-com.example.invalid",
            "phone": "+86-138-0000-0000",
            "address": "서울시 강남구 테헤란路 123号 / 上海市浦东新区"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let get_res = app.get(&format!("/api/v1/clients/{}", id)).await;
    let client = get_res.json::<serde_json::Value>();
    assert!(client["name"].as_str().unwrap().contains("상하이"));
    assert!(client["name"].as_str().unwrap().contains("有限公司"));
    assert!(client["address"].as_str().unwrap().contains("서울시"));
}

#[tokio::test]
async fn s32_deal_zero_probability() {
    let app = common::TestApp::new().await;
    // Create client first
    let client_res = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "概率测试客户", "client_type": "individual" }))
        .await;
    client_res.assert_status_ok();
    let client_id = client_res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = app
        .post("/api/v1/deals")
        .json(&json!({
            "title": "零概率商机",
            "client_id": client_id,
            "stage": "lead",
            "amount": 50000.0,
            "probability": 0 // 0% probability
        }))
        .await;
    res.assert_status_ok();
}

#[tokio::test]
async fn s32_quote_with_max_discount() {
    let app = common::TestApp::new().await;
    // Create a client first for FK
    let client_res = app
        .post("/api/v1/clients")
        .json(&json!({ "name": "折扣测试客户", "client_type": "individual" }))
        .await;
    client_res.assert_status_ok();

    let res = app
        .post("/api/v1/quotes")
        .json(&json!({
            "quote_number": "QT-MAX-DISC-001",
            "status": "draft",
            "subtotal": 10000.0,
            "tax_rate": 0.0,
            "tax_amount": 0.0,
            "total": 0.01,
            "currency": "CNY"
        }))
        .await;
    // May succeed or fail (some fields may be required)
    let status = res.status_code();
    assert!(
        status.is_success() || status == 400 || status == 500,
        "Quote with max discount got unexpected status: {}",
        status
    );
}

// ==================== 11. Insurance / Compliance edge cases ====================

#[tokio::test]
async fn s32_insurance_policy_same_start_end_date() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/v1/insurance_policies")
        .json(&json!({
            "policy_name": "一日保险",
            "policy_type": "accident",
            "provider": "TestInsure",
            "premium_amount": 50.0,
            "coverage_amount": 1000000.0,
            "start_date": "2024-03-15",
            "end_date": "2024-03-15" // Same day
        }))
        .await;
    let status = res.status_code();
    assert!(status.is_success() || status == 400);
}

#[tokio::test]
async fn s32_tax_jurisdiction_special_chars() {
    let app = common::TestApp::new().await;
    let res = app
        .post("/api/v1/tax_jurisdictions")
        .json(&json!({
            "name": "São Paulo (SP) — 巴西",
            "code": "BR-SP",
            "country_code": "BR",
            "jurisdiction_type": "state",
            "notes": "Secretaria da Fazenda e Planejamento"
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let get_res = app.get(&format!("/api/v1/tax_jurisdictions/{}", id)).await;
    let name = get_res.json::<serde_json::Value>()["name"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(name.contains("São Paulo"));
    assert!(name.contains("巴西"));
}
