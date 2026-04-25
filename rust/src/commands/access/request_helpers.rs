//! Grafana request response-shape helpers for access workflows.

use reqwest::Method;
use serde_json::{Map, Value};

use crate::common::{message, value_as_object, Result};

pub(super) fn request_object<F>(
    mut request_json: F,
    method: Method,
    path: &str,
    params: &[(String, String)],
    payload: Option<&Value>,
    error_message: &str,
) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    // Normalize "one object expected" API responses into a single helper so
    // callers can focus on workflow logic instead of response-shape validation.
    let value =
        request_json(method, path, params, payload)?.ok_or_else(|| message(error_message))?;
    Ok(value_as_object(&value, error_message)?.clone())
}

pub(super) fn request_array<F>(
    mut request_json: F,
    method: Method,
    path: &str,
    params: &[(String, String)],
    payload: Option<&Value>,
    error_message: &str,
) -> Result<Vec<Map<String, Value>>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    // Treat missing list payloads as empty collections to match Grafana list-style
    // endpoints that can legitimately return no body or an empty array.
    match request_json(method, path, params, payload)? {
        Some(Value::Array(items)) => items
            .into_iter()
            .map(|item| Ok(value_as_object(&item, error_message)?.clone()))
            .collect(),
        Some(_) => Err(message(error_message)),
        None => Ok(Vec::new()),
    }
}

pub(super) fn request_object_list_field<F>(
    mut request_json: F,
    method: Method,
    path: &str,
    params: &[(String, String)],
    payload: Option<&Value>,
    field: &str,
    error_messages: (&str, &str),
) -> Result<Vec<Map<String, Value>>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let (object_error_message, list_error_message) = error_messages;
    let object = request_object(
        &mut request_json,
        method,
        path,
        params,
        payload,
        object_error_message,
    )?;
    match object.get(field) {
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| Ok(value_as_object(item, list_error_message)?.clone()))
            .collect(),
        _ => Err(message(list_error_message)),
    }
}
