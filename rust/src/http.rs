use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, CONTENT_TYPE};
use reqwest::{Method, StatusCode, Url};
use serde_json::Value;

use crate::common::{api_response, message, Result};

#[derive(Debug, Clone)]
pub struct JsonHttpClientConfig {
    pub base_url: String,
    pub headers: Vec<(String, String)>,
    pub timeout_secs: u64,
    pub verify_ssl: bool,
}

pub struct JsonHttpClient {
    base_url: String,
    client: Client,
}

impl JsonHttpClient {
    pub fn new(config: JsonHttpClientConfig) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        for (name, value) in config.headers {
            let header_name = HeaderName::from_bytes(name.as_bytes())
                .map_err(|_| message(format!("Invalid header name: {name}")))?;
            let header_value = HeaderValue::from_str(&value)
                .map_err(|_| message(format!("Invalid header value for {name}")))?;
            headers.insert(header_name, header_value);
        }

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .danger_accept_invalid_certs(!config.verify_ssl)
            .build()?;

        Ok(Self {
            base_url: config.base_url.trim_end_matches('/').to_string(),
            client,
        })
    }

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
        let body = response.text()?;

        if status.is_client_error() || status.is_server_error() {
            return Err(api_response(status.as_u16(), url.to_string(), body));
        }

        if body.trim().is_empty() || status == StatusCode::NO_CONTENT {
            return Ok(None);
        }

        Ok(Some(serde_json::from_str(&body)?))
    }

    fn build_url(&self, path: &str, params: &[(String, String)]) -> Result<Url> {
        let mut url = Url::parse(&format!("{}{}", self.base_url, path))
            .map_err(|error| message(format!("Invalid request URL {path}: {error}")))?;
        if !params.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in params {
                pairs.append_pair(key, value);
            }
        }
        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use super::{JsonHttpClient, JsonHttpClientConfig};

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
}
