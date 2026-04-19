#[cfg(test)]
use reqwest::Method;
use serde_json::{Map, Value};

use crate::common::Result;
use crate::review_contract::{
    REVIEW_ACTION_WOULD_CREATE, REVIEW_ACTION_WOULD_DELETE, REVIEW_ACTION_WOULD_UPDATE,
};
use crate::sync::live::SyncApplyOperation;

use super::sync_live_apply_error::{refuse_live_folder_delete, unsupported_folder_sync_action};
use super::SyncLiveClient;

pub(crate) fn apply_folder_operation_with_client(
    client: &SyncLiveClient<'_>,
    operation: &SyncApplyOperation,
    allow_folder_delete: bool,
) -> Result<Value> {
    let action = operation.action.as_str();
    let identity = operation.identity.as_str();
    let desired = &operation.desired;
    match action {
        REVIEW_ACTION_WOULD_CREATE => Ok(Value::Object(
            client
                .create_folder(
                    folder_title(identity, desired),
                    identity,
                    folder_parent_uid(desired),
                )?
                .into_iter()
                .collect(),
        )),
        REVIEW_ACTION_WOULD_UPDATE => Ok(Value::Object(
            client
                .update_folder(identity, desired)?
                .into_iter()
                .collect(),
        )),
        REVIEW_ACTION_WOULD_DELETE => {
            if !allow_folder_delete {
                return Err(refuse_live_folder_delete(identity));
            }
            Ok(client.delete_folder(identity)?)
        }
        _ => Err(unsupported_folder_sync_action(action)),
    }
}

#[cfg(test)]
pub(crate) fn apply_folder_operation_with_request<F>(
    request_json: &mut F,
    operation: &SyncApplyOperation,
    allow_folder_delete: bool,
) -> Result<Value>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let action = operation.action.as_str();
    let identity = operation.identity.as_str();
    let desired = &operation.desired;
    match action {
        REVIEW_ACTION_WOULD_CREATE => Ok(request_json(
            Method::POST,
            "/api/folders",
            &[],
            Some(&Value::Object(build_folder_create_payload(
                identity, desired,
            ))),
        )?
        .unwrap_or(Value::Null)),
        REVIEW_ACTION_WOULD_UPDATE => Ok(request_json(
            Method::PUT,
            &format!("/api/folders/{identity}"),
            &[],
            Some(&Value::Object(desired.clone())),
        )?
        .unwrap_or(Value::Null)),
        REVIEW_ACTION_WOULD_DELETE => {
            if !allow_folder_delete {
                return Err(refuse_live_folder_delete(identity));
            }
            Ok(request_json(
                Method::DELETE,
                &format!("/api/folders/{identity}"),
                &[("forceDeleteRules".to_string(), "false".to_string())],
                None,
            )?
            .unwrap_or(Value::Null))
        }
        _ => Err(unsupported_folder_sync_action(action)),
    }
}

#[cfg(test)]
fn build_folder_create_payload(identity: &str, desired: &Map<String, Value>) -> Map<String, Value> {
    let mut payload = Map::new();
    payload.insert("uid".to_string(), Value::String(identity.to_string()));
    payload.insert(
        "title".to_string(),
        Value::String(folder_title(identity, desired).to_string()),
    );
    if let Some(parent_uid) = folder_parent_uid(desired) {
        payload.insert(
            "parentUid".to_string(),
            Value::String(parent_uid.to_string()),
        );
    }
    payload
}

fn folder_title<'a>(identity: &'a str, desired: &'a Map<String, Value>) -> &'a str {
    desired
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value: &&str| !value.is_empty())
        .unwrap_or(identity)
}

fn folder_parent_uid(desired: &Map<String, Value>) -> Option<&str> {
    desired
        .get("parentUid")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value: &&str| !value.is_empty())
}
