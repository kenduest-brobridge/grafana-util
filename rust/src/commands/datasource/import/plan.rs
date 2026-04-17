//! Datasource import request planning helpers.

use reqwest::Method;
use serde_json::{Map, Value};

use crate::common::{message, string_field, Result};
use crate::datasource::{fetch_datasource_by_uid_if_exists, resolve_match};
use crate::http::JsonHttpClient;

use super::datasource_import_export_support::DatasourceImportRecord;
use super::datasource_import_payload::build_import_payload_with_secret_values;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DatasourceUpdateTargetEvidence {
    pub(crate) uid: String,
    pub(crate) version: Option<i64>,
    pub(crate) read_only: bool,
}

pub(crate) struct PreparedDatasourceImportRequest {
    pub(crate) method: Method,
    pub(crate) path: String,
    pub(crate) payload: Value,
}

pub(crate) struct PreparedDatasourceImportPlan {
    pub(crate) requests: Vec<PreparedDatasourceImportRequest>,
    pub(crate) would_create: usize,
    pub(crate) would_update: usize,
    pub(crate) would_skip: usize,
}

fn datasource_record_identity(record: &DatasourceImportRecord) -> String {
    if record.uid.is_empty() {
        record.name.clone()
    } else {
        record.uid.clone()
    }
}

pub(crate) fn target_evidence_from_datasource(
    target_uid: &str,
    datasource: &Map<String, Value>,
) -> DatasourceUpdateTargetEvidence {
    DatasourceUpdateTargetEvidence {
        uid: string_field(datasource, "uid", target_uid),
        version: datasource.get("version").and_then(Value::as_i64),
        read_only: datasource
            .get("readOnly")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }
}

pub(crate) fn fetch_update_target_evidence(
    client: &JsonHttpClient,
    target_uid: &str,
    identity: &str,
) -> Result<DatasourceUpdateTargetEvidence> {
    if target_uid.trim().is_empty() {
        return Err(message(format!(
            "Datasource import update for {identity} requires a live datasource UID."
        )));
    }
    let target = fetch_datasource_by_uid_if_exists(client, target_uid)?.ok_or_else(|| {
        message(format!(
            "Datasource import update for {identity} matched UID {target_uid}, but the full target datasource could not be fetched."
        ))
    })?;
    Ok(target_evidence_from_datasource(target_uid, &target))
}

fn build_update_payload(
    record: &DatasourceImportRecord,
    secret_values: Option<&Map<String, Value>>,
    target: &DatasourceUpdateTargetEvidence,
) -> Result<Value> {
    if target.read_only {
        return Err(message(format!(
            "Datasource import for {} is blocked because target datasource uid={} is read-only. It is likely managed by Grafana provisioning.",
            datasource_record_identity(record),
            target.uid
        )));
    }
    let mut payload = build_import_payload_with_secret_values(record, secret_values)?
        .as_object()
        .cloned()
        .ok_or_else(|| message("Datasource import payload must be an object."))?;
    payload.insert("uid".to_string(), Value::String(target.uid.clone()));
    if let Some(version) = target.version {
        payload.insert("version".to_string(), Value::Number(version.into()));
    }
    Ok(Value::Object(payload))
}

pub(crate) fn prepare_datasource_import_plan(
    client: &JsonHttpClient,
    records: &[DatasourceImportRecord],
    live: &[Map<String, Value>],
    replace_existing: bool,
    update_existing_only: bool,
    secret_values: Option<&Map<String, Value>>,
) -> Result<PreparedDatasourceImportPlan> {
    prepare_datasource_import_plan_with_target_resolver(
        records,
        live,
        replace_existing,
        update_existing_only,
        secret_values,
        |target_uid, identity| fetch_update_target_evidence(client, target_uid, identity),
    )
}

pub(crate) fn prepare_datasource_import_plan_with_target_resolver<F>(
    records: &[DatasourceImportRecord],
    live: &[Map<String, Value>],
    replace_existing: bool,
    update_existing_only: bool,
    secret_values: Option<&Map<String, Value>>,
    mut resolve_update_target: F,
) -> Result<PreparedDatasourceImportPlan>
where
    F: FnMut(&str, &str) -> Result<DatasourceUpdateTargetEvidence>,
{
    let mut requests = Vec::new();
    let mut would_create = 0usize;
    let mut would_update = 0usize;
    let mut would_skip = 0usize;

    for record in records {
        let matching = resolve_match(record, live, replace_existing, update_existing_only);
        match matching.action {
            "would-create" => {
                let payload = build_import_payload_with_secret_values(record, secret_values)?;
                requests.push(PreparedDatasourceImportRequest {
                    method: Method::POST,
                    path: "/api/datasources".to_string(),
                    payload,
                });
                would_create += 1;
            }
            "would-update" => {
                let identity = datasource_record_identity(record);
                let target = resolve_update_target(&matching.target_uid, &identity)?;
                let payload = build_update_payload(record, secret_values, &target)?;
                requests.push(PreparedDatasourceImportRequest {
                    method: Method::PUT,
                    path: format!("/api/datasources/uid/{}", target.uid),
                    payload,
                });
                would_update += 1;
            }
            "would-skip-missing" => {
                would_skip += 1;
            }
            _ => {
                return Err(message(format!(
                    "Datasource import blocked for {}: destination={} action={}.",
                    if record.uid.is_empty() {
                        &record.name
                    } else {
                        &record.uid
                    },
                    matching.destination,
                    matching.action
                )));
            }
        }
    }

    Ok(PreparedDatasourceImportPlan {
        requests,
        would_create,
        would_update,
        would_skip,
    })
}
