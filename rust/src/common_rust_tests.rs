use super::{resolve_auth_headers, sanitize_path_component};

#[test]
fn sanitize_path_component_normalizes_symbols_and_spaces() {
    assert_eq!(sanitize_path_component(" Ops / CPU % "), "Ops_CPU");
    assert_eq!(sanitize_path_component("..."), "untitled");
}

#[test]
fn resolve_auth_headers_prefers_bearer_token() {
    let headers = resolve_auth_headers(Some("abc123"), Some("user"), Some("pass")).unwrap();
    assert_eq!(headers[0], ("Authorization".to_string(), "Bearer abc123".to_string()));
}
