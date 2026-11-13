use super::error::{ApiError, api_err};
use crate::auth::AuthUser;
use crate::db::models::Asset;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use std::collections::HashMap;

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/", get(list_assets).post(create_asset))
        .route("/overview", get(asset_overview))
        .route("/form_options", get(asset_form_options))
        .route("/{id}/mark_maintenance", post(mark_asset_maintenance))
        .route("/{id}/mark_scrapped", post(mark_asset_scrapped))
        .route("/{id}/mark_in_use", post(mark_asset_in_use))
        .route(
            "/{id}",
            get(get_asset).put(update_asset).delete(delete_asset),
        )
        .with_state(pool)
}

#[derive(Deserialize)]
pub struct CreateAssetRequest {
    pub name: String,
    pub asset_type: Option<String>,
    pub description: Option<String>,
    pub serial_number: Option<String>,
    pub location: Option<String>,
    pub purchase_date: Option<String>,
    pub purchase_price: Option<Value>,
    pub current_value: Option<Value>,
    pub category_id: Option<String>,
    pub status: Option<String>,
    pub expiry_date: Option<String>,
    pub useful_life: Option<Value>,
    pub salvage_rate: Option<Value>,
    pub depreciation_method: Option<String>,
    pub invoice_id: Option<String>,
    pub contract_id: Option<String>,
    pub registration_number: Option<String>,
    pub holder_name: Option<String>,
    pub data_type: Option<String>,
    pub tech_stack: Option<String>,
    pub version: Option<String>,
    pub license_type: Option<String>,
    pub patent_number: Option<String>,
    pub patent_type: Option<String>,
    pub trademark_class: Option<String>,
    pub data_volume: Option<String>,
    pub data_format: Option<String>,
    pub access_url: Option<String>,
}

#[derive(Serialize)]
pub struct AssetResponse {
    pub data: Asset,
}

#[derive(Serialize)]
pub struct AssetsListResponse {
    pub data: Vec<AssetListItem>,
    pub count: i64,
}

#[derive(Serialize)]
pub struct AssetCategoryRef {
    pub name: String,
}

#[derive(Serialize)]
pub struct AssetListItem {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub asset_type: Option<String>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub serial_number: Option<String>,
    pub location: Option<String>,
    pub purchase_date: Option<String>,
    pub purchase_price: Option<f64>,
    pub current_value: Option<f64>,
    pub expiry_date: Option<String>,
    pub category_id: Option<String>,
    pub image_url: Option<String>,
    pub useful_life: Option<i64>,
    pub salvage_rate: Option<f64>,
    pub depreciation_method: Option<String>,
    pub monthly_depreciation: Option<f64>,
    pub accumulated_depreciation: Option<f64>,
    pub invoice_id: Option<String>,
    pub contract_id: Option<String>,
    pub registration_number: Option<String>,
    pub holder_name: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub asset_categories: Option<AssetCategoryRef>,
    pub expiry_status: Option<String>,
    pub days_until_expiry: Option<i64>,
    pub net_value: f64,
}

#[derive(sqlx::FromRow)]
struct AssetListRow {
    id: String,
    company_id: String,
    name: String,
    asset_type: Option<String>,
    status: Option<String>,
    description: Option<String>,
    serial_number: Option<String>,
    location: Option<String>,
    purchase_date: Option<String>,
    purchase_price: Option<f64>,
    current_value: Option<f64>,
    expiry_date: Option<String>,
    category_id: Option<String>,
    image_url: Option<String>,
    useful_life: Option<i64>,
    salvage_rate: Option<f64>,
    depreciation_method: Option<String>,
    monthly_depreciation: Option<f64>,
    accumulated_depreciation: Option<f64>,
    invoice_id: Option<String>,
    contract_id: Option<String>,
    registration_number: Option<String>,
    holder_name: Option<String>,
    created_by: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
    category_name: Option<String>,
    expiry_status: Option<String>,
    days_until_expiry: Option<i64>,
    net_value: f64,
}

#[derive(sqlx::FromRow)]
struct DataAssetMetaRow {
    id: String,
    asset_id: String,
    data_type: String,
    tech_stack: Option<String>,
    version: Option<String>,
    license_type: Option<String>,
    patent_number: Option<String>,
    patent_type: Option<String>,
    trademark_class: Option<String>,
    data_volume: Option<String>,
    data_format: Option<String>,
    access_url: Option<String>,
}

impl From<AssetListRow> for AssetListItem {
    fn from(row: AssetListRow) -> Self {
        Self {
            id: row.id,
            company_id: row.company_id,
            name: row.name,
            asset_type: row.asset_type,
            status: row.status,
            description: row.description,
            serial_number: row.serial_number,
            location: row.location,
            purchase_date: row.purchase_date,
            purchase_price: row.purchase_price,
            current_value: row.current_value,
            expiry_date: row.expiry_date,
            category_id: row.category_id,
            image_url: row.image_url,
            useful_life: row.useful_life,
            salvage_rate: row.salvage_rate,
            depreciation_method: row.depreciation_method,
            monthly_depreciation: row.monthly_depreciation,
            accumulated_depreciation: row.accumulated_depreciation,
            invoice_id: row.invoice_id,
            contract_id: row.contract_id,
            registration_number: row.registration_number,
            holder_name: row.holder_name,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
            asset_categories: row.category_name.map(|name| AssetCategoryRef { name }),
            expiry_status: row.expiry_status,
            days_until_expiry: row.days_until_expiry,
            net_value: row.net_value,
        }
    }
}

#[derive(Serialize)]
pub struct AssetOverview {
    pub total: i64,
    pub in_use: i64,
    pub total_value: f64,
    pub total_depreciation: f64,
    pub total_value_label: String,
    pub total_depreciation_label: String,
    pub expiring_soon: i64,
    pub expired: i64,
    pub type_counts: Value,
    pub assets: Vec<Value>,
    pub categories: Vec<Value>,
    pub status_options: Vec<Value>,
    pub asset_type_options: Vec<Value>,
    pub category_options: Vec<Value>,
    pub permissions: Value,
    pub empty_message: String,
    pub create_action: Value,
    pub category_create_action: Value,
}

#[derive(Deserialize)]
struct AssetOverviewQuery {
    search: Option<String>,
    status: Option<String>,
    category_id: Option<String>,
    asset_type: Option<String>,
}

#[derive(Deserialize)]
struct AssetFormOptionsQuery {
    asset_type: Option<String>,
}

#[derive(sqlx::FromRow)]
struct AssetOverviewRow {
    total: i64,
    in_use: i64,
    total_value: f64,
    total_depreciation: f64,
    physical: i64,
    virtual_count: i64,
    data: i64,
    intangible: i64,
    expiring_soon: i64,
    expired: i64,
}

#[derive(sqlx::FromRow, Clone)]
struct AssetCategoryRow {
    id: String,
    name: String,
    category_type: Option<String>,
    description: Option<String>,
}

async fn list_assets(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(q): Query<super::generic::ListQuery>,
) -> Result<Json<AssetsListResponse>, ApiError> {
    let limit = q.limit.or(q.page_size).unwrap_or(100).min(1000).max(1);
    let offset = q
        .offset
        .or_else(|| q.page.map(|p| (p.max(1) - 1) * limit))
        .unwrap_or(0)
        .max(0);
    let rows: Vec<AssetListRow> = sqlx::query_as(
        r#"
        SELECT
            a.id,
            a.company_id,
            a.name,
            a.asset_type,
            a.status,
            a.description,
            a.serial_number,
            a.location,
            a.purchase_date,
            a.purchase_price,
            a.current_value,
            a.expiry_date,
            a.category_id,
            a.image_url,
            a.useful_life,
            a.salvage_rate,
            a.depreciation_method,
            a.monthly_depreciation,
            a.accumulated_depreciation,
            a.invoice_id,
            a.contract_id,
            a.registration_number,
            a.holder_name,
            a.created_by,
            a.created_at,
            a.updated_at,
            c.name AS category_name,
            CASE
                WHEN a.asset_type IN ('virtual', 'data', 'intangible') AND a.expiry_date IS NOT NULL THEN
                    CASE
                        WHEN date(a.expiry_date) < date('now') THEN 'expired'
                        WHEN date(a.expiry_date) <= date('now', '+7 days') THEN 'urgent'
                        WHEN date(a.expiry_date) <= date('now', '+30 days') THEN 'expiring_soon'
                        ELSE NULL
                    END
                ELSE NULL
            END AS expiry_status,
            CASE
                WHEN a.asset_type IN ('virtual', 'data', 'intangible') AND a.expiry_date IS NOT NULL
                THEN CAST(julianday(date(a.expiry_date)) - julianday(date('now')) AS INTEGER)
                ELSE NULL
            END AS days_until_expiry,
            MAX(COALESCE(a.current_value, 0.0) - COALESCE(a.accumulated_depreciation, 0.0), 0.0) AS net_value
        FROM assets a
        LEFT JOIN asset_categories c ON c.id = a.category_id AND c.company_id = a.company_id
        WHERE a.company_id = ?
        ORDER BY a.created_at DESC
        LIMIT ? OFFSET ?
        "#
    )
    .bind(&user.company_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    let assets = rows.into_iter().map(Into::into).collect();

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM assets WHERE company_id = ?")
        .bind(&user.company_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(Json(AssetsListResponse {
        data: assets,
        count,
    }))
}

async fn asset_overview(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(query): Query<AssetOverviewQuery>,
) -> Result<Json<AssetOverview>, ApiError> {
    let row: AssetOverviewRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) AS total,
            COALESCE(SUM(CASE WHEN status IN ('active', 'in_use') THEN 1 ELSE 0 END), 0) AS in_use,
            COALESCE(SUM(COALESCE(current_value, 0.0)), 0.0) AS total_value,
            COALESCE(SUM(COALESCE(accumulated_depreciation, 0.0)), 0.0) AS total_depreciation,
            COALESCE(SUM(CASE WHEN asset_type = 'physical' THEN 1 ELSE 0 END), 0) AS physical,
            COALESCE(SUM(CASE WHEN asset_type = 'virtual' THEN 1 ELSE 0 END), 0) AS virtual_count,
            COALESCE(SUM(CASE WHEN asset_type = 'data' THEN 1 ELSE 0 END), 0) AS data,
            COALESCE(SUM(CASE WHEN asset_type = 'intangible' THEN 1 ELSE 0 END), 0) AS intangible,
            COALESCE(SUM(CASE
                WHEN asset_type IN ('virtual', 'data', 'intangible')
                 AND expiry_date IS NOT NULL
                 AND date(expiry_date) >= date('now')
                 AND date(expiry_date) <= date('now', '+30 days')
                THEN 1 ELSE 0 END), 0) AS expiring_soon,
            COALESCE(SUM(CASE
                WHEN asset_type IN ('virtual', 'data', 'intangible')
                 AND expiry_date IS NOT NULL
                 AND date(expiry_date) < date('now')
                THEN 1 ELSE 0 END), 0) AS expired
        FROM assets
        WHERE company_id = ?
        "#,
    )
    .bind(&user.company_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let rows: Vec<AssetListRow> = sqlx::query_as(
        r#"
        SELECT
            a.id,
            a.company_id,
            a.name,
            a.asset_type,
            a.status,
            a.description,
            a.serial_number,
            a.location,
            a.purchase_date,
            a.purchase_price,
            a.current_value,
            a.expiry_date,
            a.category_id,
            a.image_url,
            a.useful_life,
            a.salvage_rate,
            a.depreciation_method,
            a.monthly_depreciation,
            a.accumulated_depreciation,
            a.invoice_id,
            a.contract_id,
            a.registration_number,
            a.holder_name,
            a.created_by,
            a.created_at,
            a.updated_at,
            c.name AS category_name,
            CASE
                WHEN a.asset_type IN ('virtual', 'data', 'intangible') AND a.expiry_date IS NOT NULL THEN
                    CASE
                        WHEN date(a.expiry_date) < date('now') THEN 'expired'
                        WHEN date(a.expiry_date) <= date('now', '+7 days') THEN 'urgent'
                        WHEN date(a.expiry_date) <= date('now', '+30 days') THEN 'expiring_soon'
                        ELSE NULL
                    END
                ELSE NULL
            END AS expiry_status,
            CASE
                WHEN a.asset_type IN ('virtual', 'data', 'intangible') AND a.expiry_date IS NOT NULL
                THEN CAST(julianday(date(a.expiry_date)) - julianday(date('now')) AS INTEGER)
                ELSE NULL
            END AS days_until_expiry,
            MAX(COALESCE(a.current_value, 0.0) - COALESCE(a.accumulated_depreciation, 0.0), 0.0) AS net_value
        FROM assets a
        LEFT JOIN asset_categories c ON c.id = a.category_id AND c.company_id = a.company_id
        WHERE a.company_id = ?
        ORDER BY a.created_at DESC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let categories: Vec<AssetCategoryRow> = sqlx::query_as(
        r#"
        SELECT id, name, category_type, description
        FROM asset_categories
        WHERE company_id = ?
        ORDER BY name ASC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let can_manage_categories = current_user_can_manage_assets(&pool, &user).await?;
    let meta_by_asset = data_asset_meta_by_asset(&pool, &user.company_id).await?;
    let search = query.search.unwrap_or_default().trim().to_lowercase();
    let status_filter = query.status.unwrap_or_default();
    let category_filter = query.category_id.unwrap_or_default();
    let asset_type_filter = query.asset_type.unwrap_or_default();
    let has_filters = !search.is_empty()
        || (!status_filter.is_empty() && status_filter != "all")
        || (!category_filter.is_empty() && category_filter != "all")
        || (!asset_type_filter.is_empty() && asset_type_filter != "all");

    let assets = rows
        .into_iter()
        .filter_map(|asset| {
            let status = asset.status.clone().unwrap_or_else(|| "active".to_string());
            let asset_type = asset
                .asset_type
                .clone()
                .unwrap_or_else(|| "physical".to_string());
            if !status_filter.is_empty() && status_filter != "all" && status != status_filter {
                return None;
            }
            if !category_filter.is_empty()
                && category_filter != "all"
                && asset.category_id.as_deref() != Some(category_filter.as_str())
            {
                return None;
            }
            if !asset_type_filter.is_empty()
                && asset_type_filter != "all"
                && asset_type != asset_type_filter
            {
                return None;
            }
            if !search.is_empty() {
                let search_fields = [
                    Some(asset.name.as_str()),
                    asset.serial_number.as_deref(),
                    asset.location.as_deref(),
                    asset.category_name.as_deref(),
                ];
                let matches_search = search_fields
                    .into_iter()
                    .flatten()
                    .any(|value| value.to_lowercase().contains(&search));
                if !matches_search {
                    return None;
                }
            }

            let current_value_label = value_label(asset.current_value);
            let current_value = asset.current_value.unwrap_or(0.0);
            let monthly_depreciation = asset.monthly_depreciation.unwrap_or(0.0);
            let net_value = asset.net_value;
            let can_delete = can_manage_categories
                || asset.created_by.as_deref() == Some(user.user_id.as_str())
                || asset.created_by.is_none();
            let can_mark_status = can_delete;
            let can_mark_maintenance =
                can_mark_status && !matches!(status.as_str(), "maintenance" | "scrapped");
            let can_mark_scrapped = can_mark_status && status != "scrapped";
            let can_mark_in_use =
                can_mark_status && !matches!(status.as_str(), "active" | "in_use");
            let status_badge = asset_status_badge(&status);
            let asset_type_badge = asset_type_badge(&asset_type);
            let expiry_badge =
                asset_expiry_badge(asset.expiry_status.as_deref(), asset.days_until_expiry);
            let asset_id = asset.id.clone();
            let asset_name = asset.name.clone();
            let mut asset_value = serde_json::to_value(AssetListItem::from(asset)).ok()?;
            let asset_object = asset_value.as_object_mut()?;
            asset_object.insert("asset_type".to_string(), json!(asset_type));
            asset_object.insert("asset_type_badge".to_string(), asset_type_badge);
            asset_object.insert("status".to_string(), json!(status));
            asset_object.insert("status_badge".to_string(), status_badge);
            asset_object.insert(
                "current_value_label".to_string(),
                json!(current_value_label),
            );
            asset_object.insert("expiry_badge".to_string(), expiry_badge);
            asset_object.insert(
                "monthly_depreciation_label".to_string(),
                json!(if monthly_depreciation > 0.0 {
                    format!("月折旧 {}", format_cny(monthly_depreciation))
                } else {
                    "-".to_string()
                }),
            );
            asset_object.insert("net_value_label".to_string(), json!(format_cny(net_value)));
            asset_object.insert(
                "data_asset_meta".to_string(),
                json!(meta_by_asset.get(&asset_id).cloned()),
            );
            asset_object.insert(
                "depreciation_visible".to_string(),
                json!(current_value > 0.0 && monthly_depreciation > 0.0),
            );
            asset_object.insert("can_delete".to_string(), json!(can_delete));
            asset_object.insert(
                "can_mark_maintenance".to_string(),
                json!(can_mark_maintenance),
            );
            asset_object.insert("can_mark_scrapped".to_string(), json!(can_mark_scrapped));
            asset_object.insert("can_mark_in_use".to_string(), json!(can_mark_in_use));
            asset_object.insert(
                "mark_maintenance_action".to_string(),
                asset_status_action(
                    &asset_id,
                    &status,
                    can_mark_status,
                    "maintenance",
                    "mark_maintenance",
                    "标记维修中",
                    "资产已标记为维修中",
                ),
            );
            asset_object.insert(
                "mark_scrapped_action".to_string(),
                asset_status_action(
                    &asset_id,
                    &status,
                    can_mark_status,
                    "scrapped",
                    "mark_scrapped",
                    "标记已报废",
                    "资产已标记为已报废",
                ),
            );
            asset_object.insert(
                "mark_in_use_action".to_string(),
                asset_status_action(
                    &asset_id,
                    &status,
                    can_mark_status,
                    "in_use",
                    "mark_in_use",
                    "标记在用",
                    "资产已标记为在用",
                ),
            );
            asset_object.insert(
                "delete_action".to_string(),
                asset_delete_action(&asset_id, Some(&asset_name), can_delete),
            );
            asset_object.insert(
                "update_action".to_string(),
                asset_update_action(&asset_id, can_delete),
            );

            Some(asset_value)
        })
        .collect();

    let category_values = categories
        .iter()
        .map(|category| asset_category_value(category, can_manage_categories))
        .collect::<Vec<_>>();
    let mut category_options = vec![json!({
        "value": "all",
        "label": "全部分类",
        "category_type": "all",
        "category_type_label": "全部分类",
    })];
    category_options.extend(categories
        .iter()
        .filter(|category| {
            let category_type = category.category_type.as_deref().unwrap_or("general");
            asset_type_filter.is_empty()
                || asset_type_filter == "all"
                || category_type_matches(category_type, &asset_type_filter)
                || category.id == category_filter
        })
        .map(|category| {
            json!({
                "value": category.id,
                "label": category.name,
                "category_type": category.category_type,
                "category_type_label": category_type_label(category.category_type.as_deref().unwrap_or("general")),
            })
        })
        .collect::<Vec<_>>());

    let empty_message = if has_filters {
        "没有找到匹配的资产"
    } else {
        "暂无资产数据，点击上方按钮添加"
    }
    .to_string();

    Ok(Json(AssetOverview {
        total: row.total,
        in_use: row.in_use,
        total_value: row.total_value,
        total_depreciation: row.total_depreciation,
        total_value_label: format_cny(row.total_value),
        total_depreciation_label: format_cny(row.total_depreciation),
        expiring_soon: row.expiring_soon,
        expired: row.expired,
        type_counts: json!({
            "physical": row.physical,
            "virtual": row.virtual_count,
            "data": row.data,
            "intangible": row.intangible,
        }),
        assets,
        categories: category_values,
        status_options: asset_status_options(),
        asset_type_options: asset_type_options(),
        category_options,
        permissions: json!({
            "can_manage_categories": can_manage_categories,
        }),
        empty_message,
        create_action: asset_create_action(true),
        category_create_action: asset_category_create_action(can_manage_categories),
    }))
}

async fn data_asset_meta_by_asset(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<HashMap<String, Value>, ApiError> {
    let rows: Vec<DataAssetMetaRow> = sqlx::query_as(
        r#"
        SELECT
            id,
            asset_id,
            data_type,
            tech_stack,
            version,
            license_type,
            patent_number,
            patent_type,
            trademark_class,
            data_volume,
            data_format,
            access_url
        FROM data_asset_meta
        WHERE company_id = ?
        "#,
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.asset_id.clone(),
                json!({
                    "id": row.id,
                    "asset_id": row.asset_id,
                    "data_type": row.data_type,
                    "tech_stack": row.tech_stack,
                    "version": row.version,
                    "license_type": row.license_type,
                    "patent_number": row.patent_number,
                    "patent_type": row.patent_type,
                    "trademark_class": row.trademark_class,
                    "data_volume": row.data_volume,
                    "data_format": row.data_format,
                    "access_url": row.access_url,
                }),
            )
        })
        .collect())
}

async fn asset_form_options(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Query(query): Query<AssetFormOptionsQuery>,
) -> Result<Json<Value>, ApiError> {
    let selected_asset_type = query
        .asset_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "all")
        .unwrap_or("physical");

    let categories: Vec<AssetCategoryRow> = sqlx::query_as(
        r#"
        SELECT id, name, category_type, description
        FROM asset_categories
        WHERE company_id = ?
        ORDER BY name ASC
        "#,
    )
    .bind(&user.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let category_options = categories
        .iter()
        .filter(|category| {
            let category_type = category.category_type.as_deref().unwrap_or("general");
            category_type_matches(category_type, selected_asset_type)
        })
        .map(|category| {
            let category_type = category.category_type.as_deref().unwrap_or("general");
            json!({
                "value": category.id,
                "label": category.name,
                "category_type": category_type,
                "category_type_label": category_type_label(category_type),
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(json!({
        "status_options": asset_form_status_options(),
        "asset_type_options": asset_form_type_options(),
        "category_options": category_options,
        "data_type_options": data_type_options(),
        "intangible_type_options": intangible_type_options(),
        "depreciation_method_options": depreciation_method_options(),
        "license_type_options": license_type_options(),
        "patent_type_options": patent_type_options(),
        "default_values": {
            "asset_type": "physical",
            "status": "in_use",
            "salvage_rate": "5",
            "depreciation_method": "straight_line",
        },
    })))
}

async fn current_user_can_manage_assets(
    pool: &SqlitePool,
    user: &AuthUser,
) -> Result<bool, ApiError> {
    let role: Option<(String,)> = sqlx::query_as(
        "SELECT role FROM company_members WHERE user_id = ? AND company_id = ? LIMIT 1",
    )
    .bind(&user.user_id)
    .bind(&user.company_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(matches!(
        role.as_ref().map(|row| row.0.as_str()),
        Some("owner" | "admin")
    ))
}

fn format_cny(amount: f64) -> String {
    let rounded = (amount * 100.0).round() / 100.0;
    if (rounded.fract()).abs() < 0.005 {
        format!("¥{}", rounded as i64)
    } else {
        let mut value = format!("{rounded:.2}");
        while value.contains('.') && value.ends_with('0') {
            value.pop();
        }
        if value.ends_with('.') {
            value.pop();
        }
        format!("¥{value}")
    }
}

fn value_label(value: Option<f64>) -> String {
    value.map(format_cny).unwrap_or_else(|| "-".to_string())
}

fn asset_status_badge(status: &str) -> Value {
    match status {
        "active" | "in_use" => json!({
            "value": status,
            "label": "在用",
            "variant": "default",
        }),
        "idle" => json!({
            "value": "idle",
            "label": "闲置",
            "variant": "secondary",
        }),
        "maintenance" => json!({
            "value": "maintenance",
            "label": "维修中",
            "variant": "outline",
        }),
        "scrapped" => json!({
            "value": "scrapped",
            "label": "已报废",
            "variant": "destructive",
        }),
        value => json!({
            "value": value,
            "label": value,
            "variant": "outline",
        }),
    }
}

fn asset_status_options() -> Vec<Value> {
    let mut options = vec![json!({
        "value": "all",
        "label": "全部状态",
        "variant": "outline",
    })];
    options.extend(
        ["active", "in_use", "idle", "maintenance", "scrapped"]
            .into_iter()
            .map(asset_status_badge)
            .collect::<Vec<_>>(),
    );
    options
}

fn asset_form_status_options() -> Vec<Value> {
    [
        ("in_use", "在用"),
        ("idle", "闲置"),
        ("maintenance", "维修中"),
        ("scrapped", "已报废"),
    ]
    .into_iter()
    .map(|(value, label)| {
        json!({
            "value": value,
            "label": label,
        })
    })
    .collect()
}

fn asset_type_badge(asset_type: &str) -> Value {
    match asset_type {
        "physical" => json!({
            "value": "physical",
            "label": "固定资产",
            "variant": "default",
        }),
        "virtual" => json!({
            "value": "virtual",
            "label": "虚拟资产",
            "variant": "secondary",
        }),
        "data" => json!({
            "value": "data",
            "label": "数据资产",
            "variant": "outline",
        }),
        "intangible" => json!({
            "value": "intangible",
            "label": "无形资产",
            "variant": "outline",
        }),
        value => json!({
            "value": value,
            "label": value,
            "variant": "outline",
        }),
    }
}

fn asset_type_options() -> Vec<Value> {
    let mut options = vec![json!({
        "value": "all",
        "label": "全部类型",
        "variant": "outline",
    })];
    options.extend(
        ["physical", "virtual", "data", "intangible"]
            .into_iter()
            .map(asset_type_badge)
            .collect::<Vec<_>>(),
    );
    options
}

fn asset_form_type_options() -> Vec<Value> {
    [
        ("physical", "固定资产", "办公设备、机械等"),
        ("virtual", "虚拟资产", "域名、账号、订阅"),
        ("data", "数据资产", "软件、数据集、算法"),
        ("intangible", "无形资产", "专利、商标、版权"),
    ]
    .into_iter()
    .map(|(value, label, desc)| {
        json!({
            "value": value,
            "label": label,
            "desc": desc,
        })
    })
    .collect()
}

fn data_type_options() -> Vec<Value> {
    [
        ("software", "软件/系统"),
        ("dataset", "数据集"),
        ("copyright", "著作权"),
        ("trade_secret", "商业秘密"),
    ]
    .into_iter()
    .map(|(value, label)| {
        json!({
            "value": value,
            "label": label,
        })
    })
    .collect()
}

fn intangible_type_options() -> Vec<Value> {
    [
        ("patent", "专利"),
        ("trademark", "商标"),
        ("domain", "域名"),
    ]
    .into_iter()
    .map(|(value, label)| {
        json!({
            "value": value,
            "label": label,
        })
    })
    .collect()
}

fn depreciation_method_options() -> Vec<Value> {
    [
        ("straight_line", "直线法（年限平均法）"),
        ("declining_balance", "双倍余额递减法"),
        ("sum_of_years", "年数总和法"),
    ]
    .into_iter()
    .map(|(value, label)| {
        json!({
            "value": value,
            "label": label,
        })
    })
    .collect()
}

fn license_type_options() -> Vec<Value> {
    [
        ("proprietary", "自研"),
        ("commercial", "商业购买"),
        ("opensource", "开源"),
        ("saas", "SaaS 订阅"),
    ]
    .into_iter()
    .map(|(value, label)| {
        json!({
            "value": value,
            "label": label,
        })
    })
    .collect()
}

fn patent_type_options() -> Vec<Value> {
    [
        ("invention", "发明专利"),
        ("utility", "实用新型"),
        ("design", "外观设计"),
    ]
    .into_iter()
    .map(|(value, label)| {
        json!({
            "value": value,
            "label": label,
        })
    })
    .collect()
}

fn asset_expiry_badge(expiry_status: Option<&str>, days_until_expiry: Option<i64>) -> Value {
    let days = days_until_expiry.unwrap_or(0);
    match expiry_status {
        Some("expired") => json!({
            "value": "expired",
            "label": "已过期",
            "variant": "destructive",
            "visible": true,
        }),
        Some("urgent") => json!({
            "value": "urgent",
            "label": format!("紧急（{}天）", days),
            "variant": "destructive",
            "visible": true,
        }),
        Some("expiring_soon") => json!({
            "value": "expiring_soon",
            "label": format!("即将到期（{}天）", days),
            "variant": "outline",
            "class_name": "border-yellow-500 text-yellow-600",
            "visible": true,
        }),
        _ => json!({
            "value": "none",
            "label": "-",
            "variant": "outline",
            "visible": false,
        }),
    }
}

fn category_type_matches(category_type: &str, asset_type: &str) -> bool {
    matches!(category_type, "both" | "general") || category_type == asset_type
}

fn category_type_label(category_type: &str) -> &'static str {
    match category_type {
        "physical" => "固定资产",
        "virtual" => "虚拟资产",
        "data" => "数据资产",
        "intangible" => "无形资产",
        _ => "通用",
    }
}

fn asset_category_value(category: &AssetCategoryRow, can_delete: bool) -> Value {
    let category_type = category.category_type.as_deref().unwrap_or("general");
    json!({
        "id": category.id,
        "value": category.id,
        "name": category.name,
        "label": category.name,
        "description": category.description,
        "category_type": category_type,
        "category_type_label": category_type_label(category_type),
        "update_action": asset_category_update_action(&category.id, can_delete),
        "delete_action": asset_category_delete_action(&category.id, &category.name, can_delete),
    })
}

fn backend_write_action(
    method: &str,
    endpoint: String,
    label: &str,
    success_title: &str,
    success_message: &str,
) -> Value {
    json!({
        "method": method,
        "endpoint": endpoint,
        "label": label,
        "success_title": success_title,
        "success_message": success_message,
    })
}

fn asset_create_action(can_create: bool) -> Value {
    if !can_create {
        return json!(null);
    }

    backend_write_action(
        "POST",
        "/api/assets".to_string(),
        "添加资产",
        "创建成功",
        "资产添加成功",
    )
}

fn asset_update_action(id: &str, can_update: bool) -> Value {
    if !can_update {
        return json!(null);
    }

    backend_write_action(
        "PUT",
        format!("/api/assets/{id}"),
        "保存资产",
        "更新成功",
        "资产更新成功",
    )
}

fn asset_category_create_action(can_create: bool) -> Value {
    if !can_create {
        return json!(null);
    }

    backend_write_action(
        "POST",
        "/api/categories".to_string(),
        "添加分类",
        "创建成功",
        "分类添加成功",
    )
}

fn asset_category_update_action(id: &str, can_update: bool) -> Value {
    if !can_update {
        return json!(null);
    }

    backend_write_action(
        "PUT",
        format!("/api/categories/{id}"),
        "保存分类",
        "更新成功",
        "分类更新成功",
    )
}

fn asset_delete_action(id: &str, name: Option<&str>, can_delete: bool) -> Value {
    if !can_delete {
        return json!(null);
    }
    let name = name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("该资产");

    json!({
        "method": "DELETE",
        "endpoint": format!("/api/assets/{id}"),
        "confirm_message": format!("确定要删除资产「{name}」吗？此操作无法撤销。"),
        "success_title": "删除成功",
        "success_message": "资产删除成功",
    })
}

fn asset_category_delete_action(id: &str, name: &str, can_delete: bool) -> Value {
    if !can_delete {
        return json!(null);
    }
    let name = name.trim();
    let name = if name.is_empty() { "该分类" } else { name };

    json!({
        "method": "DELETE",
        "endpoint": format!("/api/categories/{id}"),
        "confirm_message": format!("确定要删除分类「{name}」吗？已关联的资产将取消分类关联。"),
        "success_title": "删除成功",
        "success_message": "分类删除成功",
    })
}

fn asset_can_transition_to(status: &str, target_status: &str) -> bool {
    match target_status {
        "maintenance" => !matches!(status, "maintenance" | "scrapped"),
        "scrapped" => status != "scrapped",
        "in_use" => !matches!(status, "active" | "in_use"),
        _ => false,
    }
}

fn asset_status_action(
    id: &str,
    status: &str,
    can_modify: bool,
    target_status: &str,
    action_path: &str,
    label: &str,
    success_message: &str,
) -> Value {
    if !can_modify || !asset_can_transition_to(status, target_status) {
        return json!(null);
    }

    json!({
        "method": "POST",
        "endpoint": format!("/api/assets/{id}/{action_path}"),
        "label": label,
        "success_title": "状态已更新",
        "success_message": success_message,
        "target_status": target_status,
    })
}

fn asset_display_value(
    asset: AssetListRow,
    data_asset_meta: Option<Value>,
    can_modify: bool,
) -> Option<Value> {
    let asset_id = asset.id.clone();
    let asset_name = asset.name.clone();
    let status = asset.status.clone().unwrap_or_else(|| "active".to_string());
    let asset_type = asset
        .asset_type
        .clone()
        .unwrap_or_else(|| "physical".to_string());
    let category_name = asset.category_name.clone();
    let current_value_label = value_label(asset.current_value);
    let current_value = asset.current_value.unwrap_or(0.0);
    let monthly_depreciation = asset.monthly_depreciation.unwrap_or(0.0);
    let net_value = asset.net_value;

    let mut asset_value = serde_json::to_value(AssetListItem::from(asset)).ok()?;
    let asset_object = asset_value.as_object_mut()?;
    asset_object.insert("asset_type".to_string(), json!(asset_type.clone()));
    asset_object.insert(
        "asset_type_badge".to_string(),
        asset_type_badge(&asset_type),
    );
    asset_object.insert("status".to_string(), json!(status.clone()));
    asset_object.insert("status_badge".to_string(), asset_status_badge(&status));
    asset_object.insert("category_name".to_string(), json!(category_name));
    asset_object.insert(
        "current_value_label".to_string(),
        json!(current_value_label),
    );
    asset_object.insert(
        "monthly_depreciation_label".to_string(),
        json!(if monthly_depreciation > 0.0 {
            format!("月折旧 {}", format_cny(monthly_depreciation))
        } else {
            "-".to_string()
        }),
    );
    asset_object.insert("net_value_label".to_string(), json!(format_cny(net_value)));
    asset_object.insert("data_asset_meta".to_string(), json!(data_asset_meta));
    asset_object.insert(
        "depreciation_visible".to_string(),
        json!(current_value > 0.0 && monthly_depreciation > 0.0),
    );
    asset_object.insert("can_delete".to_string(), json!(can_modify));
    asset_object.insert(
        "can_mark_maintenance".to_string(),
        json!(can_modify && !matches!(status.as_str(), "maintenance" | "scrapped")),
    );
    asset_object.insert(
        "can_mark_scrapped".to_string(),
        json!(can_modify && status != "scrapped"),
    );
    asset_object.insert(
        "can_mark_in_use".to_string(),
        json!(can_modify && !matches!(status.as_str(), "active" | "in_use")),
    );
    asset_object.insert("id".to_string(), json!(asset_id));
    asset_object.insert(
        "mark_maintenance_action".to_string(),
        asset_status_action(
            &asset_id,
            &status,
            can_modify,
            "maintenance",
            "mark_maintenance",
            "标记维修中",
            "资产已标记为维修中",
        ),
    );
    asset_object.insert(
        "mark_scrapped_action".to_string(),
        asset_status_action(
            &asset_id,
            &status,
            can_modify,
            "scrapped",
            "mark_scrapped",
            "标记已报废",
            "资产已标记为已报废",
        ),
    );
    asset_object.insert(
        "mark_in_use_action".to_string(),
        asset_status_action(
            &asset_id,
            &status,
            can_modify,
            "in_use",
            "mark_in_use",
            "标记在用",
            "资产已标记为在用",
        ),
    );
    asset_object.insert(
        "delete_action".to_string(),
        asset_delete_action(&asset_id, Some(&asset_name), can_modify),
    );
    asset_object.insert(
        "update_action".to_string(),
        asset_update_action(&asset_id, can_modify),
    );

    Some(asset_value)
}

async fn get_asset(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let asset: Option<AssetListRow> = sqlx::query_as(
        r#"
        SELECT
            a.id,
            a.company_id,
            a.name,
            a.asset_type,
            a.status,
            a.description,
            a.serial_number,
            a.location,
            a.purchase_date,
            a.purchase_price,
            a.current_value,
            a.expiry_date,
            a.category_id,
            a.image_url,
            a.useful_life,
            a.salvage_rate,
            a.depreciation_method,
            a.monthly_depreciation,
            a.accumulated_depreciation,
            a.invoice_id,
            a.contract_id,
            a.registration_number,
            a.holder_name,
            a.created_by,
            a.created_at,
            a.updated_at,
            c.name AS category_name,
            CASE
                WHEN a.asset_type IN ('virtual', 'data', 'intangible') AND a.expiry_date IS NOT NULL THEN
                    CASE
                        WHEN date(a.expiry_date) < date('now') THEN 'expired'
                        WHEN date(a.expiry_date) <= date('now', '+7 days') THEN 'urgent'
                        WHEN date(a.expiry_date) <= date('now', '+30 days') THEN 'expiring_soon'
                        ELSE NULL
                    END
                ELSE NULL
            END AS expiry_status,
            CASE
                WHEN a.asset_type IN ('virtual', 'data', 'intangible') AND a.expiry_date IS NOT NULL
                THEN CAST(julianday(date(a.expiry_date)) - julianday(date('now')) AS INTEGER)
                ELSE NULL
            END AS days_until_expiry,
            MAX(COALESCE(a.current_value, 0.0) - COALESCE(a.accumulated_depreciation, 0.0), 0.0) AS net_value
        FROM assets a
        LEFT JOIN asset_categories c ON c.id = a.category_id AND c.company_id = a.company_id
        WHERE a.id = ? AND a.company_id = ?
        "#,
    )
    .bind(&id)
    .bind(&user.company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let Some(asset) = asset else {
        return Err(api_err(StatusCode::NOT_FOUND, "asset not found"));
    };
    let can_manage = current_user_can_manage_assets(&pool, &user).await?;
    let can_modify = can_manage
        || asset.created_by.as_deref() == Some(user.user_id.as_str())
        || asset.created_by.is_none();
    let meta_by_asset = data_asset_meta_by_asset(&pool, &user.company_id).await?;
    let asset_id = asset.id.clone();
    let data_asset_meta = meta_by_asset.get(&asset_id).cloned();
    let detail = asset_display_value(asset, data_asset_meta, can_modify)
        .ok_or_else(|| api_err(StatusCode::INTERNAL_SERVER_ERROR, "serialization failed"))?;

    Ok(Json(json!({ "data": detail })))
}

async fn create_asset(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Json(req): Json<CreateAssetRequest>,
) -> Result<Json<AssetResponse>, ApiError> {
    let purchase_price = parse_optional_asset_money("purchase_price", req.purchase_price.as_ref())?;
    let current_value = parse_optional_asset_money("current_value", req.current_value.as_ref())?;
    let useful_life = parse_optional_asset_i64("useful_life", req.useful_life.as_ref())?;
    let salvage_rate = parse_optional_asset_money("salvage_rate", req.salvage_rate.as_ref())?
        .map(|rate| if rate > 1.0 { rate / 100.0 } else { rate });
    let purchase_date =
        normalize_optional_asset_date("purchase_date", req.purchase_date.as_deref())?;
    let expiry_date = normalize_optional_asset_date("expiry_date", req.expiry_date.as_deref())?;

    // R2-3: Validate non-negative prices
    if let Some(price) = purchase_price
        && price < 0.0
    {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "purchase_price must be non-negative",
        ));
    }
    if let Some(val) = current_value
        && val < 0.0
    {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "current_value must be non-negative",
        ));
    }
    if let Some(life) = useful_life
        && life < 0
    {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "useful_life must be non-negative",
        ));
    }
    if let Some(rate) = salvage_rate
        && !(0.0..=1.0).contains(&rate)
    {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "salvage_rate must be between 0 and 1",
        ));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let asset_type = req.asset_type.as_deref().unwrap_or("physical");
    let depreciation_method = req
        .depreciation_method
        .clone()
        .unwrap_or_else(|| "straight_line".to_string());
    let monthly_depreciation = if asset_type == "physical" {
        calculate_monthly_depreciation(
            purchase_price.unwrap_or(0.0),
            salvage_rate.unwrap_or(0.05),
            useful_life.unwrap_or(0),
            &depreciation_method,
        )
    } else {
        None
    };

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    sqlx::query(
        r#"
        INSERT INTO assets (
            id, company_id, name, asset_type, status, description, serial_number,
            location, purchase_date, purchase_price, current_value, expiry_date,
            category_id, useful_life, salvage_rate, depreciation_method,
            monthly_depreciation, invoice_id, contract_id, registration_number,
            holder_name, created_by
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&user.company_id)
    .bind(&req.name)
    .bind(&req.asset_type)
    .bind("in_use")
    .bind(&req.description)
    .bind(&req.serial_number)
    .bind(&req.location)
    .bind(&purchase_date)
    .bind(purchase_price)
    .bind(current_value)
    .bind(&expiry_date)
    .bind(&req.category_id)
    .bind(useful_life)
    .bind(salvage_rate.unwrap_or(0.05))
    .bind(&depreciation_method)
    .bind(monthly_depreciation)
    .bind(&req.invoice_id)
    .bind(&req.contract_id)
    .bind(&req.registration_number)
    .bind(&req.holder_name)
    .bind(&user.user_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    // Audit log
    let log_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO operation_logs (id, company_id, user_id, action, entity_type, entity_id) VALUES (?, ?, ?, 'create', 'assets', ?)")
        .bind(&log_id).bind(&user.company_id).bind(&user.user_id).bind(&id)
        .execute(&mut *tx).await.ok();

    tx.commit()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    if let Some(meta_input) = data_asset_meta_input_from_request(&req) {
        upsert_data_asset_meta(&pool, &user.company_id, &id, meta_input).await?;
    }

    // R2-1: Re-fetch with company_id scoping
    let asset: Asset = sqlx::query_as("SELECT * FROM assets WHERE id = ? AND company_id = ?")
        .bind(&id)
        .bind(&user.company_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(Json(AssetResponse { data: asset }))
}

fn parse_optional_asset_money(field: &str, value: Option<&Value>) -> Result<Option<f64>, ApiError> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    crate::api::generic::parse_form_money(value)
        .map(Some)
        .ok_or_else(|| {
            api_err(
                StatusCode::BAD_REQUEST,
                &format!("{} must be a valid number", field),
            )
        })
}

fn parse_optional_asset_i64(field: &str, value: Option<&Value>) -> Result<Option<i64>, ApiError> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    crate::api::generic::parse_form_i64(value)
        .map(Some)
        .ok_or_else(|| {
            api_err(
                StatusCode::BAD_REQUEST,
                &format!("{} must be a valid integer", field),
            )
        })
}

fn normalize_optional_asset_date(
    field: &str,
    value: Option<&str>,
) -> Result<Option<String>, ApiError> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.trim().is_empty() {
        return Ok(None);
    }
    crate::api::generic::normalize_form_date(value)
        .map(Some)
        .ok_or_else(|| {
            api_err(
                StatusCode::BAD_REQUEST,
                &format!("{} must be a valid date", field),
            )
        })
}

struct DataAssetMetaInput {
    data_type: String,
    tech_stack: Option<String>,
    version: Option<String>,
    license_type: Option<String>,
    patent_number: Option<String>,
    patent_type: Option<String>,
    trademark_class: Option<String>,
    data_volume: Option<String>,
    data_format: Option<String>,
    access_url: Option<String>,
}

fn clean_optional_string(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn data_asset_meta_input(
    data_type: Option<String>,
    tech_stack: Option<String>,
    version: Option<String>,
    license_type: Option<String>,
    patent_number: Option<String>,
    patent_type: Option<String>,
    trademark_class: Option<String>,
    data_volume: Option<String>,
    data_format: Option<String>,
    access_url: Option<String>,
) -> Option<DataAssetMetaInput> {
    Some(DataAssetMetaInput {
        data_type: clean_optional_string(data_type)?,
        tech_stack: clean_optional_string(tech_stack),
        version: clean_optional_string(version),
        license_type: clean_optional_string(license_type),
        patent_number: clean_optional_string(patent_number),
        patent_type: clean_optional_string(patent_type),
        trademark_class: clean_optional_string(trademark_class),
        data_volume: clean_optional_string(data_volume),
        data_format: clean_optional_string(data_format),
        access_url: clean_optional_string(access_url),
    })
}

fn data_asset_meta_input_from_request(req: &CreateAssetRequest) -> Option<DataAssetMetaInput> {
    data_asset_meta_input(
        req.data_type.clone(),
        req.tech_stack.clone(),
        req.version.clone(),
        req.license_type.clone(),
        req.patent_number.clone(),
        req.patent_type.clone(),
        req.trademark_class.clone(),
        req.data_volume.clone(),
        req.data_format.clone(),
        req.access_url.clone(),
    )
}

fn take_optional_string(map: &mut serde_json::Map<String, Value>, key: &str) -> Option<String> {
    match map.remove(key) {
        Some(Value::String(value)) => Some(value),
        Some(Value::Null) | None => None,
        Some(value) => value.as_str().map(str::to_string),
    }
}

fn data_asset_meta_input_from_body(body: &mut Value) -> Option<DataAssetMetaInput> {
    let map = body.as_object_mut()?;
    let data_type = take_optional_string(map, "data_type");
    let tech_stack = take_optional_string(map, "tech_stack");
    let version = take_optional_string(map, "version");
    let license_type = take_optional_string(map, "license_type");
    let patent_number = take_optional_string(map, "patent_number");
    let patent_type = take_optional_string(map, "patent_type");
    let trademark_class = take_optional_string(map, "trademark_class");
    let data_volume = take_optional_string(map, "data_volume");
    let data_format = take_optional_string(map, "data_format");
    let access_url = take_optional_string(map, "access_url");
    map.remove("data_asset_meta");

    data_asset_meta_input(
        data_type,
        tech_stack,
        version,
        license_type,
        patent_number,
        patent_type,
        trademark_class,
        data_volume,
        data_format,
        access_url,
    )
}

async fn upsert_data_asset_meta(
    pool: &SqlitePool,
    company_id: &str,
    asset_id: &str,
    input: DataAssetMetaInput,
) -> Result<(), ApiError> {
    let existing_id: Option<String> =
        sqlx::query_scalar("SELECT id FROM data_asset_meta WHERE asset_id = ? AND company_id = ?")
            .bind(asset_id)
            .bind(company_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    if let Some(id) = existing_id {
        sqlx::query(
            r#"
            UPDATE data_asset_meta
            SET data_type = ?, tech_stack = ?, version = ?, license_type = ?,
                patent_number = ?, patent_type = ?, trademark_class = ?,
                data_volume = ?, data_format = ?, access_url = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ? AND company_id = ?
            "#,
        )
        .bind(input.data_type)
        .bind(input.tech_stack)
        .bind(input.version)
        .bind(input.license_type)
        .bind(input.patent_number)
        .bind(input.patent_type)
        .bind(input.trademark_class)
        .bind(input.data_volume)
        .bind(input.data_format)
        .bind(input.access_url)
        .bind(id)
        .bind(company_id)
        .execute(pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    } else {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO data_asset_meta (
                id, company_id, asset_id, data_type, tech_stack, version,
                license_type, patent_number, patent_type, trademark_class,
                data_volume, data_format, access_url
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(company_id)
        .bind(asset_id)
        .bind(input.data_type)
        .bind(input.tech_stack)
        .bind(input.version)
        .bind(input.license_type)
        .bind(input.patent_number)
        .bind(input.patent_type)
        .bind(input.trademark_class)
        .bind(input.data_volume)
        .bind(input.data_format)
        .bind(input.access_url)
        .execute(pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    }

    Ok(())
}

fn calculate_monthly_depreciation(
    purchase_price: f64,
    salvage_rate: f64,
    useful_life_months: i64,
    method: &str,
) -> Option<f64> {
    if purchase_price <= 0.0 || useful_life_months <= 0 {
        return None;
    }

    let salvage_value = purchase_price * salvage_rate;
    let depreciable_amount = (purchase_price - salvage_value).max(0.0);
    let useful_life_months_f = useful_life_months as f64;
    let monthly = match method {
        "declining_balance" => {
            let useful_life_years = useful_life_months_f / 12.0;
            if useful_life_years <= 0.0 {
                0.0
            } else {
                (purchase_price * (2.0 / useful_life_years)) / 12.0
            }
        }
        "sum_of_years" => {
            let years = useful_life_months_f / 12.0;
            let sum_of_years = (years * (years + 1.0)) / 2.0;
            if sum_of_years <= 0.0 {
                0.0
            } else {
                (depreciable_amount * (years / sum_of_years)) / 12.0
            }
        }
        _ => depreciable_amount / useful_life_months_f,
    };

    Some((monthly * 100.0).round() / 100.0)
}

async fn update_asset(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(mut body): Json<Value>,
) -> Result<Json<AssetResponse>, ApiError> {
    if let Some(map) = body.as_object_mut() {
        map.remove("status");
    }
    let meta_input = data_asset_meta_input_from_body(&mut body);

    let has_update_fields = body.as_object().map(|map| !map.is_empty()).unwrap_or(true);
    if has_update_fields {
        // Use generic_update for partial update support (no longer requires all fields)
        let _ = crate::api::generic::generic_update(
            &pool,
            "assets",
            &id,
            body,
            &user.company_id,
            &user.user_id,
        )
        .await?;
    }

    if let Some(meta_input) = meta_input {
        upsert_data_asset_meta(&pool, &user.company_id, &id, meta_input).await?;
    }

    // Audit log
    let log_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO operation_logs (id, company_id, user_id, action, entity_type, entity_id) VALUES (?, ?, ?, 'update', 'assets', ?)")
        .bind(&log_id).bind(&user.company_id).bind(&user.user_id).bind(&id)
        .execute(&pool).await.ok();

    // Re-fetch
    let asset: Asset = sqlx::query_as("SELECT * FROM assets WHERE id = ? AND company_id = ?")
        .bind(&id)
        .bind(&user.company_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| api_err(StatusCode::NOT_FOUND, "asset not found"))?;

    Ok(Json(AssetResponse { data: asset }))
}

async fn mark_asset_maintenance(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    update_asset_status(&pool, &user, &id, "maintenance").await
}

async fn mark_asset_scrapped(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    update_asset_status(&pool, &user, &id, "scrapped").await
}

async fn mark_asset_in_use(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    update_asset_status(&pool, &user, &id, "in_use").await
}

async fn update_asset_status(
    pool: &SqlitePool,
    user: &AuthUser,
    id: &str,
    status: &str,
) -> Result<Json<Value>, ApiError> {
    let asset = fetch_scoped_asset(pool, id, &user.company_id).await?;
    if !asset_can_modify_status(pool, user, asset.created_by.as_deref()).await? {
        return Err(api_err(StatusCode::FORBIDDEN, "forbidden"));
    }

    sqlx::query(
        "UPDATE assets SET status = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? AND company_id = ?",
    )
    .bind(status)
    .bind(id)
    .bind(&user.company_id)
    .execute(pool)
    .await
    .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let asset = fetch_scoped_asset(pool, id, &user.company_id).await?;
    Ok(Json(asset_status_action_value(
        asset.id.as_str(),
        asset.status.as_deref().unwrap_or("in_use"),
        true,
    )))
}

async fn fetch_scoped_asset(
    pool: &SqlitePool,
    id: &str,
    company_id: &str,
) -> Result<Asset, ApiError> {
    sqlx::query_as("SELECT * FROM assets WHERE id = ? AND company_id = ?")
        .bind(id)
        .bind(company_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?
        .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "asset not found"))
}

async fn asset_can_modify_status(
    pool: &SqlitePool,
    user: &AuthUser,
    created_by: Option<&str>,
) -> Result<bool, ApiError> {
    let can_manage = current_user_can_manage_assets(pool, user).await?;
    Ok(can_manage || created_by == Some(user.user_id.as_str()) || created_by.is_none())
}

fn asset_status_action_value(id: &str, status: &str, can_modify: bool) -> Value {
    json!({
        "id": id,
        "status": status,
        "status_badge": asset_status_badge(status),
        "can_mark_maintenance": can_modify && !matches!(status, "maintenance" | "scrapped"),
        "can_mark_scrapped": can_modify && status != "scrapped",
        "can_mark_in_use": can_modify && !matches!(status, "active" | "in_use"),
        "mark_maintenance_action": asset_status_action(
            id,
            status,
            can_modify,
            "maintenance",
            "mark_maintenance",
            "标记维修中",
            "资产已标记为维修中",
        ),
        "mark_scrapped_action": asset_status_action(
            id,
            status,
            can_modify,
            "scrapped",
            "mark_scrapped",
            "标记已报废",
            "资产已标记为已报废",
        ),
        "mark_in_use_action": asset_status_action(
            id,
            status,
            can_modify,
            "in_use",
            "mark_in_use",
            "标记在用",
            "资产已标记为在用",
        ),
    })
}

async fn delete_asset(
    State(pool): State<SqlitePool>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let asset_owner: Option<(Option<String>,)> =
        sqlx::query_as("SELECT created_by FROM assets WHERE id = ? AND company_id = ?")
            .bind(&id)
            .bind(&user.company_id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;
    let Some((created_by,)) = asset_owner else {
        return Err(api_err(StatusCode::NOT_FOUND, "asset not found"));
    };
    let can_manage = current_user_can_manage_assets(&pool, &user).await?;
    let can_delete =
        can_manage || created_by.as_deref() == Some(user.user_id.as_str()) || created_by.is_none();
    if !can_delete {
        return Err(api_err(StatusCode::FORBIDDEN, "forbidden"));
    }

    // R2-8: Check for all dependent records before deleting (aligned with generic.rs)
    for (dep_table, dep_col) in &[
        ("data_asset_meta", "asset_id"),
        ("asset_records", "asset_id"),
    ] {
        if let Ok((n,)) = sqlx::query_as::<_, (i64,)>(&format!(
            "SELECT COUNT(*) FROM {} WHERE {} = ? AND company_id = ?",
            dep_table, dep_col
        ))
        .bind(&id)
        .bind(&user.company_id)
        .fetch_one(&pool)
        .await
            && n > 0
        {
            return Err(api_err(
                StatusCode::CONFLICT,
                "cannot delete: has dependent records",
            ));
        }
    }

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    let result = sqlx::query("DELETE FROM assets WHERE id = ? AND company_id = ?")
        .bind(&id)
        .bind(&user.company_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    if result.rows_affected() == 0 {
        return Err(api_err(StatusCode::NOT_FOUND, "asset not found"));
    }

    // Audit log
    let log_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO operation_logs (id, company_id, user_id, action, entity_type, entity_id) VALUES (?, ?, ?, 'delete', 'assets', ?)")
        .bind(&log_id).bind(&user.company_id).bind(&user.user_id).bind(&id)
        .execute(&mut *tx).await.ok();

    tx.commit()
        .await
        .map_err(|_| api_err(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?;

    Ok(StatusCode::NO_CONTENT)
}
