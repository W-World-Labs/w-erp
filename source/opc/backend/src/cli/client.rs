use comfy_table::{Table, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL_CONDENSED};
use reqwest::{Client, header};
use serde_json::{Value, json};

use super::Cli;

pub(crate) struct CliClient {
    pub(crate) client: Client,
    pub(crate) base_url: String,
    pub(crate) format: String,
    pub(crate) quiet: bool,
}

impl CliClient {
    pub(crate) fn new(cli: &Cli) -> Result<Self, Box<dyn std::error::Error>> {
        let mut headers = header::HeaderMap::new();
        if let Some(ref session) = cli.session {
            headers.insert(
                "authorization",
                header::HeaderValue::from_str(&format!("Bearer {}", session))?,
            );
        }
        if let Some(ref key) = cli.api_key {
            headers.insert("x-api-key", header::HeaderValue::from_str(key)?);
        }
        let client = Client::builder().default_headers(headers).build()?;
        Ok(Self {
            client,
            base_url: cli.server.trim_end_matches('/').to_string(),
            format: cli.format.clone(),
            quiet: cli.quiet,
        })
    }

    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    pub(crate) fn is_json(&self) -> bool {
        self.format == "json"
    }

    pub(crate) async fn get(&self, path: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let resp = self.client.get(self.url(path)).send().await?;
        let status = resp.status();
        let body: Value = resp
            .json()
            .await
            .unwrap_or_else(|_| json!({"error": "non-JSON response"}));
        if !status.is_success() {
            let err = body["error"].as_str().unwrap_or("unknown error");
            return Err(format!("{} — {}", status, err).into());
        }
        Ok(body)
    }

    pub(crate) async fn post(
        &self,
        path: &str,
        body: &Value,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let resp = self.client.post(self.url(path)).json(body).send().await?;
        let status = resp.status();
        let body: Value = resp
            .json()
            .await
            .unwrap_or_else(|_| json!({"error": "non-JSON response"}));
        if !status.is_success() {
            let err = body["error"].as_str().unwrap_or("unknown error");
            return Err(format!("{} — {}", status, err).into());
        }
        Ok(body)
    }

    pub(crate) async fn put(
        &self,
        path: &str,
        body: &Value,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let resp = self.client.put(self.url(path)).json(body).send().await?;
        let status = resp.status();
        let body: Value = resp
            .json()
            .await
            .unwrap_or_else(|_| json!({"error": "non-JSON response"}));
        if !status.is_success() {
            let err = body["error"].as_str().unwrap_or("unknown error");
            return Err(format!("{} — {}", status, err).into());
        }
        Ok(body)
    }

    pub(crate) async fn delete_req(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let resp = self.client.delete(self.url(path)).send().await?;
        let status = resp.status();
        if !status.is_success() {
            let body: Value = resp.json().await.unwrap_or_default();
            let err = body["error"].as_str().unwrap_or("delete failed");
            return Err(format!("{} — {}", status, err).into());
        }
        Ok(())
    }

    pub(crate) async fn get_bytes(
        &self,
        path: &str,
    ) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
        let resp = self.client.get(self.url(path)).send().await?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()).into());
        }
        let ct = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        let bytes = resp.bytes().await?.to_vec();
        Ok((bytes, ct))
    }

    pub(crate) fn print_value(&self, v: &Value) {
        match self.format.as_str() {
            "json" => println!("{}", serde_json::to_string_pretty(v).unwrap()),
            _ => self.print_value_table(v),
        }
    }

    fn print_value_table(&self, v: &Value) {
        if let Some(obj) = v.as_object() {
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL_CONDENSED)
                .apply_modifier(UTF8_ROUND_CORNERS);
            table.set_header(vec!["Field", "Value"]);
            for (k, val) in obj {
                if k == "data" {
                    self.print_list_or_item(val);
                    return;
                }
                table.add_row(vec![k.as_str(), &val_to_str(val)]);
            }
            println!("{table}");
        } else {
            println!("{}", serde_json::to_string_pretty(v).unwrap());
        }
    }

    pub(crate) fn print_list_or_item(&self, v: &Value) {
        match self.format.as_str() {
            "json" => println!("{}", serde_json::to_string_pretty(v).unwrap()),
            "csv" => self.print_csv(v),
            _ => self.print_auto_table(v),
        }
    }

    fn print_auto_table(&self, v: &Value) {
        match v {
            Value::Array(arr) if !arr.is_empty() => {
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL_CONDENSED)
                    .apply_modifier(UTF8_ROUND_CORNERS);
                if let Some(obj) = arr[0].as_object() {
                    let headers: Vec<&str> = obj.keys().map(|s| s.as_str()).collect();
                    table.set_header(&headers);
                    for item in arr {
                        if let Some(o) = item.as_object() {
                            let row: Vec<String> = headers
                                .iter()
                                .map(|h| val_to_str(o.get(*h).unwrap_or(&Value::Null)))
                                .collect();
                            table.add_row(row);
                        }
                    }
                }
                println!("{table}");
            }
            Value::Object(obj) => {
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL_CONDENSED)
                    .apply_modifier(UTF8_ROUND_CORNERS);
                table.set_header(vec!["Field", "Value"]);
                for (k, val) in obj {
                    table.add_row(vec![k.as_str(), &val_to_str(val)]);
                }
                println!("{table}");
            }
            _ => println!("{}", serde_json::to_string_pretty(v).unwrap()),
        }
    }

    fn print_csv(&self, v: &Value) {
        if let Value::Array(arr) = v {
            if let Some(first) = arr.first().and_then(|f| f.as_object()) {
                let headers: Vec<&str> = first.keys().map(|s| s.as_str()).collect();
                println!("{}", headers.join(","));
                for item in arr {
                    if let Some(o) = item.as_object() {
                        let row: Vec<String> = headers
                            .iter()
                            .map(|h| csv_escape(&val_to_str(o.get(*h).unwrap_or(&Value::Null))))
                            .collect();
                        println!("{}", row.join(","));
                    }
                }
            }
        }
    }
}

fn val_to_str(v: &Value) -> String {
    match v {
        Value::String(s) => {
            if s.len() > 60 {
                format!("{}…", &s[..57])
            } else {
                s.clone()
            }
        }
        Value::Null => "—".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        _ => serde_json::to_string(v).unwrap_or_default(),
    }
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
