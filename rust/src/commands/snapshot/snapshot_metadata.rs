use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde_json::{json, Map, Value};

use crate::access;
use crate::common::Result;
use crate::dashboard::{CommonCliArgs, EXPORT_METADATA_FILENAME};

use super::snapshot_artifacts::build_snapshot_paths;

pub(crate) fn export_scope_kind_from_metadata_value(metadata: &Value) -> &str {
    metadata
        .get("scopeKind")
        .and_then(Value::as_str)
        .unwrap_or_else(|| {
            match metadata
                .get("variant")
                .and_then(Value::as_str)
                .unwrap_or_default()
            {
                "all-orgs-root" => "all-orgs-root",
                "root" => "org-root",
                _ => "",
            }
        })
}

fn rewrite_export_scope_kind(metadata_path: &Path, scope_kind: &str) -> Result<()> {
    if !metadata_path.is_file() {
        return Ok(());
    }
    let mut metadata =
        crate::common::load_json_object_file(metadata_path, "Snapshot export metadata")?;
    let object = metadata
        .as_object_mut()
        .ok_or_else(|| crate::common::message("Snapshot export metadata must be a JSON object."))?;
    object.insert(
        "scopeKind".to_string(),
        Value::String(scope_kind.to_string()),
    );
    fs::write(metadata_path, serde_json::to_string_pretty(&metadata)?)?;
    Ok(())
}

pub(super) fn annotate_snapshot_root_scope_kinds(output_dir: &Path) -> Result<()> {
    let paths = build_snapshot_paths(output_dir);
    rewrite_export_scope_kind(
        &paths.dashboards.join(EXPORT_METADATA_FILENAME),
        "workspace-root",
    )?;
    rewrite_export_scope_kind(
        &paths
            .datasources
            .join(super::super::SNAPSHOT_DATASOURCE_EXPORT_METADATA_FILENAME),
        "workspace-root",
    )?;
    Ok(())
}

fn snapshot_common_source(common: &CommonCliArgs) -> Map<String, Value> {
    let mut source = Map::from_iter(vec![("url".to_string(), Value::String(common.url.clone()))]);
    if let Some(profile) = &common.profile {
        source.insert("profile".to_string(), Value::String(profile.clone()));
    }
    source
}

fn load_metadata_count(metadata_path: &Path, count_keys: &[&str]) -> Result<Option<usize>> {
    if !metadata_path.is_file() {
        return Ok(None);
    }
    let metadata = crate::common::load_json_object_file(metadata_path, "Snapshot lane metadata")?;
    let object = metadata
        .as_object()
        .ok_or_else(|| crate::common::message("Snapshot lane metadata must be a JSON object."))?;
    for key in count_keys {
        if let Some(count) = object.get(*key).and_then(Value::as_u64) {
            return Ok(Some(count as usize));
        }
    }
    Ok(None)
}

pub(super) fn load_snapshot_lane_metadata_summary(
    lane_dir: &Path,
    payload_filename: &str,
    count_keys: &[&str],
    lane_kind: &str,
    lane_resource: &str,
) -> Result<Value> {
    let metadata_path = lane_dir.join(EXPORT_METADATA_FILENAME);
    let payload_path = lane_dir.join(payload_filename);
    let metadata_present = metadata_path.is_file();
    let payload_present = payload_path.is_file();
    let record_count = load_metadata_count(&metadata_path, count_keys)?.unwrap_or_else(|| {
        if payload_present {
            match fs::read_to_string(&payload_path)
                .ok()
                .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
            {
                Some(Value::Array(values)) => values.len(),
                Some(Value::Object(object)) => object
                    .get("records")
                    .and_then(Value::as_array)
                    .map(Vec::len)
                    .unwrap_or_default(),
                _ => 0,
            }
        } else {
            0
        }
    });
    Ok(json!({
        "present": metadata_present || payload_present,
        "metadataPresent": metadata_present,
        "payloadPresent": payload_present,
        "path": lane_dir.to_string_lossy(),
        "metadataPath": metadata_path.to_string_lossy(),
        "payloadPath": payload_path.to_string_lossy(),
        "kind": lane_kind,
        "resource": lane_resource,
        "recordCount": record_count,
    }))
}

pub(crate) fn build_snapshot_root_metadata(
    output_dir: &Path,
    common: &CommonCliArgs,
) -> Result<Value> {
    let paths = build_snapshot_paths(output_dir);
    let dashboard_metadata = load_snapshot_lane_metadata_summary(
        &paths.dashboards,
        "index.json",
        &["dashboardCount"],
        super::super::ROOT_INDEX_KIND,
        "dashboards",
    )?;
    let datasource_metadata = load_snapshot_lane_metadata_summary(
        &paths.datasources,
        super::super::SNAPSHOT_DATASOURCE_EXPORT_FILENAME,
        &["datasourceCount"],
        super::super::SNAPSHOT_DATASOURCE_ROOT_INDEX_KIND,
        "datasources",
    )?;
    let access_root = paths.access.clone();
    let access_users = load_snapshot_lane_metadata_summary(
        &access_root.join(super::super::SNAPSHOT_ACCESS_USERS_DIR),
        "users.json",
        &["recordCount"],
        access::ACCESS_EXPORT_KIND_USERS,
        "users",
    )?;
    let access_teams = load_snapshot_lane_metadata_summary(
        &access_root.join(super::super::SNAPSHOT_ACCESS_TEAMS_DIR),
        "teams.json",
        &["recordCount"],
        access::ACCESS_EXPORT_KIND_TEAMS,
        "teams",
    )?;
    let access_orgs = load_snapshot_lane_metadata_summary(
        &access_root.join(super::super::SNAPSHOT_ACCESS_ORGS_DIR),
        "orgs.json",
        &["recordCount"],
        access::ACCESS_EXPORT_KIND_ORGS,
        "orgs",
    )?;
    let access_service_accounts = load_snapshot_lane_metadata_summary(
        &access_root.join(super::super::SNAPSHOT_ACCESS_SERVICE_ACCOUNTS_DIR),
        "service-accounts.json",
        &["recordCount"],
        access::ACCESS_EXPORT_KIND_SERVICE_ACCOUNTS,
        "service-accounts",
    )?;
    let summary = json!({
        "dashboardCount": dashboard_metadata
            .get("recordCount")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        "datasourceCount": datasource_metadata
            .get("recordCount")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        "accessUserCount": access_users
            .get("recordCount")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        "accessTeamCount": access_teams
            .get("recordCount")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        "accessOrgCount": access_orgs
            .get("recordCount")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        "accessServiceAccountCount": access_service_accounts
            .get("recordCount")
            .and_then(Value::as_u64)
            .unwrap_or(0),
    });
    Ok(json!({
        "kind": "grafana-utils-snapshot-root",
        "schemaVersion": 2,
        "toolVersion": crate::common::tool_version(),
        "capturedAt": DateTime::<Utc>::from(std::time::SystemTime::now()).to_rfc3339(),
        "source": snapshot_common_source(common),
        "paths": {
            "root": output_dir.to_string_lossy(),
            "dashboards": paths.dashboards.to_string_lossy(),
            "datasources": paths.datasources.to_string_lossy(),
            "access": paths.access.to_string_lossy(),
        },
        "summary": summary,
        "lanes": {
            "dashboards": dashboard_metadata,
            "datasources": datasource_metadata,
            "access": {
                "users": access_users,
                "teams": access_teams,
                "orgs": access_orgs,
                "serviceAccounts": access_service_accounts,
            },
        }
    }))
}
