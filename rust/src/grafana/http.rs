//! Shared HTTP transport for all Rust domains.
//! Wraps reqwest blocking client creation, URL building, query encoding, and request/response error mapping.
use std::fs;

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, CONTENT_TYPE};
use reqwest::{Certificate, Method, StatusCode, Url};
use serde_json::Value;

use crate::common::{
    api_response, invalid_header_name, invalid_header_value, invalid_url, message, Result,
};

/// Struct definition for JsonHttpClientConfig.
#[derive(Debug, Clone)]
pub struct JsonHttpClientConfig {
    pub base_url: String,
    pub headers: Vec<(String, String)>,
    pub timeout_secs: u64,
    pub verify_ssl: bool,
}

/// Struct definition for JsonHttpClient.
#[derive(Clone)]
pub struct JsonHttpClient {
    base_url: String,
    client: Client,
}

impl JsonHttpClient {
    pub fn new(config: JsonHttpClientConfig) -> Result<Self> {
        Self::new_with_ca_cert(config, None)
    }

    pub fn new_with_ca_cert(
        config: JsonHttpClientConfig,
        ca_cert: Option<&std::path::Path>,
    ) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        for (name, value) in config.headers {
            let header_name =
                HeaderName::from_bytes(name.as_bytes()).map_err(|_| invalid_header_name(&name))?;
            let header_value = HeaderValue::from_str(&value)
                .map_err(|error| invalid_header_value(&name, error))?;
            headers.insert(header_name, header_value);
        }

        let mut builder = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .danger_accept_invalid_certs(!config.verify_ssl);
        if let Some(ca_cert_path) = ca_cert {
            let pem_bundle = fs::read(ca_cert_path)?;
            for cert in Certificate::from_pem_bundle(&pem_bundle)? {
                builder = builder.add_root_certificate(cert);
            }
        }
        let client = builder.build()?;

        Ok(Self {
            base_url: config.base_url.trim_end_matches('/').to_string(),
            client,
        })
    }

    /// Low-level HTTP execution hook used by all domain clients.
    /// Returns decoded JSON on success and maps non-2xx responses through domain Result errors.
    pub fn request_json(
        &self,
        method: Method,
        path: &str,
        params: &[(String, String)],
        payload: Option<&Value>,
    ) -> Result<Option<Value>> {
        let url = self.build_url(path, params)?;
        let mut request = self.client.request(method, url.clone());
        if payload.is_some() {
            request = request.header(CONTENT_TYPE, "application/json");
        }
        if let Some(payload) = payload {
            request = request.json(payload);
        }

        let response = request.send()?;
        let status = response.status();
        let body = response.bytes()?;

        if status.is_client_error() || status.is_server_error() {
            let body_text = String::from_utf8_lossy(&body).into_owned();
            return Err(api_response(status.as_u16(), url.to_string(), body_text));
        }

        if status == StatusCode::NO_CONTENT || body.iter().all(u8::is_ascii_whitespace) {
            return Ok(None);
        }

        Ok(Some(serde_json::from_slice(&body)?))
    }

    // Keep URL assembly consistent across callers.
    fn build_url(&self, path: &str, params: &[(String, String)]) -> Result<Url> {
        let context = format!("request path {path}");
        let url_text = format!("{}{}", self.base_url, path);
        let mut url = parse_request_url(&url_text, &context)?;
        if !params.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in params {
                pairs.append_pair(key, value);
            }
        }
        Ok(url)
    }
}

fn parse_request_url(url_text: &str, context: &str) -> Result<Url> {
    match Url::parse(url_text) {
        Ok(url) => Ok(url),
        Err(error) if error.to_string() == "invalid IPv4 address" => {
            let Some(host) = numeric_suffix_dns_host(url_text) else {
                return Err(invalid_url(context, error));
            };
            Err(message(format!("Unknown host {host} for {context}")))
        }
        Err(error) => Err(invalid_url(context, error)),
    }
}

fn numeric_suffix_dns_host(url_text: &str) -> Option<&str> {
    let scheme_end = url_text.find("://")?;
    let authority_start = scheme_end + 3;
    let authority_tail = &url_text[authority_start..];
    let authority_len = authority_tail
        .find(['/', '?', '#'])
        .unwrap_or(authority_tail.len());
    let authority_end = authority_start + authority_len;
    let authority = &url_text[authority_start..authority_end];
    let host_start_offset = authority.rfind('@').map_or(0, |index| index + 1);
    let host_port = &authority[host_start_offset..];
    if host_port.starts_with('[') {
        return None;
    }

    let host_len = match host_port.rfind(':') {
        Some(index) if host_port[index + 1..].chars().all(|ch| ch.is_ascii_digit()) => index,
        _ => host_port.len(),
    };
    let host = &host_port[..host_len];
    if !is_numeric_suffix_dns_host(host) {
        return None;
    }

    Some(host)
}

fn is_numeric_suffix_dns_host(host: &str) -> bool {
    if host.ends_with('.') {
        return false;
    }
    let labels = host.split('.').collect::<Vec<_>>();
    let Some(last) = labels.last() else {
        return false;
    };
    labels.len() >= 2
        && last.chars().all(|ch| ch.is_ascii_digit())
        && labels
            .iter()
            .any(|label| label.chars().any(|ch| ch.is_ascii_alphabetic()))
        && labels.iter().all(|label| is_dns_label(label))
}

fn is_dns_label(label: &str) -> bool {
    !label.is_empty()
        && !label.starts_with('-')
        && !label.ends_with('-')
        && label
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
}

#[cfg(test)]
#[path = "http_rust_tests.rs"]
mod http_rust_tests;
