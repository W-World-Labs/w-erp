//! Canonical onboarding pipeline — the single source of truth for the new-user
//! value-realization journey.
//!
//! Both the HTTP handler (`api::generic::onboarding_progress`) and the AI agent
//! tool (`api::agent`) render progress from here, so the pipeline — its steps,
//! copy, tiers and completion rules — is defined in exactly one place. The
//! frontend (ProgressTracker / OnboardingTutorial) only renders what this module
//! emits; it must not recompute completion locally (see
//! `frontend/src/lib/backend-owned-onboarding.test.ts`).
//!
//! Tier 1 (激活) is the value path a solo founder walks to earn the first dollar:
//! 建立公司 → 加客户 → 报价 → 开票 → 记工时 → 收款. Tier 2 (运营) is the
//! back-office ledger that can be filled in over time.

use serde_json::{Value, json};
use sqlx::{Row, SqlitePool};

/// One step/module in the onboarding pipeline.
pub struct OnboardingModuleSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub icon_key: &'static str,
    pub route: &'static str,
    pub description: &'static str,
    pub tutorial_title: &'static str,
    pub tutorial_description: &'static str,
    pub action_label: &'static str,
    pub count_sql: &'static str,
    pub show_in_tutorial: bool,
    /// 1 = 激活 (value path), 2 = 运营 (back-office ledger)
    pub tier: u8,
}

const TIER1_TITLE: &str = "激活 · 跑通获客到收款";
const TIER2_TITLE: &str = "运营 · 把后台补齐";

/// The ordered onboarding pipeline. Tier 1 first (value path), Tier 2 after.
pub fn module_specs() -> Vec<OnboardingModuleSpec> {
    vec![
        // ---------- Tier 1: 价值主路径 ----------
        OnboardingModuleSpec {
            id: "company",
            name: "公司信息",
            icon_key: "Building2",
            route: "/company-info",
            description: "公司基本信息和注册资料",
            tutorial_title: "建立你的公司",
            tutorial_description: "填写公司名称、注册资料和法人信息。这是系统的根，后续报价、发票、合同都会用到。",
            action_label: "完善公司信息",
            count_sql: "SELECT COUNT(*) FROM company_info WHERE company_id = ?",
            show_in_tutorial: true,
            tier: 1,
        },
        OnboardingModuleSpec {
            id: "clients",
            name: "客户",
            icon_key: "Users",
            route: "/crm",
            description: "客户与联系人",
            tutorial_title: "添加第一个客户",
            tutorial_description: "录入你的第一个客户。有了客户，才能给他报价、开发票、跟踪回款。",
            action_label: "添加客户",
            count_sql: "SELECT COUNT(*) FROM clients WHERE company_id = ?",
            show_in_tutorial: true,
            tier: 1,
        },
        OnboardingModuleSpec {
            id: "quotes",
            name: "报价单",
            icon_key: "FileText",
            route: "/quotes",
            description: "报价单与转化",
            tutorial_title: "给客户出一份报价",
            tutorial_description: "为客户创建报价单。客户确认后可一键转成发票，省去重复录入。",
            action_label: "创建报价",
            count_sql: "SELECT COUNT(*) FROM quotes WHERE company_id = ?",
            show_in_tutorial: true,
            tier: 1,
        },
        OnboardingModuleSpec {
            id: "invoices",
            name: "发票",
            icon_key: "FileSpreadsheet",
            route: "/invoices",
            description: "收支发票",
            tutorial_title: "开出第一张发票",
            tutorial_description: "把报价转成发票，或直接开票。发票是收款和应收账款跟踪的起点。",
            action_label: "开具发票",
            // Connectivity: only receivable (AR) invoices count toward the value path.
            // Include legacy aliases until all writers have been unified to `receivable`.
            count_sql: "SELECT COUNT(*) FROM invoices WHERE company_id = ? AND LOWER(COALESCE(invoice_type, '')) IN ('receivable', 'output', 'income', '销项')",
            show_in_tutorial: true,
            tier: 1,
        },
        OnboardingModuleSpec {
            id: "time",
            name: "工时",
            icon_key: "Clock",
            route: "/time-tracking",
            description: "可计费工时",
            tutorial_title: "记录可计费工时",
            tutorial_description: "按项目或客户记录工时。工时可按时薪汇总，一键生成发票。",
            action_label: "记录工时",
            count_sql: "SELECT COUNT(*) FROM time_entries WHERE company_id = ?",
            show_in_tutorial: true,
            tier: 1,
        },
        OnboardingModuleSpec {
            id: "payments",
            name: "收款",
            icon_key: "Wallet",
            // Guide users to record payment from an invoice (ensures invoice_id link).
            route: "/invoices",
            description: "收款与应收",
            tutorial_title: "记录第一笔收款",
            tutorial_description: "在发票上记录客户回款（收款需关联到发票），系统据此跟踪应收和现金流。至此你已跑通获客到收款的闭环。",
            action_label: "记录收款",
            // Connectivity: only payments linked to an invoice complete the loop.
            count_sql: "SELECT COUNT(*) FROM payments p WHERE p.company_id = ? AND p.status IN ('completed', 'paid') AND p.amount > 0 AND p.payment_type IN ('income', 'receipt', '收入') AND EXISTS (SELECT 1 FROM invoices i WHERE i.id = p.invoice_id AND i.company_id = p.company_id AND LOWER(COALESCE(i.invoice_type, '')) IN ('receivable', 'output', 'income', '销项'))",
            show_in_tutorial: true,
            tier: 1,
        },
        // ---------- Tier 2: 运营台账 ----------
        OnboardingModuleSpec {
            id: "employees",
            name: "员工管理",
            icon_key: "Users",
            route: "/employees",
            description: "管理员工信息与文档",
            tutorial_title: "录入员工信息",
            tutorial_description: "添加员工基本信息、合同、薪资等，支持上传身份证、学历证书等文档。",
            action_label: "添加员工",
            count_sql: "SELECT COUNT(*) FROM employees WHERE company_id = ?",
            show_in_tutorial: true,
            tier: 2,
        },
        OnboardingModuleSpec {
            id: "salaries",
            name: "工资管理",
            icon_key: "DollarSign",
            route: "/salary",
            description: "管理员工薪资发放",
            tutorial_title: "录入工资信息",
            tutorial_description: "记录员工工资发放，包括基本工资、奖金、扣款等，可上传工资条附件。",
            action_label: "添加工资",
            count_sql: "SELECT COUNT(*) FROM salary_records WHERE company_id = ?",
            show_in_tutorial: true,
            tier: 2,
        },
        OnboardingModuleSpec {
            id: "projects",
            name: "项目管理",
            icon_key: "FolderKanban",
            route: "/projects",
            description: "管理项目进度与文档",
            tutorial_title: "录入项目信息",
            tutorial_description: "管理项目进度、预算和团队成员，可上传项目文档和合同；工时可挂到项目上。",
            action_label: "添加项目",
            count_sql: "SELECT COUNT(*) FROM projects WHERE company_id = ?",
            show_in_tutorial: true,
            tier: 2,
        },
        OnboardingModuleSpec {
            id: "assets",
            name: "资产管理",
            icon_key: "Package",
            route: "/assets",
            description: "管理公司固定资产",
            tutorial_title: "录入资产信息",
            tutorial_description: "添加电脑、设备、办公家具等固定资产，便于资产管理和折旧计算。",
            action_label: "添加资产",
            count_sql: "SELECT COUNT(*) FROM assets WHERE company_id = ?",
            show_in_tutorial: true,
            tier: 2,
        },
        OnboardingModuleSpec {
            id: "contracts",
            name: "合同管理",
            icon_key: "FileSpreadsheet",
            route: "/contracts",
            description: "管理合同文件",
            tutorial_title: "归档合同",
            tutorial_description: "归档合同文件并维护关键期限，便于后续审计和履约提醒。",
            action_label: "添加合同",
            count_sql: "SELECT COUNT(*) FROM contracts WHERE company_id = ?",
            show_in_tutorial: true,
            tier: 2,
        },
        OnboardingModuleSpec {
            id: "licenses",
            name: "证照管理",
            icon_key: "FileText",
            route: "/licenses",
            description: "管理证照有效期",
            tutorial_title: "录入证照信息",
            tutorial_description: "添加营业执照、许可证等重要证照，系统会在到期前自动提醒。",
            action_label: "添加证照",
            count_sql: "SELECT COUNT(*) FROM licenses WHERE company_id = ?",
            show_in_tutorial: true,
            tier: 2,
        },
        OnboardingModuleSpec {
            id: "rents",
            name: "场地租赁",
            icon_key: "Home",
            route: "/rent",
            description: "管理场地租赁信息",
            tutorial_title: "录入场地租赁",
            tutorial_description: "添加办公场地、仓库等租赁信息，包括租金、合同期限等，可上传合同附件。",
            action_label: "添加场地租赁",
            count_sql: "SELECT COUNT(*) FROM rent_records WHERE company_id = ?",
            show_in_tutorial: true,
            tier: 2,
        },
        OnboardingModuleSpec {
            id: "budgets",
            name: "预算管理",
            icon_key: "TrendingUp",
            route: "/budget-management",
            description: "制定与监控预算",
            tutorial_title: "设置预算",
            tutorial_description: "制定各类预算计划，实时监控预算执行情况，支持上传预算相关附件。",
            action_label: "添加预算",
            count_sql: "SELECT COUNT(*) FROM budgets WHERE company_id = ?",
            show_in_tutorial: true,
            tier: 2,
        },
        OnboardingModuleSpec {
            id: "branches",
            name: "分支机构",
            icon_key: "Building2",
            route: "/branches",
            description: "管理多个分支机构",
            tutorial_title: "设置分支机构",
            tutorial_description: "如有子公司、分公司等，添加各分支的基本信息和负责人。",
            action_label: "添加分支机构",
            count_sql: "SELECT COUNT(*) FROM branches WHERE company_id = ?",
            show_in_tutorial: true,
            tier: 2,
        },
    ]
}

async fn count_rows(pool: &SqlitePool, company_id: &str, sql: &str) -> Result<i64, sqlx::Error> {
    let (count,): (i64,) = sqlx::query_as(sql).bind(company_id).fetch_one(pool).await?;
    Ok(count)
}

async fn is_demo_company(pool: &SqlitePool, company_id: &str) -> Result<bool, sqlx::Error> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM companies WHERE id = ? AND is_demo = 1 AND COALESCE(is_deleted, 0) = 0",
    )
    .bind(company_id)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

fn status_badge(completed: bool) -> Value {
    if completed {
        json!({
            "label": "已完成",
            "variant": "default",
            "class_name": "bg-green-600 text-white hover:bg-green-600"
        })
    } else {
        json!({
            "label": "待完善",
            "variant": "secondary",
            "class_name": "bg-muted text-muted-foreground"
        })
    }
}

fn tier_title(tier: u8) -> &'static str {
    match tier {
        1 => TIER1_TITLE,
        _ => TIER2_TITLE,
    }
}

fn pct(done: i64, total: i64) -> i64 {
    if total == 0 {
        0
    } else {
        ((done as f64 / total as f64) * 100.0).round() as i64
    }
}

struct GoldenInvoice {
    invoice_number: String,
    amount: f64,
    status: String,
    paid_amount: f64,
}

fn activation_step(
    id: &str,
    label: &str,
    route: &str,
    completed: bool,
    evidence_label: String,
) -> Value {
    json!({
        "id": id,
        "label": label,
        "route": route,
        "completed": completed,
        "evidence_label": evidence_label,
        "status_badge": status_badge(completed),
    })
}

fn activation_next_action(current_step_id: &str) -> Value {
    match current_step_id {
        "company" => json!({
            "route": "/company-info",
            "label": "完善公司信息",
            "description": "先填写公司主体信息，后续报价和发票会复用这些资料。",
            "action": null
        }),
        "client" => json!({
            "route": "/crm",
            "label": "添加第一个客户",
            "description": "录入客户后，才能把报价、发票和收款串成一条业务线。",
            "action": null
        }),
        "quote" => json!({
            "route": "/quotes",
            "label": "创建报价",
            "description": "给客户创建第一份报价，作为后续转发票的起点。",
            "action": null
        }),
        "invoice" => json!({
            "route": "/quotes",
            "label": "将报价转为发票",
            "description": "使用报价行里的转发票动作，生成与报价相连的应收发票。",
            "action": null
        }),
        "invoice_pdf" => json!({
            "route": "/invoices",
            "label": "下载发票 PDF",
            "description": "确认发票可交付给客户，下载第一张发票 PDF。",
            "action": null
        }),
        "payment" => json!({
            "route": "/invoices",
            "label": "记录收款",
            "description": "在发票行使用记录收款动作，生成与发票相连的收入流水。",
            "action": null
        }),
        "ar_clear" => json!({
            "route": "/payments",
            "label": "确认应收清零",
            "description": "收款覆盖发票金额后，发票应结清并退出应收账龄。",
            "action": null
        }),
        _ => json!({
            "route": "/payments",
            "label": "查看收款闭环",
            "description": "第一笔报价到收款闭环已完成，可以继续查看收款和应收状态。",
            "action": null
        }),
    }
}

fn money_evidence(amount: f64) -> String {
    if amount.fract().abs() < 0.001 {
        format!("¥{:.0}", amount)
    } else {
        format!("¥{:.2}", amount)
    }
}

async fn quote_linked_invoice(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<Option<GoldenInvoice>, sqlx::Error> {
    let quote_row = sqlx::query(
        r#"
        SELECT quote_number, invoice_id
          FROM quotes
         WHERE company_id = ?
         ORDER BY
           CASE
             WHEN invoice_id IS NOT NULL AND invoice_id != '' THEN 1
             ELSE 0
           END DESC,
           created_at ASC,
           id ASC
         LIMIT 1
        "#,
    )
    .bind(company_id)
    .fetch_optional(pool)
    .await?;

    let Some(quote_row) = quote_row else {
        return Ok(None);
    };
    let invoice_id = quote_row
        .try_get::<Option<String>, _>("invoice_id")?
        .unwrap_or_default();
    if invoice_id.trim().is_empty() {
        return Ok(None);
    }

    let invoice_row = sqlx::query(
        r#"
        SELECT invoice_number, COALESCE(amount, 0) AS amount, COALESCE(status, '') AS status
          FROM invoices
         WHERE company_id = ?
           AND id = ?
           AND COALESCE(invoice_type, 'receivable') NOT IN ('input', 'payable')
         LIMIT 1
        "#,
    )
    .bind(company_id)
    .bind(&invoice_id)
    .fetch_optional(pool)
    .await?;

    let Some(invoice_row) = invoice_row else {
        return Ok(None);
    };
    let paid_amount = sqlx::query_scalar::<_, f64>(
        r#"
        SELECT CAST(COALESCE(SUM(amount), 0) AS REAL)
          FROM payments
         WHERE company_id = ?
           AND invoice_id = ?
           AND payment_type IN ('income', 'receipt', '收入')
           AND status IN ('completed', 'paid')
        "#,
    )
    .bind(company_id)
    .bind(&invoice_id)
    .fetch_one(pool)
    .await?;

    Ok(Some(GoldenInvoice {
        invoice_number: invoice_row.try_get("invoice_number")?,
        amount: invoice_row.try_get("amount")?,
        status: invoice_row.try_get("status")?,
        paid_amount,
    }))
}

pub async fn build_activation_loop(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<Value, sqlx::Error> {
    let company_info_done = count_rows(
        pool,
        company_id,
        "SELECT COUNT(*) FROM company_info WHERE company_id = ?",
    )
    .await?
        > 0;

    let client_name = sqlx::query_scalar::<_, String>(
        "SELECT name FROM clients WHERE company_id = ? ORDER BY created_at ASC, id ASC LIMIT 1",
    )
    .bind(company_id)
    .fetch_optional(pool)
    .await?;

    let quote_number = sqlx::query_scalar::<_, String>(
        "SELECT quote_number FROM quotes WHERE company_id = ? ORDER BY created_at ASC, id ASC LIMIT 1",
    )
    .bind(company_id)
    .fetch_optional(pool)
    .await?;

    let invoice = quote_linked_invoice(pool, company_id).await?;
    let invoice_done = invoice.is_some();
    let invoice_number = invoice
        .as_ref()
        .map(|invoice| invoice.invoice_number.clone())
        .unwrap_or_default();
    let payment_done = invoice
        .as_ref()
        .map(|invoice| invoice.paid_amount > 0.0)
        .unwrap_or(false);
    let ar_clear_done = invoice
        .as_ref()
        .map(|invoice| invoice.status == "paid" && invoice.paid_amount + 0.000001 >= invoice.amount)
        .unwrap_or(false);

    let steps = vec![
        activation_step(
            "company",
            "公司信息",
            "/company-info",
            company_info_done,
            if company_info_done {
                "已填写公司信息".to_string()
            } else {
                "尚未填写公司信息".to_string()
            },
        ),
        activation_step(
            "client",
            "客户",
            "/crm",
            client_name.is_some(),
            client_name
                .map(|name| format!("已添加客户：{name}"))
                .unwrap_or_else(|| "尚未添加客户".to_string()),
        ),
        activation_step(
            "quote",
            "报价",
            "/quotes",
            quote_number.is_some(),
            quote_number
                .map(|number| format!("已创建报价：{number}"))
                .unwrap_or_else(|| "尚未创建报价".to_string()),
        ),
        activation_step(
            "invoice",
            "报价转发票",
            "/quotes",
            invoice_done,
            if invoice_done {
                format!("已生成发票：{invoice_number}")
            } else {
                "尚未生成与报价相连的发票".to_string()
            },
        ),
        activation_step(
            "invoice_pdf",
            "发票 PDF",
            "/invoices",
            invoice_done,
            if invoice_done {
                format!("PDF 可下载：{invoice_number}")
            } else {
                "发票生成后即可下载 PDF".to_string()
            },
        ),
        activation_step(
            "payment",
            "收款",
            "/invoices",
            payment_done,
            invoice
                .as_ref()
                .filter(|invoice| invoice.paid_amount > 0.0)
                .map(|invoice| format!("已记录收款：{}", money_evidence(invoice.paid_amount)))
                .unwrap_or_else(|| "尚未记录发票收款".to_string()),
        ),
        activation_step(
            "ar_clear",
            "AR 清零",
            "/payments",
            ar_clear_done,
            if ar_clear_done {
                "发票已结清，应收账龄已清零".to_string()
            } else {
                "发票结清后会退出应收账龄".to_string()
            },
        ),
    ];

    let total_count = steps.len() as i64;
    let completed_count = steps
        .iter()
        .filter(|step| step["completed"].as_bool() == Some(true))
        .count() as i64;
    let current_step_id = steps
        .iter()
        .find(|step| step["completed"].as_bool() != Some(true))
        .and_then(|step| step["id"].as_str())
        .unwrap_or("complete");
    let rate = pct(completed_count, total_count);
    let completed = completed_count == total_count;

    Ok(json!({
        "id": "first_paid_invoice",
        "title": "第一笔收款闭环",
        "completed": completed,
        "current_step_id": current_step_id,
        "completed_count": completed_count,
        "total_count": total_count,
        "remaining_count": total_count.saturating_sub(completed_count),
        "rate": rate,
        "rate_label": format!("{}%", rate),
        "label": format!("第一笔收款闭环 {}/{}", completed_count, total_count),
        "status_badge": status_badge(completed),
        "next_action": activation_next_action(current_step_id),
        "steps": steps,
    }))
}

/// Build the full onboarding-progress payload for a company. This is the exact
/// shape both the HTTP endpoint and the agent tool return, and the only place
/// completion is computed.
pub async fn build_progress(pool: &SqlitePool, company_id: &str) -> Result<Value, sqlx::Error> {
    let specs = module_specs();

    let mut modules = Vec::with_capacity(specs.len());
    let mut tutorial_steps = Vec::new();
    let mut completed_count = 0_i64;
    let mut total_records = 0_i64;

    // Activation = Tier 1 only (the value path). Tracked separately so the
    // tutorial can celebrate "first dollar" without waiting on the whole ledger.
    let mut activation_completed = 0_i64;
    let mut activation_total = 0_i64;

    // Section headers: emit a title on the first item of each tier.
    let mut last_module_tier = 0_u8;
    let mut last_step_tier = 0_u8;

    for spec in &specs {
        let count = count_rows(pool, company_id, spec.count_sql).await?;
        let completed = count > 0;
        if completed {
            completed_count += 1;
        }
        total_records += count;
        if spec.tier == 1 {
            activation_total += 1;
            if completed {
                activation_completed += 1;
            }
        }

        let module_section = if spec.tier != last_module_tier {
            tier_title(spec.tier)
        } else {
            ""
        };
        last_module_tier = spec.tier;
        modules.push(json!({
            "id": spec.id,
            "name": spec.name,
            "icon_key": spec.icon_key,
            "route": spec.route,
            "completed": completed,
            "count": count,
            "count_label": format!("{} 条数据", count),
            "description": spec.description,
            "status_badge": status_badge(completed),
            "tier": spec.tier,
            "section_title": module_section,
        }));

        if spec.show_in_tutorial {
            let step_section = if spec.tier != last_step_tier {
                tier_title(spec.tier)
            } else {
                ""
            };
            last_step_tier = spec.tier;
            tutorial_steps.push(json!({
                "id": spec.id,
                "title": spec.tutorial_title,
                "description": spec.tutorial_description,
                "icon_key": spec.icon_key,
                "route": spec.route,
                "completed": completed,
                "action_label": spec.action_label,
                "status_badge": status_badge(completed),
                "tier": spec.tier,
                "section_title": step_section,
            }));
        }
    }

    let total_count = modules.len() as i64;
    let remaining_count = total_count.saturating_sub(completed_count);
    let progress_rate = pct(completed_count, total_count);
    let all_completed = total_count > 0 && completed_count == total_count;

    let activation_rate = pct(activation_completed, activation_total);
    let activation_remaining = activation_total.saturating_sub(activation_completed);
    let activation_all_done = activation_total > 0 && activation_completed == activation_total;

    let demo_company = is_demo_company(pool, company_id).await?;
    let tutorial_mode = if demo_company {
        "demo_walkthrough"
    } else {
        "data_completion"
    };

    let first_incomplete_index = tutorial_steps
        .iter()
        .position(|step| step["completed"].as_bool() != Some(true))
        .unwrap_or(0);

    // Demo CTA: while exploring the seeded sample company, foreground "create my
    // company" so the user moves from the sandbox into a real, owned company.
    let cta = if demo_company {
        json!({
            "kind": "create_company",
            "title": "你正在浏览示例公司",
            "description": "这些是示例数据，方便你先逛一圈。准备好后，创建你自己的公司，跟着引导跑通「获客 → 报价 → 开票 → 收款」。",
            "action_label": "创建我的公司",
            "input_placeholder": "输入你的公司名称"
        })
    } else {
        Value::Null
    };
    let activation_loop = build_activation_loop(pool, company_id).await?;
    let operating_backbone =
        crate::app::operating_backbone::build_operating_backbone(pool, company_id).await?;

    Ok(json!({
        "summary": {
            "completed_count": completed_count,
            "total_count": total_count,
            "remaining_count": remaining_count,
            "total_records": total_records,
            "progress_rate": progress_rate,
            "progress_rate_label": format!("{}%", progress_rate),
            "completed_label": format!("已完成 {}/{} 个模块", completed_count, total_count),
            "total_records_label": format!("共录入 {} 条数据", total_records),
            "all_completed": all_completed,
            "first_incomplete_index": first_incomplete_index,
            "demo_company": demo_company,
            "tutorial_mode": tutorial_mode,
            "tutorial_storage_key": format!("onboarding-tutorial-dismissed:{}", company_id),
            "activation": {
                "completed_count": activation_completed,
                "total_count": activation_total,
                "remaining_count": activation_remaining,
                "rate": activation_rate,
                "rate_label": format!("{}%", activation_rate),
                "label": format!("激活进度 {}/{}", activation_completed, activation_total),
                "completed": activation_all_done
            },
            "activation_loop": activation_loop,
            "operating_backbone": operating_backbone,
            "cta": cta
        },
        "modules": modules,
        "tutorial_steps": tutorial_steps,
        "incomplete_tip": {
            "title": "先跑通第一笔收款",
            "description": "按顺序：公司 → 客户 → 报价 → 发票 → 收款。做完一步，早晨工作台会指出下一步。证照、保单、银行账户补上后，停业风险也会出现在同一页。"
        },
        "completed_tip": {
            "title": "台账齐了，以后先看工作台",
            "description": "每天打开智能工作台：企业风险是会停摆的事，待跟是会漏单的事。不必把每个模块都点一遍。"
        },
        "tutorial_completed_tip": {
            "title": "第一笔收款闭环已经跑通",
            "description": "去仪表盘看早晨工作台。下一步把证照、保单、银行账户和房租记上，系统才能在停业前喊你。"
        }
    }))
}
