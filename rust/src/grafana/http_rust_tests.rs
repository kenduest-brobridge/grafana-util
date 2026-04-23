//! HTTP transport unit tests.
//! Checks client construction behavior and can be extended for request/URL building
//! contract coverage.
use super::{JsonHttpClient, JsonHttpClientConfig};
use crate::common::GrafanaCliError;
use reqwest::Method;
use serde_json::json;
use std::io::ErrorKind;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

#[test]
fn client_builder_accepts_basic_config() {
    let client = JsonHttpClient::new(JsonHttpClientConfig {
        base_url: "http://127.0.0.1:3000".to_string(),
        headers: vec![("Authorization".to_string(), "Bearer token".to_string())],
        timeout_secs: 30,
        verify_ssl: false,
    });
    assert!(client.is_ok());
}

#[test]
fn client_builder_rejects_invalid_header_name_with_typed_error() {
    let result = JsonHttpClient::new(JsonHttpClientConfig {
        base_url: "http://127.0.0.1:3000".to_string(),
        headers: vec![("Bad Header".to_string(), "Bearer token".to_string())],
        timeout_secs: 30,
        verify_ssl: false,
    });

    let error = match result {
        Ok(_) => panic!("expected invalid header name error"),
        Err(error) => error,
    };

    assert!(matches!(error, GrafanaCliError::HeaderName { .. }));
    assert_eq!(error.to_string(), "Invalid header name: Bad Header");
}

#[test]
fn client_builder_rejects_invalid_header_value_with_typed_error() {
    let result = JsonHttpClient::new(JsonHttpClientConfig {
        base_url: "http://127.0.0.1:3000".to_string(),
        headers: vec![("Authorization".to_string(), "token\nnewline".to_string())],
        timeout_secs: 30,
        verify_ssl: false,
    });

    let error = match result {
        Ok(_) => panic!("expected invalid header value error"),
        Err(error) => error,
    };

    assert!(matches!(error, GrafanaCliError::HeaderValue { .. }));
    assert!(error
        .to_string()
        .starts_with("Invalid header value for Authorization:"));
}

#[test]
fn request_json_rejects_invalid_url_with_typed_error() {
    let client = JsonHttpClient::new(JsonHttpClientConfig {
        base_url: "http://[::1".to_string(),
        headers: Vec::new(),
        timeout_secs: 30,
        verify_ssl: false,
    })
    .unwrap();

    let error = client
        .request_json(Method::GET, "/api/search", &[], None)
        .unwrap_err();

    assert!(matches!(error, GrafanaCliError::Url { .. }));
    assert!(error
        .to_string()
        .starts_with("Invalid URL for request path /api/search: "));
}

#[test]
fn request_json_preserves_api_response_context() {
    let listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => listener,
        Err(error) if error.kind() == ErrorKind::PermissionDenied => return,
        Err(error) => panic!("failed to bind test listener: {error}"),
    };
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();

        let mut request = Vec::new();
        let mut buffer = [0_u8; 1024];
        loop {
            let bytes_read = stream.read(&mut buffer).unwrap();
            if bytes_read == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes_read]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }

        let response = b"HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: 9\r\nConnection: close\r\n\r\nnot found";
        stream.write_all(response).unwrap();
        let _ = stream.flush();
    });

    let client = JsonHttpClient::new(JsonHttpClientConfig {
        base_url: format!("http://{address}"),
        headers: vec![],
        timeout_secs: 30,
        verify_ssl: false,
    })
    .unwrap();

    let error = client
        .request_json(
            Method::GET,
            "/api/search",
            &[
                ("query".to_string(), "alerting rules".to_string()),
                ("folder".to_string(), "Ops/Prod".to_string()),
            ],
            None,
        )
        .unwrap_err();

    server.join().unwrap();

    match error {
        GrafanaCliError::ApiResponse {
            status_code,
            url,
            body,
        } => {
            assert_eq!(status_code, 404);
            assert_eq!(
                url,
                format!("http://{address}/api/search?query=alerting+rules&folder=Ops%2FProd")
            );
            assert_eq!(body, "not found");
        }
        other => panic!("expected ApiResponse error, got {other:?}"),
    }
}

#[test]
fn request_json_uses_reqwest_managed_accept_encoding_and_parses_success_json() {
    let listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => listener,
        Err(error) if error.kind() == ErrorKind::PermissionDenied => return,
        Err(error) => panic!("failed to bind test listener: {error}"),
    };
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();

        let mut request = Vec::new();
        let mut buffer = [0_u8; 1024];
        loop {
            let bytes_read = stream.read(&mut buffer).unwrap();
            if bytes_read == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes_read]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }

        let response = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 11\r\nConnection: close\r\n\r\n{\"ok\":true}";
        stream.write_all(response).unwrap();
        let _ = stream.flush();

        String::from_utf8_lossy(&request).to_string()
    });

    let client = JsonHttpClient::new(JsonHttpClientConfig {
        base_url: format!("http://{address}"),
        headers: vec![],
        timeout_secs: 30,
        verify_ssl: false,
    })
    .unwrap();

    let body = client
        .request_json(Method::GET, "/api/search", &[], None)
        .unwrap()
        .unwrap();
    let request = server.join().unwrap();
    let normalized_request = request.to_ascii_lowercase();

    assert_eq!(body, json!({"ok": true}));
    assert!(normalized_request.contains("accept: application/json"));
    assert!(normalized_request.contains("accept-encoding:"));
    assert!(!normalized_request.contains("accept-encoding: identity"));
}

#[test]
fn request_json_treats_whitespace_success_body_as_empty() {
    let listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => listener,
        Err(error) if error.kind() == ErrorKind::PermissionDenied => return,
        Err(error) => panic!("failed to bind test listener: {error}"),
    };
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();

        let mut request = Vec::new();
        let mut buffer = [0_u8; 1024];
        loop {
            let bytes_read = stream.read(&mut buffer).unwrap();
            if bytes_read == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes_read]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }

        let response = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 4\r\nConnection: close\r\n\r\n \n\t ";
        stream.write_all(response).unwrap();
        let _ = stream.flush();
    });

    let client = JsonHttpClient::new(JsonHttpClientConfig {
        base_url: format!("http://{address}"),
        headers: vec![],
        timeout_secs: 30,
        verify_ssl: false,
    })
    .unwrap();

    let body = client
        .request_json(Method::GET, "/api/search", &[], None)
        .unwrap();

    server.join().unwrap();
    assert_eq!(body, None);
}
