use serde_json::{Value, json};
use sqlx::SqlitePool;

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

fn pct(done: i64, total: i64) -> i64 {
    if total == 0 {
        0
    } else {
        ((done as f64 / total as f64) * 100.0).round() as i64
    }
}

fn operating_step(
    id: &str,
    label: &str,
    route: &str,
    evidence: Option<String>,
    missing_label: &str,
) -> Value {
    let completed = evidence.is_some();
    json!({
        "id": id,
        "label": label,
        "route": route,
        "completed": completed,
        "evidence_label": evidence.unwrap_or_else(|| missing_label.to_string()),
        "status_badge": status_badge(completed),
    })
}

async fn first_operating_evidence(
    pool: &SqlitePool,
    company_id: &str,
    sql: &str,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(sql)
        .bind(company_id)
        .fetch_optional(pool)
        .await
}
fn operating_next_action(current_step_id: &str) -> Value {
    match current_step_id {
        "contracts" => json!({
            "route": "/contracts",
            "label": "归档当前合同",
            "description": "先补一份有效合同，让项目、资产、租赁和提醒可以追溯到业务依据。",
            "action": null
        }),
        "projects" => json!({
            "route": "/projects",
            "label": "建立当前项目",
            "description": "创建正在推进的项目，后续预算、工时和合同可以围绕项目归集。",
            "action": null
        }),
        "employees" => json!({
            "route": "/employees",
            "label": "录入在职员工",
            "description": "录入至少一名在职员工，工资和人员成本才能进入运营台账。",
            "action": null
        }),
        "payroll" => json!({
            "route": "/salary",
            "label": "连接本月工资",
            "description": "为在职员工添加本月工资记录，避免孤立工资行误判为薪酬闭环。",
            "action": null
        }),
        "assets" => json!({
            "route": "/assets",
            "label": "登记有效资产",
            "description": "登记当前仍有效的资产，让折旧、到期和合同来源可审计。",
            "action": null
        }),
        "licenses" => json!({
            "route": "/licenses",
            "label": "维护有效证照",
            "description": "录入仍有效的证照和到期日，后续提醒可自动跟进。",
            "action": null
        }),
        "rent" => json!({
            "route": "/rent",
            "label": "录入当前租赁",
            "description": "记录仍在生效的办公场地或仓储租赁，租金和合同到期才可追踪。",
            "action": null
        }),
        "budgets" => json!({
            "route": "/budget-management",
            "label": "设置当前预算",
            "description": "为当前期间设置预算，让项目和运营成本有控制线。",
            "action": null
        }),
        "reminders" => json!({
            "route": "/reminders",
            "label": "添加关联提醒",
            "description": "为合同、证照、资产或租赁等运营对象添加待处理提醒，保证关键期限不会遗漏。",
            "action": null
        }),
        _ => json!({
            "route": "/reminders",
            "label": "查看运营提醒",
            "description": "运营骨架已就位，可以从提醒中心查看后续动作和风险。",
            "action": null
        }),
    }
}

pub async fn build_operating_backbone(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<Value, sqlx::Error> {
    let contract = first_operating_evidence(
        pool,
        company_id,
        r#"
        SELECT '已归档当前合同：' || COALESCE(contract_name, contract_number, '未命名合同')
          FROM contracts
         WHERE company_id = ?
           AND LOWER(COALESCE(status, 'active')) NOT IN (
               'inactive', 'expired', 'terminated', 'cancelled', 'canceled', 'archived', 'completed'
           )
           AND (end_date IS NULL OR end_date = '' OR end_date >= date('now'))
         ORDER BY COALESCE(end_date, '9999-12-31') ASC, created_at ASC, id ASC
         LIMIT 1
        "#,
    )
    .await?;

    let project = first_operating_evidence(
        pool,
        company_id,
        r#"
        SELECT '当前项目已连接合同：' || COALESCE(p.name, '未命名项目')
          FROM projects p
          JOIN contracts c
            ON c.company_id = p.company_id
           AND c.project_id = p.id
         WHERE p.company_id = ?
           AND LOWER(COALESCE(p.status, 'active')) IN ('active', 'ongoing', 'in_progress', 'planning')
           AND COALESCE(p.progress, 0) < 100
           AND LOWER(COALESCE(c.status, 'active')) NOT IN (
               'inactive', 'expired', 'terminated', 'cancelled', 'canceled', 'archived', 'completed'
           )
           AND (c.end_date IS NULL OR c.end_date = '' OR c.end_date >= date('now'))
         ORDER BY p.updated_at DESC, p.created_at ASC, p.id ASC
         LIMIT 1
        "#,
    )
    .await?;

    let employee = first_operating_evidence(
        pool,
        company_id,
        r#"
        SELECT '在职员工：' || COALESCE(name, full_name, employee_number, '员工')
          FROM employees
         WHERE company_id = ?
           AND LOWER(COALESCE(status, 'active')) = 'active'
         ORDER BY created_at ASC, id ASC
         LIMIT 1
        "#,
    )
    .await?;

    let payroll = first_operating_evidence(
        pool,
        company_id,
        r#"
        SELECT '本月工资已连接员工：' || COALESCE(e.name, e.full_name, s.employee_name, '员工')
          FROM salary_records s
          JOIN employees e
            ON e.company_id = s.company_id
           AND e.id = s.employee_id
         WHERE s.company_id = ?
           AND LOWER(COALESCE(e.status, 'active')) = 'active'
           AND s.period = strftime('%Y-%m', 'now')
         ORDER BY s.created_at ASC, s.id ASC
         LIMIT 1
        "#,
    )
    .await?;

    let asset = first_operating_evidence(
        pool,
        company_id,
        r#"
        SELECT '有效资产已连接合同：' || COALESCE(a.name, a.serial_number, '资产')
          FROM assets a
          JOIN contracts c
            ON c.company_id = a.company_id
           AND c.id = a.contract_id
         WHERE a.company_id = ?
           AND LOWER(COALESCE(a.status, 'active')) = 'active'
           AND (a.expiry_date IS NULL OR a.expiry_date = '' OR a.expiry_date >= date('now'))
           AND LOWER(COALESCE(c.status, 'active')) NOT IN (
               'inactive', 'expired', 'terminated', 'cancelled', 'canceled', 'archived', 'completed'
           )
           AND (c.end_date IS NULL OR c.end_date = '' OR c.end_date >= date('now'))
         ORDER BY COALESCE(a.expiry_date, '9999-12-31') ASC, a.created_at ASC, a.id ASC
         LIMIT 1
        "#,
    )
    .await?;

    let license = first_operating_evidence(
        pool,
        company_id,
        r#"
        SELECT '有效证照：' || COALESCE(license_name, license_number, '证照')
          FROM licenses
         WHERE company_id = ?
           AND LOWER(COALESCE(status, 'valid')) NOT IN (
               'inactive', 'expired', 'revoked', 'cancelled', 'canceled', 'archived'
           )
           AND (expiry_date IS NULL OR expiry_date = '' OR expiry_date >= date('now'))
         ORDER BY COALESCE(expiry_date, '9999-12-31') ASC, created_at ASC, id ASC
         LIMIT 1
        "#,
    )
    .await?;

    let rent = first_operating_evidence(
        pool,
        company_id,
        r#"
        SELECT '当前租赁已连接合同：' || COALESCE(rr.location_name, rr.address, '租赁记录')
          FROM rent_records rr
          JOIN contracts c
            ON c.company_id = rr.company_id
           AND c.id = rr.contract_id
         WHERE rr.company_id = ?
           AND LOWER(COALESCE(rr.status, 'active')) = 'active'
           AND COALESCE(rr.monthly_rent, 0) > 0
           AND (
               rr.contract_end_date IS NULL
               OR rr.contract_end_date = ''
               OR rr.contract_end_date >= date('now')
           )
           AND LOWER(COALESCE(c.status, 'active')) NOT IN (
               'inactive', 'expired', 'terminated', 'cancelled', 'canceled', 'archived', 'completed'
           )
           AND (c.end_date IS NULL OR c.end_date = '' OR c.end_date >= date('now'))
         ORDER BY COALESCE(rr.contract_end_date, '9999-12-31') ASC, rr.created_at ASC, rr.id ASC
         LIMIT 1
        "#,
    )
    .await?;

    let budget = first_operating_evidence(
        pool,
        company_id,
        r#"
        SELECT '当前预算已连接项目：' || COALESCE(b.category, '预算')
          FROM budgets b
          JOIN projects p
            ON p.company_id = b.company_id
           AND p.id = b.project_id
         WHERE b.company_id = ?
           AND LOWER(COALESCE(b.status, 'active')) = 'active'
           AND COALESCE(b.budget_amount, 0) > 0
           AND b.period IN (
               strftime('%Y-%m', 'now'),
               strftime('%Y', 'now'),
               strftime('%Y', 'now') || '-Q' ||
                   CASE
                     WHEN CAST(strftime('%m', 'now') AS INTEGER) BETWEEN 1 AND 3 THEN '1'
                     WHEN CAST(strftime('%m', 'now') AS INTEGER) BETWEEN 4 AND 6 THEN '2'
                     WHEN CAST(strftime('%m', 'now') AS INTEGER) BETWEEN 7 AND 9 THEN '3'
                     ELSE '4'
                   END
           )
           AND LOWER(COALESCE(p.status, 'active')) IN ('active', 'ongoing', 'in_progress', 'planning')
           AND COALESCE(p.progress, 0) < 100
         ORDER BY b.created_at ASC, b.id ASC
         LIMIT 1
        "#,
    )
    .await?;

    let reminder = first_operating_evidence(
        pool,
        company_id,
        r#"
        SELECT '待处理提醒：' || COALESCE(r.title, '运营提醒')
          FROM reminders r
         WHERE r.company_id = ?
           AND COALESCE(r.is_completed, 0) = 0
           AND LOWER(COALESCE(r.status, 'pending')) NOT IN ('completed', 'dismissed', 'done')
           AND COALESCE(r.entity_type, '') != ''
           AND COALESCE(r.entity_id, '') != ''
           AND (
               (
                   LOWER(r.entity_type) IN ('contracts', 'contract')
                   AND EXISTS (
                       SELECT 1
                         FROM contracts c
                        WHERE c.company_id = r.company_id
                          AND c.id = r.entity_id
                          AND LOWER(COALESCE(c.status, 'active')) NOT IN (
                              'inactive', 'expired', 'terminated', 'cancelled', 'canceled', 'archived', 'completed'
                          )
                          AND (c.end_date IS NULL OR c.end_date = '' OR c.end_date >= date('now'))
                   )
               )
               OR (
                   LOWER(r.entity_type) IN ('projects', 'project')
                   AND EXISTS (
                       SELECT 1
                         FROM projects p
                        WHERE p.company_id = r.company_id
                          AND p.id = r.entity_id
                          AND LOWER(COALESCE(p.status, 'active')) IN ('active', 'ongoing', 'in_progress', 'planning')
                          AND COALESCE(p.progress, 0) < 100
                   )
               )
               OR (
                   LOWER(r.entity_type) IN ('employees', 'employee')
                   AND EXISTS (
                       SELECT 1
                         FROM employees e
                        WHERE e.company_id = r.company_id
                          AND e.id = r.entity_id
                          AND LOWER(COALESCE(e.status, 'active')) = 'active'
                   )
               )
               OR (
                   LOWER(r.entity_type) IN ('salary_records', 'salary_record', 'payroll')
                   AND EXISTS (
                       SELECT 1
                         FROM salary_records s
                         JOIN employees e
                           ON e.company_id = s.company_id
                          AND e.id = s.employee_id
                        WHERE s.company_id = r.company_id
                          AND s.id = r.entity_id
                          AND LOWER(COALESCE(e.status, 'active')) = 'active'
                          AND s.period = strftime('%Y-%m', 'now')
                   )
               )
               OR (
                   LOWER(r.entity_type) IN ('assets', 'asset')
                   AND EXISTS (
                       SELECT 1
                         FROM assets a
                        WHERE a.company_id = r.company_id
                          AND a.id = r.entity_id
                          AND LOWER(COALESCE(a.status, 'active')) = 'active'
                          AND (a.expiry_date IS NULL OR a.expiry_date = '' OR a.expiry_date >= date('now'))
                   )
               )
               OR (
                   LOWER(r.entity_type) IN ('licenses', 'license')
                   AND EXISTS (
                       SELECT 1
                         FROM licenses l
                        WHERE l.company_id = r.company_id
                          AND l.id = r.entity_id
                          AND LOWER(COALESCE(l.status, 'valid')) NOT IN (
                              'inactive', 'expired', 'revoked', 'cancelled', 'canceled', 'archived'
                          )
                          AND (l.expiry_date IS NULL OR l.expiry_date = '' OR l.expiry_date >= date('now'))
                   )
               )
               OR (
                   LOWER(r.entity_type) IN ('rent_records', 'rent_record', 'rent')
                   AND EXISTS (
                       SELECT 1
                         FROM rent_records rr
                        WHERE rr.company_id = r.company_id
                          AND rr.id = r.entity_id
                          AND LOWER(COALESCE(rr.status, 'active')) = 'active'
                          AND COALESCE(rr.monthly_rent, 0) > 0
                          AND (
                              rr.contract_end_date IS NULL
                              OR rr.contract_end_date = ''
                              OR rr.contract_end_date >= date('now')
                          )
                   )
               )
               OR (
                   LOWER(r.entity_type) IN ('budgets', 'budget')
                   AND EXISTS (
                       SELECT 1
                         FROM budgets b
                        WHERE b.company_id = r.company_id
                          AND b.id = r.entity_id
                          AND LOWER(COALESCE(b.status, 'active')) = 'active'
                          AND COALESCE(b.budget_amount, 0) > 0
                          AND b.period IN (
                              strftime('%Y-%m', 'now'),
                              strftime('%Y', 'now'),
                              strftime('%Y', 'now') || '-Q' ||
                                  CASE
                                    WHEN CAST(strftime('%m', 'now') AS INTEGER) BETWEEN 1 AND 3 THEN '1'
                                    WHEN CAST(strftime('%m', 'now') AS INTEGER) BETWEEN 4 AND 6 THEN '2'
                                    WHEN CAST(strftime('%m', 'now') AS INTEGER) BETWEEN 7 AND 9 THEN '3'
                                    ELSE '4'
                                  END
                          )
                   )
               )
           )
         ORDER BY COALESCE(r.remind_date, r.due_date, r.created_at) ASC, r.created_at ASC, r.id ASC
         LIMIT 1
        "#,
    )
    .await?;

    let steps = vec![
        operating_step(
            "contracts",
            "合同",
            "/contracts",
            contract,
            "尚未归档当前有效合同",
        ),
        operating_step(
            "projects",
            "项目",
            "/projects",
            project,
            "尚未建立与合同相连的当前项目",
        ),
        operating_step(
            "employees",
            "员工",
            "/employees",
            employee,
            "尚未录入在职员工",
        ),
        operating_step(
            "payroll",
            "工资",
            "/salary",
            payroll,
            "尚未为在职员工连接本月工资",
        ),
        operating_step(
            "assets",
            "资产",
            "/assets",
            asset,
            "尚未登记与合同相连的当前有效资产",
        ),
        operating_step("licenses", "证照", "/licenses", license, "尚未维护有效证照"),
        operating_step(
            "rent",
            "租赁",
            "/rent",
            rent,
            "尚未录入与合同相连的当前租赁",
        ),
        operating_step(
            "budgets",
            "预算",
            "/budget-management",
            budget,
            "尚未设置与项目相连的当前期间预算",
        ),
        operating_step(
            "reminders",
            "提醒",
            "/reminders",
            reminder,
            "尚未添加关联到现有运营对象的待处理提醒",
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
        "id": "operating_backbone",
        "title": "运营骨架",
        "completed": completed,
        "current_step_id": current_step_id,
        "completed_count": completed_count,
        "total_count": total_count,
        "remaining_count": total_count.saturating_sub(completed_count),
        "rate": rate,
        "rate_label": format!("{}%", rate),
        "label": format!("运营骨架 {}/{}", completed_count, total_count),
        "status_badge": status_badge(completed),
        "next_action": operating_next_action(current_step_id),
        "steps": steps,
    }))
}
