#[cfg(test)]
use reqwest::Method;
use serde_json::{Map, Value};

use crate::common::Result;
use crate::review_contract::{REVIEW_ACTION_WOULD_DELETE, REVIEW_ACTION_WOULD_UPDATE};
use crate::sync::live::SyncApplyOperation;

use super::SyncLiveClient;

pub(crate) fn apply_dashboard_operation_with_client(
    client: &SyncLiveClient<'_>,
    operation: &SyncApplyOperation,
) -> Result<Value> {
    let action = operation.action.as_str();
    let identity = operation.identity.as_str();
    if action == REVIEW_ACTION_WOULD_DELETE {
        return client.delete_dashboard(identity);
    }
    let body = build_dashboard_body(identity, &operation.desired);
    client.upsert_dashboard(
        &body,
        action == REVIEW_ACTION_WOULD_UPDATE,
        dashboard_folder_uid(&body),
    )
}

#[cfg(test)]
pub(crate) fn apply_dashboard_operation_with_request<F>(
    request_json: &mut F,
    operation: &SyncApplyOperation,
) -> Result<Value>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let action = operation.action.as_str();
    let identity = operation.identity.as_str();
    if action == REVIEW_ACTION_WOULD_DELETE {
        return Ok(request_json(
            Method::DELETE,
            &format!("/api/dashboards/uid/{identity}"),
            &[],
            None,
        )?
        .unwrap_or(Value::Null));
    }
    let body = build_dashboard_body(identity, &operation.desired);
    Ok(request_json(
        Method::POST,
        "/api/dashboards/db",
        &[],
        Some(&Value::Object(build_dashboard_upsert_payload(
            &body,
            action == REVIEW_ACTION_WOULD_UPDATE,
        ))),
    )?
    .unwrap_or(Value::Null))
}

fn build_dashboard_body(identity: &str, desired: &Map<String, Value>) -> Map<String, Value> {
    let mut body = desired.clone();
    body.insert("uid".to_string(), Value::String(identity.to_string()));
    body.insert(
        "title".to_string(),
        Value::String(dashboard_title(identity, &body).to_string()),
    );
    body.remove("id");
    body
}

#[cfg(test)]
fn build_dashboard_upsert_payload(
    body: &Map<String, Value>,
    overwrite: bool,
) -> Map<String, Value> {
    let mut payload = Map::new();
    payload.insert("dashboard".to_string(), Value::Object(body.clone()));
    payload.insert("overwrite".to_string(), Value::Bool(overwrite));
    if let Some(folder_uid) = dashboard_folder_uid(body) {
        payload.insert(
            "folderUid".to_string(),
            Value::String(folder_uid.to_string()),
        );
    }
    payload
}

fn dashboard_title<'a>(identity: &'a str, body: &'a Map<String, Value>) -> &'a str {
    body.get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value: &&str| !value.is_empty())
        .unwrap_or(identity)
}

fn dashboard_folder_uid(body: &Map<String, Value>) -> Option<&str> {
    body.get("folderUid")
        .or_else(|| body.get("folderUID"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value: &&str| !value.is_empty())
}
