use chrono::{Datelike, NaiveDate, Timelike, Utc};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize)]
pub struct WorkbenchSummary {
    pub generated_at: String,
    pub generated_date_label: String,
    pub greeting: String,
    pub operating_control: OperatingControl,
    pub stats: WorkbenchStats,
    pub header_badges: Vec<WorkbenchHeaderBadge>,
    pub urgent_items: Vec<UrgentItem>,
    pub application_reminders: Vec<UrgentItem>,
    pub asset_expiry_reminders: Vec<UrgentItem>,
    pub attention_items: Vec<UrgentItem>,
    pub today_tasks: Vec<TodayTask>,
    pub today_tasks_count: i64,
    pub todos: Vec<TodoItem>,
    pub ai_suggestions: Vec<String>,
    pub completion_rate: i64,
    pub stage_stats: StageStats,
    pub monthly_expense_trend: Vec<MonthlyExpensePoint>,
    pub overview_cards: Vec<WorkbenchOverviewCard>,
    pub quick_actions: Vec<WorkbenchQuickAction>,
    pub survival_items: Vec<UrgentItem>,
    pub survival_count: i64,
    pub survival_count_label: String,
    pub survival_title: String,
    pub survival_subtitle: String,
    pub guidance: WorkbenchGuidance,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkbenchGuidance {
    pub visible: bool,
    pub kind: String,
    pub title: String,
    pub subtitle: String,
    pub progress_label: String,
    pub next_label: String,
    pub next_description: String,
    pub next_route: String,
    pub empty_urgent_title: String,
    pub empty_urgent_description: String,
    pub steps: Vec<WorkbenchGuidanceStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkbenchGuidanceStep {
    pub id: String,
    pub label: String,
    pub description: String,
    pub completed: bool,
    pub route: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkbenchStats {
    pub pending_invoices: i64,
    pub pending_invoices_label: String,
    pub active_contracts: i64,
    pub active_contracts_label: String,
    pub expiring_assets: i64,
    pub expiring_assets_label: String,
    pub ongoing_projects: i64,
    pub ongoing_projects_label: String,
    pub employees: i64,
    pub employees_label: String,
    pub monthly_expense: f64,
    pub monthly_expense_label: String,
    pub overdue_receivables: i64,
    pub overdue_receivables_label: String,
    pub overdue_receivable_amount: f64,
    pub overdue_receivable_amount_label: String,
    pub month_cash_inflow: f64,
    pub month_cash_outflow: f64,
    pub month_cash_net: f64,
    pub month_cash_inflow_label: String,
    pub month_cash_outflow_label: String,
    pub month_cash_net_label: String,
    pub month_cash_stressed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkbenchHeaderBadge {
    pub label: String,
    pub variant: String,
    pub class_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UrgentItem {
    pub id: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub title: String,
    pub description: String,
    pub priority: String,
    pub due_date: Option<String>,
    pub days_left: Option<i64>,
    pub days_left_label: Option<String>,
    pub priority_badge: WorkbenchBadge,
    pub type_icon: String,
    pub action: String,
    pub path: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct TodayTask {
    pub id: String,
    pub title: String,
    pub priority: String,
    pub due_date: Option<String>,
    pub source_type: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TodoItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub priority: String,
    pub due_date: Option<String>,
    pub completed: bool,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StageStats {
    pub total_assets: i64,
    pub total_assets_label: String,
    pub total_value: f64,
    pub total_value_label: String,
    pub pending_invoices: i64,
    pub pending_invoices_label: String,
    pub total_salary: f64,
    pub total_salary_label: String,
    pub ongoing_projects: i64,
    pub ongoing_projects_label: String,
    pub active_contracts: i64,
    pub active_contracts_label: String,
    pub employee_count: i64,
    pub employee_count_label: String,
    pub total_projects: i64,
    pub total_projects_label: String,
    pub total_contracts: i64,
    pub total_contracts_label: String,
    pub project_execution_rate_label: String,
    pub contract_activity_rate_label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonthlyExpensePoint {
    pub month: String,
    pub salary: f64,
    pub salary_label: String,
    pub invoice: f64,
    pub invoice_label: String,
    pub total: f64,
    pub total_label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkbenchBadge {
    pub label: String,
    pub class_name: String,
    pub color_class: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkbenchOverviewCard {
    pub key: String,
    pub icon: String,
    pub title: String,
    pub value_label: String,
    pub subtitle: Option<String>,
    pub trend_label: Option<String>,
    pub trend_up: Option<bool>,
    pub trend_variant: Option<String>,
    pub color: String,
    pub class_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkbenchQuickAction {
    pub icon: String,
    pub label: String,
    pub description: String,
    pub path: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OperatingControl {
    pub id: String,
    pub title: String,
    pub status: String,
    pub status_label: String,
    pub tone: String,
    pub backbone_rate: i64,
    pub backbone_rate_label: String,
    pub completed: bool,
    pub blocked_step_id: Option<String>,
    pub blocked_step_label: Option<String>,
    pub risk_count: i64,
    pub critical_count: i64,
    pub risk_count_label: String,
    pub summary_label: String,
    pub next_action: Value,
    pub control_items: Vec<OperatingControlItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OperatingControlItem {
    pub id: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub title: String,
    pub description: String,
    pub priority: String,
    pub route: String,
    pub action_label: String,
    pub badge: WorkbenchBadge,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct InsightsResponse {
    pub insights: Vec<OperationalInsight>,
    pub critical_insights: Vec<OperationalInsight>,
    pub other_insights: Vec<OperationalInsight>,
    pub operating_control: OperatingControl,
    pub completeness: CompletenessResponse,
    pub summary: InsightSummary,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OperationalInsight {
    pub id: String,
    #[serde(rename = "type")]
    pub insight_type: String,
    pub title: String,
    pub description: String,
    pub priority: String,
    pub priority_badge: WorkbenchBadge,
    pub category: String,
    pub icon: String,
    pub display_type: String,
    pub display_icon: String,
    pub icon_class_name: String,
    pub icon_bg_class_name: String,
    pub card_border_class_name: String,
    pub card_ring_class_name: String,
    pub action_class_name: String,
    pub action: String,
    pub action_label: String,
    pub confidence: f64,
    pub data: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompletenessResponse {
    pub overall_score: i64,
    pub overall_score_label: String,
    pub overall_gradient_class: String,
    pub unlock_title: String,
    pub unlock_description: String,
    pub completed_count: i64,
    pub incomplete_count: i64,
    pub total_count: i64,
    pub completed_label: String,
    pub incomplete_label: String,
    pub is_complete: bool,
    pub status_title: String,
    pub status_description: String,
    pub completion_badge_label: String,
    pub remaining_overflow_count: i64,
    pub remaining_overflow_label: Option<String>,
    pub items: Vec<CompletenessItem>,
    pub completed_items: Vec<CompletenessItem>,
    pub incomplete_items: Vec<CompletenessItem>,
    pub top_incomplete_items: Vec<CompletenessItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompletenessItem {
    pub key: String,
    pub label: String,
    pub icon: String,
    pub path: String,
    pub score: i64,
    pub max_score: i64,
    pub percentage: i64,
    pub percentage_label: String,
    pub missing_items: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InsightSummary {
    pub total: i64,
    pub critical: i64,
    pub high: i64,
    pub medium: i64,
    pub low: i64,
}

pub async fn build_summary(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<WorkbenchSummary, sqlx::Error> {
    let today = Utc::now().date_naive();
    let current_month = today.format("%Y-%m").to_string();

    let pending_invoices = count(
        pool,
        "SELECT COUNT(*) FROM invoices WHERE company_id = ? AND status = 'pending'",
        company_id,
    )
    .await?;

    let active_contracts = count(
        pool,
        "SELECT COUNT(*) FROM contracts WHERE company_id = ? AND status = 'active'",
        company_id,
    )
    .await?;

    let ongoing_projects = count(
        pool,
        "SELECT COUNT(*) FROM projects WHERE company_id = ? AND status IN ('ongoing', 'active')",
        company_id,
    )
    .await?;

    let employees = count(
        pool,
        "SELECT COUNT(*) FROM employees WHERE company_id = ? AND status = 'active'",
        company_id,
    )
    .await?;

    let invoice_expense = monthly_invoice_expense(pool, company_id, &current_month).await?;
    let salary_expense = monthly_salary_expense(pool, company_id, &current_month).await?;
    let monthly_expense = invoice_expense + salary_expense;
    let stage_stats = build_stage_stats(
        pool,
        company_id,
        pending_invoices,
        active_contracts,
        ongoing_projects,
        employees,
        salary_expense,
    )
    .await?;
    let monthly_expense_trend = monthly_expense_trend(pool, company_id, today).await?;

    let overdue = overdue_receivable_snapshot(pool, company_id, today).await?;
    let cash = month_cash_snapshot(pool, company_id, today).await?;
    let mut urgent_items = Vec::new();
    urgent_items.extend(overdue.items.clone());
    if let Some(item) = cash.urgent.clone() {
        urgent_items.push(item);
    }
    urgent_items.extend(invoice_urgent_items(pool, company_id, today).await?);
    urgent_items.extend(contract_urgent_items(pool, company_id, today).await?);
    urgent_items.extend(asset_urgent_items(pool, company_id, today).await?);
    urgent_items.extend(application_urgent_items(pool, company_id, today).await?);
    urgent_items.extend(tax_deadline_urgent_items(pool, company_id, today).await?);
    urgent_items.extend(subscription_urgent_items(pool, company_id, today).await?);
    urgent_items.extend(payable_urgent_items(pool, company_id, today).await?);
    urgent_items.extend(rent_urgent_items(pool, company_id, today).await?);
    urgent_items.extend(license_urgent_items(pool, company_id, today).await?);
    urgent_items.extend(insurance_urgent_items(pool, company_id, today).await?);
    urgent_items.extend(stale_quote_urgent_items(pool, company_id, today).await?);
    urgent_items.extend(recurring_invoice_urgent_items(pool, company_id, today).await?);
    urgent_items.extend(cold_deal_urgent_items(pool, company_id, today).await?);
    if let Some(item) = payroll_urgent_item(pool, company_id, today).await? {
        urgent_items.push(item);
    }
    if let Some(item) = bank_cash_urgent_item(pool, company_id, today).await? {
        urgent_items.push(item);
    }
    if let Some(item) = unbilled_time_urgent_item(pool, company_id).await? {
        urgent_items.push(item);
    }
    if let Some(item) = mailbox_inbox_urgent_item(pool, company_id).await? {
        urgent_items.push(item);
    }
    if let Some(close_item) = monthly_close_urgent_item(pool, company_id, today).await? {
        urgent_items.push(close_item);
    }
    urgent_items.sort_by(|a, b| {
        priority_rank(&a.priority)
            .cmp(&priority_rank(&b.priority))
            .then(
                a.days_left
                    .unwrap_or(i64::MAX)
                    .cmp(&b.days_left.unwrap_or(i64::MAX)),
            )
            .then(a.title.cmp(&b.title))
    });

    let application_reminders = urgent_items
        .iter()
        .filter(|item| item.item_type == "application")
        .cloned()
        .collect::<Vec<_>>();
    let asset_expiry_reminders = urgent_items
        .iter()
        .filter(|item| item.item_type == "asset")
        .cloned()
        .collect::<Vec<_>>();
    let expiring_assets = asset_expiry_reminders.len() as i64;
    let today_tasks = build_today_tasks(&urgent_items);
    let todos = build_todos(&urgent_items);
    let completion_rate = completion_rate(&todos);
    let ai_suggestions = suggestions(
        pool,
        company_id,
        pending_invoices,
        expiring_assets,
        overdue.count,
        overdue.amount,
        cash.net,
        cash.stressed,
        &urgent_items,
    )
    .await?;
    let stats = WorkbenchStats {
        pending_invoices,
        pending_invoices_label: format!("{} 张待审发票", pending_invoices),
        active_contracts,
        active_contracts_label: active_contracts.to_string(),
        expiring_assets,
        expiring_assets_label: format!("{} 项即将到期", expiring_assets),
        ongoing_projects,
        ongoing_projects_label: ongoing_projects.to_string(),
        employees,
        employees_label: employees.to_string(),
        monthly_expense,
        monthly_expense_label: format_money(monthly_expense),
        overdue_receivables: overdue.count,
        overdue_receivables_label: format!("{} 笔逾期应收", overdue.count),
        overdue_receivable_amount: overdue.amount,
        overdue_receivable_amount_label: format_money(overdue.amount),
        month_cash_inflow: cash.inflow,
        month_cash_outflow: cash.outflow,
        month_cash_net: cash.net,
        month_cash_inflow_label: format_money(cash.inflow),
        month_cash_outflow_label: format_money(cash.outflow),
        month_cash_net_label: format_money(cash.net),
        month_cash_stressed: cash.stressed,
    };
    let survival_items = survival_items(&urgent_items);
    let attention_items = attention_items(&urgent_items);
    let header_badges = workbench_header_badges(&stats, survival_items.len() as i64);
    let operating_control = build_operating_control(pool, company_id, today).await?;
    let overview_cards = overview_cards(&stats, &monthly_expense_trend, &operating_control);
    let quick_actions = quick_actions();
    let guidance = build_guidance(pool, company_id).await?;

    Ok(WorkbenchSummary {
        generated_at: Utc::now().to_rfc3339(),
        generated_date_label: generated_date_label(today),
        greeting: greeting(),
        operating_control,
        stats,
        header_badges,
        urgent_items,
        application_reminders,
        asset_expiry_reminders,
        attention_items,
        today_tasks_count: today_tasks.len() as i64,
        today_tasks,
        todos,
        ai_suggestions,
        completion_rate,
        stage_stats,
        monthly_expense_trend,
        overview_cards,
        quick_actions,
        survival_count: survival_items.len() as i64,
        survival_count_label: if survival_items.is_empty() {
            "没有会停摆的事项".to_string()
        } else {
            format!("{} 项企业风险", survival_items.len())
        },
        survival_title: "企业风险".to_string(),
        survival_subtitle: "先处理这些：发不出工资、被罚停、证照停业或被清场。".to_string(),
        survival_items,
        guidance,
    })
}

fn json_string(value: &Value, fallback: &str) -> String {
    value
        .as_str()
        .filter(|text| !text.trim().is_empty())
        .unwrap_or(fallback)
        .to_string()
}

async fn build_guidance(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<WorkbenchGuidance, sqlx::Error> {
    let activation = crate::api::onboarding::build_activation_loop(pool, company_id).await?;
    if activation["completed"].as_bool() != Some(true) {
        let next = &activation["next_action"];
        let steps = activation["steps"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|step| {
                Some(WorkbenchGuidanceStep {
                    id: json_string(&step["id"], ""),
                    label: json_string(&step["label"], ""),
                    description: json_string(&step["evidence_label"], ""),
                    completed: step["completed"].as_bool()?,
                    route: json_string(&step["route"], "/dashboard"),
                })
            })
            .filter(|step| !step.id.is_empty())
            .collect();
        return Ok(WorkbenchGuidance {
            visible: true,
            kind: "activation".to_string(),
            title: json_string(&activation["title"], "先跑通第一笔收款"),
            subtitle: json_string(
                &next["description"],
                "空账本不是没事做。先把第一笔生意从客户收到钱跑通。",
            ),
            progress_label: json_string(&activation["label"], ""),
            next_label: json_string(&next["label"], "开始"),
            next_description: json_string(&next["description"], ""),
            next_route: json_string(&next["route"], "/company-info"),
            empty_urgent_title: "还没有要催的事".to_string(),
            empty_urgent_description: "先把第一笔生意跑通，工作台才有逾期、报价和商机可盯。"
                .to_string(),
            steps,
        });
    }

    let setup = risk_setup_steps(pool, company_id).await?;
    if setup.iter().any(|step| !step.completed) {
        let next = setup
            .iter()
            .find(|step| !step.completed)
            .cloned()
            .unwrap_or_else(|| setup[0].clone());
        let done = setup.iter().filter(|step| step.completed).count();
        return Ok(WorkbenchGuidance {
            visible: true,
            kind: "risk_setup".to_string(),
            title: "下一步：补上会停摆的台账".to_string(),
            subtitle: "证照、保单、银行账户、房租记上，早晨工作台才能提前喊你。".to_string(),
            progress_label: format!("生存台账 {done}/{}", setup.len()),
            next_label: next.label.clone(),
            next_description: next.description.clone(),
            next_route: next.route.clone(),
            empty_urgent_title: "今天没有待跟事项".to_string(),
            empty_urgent_description:
                "收款闭环已经跑通。把下面几项补上，停业风险才会出现在工作台。".to_string(),
            steps: setup,
        });
    }

    Ok(WorkbenchGuidance {
        visible: false,
        kind: "ready".to_string(),
        title: String::new(),
        subtitle: String::new(),
        progress_label: String::new(),
        next_label: String::new(),
        next_description: String::new(),
        next_route: "/dashboard".to_string(),
        empty_urgent_title: "今天没有待办".to_string(),
        empty_urgent_description: "没有会停摆或要跟进的事项。".to_string(),
        steps: Vec::new(),
    })
}

async fn risk_setup_steps(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<Vec<WorkbenchGuidanceStep>, sqlx::Error> {
    let specs = [
        (
            "licenses",
            "录入证照",
            "过期可能直接停业。",
            "/licenses",
            "SELECT COUNT(*) FROM licenses WHERE company_id = ?",
        ),
        (
            "insurance",
            "录入保单",
            "保险空窗一天，出事只能自己扛。",
            "/compliance",
            "SELECT COUNT(*) FROM insurance_policies WHERE company_id = ?",
        ),
        (
            "bank",
            "登记银行账户",
            "没有账面余额，工作台算不出够不够发工资。",
            "/bank-accounts",
            "SELECT COUNT(*) FROM bank_accounts WHERE company_id = ?",
        ),
        (
            "rent",
            "登记房租",
            "晚付滞纳金，租约到期再谈容易被清场。",
            "/rent",
            "SELECT COUNT(*) FROM rent_records WHERE company_id = ?",
        ),
    ];
    let mut steps = Vec::new();
    for (id, label, description, route, sql) in specs {
        let completed = count(pool, sql, company_id).await? > 0;
        steps.push(WorkbenchGuidanceStep {
            id: id.to_string(),
            label: label.to_string(),
            description: description.to_string(),
            completed,
            route: route.to_string(),
        });
    }
    Ok(steps)
}

fn is_survival_item(item: &UrgentItem) -> bool {
    let id = item.id.as_str();
    id.starts_with("cash-gap:")
        || id.starts_with("receivable:")
        || id.starts_with("tax-deadline:")
        || id.starts_with("license:")
        || id.starts_with("insurance:")
        || id.starts_with("rent-")
        || id.starts_with("payable:")
        || id.starts_with("payroll:")
        || id.starts_with("bank-cash")
        || id.starts_with("domain:")
}

fn survival_items(urgent_items: &[UrgentItem]) -> Vec<UrgentItem> {
    urgent_items
        .iter()
        .filter(|item| is_survival_item(item))
        .take(8)
        .cloned()
        .collect()
}

fn attention_items(urgent_items: &[UrgentItem]) -> Vec<UrgentItem> {
    urgent_items
        .iter()
        .filter(|item| !is_survival_item(item))
        .take(5)
        .cloned()
        .collect()
}

fn workbench_header_badges(
    stats: &WorkbenchStats,
    survival_count: i64,
) -> Vec<WorkbenchHeaderBadge> {
    let mut badges = Vec::new();
    if survival_count > 0 {
        badges.push(WorkbenchHeaderBadge {
            label: format!("{survival_count} 项企业风险"),
            variant: "destructive".to_string(),
            class_name: "py-1.5 px-3".to_string(),
        });
    }
    if stats.overdue_receivables > 0 {
        badges.push(WorkbenchHeaderBadge {
            label: format!(
                "{} · {}",
                stats.overdue_receivables_label, stats.overdue_receivable_amount_label
            ),
            variant: "destructive".to_string(),
            class_name: "py-1.5 px-3".to_string(),
        });
    }
    if stats.pending_invoices > 0 {
        badges.push(WorkbenchHeaderBadge {
            label: stats.pending_invoices_label.clone(),
            variant: "secondary".to_string(),
            class_name: "py-1.5 px-3".to_string(),
        });
    }
    if stats.expiring_assets > 0 {
        badges.push(WorkbenchHeaderBadge {
            label: stats.expiring_assets_label.clone(),
            variant: "outline".to_string(),
            class_name: "py-1.5 px-3 border-warning text-warning".to_string(),
        });
    }
    if stats.month_cash_stressed {
        badges.push(WorkbenchHeaderBadge {
            label: format!("本月现金缺口 {}", stats.month_cash_net_label),
            variant: "destructive".to_string(),
            class_name: "py-1.5 px-3".to_string(),
        });
    }
    badges
}

async fn build_operating_control(
    pool: &SqlitePool,
    company_id: &str,
    today: NaiveDate,
) -> Result<OperatingControl, sqlx::Error> {
    let backbone =
        crate::app::operating_backbone::build_operating_backbone(pool, company_id).await?;
    let completed = backbone["completed"].as_bool().unwrap_or(false);
    let backbone_rate = backbone["rate"].as_i64().unwrap_or(0);
    let backbone_rate_label = value_string(&backbone["rate_label"], format!("{}%", backbone_rate));
    let current_step_id = value_string(&backbone["current_step_id"], "complete".to_string());
    let next_action = backbone["next_action"].clone();
    let mut control_items = Vec::new();

    let blocked_step = if completed {
        None
    } else {
        find_backbone_step(&backbone, &current_step_id)
    };

    if let Some(step) = blocked_step.as_ref() {
        let route = value_string(&step["route"], "/dashboard".to_string());
        let label = value_string(&step["label"], "运营骨架".to_string());
        let evidence_label = value_string(&step["evidence_label"], "缺少运营证据".to_string());
        let action_label = value_string(&next_action["label"], "去处理".to_string());
        control_items.push(OperatingControlItem {
            id: "backbone_gap".to_string(),
            item_type: "backbone_gap".to_string(),
            title: format!("运营骨架缺口：{}", label),
            description: evidence_label,
            priority: "critical".to_string(),
            route,
            action_label,
            badge: priority_badge("critical"),
            metadata: json!({
                "blocked_step_id": current_step_id,
                "backbone_rate": backbone_rate,
            }),
        });
    }

    control_items.extend(operating_deadline_control_items(pool, company_id, today).await?);

    let risk_count = control_items.len() as i64;
    let critical_count = control_items
        .iter()
        .filter(|item| item.priority == "critical")
        .count() as i64;
    let (status, status_label, tone) = if critical_count > 0 {
        ("blocked", "需要处理", "danger")
    } else if risk_count > 0 {
        ("attention", "需要关注", "warning")
    } else {
        ("healthy", "健康", "success")
    };
    let blocked_step_id = blocked_step
        .as_ref()
        .and_then(|step| step["id"].as_str().map(str::to_string));
    let blocked_step_label = blocked_step
        .as_ref()
        .and_then(|step| step["label"].as_str().map(str::to_string));
    let summary_label = if let Some(label) = blocked_step_label.as_ref() {
        format!("运营骨架卡在{}，请先补齐关键证据。", label)
    } else if risk_count > 0 {
        format!("运营骨架完整，但有 {} 个运营风险需要跟进。", risk_count)
    } else {
        "运营骨架健康，暂无关键风险。".to_string()
    };

    Ok(OperatingControl {
        id: "operating_control".to_string(),
        title: "运营指挥台".to_string(),
        status: status.to_string(),
        status_label: status_label.to_string(),
        tone: tone.to_string(),
        backbone_rate,
        backbone_rate_label,
        completed,
        blocked_step_id,
        blocked_step_label,
        risk_count,
        critical_count,
        risk_count_label: format!("{} 个风险", risk_count),
        summary_label,
        next_action,
        control_items,
    })
}

fn find_backbone_step(backbone: &Value, step_id: &str) -> Option<Value> {
    backbone["steps"].as_array().and_then(|steps| {
        steps
            .iter()
            .find(|step| step["id"].as_str() == Some(step_id))
            .cloned()
    })
}

fn value_string(value: &Value, fallback: String) -> String {
    value.as_str().map(str::to_string).unwrap_or(fallback)
}

async fn operating_deadline_control_items(
    pool: &SqlitePool,
    company_id: &str,
    today: NaiveDate,
) -> Result<Vec<OperatingControlItem>, sqlx::Error> {
    let rows: Vec<(
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        r#"
        SELECT id, title, description, COALESCE(remind_date, due_date) AS due_date,
               entity_type, entity_id
          FROM reminders
         WHERE company_id = ?
           AND COALESCE(is_completed, 0) = 0
           AND LOWER(COALESCE(status, 'pending')) NOT IN ('completed', 'dismissed', 'done')
           AND LOWER(COALESCE(entity_type, '')) IN (
               'contracts', 'contract', 'projects', 'project', 'employees', 'employee',
               'salary_records', 'salary_record', 'payroll', 'assets', 'asset',
               'licenses', 'license', 'rent_records', 'rent_record', 'rent',
               'budgets', 'budget'
           )
           AND COALESCE(entity_id, '') != ''
           AND COALESCE(remind_date, due_date) IS NOT NULL
         ORDER BY COALESCE(remind_date, due_date) ASC, created_at ASC, id ASC
         LIMIT 8
        "#,
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(
            |(id, title, description, due_date, entity_type, entity_id)| {
                let days_left = due_date
                    .as_deref()
                    .and_then(|date| days_until(date, today))?;
                if days_left > 14 {
                    return None;
                }
                let priority = if days_left < 0 { "critical" } else { "high" };
                let date_label = days_left_label_for_type("reminder", days_left);
                Some(OperatingControlItem {
                    id: format!("operating_deadline:{}", id),
                    item_type: "operating_deadline".to_string(),
                    title: format!("运营提醒：{}", title),
                    description: description.unwrap_or_else(|| format!("{}需要处理", date_label)),
                    priority: priority.to_string(),
                    route: "/reminders".to_string(),
                    action_label: "处理提醒".to_string(),
                    badge: priority_badge(priority),
                    metadata: json!({
                        "reminder_id": id,
                        "due_date": due_date,
                        "days_left": days_left,
                        "days_left_label": date_label,
                        "entity_type": entity_type,
                        "entity_id": entity_id,
                    }),
                })
            },
        )
        .collect())
}

pub async fn build_insights(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<InsightsResponse, sqlx::Error> {
    let summary_data = build_summary(pool, company_id).await?;
    let completeness = build_completeness(pool, company_id).await?;
    let operating_control = summary_data.operating_control.clone();
    let mut insights = Vec::new();

    for item in summary_data.urgent_items.iter().take(5) {
        insights.push(operational_insight(
            format!("urgent:{}", item.id),
            "urgent",
            item.title.clone(),
            item.description.clone(),
            item.priority.clone(),
            item.item_type.clone(),
            item.path.clone(),
            0.92,
            json!({
                "source_id": item.id,
                "due_date": item.due_date,
                "days_left": item.days_left,
                "metadata": item.metadata,
            }),
        ));
    }

    if summary_data.stats.pending_invoices > 0 {
        insights.push(operational_insight(
            "finance:pending-invoices".to_string(),
            "finance",
            "待处理发票".to_string(),
            format!(
                "当前有 {} 张发票待处理，建议按到期日优先审批。",
                summary_data.stats.pending_invoices
            ),
            if summary_data.stats.pending_invoices > 5 {
                "high".to_string()
            } else {
                "medium".to_string()
            },
            "invoice".to_string(),
            "/invoices".to_string(),
            0.88,
            json!({ "pending_invoices": summary_data.stats.pending_invoices }),
        ));
    }

    for item in operating_control.control_items.iter().take(4) {
        let insight_id = if item.item_type == "backbone_gap" {
            "operating_control:backbone_gap".to_string()
        } else {
            format!("operating_control:{}", item.id)
        };
        insights.push(operational_insight(
            insight_id,
            "operating_control",
            item.title.clone(),
            item.description.clone(),
            item.priority.clone(),
            "operating_control".to_string(),
            item.route.clone(),
            0.9,
            json!({
                "source_id": item.id,
                "control_type": item.item_type,
                "metadata": item.metadata,
            }),
        ));
    }

    for item in completeness
        .items
        .iter()
        .filter(|item| item.score < item.max_score)
        .take(4)
    {
        insights.push(operational_insight(
            format!("completeness:{}", item.key),
            "completion",
            format!("完善{}", item.label),
            item.description.clone(),
            "medium".to_string(),
            item.key.clone(),
            item.path.clone(),
            0.78,
            json!({
                "score": item.score,
                "max_score": item.max_score,
                "missing_items": item.missing_items,
            }),
        ));
    }

    if insights.is_empty() {
        insights.push(operational_insight(
            "operations:healthy".to_string(),
            "status",
            "运营状态稳定".to_string(),
            "当前没有识别到高优先级待办，建议继续保持数据更新。".to_string(),
            "low".to_string(),
            "operations".to_string(),
            "/dashboard".to_string(),
            0.72,
            json!({}),
        ));
    }

    let insight_summary = summarize_insights(&insights);
    let critical_insights = insights
        .iter()
        .filter(|item| item.priority == "critical")
        .cloned()
        .collect::<Vec<_>>();
    let other_insights = insights
        .iter()
        .filter(|item| item.priority != "critical")
        .take(6)
        .cloned()
        .collect::<Vec<_>>();

    Ok(InsightsResponse {
        insights,
        critical_insights,
        other_insights,
        operating_control,
        completeness,
        summary: insight_summary,
        generated_at: Utc::now().to_rfc3339(),
    })
}

#[allow(clippy::too_many_arguments)]
fn operational_insight(
    id: String,
    insight_type: &str,
    title: String,
    description: String,
    priority: String,
    category: String,
    action: String,
    confidence: f64,
    data: Value,
) -> OperationalInsight {
    let display_type = insight_display_type(insight_type, &priority).to_string();
    let display = insight_display_metadata(&display_type);

    OperationalInsight {
        id,
        insight_type: insight_type.to_string(),
        title,
        description,
        priority_badge: insight_priority_badge(&priority),
        priority: priority.clone(),
        icon: insight_category_icon(&category).to_string(),
        category,
        display_type,
        display_icon: display.icon.to_string(),
        icon_class_name: display.icon_class_name.to_string(),
        icon_bg_class_name: display.icon_bg_class_name.to_string(),
        card_border_class_name: display.card_border_class_name.to_string(),
        card_ring_class_name: insight_card_ring_class(&priority).to_string(),
        action_class_name: display.action_class_name.to_string(),
        action,
        action_label: insight_action_label(insight_type).to_string(),
        confidence,
        data,
    }
}

struct InsightDisplayMetadata {
    icon: &'static str,
    icon_class_name: &'static str,
    icon_bg_class_name: &'static str,
    card_border_class_name: &'static str,
    action_class_name: &'static str,
}

fn insight_display_type(insight_type: &str, priority: &str) -> &'static str {
    if insight_type == "completion" {
        "completion"
    } else if insight_type == "finance" {
        "data"
    } else if matches!(priority, "critical" | "high") {
        "warning"
    } else {
        "suggestion"
    }
}

fn insight_display_metadata(display_type: &str) -> InsightDisplayMetadata {
    match display_type {
        "warning" => InsightDisplayMetadata {
            icon: "AlertTriangle",
            icon_class_name: "text-red-500",
            icon_bg_class_name: "bg-red-500/10",
            card_border_class_name: "border-red-500/30",
            action_class_name: "text-red-500",
        },
        "data" => InsightDisplayMetadata {
            icon: "TrendingUp",
            icon_class_name: "text-purple-500",
            icon_bg_class_name: "bg-purple-500/10",
            card_border_class_name: "border-purple-500/30",
            action_class_name: "text-purple-500",
        },
        "completion" => InsightDisplayMetadata {
            icon: "CheckCircle",
            icon_class_name: "text-orange-500",
            icon_bg_class_name: "bg-orange-500/10",
            card_border_class_name: "border-orange-500/30",
            action_class_name: "text-orange-500",
        },
        _ => InsightDisplayMetadata {
            icon: "Lightbulb",
            icon_class_name: "text-blue-500",
            icon_bg_class_name: "bg-blue-500/10",
            card_border_class_name: "border-blue-500/30",
            action_class_name: "text-blue-500",
        },
    }
}

fn insight_priority_badge(priority: &str) -> WorkbenchBadge {
    match priority {
        "critical" => WorkbenchBadge {
            label: "紧急".to_string(),
            class_name: "bg-red-500 text-white animate-pulse".to_string(),
            color_class: "bg-red-500".to_string(),
        },
        "high" => WorkbenchBadge {
            label: "重要".to_string(),
            class_name: "bg-orange-500 text-white".to_string(),
            color_class: "bg-orange-500".to_string(),
        },
        "medium" => WorkbenchBadge {
            label: "建议".to_string(),
            class_name: "bg-blue-500 text-white".to_string(),
            color_class: "bg-blue-500".to_string(),
        },
        _ => WorkbenchBadge {
            label: "提示".to_string(),
            class_name: "bg-gray-500 text-white".to_string(),
            color_class: "bg-gray-500".to_string(),
        },
    }
}

fn insight_card_ring_class(priority: &str) -> &'static str {
    if priority == "critical" {
        "ring-1 ring-red-500/20"
    } else {
        ""
    }
}

fn insight_category_icon(category: &str) -> &'static str {
    match category {
        "invoice" => "📄",
        "contract" => "📝",
        "asset" => "🏢",
        "application" => "🎯",
        "company" => "🏢",
        "employees" => "👥",
        "bank" => "🏦",
        "salary" => "💰",
        "tax" => "％",
        "operations" => "✨",
        _ => "💡",
    }
}

fn insight_action_label(insight_type: &str) -> &'static str {
    match insight_type {
        "completion" => "去完善",
        "urgent" => "去处理",
        _ => "查看",
    }
}

async fn count(pool: &SqlitePool, sql: &str, company_id: &str) -> Result<i64, sqlx::Error> {
    let (value,): (i64,) = sqlx::query_as(sql).bind(company_id).fetch_one(pool).await?;
    Ok(value)
}

async fn monthly_invoice_expense(
    pool: &SqlitePool,
    company_id: &str,
    current_month: &str,
) -> Result<f64, sqlx::Error> {
    let (value,): (f64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(amount), 0.0) FROM invoices WHERE company_id = ? AND substr(COALESCE(invoice_date, ''), 1, 7) = ?",
    )
    .bind(company_id)
    .bind(current_month)
    .fetch_one(pool)
    .await?;
    Ok(value)
}

async fn monthly_salary_expense(
    pool: &SqlitePool,
    company_id: &str,
    current_month: &str,
) -> Result<f64, sqlx::Error> {
    let (value,): (f64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(COALESCE(base_salary, 0.0) + COALESCE(bonus, 0.0)), 0.0) FROM salary_records WHERE company_id = ? AND period = ?",
    )
    .bind(company_id)
    .bind(current_month)
    .fetch_one(pool)
    .await?;
    Ok(value)
}

async fn build_stage_stats(
    pool: &SqlitePool,
    company_id: &str,
    pending_invoices: i64,
    active_contracts: i64,
    ongoing_projects: i64,
    employees: i64,
    salary_expense: f64,
) -> Result<StageStats, sqlx::Error> {
    let total_assets = count(
        pool,
        "SELECT COUNT(*) FROM assets WHERE company_id = ?",
        company_id,
    )
    .await?;
    let (total_value,): (f64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(COALESCE(current_value, 0.0)), 0.0) FROM assets WHERE company_id = ?",
    )
    .bind(company_id)
    .fetch_one(pool)
    .await?;
    let total_projects = count(
        pool,
        "SELECT COUNT(*) FROM projects WHERE company_id = ?",
        company_id,
    )
    .await?;
    let total_contracts = count(
        pool,
        "SELECT COUNT(*) FROM contracts WHERE company_id = ?",
        company_id,
    )
    .await?;

    Ok(StageStats {
        total_assets,
        total_assets_label: total_assets.to_string(),
        total_value,
        total_value_label: format_money(total_value),
        pending_invoices,
        pending_invoices_label: pending_invoices.to_string(),
        total_salary: salary_expense,
        total_salary_label: format_money(salary_expense),
        ongoing_projects,
        ongoing_projects_label: ongoing_projects.to_string(),
        active_contracts,
        active_contracts_label: active_contracts.to_string(),
        employee_count: employees,
        employee_count_label: employees.to_string(),
        total_projects,
        total_projects_label: total_projects.to_string(),
        total_contracts,
        total_contracts_label: total_contracts.to_string(),
        project_execution_rate_label: percent_label(ongoing_projects, total_projects),
        contract_activity_rate_label: percent_label(active_contracts, total_contracts),
    })
}

async fn monthly_expense_trend(
    pool: &SqlitePool,
    company_id: &str,
    today: NaiveDate,
) -> Result<Vec<MonthlyExpensePoint>, sqlx::Error> {
    let mut points = Vec::new();

    for offset in (0..6).rev() {
        let month = shift_month(today, -(offset as i32));
        let period = format!("{:04}-{:02}", month.year(), month.month());
        let salary = monthly_salary_expense(pool, company_id, &period).await?;
        let invoice = monthly_invoice_expense(pool, company_id, &period).await?;

        points.push(MonthlyExpensePoint {
            month: format!("{}月", month.month()),
            salary,
            salary_label: format_money(salary),
            invoice,
            invoice_label: format_money(invoice),
            total: salary + invoice,
            total_label: format_money(salary + invoice),
        });
    }

    Ok(points)
}

fn last_day_of_month(date: NaiveDate) -> NaiveDate {
    let next_month = if date.month() == 12 {
        NaiveDate::from_ymd_opt(date.year() + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1)
    };
    next_month
        .and_then(|first| first.pred_opt())
        .unwrap_or(date)
}

fn shift_month(date: NaiveDate, offset: i32) -> NaiveDate {
    let month_index = date.year() * 12 + date.month() as i32 - 1 + offset;
    let year = month_index.div_euclid(12);
    let month = month_index.rem_euclid(12) as u32 + 1;
    NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(date)
}

fn urgent_item(
    id: String,
    item_type: &str,
    title: String,
    description: String,
    priority: String,
    due_date: Option<String>,
    days_left: Option<i64>,
    action: &str,
    path: &str,
    metadata: Value,
) -> UrgentItem {
    UrgentItem {
        id,
        item_type: item_type.to_string(),
        title,
        description,
        priority_badge: priority_badge_for_type(item_type, &priority),
        priority,
        due_date,
        days_left_label: days_left.map(|days| days_left_label_for_type(item_type, days)),
        days_left,
        type_icon: type_icon(item_type).to_string(),
        action: action.to_string(),
        path: path.to_string(),
        metadata,
    }
}

async fn invoice_urgent_items(
    pool: &SqlitePool,
    company_id: &str,
    today: NaiveDate,
) -> Result<Vec<UrgentItem>, sqlx::Error> {
    let rows: Vec<(
        String,
        String,
        f64,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT id, invoice_number, amount, due_date, supplier, description
             FROM invoices
             WHERE company_id = ? AND status = 'pending'
             ORDER BY due_date IS NULL, due_date ASC, created_at DESC
             LIMIT 5",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, number, amount, due_date, supplier, description)| {
            let days_left = due_date.as_deref().and_then(|date| days_until(date, today));
            let item_description = description.unwrap_or_else(|| {
                format!(
                    "{}金额 ¥{:.2}",
                    supplier
                        .map(|name| format!("{}，", name))
                        .unwrap_or_default(),
                    amount
                )
            });
            urgent_item(
                format!("invoice:{}", id),
                "invoice",
                format!("发票待处理: {}", number),
                item_description,
                priority_for_days(days_left, 7),
                due_date,
                days_left,
                "去审批",
                "/invoices",
                json!({ "amount": amount, "invoice_number": number }),
            )
        })
        .collect())
}

struct OverdueReceivableSnapshot {
    count: i64,
    amount: f64,
    items: Vec<UrgentItem>,
}

async fn overdue_receivable_snapshot(
    pool: &SqlitePool,
    company_id: &str,
    today: NaiveDate,
) -> Result<OverdueReceivableSnapshot, sqlx::Error> {
    let rows: Vec<(String, String, f64, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT i.id, i.invoice_number,
                MAX(0, i.amount - COALESCE((
                    SELECT SUM(p.amount) FROM payments p
                    WHERE p.company_id = i.company_id AND p.invoice_id = i.id AND p.status IN ('completed', 'paid')
                ), 0)) AS amount_due,
                i.due_date,
                COALESCE(NULLIF(c.name, ''), NULLIF(i.supplier, ''), '客户')
         FROM invoices i
         LEFT JOIN clients c ON c.id = i.client_id AND c.company_id = i.company_id
         WHERE i.company_id = ?
           AND i.invoice_type = 'receivable'
           AND i.status IN ('sent', 'issued', 'overdue', 'partially_paid')
           AND i.due_date IS NOT NULL AND TRIM(i.due_date) != ''
           AND date(i.due_date) < date(?)
         ORDER BY date(i.due_date) ASC, i.amount DESC",
    )
    .bind(company_id)
    .bind(today.format("%Y-%m-%d").to_string())
    .fetch_all(pool)
    .await?;

    let mut count = 0i64;
    let mut amount = 0.0;
    let mut items = Vec::new();
    for (id, number, amount_due, due_date, client) in rows {
        if amount_due <= 0.0 {
            continue;
        }
        count += 1;
        amount += amount_due;
        if items.len() >= 5 {
            continue;
        }
        let days_left = due_date.as_deref().and_then(|date| days_until(date, today));
        let client_name = client.unwrap_or_else(|| "客户".to_string());
        items.push(urgent_item(
            format!("receivable:{}", id),
            "invoice",
            format!("逾期应收 {number} · {client_name}"),
            format!("未收回 {}，先催这一笔再接新单。", format_money(amount_due)),
            priority_for_days(days_left, 7),
            due_date,
            days_left,
            "去追款",
            "/payments",
            json!({ "amount_due": amount_due, "invoice_number": number, "client": client_name }),
        ));
    }
    Ok(OverdueReceivableSnapshot {
        count,
        amount,
        items,
    })
}

fn is_payable_invoice_type(invoice_type: &str) -> bool {
    matches!(
        invoice_type.to_lowercase().as_str(),
        "payable" | "input" | "expense" | "进项" | "进项发票"
    )
}

async fn payable_urgent_items(
    pool: &SqlitePool,
    company_id: &str,
    today: NaiveDate,
) -> Result<Vec<UrgentItem>, sqlx::Error> {
    let rows: Vec<(
        String,
        String,
        Option<String>,
        f64,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT i.id, i.invoice_number, i.invoice_type,
                MAX(0, i.amount - COALESCE((
                    SELECT SUM(p.amount) FROM payments p
                    WHERE p.company_id = i.company_id AND p.invoice_id = i.id AND p.status IN ('completed', 'paid')
                ), 0)) AS amount_due,
                i.due_date,
                i.supplier,
                i.status
         FROM invoices i
         WHERE i.company_id = ?
           AND i.status IN ('pending', 'approved', 'sent', 'overdue', 'partially_paid')
           AND i.due_date IS NOT NULL AND TRIM(i.due_date) != ''
         ORDER BY date(i.due_date) ASC, i.amount DESC",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    let mut items = Vec::new();
    for (id, number, invoice_type, amount_due, due_date, supplier, status) in rows {
        if amount_due <= 0.0 {
            continue;
        }
        if invoice_type
            .as_deref()
            .is_none_or(|kind| !is_payable_invoice_type(kind))
        {
            continue;
        }
        let days_left = due_date.as_deref().and_then(|date| days_until(date, today));
        if days_left.is_none_or(|days| days > 7) {
            continue;
        }
        if items.len() >= 5 {
            continue;
        }
        let vendor = supplier
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "供应商".to_string());
        let description = if days_left.is_some_and(|days| days < 0) {
            format!(
                "{} · 未付 {}。晚付会被加滞纳金，重的会被改成现结。",
                vendor,
                format_money(amount_due)
            )
        } else {
            format!(
                "{} · 未付 {}。这周该付，别等催单。",
                vendor,
                format_money(amount_due)
            )
        };
        items.push(urgent_item(
            format!("payable:{id}"),
            "invoice",
            format!("应付到期 {number} · {vendor}"),
            description,
            priority_for_days(days_left, 3),
            due_date,
            days_left,
            "去付款",
            "/invoices",
            json!({
                "amount_due": amount_due,
                "invoice_number": number,
                "status": status
            }),
        ));
    }
    Ok(items)
}

async fn rent_urgent_items(
    pool: &SqlitePool,
    company_id: &str,
    today: NaiveDate,
) -> Result<Vec<UrgentItem>, sqlx::Error> {
    let rows: Vec<(
        String,
        String,
        f64,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT id, location_name, COALESCE(monthly_rent, 0.0), next_payment_date,
                contract_end_date, landlord
         FROM rent_records
         WHERE company_id = ? AND COALESCE(status, 'active') = 'active'",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    let mut items = Vec::new();
    for (id, location, rent, next_payment, contract_end, landlord) in rows {
        let who = landlord
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "房东".to_string());
        if let Some(days_left) = next_payment
            .as_deref()
            .and_then(|date| days_until(date, today))
        {
            if days_left <= 7 {
                let description = if days_left < 0 {
                    format!(
                        "{who} · 月租 {}。付款日已过 {} 天，晚付常直接出滞纳金。",
                        format_money(rent),
                        days_left.abs()
                    )
                } else {
                    format!(
                        "{who} · 月租 {}。这周该付房租，别等催缴。",
                        format_money(rent)
                    )
                };
                items.push(urgent_item(
                    format!("rent-pay:{id}"),
                    "invoice",
                    format!("房租到期：{location}"),
                    description,
                    priority_for_days(Some(days_left), 3),
                    next_payment
                        .as_ref()
                        .map(|value| value.chars().take(10).collect()),
                    Some(days_left),
                    "去付租",
                    "/rent",
                    json!({ "monthly_rent": rent }),
                ));
            }
        }
        if let Some(days_left) = contract_end
            .as_deref()
            .and_then(|date| days_until(date, today))
        {
            if days_left <= 30 {
                items.push(urgent_item(
                    format!("rent-lease:{id}"),
                    "contract",
                    format!("租约到期：{location}"),
                    format!("{who}。到期当天再谈续租，容易被涨价或清场。"),
                    priority_for_days(Some(days_left), 14),
                    contract_end
                        .as_ref()
                        .map(|value| value.chars().take(10).collect()),
                    Some(days_left),
                    "去处理",
                    "/rent",
                    json!({ "monthly_rent": rent }),
                ));
            }
        }
    }
    items.sort_by_key(|item| item.days_left.unwrap_or(i64::MAX));
    items.truncate(5);
    Ok(items)
}

async fn license_urgent_items(
    pool: &SqlitePool,
    company_id: &str,
    today: NaiveDate,
) -> Result<Vec<UrgentItem>, sqlx::Error> {
    let rows: Vec<(String, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT id, license_name, expiry_date, issuing_authority
         FROM licenses
         WHERE company_id = ?
           AND COALESCE(status, 'valid') IN ('valid', 'active')
           AND expiry_date IS NOT NULL AND TRIM(expiry_date) != ''",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    let mut items = Vec::new();
    for (id, name, expiry, authority) in rows {
        let Some(days_left) = expiry.as_deref().and_then(|date| days_until(date, today)) else {
            continue;
        };
        if days_left > 30 {
            continue;
        }
        let who = authority.unwrap_or_default();
        items.push(urgent_item(
            format!("license:{id}"),
            "contract",
            format!("证照到期：{name}"),
            if who.is_empty() {
                "过期当天再补，可能直接停业。".to_string()
            } else {
                format!("发证机关 {who}。过期当天再补，可能直接停业。")
            },
            priority_for_days(Some(days_left), 14),
            expiry.map(|value| value.chars().take(10).collect()),
            Some(days_left),
            "去处理",
            "/licenses",
            json!({ "license_name": name }),
        ));
    }
    items.sort_by_key(|item| item.days_left.unwrap_or(i64::MAX));
    items.truncate(5);
    Ok(items)
}

fn insurance_type_label(insurance_type: &str) -> &'static str {
    match insurance_type {
        "workers_comp" => "工伤/雇主责任",
        "liability" => "责任险",
        "professional" => "职业责任",
        "property" => "财产险",
        "health" => "健康险",
        "cyber" => "网络责任",
        "vehicle" => "车险",
        _ => "保单",
    }
}

async fn insurance_urgent_items(
    pool: &SqlitePool,
    company_id: &str,
    today: NaiveDate,
) -> Result<Vec<UrgentItem>, sqlx::Error> {
    let rows: Vec<(
        String,
        String,
        String,
        Option<String>,
        f64,
        Option<String>,
        String,
    )> = sqlx::query_as(
        "SELECT id, policy_name, insurance_type, provider, COALESCE(premium_amount, 0.0),
                renewal_date, end_date
         FROM insurance_policies
         WHERE company_id = ?
           AND COALESCE(status, 'active') IN ('active', 'pending')
           AND end_date IS NOT NULL AND TRIM(end_date) != ''",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    let mut items = Vec::new();
    for (id, name, insurance_type, provider, premium, renewal, end_date) in rows {
        let due_raw = renewal
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(end_date.as_str());
        let Some(days_left) = days_until(due_raw, today) else {
            continue;
        };
        if days_left > 30 {
            continue;
        }
        let kind = insurance_type_label(&insurance_type);
        let who = provider.unwrap_or_default();
        let description = if days_left < 0 {
            format!(
                "{}已过期 {} 天。保险空窗一天，出事就得自己扛。",
                if who.is_empty() {
                    format!("{kind}，")
                } else {
                    format!("{who} · {kind}，")
                },
                days_left.abs()
            )
        } else {
            format!(
                "{}保费 {}。到期当天再续，可能已经没保障。",
                if who.is_empty() {
                    format!("{kind}，")
                } else {
                    format!("{who} · {kind}，")
                },
                format_money(premium)
            )
        };
        items.push(urgent_item(
            format!("insurance:{id}"),
            "contract",
            format!("保单续期：{name}"),
            description,
            priority_for_days(Some(days_left), 14),
            Some(due_raw.chars().take(10).collect()),
            Some(days_left),
            "去续保",
            "/compliance",
            json!({
                "insurance_type": insurance_type,
                "premium": premium
            }),
        ));
    }
    items.sort_by_key(|item| item.days_left.unwrap_or(i64::MAX));
    items.truncate(5);
    Ok(items)
}

async fn stale_quote_urgent_items(
    pool: &SqlitePool,
    company_id: &str,
    today: NaiveDate,
) -> Result<Vec<UrgentItem>, sqlx::Error> {
    let rows: Vec<(
        String,
        String,
        Option<String>,
        f64,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT q.id, q.quote_number, q.title, COALESCE(q.total, 0.0), q.valid_until,
                COALESCE(q.updated_at, q.created_at), c.name
         FROM quotes q
         LEFT JOIN clients c ON c.id = q.client_id AND c.company_id = q.company_id
         WHERE q.company_id = ?
           AND q.status = 'sent'
           AND (q.invoice_id IS NULL OR q.invoice_id = '')",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    let mut items = Vec::new();
    for (id, number, title, total, valid_until, touched, client) in rows {
        let until_days = valid_until
            .as_deref()
            .and_then(|date| days_until(date, today));
        let idle_days = touched
            .as_deref()
            .and_then(|date| days_until(date, today))
            .map(|days| -days);
        let expiring = until_days.is_some_and(|days| days <= 14);
        let idle = idle_days.is_some_and(|days| days >= 5);
        if !expiring && !idle {
            continue;
        }
        let client_name = client.unwrap_or_else(|| "客户".to_string());
        let headline = title
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(number.clone());
        let description = if let Some(days) = until_days {
            if days < 0 {
                format!("{client_name} 的报价已过有效期 {} 天。", days.abs())
            } else {
                format!(
                    "{client_name} · {}，{} 天后失效。发出去没跟就会丢。",
                    format_money(total),
                    days
                )
            }
        } else {
            format!(
                "{client_name} · {}。发出去已 {} 天没有下文。",
                format_money(total),
                idle_days.unwrap_or(0)
            )
        };
        items.push(urgent_item(
            format!("quote:{id}"),
            "application",
            format!("报价待跟：{headline}"),
            description,
            priority_for_days(until_days.or(Some(0)), 7),
            valid_until.map(|value| value.chars().take(10).collect()),
            until_days.or(Some(0)),
            "去跟进",
            "/quotes",
            json!({ "quote_number": number, "total": total }),
        ));
    }
    items.sort_by_key(|item| item.days_left.unwrap_or(0));
    items.truncate(5);
    Ok(items)
}

fn recurring_frequency_days(frequency: &str) -> i64 {
    match frequency.to_lowercase().as_str() {
        "weekly" => 7,
        "quarterly" => 93,
        "yearly" | "annual" => 366,
        _ => 31,
    }
}

fn frequency_label(frequency: &str) -> &'static str {
    match frequency.to_lowercase().as_str() {
        "weekly" => "每周",
        "quarterly" => "每季",
        "yearly" | "annual" => "每年",
        _ => "每月",
    }
}

async fn recurring_invoice_urgent_items(
    pool: &SqlitePool,
    company_id: &str,
    today: NaiveDate,
) -> Result<Vec<UrgentItem>, sqlx::Error> {
    let rows: Vec<(
        String,
        String,
        f64,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        i64,
    )> = sqlx::query_as(
        "SELECT r.id, r.title, COALESCE(r.amount, 0.0), r.frequency, r.next_invoice_date,
                r.last_generated_at, r.end_date, c.name, COALESCE(r.auto_send, 0)
         FROM recurring_invoices r
         LEFT JOIN clients c ON c.id = r.client_id AND c.company_id = r.company_id
         WHERE r.company_id = ? AND COALESCE(r.status, 'active') = 'active'
           AND r.next_invoice_date IS NOT NULL AND TRIM(r.next_invoice_date) != ''",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    let mut items = Vec::new();
    for (id, title, amount, frequency, next_invoice, last_generated, end_date, client, auto_send) in
        rows
    {
        if end_date
            .as_deref()
            .and_then(|date| days_until(date, today))
            .is_some_and(|days| days < 0)
        {
            continue;
        }
        let next_days = days_until(&next_invoice, today);
        let days_since_last = last_generated
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .and_then(|date| days_until(date, today))
            .map(|days| -days);
        let due_soon = next_days.is_some_and(|days| days <= 14);
        let silent_gap =
            days_since_last.is_some_and(|days| days >= recurring_frequency_days(&frequency) + 14);
        if !due_soon && !silent_gap {
            continue;
        }
        let client_name = client
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "客户".to_string());
        let cadence = frequency_label(&frequency);
        let description = if next_days.is_some_and(|days| days < 0) {
            format!(
                "{client_name} · {} · {}。该开票日已过 {} 天，客户停了往往不会打电话。",
                cadence,
                format_money(amount),
                next_days.map(|days| days.abs()).unwrap_or(0)
            )
        } else if due_soon {
            format!(
                "{client_name} · {} · {}。{}",
                cadence,
                format_money(amount),
                if auto_send != 0 {
                    "到点会自动生成，确认后再发出去。"
                } else {
                    "不会自动发出，到点要自己开。"
                }
            )
        } else {
            format!(
                "{client_name} · {} · {}。已经 {} 天没出过票，包月客户悄悄停了也看不出来。",
                cadence,
                format_money(amount),
                days_since_last.unwrap_or(0)
            )
        };
        items.push(urgent_item(
            format!("recurring:{id}"),
            "invoice",
            format!("周期发票：{title}"),
            description,
            priority_for_days(next_days.or(Some(0)), 7),
            Some(next_invoice.chars().take(10).collect()),
            next_days.or(Some(0)),
            "去开票",
            "/recurring-invoices",
            json!({ "amount": amount, "frequency": frequency, "auto_send": auto_send != 0 }),
        ));
    }
    items.sort_by_key(|item| item.days_left.unwrap_or(i64::MAX));
    items.truncate(5);
    Ok(items)
}

fn deal_stage_label(stage: &str) -> &'static str {
    match stage {
        "qualified" => "已确认",
        "proposal" => "报价中",
        "negotiation" => "谈判中",
        _ => "线索",
    }
}

async fn cold_deal_urgent_items(
    pool: &SqlitePool,
    company_id: &str,
    today: NaiveDate,
) -> Result<Vec<UrgentItem>, sqlx::Error> {
    let rows: Vec<(
        String,
        String,
        String,
        f64,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT d.id, d.title, COALESCE(d.stage, 'lead'), COALESCE(d.amount, 0.0),
                d.expected_close_date, COALESCE(d.updated_at, d.created_at), c.name
         FROM deals d
         LEFT JOIN clients c ON c.id = d.client_id AND c.company_id = d.company_id
         WHERE d.company_id = ?
           AND COALESCE(d.stage, 'lead') NOT IN ('won', 'lost')",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    let mut items = Vec::new();
    for (id, title, stage, amount, close_date, touched, client) in rows {
        let close_days = close_date
            .as_deref()
            .and_then(|date| days_until(date, today));
        let idle_days = touched
            .as_deref()
            .and_then(|date| days_until(date, today))
            .map(|days| -days);
        let closing = close_days.is_some_and(|days| days <= 14);
        let idle = idle_days.is_some_and(|days| days >= 5);
        if !(closing || idle) {
            continue;
        }
        let client_name = client
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "客户".to_string());
        let stage_label = deal_stage_label(&stage);
        let description = if close_days.is_some_and(|days| days < 0) {
            format!(
                "{client_name} · {stage_label} · {}。预计成交日已过 {} 天，再不跟这单就凉了。",
                format_money(amount),
                close_days.map(|days| days.abs()).unwrap_or(0)
            )
        } else if closing {
            format!(
                "{client_name} · {stage_label} · {}。预计 {} 天后该收口，别让它自己凉掉。",
                format_money(amount),
                close_days.unwrap_or(0)
            )
        } else {
            format!(
                "{client_name} · {stage_label} · {}。已经 {} 天没动，线索凉得比你想象的快。",
                format_money(amount),
                idle_days.unwrap_or(0)
            )
        };
        items.push(urgent_item(
            format!("deal:{id}"),
            "application",
            format!("商机待跟：{title}"),
            description,
            priority_for_days(close_days.or(Some(0)), 7),
            close_date.map(|value| value.chars().take(10).collect()),
            close_days.or(Some(0)),
            "去跟进",
            "/deals",
            json!({ "stage": stage, "amount": amount }),
        ));
    }
    items.sort_by_key(|item| item.days_left.unwrap_or(0));
    items.truncate(5);
    Ok(items)
}

fn previous_period(today: NaiveDate) -> String {
    if today.month() == 1 {
        format!("{:04}-12", today.year() - 1)
    } else {
        format!("{:04}-{:02}", today.year(), today.month() - 1)
    }
}

async fn unpaid_salary_snapshot(
    pool: &SqlitePool,
    company_id: &str,
    period: &str,
) -> Result<(i64, f64), sqlx::Error> {
    let row: (i64, f64) = sqlx::query_as(
        "SELECT COUNT(*),
                COALESCE(SUM(COALESCE(base_salary, 0.0) + COALESCE(bonus, 0.0)), 0.0)
         FROM salary_records
         WHERE company_id = ?
           AND period = ?
           AND COALESCE(status, 'pending') NOT IN ('paid', 'cancelled', 'void')",
    )
    .bind(company_id)
    .bind(period)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

async fn payroll_urgent_item(
    pool: &SqlitePool,
    company_id: &str,
    today: NaiveDate,
) -> Result<Option<UrgentItem>, sqlx::Error> {
    let last_period = previous_period(today);
    let (last_count, last_amount) = unpaid_salary_snapshot(pool, company_id, &last_period).await?;
    if last_count > 0 {
        let month_end = last_day_of_month(
            NaiveDate::from_ymd_opt(
                last_period[0..4].parse().unwrap_or(today.year()),
                last_period[5..7].parse().unwrap_or(1),
                1,
            )
            .unwrap_or(today),
        );
        let days_left = (month_end - today).num_days();
        return Ok(Some(urgent_item(
            format!("payroll:{last_period}"),
            "salary",
            format!("{last_period} 工资还没发"),
            format!(
                "{} 笔合计 {}。发不出工资，人先走，公司也停。",
                last_count,
                format_money(last_amount)
            ),
            "critical".to_string(),
            Some(month_end.to_string()),
            Some(days_left),
            "去发工资",
            "/salary",
            json!({ "period": last_period, "count": last_count, "amount": last_amount }),
        )));
    }

    if today.day() < 20 {
        return Ok(None);
    }
    let period = today.format("%Y-%m").to_string();
    let (count, amount) = unpaid_salary_snapshot(pool, company_id, &period).await?;
    if count <= 0 {
        return Ok(None);
    }
    let month_end = last_day_of_month(today);
    let days_left = i64::from(month_end.day()) - i64::from(today.day());
    Ok(Some(urgent_item(
        format!("payroll:{period}"),
        "salary",
        format!("{period} 工资还没发"),
        format!(
            "{} 笔合计 {}。月底发不出工资，比催款更伤。",
            count,
            format_money(amount)
        ),
        priority_for_days(Some(days_left), 5),
        Some(month_end.to_string()),
        Some(days_left),
        "去发工资",
        "/salary",
        json!({ "period": period, "count": count, "amount": amount }),
    )))
}

async fn bank_cash_urgent_item(
    pool: &SqlitePool,
    company_id: &str,
    today: NaiveDate,
) -> Result<Option<UrgentItem>, sqlx::Error> {
    let accounts: (i64, f64) = sqlx::query_as(
        "SELECT COUNT(*), COALESCE(SUM(COALESCE(balance, 0.0)), 0.0)
         FROM bank_accounts
         WHERE company_id = ? AND COALESCE(status, 'active') = 'active'",
    )
    .bind(company_id)
    .fetch_one(pool)
    .await?;
    let (account_count, cash) = accounts;
    if account_count <= 0 {
        return Ok(None);
    }

    let period = today.format("%Y-%m").to_string();
    let (salary_count, salary) = unpaid_salary_snapshot(pool, company_id, &period).await?;
    let last_period = previous_period(today);
    let (_, last_salary) = unpaid_salary_snapshot(pool, company_id, &last_period).await?;
    let (rent,): (f64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(COALESCE(monthly_rent, 0.0)), 0.0)
         FROM rent_records
         WHERE company_id = ? AND COALESCE(status, 'active') = 'active'
           AND next_payment_date IS NOT NULL AND TRIM(next_payment_date) != ''
           AND date(next_payment_date) <= date(?, '+14 days')",
    )
    .bind(company_id)
    .bind(today.format("%Y-%m-%d").to_string())
    .fetch_one(pool)
    .await?;
    let (payables,): (f64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(MAX(0, i.amount - COALESCE((
                    SELECT SUM(p.amount) FROM payments p
                    WHERE p.company_id = i.company_id AND p.invoice_id = i.id AND p.status IN ('completed', 'paid')
                ), 0))), 0.0)
         FROM invoices i
         WHERE i.company_id = ?
           AND LOWER(COALESCE(i.invoice_type, '')) IN ('payable', 'input', 'expense', '进项', '进项发票')
           AND i.status IN ('pending', 'approved', 'sent', 'overdue', 'partially_paid')
           AND i.due_date IS NOT NULL AND TRIM(i.due_date) != ''
           AND date(i.due_date) <= date(?, '+14 days')",
    )
    .bind(company_id)
    .bind(today.format("%Y-%m-%d").to_string())
    .fetch_one(pool)
    .await?;

    let must_pay = salary + last_salary + rent + payables;
    if must_pay <= 0.0 || cash + 0.005 >= must_pay {
        return Ok(None);
    }
    let _ = salary_count;
    Ok(Some(urgent_item(
        "bank-cash".to_string(),
        "tax",
        "账上现金不够付近两周的硬支出".to_string(),
        format!(
            "银行 {}，近两周工资/房租/应付 {}。账上赚钱不等于账上有钱。",
            format_money(cash),
            format_money(must_pay)
        ),
        "critical".to_string(),
        Some(today.format("%Y-%m-%d").to_string()),
        Some(0),
        "看账户",
        "/bank-accounts",
        json!({
            "cash": cash,
            "must_pay": must_pay,
            "salary": salary + last_salary,
            "rent": rent,
            "payables": payables
        }),
    )))
}

struct MonthCashSnapshot {
    inflow: f64,
    outflow: f64,
    net: f64,
    stressed: bool,
    urgent: Option<UrgentItem>,
}

async fn month_due_amount(
    pool: &SqlitePool,
    company_id: &str,
    period: &str,
    receivable: bool,
) -> Result<f64, sqlx::Error> {
    let type_clause = if receivable {
        "i.invoice_type = 'receivable'"
    } else {
        "COALESCE(i.invoice_type, '') != 'receivable'"
    };
    let sql = format!(
        "SELECT COALESCE(SUM(CASE
            WHEN i.amount - COALESCE((
                SELECT SUM(p.amount) FROM payments p
                WHERE p.company_id = i.company_id AND p.invoice_id = i.id AND p.status IN ('completed', 'paid')
            ), 0) > 0
            THEN i.amount - COALESCE((
                SELECT SUM(p.amount) FROM payments p
                WHERE p.company_id = i.company_id AND p.invoice_id = i.id AND p.status IN ('completed', 'paid')
            ), 0)
            ELSE 0 END), 0.0)
         FROM invoices i
         WHERE i.company_id = ?
           AND {type_clause}
           AND i.status NOT IN ('paid', 'cancelled', 'void', 'draft')
           AND substr(COALESCE(i.due_date, i.invoice_date, ''), 1, 7) = ?"
    );
    let (value,): (f64,) = sqlx::query_as(&sql)
        .bind(company_id)
        .bind(period)
        .fetch_one(pool)
        .await?;
    Ok(value)
}

async fn month_cash_snapshot(
    pool: &SqlitePool,
    company_id: &str,
    today: NaiveDate,
) -> Result<MonthCashSnapshot, sqlx::Error> {
    let period = today.format("%Y-%m").to_string();
    let salary = monthly_salary_expense(pool, company_id, &period).await?;
    let (rent,): (f64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(COALESCE(monthly_rent, 0.0)), 0.0)
         FROM rent_records WHERE company_id = ? AND status = 'active'",
    )
    .bind(company_id)
    .fetch_one(pool)
    .await?;
    let (virtual_assets,): (f64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(COALESCE(current_value, 0.0)), 0.0)
         FROM assets WHERE company_id = ? AND asset_type = 'virtual'",
    )
    .bind(company_id)
    .fetch_one(pool)
    .await?;
    let subscription = virtual_assets / 12.0;
    let receivables = month_due_amount(pool, company_id, &period, true).await?;
    let payables = month_due_amount(pool, company_id, &period, false).await?;
    let inflow = receivables;
    let outflow = salary + rent + subscription + payables;
    let net = inflow - outflow;
    let stressed = net < -0.005 && outflow > 0.0;
    let urgent = if stressed {
        Some(urgent_item(
            format!("cash-gap:{period}"),
            "tax",
            "本月现金可能不够覆盖工资和房租".to_string(),
            format!(
                "预计进 {}、出 {}，净额 {}。先催应收或推迟非必要支出。",
                format_money(inflow),
                format_money(outflow),
                format_money(net)
            ),
            "critical".to_string(),
            Some(last_day_of_month(today).to_string()),
            Some(i64::from(last_day_of_month(today).day()) - i64::from(today.day())),
            "看现金流",
            "/cash-flow-forecast",
            json!({
                "inflow": inflow,
                "outflow": outflow,
                "net": net,
                "salary": salary,
                "rent": rent
            }),
        ))
    } else {
        None
    };
    Ok(MonthCashSnapshot {
        inflow,
        outflow,
        net,
        stressed,
        urgent,
    })
}

fn clamp_date(year: i32, month: u32, day: u32) -> Option<NaiveDate> {
    let first = NaiveDate::from_ymd_opt(year, month, 1)?;
    let last = last_day_of_month(first).day();
    NaiveDate::from_ymd_opt(year, month, day.min(last))
}

fn next_tax_deadline(
    today: NaiveDate,
    month: i64,
    day: i64,
    recurrence: &str,
) -> Option<NaiveDate> {
    let month = u32::try_from(month)
        .ok()
        .filter(|value| (1..=12).contains(value))?;
    let day = u32::try_from(day)
        .ok()
        .filter(|value| (1..=31).contains(value))?;
    if recurrence == "monthly" {
        let this_month = clamp_date(today.year(), today.month(), day)?;
        if this_month >= today {
            return Some(this_month);
        }
        return if today.month() == 12 {
            clamp_date(today.year() + 1, 1, day)
        } else {
            clamp_date(today.year(), today.month() + 1, day)
        };
    }
    let this_year = clamp_date(today.year(), month, day)?;
    if this_year >= today {
        Some(this_year)
    } else {
        clamp_date(today.year() + 1, month, day)
    }
}

async fn tax_deadline_urgent_items(
    pool: &SqlitePool,
    company_id: &str,
    today: NaiveDate,
) -> Result<Vec<UrgentItem>, sqlx::Error> {
    let rows: Vec<(
        String,
        String,
        Option<String>,
        i64,
        i64,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT id, title, title_local, deadline_month, deadline_day, recurrence, penalty_info
         FROM tax_deadlines
         WHERE company_id = ? AND COALESCE(is_active, 1) = 1",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    let mut items: Vec<UrgentItem> = rows
        .into_iter()
        .filter_map(
            |(id, title, title_local, month, day, recurrence, penalty)| {
                let due = next_tax_deadline(
                    today,
                    month,
                    day,
                    recurrence.as_deref().unwrap_or("yearly"),
                )?;
                let days_left = (due - today).num_days();
                if days_left > 14 || days_left < -2 {
                    return None;
                }
                let label = title_local
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or(title);
                let description = penalty
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| "错过申报会产生滞纳金，比催款更贵。".to_string());
                Some(urgent_item(
                    format!("tax-deadline:{id}"),
                    "tax",
                    format!("税务截止：{label}"),
                    description,
                    priority_for_days(Some(days_left), 7),
                    Some(due.to_string()),
                    Some(days_left),
                    "看日历",
                    "/tax-calendar",
                    json!({ "deadline": due.to_string() }),
                ))
            },
        )
        .collect();
    items.sort_by_key(|item| item.days_left.unwrap_or(i64::MAX));
    items.truncate(5);
    Ok(items)
}

async fn mailbox_inbox_urgent_item(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<Option<UrgentItem>, sqlx::Error> {
    let pending = count(
        pool,
        "SELECT COUNT(*) FROM invoice_mail_imports WHERE company_id = ? AND status = 'pending_confirmation'",
        company_id,
    )
    .await
    .unwrap_or(0);
    if pending <= 0 {
        return Ok(None);
    }
    Ok(Some(urgent_item(
        "mailbox-inbox".to_string(),
        "invoice",
        format!("邮箱里有 {pending} 张发票待确认"),
        "确认后才进台账。堆到月底会和关账抢同一天。".to_string(),
        if pending >= 5 {
            "high".to_string()
        } else {
            "medium".to_string()
        },
        None,
        Some(0),
        "去确认",
        "/settings",
        json!({ "pending_confirmation": pending }),
    )))
}

async fn subscription_urgent_items(
    pool: &SqlitePool,
    company_id: &str,
    today: NaiveDate,
) -> Result<Vec<UrgentItem>, sqlx::Error> {
    let software: Vec<(String, String, Option<String>, String, f64, i64)> = sqlx::query_as(
        "SELECT id, software_name, vendor, next_billing_date, COALESCE(total_price, 0.0), COALESCE(auto_renewal, 1)
         FROM software_subscriptions
         WHERE company_id = ? AND COALESCE(status, 'active') = 'active'
           AND next_billing_date IS NOT NULL AND TRIM(next_billing_date) != ''",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    let mut items = Vec::new();
    for (id, name, vendor, next_billing, price, auto_renewal) in software {
        let Some(days_left) = days_until(&next_billing, today) else {
            continue;
        };
        if days_left > 14 || days_left < -2 {
            continue;
        }
        let vendor_name = vendor.unwrap_or_default();
        let renew = if auto_renewal != 0 {
            "会自动续费扣款"
        } else {
            "不会自动续，过期会停"
        };
        items.push(urgent_item(
            format!("subscription:{id}"),
            "asset",
            format!("订阅续费：{name}"),
            format!(
                "{}{}。{}",
                if vendor_name.is_empty() {
                    String::new()
                } else {
                    format!("{vendor_name}，")
                },
                format_money(price),
                renew
            ),
            priority_for_days(Some(days_left), 7),
            Some(next_billing.chars().take(10).collect()),
            Some(days_left),
            "去检查",
            "/cloud-cost-management",
            json!({ "price": price, "auto_renewal": auto_renewal != 0 }),
        ));
    }

    let domains: Vec<(String, String, String, f64, i64)> = sqlx::query_as(
        "SELECT id, domain_name, expiry_date, COALESCE(annual_fee, 0.0), COALESCE(auto_renewal, 0)
         FROM domain_subscriptions
         WHERE company_id = ? AND COALESCE(status, 'active') = 'active'
           AND expiry_date IS NOT NULL AND TRIM(expiry_date) != ''",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;
    for (id, name, expiry, fee, auto_renewal) in domains {
        let Some(days_left) = days_until(&expiry, today) else {
            continue;
        };
        if days_left > 14 || days_left < -2 {
            continue;
        }
        items.push(urgent_item(
            format!("domain:{id}"),
            "asset",
            format!("域名到期：{name}"),
            format!(
                "年费 {}。{}",
                format_money(fee),
                if auto_renewal != 0 {
                    "已开自动续费。"
                } else {
                    "未开自动续费，过期域名会被人抢注。"
                }
            ),
            priority_for_days(Some(days_left), 7),
            Some(expiry.chars().take(10).collect()),
            Some(days_left),
            "去检查",
            "/cloud-cost-management",
            json!({ "annual_fee": fee }),
        ));
    }
    items.sort_by_key(|item| item.days_left.unwrap_or(i64::MAX));
    items.truncate(5);
    Ok(items)
}

async fn unbilled_time_urgent_item(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<Option<UrgentItem>, sqlx::Error> {
    let row: Option<(i64, f64)> = sqlx::query_as(
        "SELECT COUNT(*), COALESCE(SUM(COALESCE(amount, 0.0)), 0.0)
         FROM time_entries
         WHERE company_id = ?
           AND COALESCE(is_billable, 0) = 1
           AND COALESCE(status, 'logged') != 'invoiced'
           AND (invoice_id IS NULL OR invoice_id = '')",
    )
    .bind(company_id)
    .fetch_optional(pool)
    .await?;
    let Some((count, amount)) = row else {
        return Ok(None);
    };
    if count <= 0 {
        return Ok(None);
    }
    Ok(Some(urgent_item(
        "unbilled-time".to_string(),
        "invoice",
        format!("有 {count} 条工时还没开票"),
        format!(
            "合计 {}。记了工时却没转发票，等于白做。",
            format_money(amount)
        ),
        if count >= 8 || amount >= 3000.0 {
            "high".to_string()
        } else {
            "medium".to_string()
        },
        None,
        Some(0),
        "去开票",
        "/time-tracking",
        json!({ "unbilled_count": count, "unbilled_amount": amount }),
    )))
}

async fn monthly_close_urgent_item(
    pool: &SqlitePool,
    company_id: &str,
    today: NaiveDate,
) -> Result<Option<UrgentItem>, sqlx::Error> {
    if today.day() < 25 {
        return Ok(None);
    }
    let activity = count(
        pool,
        "SELECT COUNT(*) FROM invoices WHERE company_id = ?",
        company_id,
    )
    .await?;
    if activity == 0 {
        return Ok(None);
    }
    let period = today.format("%Y-%m").to_string();
    let archived: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM files
         WHERE company_id = ? AND entity_type = 'monthly_report'
           AND original_name LIKE ?",
    )
    .bind(company_id)
    .bind(format!("%{period}%"))
    .fetch_one(pool)
    .await
    .unwrap_or(0);
    if archived > 0 {
        return Ok(None);
    }
    let month_end = last_day_of_month(today);
    let days_left = i64::from(month_end.day()) - i64::from(today.day());
    Ok(Some(urgent_item(
        format!("monthly-close:{period}"),
        "tax",
        format!("{period} 月度关账"),
        "月底先把收支和应收对上，比事后翻账本省几天。".to_string(),
        priority_for_days(Some(days_left), 3),
        Some(month_end.to_string()),
        Some(days_left),
        "去关账",
        "/financial-reports",
        json!({ "period": period }),
    )))
}

async fn contract_urgent_items(
    pool: &SqlitePool,
    company_id: &str,
    today: NaiveDate,
) -> Result<Vec<UrgentItem>, sqlx::Error> {
    let rows: Vec<(String, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT id, contract_name, counterparty, end_date
         FROM contracts
         WHERE company_id = ? AND status = 'active' AND end_date IS NOT NULL",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|(id, name, counterparty, end_date)| {
            let days_left = end_date
                .as_deref()
                .and_then(|date| days_until(date, today))?;
            if days_left > 30 {
                return None;
            }

            let description = counterparty
                .map(|value| format!("交易对手: {}", value))
                .unwrap_or_else(|| "请确认续约、终止或重签安排。".to_string());

            Some(urgent_item(
                format!("contract:{}", id),
                "contract",
                format!("合同即将到期: {}", name),
                description,
                priority_for_days(Some(days_left), 7),
                end_date,
                Some(days_left),
                "查看",
                "/contracts",
                json!({ "contract_name": name }),
            ))
        })
        .collect())
}

async fn asset_urgent_items(
    pool: &SqlitePool,
    company_id: &str,
    today: NaiveDate,
) -> Result<Vec<UrgentItem>, sqlx::Error> {
    let rows: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, name, asset_type, expiry_date
         FROM assets
         WHERE company_id = ?
           AND status = 'active'
           AND asset_type IN ('virtual', 'data', 'intangible')
           AND expiry_date IS NOT NULL",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|(id, name, asset_type, expiry_date)| {
            let days_left = expiry_date
                .as_deref()
                .and_then(|date| days_until(date, today))?;
            if days_left > 30 {
                return None;
            }

            Some(urgent_item(
                format!("asset:{}", id),
                "asset",
                format!("资产权益即将到期: {}", name),
                "请确认续费、续展或替代安排。".to_string(),
                priority_for_days(Some(days_left), 7),
                expiry_date,
                Some(days_left),
                "续展",
                "/assets",
                json!({ "asset_name": name, "asset_type": asset_type }),
            ))
        })
        .collect())
}

async fn application_urgent_items(
    pool: &SqlitePool,
    company_id: &str,
    today: NaiveDate,
) -> Result<Vec<UrgentItem>, sqlx::Error> {
    let rows: Vec<(String, String, String, Option<String>, i64)> = sqlx::query_as(
        "SELECT id, title, status, deadline, COALESCE(progress, 0)
         FROM applications
         WHERE company_id = ?
           AND status NOT IN ('approved', 'rejected', 'completed')
           AND deadline IS NOT NULL",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|(id, title, status, deadline, progress)| {
            let days_left = deadline
                .as_deref()
                .and_then(|date| days_until(date, today))?;
            if days_left > 14 {
                return None;
            }

            Some(urgent_item(
                format!("application:{}", id),
                "application",
                format!("申报即将截止: {}", title),
                format!("当前状态 {}，完成度 {}%。", status, progress),
                priority_for_days(Some(days_left), 3),
                deadline,
                Some(days_left),
                "继续",
                "/applications",
                json!({ "application_title": title, "status": status, "progress": progress }),
            ))
        })
        .collect())
}

async fn suggestions(
    pool: &SqlitePool,
    company_id: &str,
    pending_invoices: i64,
    expiring_assets: i64,
    overdue_receivables: i64,
    overdue_amount: f64,
    month_cash_net: f64,
    month_cash_stressed: bool,
    urgent_items: &[UrgentItem],
) -> Result<Vec<String>, sqlx::Error> {
    let low_progress_applications = count(
        pool,
        "SELECT COUNT(*) FROM applications WHERE company_id = ? AND status = 'preparing' AND COALESCE(progress, 0) < 50",
        company_id,
    )
    .await?;

    let mut suggestions = Vec::new();
    if month_cash_stressed {
        suggestions.push(format!(
            "本月净现金 {}。账上赚钱不等于账上有钱，先看工资和房租能不能覆盖。",
            format_money(month_cash_net)
        ));
    }
    let tax_due = urgent_items
        .iter()
        .filter(|item| item.id.starts_with("tax-deadline:"))
        .count();
    if tax_due > 0 {
        suggestions.push(format!(
            "有 {tax_due} 个税务截止日临近。错过申报会被罚，比催款更贵。"
        ));
    }
    if urgent_items.iter().any(|item| item.id == "mailbox-inbox") {
        suggestions.push("邮箱里还有发票待确认，确认后才进台账，别堆到月底。".to_string());
    }
    if urgent_items
        .iter()
        .any(|item| item.id.starts_with("subscription:") || item.id.starts_with("domain:"))
    {
        suggestions
            .push("有订阅或域名快到期。自动续费会静默扣款，不用的席位现在取消最省钱。".to_string());
    }
    if urgent_items.iter().any(|item| item.id == "unbilled-time") {
        suggestions.push("有已记录、未开票的工时。不转发票就等于免费加班。".to_string());
    }
    if overdue_receivables > 0 {
        suggestions.push(format!(
            "有 {} 笔逾期应收共 {}，先催回再接新单，避免用自己的现金给客户垫款。",
            overdue_receivables,
            format_money(overdue_amount)
        ));
    }
    if urgent_items
        .iter()
        .any(|item| item.id.starts_with("payable:"))
    {
        suggestions
            .push("有供应商账单要到期或已经过期。晚付会被加滞纳金，再晚就被改成现结。".to_string());
    }
    if urgent_items.iter().any(|item| item.id.starts_with("rent-")) {
        suggestions.push("有房租或租约临近。晚付滞纳金，到期再谈续租容易被涨价。".to_string());
    }
    if urgent_items
        .iter()
        .any(|item| item.id.starts_with("license:"))
    {
        suggestions.push("有证照快到期或已过期。过期当天再补，业务可能直接停摆。".to_string());
    }
    if urgent_items
        .iter()
        .any(|item| item.id.starts_with("insurance:"))
    {
        suggestions.push("有保单即将到期或已过期。保险空窗一天，出事就得自己扛。".to_string());
    }
    if urgent_items
        .iter()
        .any(|item| item.id.starts_with("payroll:"))
    {
        suggestions.push("有工资还没发。发不出工资，人先走，公司也停。".to_string());
    }
    if urgent_items.iter().any(|item| item.id == "bank-cash") {
        suggestions
            .push("账上现金不够付近两周的工资、房租或应付。先催收回来或砍非必要支出。".to_string());
    }
    if urgent_items
        .iter()
        .any(|item| item.id.starts_with("quote:"))
    {
        suggestions
            .push("有发出去的报价还没跟。发出去那天没写下一次跟进日，这单基本就丢了。".to_string());
    }
    if urgent_items
        .iter()
        .any(|item| item.id.starts_with("recurring:"))
    {
        suggestions.push(
            "有包月/周期发票该开或已经中断。客户停了不会通知你，晚两个月才发现等于白做。"
                .to_string(),
        );
    }
    if urgent_items.iter().any(|item| item.id.starts_with("deal:")) {
        suggestions.push(
            "有销售机会几天没动或过了预计成交日。线索凉掉通常不是没单，是忘了跟。".to_string(),
        );
    }
    if pending_invoices > 0 {
        suggestions.push(format!(
            "您有 {} 张发票待处理，建议按到期日优先审批。",
            pending_invoices
        ));
    }
    if expiring_assets > 0 {
        suggestions.push(format!(
            "有 {} 项数字/无形资产即将到期，建议提前安排续展。",
            expiring_assets
        ));
    }
    if low_progress_applications > 0 {
        suggestions.push(format!(
            "有 {} 个申报项目进度低于 50%，建议补齐材料并明确负责人。",
            low_progress_applications
        ));
    }
    let high_priority = urgent_items
        .iter()
        .filter(|item| item.priority == "high" || item.priority == "critical")
        .count();
    if high_priority > 0 {
        suggestions.push(format!(
            "当前有 {} 个高优先级事项，请优先处理。",
            high_priority
        ));
    }

    Ok(suggestions)
}

async fn build_completeness(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<CompletenessResponse, sqlx::Error> {
    let company = {
        let (value,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM companies WHERE id = ?")
            .bind(company_id)
            .fetch_one(pool)
            .await?;
        value
    };

    let items = vec![
        completeness_item(
            "company",
            "企业资料",
            "Building2",
            "/settings/company",
            company,
            "企业基础资料是自动化运维、合同、财税和审计流程的基础。",
        ),
        completeness_item(
            "employees",
            "员工档案",
            "Users",
            "/employees",
            count(
                pool,
                "SELECT COUNT(*) FROM employees WHERE company_id = ?",
                company_id,
            )
            .await?,
            "补齐员工档案后，薪酬、社保和权限流程可以由后端统一编排。",
        ),
        completeness_item(
            "invoices",
            "发票台账",
            "ReceiptText",
            "/invoices",
            count(
                pool,
                "SELECT COUNT(*) FROM invoices WHERE company_id = ?",
                company_id,
            )
            .await?,
            "发票数据驱动预算、现金流、付款和税务提醒。",
        ),
        completeness_item(
            "contracts",
            "合同台账",
            "FileText",
            "/contracts",
            count(
                pool,
                "SELECT COUNT(*) FROM contracts WHERE company_id = ?",
                company_id,
            )
            .await?,
            "合同数据驱动续约、付款、收入确认和风险审查。",
        ),
        completeness_item(
            "assets",
            "资产台账",
            "Archive",
            "/assets",
            count(
                pool,
                "SELECT COUNT(*) FROM assets WHERE company_id = ?",
                company_id,
            )
            .await?,
            "资产数据驱动折旧、续费、权属和研发费用归集。",
        ),
        completeness_item(
            "projects",
            "项目管理",
            "Briefcase",
            "/projects",
            count(
                pool,
                "SELECT COUNT(*) FROM projects WHERE company_id = ?",
                company_id,
            )
            .await?,
            "项目数据驱动预算执行、申报、研发费用和里程碑跟踪。",
        ),
        completeness_item(
            "bank",
            "银行账户",
            "Landmark",
            "/bank",
            count(
                pool,
                "SELECT COUNT(*) FROM bank_accounts WHERE company_id = ?",
                company_id,
            )
            .await?,
            "银行账户数据驱动付款核销、现金流和对账。",
        ),
        completeness_item(
            "salary",
            "薪酬记录",
            "WalletCards",
            "/salary",
            count(
                pool,
                "SELECT COUNT(*) FROM salary_records WHERE company_id = ?",
                company_id,
            )
            .await?,
            "薪酬记录驱动月度费用、个税和员工成本分析。",
        ),
        completeness_item(
            "tax",
            "税务记录",
            "BadgePercent",
            "/tax",
            count(
                pool,
                "SELECT COUNT(*) FROM tax_records WHERE company_id = ?",
                company_id,
            )
            .await?,
            "税务记录驱动申报、缴款和合规预警。",
        ),
        completeness_item(
            "applications",
            "项目申报",
            "ClipboardCheck",
            "/applications",
            count(
                pool,
                "SELECT COUNT(*) FROM applications WHERE company_id = ?",
                company_id,
            )
            .await?,
            "申报数据驱动政策匹配、材料复用和截止期提醒。",
        ),
    ];

    let total: i64 = items.iter().map(|item| item.percentage).sum();
    let overall_score = if items.is_empty() {
        0
    } else {
        total / items.len() as i64
    };
    let completed_items: Vec<CompletenessItem> = items
        .iter()
        .filter(|item| item.percentage >= 100)
        .cloned()
        .collect();
    let incomplete_items: Vec<CompletenessItem> = items
        .iter()
        .filter(|item| item.percentage < 100)
        .cloned()
        .collect();
    let top_incomplete_items: Vec<CompletenessItem> =
        incomplete_items.iter().take(3).cloned().collect();
    let completed_count = completed_items.len() as i64;
    let incomplete_count = incomplete_items.len() as i64;
    let total_count = items.len() as i64;
    let remaining_overflow_count = (incomplete_count - top_incomplete_items.len() as i64).max(0);
    let is_complete = incomplete_count == 0 && total_count > 0;
    let overall_score_label = format!("{}%", overall_score);
    let overall_gradient_class = if overall_score >= 80 {
        "from-green-500 to-emerald-500"
    } else if overall_score >= 50 {
        "from-yellow-500 to-orange-500"
    } else {
        "from-orange-500 to-red-500"
    }
    .to_string();
    let completion_badge_label = if is_complete {
        format!("{} 完成", overall_score_label)
    } else {
        overall_score_label.clone()
    };
    let remaining_overflow_label = if remaining_overflow_count > 0 {
        Some(format!("还有 {} 个模块待完善...", remaining_overflow_count))
    } else {
        None
    };
    let status_title = if is_complete {
        "系统数据已完善".to_string()
    } else {
        "快速完善系统数据".to_string()
    };
    let status_description = if is_complete {
        "所有核心模块均已达到后端完整度要求".to_string()
    } else {
        format!(
            "还有 {} 个模块待完善，后端会据此解锁更多自动化能力",
            incomplete_count
        )
    };

    Ok(CompletenessResponse {
        overall_score,
        overall_score_label,
        overall_gradient_class,
        unlock_title: "完善数据可解锁".to_string(),
        unlock_description: "智能分析报告 · 成本优化建议 · 风险预警提醒".to_string(),
        completed_count,
        incomplete_count,
        total_count,
        completed_label: format!("已完成 {}/{} 个模块", completed_count, total_count),
        incomplete_label: format!("还有 {} 个模块待完善", incomplete_count),
        is_complete,
        status_title,
        status_description,
        completion_badge_label,
        remaining_overflow_count,
        remaining_overflow_label,
        items,
        completed_items,
        incomplete_items,
        top_incomplete_items,
    })
}

fn completeness_item(
    key: &str,
    label: &str,
    icon: &str,
    path: &str,
    count: i64,
    description: &str,
) -> CompletenessItem {
    let score = if count > 0 { 1 } else { 0 };
    let missing_items = if score == 0 {
        vec![format!("缺少{}", label)]
    } else {
        Vec::new()
    };

    CompletenessItem {
        key: key.to_string(),
        label: label.to_string(),
        icon: icon.to_string(),
        path: path.to_string(),
        score,
        max_score: 1,
        percentage: score * 100,
        percentage_label: format!("{}%", score * 100),
        missing_items,
        description: description.to_string(),
    }
}

fn build_today_tasks(urgent_items: &[UrgentItem]) -> Vec<TodayTask> {
    urgent_items
        .iter()
        .filter(|item| item.days_left.unwrap_or(i64::MAX) <= 1)
        .take(5)
        .map(|item| TodayTask {
            id: item.id.clone(),
            title: item.title.clone(),
            priority: item.priority.clone(),
            due_date: item.due_date.clone(),
            source_type: item.item_type.clone(),
            status: "pending".to_string(),
        })
        .collect()
}

fn build_todos(urgent_items: &[UrgentItem]) -> Vec<TodoItem> {
    urgent_items
        .iter()
        .take(5)
        .map(|item| TodoItem {
            id: item.id.clone(),
            title: item.title.clone(),
            description: item.description.clone(),
            category: category_label(&item.item_type).to_string(),
            priority: item.priority.clone(),
            due_date: item.due_date.clone(),
            completed: false,
            path: item.path.clone(),
        })
        .collect()
}

fn completion_rate(todos: &[TodoItem]) -> i64 {
    if todos.is_empty() {
        return 0;
    }

    let completed = todos.iter().filter(|item| item.completed).count() as i64;
    completed * 100 / todos.len() as i64
}

fn overview_cards(
    stats: &WorkbenchStats,
    monthly_expense_trend: &[MonthlyExpensePoint],
    operating_control: &OperatingControl,
) -> Vec<WorkbenchOverviewCard> {
    let (trend_label, trend_up) = expense_trend(monthly_expense_trend)
        .map(|(label, is_up)| (Some(label), Some(is_up)))
        .unwrap_or((None, None));
    let trend_variant = trend_up.map(overview_trend_variant).map(str::to_string);

    let mut cards = vec![
        WorkbenchOverviewCard {
            key: "overdue_receivables".to_string(),
            icon: "DollarSign".to_string(),
            title: "逾期应收".to_string(),
            value_label: stats.overdue_receivable_amount_label.clone(),
            subtitle: Some(stats.overdue_receivables_label.clone()),
            trend_label: None,
            trend_up: None,
            trend_variant: None,
            color: "orange".to_string(),
            class_name: overview_card_class_name("orange").to_string(),
        },
        WorkbenchOverviewCard {
            key: "month_cash_net".to_string(),
            icon: "TrendingUp".to_string(),
            title: "本月净现金".to_string(),
            value_label: stats.month_cash_net_label.clone(),
            subtitle: Some(format!(
                "进 {} / 出 {}",
                stats.month_cash_inflow_label, stats.month_cash_outflow_label
            )),
            trend_label: None,
            trend_up: Some(!stats.month_cash_stressed),
            trend_variant: Some(
                if stats.month_cash_stressed {
                    "secondary"
                } else {
                    "default"
                }
                .to_string(),
            ),
            color: if stats.month_cash_stressed {
                "orange".to_string()
            } else {
                "green".to_string()
            },
            class_name: overview_card_class_name(if stats.month_cash_stressed {
                "orange"
            } else {
                "green"
            })
            .to_string(),
        },
        WorkbenchOverviewCard {
            key: "monthly_expense".to_string(),
            icon: "TrendingUp".to_string(),
            title: "本月支出".to_string(),
            value_label: stats.monthly_expense_label.clone(),
            subtitle: None,
            trend_label,
            trend_up,
            trend_variant,
            color: "blue".to_string(),
            class_name: overview_card_class_name("blue").to_string(),
        },
        WorkbenchOverviewCard {
            key: "ongoing_projects".to_string(),
            icon: "FileText".to_string(),
            title: "进行中项目".to_string(),
            value_label: stats.ongoing_projects_label.clone(),
            subtitle: Some("个项目".to_string()),
            trend_label: None,
            trend_up: None,
            trend_variant: None,
            color: "purple".to_string(),
            class_name: overview_card_class_name("purple").to_string(),
        },
        WorkbenchOverviewCard {
            key: "active_contracts".to_string(),
            icon: "Building2".to_string(),
            title: "活跃合同".to_string(),
            value_label: stats.active_contracts_label.clone(),
            subtitle: Some("份合同".to_string()),
            trend_label: None,
            trend_up: None,
            trend_variant: None,
            color: "green".to_string(),
            class_name: overview_card_class_name("green").to_string(),
        },
        WorkbenchOverviewCard {
            key: "employees".to_string(),
            icon: "Users".to_string(),
            title: "团队成员".to_string(),
            value_label: stats.employees_label.clone(),
            subtitle: Some("人".to_string()),
            trend_label: None,
            trend_up: None,
            trend_variant: None,
            color: "orange".to_string(),
            class_name: overview_card_class_name("orange").to_string(),
        },
    ];
    cards.push(WorkbenchOverviewCard {
        key: "operating_control".to_string(),
        icon: "Target".to_string(),
        title: operating_control.title.clone(),
        value_label: operating_control.backbone_rate_label.clone(),
        subtitle: Some(operating_control.status_label.clone()),
        trend_label: Some(operating_control.risk_count_label.clone()),
        trend_up: Some(operating_control.critical_count == 0),
        trend_variant: Some(if operating_control.critical_count == 0 {
            "default".to_string()
        } else {
            "secondary".to_string()
        }),
        color: "orange".to_string(),
        class_name: overview_card_class_name("orange").to_string(),
    });
    cards
}

fn overview_trend_variant(is_up: bool) -> &'static str {
    if is_up { "default" } else { "secondary" }
}

fn overview_card_class_name(color: &str) -> &'static str {
    match color {
        "green" => "from-green-500/10 to-green-500/5 text-green-500",
        "purple" => "from-purple-500/10 to-purple-500/5 text-purple-500",
        "orange" => "from-orange-500/10 to-orange-500/5 text-orange-500",
        _ => "from-blue-500/10 to-blue-500/5 text-blue-500",
    }
}

fn quick_actions() -> Vec<WorkbenchQuickAction> {
    vec![
        WorkbenchQuickAction {
            icon: "DollarSign".to_string(),
            label: "应收追款".to_string(),
            description: "看谁欠你钱，按账龄催".to_string(),
            path: "/payments".to_string(),
            color: "from-orange-500 to-orange-600".to_string(),
        },
        WorkbenchQuickAction {
            icon: "FileText".to_string(),
            label: "应付账单".to_string(),
            description: "供应商账期别拖到现结".to_string(),
            path: "/invoices".to_string(),
            color: "from-blue-500 to-blue-600".to_string(),
        },
        WorkbenchQuickAction {
            icon: "TrendingUp".to_string(),
            label: "现金流".to_string(),
            description: "本月工资房租能不能盖住".to_string(),
            path: "/cash-flow-forecast".to_string(),
            color: "from-blue-500 to-blue-600".to_string(),
        },
        WorkbenchQuickAction {
            icon: "DollarSign".to_string(),
            label: "税务日历".to_string(),
            description: "别错过申报和预缴".to_string(),
            path: "/tax-calendar".to_string(),
            color: "from-purple-500 to-purple-600".to_string(),
        },
        WorkbenchQuickAction {
            icon: "FileText".to_string(),
            label: "证照年检".to_string(),
            description: "过期可能直接停业".to_string(),
            path: "/licenses".to_string(),
            color: "from-orange-500 to-orange-600".to_string(),
        },
        WorkbenchQuickAction {
            icon: "Users".to_string(),
            label: "工时开票".to_string(),
            description: "把未开票工时转成发票".to_string(),
            path: "/time-tracking".to_string(),
            color: "from-green-500 to-green-600".to_string(),
        },
        WorkbenchQuickAction {
            icon: "Target".to_string(),
            label: "报价跟进".to_string(),
            description: "发出去的报价别石沉大海".to_string(),
            path: "/quotes".to_string(),
            color: "from-purple-500 to-purple-600".to_string(),
        },
        WorkbenchQuickAction {
            icon: "Target".to_string(),
            label: "销售机会".to_string(),
            description: "凉掉的商机先捞回来".to_string(),
            path: "/deals".to_string(),
            color: "from-green-500 to-green-600".to_string(),
        },
        WorkbenchQuickAction {
            icon: "FileText".to_string(),
            label: "周期发票".to_string(),
            description: "包月客户别漏开、别漏看".to_string(),
            path: "/recurring-invoices".to_string(),
            color: "from-orange-500 to-orange-600".to_string(),
        },
        WorkbenchQuickAction {
            icon: "FileText".to_string(),
            label: "新建发票".to_string(),
            description: "录入进项/销项发票".to_string(),
            path: "/invoices".to_string(),
            color: "from-blue-500 to-blue-600".to_string(),
        },
        WorkbenchQuickAction {
            icon: "DollarSign".to_string(),
            label: "工资发放".to_string(),
            description: "处理本月工资".to_string(),
            path: "/salary".to_string(),
            color: "from-green-500 to-green-600".to_string(),
        },
        WorkbenchQuickAction {
            icon: "Target".to_string(),
            label: "项目申报".to_string(),
            description: "创建新的申报项目".to_string(),
            path: "/applications".to_string(),
            color: "from-purple-500 to-purple-600".to_string(),
        },
        WorkbenchQuickAction {
            icon: "Building2".to_string(),
            label: "资产登记".to_string(),
            description: "录入新资产".to_string(),
            path: "/assets".to_string(),
            color: "from-orange-500 to-orange-600".to_string(),
        },
    ]
}

fn expense_trend(points: &[MonthlyExpensePoint]) -> Option<(String, bool)> {
    let current = points.last()?;
    let previous = points.iter().rev().nth(1)?;
    if previous.total <= 0.0 {
        return None;
    }

    let change = ((current.total - previous.total) / previous.total) * 100.0;
    let sign = if change >= 0.0 { "+" } else { "" };
    Some((format!("{}{:.0}%", sign, change), change >= 0.0))
}

fn generated_date_label(today: NaiveDate) -> String {
    let weekday = match today.weekday() {
        chrono::Weekday::Mon => "星期一",
        chrono::Weekday::Tue => "星期二",
        chrono::Weekday::Wed => "星期三",
        chrono::Weekday::Thu => "星期四",
        chrono::Weekday::Fri => "星期五",
        chrono::Weekday::Sat => "星期六",
        chrono::Weekday::Sun => "星期日",
    };

    format!(
        "{}年{}月{}日 {}",
        today.year(),
        today.month(),
        today.day(),
        weekday
    )
}

fn priority_badge(priority: &str) -> WorkbenchBadge {
    match priority {
        "critical" => WorkbenchBadge {
            label: "逾期".to_string(),
            class_name: "text-destructive".to_string(),
            color_class: "bg-destructive".to_string(),
        },
        "high" => WorkbenchBadge {
            label: "紧急".to_string(),
            class_name: "text-destructive".to_string(),
            color_class: "bg-destructive".to_string(),
        },
        "medium" => WorkbenchBadge {
            label: "重要".to_string(),
            class_name: "text-warning".to_string(),
            color_class: "bg-warning".to_string(),
        },
        _ => WorkbenchBadge {
            label: "一般".to_string(),
            class_name: "text-primary".to_string(),
            color_class: "bg-primary".to_string(),
        },
    }
}

fn priority_badge_for_type(item_type: &str, priority: &str) -> WorkbenchBadge {
    match item_type {
        "asset" => match priority {
            "critical" => WorkbenchBadge {
                label: "逾期".to_string(),
                class_name: "text-destructive".to_string(),
                color_class: "bg-destructive".to_string(),
            },
            "high" => WorkbenchBadge {
                label: "紧急".to_string(),
                class_name: "text-destructive".to_string(),
                color_class: "bg-destructive".to_string(),
            },
            "medium" => WorkbenchBadge {
                label: "即将到期".to_string(),
                class_name: "text-warning".to_string(),
                color_class: "bg-warning".to_string(),
            },
            _ => WorkbenchBadge {
                label: "关注".to_string(),
                class_name: "text-primary".to_string(),
                color_class: "bg-primary".to_string(),
            },
        },
        _ => priority_badge(priority),
    }
}

fn days_left_label_for_type(item_type: &str, days_left: i64) -> String {
    let action = match item_type {
        "asset" | "contract" => "到期",
        "application" | "invoice" | "tax" => "截止",
        _ => "",
    };

    if days_left < 0 {
        if action.is_empty() {
            format!("{} 天前", days_left.abs())
        } else {
            format!("已逾期 {} 天", days_left.abs())
        }
    } else if days_left == 0 {
        if action.is_empty() {
            "今天".to_string()
        } else {
            format!("今天{}", action)
        }
    } else if action.is_empty() {
        format!("{} 天后", days_left)
    } else {
        format!("{} 天后{}", days_left, action)
    }
}

fn type_icon(item_type: &str) -> &'static str {
    match item_type {
        "invoice" | "contract" => "FileText",
        "asset" => "Building2",
        "tax" => "DollarSign",
        "application" => "Target",
        "salary" => "Users",
        _ => "Bell",
    }
}

fn percent_label(numerator: i64, denominator: i64) -> String {
    if denominator <= 0 {
        "0%".to_string()
    } else {
        format!("{:.0}%", numerator as f64 / denominator as f64 * 100.0)
    }
}

fn format_money(value: f64) -> String {
    format!("¥{}", format_decimal_with_commas(value, 2))
}

fn format_decimal_with_commas(value: f64, decimals: usize) -> String {
    let safe_value = if value.is_finite() { value } else { 0.0 };
    let sign = if safe_value < 0.0 { "-" } else { "" };
    let raw = format!("{:.*}", decimals, safe_value.abs());
    let (integer, fraction) = raw.split_once('.').unwrap_or((raw.as_str(), ""));
    let mut grouped_reversed = String::new();

    for (index, ch) in integer.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped_reversed.push(',');
        }
        grouped_reversed.push(ch);
    }

    let grouped_integer = grouped_reversed.chars().rev().collect::<String>();
    if decimals == 0 {
        format!("{sign}{grouped_integer}")
    } else {
        format!("{sign}{grouped_integer}.{fraction}")
    }
}

fn summarize_insights(insights: &[OperationalInsight]) -> InsightSummary {
    InsightSummary {
        total: insights.len() as i64,
        critical: insights
            .iter()
            .filter(|item| item.priority == "critical")
            .count() as i64,
        high: insights
            .iter()
            .filter(|item| item.priority == "high")
            .count() as i64,
        medium: insights
            .iter()
            .filter(|item| item.priority == "medium")
            .count() as i64,
        low: insights
            .iter()
            .filter(|item| item.priority == "low")
            .count() as i64,
    }
}

fn days_until(value: &str, today: NaiveDate) -> Option<i64> {
    let prefix: String = value.chars().take(10).collect();
    NaiveDate::parse_from_str(&prefix, "%Y-%m-%d")
        .ok()
        .map(|date| (date - today).num_days())
}

fn priority_for_days(days_left: Option<i64>, high_window_days: i64) -> String {
    match days_left {
        Some(days) if days < 0 => "critical".to_string(),
        Some(days) if days <= high_window_days => "high".to_string(),
        Some(days) if days <= 14 => "medium".to_string(),
        Some(_) => "low".to_string(),
        None => "medium".to_string(),
    }
}

fn priority_rank(priority: &str) -> i32 {
    match priority {
        "critical" => 0,
        "high" => 1,
        "medium" => 2,
        "low" => 3,
        _ => 4,
    }
}

fn category_label(item_type: &str) -> &'static str {
    match item_type {
        "invoice" | "tax" => "财务",
        "contract" => "合同",
        "asset" => "资产",
        "application" => "申报",
        _ => "运营",
    }
}

fn greeting() -> String {
    let hour = Utc::now().hour();
    match hour {
        5..=11 => "早上好".to_string(),
        12..=17 => "下午好".to_string(),
        18..=22 => "晚上好".to_string(),
        _ => "夜深了".to_string(),
    }
}
