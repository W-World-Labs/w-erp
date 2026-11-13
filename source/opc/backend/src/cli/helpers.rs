use serde_json::{Value, json};

pub(crate) fn parse_kv(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=VALUE: no '=' in '{}'", s))?;
    let key = s[..pos].trim();
    if key.is_empty() {
        return Err(format!("invalid KEY=VALUE: empty key in '{}'", s));
    }
    Ok((key.to_string(), s[pos + 1..].to_string()))
}

pub(crate) fn parse_json_arg(name: &str, raw: &str) -> Result<Value, Box<dyn std::error::Error>> {
    serde_json::from_str(raw).map_err(|e| format!("invalid JSON for {}: {}", name, e).into())
}

pub(crate) fn read_contract_text(
    text: Option<String>,
    file: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    match (text, file) {
        (Some(_), Some(_)) => Err("provide exactly one of --text or --file".into()),
        (None, None) => Err("provide exactly one of --text or --file".into()),
        (Some(text), None) => Ok(text),
        (None, Some(file)) => Ok(std::fs::read_to_string(file)?),
    }
}

pub(crate) fn build_body(
    json_str: &Option<String>,
    fields: &[(String, String)],
) -> Result<Value, Box<dyn std::error::Error>> {
    if let Some(j) = json_str {
        return parse_json_arg("--json", j);
    }

    let mut map = serde_json::Map::new();
    for (k, v) in fields {
        if let Ok(n) = v.parse::<f64>() {
            map.insert(k.clone(), json!(n));
        } else if v == "true" {
            map.insert(k.clone(), json!(true));
        } else if v == "false" {
            map.insert(k.clone(), json!(false));
        } else {
            map.insert(k.clone(), json!(v));
        }
    }

    if map.is_empty() {
        return Err("provide --json or --field key=value".into());
    }

    Ok(Value::Object(map))
}

pub(crate) fn entity_api_path(entity: &str) -> String {
    match entity {
        "invoices" => "/api/invoices".to_string(),
        "assets" => "/api/assets".to_string(),
        "employees" => "/api/employees".to_string(),
        "budgets" => "/api/budgets".to_string(),
        "policies" => "/api/policies".to_string(),
        "global_opc_hubs" => "/api/global_opc_hubs".to_string(),
        other => format!("/api/v1/{}", other),
    }
}
