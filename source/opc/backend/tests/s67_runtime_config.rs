#[test]
fn defaults_to_loopback_on_standard_port() {
    let addr = opc::runtime_config::bind_addr_from_values(None, None)
        .expect("default host and port should produce a socket address");

    assert_eq!(addr.to_string(), "127.0.0.1:8987");
}

#[test]
fn uses_explicit_host_and_port() {
    let addr = opc::runtime_config::bind_addr_from_values(Some("127.0.0.1"), Some("8991"))
        .expect("explicit host and port should produce a socket address");

    assert_eq!(addr.to_string(), "127.0.0.1:8991");
}

#[test]
fn frontend_dir_uses_deployment_override() {
    let dir = opc::runtime_config::frontend_dir_from_values(
        Some("/srv/example-erp/frontend/dist"),
        "/build/source/backend",
    );

    assert_eq!(
        dir,
        std::path::PathBuf::from("/srv/example-erp/frontend/dist")
    );
}

#[test]
fn frontend_dir_defaults_relative_to_manifest_dir() {
    let dir = opc::runtime_config::frontend_dir_from_values(None, "/build/source/backend");

    assert_eq!(
        dir,
        std::path::PathBuf::from("/build/source/backend/../frontend/dist")
    );
}

#[test]
fn ai_model_defaults_to_deepseek_flash() {
    assert_eq!(
        opc::runtime_config::ai_model_from_value(None),
        opc::runtime_config::DEFAULT_AI_MODEL
    );
    assert_eq!(
        opc::runtime_config::ai_model_from_value(Some("  ")),
        opc::runtime_config::DEFAULT_AI_MODEL
    );
    assert_eq!(
        opc::runtime_config::ai_model_from_value(Some("deepseek-v4-pro")),
        "deepseek-v4-pro"
    );
}

#[test]
fn document_parser_backend_defaults_to_ollama_vision_and_accepts_openai_vision() {
    assert_eq!(
        opc::runtime_config::document_parser_backend_from_value(None),
        opc::runtime_config::DEFAULT_DOCUMENT_PARSER_BACKEND
    );
    assert_eq!(
        opc::runtime_config::document_parser_backend_from_value(Some(" liteparse ")),
        "liteparse"
    );
    assert_eq!(
        opc::runtime_config::document_parser_backend_from_value(Some("paddleocr")),
        "paddleocr"
    );
    assert_eq!(
        opc::runtime_config::document_parser_backend_from_value(Some("openai-vision")),
        opc::runtime_config::OPENAI_VISION_BACKEND
    );
}
