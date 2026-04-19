use reqwest::Method;
use serde_json::Value;

use crate::common::Result;
use crate::review_contract::{
    REVIEW_ACTION_WOULD_CREATE, REVIEW_ACTION_WOULD_DELETE, REVIEW_ACTION_WOULD_UPDATE,
};
use crate::sync::live::SyncApplyOperation;

use super::super::sync_live_apply_alert::apply_alert_operation_with_request;
use super::super::sync_live_apply_dashboard::apply_dashboard_operation_with_request;
use super::super::sync_live_apply_datasource::{
    resolve_live_datasource_id, resolve_live_datasource_target,
};
use super::super::sync_live_apply_error::{
    datasource_sync_target_not_resolved, unsupported_datasource_sync_action,
    unsupported_sync_resource_kind,
};
use super::super::sync_live_apply_folder::apply_folder_operation_with_request;
use super::super::sync_live_apply_phase::execute_live_apply_phase;

pub(crate) fn execute_live_apply_with_request<F>(
    mut request_json: F,
    operations: &[SyncApplyOperation],
    allow_folder_delete: bool,
    allow_policy_reset: bool,
) -> Result<Value>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    execute_live_apply_phase(operations, allow_policy_reset, |operation| {
        apply_live_operation_with_request(&mut request_json, operation, allow_folder_delete)
    })
}

fn apply_live_operation_with_request<F>(
    request_json: &mut F,
    operation: &SyncApplyOperation,
    allow_folder_delete: bool,
) -> Result<Value>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let kind = operation.kind.as_str();
    match kind {
        "folder" => {
            apply_folder_operation_with_request(request_json, operation, allow_folder_delete)
        }
        "dashboard" => apply_dashboard_operation_with_request(request_json, operation),
        "datasource" => apply_datasource_operation_with_request(request_json, operation),
        "alert"
        | "alert-contact-point"
        | "alert-mute-timing"
        | "alert-policy"
        | "alert-template" => apply_alert_operation_with_request(request_json, operation),
        _ => Err(unsupported_sync_resource_kind(kind)),
    }
}

fn apply_datasource_operation_with_request<F>(
    request_json: &mut F,
    operation: &SyncApplyOperation,
) -> Result<Value>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let action = operation.action.as_str();
    let identity = operation.identity.as_str();
    let mut body = operation.desired.clone();
    if !identity.is_empty() {
        body.entry("uid".to_string())
            .or_insert_with(|| Value::String(identity.to_string()));
    }
    let title = body
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value: &&str| !value.is_empty())
        .unwrap_or(identity);
    body.insert("name".to_string(), Value::String(title.to_string()));
    match action {
        REVIEW_ACTION_WOULD_CREATE => Ok(request_json(
            Method::POST,
            "/api/datasources",
            &[],
            Some(&Value::Object(body)),
        )?
        .unwrap_or(Value::Null)),
        REVIEW_ACTION_WOULD_UPDATE => {
            let datasources = match request_json(Method::GET, "/api/datasources", &[], None)? {
                Some(Value::Array(items)) => items,
                Some(_) => {
                    return Err(crate::common::message(
                        "Unexpected datasource list response from Grafana.",
                    ))
                }
                None => Vec::new(),
            };
            let datasources = datasources
                .iter()
                .map(|item| {
                    crate::sync::require_json_object(item, "Grafana datasource payload").cloned()
                })
                .collect::<Result<Vec<_>>>()?;
            let target = resolve_live_datasource_target(&datasources, identity)?
                .ok_or_else(|| datasource_sync_target_not_resolved(identity))?;
            let datasource_id = resolve_live_datasource_id(&target, "update")?;
            Ok(request_json(
                Method::PUT,
                &format!("/api/datasources/{datasource_id}"),
                &[],
                Some(&Value::Object(body)),
            )?
            .unwrap_or(Value::Null))
        }
        REVIEW_ACTION_WOULD_DELETE => {
            let datasources = match request_json(Method::GET, "/api/datasources", &[], None)? {
                Some(Value::Array(items)) => items,
                Some(_) => {
                    return Err(crate::common::message(
                        "Unexpected datasource list response from Grafana.",
                    ))
                }
                None => Vec::new(),
            };
            let datasources = datasources
                .iter()
                .map(|item| {
                    crate::sync::require_json_object(item, "Grafana datasource payload").cloned()
                })
                .collect::<Result<Vec<_>>>()?;
            let target = resolve_live_datasource_target(&datasources, identity)?
                .ok_or_else(|| datasource_sync_target_not_resolved(identity))?;
            let datasource_id = resolve_live_datasource_id(&target, "delete")?;
            Ok(request_json(
                Method::DELETE,
                &format!("/api/datasources/{datasource_id}"),
                &[],
                None,
            )?
            .unwrap_or(Value::Null))
        }
        _ => Err(unsupported_datasource_sync_action(action)),
    }
}
