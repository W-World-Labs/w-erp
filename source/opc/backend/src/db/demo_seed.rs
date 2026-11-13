// Public demonstration data only; personal identifiers use deliberately invalid placeholders.
use sqlx::SqlitePool;
use uuid::Uuid;

pub const DEMO_COMPANY_NAME: &str = "Demo - 语数科技";
pub const DEMO_COMPANY_LEGAL_NAME: &str = "虚构示例科技有限公司";
/// Fake unified social credit code carried only by the seeded demo company. Used
/// as a robust provenance fingerprint (real companies cannot have this), e.g. to
/// backfill the is_demo flag for demo companies created before the column existed
/// — far safer than matching on the user-editable display name.
pub const DEMO_COMPANY_USCC: &str = "91440300DEMOYUSHU";

pub async fn ensure_demo_company_for_user(
    pool: &SqlitePool,
    user_id: &str,
    _user_email: &str,
) -> Result<String, sqlx::Error> {
    if let Some(company_id) = sqlx::query_scalar::<_, String>(
        r#"
        SELECT cm.company_id
        FROM company_members cm
        JOIN companies c ON c.id = cm.company_id
        WHERE cm.user_id = ?
          AND c.is_demo = 1
          AND COALESCE(c.is_deleted, 0) = 0
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    {
        seed_demo_company_records(pool, &company_id, user_id).await?;
        return Ok(company_id);
    }

    let existing_memberships = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM company_members cm
        JOIN companies c ON c.id = cm.company_id
        WHERE cm.user_id = ? AND COALESCE(c.is_deleted, 0) = 0
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let company_id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO companies (
            id, name, legal_name, tax_id, registration_number, address, city, province,
            postal_code, phone, email, website, industry, founded_date, description,
            legal_representative, unified_social_credit_code, registered_capital,
            country, timezone, locale, base_currency, is_demo
        )
        VALUES (
            ?, ?, ?, '91440300DEMOYUSHU', '440300-DEMO-2024-001',
            '深圳市南山区粤海街道科苑南路 2666 号', '深圳', '广东省',
            '518000', '+86-755-2666-2024', 'demo@yushu-tech.example',
            'https://demo.yushu-tech.example', '人工智能与企业数字化服务',
            '2021-06-18',
            '面向成长型科技公司的经营、财税、合同、政策与资产管理样本公司。',
            '林语杉', '91440300DEMOYUSHU', 5000000,
            'CN', 'Asia/Shanghai', 'zh', 'CNY', 1
        )
        "#,
    )
    .bind(&company_id)
    .bind(DEMO_COMPANY_NAME)
    .bind(DEMO_COMPANY_LEGAL_NAME)
    .execute(pool)
    .await?;

    let member_id = Uuid::new_v4().to_string();
    let is_default = if existing_memberships == 0 { 1 } else { 0 };
    sqlx::query(
        r#"
        INSERT INTO company_members (id, user_id, company_id, role, permissions, is_default)
        VALUES (?, ?, ?, 'owner', '{"*":"rw"}', ?)
        "#,
    )
    .bind(&member_id)
    .bind(user_id)
    .bind(&company_id)
    .bind(is_default)
    .execute(pool)
    .await?;

    seed_demo_company_records(pool, &company_id, user_id).await?;
    Ok(company_id)
}

/// Whether `user_id` already owns a (non-deleted) demo company.
pub async fn user_has_demo_company(pool: &SqlitePool, user_id: &str) -> Result<bool, sqlx::Error> {
    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM company_members cm
        JOIN companies c ON c.id = cm.company_id
        WHERE cm.user_id = ?
          AND c.is_demo = 1
          AND COALESCE(c.is_deleted, 0) = 0
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

/// Backfill a demo company for every existing user who does not have one yet.
/// Idempotent: users who already own a demo company are skipped. Returns the
/// number of users newly seeded. Used at startup so pre-Demo-feature users get
/// sample data without having to wait for their next login.
pub async fn backfill_demo_for_all_users(pool: &SqlitePool) -> Result<usize, sqlx::Error> {
    // Cheap pre-check: once every user has a demo company this is a single COUNT,
    // so the common (already-backfilled) startup path does no per-user work.
    let pending = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM users u
        WHERE NOT EXISTS (
            SELECT 1 FROM company_members cm
            JOIN companies c ON c.id = cm.company_id
            WHERE cm.user_id = u.id AND c.is_demo = 1 AND COALESCE(c.is_deleted, 0) = 0
        )
        "#,
    )
    .fetch_one(pool)
    .await?;
    if pending == 0 {
        return Ok(0);
    }

    let users = sqlx::query_as::<_, (String, String)>("SELECT id, email FROM users")
        .fetch_all(pool)
        .await?;

    let mut seeded = 0usize;
    for (user_id, email) in users {
        // A single user's transient error (guard or seed) must not abort the
        // whole backfill — log and move on.
        match user_has_demo_company(pool, &user_id).await {
            Ok(true) => continue,
            Ok(false) => {}
            Err(e) => {
                tracing::warn!("demo backfill guard failed for user {}: {}", user_id, e);
                continue;
            }
        }
        match ensure_demo_company_for_user(pool, &user_id, &email).await {
            Ok(_) => seeded += 1,
            Err(e) => tracing::warn!("demo backfill failed for user {}: {}", user_id, e),
        }
    }
    Ok(seeded)
}

pub async fn seed_demo_company_records(
    pool: &SqlitePool,
    company_id: &str,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    seed_company_profile(pool, company_id, user_id).await?;
    seed_settings(pool, company_id, user_id).await?;
    seed_branch(pool, company_id).await?;
    seed_social_insurance(pool, company_id).await?;

    let category_id = ensure_asset_category(pool, company_id).await?;
    let client_id = ensure_client(pool, company_id, user_id).await?;
    let contact_id = ensure_contact(pool, company_id, &client_id, user_id).await?;
    seed_interaction(pool, company_id, &client_id, &contact_id, user_id).await?;

    let project_id = ensure_project(pool, company_id).await?;
    let employee_id = ensure_employee(pool, company_id).await?;
    let deal_id = ensure_deal(pool, company_id, &client_id, &contact_id, user_id).await?;
    let quote_id =
        ensure_quote(pool, company_id, &client_id, &contact_id, &deal_id, user_id).await?;
    seed_quote_item(pool, company_id, &quote_id).await?;
    seed_recurring_invoice(pool, company_id, &client_id, user_id).await?;

    let contract_id = ensure_contract(pool, company_id, &project_id).await?;
    let invoice_id = ensure_invoice(pool, company_id, &contract_id, user_id).await?;
    let bank_account_id = ensure_bank_account(pool, company_id).await?;
    let payment_id = ensure_payment(
        pool,
        company_id,
        &bank_account_id,
        &invoice_id,
        &contract_id,
    )
    .await?;
    seed_bank_transaction(pool, company_id, &bank_account_id, &payment_id).await?;
    seed_invoice_reminder(pool, company_id, &invoice_id, &client_id, user_id).await?;

    let asset_id = ensure_asset(
        pool,
        company_id,
        &category_id,
        &invoice_id,
        &contract_id,
        user_id,
    )
    .await?;
    let data_asset_id = ensure_data_asset(pool, company_id, &category_id, user_id).await?;
    seed_asset_record(pool, company_id, &asset_id).await?;
    seed_data_asset_meta(pool, company_id, &data_asset_id).await?;

    seed_salary(pool, company_id, &employee_id).await?;
    seed_budget(pool, company_id, &project_id).await?;
    seed_rent(pool, company_id, &contract_id).await?;
    seed_license(pool, company_id).await?;
    seed_time_entry(
        pool,
        company_id,
        &project_id,
        &client_id,
        &employee_id,
        &invoice_id,
        user_id,
    )
    .await?;
    seed_milestone(pool, company_id, &project_id).await?;
    seed_application_assets(pool, company_id, &project_id, user_id).await?;
    seed_tax_assets(pool, company_id, &invoice_id, &payment_id, user_id).await?;
    seed_cloud_assets(pool, company_id, &project_id).await?;
    seed_subscription_assets(pool, company_id, &payment_id, &contract_id).await?;
    seed_compliance_assets(pool, company_id, &employee_id, user_id).await?;
    seed_vendor(pool, company_id).await?;
    seed_rd_cost(
        pool,
        company_id,
        &project_id,
        &employee_id,
        &asset_id,
        &invoice_id,
    )
    .await?;
    seed_kpi(pool, company_id).await?;
    seed_exchange_rate(pool, company_id).await?;
    seed_reminder(pool, company_id, &contract_id).await?;
    seed_inactive_webhook(pool, company_id, user_id).await?;
    seed_demo_error_log(pool, company_id, user_id).await?;

    Ok(())
}

async fn first_id(
    pool: &SqlitePool,
    table: &str,
    company_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    let sql =
        format!("SELECT id FROM {table} WHERE company_id = ? ORDER BY created_at, id LIMIT 1");
    sqlx::query_scalar::<_, String>(&sql)
        .bind(company_id)
        .fetch_optional(pool)
        .await
}

async fn has_records(
    pool: &SqlitePool,
    table: &str,
    company_id: &str,
) -> Result<bool, sqlx::Error> {
    let sql = format!("SELECT COUNT(*) FROM {table} WHERE company_id = ?");
    let count = sqlx::query_scalar::<_, i64>(&sql)
        .bind(company_id)
        .fetch_one(pool)
        .await?;
    Ok(count > 0)
}

async fn seed_company_profile(
    pool: &SqlitePool,
    company_id: &str,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    if !has_records(pool, "company_info", company_id).await? {
        sqlx::query(
            r#"
            INSERT INTO company_info (
                id, company_id, company_name, legal_representative,
                unified_social_credit_code, registration_date, registered_capital,
                taxpayer_type, tax_registration_number, tax_authority, vat_rate,
                invoice_limit, cash_flow_threshold, contact_phone, contact_email,
                founder_education, founder_is_expat, founder_is_overseas_returnee,
                company_address, business_scope, notes
            )
            VALUES (
                ?, ?, ?, '林语杉', '91440300DEMOYUSHU', '2021-06-18', 5000000,
                'general', '91440300DEMOYUSHU', '国家税务总局深圳市南山区税务局', 0.06,
                1000000, 300000, '+86-755-2666-2024', 'finance@yushu-tech.example',
                '硕士', 0, 1, '深圳市南山区粤海街道科苑南路 2666 号',
                '软件开发、人工智能应用、数据处理服务、企业管理咨询。',
                'Demo 公司档案用于新用户完整体验平台能力。'
            )
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(company_id)
        .bind(DEMO_COMPANY_LEGAL_NAME)
        .execute(pool)
        .await?;
    }

    if !has_records(pool, "profiles", company_id).await? {
        sqlx::query(
            r#"
            INSERT INTO profiles (
                id, company_id, user_id, display_name, phone, department,
                position, email, bio
            )
            VALUES (?, ?, ?, 'Demo 体验管理员', '+86-755-2666-0001',
                    '经营管理部', '所有者', 'owner@yushu-tech.example',
                    '负责 Demo 公司经营、财税、政策和合同全流程体验。')
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(company_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    }

    if !has_records(pool, "user_roles", company_id).await? {
        sqlx::query(
            r#"
            INSERT INTO user_roles (id, company_id, user_id, role, permissions)
            VALUES (?, ?, ?, 'owner', '{"contracts":"rw","finance":"rw","policies":"rw"}')
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(company_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_settings(
    pool: &SqlitePool,
    company_id: &str,
    _user_id: &str,
) -> Result<(), sqlx::Error> {
    if !has_records(pool, "ai_configurations", company_id).await? {
        sqlx::query(
            r#"
            INSERT INTO ai_configurations (id, company_id, provider, model, endpoint, enabled)
            VALUES (?, ?, 'deepseek', 'deepseek-v4-flash', 'https://api.deepseek.com', 0)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(company_id)
        .execute(pool)
        .await?;
    }

    if !has_records(pool, "email_configurations", company_id).await? {
        sqlx::query(
            r#"
            INSERT INTO email_configurations (
                id, company_id, smtp_host, smtp_port, smtp_user,
                sender_name, sender_email, enabled
            )
            VALUES (?, ?, 'smtp.demo.yushu-tech.example', 587,
                    'notice@yushu-tech.example', '语数科技财务通知',
                    'notice@yushu-tech.example', 0)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(company_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_branch(pool: &SqlitePool, company_id: &str) -> Result<(), sqlx::Error> {
    if has_records(pool, "branches", company_id).await? {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO branches (
            id, company_id, branch_name, branch_code, branch_type,
            legal_representative, registered_capital, contact_person,
            contact_phone, contact_email, branch_address, business_scope,
            registration_date, employee_count, status, notes
        )
        VALUES (?, ?, '语数科技杭州研发中心', 'HZ-RD', 'branch',
                '周知远', '1000000', '周知远', '+86-571-8800-2024',
                'rd-hz@yushu-tech.example', '杭州市余杭区未来科技城',
                '智能文档解析、经营分析模型研发', '2023-04-10', 12,
                'active', '覆盖研发费用归集、云成本和数据资产示例。')
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .execute(pool)
    .await
    .map(|_| ())
}

async fn seed_social_insurance(pool: &SqlitePool, company_id: &str) -> Result<(), sqlx::Error> {
    if has_records(pool, "social_insurance_configs", company_id).await? {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO social_insurance_configs (
            id, company_id, city, social_security_base_min, social_security_base_max,
            housing_fund_base_min, housing_fund_base_max, pension_personal_rate,
            pension_company_rate, medical_personal_rate, medical_company_rate,
            unemployment_personal_rate, unemployment_company_rate, work_injury_company_rate,
            maternity_company_rate, housing_fund_personal_rate, housing_fund_company_rate,
            notes
        )
        VALUES (?, ?, '深圳', 2360, 41190, 2360, 41190,
                0.08, 0.16, 0.02, 0.06, 0.003, 0.007, 0.001, 0.005,
                0.07, 0.07, 'Demo 社保公积金参数用于薪酬与合规测试。')
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .execute(pool)
    .await
    .map(|_| ())
}

async fn ensure_asset_category(pool: &SqlitePool, company_id: &str) -> Result<String, sqlx::Error> {
    if let Some(id) = first_id(pool, "asset_categories", company_id).await? {
        return Ok(id);
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO asset_categories (id, company_id, name, category_type, description)
        VALUES (?, ?, '研发与数据资产', 'technology', '服务器、软件、数据集和算法模型资产')
        "#,
    )
    .bind(&id)
    .bind(company_id)
    .execute(pool)
    .await?;
    Ok(id)
}

async fn ensure_client(
    pool: &SqlitePool,
    company_id: &str,
    user_id: &str,
) -> Result<String, sqlx::Error> {
    if let Some(id) = first_id(pool, "clients", company_id).await? {
        return Ok(id);
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO clients (
            id, company_id, name, short_name, client_type, industry,
            website, email, phone, address, city, country, tax_id,
            source, notes, tags, annual_revenue, employee_count, created_by
        )
        VALUES (?, ?, '星河制造集团有限公司', '星河制造', 'active',
                '高端制造', 'https://demo-client.example',
                'it@gxmfg.example', '+86-755-8899-6600',
                '深圳市龙华区智能制造产业园', '深圳', 'CN',
                '91440300DEMOCLIENT', 'referral',
                '核心客户，采购经营分析平台与文档智能服务。',
                '["vip","manufacturing","ai-docs"]', 280000000, 620, ?)
        "#,
    )
    .bind(&id)
    .bind(company_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(id)
}

async fn ensure_contact(
    pool: &SqlitePool,
    company_id: &str,
    client_id: &str,
    user_id: &str,
) -> Result<String, sqlx::Error> {
    if let Some(id) = first_id(pool, "contacts", company_id).await? {
        return Ok(id);
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO contacts (
            id, company_id, client_id, name, title, email, phone,
            mobile, department, is_primary, notes, created_by
        )
        VALUES (?, ?, ?, '陈卓', '数字化负责人', 'chenzhuo@gxmfg.example',
                '+86-755-8899-6618', '+86-138-0000-2024', '信息化中心',
                1, 'Demo 主联系人，覆盖 CRM、报价、合同和回款流程。', ?)
        "#,
    )
    .bind(&id)
    .bind(company_id)
    .bind(client_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(id)
}

async fn seed_interaction(
    pool: &SqlitePool,
    company_id: &str,
    client_id: &str,
    contact_id: &str,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    if has_records(pool, "interactions", company_id).await? {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO interactions (
            id, company_id, client_id, contact_id, interaction_type,
            subject, description, interaction_date, duration_minutes,
            outcome, next_action, next_action_date, created_by
        )
        VALUES (?, ?, ?, ?, 'meeting', '经营分析平台需求澄清',
                '确认合同分析、政策匹配、发票回款和云成本看板为首批上线范围。',
                '2026-05-20T10:00:00Z', 60, 'positive',
                '发送正式报价与试点合同', '2026-05-27', ?)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .bind(client_id)
    .bind(contact_id)
    .bind(user_id)
    .execute(pool)
    .await
    .map(|_| ())
}

async fn ensure_project(pool: &SqlitePool, company_id: &str) -> Result<String, sqlx::Error> {
    if let Some(id) = first_id(pool, "projects", company_id).await? {
        return Ok(id);
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO projects (
            id, company_id, name, description, project_type, leader_name,
            status, priority, budget, spent, start_date, end_date, client,
            progress, team_members, goals, notes, milestones
        )
        VALUES (?, ?, '智能经营分析平台一期', '为制造业客户建设合同、财税、政策、资产一体化工作台。',
                'client_delivery', '周知远', 'active', 'high', 1200000,
                430000, '2026-04-01', '2026-09-30', '星河制造集团有限公司',
                58, 6, '交付政策分析、合同风险审查、回款预测和云成本治理。',
                'Demo 项目用于串联项目、预算、工时、研发费用和材料管理。',
                '[{"title":"需求确认","status":"done"},{"title":"MVP上线","status":"active"}]')
        "#,
    )
    .bind(&id)
    .bind(company_id)
    .execute(pool)
    .await?;
    Ok(id)
}

async fn ensure_employee(pool: &SqlitePool, company_id: &str) -> Result<String, sqlx::Error> {
    if let Some(id) = first_id(pool, "employees", company_id).await? {
        return Ok(id);
    }

    let lead_id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO employees (
            id, company_id, name, full_name, employee_number, email, phone,
            department, position, hire_date, status, employment_type,
            base_salary, social_security_base, housing_fund_base, notes,
            contract_start_date, contract_end_date, bank_account, bank_name,
            emergency_contact, emergency_phone, id_number
        )
        VALUES (?, ?, '周知远', '周知远', 'YS-2024-001',
                'zyzhou@yushu-tech.example', '+86-139-0000-2024',
                '研发部', 'AI 产品负责人', '2024-02-18', 'active',
                'full_time', 42000, 30000, 30000,
                '负责文档解析、政策分析与合同智能审查能力。',
                '2024-02-18', '2027-02-17', '0002020400002024',
                '招商银行深圳分行', '林语杉', '+86-138-0000-0001',
                '000300199001012024')
        "#,
    )
    .bind(&lead_id)
    .bind(company_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO employees (
            id, company_id, name, full_name, employee_number, email, phone,
            department, position, hire_date, status, employment_type,
            base_salary, social_security_base, housing_fund_base, notes
        )
        VALUES (?, ?, '许明月', '许明月', 'YS-2024-002',
                'finance@yushu-tech.example', '+86-137-0000-2024',
                '财务部', '财税运营负责人', '2024-05-06', 'active',
                'full_time', 32000, 28000, 28000,
                '负责发票、薪酬、税务申报和回款管理。')
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .execute(pool)
    .await?;

    Ok(lead_id)
}

async fn ensure_deal(
    pool: &SqlitePool,
    company_id: &str,
    client_id: &str,
    contact_id: &str,
    user_id: &str,
) -> Result<String, sqlx::Error> {
    if let Some(id) = first_id(pool, "deals", company_id).await? {
        return Ok(id);
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO deals (
            id, company_id, client_id, contact_id, title, description,
            stage, amount, currency, probability, expected_close_date,
            source, assigned_to, tags, created_by
        )
        VALUES (?, ?, ?, ?, '经营分析平台年度服务', '合同、财税、政策与文档智能年度服务包',
                'proposal', 680000, 'CNY', 75, '2026-06-30',
                'referral', '林语杉', '["enterprise","policy","contract-ai"]', ?)
        "#,
    )
    .bind(&id)
    .bind(company_id)
    .bind(client_id)
    .bind(contact_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(id)
}

async fn ensure_quote(
    pool: &SqlitePool,
    company_id: &str,
    client_id: &str,
    contact_id: &str,
    deal_id: &str,
    user_id: &str,
) -> Result<String, sqlx::Error> {
    if let Some(id) = first_id(pool, "quotes", company_id).await? {
        return Ok(id);
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO quotes (
            id, company_id, client_id, contact_id, deal_id, quote_number,
            title, status, subtotal, tax_rate, tax_amount, discount,
            total, currency, valid_until, notes, terms, created_by
        )
        VALUES (?, ?, ?, ?, ?, 'QT-YS-2026-001', '经营分析平台一期报价',
                'sent', 641509.43, 0.06, 38490.57, 0, 680000,
                'CNY', '2026-06-15', 'Demo 报价覆盖报价单、明细、转合同流程。',
                '签约后 7 日内支付 30% 预付款，验收后支付尾款。', ?)
        "#,
    )
    .bind(&id)
    .bind(company_id)
    .bind(client_id)
    .bind(contact_id)
    .bind(deal_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(id)
}

async fn seed_quote_item(
    pool: &SqlitePool,
    company_id: &str,
    quote_id: &str,
) -> Result<(), sqlx::Error> {
    if has_records(pool, "quote_items", company_id).await? {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO quote_items (
            id, company_id, quote_id, description, quantity, unit,
            unit_price, discount_percent, amount, sort_order
        )
        VALUES (?, ?, ?, '平台实施与 AI 工作流配置', 1, '项', 680000, 0, 680000, 1)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .bind(quote_id)
    .execute(pool)
    .await
    .map(|_| ())
}

async fn seed_recurring_invoice(
    pool: &SqlitePool,
    company_id: &str,
    client_id: &str,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    if has_records(pool, "recurring_invoices", company_id).await? {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO recurring_invoices (
            id, company_id, client_id, title, description, amount,
            currency, tax_rate, frequency, start_date, next_invoice_date,
            day_of_month, auto_send, payment_terms_days, status,
            total_generated, notes, created_by
        )
        VALUES (?, ?, ?, '企业智能运营订阅费', '月度托管、模型调用和运营支持服务',
                48000, 'CNY', 0.06, 'monthly', '2026-06-01',
                '2026-06-01', 1, 0, 15, 'active', 0,
                'Demo 周期发票模板用于体验自动开票和催款提醒。', ?)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .bind(client_id)
    .bind(user_id)
    .execute(pool)
    .await
    .map(|_| ())
}

async fn ensure_contract(
    pool: &SqlitePool,
    company_id: &str,
    project_id: &str,
) -> Result<String, sqlx::Error> {
    if let Some(id) = first_id(pool, "contracts", company_id).await? {
        return Ok(id);
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO contracts (
            id, company_id, contract_name, contract_number, contract_type,
            counterparty, party_a, party_b, amount, start_date, end_date,
            status, project_id, content, notes, ai_review_status,
            ai_risk_level, ai_review_result, ai_reviewed_at, counterparty_type
        )
        VALUES (?, ?, '星河制造经营分析平台服务合同', 'YS-HT-2026-001',
                'service', '星河制造集团有限公司', '虚构示例科技有限公司',
                '星河制造集团有限公司', 680000, '2026-06-01', '2027-05-31',
                'active', ?,
                '服务范围包括合同分析、政策匹配、发票回款、云成本与数据资产治理。',
                'Demo 合同已内置 AI 审查结果，便于体验合同分析工作流。',
                'completed', '🟡 medium',
                '主要风险：验收标准需量化；数据处理责任需补充；模型输出仅作为辅助建议。',
                CURRENT_TIMESTAMP, 'client')
        "#,
    )
    .bind(&id)
    .bind(company_id)
    .bind(project_id)
    .execute(pool)
    .await?;
    Ok(id)
}

async fn ensure_invoice(
    pool: &SqlitePool,
    company_id: &str,
    contract_id: &str,
    user_id: &str,
) -> Result<String, sqlx::Error> {
    if let Some(id) = first_id(pool, "invoices", company_id).await? {
        return Ok(id);
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO invoices (
            id, company_id, invoice_number, invoice_type, amount,
            tax_amount, status, invoice_date, due_date, supplier,
            description, created_by, contract_id
        )
        VALUES (?, ?, 'INV-YS-2026-001', 'income', 204000, 12240,
                'issued', '2026-06-01', '2026-06-16',
                '星河制造集团有限公司', '平台一期预付款发票', ?, ?)
        "#,
    )
    .bind(&id)
    .bind(company_id)
    .bind(user_id)
    .bind(contract_id)
    .execute(pool)
    .await?;
    Ok(id)
}

async fn ensure_bank_account(pool: &SqlitePool, company_id: &str) -> Result<String, sqlx::Error> {
    if let Some(id) = first_id(pool, "bank_accounts", company_id).await? {
        return Ok(id);
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO bank_accounts (
            id, company_id, bank_name, account_name, account_number,
            account_type, balance, currency, status, notes
        )
        VALUES (?, ?, '招商银行深圳科技园支行', '虚构示例科技有限公司',
                '75590000002024', 'checking', 1250000, 'CNY',
                'active', 'Demo 经营账户，用于现金流和回款测试。')
        "#,
    )
    .bind(&id)
    .bind(company_id)
    .execute(pool)
    .await?;
    Ok(id)
}

async fn ensure_payment(
    pool: &SqlitePool,
    company_id: &str,
    bank_account_id: &str,
    invoice_id: &str,
    contract_id: &str,
) -> Result<String, sqlx::Error> {
    if let Some(id) = first_id(pool, "payments", company_id).await? {
        return Ok(id);
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO payments (
            id, company_id, payment_no, payment_type, payment_method,
            amount, currency, category, status, payee, payer,
            bank_account_id, invoice_id, contract_id, description,
            payment_date, due_date, invoice_required
        )
        VALUES (?, ?, 'PAY-YS-2026-001', 'income', 'bank_transfer',
                204000, 'CNY', 'project_prepayment', 'completed',
                '虚构示例科技有限公司', '星河制造集团有限公司',
                ?, ?, ?, '平台一期预付款回款', '2026-06-05',
                '2026-06-16', 1)
        "#,
    )
    .bind(&id)
    .bind(company_id)
    .bind(bank_account_id)
    .bind(invoice_id)
    .bind(contract_id)
    .execute(pool)
    .await?;
    Ok(id)
}

async fn seed_bank_transaction(
    pool: &SqlitePool,
    company_id: &str,
    bank_account_id: &str,
    payment_id: &str,
) -> Result<(), sqlx::Error> {
    if has_records(pool, "bank_transactions", company_id).await? {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO bank_transactions (
            id, company_id, account_id, transaction_type, amount,
            balance_after, counterparty, description, transaction_date, payment_id
        )
        VALUES (?, ?, ?, 'income', 204000, 1454000,
                '星河制造集团有限公司', '平台一期预付款到账',
                '2026-06-05', ?)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .bind(bank_account_id)
    .bind(payment_id)
    .execute(pool)
    .await
    .map(|_| ())
}

async fn seed_invoice_reminder(
    pool: &SqlitePool,
    company_id: &str,
    invoice_id: &str,
    client_id: &str,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    if has_records(pool, "invoice_reminders", company_id).await? {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO invoice_reminders (
            id, company_id, invoice_id, client_id, reminder_type,
            scheduled_date, status, subject, message, channel, created_by
        )
        VALUES (?, ?, ?, ?, 'before_due', '2026-06-12', 'pending',
                '语数科技发票付款提醒',
                '星河制造平台一期预付款将于 2026-06-16 到期。',
                'email', ?)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .bind(invoice_id)
    .bind(client_id)
    .bind(user_id)
    .execute(pool)
    .await
    .map(|_| ())
}

async fn ensure_asset(
    pool: &SqlitePool,
    company_id: &str,
    category_id: &str,
    invoice_id: &str,
    contract_id: &str,
    user_id: &str,
) -> Result<String, sqlx::Error> {
    if let Some(id) = first_id(pool, "assets", company_id).await? {
        return Ok(id);
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO assets (
            id, company_id, name, asset_type, status, description,
            serial_number, location, purchase_date, purchase_price,
            current_value, category_id, useful_life, salvage_rate,
            depreciation_method, monthly_depreciation, accumulated_depreciation,
            invoice_id, contract_id, registration_number, holder_name, created_by
        )
        VALUES (?, ?, 'A100 推理服务器', 'physical', 'active',
                '用于文档解析、政策匹配和合同风险分析的 GPU 推理服务器。',
                'YS-GPU-2026-001', '深圳总部机房', '2026-03-18',
                280000, 252000, ?, 36, 0.05, 'straight_line',
                7388.89, 28000, ?, ?, 'YS-ASSET-2026-001',
                '虚构示例科技有限公司', ?)
        "#,
    )
    .bind(&id)
    .bind(company_id)
    .bind(category_id)
    .bind(invoice_id)
    .bind(contract_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(id)
}

async fn ensure_data_asset(
    pool: &SqlitePool,
    company_id: &str,
    category_id: &str,
    user_id: &str,
) -> Result<String, sqlx::Error> {
    let existing = sqlx::query_scalar::<_, String>(
        "SELECT id FROM assets WHERE company_id = ? AND asset_type = 'data' LIMIT 1",
    )
    .bind(company_id)
    .fetch_optional(pool)
    .await?;
    if let Some(id) = existing {
        return Ok(id);
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO assets (
            id, company_id, name, asset_type, status, description,
            location, purchase_date, purchase_price, current_value,
            category_id, registration_number, holder_name, created_by
        )
        VALUES (?, ?, '合同与政策语料数据集', 'data', 'active',
                '用于合同条款抽取、政策匹配和风险标签训练的内部标注数据集。',
                'DaaS / Demo Workspace', '2026-02-01', 160000, 180000,
                ?, 'YS-DATA-2026-001', '虚构示例科技有限公司', ?)
        "#,
    )
    .bind(&id)
    .bind(company_id)
    .bind(category_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(id)
}

async fn seed_asset_record(
    pool: &SqlitePool,
    company_id: &str,
    asset_id: &str,
) -> Result<(), sqlx::Error> {
    if has_records(pool, "asset_records", company_id).await? {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO asset_records (id, company_id, asset_id, record_type, description, amount, record_date)
        VALUES (?, ?, ?, 'depreciation', '2026 年 5 月服务器折旧', 7388.89, '2026-05-31')
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .bind(asset_id)
    .execute(pool)
    .await
    .map(|_| ())
}

async fn seed_data_asset_meta(
    pool: &SqlitePool,
    company_id: &str,
    asset_id: &str,
) -> Result<(), sqlx::Error> {
    if has_records(pool, "data_asset_meta", company_id).await? {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO data_asset_meta (
            id, company_id, asset_id, data_type, tech_stack, version,
            license_type, data_volume, data_format, access_url
        )
        VALUES (?, ?, ?, 'dataset', 'liteparse,paddleocr,deepseek',
                '2026.05', '自研', '12.8GB / 38,000 文档片段',
                'jsonl,pdf,docx', 'daas://demo-yushu/contracts-policy-corpus')
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .bind(asset_id)
    .execute(pool)
    .await
    .map(|_| ())
}

async fn seed_salary(
    pool: &SqlitePool,
    company_id: &str,
    employee_id: &str,
) -> Result<(), sqlx::Error> {
    if has_records(pool, "salary_records", company_id).await? {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO salary_records (
            id, company_id, employee_id, employee_name, department, position,
            period, base_salary, bonus, deductions, social_security,
            social_security_company, housing_fund, housing_fund_company,
            net_salary, status, payment_date, notes
        )
        VALUES (?, ?, ?, '周知远', '研发部', 'AI 产品负责人',
                '2026-05', 42000, 8000, 1200, 2400, 4800, 2100,
                2100, 44300, 'paid', '2026-05-31',
                'Demo 工资记录用于薪酬、社保和研发费用联动。')
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .bind(employee_id)
    .execute(pool)
    .await
    .map(|_| ())
}

async fn seed_budget(
    pool: &SqlitePool,
    company_id: &str,
    project_id: &str,
) -> Result<(), sqlx::Error> {
    if has_records(pool, "budgets", company_id).await? {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO budgets (
            id, company_id, category, budget_type, period,
            budget_amount, actual_amount, status, project_id, notes
        )
        VALUES (?, ?, '研发与交付成本', 'project', '2026-Q2',
                600000, 430000, 'active', ?,
                '覆盖人力、云资源、设备折旧和外包材料。')
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .bind(project_id)
    .execute(pool)
    .await
    .map(|_| ())
}

async fn seed_rent(
    pool: &SqlitePool,
    company_id: &str,
    contract_id: &str,
) -> Result<(), sqlx::Error> {
    if has_records(pool, "rent_records", company_id).await? {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO rent_records (
            id, company_id, location_name, property_type, address, area,
            monthly_rent, deposit, contract_start_date, contract_end_date,
            next_payment_date, payment_cycle, landlord, landlord_contact,
            contract_id, status, notes
        )
        VALUES (?, ?, '深圳总部办公室', 'office',
                '深圳市南山区粤海街道科苑南路 2666 号', 680,
                96000, 192000, '2025-01-01', '2027-12-31',
                '2026-06-01', 'monthly', '深圳湾科技园运营有限公司',
                '+86-755-2600-0000', ?, 'active',
                'Demo 场地租赁用于费用、合同与提醒联动。')
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .bind(contract_id)
    .execute(pool)
    .await
    .map(|_| ())
}

async fn seed_license(pool: &SqlitePool, company_id: &str) -> Result<(), sqlx::Error> {
    if has_records(pool, "licenses", company_id).await? {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO licenses (
            id, company_id, license_name, license_type, license_number,
            issuing_authority, issue_date, expiry_date, status, notes
        )
        VALUES (?, ?, '增值电信业务经营许可证', 'telecom',
                '粤B2-DEMO-2026', '广东省通信管理局', '2026-01-15',
                '2031-01-14', 'valid', 'Demo 证照用于到期提醒和合规看板。')
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .execute(pool)
    .await
    .map(|_| ())
}

async fn seed_time_entry(
    pool: &SqlitePool,
    company_id: &str,
    project_id: &str,
    client_id: &str,
    employee_id: &str,
    invoice_id: &str,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    if has_records(pool, "time_entries", company_id).await? {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO time_entries (
            id, company_id, project_id, client_id, employee_id,
            description, start_time, end_time, duration_minutes,
            date, is_billable, hourly_rate, amount, invoice_id,
            status, tags, created_by
        )
        VALUES (?, ?, ?, ?, ?, '合同条款抽取与政策匹配规则配置',
                '2026-05-21T09:30:00Z', '2026-05-21T16:30:00Z',
                360, '2026-05-21', 1, 900, 5400, ?,
                'invoiced', '["ai","contract","policy"]', ?)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .bind(project_id)
    .bind(client_id)
    .bind(employee_id)
    .bind(invoice_id)
    .bind(user_id)
    .execute(pool)
    .await
    .map(|_| ())
}

async fn seed_milestone(
    pool: &SqlitePool,
    company_id: &str,
    project_id: &str,
) -> Result<(), sqlx::Error> {
    if has_records(pool, "milestones", company_id).await? {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO milestones (
            id, company_id, title, description, milestone_type,
            importance, milestone_date, project_id, notes
        )
        VALUES (?, ?, '合同与政策智能工作流上线',
                '上线合同分析、合同生成、政策匹配和材料复用链路。',
                'delivery', 'high', '2026-06-20', ?, 'Demo 里程碑')
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .bind(project_id)
    .execute(pool)
    .await
    .map(|_| ())
}

async fn seed_application_assets(
    pool: &SqlitePool,
    company_id: &str,
    project_id: &str,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    let application_id = if let Some(id) = first_id(pool, "applications", company_id).await? {
        id
    } else {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO applications (
                id, company_id, application_no, title, type, category,
                deadline, status, progress, notes, project_id
            )
            VALUES (?, ?, 'APP-YS-2026-001', '深圳市科技型中小企业研发补贴',
                    'subsidy', 'rd', '2026-07-31', 'in_progress',
                    65, 'Demo 政策申报用于政策分析、材料库和项目联动。', ?)
            "#,
        )
        .bind(&id)
        .bind(company_id)
        .bind(project_id)
        .execute(pool)
        .await?;
        id
    };

    let material_id = if let Some(id) = first_id(pool, "material_library", company_id).await? {
        id
    } else {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO material_library (
                id, company_id, name, category, upload_date, file_size,
                file_path, used_count, source_type, source_id
            )
            VALUES (?, ?, '语数科技商业计划书.pdf', 'business_plan',
                    '2026-05-10', '1.8 MB',
                    '/demo/materials/yushu-business-plan.pdf', 3,
                    'application', ?)
            "#,
        )
        .bind(&id)
        .bind(company_id)
        .bind(&application_id)
        .execute(pool)
        .await?;
        id
    };

    if !has_records(pool, "material_templates", company_id).await? {
        sqlx::query(
            r#"
            INSERT INTO material_templates (
                id, company_id, template_type, template_name,
                template_category, material_name, required, description,
                display_order
            )
            VALUES (?, ?, 'subsidy_rd', '研发补贴申报材料清单',
                    'policy_application', '商业计划书与研发费用说明', 1,
                    '用于政策申报教程的材料模板。', 1)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(company_id)
        .execute(pool)
        .await?;
    }

    if !has_records(pool, "application_materials", company_id).await? {
        sqlx::query(
            r#"
            INSERT INTO application_materials (
                id, company_id, application_id, name, required, status,
                description, upload_date, file_size, file_path,
                can_reuse, reuse_source
            )
            VALUES (?, ?, ?, '商业计划书', 1, 'completed',
                    '展示材料复用与版本管理。', '2026-05-10',
                    '1.8 MB', '/demo/materials/yushu-business-plan.pdf',
                    1, ?)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(company_id)
        .bind(&application_id)
        .bind(&material_id)
        .execute(pool)
        .await?;
    }

    if !has_records(pool, "material_versions", company_id).await? {
        sqlx::query(
            r#"
            INSERT INTO material_versions (
                id, company_id, material_id, version_number, file_path,
                file_name, file_size, description, change_notes,
                created_by, is_current
            )
            VALUES (?, ?, ?, 1, '/demo/materials/yushu-business-plan-v1.pdf',
                    '语数科技商业计划书-v1.pdf', '1.8 MB',
                    'Demo 初始版本', '首次上传', ?, 1)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(company_id)
        .bind(&material_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_tax_assets(
    pool: &SqlitePool,
    company_id: &str,
    invoice_id: &str,
    payment_id: &str,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    if !has_records(pool, "tax_records", company_id).await? {
        sqlx::query(
            r#"
            INSERT INTO tax_records (
                id, company_id, tax_type, period, declaration_type,
                declaration_period, period_start, period_end, taxable_amount,
                tax_amount, declared_amount, status, due_date,
                declaration_deadline, invoice_id, payment_id, notes
            )
            VALUES (?, ?, 'vat', '2026-05', 'monthly', '2026-05',
                    '2026-05-01', '2026-05-31', 204000, 12240,
                    12240, 'declared', '2026-06-15', '2026-06-15',
                    ?, ?, 'Demo 增值税记录用于税务管理和发票联动。')
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(company_id)
        .bind(invoice_id)
        .bind(payment_id)
        .execute(pool)
        .await?;
    }

    let jurisdiction_id = if let Some(id) = first_id(pool, "tax_jurisdictions", company_id).await? {
        id
    } else {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO tax_jurisdictions (
                id, company_id, code, name, name_local, country_code,
                region_code, jurisdiction_type, currency, fiscal_year_start,
                corporate_tax_rate, personal_tax_rate, vat_rate,
                has_state_income_tax, notes, is_active, created_by
            )
            VALUES (?, ?, 'CN-GD-SZ', 'Shenzhen, Guangdong', '广东省深圳市',
                    'CN', 'GD-SZ', 'city', 'CNY', '01-01',
                    0.25, 0.45, 0.06, 1,
                    'Demo 中国税务管辖区。', 1, ?)
            "#,
        )
        .bind(&id)
        .bind(company_id)
        .bind(user_id)
        .execute(pool)
        .await?;
        id
    };

    if !has_records(pool, "tax_deadlines", company_id).await? {
        sqlx::query(
            r#"
            INSERT INTO tax_deadlines (
                id, company_id, jurisdiction_id, jurisdiction_code,
                tax_type, title, title_local, description, deadline_month,
                deadline_day, recurrence, quarter, is_estimated_payment,
                applies_to, penalty_info, filing_url, is_system,
                is_active, notes, created_by
            )
            VALUES (?, ?, ?, 'CN-GD-SZ', 'vat', 'Monthly VAT Filing',
                    '增值税月度申报', '每月申报上月增值税。',
                    6, 15, 'monthly', NULL, 0, 'corporation',
                    '逾期可能产生滞纳金。', 'https://etax.guangdong.chinatax.gov.cn',
                    0, 1, 'Demo 税务日历事件。', ?)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(company_id)
        .bind(&jurisdiction_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_cloud_assets(
    pool: &SqlitePool,
    company_id: &str,
    project_id: &str,
) -> Result<(), sqlx::Error> {
    let provider_id = if let Some(id) = first_id(pool, "cloud_providers", company_id).await? {
        id
    } else {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO cloud_providers (
                id, company_id, provider_name, provider_type, account_name,
                account_id, region, contact_person, contact_email, notes
            )
            VALUES (?, ?, '阿里云', 'public_cloud', '语数科技生产账号',
                    'aliyun-demo-2026', 'cn-shenzhen', '云成本支持',
                    'cloud@yushu-tech.example', 'Demo 云服务商账号。')
            "#,
        )
        .bind(&id)
        .bind(company_id)
        .execute(pool)
        .await?;
        id
    };

    if !has_records(pool, "cloud_resources", company_id).await? {
        sqlx::query(
            r#"
            INSERT INTO cloud_resources (
                id, company_id, provider_id, resource_type, resource_name,
                instance_id, specification, region, purchase_type,
                start_date, monthly_cost, status, notes, project_id
            )
            VALUES (?, ?, ?, 'gpu_instance', '合同分析推理节点',
                    'ecs-demo-gpu-001', 'ecs.gn7i-c16g1.4xlarge',
                    'cn-shenzhen', 'subscription', '2026-04-01',
                    48600, 'running', 'Demo 云资源用于云成本治理。', ?)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(company_id)
        .bind(&provider_id)
        .bind(project_id)
        .execute(pool)
        .await?;
    }

    if !has_records(pool, "cloud_bills", company_id).await? {
        sqlx::query(
            r#"
            INSERT INTO cloud_bills (
                id, company_id, provider_id, billing_period,
                total_amount, currency, status, notes
            )
            VALUES (?, ?, ?, '2026-05', 48600, 'CNY',
                    'confirmed', 'Demo 云账单覆盖成本趋势和项目分摊。')
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(company_id)
        .bind(&provider_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_subscription_assets(
    pool: &SqlitePool,
    company_id: &str,
    payment_id: &str,
    contract_id: &str,
) -> Result<(), sqlx::Error> {
    if !has_records(pool, "software_subscriptions", company_id).await? {
        sqlx::query(
            r#"
            INSERT INTO software_subscriptions (
                id, company_id, software_name, vendor, subscription_type,
                plan_name, license_count, total_price, billing_cycle,
                start_date, next_billing_date, auto_renewal, status,
                usage_department, account_email, notes, payment_id, contract_id
            )
            VALUES (?, ?, 'DeepSeek API 企业调用包', 'DeepSeek',
                    'api_usage', 'flash-pro-mixed', 10, 12000,
                    'monthly', '2026-05-01', '2026-06-01',
                    1, 'active', '研发部', 'aiops@yushu-tech.example',
                    'Demo 软件订阅不包含真实密钥。', ?, ?)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(company_id)
        .bind(payment_id)
        .bind(contract_id)
        .execute(pool)
        .await?;
    }

    if !has_records(pool, "domain_subscriptions", company_id).await? {
        sqlx::query(
            r#"
            INSERT INTO domain_subscriptions (
                id, company_id, domain_name, registrar, registration_date,
                expiry_date, initial_purchase_cost, annual_fee, auto_renewal,
                dns_provider, status, contact_email, notes, payment_id
            )
            VALUES (?, ?, 'yushu-tech.example', 'Demo Registrar',
                    '2024-06-01', '2027-06-01', 600, 120,
                    1, 'Cloudflare', 'active', 'ops@yushu-tech.example',
                    '保留 example 域名，避免真实外发风险。', ?)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(company_id)
        .bind(payment_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_compliance_assets(
    pool: &SqlitePool,
    company_id: &str,
    employee_id: &str,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    if !has_records(pool, "insurance_policies", company_id).await? {
        sqlx::query(
            r#"
            INSERT INTO insurance_policies (
                id, company_id, policy_name, policy_number, insurance_type,
                provider, premium_amount, premium_frequency, coverage_amount,
                deductible, start_date, end_date, renewal_date, auto_renew,
                status, beneficiary, country, notes, created_by
            )
            VALUES (?, ?, '网络安全责任险', 'INS-DEMO-2026-001',
                    'cyber', 'Demo Insurance', 36000, 'yearly',
                    5000000, 50000, '2026-01-01', '2026-12-31',
                    '2026-11-30', 1, 'active', '虚构示例科技有限公司',
                    'CN', 'Demo 保单用于合规和续保提醒。', ?)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(company_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    }

    if !has_records(pool, "pension_plans", company_id).await? {
        sqlx::query(
            r#"
            INSERT INTO pension_plans (
                id, company_id, plan_name, plan_type, country, provider,
                employee_id, contribution_employer, contribution_employee,
                contribution_frequency, start_date, current_value,
                status, notes, created_by
            )
            VALUES (?, ?, '深圳企业职工基本养老保险', 'national_pension',
                    'CN', '深圳社保局', ?, 4800, 2400, 'monthly',
                    '2024-02-18', 118000, 'active',
                    'Demo 年金/社保计划用于薪酬合规联动。', ?)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(company_id)
        .bind(employee_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_vendor(pool: &SqlitePool, company_id: &str) -> Result<(), sqlx::Error> {
    if has_records(pool, "vendors", company_id).await? {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO vendors (
            id, company_id, name, short_name, unified_code, legal_person,
            registered_capital, establishment_date, business_scope, address,
            contact_person, contact_phone, contact_email, bank_name,
            bank_account, vendor_type, status, notes
        )
        VALUES (?, ?, '深圳云启算力服务有限公司', '云启算力',
                '91440300DEMOVENDOR', '王云启', '2000万元',
                '2020-03-12', '云计算基础设施与 GPU 租赁',
                '深圳市南山区科技园', '王云启', '+86-755-2233-2024',
                'sales@cloud-demo.example', '招商银行深圳分行',
                '75590000008888', 'supplier', 'active',
                'Demo 供应商用于采购、合同和云成本场景。')
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .execute(pool)
    .await
    .map(|_| ())
}

async fn seed_rd_cost(
    pool: &SqlitePool,
    company_id: &str,
    project_id: &str,
    employee_id: &str,
    asset_id: &str,
    invoice_id: &str,
) -> Result<(), sqlx::Error> {
    if has_records(pool, "rd_costs", company_id).await? {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO rd_costs (
            id, company_id, project_id, cost_type, amount,
            description, cost_date, employee_id, asset_id, invoice_id
        )
        VALUES (?, ?, ?, 'personnel', 42000,
                'AI 产品负责人工资计入研发费用', '2026-05-31',
                ?, ?, ?)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .bind(project_id)
    .bind(employee_id)
    .bind(asset_id)
    .bind(invoice_id)
    .execute(pool)
    .await
    .map(|_| ())
}

async fn seed_kpi(pool: &SqlitePool, company_id: &str) -> Result<(), sqlx::Error> {
    if !has_records(pool, "kpi_metrics", company_id).await? {
        sqlx::query(
            r#"
            INSERT INTO kpi_metrics (
                id, company_id, name, category, value, target,
                unit, period, description
            )
            VALUES (?, ?, '月度经常性收入', 'finance',
                    386000, 500000, 'CNY', '2026-05',
                    'Demo KPI 用于经营看板。')
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(company_id)
        .execute(pool)
        .await?;
    }

    let config_id = if let Some(id) = first_id(pool, "kpi_configs", company_id).await? {
        id
    } else {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO kpi_configs (
                id, company_id, name, category, description, unit,
                target_value, warning_threshold, danger_threshold, is_active
            )
            VALUES (?, ?, 'MRR', 'finance', '月度经常性收入',
                    'CNY', 500000, 350000, 250000, 1)
            "#,
        )
        .bind(&id)
        .bind(company_id)
        .execute(pool)
        .await?;
        id
    };

    if !has_records(pool, "kpi_records", company_id).await? {
        sqlx::query(
            r#"
            INSERT INTO kpi_records (id, company_id, kpi_id, value, period, notes)
            VALUES (?, ?, ?, 386000, '2026-05', 'Demo KPI 记录')
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(company_id)
        .bind(config_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_exchange_rate(pool: &SqlitePool, company_id: &str) -> Result<(), sqlx::Error> {
    if has_records(pool, "exchange_rates", company_id).await? {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO exchange_rates (
            id, company_id, from_currency, to_currency,
            rate, effective_date, source
        )
        VALUES (?, ?, 'USD', 'CNY', 7.24, '2026-05-01', 'manual-demo')
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .execute(pool)
    .await
    .map(|_| ())
}

async fn seed_reminder(
    pool: &SqlitePool,
    company_id: &str,
    contract_id: &str,
) -> Result<(), sqlx::Error> {
    if has_records(pool, "reminders", company_id).await? {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO reminders (
            id, company_id, title, description, reminder_type,
            priority, due_date, remind_date, status, is_completed,
            entity_type, entity_id
        )
        VALUES (?, ?, '客户服务合同续约准备',
                '提前准备星河制造年度服务续约材料。',
                'contract', 'high', '2027-04-30', '2027-04-01',
                'pending', 0, 'contract', ?)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .bind(contract_id)
    .execute(pool)
    .await
    .map(|_| ())
}

async fn seed_inactive_webhook(
    pool: &SqlitePool,
    company_id: &str,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    if has_records(pool, "webhooks", company_id).await? {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO webhooks (
            id, company_id, url, events, is_active,
            description, failure_count, created_by
        )
        VALUES (?, ?, 'https://webhook.site/demo-yushu-disabled',
                'invoice.created,payment.received,contract.expiring',
                0, 'Demo Webhook 默认停用，避免真实外发。', 0, ?)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .bind(user_id)
    .execute(pool)
    .await
    .map(|_| ())
}

async fn seed_demo_error_log(
    pool: &SqlitePool,
    company_id: &str,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    if has_records(pool, "error_logs", company_id).await? {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO error_logs (
            id, company_id, created_by, error_type, error_message,
            page_url, user_agent, metadata
        )
        VALUES (?, ?, ?, 'demo_notice',
                'Demo 数据：这里展示前端错误监控记录样例。',
                '/dashboard', 'OPC Demo Browser',
                '{"demo":true,"severity":"info"}')
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(company_id)
    .bind(user_id)
    .execute(pool)
    .await
    .map(|_| ())
}
