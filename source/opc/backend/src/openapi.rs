use serde_json::{Map, Value, json};
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
struct RouteSpec {
    method: String,
    path: String,
    tag: String,
    protected: bool,
}

struct ModuleSource {
    prefix: &'static str,
    source: &'static str,
    function: &'static str,
    tag: &'static str,
}

const API_MODULES: &[ModuleSource] = &[
    module(
        "/assets",
        include_str!("api/assets.rs"),
        "pub fn router",
        "assets",
    ),
    module(
        "/invoices",
        include_str!("api/invoices.rs"),
        "pub fn router",
        "invoices",
    ),
    module(
        "/budgets",
        include_str!("api/budgets.rs"),
        "pub fn router",
        "budgets",
    ),
    module(
        "/employees",
        include_str!("api/employees.rs"),
        "pub fn router",
        "employees",
    ),
    module(
        "/bank",
        include_str!("api/bank.rs"),
        "pub fn router",
        "bank",
    ),
    module(
        "/companies",
        include_str!("api/companies.rs"),
        "pub fn router",
        "companies",
    ),
    module(
        "/files",
        include_str!("api/files.rs"),
        "pub fn router",
        "files",
    ),
    module(
        "/agent",
        include_str!("api/agent.rs"),
        "pub fn router",
        "agent",
    ),
    module(
        "",
        include_str!("api/agent.rs"),
        "pub fn mcp_router",
        "agent",
    ),
    module("/ai", include_str!("api/ai.rs"), "pub fn router", "ai"),
    module(
        "/workbench",
        include_str!("api/workbench.rs"),
        "pub fn router",
        "workbench",
    ),
    module(
        "/imports",
        include_str!("api/imports.rs"),
        "pub fn router",
        "imports",
    ),
    module(
        "",
        include_str!("api/invoice_mail_imports.rs"),
        "pub fn router",
        "invoice-mail-imports",
    ),
    module("/kpi", include_str!("api/kpi.rs"), "pub fn router", "kpi"),
    module(
        "/policies",
        include_str!("api/policies.rs"),
        "pub fn router",
        "policies",
    ),
    module(
        "/global_opc_hubs",
        include_str!("api/global_opc_hubs.rs"),
        "pub fn router",
        "global-opc",
    ),
    module(
        "/api-keys",
        include_str!("api/api_keys.rs"),
        "pub fn router",
        "api-keys",
    ),
    module(
        "",
        include_str!("api/operations.rs"),
        "pub fn router",
        "operations",
    ),
    module(
        "",
        include_str!("api/collections.rs"),
        "pub fn router",
        "collections",
    ),
    module("", include_str!("api/nav.rs"), "pub fn router", "settings"),
    module(
        "",
        include_str!("api/backups.rs"),
        "pub fn router",
        "settings",
    ),
    module(
        "",
        include_str!("api/reports.rs"),
        "pub fn router",
        "reports",
    ),
    module(
        "",
        include_str!("api/generic.rs"),
        "pub fn router",
        "business",
    ),
];

const fn module(
    prefix: &'static str,
    source: &'static str,
    function: &'static str,
    tag: &'static str,
) -> ModuleSource {
    ModuleSource {
        prefix,
        source,
        function,
        tag,
    }
}

pub fn document() -> Value {
    let mut paths = Map::new();
    for route in registered_routes() {
        let path_item = paths
            .entry(route.path.clone())
            .or_insert_with(|| Value::Object(Map::new()));
        let methods = path_item
            .as_object_mut()
            .expect("OpenAPI path item must be an object");
        methods.insert(route.method.to_ascii_lowercase(), operation(&route));
    }

    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "OPC API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "OPC Rust API contract generated from current Axum route declarations. The business API is exposed under both /api/v1 and the legacy /api prefix.",
        },
        "servers": [
            { "url": "https://opc.myfastools.com", "description": "Production" },
        ],
        "paths": paths,
        "components": {
            "securitySchemes": {
                "SessionBearer": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "OPC session token",
                },
                "ApiKey": {
                    "type": "apiKey",
                    "in": "header",
                    "name": "x-api-key",
                },
            },
        },
        "tags": [
            { "name": "system" },
            { "name": "auth" },
            { "name": "business" },
            { "name": "companies" },
            { "name": "invoices" },
            { "name": "assets" },
            { "name": "files" },
            { "name": "ai" },
            { "name": "agent" },
            { "name": "operations" },
            { "name": "collections" },
            { "name": "settings" },
            { "name": "reports" },
        ],
    })
}

fn registered_routes() -> Vec<RouteSpec> {
    let mut routes = vec![
        public("GET", "/openapi.json", "system"),
        public("GET", "/api/openapi.json", "system"),
        public("GET", "/health", "system"),
        // Aliases registered in api::router and whitelisted in require_auth so
        // monitors probing under the API prefix get a real health probe.
        public("GET", "/api/health", "system"),
        public("GET", "/api/v1/health", "system"),
    ];

    routes.extend(extract_routes(
        include_str!("auth/mod.rs"),
        "",
        "pub fn router_with_config",
        "auth",
        false,
    ));

    for base in ["/api/v1", "/api"] {
        for module in API_MODULES {
            let prefix = join_paths(base, module.prefix);
            routes.extend(extract_routes(
                module.source,
                &prefix,
                module.function,
                module.tag,
                true,
            ));
        }
    }

    let mut seen = BTreeSet::new();
    routes
        .into_iter()
        .filter(|route| seen.insert((route.method.clone(), route.path.clone())))
        .collect()
}

fn public(method: &str, path: &str, tag: &str) -> RouteSpec {
    RouteSpec {
        method: method.to_string(),
        path: path.to_string(),
        tag: tag.to_string(),
        protected: false,
    }
}

fn extract_routes(
    source: &str,
    prefix: &str,
    function: &str,
    tag: &str,
    protected: bool,
) -> Vec<RouteSpec> {
    let Some(body) = function_body(source, function) else {
        return Vec::new();
    };

    let mut routes = Vec::new();
    let mut offset = 0usize;
    while let Some(relative) = body[offset..].find(".route(") {
        let route_start = offset + relative;
        let open_paren = route_start + ".route".len();
        let Some(close_paren) = matching_delimiter(body, open_paren, '(', ')') else {
            break;
        };
        let route_call = &body[open_paren + 1..close_paren];
        if let Some((path, after_path)) = first_string_literal(route_call) {
            let methods = route_methods(&route_call[after_path..]);
            let full_path = normalize_openapi_path(&join_paths(prefix, &path));
            for method in methods {
                routes.push(RouteSpec {
                    method: method.to_string(),
                    path: full_path.clone(),
                    tag: tag.to_string(),
                    protected,
                });
            }
        }
        offset = close_paren + 1;
    }
    routes
}

fn function_body<'a>(source: &'a str, function: &str) -> Option<&'a str> {
    let function_start = source.find(function)?;
    let brace = source[function_start..].find('{')? + function_start;
    let end = matching_delimiter(source, brace, '{', '}')?;
    Some(&source[brace + 1..end])
}

fn route_methods(call_tail: &str) -> Vec<&'static str> {
    let mut methods = Vec::new();
    for (method, needle) in [
        ("GET", "get("),
        ("POST", "post("),
        ("PUT", "put("),
        ("PATCH", "patch("),
        ("DELETE", "delete("),
    ] {
        if call_tail.contains(needle) {
            methods.push(method);
        }
    }
    methods
}

fn first_string_literal(input: &str) -> Option<(String, usize)> {
    let mut start = None;
    for (idx, ch) in input.char_indices() {
        if ch == '"' {
            start = Some(idx + ch.len_utf8());
            break;
        }
    }
    let start = start?;
    let mut escaped = false;
    for (relative, ch) in input[start..].char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            let end = start + relative;
            return Some((input[start..end].to_string(), end + ch.len_utf8()));
        }
    }
    None
}

fn matching_delimiter(source: &str, open_index: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    for (idx, ch) in source
        .char_indices()
        .skip_while(|(idx, _)| *idx < open_index)
    {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
        } else if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return Some(idx);
            }
        }
    }
    None
}

fn join_paths(left: &str, right: &str) -> String {
    let left = left.trim_end_matches('/');
    let right = right.trim_start_matches('/');
    match (left.is_empty(), right.is_empty()) {
        (true, true) => "/".to_string(),
        (true, false) => format!("/{right}"),
        (false, true) => left.to_string(),
        (false, false) => format!("{left}/{right}"),
    }
}

fn normalize_openapi_path(path: &str) -> String {
    let path = path.replace("{*", "{");
    if path.len() > 1 {
        path.trim_end_matches('/').to_string()
    } else {
        path
    }
}

fn operation(route: &RouteSpec) -> Value {
    let mut value = json!({
        "summary": route_summary(route),
        "operationId": operation_id(route),
        "tags": [route.tag],
        "responses": {
            "200": {
                "description": "OK",
                "content": {
                    "application/json": {
                        "schema": {
                            "type": "object",
                            "additionalProperties": true,
                        },
                    },
                },
            },
            "401": { "description": "Authentication required" },
            "403": { "description": "Current company membership or operation permission denied" },
            "404": { "description": "Not found" },
            "500": { "description": "Internal server error" },
        },
        "x-opc-protected": route.protected,
    });

    if route.protected {
        value["security"] = json!([
            { "SessionBearer": [] },
            { "ApiKey": [] },
        ]);
    }

    if route.path.contains("/settings/backups") || route.path.ends_with("/settings/restore") {
        value["description"] = json!(
            "Instance-wide backups contain all tenants. Tenant sessions and API keys, including company owners and admins, cannot list, create, download or restore these archives."
        );
        value["responses"].as_object_mut().unwrap().remove("200");
        value["responses"]["403"] = json!({"description":"Instance backup operations are not available to product tenants"});
    }

    if route.method == "POST" && route.path.ends_with("/api-keys") {
        value["description"] = json!(
            "Creating API keys requires a user session and current company membership. API keys cannot mint additional credentials."
        );
        value["security"] = json!([{ "SessionBearer": [] }]);
    }

    if route.path.ends_with("/ai/chat")
        || route.path.ends_with("/ai/contract-review")
        || route.path.ends_with("/ai/parse-document")
        || route.path.ends_with("/ai/analyze")
        || route.path.ends_with("/ai/audit")
    {
        value["responses"]["502"] = json!({"description": "Model unavailable, malformed output or no usable content; no successful result"});
        value["responses"]["503"] = json!({"description": "Text AI is not configured"});
        value["responses"]["400"] = json!({"description": "Empty or invalid input"});
        value["responses"]["413"] = json!({"description": "Input exceeds size or history limits"});
    }
    if route.path.ends_with("/ai/chat") {
        value["description"] = json!(
            "Founder decision assistance grounded in current-company ledger aggregates. Ledger data may be incomplete and is not bank balance. Supplied context and history are untrusted data; only the server sets system instructions. Chat is read-only. Message: nonblank, at most 16000 UTF-8 bytes; history: at most 40 turns and 64000 content bytes; context: at most 16000 bytes."
        );
    }
    if route.path.ends_with("/ai/contract-review") {
        value["description"] = json!(
            "Review supplied contract text, not a guarantee of completeness or legal safety. contract_text must be nonblank and at most 100000 UTF-8 bytes; contract_type at most 200 bytes and counterparty_type at most 100 bytes."
        );
    }

    let parameters = path_parameters(&route.path);
    if !parameters.is_empty() {
        value["parameters"] = Value::Array(parameters);
    }

    if matches!(route.method.as_str(), "POST" | "PUT" | "PATCH") {
        value["requestBody"] = json!({
            "required": false,
            "content": {
                "application/json": {
                    "schema": {
                        "type": "object",
                        "additionalProperties": true,
                    },
                },
            },
        });
    }

    if route.path.ends_with("/email/send") {
        value["description"] = json!(
            "Current-company authenticated mail send. A selected w-mail provider uses only the operator-managed project credential/mailbox, never tenant SMTP credentials. W-Mail requires caller-stable operation_id (8..128 ASCII letters/digits/_/-); retain it across retries and use a new key only for a new intended message. Same key with changed content returns409. POST-start is durable before network; unknown operations are reconciled by GET only (at most6 checks/2days), then history remains indefinitely. provider_accepted is not recipient delivery. Configured SMTP remains saved but returns503 smtp_transport_unsupported; it no longer simulates successful delivery."
        );
        value["requestBody"]["required"] = json!(true);
        value["requestBody"]["content"]["application/json"]["schema"] = json!({"type":"object","required":["to","subject"],"properties":{"to":{"type":"string"},"subject":{"type":"string","maxLength":998},"body":{"type":"string","description":"Optional HTML body; at most256KiB"},"text":{"type":"string","description":"Optional plain text; at most256KiB"},"operation_id":{"type":"string","pattern":"^[A-Za-z0-9_-]{8,128}$","description":"Required for W-Mail; original stable client operation key"}}});
        value["responses"]["202"] = json!({"description":"Submitted or unknown; inspect status, delivered, log_id, job_id, provider_id. No delivery claim for queued/unknown/provider_accepted."});
        value["responses"]["400"] =
            json!({"description":"Invalid mail content or missing/invalid W-Mail operation_id"});
        value["responses"]["409"] =
            json!({"description":"Original operation key was reused with different content"});
        value["responses"]["502"] = json!({"description":"No provider/incomplete SMTP configuration or definite recipient failure"});
        value["responses"]["503"] = json!({"description":"Managed W-Mail is unavailable, or configured SMTP transport is unsupported; no simulated send. SMTP failure returns fixed error plus log_id for the preserved current-company attempt history."});
    }
    if route.path.contains("/email_configurations") {
        value["description"] = json!(
            "Current-company email configuration. provider is smtp (default) or w-mail. w-mail selects the instance-managed project channel; platform keys are never returned or stored in this tenant row. Existing SMTP fields remain saved and selectable; no SMTP transport is implemented. Mail delivery history retains all previous operations, including unknown after reconciliation stops."
        );
        if matches!(route.method.as_str(), "POST" | "PUT") {
            value["requestBody"]["content"]["application/json"]["schema"] = json!({"type":"object","properties":{"provider":{"type":"string","enum":["smtp","w-mail"],"default":"smtp"},"enabled":{"type":"integer","enum":[0,1]}},"additionalProperties":true});
        }
    }

    if route.path.ends_with("/ai/models") {
        value["description"] = json!(
            "Lists model IDs visible to the server-configured gateway credential via its OpenAI-compatible /models endpoint. No credentials or provider metadata are returned. Supports W-MaaS /v1/models; this does not change deployment configuration."
        );
        value["responses"]["502"] = json!({"description":"Catalog unavailable or invalid; do not treat as an empty successful list"});
        value["responses"]["503"] = json!({"description":"Text AI gateway not configured"});
        value["responses"]["200"]["content"]["application/json"]["schema"] = json!({
            "type":"object", "required":["models","default_model"],
            "properties":{
                "models":{"type":"array","items":{"type":"object","required":["id"],"properties":{"id":{"type":"string"}}}},
                "default_model":{"type":"string"}
            }
        });
    }
    if route.path.ends_with("/ai/chat") {
        value["requestBody"]["required"] = json!(true);
        value["requestBody"]["content"]["application/json"]["schema"] = json!({
            "type":"object", "required":["message"], "additionalProperties":true,
            "properties":{
                "message":{"type":"string","description":"Nonblank; at most 16000 UTF-8 bytes"},
                "model":{"type":["string","null"],"description":"Optional request-local selection from /ai/models. Blank/omitted uses server default. Unlisted model: 400; catalog unavailable: 502. Never silently substitutes another model."},
                "context":{"type":["string","null"],"description":"Untrusted page context; at most 16000 bytes"},
                "path":{"type":["string","null"]},
                "history":{"type":["array","null"],"maxItems":40,"description":"At most 64000 total content bytes; supplied system/developer roles are treated as user data","items":{"type":"object","required":["role","content"],"properties":{"role":{"type":"string"},"content":{"type":"string"}}}}
            }
        });
    }
    if route.path.ends_with("/retrigger_review") {
        value["responses"]["400"] = json!({"description":"Contract has no reviewable content or notes; title alone is insufficient"});
    }
    if route.path.ends_with("/contracts") || route.path.ends_with("/contracts/{id}") {
        if matches!(route.method.as_str(), "POST" | "PUT") {
            value["description"] = json!(
                "Saved contract content/notes are reviewed asynchronously. Client ai_review_status, ai_risk_level, ai_review_result and ai_reviewed_at are ignored; these fields are server-owned. Each edit invalidates prior review results and queues the full saved text when present."
            );
        }
    }

    value
}

fn route_summary(route: &RouteSpec) -> String {
    format!("{} {}", route.method, route.path)
}

fn path_parameters(path: &str) -> Vec<Value> {
    let mut params = Vec::new();
    let mut rest = path;
    while let Some(start) = rest.find('{') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('}') else {
            break;
        };
        let name = &after_start[..end];
        params.push(json!({
            "name": name,
            "in": "path",
            "required": true,
            "schema": { "type": "string" },
        }));
        rest = &after_start[end + 1..];
    }
    params
}

fn operation_id(route: &RouteSpec) -> String {
    let path = route
        .path
        .trim_matches('/')
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    format!("{}_{}", route.method.to_ascii_lowercase(), path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_document_covers_current_axum_routes() {
        let spec = document();
        assert_eq!(spec["openapi"], "3.1.0");
        let paths = spec["paths"].as_object().expect("paths object");
        for path in [
            "/openapi.json",
            "/api/openapi.json",
            "/health",
            "/auth/login",
            "/api/v1/companies",
            "/api/v1/companies/{id}/members",
            "/api/v1/invoices/{id}",
            "/api/v1/invoice-mailboxes",
            "/api/v1/invoice-mailboxes/{id}/sync",
            "/api/v1/invoice-mail-imports/{id}/confirm",
            "/api/v1/reports/financial-health",
            "/api/v1/reports/monthly-close",
            "/api/v1/settings/collections",
            "/api/v1/settings/nav",
            "/api/v1/collections/history",
            "/api/files/static/{path}",
            "/api/v1/mcp",
        ] {
            assert!(paths.contains_key(path), "missing path {path}");
        }
        assert!(paths["/api/v1/companies"]["get"].is_object());
        assert!(paths["/api/v1/companies"]["post"].is_object());
        assert!(paths["/api/v1/invoices/{id}"]["put"].is_object());
        assert!(
            paths.len() >= 220,
            "expected full route coverage, got {} paths",
            paths.len()
        );
    }
}
