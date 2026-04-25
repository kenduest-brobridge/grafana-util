//! Dashboard bundle input normalization helpers.

use super::bundle_inputs_datasource::{
    load_datasource_provisioning_records, normalize_datasource_bundle_item,
};
use super::json::{
    discover_json_files, load_json_array_file, load_json_value, require_json_object,
};
use crate::common::{message, Result};
use crate::dashboard::DASHBOARD_PERMISSION_BUNDLE_FILENAME;
use crate::dashboard::{
    load_dashboard_source, resolve_dashboard_workspace_variant_dir, DashboardImportInputFormat,
    RAW_EXPORT_SUBDIR,
};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::Path;

pub(crate) type DashboardBundleSections = (Vec<Value>, Vec<Value>, Vec<Value>, Map<String, Value>);

#[derive(Debug, Clone, Default)]
struct DashboardOwnershipSource {
    ownership: String,
    provenance: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct DashboardOwnershipIndex {
    by_path: BTreeMap<String, DashboardOwnershipSource>,
    by_uid: BTreeMap<String, DashboardOwnershipSource>,
}

fn normalize_text(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .unwrap_or_default()
        .to_string()
}

fn normalize_string_list(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn dashboard_ownership_source_from_object(
    object: &Map<String, Value>,
) -> Option<DashboardOwnershipSource> {
    let ownership = normalize_text(object.get("ownership"));
    let provenance = normalize_string_list(object.get("provenance"));
    if ownership.is_empty() && provenance.is_empty() {
        None
    } else {
        Some(DashboardOwnershipSource {
            ownership,
            provenance,
        })
    }
}

fn load_dashboard_ownership_index(dashboard_source_dir: &Path) -> DashboardOwnershipIndex {
    let index_path = dashboard_source_dir.join("index.json");
    let Ok(entries) = load_json_array_file(&index_path, "Dashboard export index") else {
        return DashboardOwnershipIndex::default();
    };
    let mut index = DashboardOwnershipIndex::default();
    for entry in entries {
        let Some(object) = entry.as_object() else {
            continue;
        };
        let Some(ownership) = dashboard_ownership_source_from_object(object) else {
            continue;
        };
        let path = normalize_text(object.get("path"));
        if !path.is_empty() {
            index.by_path.insert(path, ownership.clone());
        }
        let uid = normalize_text(object.get("uid"));
        if !uid.is_empty() {
            index.by_uid.insert(uid, ownership);
        }
    }
    index
}

fn attach_dashboard_ownership(
    item: &mut Map<String, Value>,
    ownership: Option<&DashboardOwnershipSource>,
) {
    let Some(ownership) = ownership else {
        return;
    };
    if !ownership.ownership.is_empty() {
        item.insert(
            "ownership".to_string(),
            Value::String(ownership.ownership.clone()),
        );
    }
    if !ownership.provenance.is_empty() {
        item.insert(
            "provenance".to_string(),
            Value::Array(
                ownership
                    .provenance
                    .iter()
                    .cloned()
                    .map(Value::String)
                    .collect(),
            ),
        );
    }
}

fn normalize_dashboard_bundle_item(
    document: &Value,
    source_path: &str,
    ownership: Option<&DashboardOwnershipSource>,
) -> Result<Value> {
    let mut body = if let Some(body) = document.get("dashboard").and_then(Value::as_object) {
        body.clone()
    } else {
        require_json_object(document, "Dashboard export document")?.clone()
    };
    body.remove("id");
    let uid = body
        .get("uid")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            message(format!(
                "Dashboard export document is missing dashboard.uid: {source_path}"
            ))
        })?;
    let title = body
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(uid);
    let mut item = Map::new();
    item.insert("kind".to_string(), Value::String("dashboard".to_string()));
    item.insert("uid".to_string(), Value::String(uid.to_string()));
    item.insert("title".to_string(), Value::String(title.to_string()));
    item.insert("body".to_string(), Value::Object(body));
    item.insert(
        "sourcePath".to_string(),
        Value::String(source_path.to_string()),
    );
    attach_dashboard_ownership(&mut item, ownership);
    Ok(Value::Object(item))
}

pub(crate) fn normalize_folder_bundle_item(document: &Value) -> Result<Value> {
    let object = require_json_object(document, "Folder inventory record")?;
    let uid = object
        .get("uid")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| message("Folder inventory record is missing uid."))?;
    let title = object
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(uid);
    let mut body = Map::new();
    body.insert("title".to_string(), Value::String(title.to_string()));
    if let Some(parent_uid) = object
        .get("parentUid")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        body.insert(
            "parentUid".to_string(),
            Value::String(parent_uid.to_string()),
        );
    }
    if let Some(path) = object
        .get("path")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        body.insert("path".to_string(), Value::String(path.to_string()));
    }
    Ok(serde_json::json!({
        "kind": "folder",
        "uid": uid,
        "title": title,
        "body": body,
        "sourcePath": object.get("sourcePath").cloned().unwrap_or(Value::String(String::new())),
    }))
}

pub(crate) fn load_dashboard_bundle_sections(
    dashboard_dir: &Path,
    metadata_dir: &Path,
    datasource_provisioning_file: Option<&Path>,
) -> Result<DashboardBundleSections> {
    let dashboard_source_dir =
        resolve_dashboard_workspace_variant_dir(dashboard_dir, RAW_EXPORT_SUBDIR)
            .unwrap_or_else(|| dashboard_dir.to_path_buf());
    let mut dashboards = Vec::new();
    let ownership_index = load_dashboard_ownership_index(&dashboard_source_dir);
    for path in discover_json_files(
        &dashboard_source_dir,
        &[
            "index.json",
            "export-metadata.json",
            "folders.json",
            "datasources.json",
            DASHBOARD_PERMISSION_BUNDLE_FILENAME,
        ],
    )? {
        let source_path = path
            .strip_prefix(&dashboard_source_dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let document = load_json_value(&path, "Dashboard export document")?;
        let uid = document
            .get("dashboard")
            .and_then(|dashboard| dashboard.get("uid"))
            .or_else(|| document.get("uid"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let ownership = ownership_index
            .by_path
            .get(&source_path)
            .or_else(|| uid.and_then(|uid| ownership_index.by_uid.get(uid)));
        dashboards.push(normalize_dashboard_bundle_item(
            &document,
            &source_path,
            ownership,
        )?);
    }
    let folders_path = metadata_dir.join("folders.json");
    let folders = if folders_path.is_file() {
        load_json_array_file(&folders_path, "Dashboard folder inventory")?
            .into_iter()
            .map(|item| normalize_folder_bundle_item(&item))
            .collect::<Result<Vec<_>>>()?
    } else {
        Vec::new()
    };
    let datasources = if let Some(provisioning_file) = datasource_provisioning_file {
        load_datasource_provisioning_records(provisioning_file)?
            .into_iter()
            .map(|item| normalize_datasource_bundle_item(&item))
            .collect::<Result<Vec<_>>>()?
    } else {
        let datasources_path = metadata_dir.join("datasources.json");
        if datasources_path.is_file() {
            load_json_array_file(&datasources_path, "Dashboard datasource inventory")?
                .into_iter()
                .map(|item| normalize_datasource_bundle_item(&item))
                .collect::<Result<Vec<_>>>()?
        } else {
            Vec::new()
        }
    };
    let mut metadata = Map::new();
    let export_metadata_path = metadata_dir.join("export-metadata.json");
    if export_metadata_path.is_file() {
        metadata.insert(
            "dashboardExport".to_string(),
            load_json_value(&export_metadata_path, "Dashboard export metadata")?,
        );
    }
    if let Some(provisioning_file) = datasource_provisioning_file {
        metadata.insert(
            "datasourceProvisioningFile".to_string(),
            Value::String(provisioning_file.display().to_string()),
        );
    }
    Ok((dashboards, datasources, folders, metadata))
}

pub(crate) fn load_dashboard_provisioning_bundle_sections(
    provisioning_dir: &Path,
    datasource_provisioning_file: Option<&Path>,
) -> Result<DashboardBundleSections> {
    let resolved = load_dashboard_source(
        provisioning_dir,
        DashboardImportInputFormat::Provisioning,
        None,
        false,
    )?;
    load_dashboard_bundle_sections(
        &resolved.resolved.dashboard_dir,
        &resolved.resolved.metadata_dir,
        datasource_provisioning_file,
    )
}
