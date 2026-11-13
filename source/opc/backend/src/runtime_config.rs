use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

const DEFAULT_BIND_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: &str = "8987";
pub const DEFAULT_AI_PROVIDER: &str = "deepseek";
pub const DEFAULT_AI_MODEL: &str = "deepseek-v4-flash";
pub const DEEPSEEK_CHAT_COMPLETIONS_URL: &str = "https://api.deepseek.com/v1/chat/completions";
pub const DEFAULT_DOCUMENT_PARSER_BACKEND: &str = "ollama-vision";
pub const OLLAMA_VISION_BACKEND: &str = "ollama-vision";
pub const OPENAI_VISION_BACKEND: &str = "openai-vision";

pub fn bind_addr_from_values(host: Option<&str>, port: Option<&str>) -> Result<SocketAddr, String> {
    let host = host
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_BIND_HOST);
    let port = port
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_PORT);

    let ip: IpAddr = host
        .parse()
        .map_err(|_| format!("BIND_HOST must be a valid IP address, got {host}"))?;
    let port: u16 = port
        .parse()
        .map_err(|_| format!("PORT must be a number between 0 and 65535, got {port}"))?;

    Ok(SocketAddr::from((ip, port)))
}

pub fn bind_addr_from_env() -> Result<SocketAddr, String> {
    let host = std::env::var("BIND_HOST").ok();
    let port = std::env::var("PORT").ok();
    bind_addr_from_values(host.as_deref(), port.as_deref())
}

pub fn frontend_dir_from_values(frontend_dir: Option<&str>, manifest_dir: &str) -> PathBuf {
    frontend_dir
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(manifest_dir).join("../frontend/dist"))
}

pub fn frontend_dir_from_env() -> PathBuf {
    let frontend_dir = std::env::var("FRONTEND_DIR").ok();
    frontend_dir_from_values(frontend_dir.as_deref(), env!("CARGO_MANIFEST_DIR"))
}

pub fn ai_model_from_value(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_AI_MODEL)
        .to_string()
}

pub fn ai_model_from_env() -> String {
    let model = std::env::var("AI_MODEL").ok();
    ai_model_from_value(model.as_deref())
}

pub fn document_parser_backend_from_value(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_DOCUMENT_PARSER_BACKEND)
        .to_string()
}

pub fn document_parser_backend_from_env() -> String {
    let backend = std::env::var("DOCUMENT_PARSER_BACKEND").ok();
    document_parser_backend_from_value(backend.as_deref())
}
