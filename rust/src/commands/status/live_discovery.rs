use reqwest::Method;
use serde_json::{Map, Value};

use crate::common::Result;
use crate::http::JsonHttpClient;

use super::PROJECT_STATUS_LIVE_INSTANCE_SOURCE;

fn unavailable_live_instance_discovery(error: impl Into<String>) -> Value {
    let mut instance = Map::new();
    instance.insert(
        "source".to_string(),
        Value::String(PROJECT_STATUS_LIVE_INSTANCE_SOURCE.to_string()),
    );
    instance.insert(
        "status".to_string(),
        Value::String("unavailable".to_string()),
    );
    instance.insert("error".to_string(), Value::String(error.into()));
    Value::Object(instance)
}

pub(super) fn build_live_instance_discovery_with_request<F>(request_json: &mut F) -> Value
where
    F: FnMut(reqwest::Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    match request_json(Method::GET, "/api/health", &[], None) {
        Ok(Some(Value::Object(health))) => {
            let mut instance = Map::new();
            instance.insert(
                "source".to_string(),
                Value::String(PROJECT_STATUS_LIVE_INSTANCE_SOURCE.to_string()),
            );
            instance.insert("status".to_string(), Value::String("available".to_string()));
            instance.insert("health".to_string(), Value::Object(health));
            Value::Object(instance)
        }
        Ok(Some(value)) => unavailable_live_instance_discovery(format!(
            "Grafana /api/health returned {} instead of an object.",
            live_instance_payload_type(&value)
        )),
        Ok(None) => unavailable_live_instance_discovery("Grafana /api/health returned no body."),
        Err(error) => unavailable_live_instance_discovery(error.to_string()),
    }
}

fn live_instance_payload_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}

fn build_live_instance_discovery(client: &JsonHttpClient) -> Value {
    build_live_instance_discovery_with_request(&mut |method, path, params, payload| {
        client.request_json(method, path, params, payload)
    })
}

pub(super) fn build_live_status_discovery(client: &JsonHttpClient) -> Value {
    let mut discovery = Map::new();
    discovery.insert(
        "instance".to_string(),
        build_live_instance_discovery(client),
    );
    Value::Object(discovery)
}
