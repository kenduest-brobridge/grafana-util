use serde_json::{Map, Value};

use crate::common::Result;
use crate::review_contract::{
    REVIEW_ACTION_WOULD_CREATE, REVIEW_ACTION_WOULD_DELETE, REVIEW_ACTION_WOULD_UPDATE,
};
use crate::sync::live::SyncApplyOperation;

use super::sync_live_apply_alert::apply_alert_operation_with_client;
use super::sync_live_apply_dashboard::apply_dashboard_operation_with_client;
use super::sync_live_apply_datasource::resolve_live_datasource_id;
use super::sync_live_apply_error::{
    datasource_sync_target_not_resolved, unsupported_datasource_sync_action,
    unsupported_sync_resource_kind,
};
use super::sync_live_apply_folder::apply_folder_operation_with_client;
use super::SyncLiveClient;

pub(super) fn apply_live_operation_with_client(
    client: &SyncLiveClient<'_>,
    operation: &SyncApplyOperation,
    allow_folder_delete: bool,
) -> Result<Value> {
    let kind = operation.kind.as_str();
    match kind {
        "folder" => apply_folder_operation_with_client(client, operation, allow_folder_delete),
        "dashboard" => apply_dashboard_operation_with_client(client, operation),
        "datasource" => apply_datasource_operation_with_client(client, operation),
        "alert"
        | "alert-contact-point"
        | "alert-mute-timing"
        | "alert-policy"
        | "alert-template" => apply_alert_operation_with_client(client, operation),
        _ => Err(unsupported_sync_resource_kind(kind)),
    }
}

fn apply_datasource_operation_with_client(
    client: &SyncLiveClient<'_>,
    operation: &SyncApplyOperation,
) -> Result<Value> {
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
        REVIEW_ACTION_WOULD_CREATE => Ok(Value::Object(
            client.create_datasource(&body)?.into_iter().collect(),
        )),
        REVIEW_ACTION_WOULD_UPDATE => {
            let target = client
                .resolve_datasource_target(identity)?
                .ok_or_else(|| datasource_sync_target_not_resolved(identity))?;
            let datasource_uid = resolve_live_datasource_uid(&target, identity)?;
            let datasource_id = resolve_live_datasource_id(&target, "update").ok();
            Ok(Value::Object(
                client
                    .update_datasource(&datasource_uid, datasource_id.as_deref(), &body)?
                    .into_iter()
                    .collect(),
            ))
        }
        REVIEW_ACTION_WOULD_DELETE => {
            let target = client
                .resolve_datasource_target(identity)?
                .ok_or_else(|| datasource_sync_target_not_resolved(identity))?;
            let datasource_uid = resolve_live_datasource_uid(&target, identity)?;
            let datasource_id = resolve_live_datasource_id(&target, "delete").ok();
            Ok(client.delete_datasource(&datasource_uid, datasource_id.as_deref())?)
        }
        _ => Err(unsupported_datasource_sync_action(action)),
    }
}

fn resolve_live_datasource_uid(target: &Map<String, Value>, identity: &str) -> Result<String> {
    target
        .get("uid")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| (!identity.trim().is_empty()).then_some(identity.trim()))
        .map(str::to_string)
        .ok_or_else(|| crate::common::message("Datasource live apply requires a datasource uid."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn datasource_target(uid: Value) -> Map<String, Value> {
        serde_json::from_value(json!({
            "uid": uid,
            "name": "Prometheus Main",
            "id": 7
        }))
        .expect("datasource target")
    }

    #[test]
    fn datasource_uid_prefers_live_target_uid() {
        let target = datasource_target(Value::String(" prom-main ".to_string()));

        assert_eq!(
            resolve_live_datasource_uid(&target, "fallback").unwrap(),
            "prom-main"
        );
    }

    #[test]
    fn datasource_uid_falls_back_to_identity_when_target_uid_is_blank() {
        let target = datasource_target(Value::String(" ".to_string()));

        assert_eq!(
            resolve_live_datasource_uid(&target, "prom-main").unwrap(),
            "prom-main"
        );
    }

    #[test]
    fn datasource_uid_requires_target_uid_or_identity() {
        let target = datasource_target(Value::String(" ".to_string()));

        assert_eq!(
            resolve_live_datasource_uid(&target, " ")
                .unwrap_err()
                .to_string(),
            "Datasource live apply requires a datasource uid."
        );
    }
}
