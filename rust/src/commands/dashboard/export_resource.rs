//! Dashboard resource API export helpers.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use reqwest::Method;
use serde_json::Value;

use crate::common::{message, sanitize_path_component, string_field, Result};
use crate::grafana_api::DashboardResourceApiVersion;

use super::super::{
    build_export_metadata, DashboardResourceFormat, DashboardResourceIndexEntry, ExportMetadata,
    RESOURCE_V1_EXPORT_SUBDIR,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResourceExportLane {
    pub(crate) version: DashboardResourceApiVersion,
    pub(crate) dir_name: &'static str,
    pub(crate) format_name: &'static str,
    pub(crate) api_version: &'static str,
}

impl ResourceExportLane {
    pub(crate) fn for_version(version: DashboardResourceApiVersion) -> Result<Self> {
        match version {
            DashboardResourceApiVersion::V1 => Ok(Self {
                version,
                dir_name: RESOURCE_V1_EXPORT_SUBDIR,
                format_name: "grafana-dashboard-resource-v1",
                api_version: "dashboard.grafana.app/v1",
            }),
            DashboardResourceApiVersion::V2 => Err(message(
                "dashboard.grafana.app/v2 resource HTTP API is not verified yet.",
            )),
        }
    }
}

pub(crate) fn selected_resource_lanes(
    format: DashboardResourceFormat,
) -> Result<Vec<ResourceExportLane>> {
    match format {
        DashboardResourceFormat::None => Ok(Vec::new()),
        DashboardResourceFormat::V1 => Ok(vec![ResourceExportLane::for_version(
            DashboardResourceApiVersion::V1,
        )?]),
        DashboardResourceFormat::V2 => Ok(vec![ResourceExportLane::for_version(
            DashboardResourceApiVersion::V2,
        )?]),
        DashboardResourceFormat::All => Ok(vec![
            ResourceExportLane::for_version(DashboardResourceApiVersion::V1)?,
            ResourceExportLane::for_version(DashboardResourceApiVersion::V2)?,
        ]),
    }
}

pub(crate) fn dashboard_resource_namespace(org_id: &str) -> String {
    if org_id == "1" {
        "default".to_string()
    } else {
        format!("org-{org_id}")
    }
}

pub(crate) fn fetch_dashboard_resource_with_request<F>(
    request_json: &mut F,
    lane: ResourceExportLane,
    namespace: &str,
    name: &str,
) -> Result<Value>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let version = match lane.version {
        DashboardResourceApiVersion::V1 => "v1",
        DashboardResourceApiVersion::V2 => {
            return Err(message(
                "dashboard.grafana.app/v2 resource HTTP API is not verified yet.",
            ))
        }
    };
    let path =
        format!("/apis/dashboard.grafana.app/{version}/namespaces/{namespace}/dashboards/{name}");
    match request_json(Method::GET, &path, &[], None)? {
        Some(Value::Object(object)) => Ok(Value::Object(object)),
        Some(_) => Err(message(format!(
            "Unexpected dashboard resource payload for {namespace}/{name}."
        ))),
        None => Err(message(format!(
            "Unexpected empty dashboard resource payload for {namespace}/{name}."
        ))),
    }
}

pub(crate) fn build_resource_output_path(
    objects_dir: &Path,
    resource: &Value,
    fallback_name: &str,
    folder_paths_by_key: &BTreeMap<String, String>,
    org_id: &str,
) -> (PathBuf, String, String, String) {
    let metadata = resource.get("metadata").and_then(Value::as_object);
    let name = metadata
        .map(|object| string_field(object, "name", fallback_name))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback_name.to_string());
    let title = resource
        .get("spec")
        .and_then(Value::as_object)
        .map(|object| string_field(object, "title", &name))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| name.clone());
    let folder_uid = metadata
        .and_then(|object| object.get("annotations"))
        .and_then(Value::as_object)
        .and_then(|annotations| annotations.get("grafana.app/folder"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let folder_path = if folder_uid.is_empty() {
        String::new()
    } else {
        folder_paths_by_key
            .get(&format!("{org_id}:{folder_uid}"))
            .cloned()
            .unwrap_or_default()
    };
    let file_name = format!("{}.json", sanitize_path_component(&name));
    let path = if folder_path.trim().is_empty() {
        objects_dir.join(&file_name)
    } else {
        folder_path
            .split(" / ")
            .filter(|segment| !segment.trim().is_empty())
            .fold(objects_dir.to_path_buf(), |path, segment| {
                path.join(sanitize_path_component(segment.trim()))
            })
            .join(file_name)
    };
    (path, name, title, folder_path)
}

pub(crate) fn build_resource_index_entry(
    lane: ResourceExportLane,
    name: String,
    title: String,
    path: &Path,
    folder_path: String,
    org: &str,
    org_id: &str,
) -> DashboardResourceIndexEntry {
    DashboardResourceIndexEntry {
        name,
        title,
        path: path.display().to_string(),
        format: lane.format_name.to_string(),
        api_version: lane.api_version.to_string(),
        folder_path,
        org: org.to_string(),
        org_id: org_id.to_string(),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_resource_export_metadata(
    lane: ResourceExportLane,
    dashboard_count: usize,
    org_name: &str,
    org_id: &str,
    source_url: &str,
    source_profile: Option<&str>,
    artifact_path: &Path,
    metadata_path: &Path,
) -> ExportMetadata {
    let mut metadata = build_export_metadata(
        lane.dir_name,
        dashboard_count,
        Some(lane.format_name),
        None,
        None,
        None,
        Some(org_name),
        Some(org_id),
        None,
        "live",
        Some(source_url),
        None,
        source_profile,
        artifact_path,
        metadata_path,
    );
    metadata.resource_api_version = Some(lane.api_version.to_string());
    metadata.serialization = Some("json-pretty".to_string());
    metadata.ui_import_compatible = Some(false);
    metadata
}
