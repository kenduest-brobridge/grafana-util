use crate::http::{JsonHttpClient, JsonHttpClientConfig};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

pub(super) fn build_test_client(
    responses: Vec<String>,
) -> (
    JsonHttpClient,
    Arc<Mutex<Vec<String>>>,
    thread::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let requests_thread = Arc::clone(&requests);
    let handle = thread::spawn(move || {
        for response in responses {
            let (mut stream, _) = listener.accept().unwrap();
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
            let request_line = String::from_utf8_lossy(&request)
                .lines()
                .next()
                .unwrap_or_default()
                .to_string();
            requests_thread.lock().unwrap().push(request_line);
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    let client = JsonHttpClient::new(JsonHttpClientConfig {
        base_url: format!("http://{addr}"),
        headers: vec![("Authorization".to_string(), "Bearer token".to_string())],
        timeout_secs: 5,
        verify_ssl: false,
    })
    .unwrap();
    (client, requests, handle)
}

pub(super) fn http_response(status: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}
