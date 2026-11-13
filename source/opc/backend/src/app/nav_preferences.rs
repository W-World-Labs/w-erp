//! Per-user sidebar preferences. Catalog lives in the frontend; this store
//! keeps show/hide and order JSON scoped to (user_id, company_id).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sqlx::SqlitePool;
use std::collections::BTreeMap;

pub const LOCKED_NAV_IDS: &[&str] = &["dashboard", "settings"];
const MAX_IDS: usize = 64;
const MAX_ID_LEN: usize = 64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NavPreferences {
    pub version: u32,
    pub hidden_ids: Vec<String>,
    pub category_order: Vec<String>,
    pub item_order: BTreeMap<String, Vec<String>>,
}

impl Default for NavPreferences {
    fn default() -> Self {
        Self {
            version: 1,
            hidden_ids: Vec::new(),
            category_order: Vec::new(),
            item_order: BTreeMap::new(),
        }
    }
}

fn valid_id(raw: &str) -> Option<String> {
    let id = raw.trim();
    if id.is_empty() || id.len() > MAX_ID_LEN {
        return None;
    }
    let mut chars = id.chars();
    let first = chars.next()?;
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return None;
    }
    if !chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_') {
        return None;
    }
    Some(id.to_string())
}

fn unique_ids(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut out = Vec::new();
    for id in values {
        if out.iter().any(|existing| existing == &id) {
            continue;
        }
        out.push(id);
        if out.len() >= MAX_IDS {
            break;
        }
    }
    out
}

fn parse_id_list(value: Option<&Value>) -> Vec<String> {
    let Some(Value::Array(items)) = value else {
        return Vec::new();
    };
    unique_ids(
        items
            .iter()
            .filter_map(|item| item.as_str().and_then(valid_id)),
    )
}

pub fn sanitize(raw: &Value) -> NavPreferences {
    let obj = raw.as_object();
    let hidden_ids = parse_id_list(obj.and_then(|map| map.get("hidden_ids")))
        .into_iter()
        .filter(|id| !LOCKED_NAV_IDS.contains(&id.as_str()))
        .collect();
    let category_order = parse_id_list(obj.and_then(|map| map.get("category_order")));
    let mut item_order = BTreeMap::new();
    if let Some(Value::Object(map)) = obj.and_then(|m| m.get("item_order")) {
        for (key, value) in map {
            let Some(category_id) = valid_id(key) else {
                continue;
            };
            if item_order.len() >= 16 {
                break;
            }
            item_order.insert(category_id, parse_id_list(Some(value)));
        }
    }
    NavPreferences {
        version: 1,
        hidden_ids,
        category_order,
        item_order,
    }
}

pub fn to_json(prefs: &NavPreferences, updated_at: Option<&str>) -> Value {
    let mut item_order = Map::new();
    for (key, value) in &prefs.item_order {
        item_order.insert(key.clone(), json!(value));
    }
    json!({
        "version": prefs.version,
        "hidden_ids": prefs.hidden_ids,
        "category_order": prefs.category_order,
        "item_order": Value::Object(item_order),
        "locked_ids": LOCKED_NAV_IDS,
        "updated_at": updated_at,
    })
}

pub async fn load(
    pool: &SqlitePool,
    user_id: &str,
    company_id: &str,
) -> Result<(NavPreferences, Option<String>), sqlx::Error> {
    let row: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT preferences, updated_at FROM user_nav_preferences \
         WHERE user_id = ? AND company_id = ?",
    )
    .bind(user_id)
    .bind(company_id)
    .fetch_optional(pool)
    .await?;
    let Some((raw, updated_at)) = row else {
        return Ok((NavPreferences::default(), None));
    };
    let parsed: Value = serde_json::from_str(&raw).unwrap_or_else(|_| json!({}));
    Ok((
        sanitize(&parsed),
        updated_at.filter(|value| !value.is_empty()),
    ))
}

pub async fn save(
    pool: &SqlitePool,
    user_id: &str,
    company_id: &str,
    prefs: &NavPreferences,
) -> Result<String, sqlx::Error> {
    let blob = serde_json::to_string(prefs).unwrap_or_else(|_| "{}".to_string());
    sqlx::query(
        "INSERT INTO user_nav_preferences (user_id, company_id, preferences, updated_at) \
         VALUES (?, ?, ?, datetime('now')) \
         ON CONFLICT(user_id, company_id) DO UPDATE SET \
         preferences = excluded.preferences, updated_at = datetime('now')",
    )
    .bind(user_id)
    .bind(company_id)
    .bind(&blob)
    .execute(pool)
    .await?;
    let updated_at: String = sqlx::query_scalar(
        "SELECT updated_at FROM user_nav_preferences WHERE user_id = ? AND company_id = ?",
    )
    .bind(user_id)
    .bind(company_id)
    .fetch_one(pool)
    .await?;
    Ok(updated_at)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_drops_locked_and_invalid_ids() {
        let raw = json!({
            "version": 9,
            "hidden_ids": ["invoices", "dashboard", "NOPE", "", "settings", "ok-id"],
            "category_order": ["business", "business", "overview"],
            "item_order": {
                "finance": ["payments", "!!!", "invoices"],
                "Bad Key": ["x"],
            }
        });
        let prefs = sanitize(&raw);
        assert_eq!(prefs.version, 1);
        assert_eq!(prefs.hidden_ids, vec!["invoices", "ok-id"]);
        assert_eq!(prefs.category_order, vec!["business", "overview"]);
        assert_eq!(
            prefs.item_order.get("finance").unwrap(),
            &vec!["payments".to_string(), "invoices".to_string()]
        );
        assert!(!prefs.item_order.contains_key("Bad Key"));
    }
}
