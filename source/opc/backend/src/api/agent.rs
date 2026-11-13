use crate::auth::middleware::AuthUser;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use sqlx::{QueryBuilder, Row, Sqlite, SqlitePool};
use std::{path::PathBuf, sync::OnceLock};
use tokio::fs;
use uuid::Uuid;

const UPLOAD_DIR: &str = "data/uploads";
const MAX_AGENT_FILE_SIZE: usize = 10 * 1024 * 1024;

#[derive(Clone, Copy)]
enum ToolKind {
    List,
    Create,
    Update,
    Delete,
    PaymentApprove,
    FileUpload,
    OnboardingProgress,
    OnboardingAutofill,
    SettingsOverview,
}

#[derive(Clone, Copy)]
struct AgentToolSpec {
    name: &'static str,
    domain: &'static str,
    title: &'static str,
    description: &'static str,
    method: &'static str,
    endpoint_template: &'static str,
    required_scope: &'static str,
    risk_level: &'static str,
    kind: ToolKind,
    table: Option<&'static str>,
    required_fields: &'static [&'static str],
}

#[derive(Clone, Copy)]
struct BusinessResource {
    domain: &'static str,
    table: &'static str,
    title: &'static str,
    required_fields: &'static [&'static str],
}

#[derive(Deserialize)]
struct AgentExecuteRequest {
    tool_name: String,
    agent_name: Option<String>,
    #[serde(default)]
    arguments: Value,
}

#[derive(Deserialize)]
struct McpRequest {
    jsonrpc: Option<String>,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug)]
struct AgentError {
    status: StatusCode,
    message: String,
}

enum AgentBindValue {
    Text(String),
    Integer(i64),
    Real(f64),
    Null,
}

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/tools", get(list_tools))
        .route("/tools/{name}/schema", get(tool_schema))
        .route("/execute", post(execute_agent))
        .with_state(pool)
}

pub fn mcp_router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/mcp", post(mcp_handler))
        .with_state(pool)
}

fn business_resources() -> Vec<BusinessResource> {
    vec![
        BusinessResource {
            domain: "assets",
            table: "assets",
            title: "资产",
            required_fields: &["name"],
        },
        BusinessResource {
            domain: "invoices",
            table: "invoices",
            title: "发票",
            required_fields: &["invoice_number", "amount"],
        },
        BusinessResource {
            domain: "branches",
            table: "branches",
            title: "分支机构",
            required_fields: &["branch_name"],
        },
        BusinessResource {
            domain: "company_info",
            table: "company_info",
            title: "公司信息",
            required_fields: &["company_name"],
        },
        BusinessResource {
            domain: "payments",
            table: "payments",
            title: "付款",
            required_fields: &["payment_type", "amount"],
        },
        BusinessResource {
            domain: "contracts",
            table: "contracts",
            title: "合同",
            required_fields: &["contract_name"],
        },
        BusinessResource {
            domain: "projects",
            table: "projects",
            title: "项目",
            required_fields: &["name"],
        },
        BusinessResource {
            domain: "vendors",
            table: "vendors",
            title: "供应商",
            required_fields: &["name"],
        },
        BusinessResource {
            domain: "clients",
            table: "clients",
            title: "客户",
            required_fields: &["name"],
        },
        BusinessResource {
            domain: "deals",
            table: "deals",
            title: "销售机会",
            required_fields: &["client_id", "title"],
        },
        BusinessResource {
            domain: "quotes",
            table: "quotes",
            title: "报价",
            required_fields: &["client_id", "quote_number"],
        },
        BusinessResource {
            domain: "employees",
            table: "employees",
            title: "员工",
            required_fields: &["name"],
        },
        BusinessResource {
            domain: "salary",
            table: "salary_records",
            title: "工资",
            required_fields: &["employee_id", "period"],
        },
        BusinessResource {
            domain: "rent",
            table: "rent_records",
            title: "场地租赁",
            required_fields: &["location_name", "monthly_rent"],
        },
        BusinessResource {
            domain: "licenses",
            table: "licenses",
            title: "证照",
            required_fields: &["license_name"],
        },
        BusinessResource {
            domain: "reminders",
            table: "reminders",
            title: "提醒",
            required_fields: &["title"],
        },
        BusinessResource {
            domain: "budgets",
            table: "budgets",
            title: "预算",
            required_fields: &["category", "budget_type", "period"],
        },
        BusinessResource {
            domain: "tax",
            table: "tax_records",
            title: "税务",
            required_fields: &["tax_type", "period"],
        },
        BusinessResource {
            domain: "kpi",
            table: "kpi_configs",
            title: "KPI",
            required_fields: &["name"],
        },
        BusinessResource {
            domain: "bank_accounts",
            table: "bank_accounts",
            title: "银行账户",
            required_fields: &["bank_name", "account_name", "account_number"],
        },
        BusinessResource {
            domain: "imports",
            table: "material_library",
            title: "导入材料",
            required_fields: &["name"],
        },
        BusinessResource {
            domain: "policies",
            table: "policy_applications",
            title: "政策申报",
            required_fields: &["policy_id"],
        },
        BusinessResource {
            domain: "applications",
            table: "applications",
            title: "项目申报",
            required_fields: &["title", "type"],
        },
        BusinessResource {
            domain: "profiles",
            table: "profiles",
            title: "用户档案",
            required_fields: &["display_name"],
        },
        BusinessResource {
            domain: "cloud_cost",
            table: "cloud_resources",
            title: "云成本",
            required_fields: &["resource_type", "resource_name", "start_date"],
        },
        BusinessResource {
            domain: "recurring_invoices",
            table: "recurring_invoices",
            title: "周期发票",
            required_fields: &[
                "title",
                "amount",
                "frequency",
                "start_date",
                "next_invoice_date",
            ],
        },
        BusinessResource {
            domain: "time_tracking",
            table: "time_entries",
            title: "工时",
            required_fields: &["description", "date"],
        },
    ]
}

fn agent_tools() -> &'static [AgentToolSpec] {
    static AGENT_TOOLS: OnceLock<Vec<AgentToolSpec>> = OnceLock::new();
    AGENT_TOOLS.get_or_init(build_agent_tools).as_slice()
}

fn build_agent_tools() -> Vec<AgentToolSpec> {
    let mut tools = Vec::new();

    for resource in business_resources() {
        tools.push(AgentToolSpec {
            name: Box::leak(format!("{}.list", resource.domain).into_boxed_str()),
            domain: resource.domain,
            title: Box::leak(format!("查询{}", resource.title).into_boxed_str()),
            description: Box::leak(
                format!("后端租户隔离查询{}数据", resource.title).into_boxed_str(),
            ),
            method: "GET",
            endpoint_template: Box::leak(format!("/api/v1/{}", resource.domain).into_boxed_str()),
            required_scope: Box::leak(format!("{}:read", resource.domain).into_boxed_str()),
            risk_level: "low",
            kind: ToolKind::List,
            table: Some(resource.table),
            required_fields: &[],
        });
        tools.push(AgentToolSpec {
            name: Box::leak(format!("{}.create", resource.domain).into_boxed_str()),
            domain: resource.domain,
            title: Box::leak(format!("创建{}", resource.title).into_boxed_str()),
            description: Box::leak(
                format!("后端租户隔离创建{}数据", resource.title).into_boxed_str(),
            ),
            method: "POST",
            endpoint_template: Box::leak(format!("/api/v1/{}", resource.domain).into_boxed_str()),
            required_scope: Box::leak(format!("{}:write", resource.domain).into_boxed_str()),
            risk_level: "medium",
            kind: ToolKind::Create,
            table: Some(resource.table),
            required_fields: resource.required_fields,
        });
        tools.push(AgentToolSpec {
            name: Box::leak(format!("{}.update", resource.domain).into_boxed_str()),
            domain: resource.domain,
            title: Box::leak(format!("更新{}", resource.title).into_boxed_str()),
            description: Box::leak(
                format!("按 id 更新租户内{}数据", resource.title).into_boxed_str(),
            ),
            method: "PUT",
            endpoint_template: Box::leak(
                format!("/api/v1/{}/{{id}}", resource.domain).into_boxed_str(),
            ),
            required_scope: Box::leak(format!("{}:write", resource.domain).into_boxed_str()),
            risk_level: "medium",
            kind: ToolKind::Update,
            table: Some(resource.table),
            required_fields: &["id"],
        });
        tools.push(AgentToolSpec {
            name: Box::leak(format!("{}.delete", resource.domain).into_boxed_str()),
            domain: resource.domain,
            title: Box::leak(format!("删除{}", resource.title).into_boxed_str()),
            description: Box::leak(
                format!("按 id 删除租户内{}数据", resource.title).into_boxed_str(),
            ),
            method: "DELETE",
            endpoint_template: Box::leak(
                format!("/api/v1/{}/{{id}}", resource.domain).into_boxed_str(),
            ),
            required_scope: Box::leak(format!("{}:write", resource.domain).into_boxed_str()),
            risk_level: "high",
            kind: ToolKind::Delete,
            table: Some(resource.table),
            required_fields: &["id"],
        });
    }

    tools.extend([
        AgentToolSpec {
            name: "payments.approve",
            domain: "payments",
            title: "审批付款",
            description: "将租户内付款记录标记为已支付",
            method: "POST",
            endpoint_template: "/api/v1/payments/{id}/approve",
            required_scope: "payments:approve",
            risk_level: "high",
            kind: ToolKind::PaymentApprove,
            table: Some("payments"),
            required_fields: &["id"],
        },
        AgentToolSpec {
            name: "files.list",
            domain: "files",
            title: "查询附件",
            description: "查询租户内附件元数据",
            method: "GET",
            endpoint_template: "/api/v1/files",
            required_scope: "files:read",
            risk_level: "low",
            kind: ToolKind::List,
            table: Some("files"),
            required_fields: &[],
        },
        AgentToolSpec {
            name: "files.upload",
            domain: "files",
            title: "上传附件",
            description: "通过 JSON/base64 上传文件并关联业务实体",
            method: "POST",
            endpoint_template: "/api/v1/files/upload",
            required_scope: "files:write",
            risk_level: "medium",
            kind: ToolKind::FileUpload,
            table: Some("files"),
            required_fields: &["original_name", "content_base64"],
        },
        AgentToolSpec {
            name: "onboarding.progress",
            domain: "onboarding",
            title: "读取新手引导进度",
            description: "读取 11 个新手引导模块的后端计算进度",
            method: "GET",
            endpoint_template: "/api/v1/onboarding/progress",
            required_scope: "onboarding:read",
            risk_level: "low",
            kind: ToolKind::OnboardingProgress,
            table: None,
            required_fields: &[],
        },
        AgentToolSpec {
            name: "onboarding.autofill",
            domain: "onboarding",
            title: "自动补齐新手引导",
            description: "为缺失的新手引导模块写入最小可用业务数据",
            method: "POST",
            endpoint_template: "/api/v1/onboarding/autofill",
            required_scope: "onboarding:write",
            risk_level: "medium",
            kind: ToolKind::OnboardingAutofill,
            table: None,
            required_fields: &["mode"],
        },
        AgentToolSpec {
            name: "audit.logs",
            domain: "audit",
            title: "查询审计日志",
            description: "查询租户内操作日志",
            method: "GET",
            endpoint_template: "/api/v1/operation_logs",
            required_scope: "audit:read",
            risk_level: "low",
            kind: ToolKind::List,
            table: Some("operation_logs"),
            required_fields: &[],
        },
        AgentToolSpec {
            name: "settings.overview",
            domain: "settings",
            title: "系统设置概览",
            description: "读取租户设置和集成概览",
            method: "GET",
            endpoint_template: "/api/v1/settings/overview",
            required_scope: "settings:read",
            risk_level: "low",
            kind: ToolKind::SettingsOverview,
            table: None,
            required_fields: &[],
        },
    ]);

    tools
}

fn find_tool(name: &str) -> Option<AgentToolSpec> {
    agent_tools().iter().copied().find(|tool| tool.name == name)
}

fn tool_input_schema(tool: &AgentToolSpec) -> Value {
    match tool.kind {
        ToolKind::FileUpload => json!({
            "type": "object",
            "properties": {
                "original_name": {"type": "string"},
                "content_type": {"type": "string", "default": "application/octet-stream"},
                "content_base64": {"type": "string"},
                "entity_type": {"type": "string"},
                "entity_id": {"type": "string"},
                "description": {"type": "string"}
            },
            "required": tool.required_fields,
            "additionalProperties": false
        }),
        ToolKind::OnboardingAutofill => json!({
            "type": "object",
            "properties": {
                "mode": {
                    "type": "string",
                    "enum": ["minimal_verified_defaults"],
                    "description": "Create only missing starter records"
                }
            },
            "required": tool.required_fields,
            "additionalProperties": false
        }),
        ToolKind::List | ToolKind::OnboardingProgress | ToolKind::SettingsOverview => json!({
            "type": "object",
            "properties": {
                "limit": {"type": "integer", "minimum": 1, "maximum": 200}
            },
            "required": [],
            "additionalProperties": false
        }),
        ToolKind::Update | ToolKind::Delete | ToolKind::PaymentApprove => json!({
            "type": "object",
            "properties": {
                "id": {"type": "string"}
            },
            "required": tool.required_fields,
            "additionalProperties": true
        }),
        ToolKind::Create => {
            let mut properties = Map::new();
            for field in tool.required_fields {
                properties.insert(
                    field.to_string(),
                    json!({"type": ["string", "number", "boolean"]}),
                );
            }
            json!({
                "type": "object",
                "properties": properties,
                "required": tool.required_fields,
                "additionalProperties": true
            })
        }
    }
}

fn tool_output_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "tool_name": {"type": "string"},
            "status": {"type": "string"},
            "result": {"type": "object"}
        },
        "required": ["tool_name", "status", "result"]
    })
}

fn tool_to_json(tool: &AgentToolSpec) -> Value {
    json!({
        "name": tool.name,
        "domain": tool.domain,
        "title": tool.title,
        "description": tool.description,
        "method": tool.method,
        "endpoint_template": tool.endpoint_template,
        "required_scope": tool.required_scope,
        "risk_level": tool.risk_level,
        "input_schema": tool_input_schema(tool),
        "output_schema": tool_output_schema()
    })
}

async fn list_tools() -> Json<Value> {
    let tools: Vec<Value> = agent_tools().iter().map(tool_to_json).collect();
    let total = tools.len();
    Json(json!({
        "tools": tools,
        "total": total
    }))
}

async fn tool_schema(Path(name): Path<String>) -> Result<Json<Value>, (StatusCode, String)> {
    let tool = find_tool(&name).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("Agent tool not found: {}", name),
        )
    })?;

    Ok(Json(json!({
        "tool": tool_to_json(&tool),
        "permissions": {
            "required_scope": tool.required_scope,
            "allowed_api_key_scopes": ["*", "read", "write", tool.required_scope]
        },
        "risk_level": tool.risk_level,
        "input_schema": tool_input_schema(&tool),
        "output_schema": tool_output_schema()
    })))
}

async fn execute_agent(
    State(pool): State<SqlitePool>,
    auth: AuthUser,
    Json(req): Json<AgentExecuteRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let agent_name = req
        .agent_name
        .clone()
        .unwrap_or_else(|| "rest-agent".to_string());
    let result = execute_tool_with_audit(
        &pool,
        &auth,
        &agent_name,
        &req.tool_name,
        req.arguments.clone(),
    )
    .await
    .map_err(|err| (err.status, err.message))?;
    Ok(Json(result))
}

async fn mcp_handler(
    State(pool): State<SqlitePool>,
    auth: AuthUser,
    Json(req): Json<McpRequest>,
) -> Json<Value> {
    let id = req.id.clone().unwrap_or(Value::Null);
    let jsonrpc = req.jsonrpc.clone().unwrap_or_else(|| "2.0".to_string());

    match req.method.as_str() {
        "initialize" => Json(json!({
            "jsonrpc": jsonrpc,
            "id": id,
            "result": {
                "protocolVersion": "2025-06-18",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "OPC Agent Gateway", "version": env!("CARGO_PKG_VERSION")}
            }
        })),
        "tools/list" => {
            let tools: Vec<Value> = agent_tools()
                .iter()
                .map(|tool| {
                    json!({
                        "name": tool.name,
                        "description": tool.description,
                        "inputSchema": tool_input_schema(tool)
                    })
                })
                .collect();
            Json(json!({
                "jsonrpc": jsonrpc,
                "id": id,
                "result": {"tools": tools}
            }))
        }
        "tools/call" => {
            let name = req.params["name"].as_str().unwrap_or_default();
            let arguments = req
                .params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            match execute_tool_with_audit(&pool, &auth, "mcp", name, arguments).await {
                Ok(result) => Json(json!({
                    "jsonrpc": jsonrpc,
                    "id": id,
                    "result": {
                        "content": [{"type": "text", "text": format!("{} executed with status {}", name, result["status"].as_str().unwrap_or("unknown"))}],
                        "structuredContent": result,
                        "isError": false
                    }
                })),
                Err(err) => Json(json!({
                    "jsonrpc": jsonrpc,
                    "id": id,
                    "result": {
                        "content": [{"type": "text", "text": err.message}],
                        "isError": true
                    }
                })),
            }
        }
        _ => Json(json!({
            "jsonrpc": jsonrpc,
            "id": id,
            "error": {"code": -32601, "message": "Method not found"}
        })),
    }
}

async fn execute_tool_with_audit(
    pool: &SqlitePool,
    auth: &AuthUser,
    agent_name: &str,
    tool_name: &str,
    arguments: Value,
) -> Result<Value, AgentError> {
    let tool = find_tool(tool_name).ok_or_else(|| AgentError {
        status: StatusCode::NOT_FOUND,
        message: format!("Agent tool not found: {tool_name}"),
    })?;

    if !api_key_scopes_allow_tool(auth, &tool) {
        let err = AgentError {
            status: StatusCode::FORBIDDEN,
            message: format!("API key scope denied for {}", tool.name),
        };
        record_agent_audit(
            pool,
            auth,
            agent_name,
            &tool,
            &arguments,
            None,
            "forbidden",
            Some(&err.message),
        )
        .await?;
        return Err(err);
    }

    let execution = execute_tool(pool, auth, &tool, &arguments).await;
    match execution {
        Ok(result) => {
            let response = json!({
                "tool_name": tool.name,
                "status": "success",
                "result": result
            });
            record_agent_audit(
                pool,
                auth,
                agent_name,
                &tool,
                &arguments,
                Some(&response),
                "success",
                None,
            )
            .await?;
            Ok(response)
        }
        Err(err) => {
            let _ = record_agent_audit(
                pool,
                auth,
                agent_name,
                &tool,
                &arguments,
                None,
                "failed",
                Some(&err.message),
            )
            .await;
            Err(err)
        }
    }
}

fn api_key_scopes_allow_tool(auth: &AuthUser, tool: &AgentToolSpec) -> bool {
    if auth.api_key_id.is_none() {
        return true;
    }

    let scopes = &auth.api_key_scopes;
    if scopes.iter().any(|scope| scope == "*") {
        return true;
    }
    if scopes.iter().any(|scope| scope == tool.required_scope) {
        return true;
    }
    if scopes.iter().any(|scope| scope == tool.domain) {
        return true;
    }

    let required_is_read = tool.required_scope.ends_with(":read");
    if required_is_read && scopes.iter().any(|scope| scope == "read") {
        return true;
    }
    if !required_is_read && scopes.iter().any(|scope| scope == "write") {
        return true;
    }
    if !required_is_read
        && scopes
            .iter()
            .any(|scope| scope == &format!("{}:write", tool.domain))
    {
        return true;
    }

    false
}

async fn record_agent_audit(
    pool: &SqlitePool,
    auth: &AuthUser,
    agent_name: &str,
    tool: &AgentToolSpec,
    request_payload: &Value,
    response_payload: Option<&Value>,
    status: &str,
    error_message: Option<&str>,
) -> Result<(), AgentError> {
    let id = Uuid::new_v4().to_string();
    let request_json = serde_json::to_string(request_payload).unwrap_or_else(|_| "{}".to_string());
    let response_json = response_payload
        .map(|value| serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string()));

    sqlx::query(
        r#"
        INSERT INTO agent_audit_logs (
            id, company_id, user_id, api_key_id, agent_name, tool_name, scope, risk_level,
            request_payload, response_payload, status, error_message
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(&auth.company_id)
    .bind(&auth.user_id)
    .bind(&auth.api_key_id)
    .bind(agent_name)
    .bind(tool.name)
    .bind(tool.required_scope)
    .bind(tool.risk_level)
    .bind(request_json)
    .bind(response_json)
    .bind(status)
    .bind(error_message)
    .execute(pool)
    .await
    .map_err(|err| AgentError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: format!("agent audit write failed: {err}"),
    })?;

    Ok(())
}

async fn execute_tool(
    pool: &SqlitePool,
    auth: &AuthUser,
    tool: &AgentToolSpec,
    arguments: &Value,
) -> Result<Value, AgentError> {
    match tool.kind {
        ToolKind::List => list_table(pool, tool.table.expect("list table"), &auth.company_id).await,
        ToolKind::Create => {
            if tool.name == "branches.create" {
                create_branch(pool, auth, arguments).await
            } else {
                create_table_record(
                    pool,
                    tool.table.expect("create table"),
                    &auth.company_id,
                    tool.required_fields,
                    arguments,
                )
                .await
            }
        }
        ToolKind::Update => {
            update_table_record(
                pool,
                tool.table.expect("update table"),
                &auth.company_id,
                arguments,
            )
            .await
        }
        ToolKind::Delete => {
            delete_table_record(
                pool,
                tool.table.expect("delete table"),
                &auth.company_id,
                arguments,
            )
            .await
        }
        ToolKind::PaymentApprove => approve_payment(pool, &auth.company_id, arguments).await,
        ToolKind::FileUpload => upload_agent_file(pool, auth, arguments).await,
        ToolKind::OnboardingProgress => onboarding_progress_value(pool, &auth.company_id).await,
        ToolKind::OnboardingAutofill => onboarding_autofill(pool, auth, arguments).await,
        ToolKind::SettingsOverview => settings_overview(pool, &auth.company_id).await,
    }
}

fn required_str<'a>(arguments: &'a Value, key: &str) -> Result<&'a str, AgentError> {
    arguments[key]
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AgentError {
            status: StatusCode::BAD_REQUEST,
            message: format!("missing required field: {key}"),
        })
}

fn json_to_bind_value(value: &Value) -> AgentBindValue {
    match value {
        Value::Null => AgentBindValue::Null,
        Value::Bool(value) => AgentBindValue::Integer(i64::from(*value)),
        Value::Number(value) => {
            if let Some(number) = value.as_i64() {
                AgentBindValue::Integer(number)
            } else if let Some(number) = value.as_f64() {
                AgentBindValue::Real(number)
            } else {
                AgentBindValue::Text(value.to_string())
            }
        }
        Value::String(value) => AgentBindValue::Text(value.clone()),
        Value::Array(_) | Value::Object(_) => AgentBindValue::Text(value.to_string()),
    }
}

async fn table_columns(pool: &SqlitePool, table: &str) -> Result<Vec<String>, AgentError> {
    let rows = sqlx::query(&format!("PRAGMA table_info({table})"))
        .fetch_all(pool)
        .await
        .map_err(|err| AgentError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("table metadata query failed: {err}"),
        })?;

    Ok(rows
        .iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect())
}

async fn list_table(pool: &SqlitePool, table: &str, company_id: &str) -> Result<Value, AgentError> {
    let count_sql = format!("SELECT COUNT(*) FROM {table} WHERE company_id = ?");
    let count: i64 = sqlx::query_scalar(&count_sql)
        .bind(company_id)
        .fetch_one(pool)
        .await
        .map_err(|err| AgentError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("list count failed: {err}"),
        })?;

    let list_sql =
        format!("SELECT id FROM {table} WHERE company_id = ? ORDER BY created_at DESC LIMIT 100");
    let rows = sqlx::query(&list_sql)
        .bind(company_id)
        .fetch_all(pool)
        .await
        .map_err(|err| AgentError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("list records failed: {err}"),
        })?;

    let records: Vec<Value> = rows
        .iter()
        .filter_map(|row| row.try_get::<String, _>("id").ok())
        .map(|id| json!({"id": id}))
        .collect();

    Ok(json!({
        "table": table,
        "count": count,
        "records": records
    }))
}

async fn create_table_record(
    pool: &SqlitePool,
    table: &str,
    company_id: &str,
    required_fields: &[&str],
    arguments: &Value,
) -> Result<Value, AgentError> {
    let object = arguments.as_object().ok_or_else(|| AgentError {
        status: StatusCode::BAD_REQUEST,
        message: "arguments must be a JSON object".to_string(),
    })?;

    for field in required_fields {
        if object.get(*field).is_none_or(|value| value.is_null()) {
            return Err(AgentError {
                status: StatusCode::BAD_REQUEST,
                message: format!("missing required field: {field}"),
            });
        }
    }

    let columns_available = table_columns(pool, table).await?;
    let has_column = |column: &str| columns_available.iter().any(|known| known == column);
    let id = object
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let mut columns = vec!["id".to_string(), "company_id".to_string()];
    let mut values = vec![
        AgentBindValue::Text(id.clone()),
        AgentBindValue::Text(company_id.to_string()),
    ];

    for (key, value) in object {
        if matches!(
            key.as_str(),
            "id" | "company_id" | "created_at" | "updated_at"
        ) {
            continue;
        }
        if has_column(key) {
            columns.push(key.clone());
            values.push(json_to_bind_value(value));
        }
    }

    let mut builder = QueryBuilder::<Sqlite>::new(format!("INSERT INTO {table} ("));
    {
        let mut separated = builder.separated(", ");
        for column in &columns {
            separated.push(column.as_str());
        }
    }
    builder.push(") VALUES (");
    {
        let mut separated = builder.separated(", ");
        for value in &values {
            match value {
                AgentBindValue::Text(value) => {
                    separated.push_bind(value.clone());
                }
                AgentBindValue::Integer(value) => {
                    separated.push_bind(*value);
                }
                AgentBindValue::Real(value) => {
                    separated.push_bind(*value);
                }
                AgentBindValue::Null => {
                    separated.push_bind(Option::<String>::None);
                }
            }
        }
    }
    builder.push(")");

    builder
        .build()
        .execute(pool)
        .await
        .map_err(|err| AgentError {
            status: StatusCode::BAD_REQUEST,
            message: format!("create failed: {err}"),
        })?;

    Ok(json!({"id": id, "table": table}))
}

async fn update_table_record(
    pool: &SqlitePool,
    table: &str,
    company_id: &str,
    arguments: &Value,
) -> Result<Value, AgentError> {
    let id = required_str(arguments, "id")?.to_string();
    let object = arguments.as_object().ok_or_else(|| AgentError {
        status: StatusCode::BAD_REQUEST,
        message: "arguments must be a JSON object".to_string(),
    })?;
    let columns_available = table_columns(pool, table).await?;

    let mut updates = Vec::new();
    for (key, value) in object {
        if matches!(
            key.as_str(),
            "id" | "company_id" | "created_at" | "updated_at"
        ) {
            continue;
        }
        if columns_available.iter().any(|known| known == key) {
            updates.push((key.clone(), json_to_bind_value(value)));
        }
    }

    if updates.is_empty() {
        return Err(AgentError {
            status: StatusCode::BAD_REQUEST,
            message: "no updatable fields supplied".to_string(),
        });
    }

    let mut builder = QueryBuilder::<Sqlite>::new(format!("UPDATE {table} SET "));
    {
        let mut separated = builder.separated(", ");
        for (column, value) in &updates {
            separated.push(format!("{column} = "));
            match value {
                AgentBindValue::Text(value) => {
                    separated.push_bind(value.clone());
                }
                AgentBindValue::Integer(value) => {
                    separated.push_bind(*value);
                }
                AgentBindValue::Real(value) => {
                    separated.push_bind(*value);
                }
                AgentBindValue::Null => {
                    separated.push_bind(Option::<String>::None);
                }
            }
        }
    }
    builder
        .push(" WHERE id = ")
        .push_bind(id.clone())
        .push(" AND company_id = ")
        .push_bind(company_id.to_string());

    let result = builder
        .build()
        .execute(pool)
        .await
        .map_err(|err| AgentError {
            status: StatusCode::BAD_REQUEST,
            message: format!("update failed: {err}"),
        })?;

    if result.rows_affected() == 0 {
        return Err(AgentError {
            status: StatusCode::NOT_FOUND,
            message: "record not found".to_string(),
        });
    }

    Ok(json!({"id": id, "table": table, "updated": true}))
}

async fn delete_table_record(
    pool: &SqlitePool,
    table: &str,
    company_id: &str,
    arguments: &Value,
) -> Result<Value, AgentError> {
    let id = required_str(arguments, "id")?.to_string();
    let sql = format!("DELETE FROM {table} WHERE id = ? AND company_id = ?");
    let result = sqlx::query(&sql)
        .bind(&id)
        .bind(company_id)
        .execute(pool)
        .await
        .map_err(|err| AgentError {
            status: StatusCode::BAD_REQUEST,
            message: format!("delete failed: {err}"),
        })?;

    if result.rows_affected() == 0 {
        return Err(AgentError {
            status: StatusCode::NOT_FOUND,
            message: "record not found".to_string(),
        });
    }

    Ok(json!({"id": id, "table": table, "deleted": true}))
}

async fn create_branch(
    pool: &SqlitePool,
    auth: &AuthUser,
    arguments: &Value,
) -> Result<Value, AgentError> {
    let branch_name = required_str(arguments, "branch_name")?.trim();
    let branch_type = arguments["branch_type"].as_str().unwrap_or("branch");
    let contact_person = arguments["contact_person"].as_str();
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO branches (id, company_id, branch_name, branch_type, contact_person, status)
        VALUES (?, ?, ?, ?, ?, 'active')
        "#,
    )
    .bind(&id)
    .bind(&auth.company_id)
    .bind(branch_name)
    .bind(branch_type)
    .bind(contact_person)
    .execute(pool)
    .await
    .map_err(|err| AgentError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: format!("branch create failed: {err}"),
    })?;

    Ok(json!({
        "id": id,
        "branch_name": branch_name,
        "branch_type": branch_type,
        "contact_person": contact_person
    }))
}

async fn approve_payment(
    pool: &SqlitePool,
    company_id: &str,
    arguments: &Value,
) -> Result<Value, AgentError> {
    let id = required_str(arguments, "id")?.to_string();
    let mut tx = pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(|_| AgentError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "payment transaction failed".into(),
        })?;
    let result = sqlx::query(
        "UPDATE payments SET status = 'paid', payment_date = COALESCE(payment_date, date('now')), updated_at = CURRENT_TIMESTAMP WHERE id = ? AND company_id = ?",
    )
    .bind(&id)
    .bind(company_id)
    .execute(&mut *tx)
    .await
    .map_err(|err| AgentError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: format!("payment approve failed: {err}"),
    })?;

    if result.rows_affected() == 0 {
        return Err(AgentError {
            status: StatusCode::NOT_FOUND,
            message: "payment not found".to_string(),
        });
    }

    let (invoice_id, salary_id, payment_date): (Option<String>, Option<String>, Option<String>) =
        sqlx::query_as("SELECT invoice_id,salary_record_id,payment_date FROM payments WHERE id=? AND company_id=?")
            .bind(&id).bind(company_id).fetch_one(&mut *tx).await.map_err(|_| AgentError {
                status: StatusCode::INTERNAL_SERVER_ERROR, message: "payment links lookup failed".into(),
            })?;
    crate::app::workflows::sync_payment_links(
        &mut tx,
        company_id,
        &id,
        payment_date.as_deref(),
        invoice_id.as_deref(),
        salary_id.as_deref(),
    )
    .await
    .map_err(|err| AgentError {
        status: err.0,
        message: "payment settlement failed".into(),
    })?;
    tx.commit().await.map_err(|_| AgentError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: "payment commit failed".into(),
    })?;

    Ok(json!({"id": id, "status": "paid"}))
}

fn safe_filename(original_name: &str) -> String {
    let cleaned: String = original_name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.trim_matches('_').is_empty() {
        "upload.bin".to_string()
    } else {
        cleaned
    }
}

async fn upload_agent_file(
    pool: &SqlitePool,
    auth: &AuthUser,
    arguments: &Value,
) -> Result<Value, AgentError> {
    let original_name = required_str(arguments, "original_name")?.trim();
    let content_base64 = required_str(arguments, "content_base64")?;
    let content_type = arguments["content_type"]
        .as_str()
        .unwrap_or("application/octet-stream");
    let entity_type = arguments["entity_type"].as_str();
    let entity_id = arguments["entity_id"].as_str();
    let description = arguments["description"].as_str();

    let bytes = general_purpose::STANDARD
        .decode(content_base64.as_bytes())
        .map_err(|err| AgentError {
            status: StatusCode::BAD_REQUEST,
            message: format!("invalid base64 file content: {err}"),
        })?;

    if bytes.len() > MAX_AGENT_FILE_SIZE {
        return Err(AgentError {
            status: StatusCode::PAYLOAD_TOO_LARGE,
            message: "file exceeds 10MB agent upload limit".to_string(),
        });
    }

    let id = Uuid::new_v4().to_string();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    hasher.update(id.as_bytes());
    let digest = format!("{:x}", hasher.finalize());
    let filename = format!("{}_{}", &digest[..16], safe_filename(original_name));
    let upload_path = PathBuf::from(UPLOAD_DIR).join(&auth.company_id);

    fs::create_dir_all(&upload_path)
        .await
        .map_err(|err| AgentError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("create upload directory failed: {err}"),
        })?;
    fs::write(upload_path.join(&filename), &bytes)
        .await
        .map_err(|err| AgentError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("write upload file failed: {err}"),
        })?;

    sqlx::query(
        r#"
        INSERT INTO files (
            id, company_id, filename, original_name, content_type, size,
            entity_type, entity_id, description, uploaded_by
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&auth.company_id)
    .bind(&filename)
    .bind(original_name)
    .bind(content_type)
    .bind(bytes.len() as i64)
    .bind(entity_type)
    .bind(entity_id)
    .bind(description)
    .bind(&auth.user_id)
    .execute(pool)
    .await
    .map_err(|err| AgentError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: format!("file metadata insert failed: {err}"),
    })?;

    Ok(json!({
        "id": id,
        "filename": filename,
        "original_name": original_name,
        "content_type": content_type,
        "size": bytes.len() as i64,
        "entity_type": entity_type,
        "entity_id": entity_id,
        "description": description
    }))
}

async fn onboarding_progress_value(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<Value, AgentError> {
    crate::api::onboarding::build_progress(pool, company_id)
        .await
        .map_err(|err| AgentError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("onboarding progress failed: {err}"),
        })
}

async fn count_table(pool: &SqlitePool, table: &str, company_id: &str) -> Result<i64, AgentError> {
    let sql = format!("SELECT COUNT(*) FROM {table} WHERE company_id = ?");
    sqlx::query_scalar(&sql)
        .bind(company_id)
        .fetch_one(pool)
        .await
        .map_err(|err| AgentError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("count failed for {table}: {err}"),
        })
}

async fn ensure_employee(pool: &SqlitePool, company_id: &str) -> Result<String, AgentError> {
    if let Some(id) = sqlx::query_scalar::<_, String>(
        "SELECT id FROM employees WHERE company_id = ? ORDER BY created_at LIMIT 1",
    )
    .bind(company_id)
    .fetch_optional(pool)
    .await
    .map_err(|err| AgentError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: format!("employee lookup failed: {err}"),
    })? {
        return Ok(id);
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO employees (id, company_id, name, position, status, base_salary) VALUES (?, ?, 'Agent Onboarding Employee', 'Operations', 'active', 1000)",
    )
    .bind(&id)
    .bind(company_id)
    .execute(pool)
    .await
    .map_err(|err| AgentError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: format!("employee autofill failed: {err}"),
    })?;
    Ok(id)
}

/// Fetch-or-create a default client, recording creation. Quotes require a client_id.
async fn ensure_client(
    pool: &SqlitePool,
    company_id: &str,
    created: &mut Vec<&'static str>,
) -> Result<String, AgentError> {
    if let Some(id) = sqlx::query_scalar::<_, String>(
        "SELECT id FROM clients WHERE company_id = ? ORDER BY created_at LIMIT 1",
    )
    .bind(company_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_db_error)?
    {
        return Ok(id);
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO clients (id, company_id, name, client_type) VALUES (?, ?, 'Agent Onboarding Client', 'active')",
    )
    .bind(&id)
    .bind(company_id)
    .execute(pool)
    .await
    .map_err(internal_db_error)?;
    created.push("clients");
    Ok(id)
}

async fn onboarding_autofill(
    pool: &SqlitePool,
    auth: &AuthUser,
    arguments: &Value,
) -> Result<Value, AgentError> {
    let mode = arguments["mode"]
        .as_str()
        .unwrap_or("minimal_verified_defaults");
    if mode != "minimal_verified_defaults" {
        return Err(AgentError {
            status: StatusCode::BAD_REQUEST,
            message: "unsupported onboarding autofill mode".to_string(),
        });
    }

    let company_id = &auth.company_id;
    let mut created = Vec::new();

    if count_table(pool, "company_info", company_id).await? == 0 {
        sqlx::query("INSERT INTO company_info (id, company_id, company_name) VALUES (?, ?, 'Agent Onboarding Company')")
            .bind(Uuid::new_v4().to_string())
            .bind(company_id)
            .execute(pool)
            .await
            .map_err(internal_db_error)?;
        created.push("company");
    }
    if count_table(pool, "branches", company_id).await? == 0 {
        sqlx::query("INSERT INTO branches (id, company_id, branch_name, branch_type, status) VALUES (?, ?, 'Agent Main Branch', 'headquarters', 'active')")
            .bind(Uuid::new_v4().to_string())
            .bind(company_id)
            .execute(pool)
            .await
            .map_err(internal_db_error)?;
        created.push("branches");
    }
    if count_table(pool, "rent_records", company_id).await? == 0 {
        sqlx::query("INSERT INTO rent_records (id, company_id, location_name, property_type, monthly_rent, status) VALUES (?, ?, 'Agent Office', 'office', 1000, 'active')")
            .bind(Uuid::new_v4().to_string())
            .bind(company_id)
            .execute(pool)
            .await
            .map_err(internal_db_error)?;
        created.push("rents");
    }
    if count_table(pool, "licenses", company_id).await? == 0 {
        sqlx::query("INSERT INTO licenses (id, company_id, license_name, license_type, status) VALUES (?, ?, 'Agent Business License', 'business', 'valid')")
            .bind(Uuid::new_v4().to_string())
            .bind(company_id)
            .execute(pool)
            .await
            .map_err(internal_db_error)?;
        created.push("licenses");
    }

    let employee_id = ensure_employee(pool, company_id).await?;
    if !created.contains(&"employees") && count_table(pool, "employees", company_id).await? == 1 {
        created.push("employees");
    }

    if count_table(pool, "assets", company_id).await? == 0 {
        sqlx::query("INSERT INTO assets (id, company_id, name, asset_type, status, purchase_price, current_value) VALUES (?, ?, 'Agent Laptop', 'physical', 'active', 5000, 5000)")
            .bind(Uuid::new_v4().to_string())
            .bind(company_id)
            .execute(pool)
            .await
            .map_err(internal_db_error)?;
        created.push("assets");
    }
    if count_table(pool, "invoices", company_id).await? == 0 {
        // Income (销项) invoice: the onboarding "invoices" module only counts
        // receivable/output/income types, and the payment default below links
        // to it so the "payments" module completes too.
        sqlx::query("INSERT INTO invoices (id, company_id, invoice_number, invoice_type, amount, status, supplier) VALUES (?, ?, 'AGENT-INV-001', 'income', 1000, 'pending', 'Agent Client')")
            .bind(Uuid::new_v4().to_string())
            .bind(company_id)
            .execute(pool)
            .await
            .map_err(internal_db_error)?;
        created.push("invoices");
    }
    if count_table(pool, "salary_records", company_id).await? == 0 {
        let period = chrono::Utc::now().format("%Y-%m").to_string();
        sqlx::query("INSERT INTO salary_records (id, company_id, employee_id, employee_name, period, base_salary, net_salary, status) VALUES (?, ?, ?, 'Agent Onboarding Employee', ?, 1000, 1000, 'pending')")
            .bind(Uuid::new_v4().to_string())
            .bind(company_id)
            .bind(&employee_id)
            .bind(period)
            .execute(pool)
            .await
            .map_err(internal_db_error)?;
        created.push("salaries");
    }
    if count_table(pool, "projects", company_id).await? == 0 {
        sqlx::query("INSERT INTO projects (id, company_id, name, project_type, status, progress, budget) VALUES (?, ?, 'Agent Setup Project', 'internal', 'active', 10, 10000)")
            .bind(Uuid::new_v4().to_string())
            .bind(company_id)
            .execute(pool)
            .await
            .map_err(internal_db_error)?;
        created.push("projects");
    }
    if count_table(pool, "budgets", company_id).await? == 0 {
        sqlx::query("INSERT INTO budgets (id, company_id, category, budget_type, period, budget_amount, status) VALUES (?, ?, 'operations', 'monthly', ?, 10000, 'active')")
            .bind(Uuid::new_v4().to_string())
            .bind(company_id)
            .bind(chrono::Utc::now().format("%Y-%m").to_string())
            .execute(pool)
            .await
            .map_err(internal_db_error)?;
        created.push("budgets");
    }
    if count_table(pool, "contracts", company_id).await? == 0 {
        sqlx::query("INSERT INTO contracts (id, company_id, contract_name, contract_type, status, amount) VALUES (?, ?, 'Agent Service Contract', 'service', 'active', 1000)")
            .bind(Uuid::new_v4().to_string())
            .bind(company_id)
            .execute(pool)
            .await
            .map_err(internal_db_error)?;
        created.push("contracts");
    }

    // Tier 1 — value path defaults (client → quote → time → payment)
    let client_id = ensure_client(pool, company_id, &mut created).await?;
    if count_table(pool, "quotes", company_id).await? == 0 {
        sqlx::query("INSERT INTO quotes (id, company_id, client_id, quote_number, status, total) VALUES (?, ?, ?, 'AGENT-QT-001', 'draft', 1000)")
            .bind(Uuid::new_v4().to_string())
            .bind(company_id)
            .bind(&client_id)
            .execute(pool)
            .await
            .map_err(internal_db_error)?;
        created.push("quotes");
    }
    if count_table(pool, "time_entries", company_id).await? == 0 {
        sqlx::query("INSERT INTO time_entries (id, company_id, description, date, duration_minutes, hourly_rate, amount, status) VALUES (?, ?, 'Agent Onboarding Work', date('now'), 60, 100, 100, 'logged')")
            .bind(Uuid::new_v4().to_string())
            .bind(company_id)
            .execute(pool)
            .await
            .map_err(internal_db_error)?;
        created.push("time");
    }
    if count_table(pool, "payments", company_id).await? == 0 {
        // The onboarding "payments" module only counts payments linked to an
        // invoice, so attach the default payment to an income invoice.
        let invoice_id: Option<String> = sqlx::query_scalar(
            "SELECT id FROM invoices WHERE company_id = ? AND LOWER(COALESCE(invoice_type, '')) IN ('receivable', 'output', 'income', '销项') LIMIT 1",
        )
        .bind(company_id)
        .fetch_optional(pool)
        .await
        .map_err(internal_db_error)?;
        sqlx::query("INSERT INTO payments (id, company_id, payment_type, amount, status, invoice_id) VALUES (?, ?, 'income', 1000, 'completed', ?)")
            .bind(Uuid::new_v4().to_string())
            .bind(company_id)
            .bind(&invoice_id)
            .execute(pool)
            .await
            .map_err(internal_db_error)?;
        created.push("payments");
    }

    let mut progress = onboarding_progress_value(pool, company_id).await?;
    progress["created_modules"] = json!(created);
    Ok(progress)
}

fn internal_db_error(err: sqlx::Error) -> AgentError {
    AgentError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: format!("database write failed: {err}"),
    }
}

async fn settings_overview(pool: &SqlitePool, company_id: &str) -> Result<Value, AgentError> {
    let api_keys: i64 = count_table(pool, "api_keys", company_id).await?;
    let profiles: i64 = count_table(pool, "profiles", company_id).await?;
    let files: i64 = count_table(pool, "files", company_id).await?;
    Ok(json!({
        "api_keys": api_keys,
        "profiles": profiles,
        "files": files,
        "agent_gateway": {
            "rest": "/api/v1/agent",
            "mcp": "/api/v1/mcp"
        }
    }))
}
