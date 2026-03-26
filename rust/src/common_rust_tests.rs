//! Common utility unit tests.
//! Verifies path sanitization, shared error helpers, and authentication-header
//! resolution logic for all Rust domains.
use super::{
    editor, invalid_header_name, invalid_header_value, invalid_url, parse_error,
    resolve_auth_headers, resolve_auth_headers_with_prompt, sanitize_path_component, tui,
    validation, GrafanaCliError,
};

#[test]
fn sanitize_path_component_normalizes_symbols_and_spaces() {
    assert_eq!(sanitize_path_component(" Ops / CPU % "), "Ops_CPU");
    assert_eq!(sanitize_path_component("..."), "untitled");
}

#[test]
fn typed_error_helpers_keep_expected_categories_and_messages() {
    let validation_error = validation("invalid input");
    assert!(matches!(validation_error, GrafanaCliError::Validation(_)));
    assert_eq!(validation_error.kind(), "validation");
    assert_eq!(validation_error.to_string(), "invalid input");

    let tui_error = tui("Interactive review cancelled.");
    assert!(matches!(tui_error, GrafanaCliError::Tui(_)));
    assert_eq!(tui_error.kind(), "tui");

    let editor_error = editor("External editor exited.");
    assert!(matches!(editor_error, GrafanaCliError::Editor(_)));
    assert_eq!(editor_error.kind(), "editor");
}

#[test]
fn structured_local_parse_and_transport_helpers_include_context() {
    let url_error = invalid_url("dashboard URL", "relative URL without a base");
    assert!(matches!(url_error, GrafanaCliError::Url { .. }));
    assert_eq!(
        url_error.to_string(),
        "Invalid URL for dashboard URL: relative URL without a base"
    );
    assert_eq!(url_error.kind(), "url");

    let header_name_error = invalid_header_name("Bad Header");
    assert!(matches!(
        header_name_error,
        GrafanaCliError::HeaderName { .. }
    ));
    assert_eq!(
        header_name_error.to_string(),
        "Invalid header name: Bad Header"
    );
    assert_eq!(header_name_error.kind(), "header-name");

    let header_value_error = invalid_header_value("Authorization", "invalid byte");
    assert!(matches!(
        header_value_error,
        GrafanaCliError::HeaderValue { .. }
    ));
    assert_eq!(
        header_value_error.to_string(),
        "Invalid header value for Authorization: invalid byte"
    );
    assert_eq!(header_value_error.kind(), "header-value");

    let parse_failure = parse_error("org ID", "value must be >= 1");
    assert!(matches!(parse_failure, GrafanaCliError::Parse { .. }));
    assert_eq!(
        parse_failure.to_string(),
        "Failed to parse org ID: value must be >= 1"
    );
    assert_eq!(parse_failure.kind(), "parse");
}

#[test]
fn resolve_auth_headers_prefers_bearer_token() {
    let headers = resolve_auth_headers(Some("abc123"), None, None, false, false).unwrap();
    assert_eq!(
        headers[0],
        ("Authorization".to_string(), "Bearer abc123".to_string())
    );
}

#[test]
fn resolve_auth_headers_rejects_mixed_token_and_basic_auth() {
    let error =
        resolve_auth_headers(Some("abc123"), Some("user"), Some("pass"), false, false).unwrap_err();
    assert!(error.to_string().contains("Choose either token auth"));
}

#[test]
fn resolve_auth_headers_rejects_partial_basic_auth() {
    let error = resolve_auth_headers(None, Some("user"), None, false, false).unwrap_err();
    assert!(error.to_string().contains(
        "Basic auth requires both --basic-user and --basic-password or --prompt-password."
    ));
}

#[test]
fn resolve_auth_headers_supports_prompt_password() {
    let headers = resolve_auth_headers_with_prompt(
        None,
        Some("user"),
        None,
        true,
        false,
        || Ok("secret".to_string()),
        || Ok("ignored".to_string()),
    )
    .unwrap();
    assert_eq!(
        headers[0],
        (
            "Authorization".to_string(),
            "Basic dXNlcjpzZWNyZXQ=".to_string()
        )
    );
}

#[test]
fn resolve_auth_headers_rejects_prompt_without_username() {
    let error = resolve_auth_headers_with_prompt(
        None,
        None,
        None,
        true,
        false,
        || Ok("secret".to_string()),
        || Ok("ignored".to_string()),
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("--prompt-password requires --basic-user."));
}

#[test]
fn resolve_auth_headers_rejects_prompt_with_explicit_password() {
    let error = resolve_auth_headers_with_prompt(
        None,
        Some("user"),
        Some("pass"),
        true,
        false,
        || Ok("secret".to_string()),
        || Ok("ignored".to_string()),
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("Choose either --basic-password or --prompt-password, not both."));
}

#[test]
fn resolve_auth_headers_supports_prompt_token() {
    let headers = resolve_auth_headers_with_prompt(
        None,
        None,
        None,
        false,
        true,
        || Ok("ignored".to_string()),
        || Ok("prompt-token".to_string()),
    )
    .unwrap();
    assert_eq!(
        headers[0],
        (
            "Authorization".to_string(),
            "Bearer prompt-token".to_string()
        )
    );
}

#[test]
fn resolve_auth_headers_rejects_prompt_token_with_explicit_token() {
    let error = resolve_auth_headers_with_prompt(
        Some("abc123"),
        None,
        None,
        false,
        true,
        || Ok("ignored".to_string()),
        || Ok("prompt-token".to_string()),
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("Choose either --token / --api-token or --prompt-token, not both."));
}
