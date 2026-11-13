use axum::{http::StatusCode, response::Json};
use serde_json::{Value, json};

/// Structured error type: (StatusCode, Json body)
/// Used across all API modules for consistent error responses
pub type ApiError = (StatusCode, Json<Value>);

/// Helper to construct a JSON error response
pub fn api_err(status: StatusCode, msg: &str) -> ApiError {
    (status, Json(json!({ "error": msg })))
}
