//! Shared Dashboard helpers for internal state transitions and reusable orchestration logic.

#[path = "export_support/permissions.rs"]
mod permissions;

use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use reqwest::Method;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::common::{message, sanitize_path_component, string_field, Result};

use super::super::export_prompt::{build_library_panel_export, collect_library_panel_uids};
use crate::dashboard::live::{
    fetch_dashboard_permissions_with_request, fetch_folder_permissions_with_request,
};
use crate::dashboard::{
    DatasourceInventoryItem, ExportDatasourceUsageSummary, FolderInventoryItem,
    DEFAULT_DASHBOARD_TITLE, DEFAULT_UNKNOWN_UID,
};

const PERMISSION_FETCH_CONCURRENCY: usize = 8;

pub(crate) use permissions::{build_permission_bundle_document, PermissionExportTarget};

use permissions::{build_permission_export_document, value_text};

pub(crate) fn build_all_orgs_output_dir(output_dir: &Path, org: &Map<String, Value>) -> PathBuf {
    let org_id = org
        .get("id")
        .map(|value| sanitize_path_component(&value.to_string()))
        .unwrap_or_else(|| DEFAULT_UNKNOWN_UID.to_string());
    let org_name = sanitize_path_component(&string_field(org, "name", "org"));
    output_dir.join(format!("org_{org_id}_{org_name}"))
}

pub(crate) fn build_used_datasource_summaries(
    datasource_inventory: &[DatasourceInventoryItem],
    used_names: &BTreeSet<String>,
    used_uids: &BTreeSet<String>,
) -> Vec<ExportDatasourceUsageSummary> {
    let mut used = Vec::new();
    let mut matched_names = BTreeSet::new();
    let mut matched_uids = BTreeSet::new();

    for datasource in datasource_inventory {
        if used_uids.contains(&datasource.uid) || used_names.contains(&datasource.name) {
            used.push(ExportDatasourceUsageSummary {
                name: datasource.name.clone(),
                uid: if datasource.uid.is_empty() {
                    None
                } else {
                    Some(datasource.uid.clone())
                },
                datasource_type: if datasource.datasource_type.is_empty() {
                    None
                } else {
                    Some(datasource.datasource_type.clone())
                },
            });
            if !datasource.uid.is_empty() {
                matched_uids.insert(datasource.uid.clone());
            }
            if !datasource.name.is_empty() {
                matched_names.insert(datasource.name.clone());
            }
        }
    }

    for name in used_names {
        if !matched_names.contains(name) {
            used.push(ExportDatasourceUsageSummary {
                name: name.clone(),
                uid: None,
                datasource_type: None,
            });
        }
    }
    for uid in used_uids {
        if !matched_uids.contains(uid) {
            used.push(ExportDatasourceUsageSummary {
                name: String::new(),
                uid: Some(uid.clone()),
                datasource_type: None,
            });
        }
    }

    used
}

pub(crate) fn collect_library_panel_exports_with_request<F>(
    mut request_json: F,
    dashboard_payload: &Value,
) -> Result<(BTreeMap<String, Value>, Vec<String>)>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut exports = BTreeMap::new();
    let mut warnings = Vec::new();

    for uid in collect_library_panel_uids(dashboard_payload) {
        match request_json(
            Method::GET,
            &format!("/api/library-elements/{uid}"),
            &[],
            None,
        ) {
            Ok(Some(value)) => match build_library_panel_export(&value) {
                Ok((export_uid, export)) => {
                    exports.insert(export_uid, export);
                }
                Err(error) => warnings.push(format!(
                    "Failed to normalize library panel {uid} for export: {error}"
                )),
            },
            Ok(None) => warnings.push(format!(
                "Library panel {uid} did not return a response from Grafana."
            )),
            Err(error) => warnings.push(format!(
                "Failed to fetch library panel {uid} for export: {error}"
            )),
        }
    }

    Ok((exports, warnings))
}

pub(crate) fn collect_permission_export_documents<F>(
    request_json: &mut F,
    summaries: &[Map<String, Value>],
    folder_inventory: &[FolderInventoryItem],
) -> Result<Vec<Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let targets = build_permission_export_targets(summaries, folder_inventory);
    let mut documents = Vec::new();
    for target in targets {
        let permissions = match target.resource_kind {
            "folder" => {
                fetch_folder_permissions_with_request(&mut *request_json, &target.resource_uid)?
            }
            _ => {
                fetch_dashboard_permissions_with_request(&mut *request_json, &target.resource_uid)?
            }
        };
        documents.push(build_permission_export_document_for_target(
            &target,
            &permissions,
        ));
    }
    Ok(documents)
}

pub(crate) fn collect_permission_export_documents_with_fetcher<F>(
    summaries: &[Map<String, Value>],
    folder_inventory: &[FolderInventoryItem],
    fetch_permissions: &F,
) -> Result<Vec<Value>>
where
    F: Fn(&PermissionExportTarget) -> Result<Vec<Map<String, Value>>> + Sync,
{
    let targets = build_permission_export_targets(summaries, folder_inventory);
    if targets.is_empty() {
        return Ok(Vec::new());
    }

    let pool = ThreadPoolBuilder::new()
        .num_threads(PERMISSION_FETCH_CONCURRENCY)
        .build()
        .map_err(|error| {
            message(format!(
                "Failed to build dashboard permission read worker pool: {error}"
            ))
        })?;
    let reads = pool.install(|| {
        targets
            .par_iter()
            .map(|target| (target.clone(), fetch_permissions(target)))
            .collect::<Vec<_>>()
    });

    let failures = reads
        .iter()
        .filter_map(|(target, result)| {
            result.as_ref().err().map(|error| {
                format!(
                    "kind={} uid={}: {error}",
                    target.resource_kind, target.resource_uid
                )
            })
        })
        .collect::<Vec<_>>();
    if !failures.is_empty() {
        return Err(message(format!(
            "Failed to fetch {} dashboard/folder permission resource(s): {}",
            failures.len(),
            failures.join("; ")
        )));
    }

    let mut documents = Vec::new();
    for (target, result) in reads {
        let permissions = result?;
        documents.push(build_permission_export_document_for_target(
            &target,
            &permissions,
        ));
    }
    Ok(documents)
}

fn build_permission_export_targets(
    summaries: &[Map<String, Value>],
    folder_inventory: &[FolderInventoryItem],
) -> Vec<PermissionExportTarget> {
    let mut targets = Vec::new();
    let mut seen_folders = BTreeSet::new();
    for folder in folder_inventory {
        if folder.uid.trim().is_empty() || !seen_folders.insert(folder.uid.clone()) {
            continue;
        }
        targets.push(PermissionExportTarget {
            resource_kind: "folder",
            resource_uid: folder.uid.clone(),
            resource_title: folder.title.clone(),
            org_name: folder.org.clone(),
            org_id: folder.org_id.clone(),
        });
    }
    for summary in summaries {
        let uid = string_field(summary, "uid", "");
        if uid.is_empty() {
            continue;
        }
        let raw_org_id = value_text(summary.get("orgId"));
        targets.push(PermissionExportTarget {
            resource_kind: "dashboard",
            resource_uid: uid,
            resource_title: string_field(summary, "title", DEFAULT_DASHBOARD_TITLE),
            org_name: string_field(summary, "orgName", "org"),
            org_id: if raw_org_id.is_empty() {
                DEFAULT_UNKNOWN_UID.to_string()
            } else {
                raw_org_id
            },
        });
    }
    targets
}

fn build_permission_export_document_for_target(
    target: &PermissionExportTarget,
    permissions: &[Map<String, Value>],
) -> Value {
    build_permission_export_document(
        target.resource_kind,
        &target.resource_uid,
        &target.resource_title,
        permissions,
        &target.org_name,
        &target.org_id,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn folder(uid: &str, title: &str) -> FolderInventoryItem {
        FolderInventoryItem {
            uid: uid.to_string(),
            title: title.to_string(),
            path: title.to_string(),
            parent_uid: None,
            org: "Main Org".to_string(),
            org_id: "1".to_string(),
        }
    }

    fn dashboard_summary(uid: &str, title: &str) -> Map<String, Value> {
        let mut summary = Map::new();
        summary.insert("uid".to_string(), Value::String(uid.to_string()));
        summary.insert("title".to_string(), Value::String(title.to_string()));
        summary.insert("orgName".to_string(), Value::String("Main Org".to_string()));
        summary.insert("orgId".to_string(), Value::from(1));
        summary
    }

    fn permission_row(role: &str) -> Map<String, Value> {
        let mut row = Map::new();
        row.insert("role".to_string(), Value::String(role.to_string()));
        row.insert("permission".to_string(), Value::from(1));
        row
    }

    #[test]
    fn collect_library_panel_exports_with_request_records_failures_as_warnings() {
        let payload = json!({
            "panels": [{
                "libraryPanel": {"uid": "shared-panel", "name": "Shared Panel"}
            }]
        });

        let (exports, warnings) = collect_library_panel_exports_with_request(
            |_method, _path, _params, _payload| Err(crate::common::message("boom")),
            &payload,
        )
        .unwrap();

        assert!(exports.is_empty());
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("shared-panel"));
    }

    #[test]
    fn collect_permission_export_documents_with_fetcher_preserves_target_order() {
        let summaries = vec![
            dashboard_summary("dash-b", "Dashboard B"),
            dashboard_summary("dash-a", "Dashboard A"),
        ];
        let folders = vec![
            folder("ops", "Operations"),
            folder("ops", "Duplicate Operations"),
            folder("infra", "Infrastructure"),
        ];

        let documents =
            collect_permission_export_documents_with_fetcher(&summaries, &folders, &|target| {
                Ok(vec![permission_row(target.resource_uid.as_str())])
            })
            .unwrap();

        let resources = documents
            .iter()
            .map(|document| {
                let resource = document["resource"].as_object().unwrap();
                (
                    resource["kind"].as_str().unwrap(),
                    resource["uid"].as_str().unwrap(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            resources,
            vec![
                ("folder", "ops"),
                ("folder", "infra"),
                ("dashboard", "dash-b"),
                ("dashboard", "dash-a")
            ]
        );
    }

    #[test]
    fn collect_permission_export_documents_with_fetcher_aggregates_failures() {
        let summaries = vec![
            dashboard_summary("dash-a", "Dashboard A"),
            dashboard_summary("dash-b", "Dashboard B"),
        ];
        let folders = vec![folder("ops", "Operations")];

        let error =
            collect_permission_export_documents_with_fetcher(&summaries, &folders, &|target| {
                match target.resource_uid.as_str() {
                    "ops" | "dash-b" => Err(message(format!("boom {}", target.resource_uid))),
                    _ => Ok(vec![permission_row(target.resource_uid.as_str())]),
                }
            })
            .unwrap_err()
            .to_string();

        assert!(error.contains("Failed to fetch 2 dashboard/folder permission resource(s)"));
        assert!(error.contains("kind=folder uid=ops: boom ops"));
        assert!(error.contains("kind=dashboard uid=dash-b: boom dash-b"));
    }
}
