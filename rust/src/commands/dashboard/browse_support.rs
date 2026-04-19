#![cfg(feature = "tui")]
use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::common::{message, string_field, Result};

use super::delete_support::normalize_folder_path;
use super::{DEFAULT_DASHBOARD_TITLE, DEFAULT_FOLDER_TITLE};

pub(crate) use super::browse_load::load_dashboard_browse_document_for_args;
pub(crate) use super::browse_live_detail::fetch_dashboard_view_lines_with_request;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DashboardBrowseSummary {
    pub root_path: Option<String>,
    pub dashboard_count: usize,
    pub folder_count: usize,
    pub org_count: usize,
    pub scope_label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DashboardBrowseNodeKind {
    Org,
    Folder,
    Dashboard,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DashboardBrowseNode {
    pub kind: DashboardBrowseNodeKind,
    pub title: String,
    pub path: String,
    pub uid: Option<String>,
    pub depth: usize,
    pub meta: String,
    pub details: Vec<String>,
    pub url: Option<String>,
    pub org_name: String,
    pub org_id: String,
    pub child_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DashboardBrowseDocument {
    pub summary: DashboardBrowseSummary,
    pub nodes: Vec<DashboardBrowseNode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FolderNodeRecord {
    title: String,
    path: String,
    uid: Option<String>,
    parent_path: Option<String>,
}

#[allow(dead_code)]
pub(crate) fn build_dashboard_browse_document(
    summaries: &[Map<String, Value>],
    folder_inventory: &[super::FolderInventoryItem],
    root_path: Option<&str>,
) -> Result<DashboardBrowseDocument> {
    build_dashboard_browse_document_for_org(
        summaries,
        folder_inventory,
        root_path,
        super::DEFAULT_ORG_NAME,
        super::DEFAULT_ORG_ID,
        false,
        false,
    )
}

pub(super) fn build_dashboard_browse_document_for_org(
    summaries: &[Map<String, Value>],
    folder_inventory: &[super::FolderInventoryItem],
    root_path: Option<&str>,
    org_name: &str,
    org_id: &str,
    local_mode: bool,
    allow_empty_root: bool,
) -> Result<DashboardBrowseDocument> {
    let normalized_root = root_path
        .map(normalize_folder_path)
        .filter(|value| !value.is_empty());
    let filtered_summaries = summaries
        .iter()
        .filter(|summary| {
            let folder_path = normalize_folder_path(&string_field(
                summary,
                "folderPath",
                &string_field(summary, "folderTitle", DEFAULT_FOLDER_TITLE),
            ));
            matches_root_path(&folder_path, normalized_root.as_deref())
        })
        .cloned()
        .collect::<Vec<_>>();

    if !allow_empty_root {
        if let Some(root) = normalized_root.as_deref() {
            let has_folder = folder_inventory
                .iter()
                .any(|folder| matches_root_path(&normalize_folder_path(&folder.path), Some(root)));
            let has_dashboard = filtered_summaries.iter().any(|summary| {
                let folder_path = normalize_folder_path(&string_field(
                    summary,
                    "folderPath",
                    &string_field(summary, "folderTitle", DEFAULT_FOLDER_TITLE),
                ));
                matches_root_path(&folder_path, Some(root))
            });
            if !has_folder && !has_dashboard {
                return Err(message(format!(
                    "Dashboard browser folder path did not match any dashboards: {root}"
                )));
            }
        }
    }

    let mut folders = BTreeMap::<String, FolderNodeRecord>::new();
    for folder in folder_inventory {
        ensure_folder_path(
            &mut folders,
            &normalize_folder_path(&folder.path),
            Some(folder.uid.clone()),
        );
    }
    for summary in &filtered_summaries {
        let folder_path = normalize_folder_path(&string_field(
            summary,
            "folderPath",
            &string_field(summary, "folderTitle", DEFAULT_FOLDER_TITLE),
        ));
        let folder_uid = string_field(summary, "folderUid", "");
        ensure_folder_path(
            &mut folders,
            &folder_path,
            (!folder_uid.is_empty()).then_some(folder_uid),
        );
    }

    let folder_keys = folders.keys().cloned().collect::<Vec<_>>();
    let mut folder_dashboard_counts = BTreeMap::<String, usize>::new();
    for folder_path in &folder_keys {
        folder_dashboard_counts.insert(folder_path.clone(), 0);
    }
    for summary in &filtered_summaries {
        let folder_path = normalize_folder_path(&string_field(
            summary,
            "folderPath",
            &string_field(summary, "folderTitle", DEFAULT_FOLDER_TITLE),
        ));
        for ancestor in folder_ancestors(&folder_path) {
            if let Some(count) = folder_dashboard_counts.get_mut(&ancestor) {
                *count += 1;
            }
        }
    }

    let mut folder_child_counts = BTreeMap::<String, usize>::new();
    for record in folders.values() {
        if let Some(parent_path) = record.parent_path.as_ref() {
            *folder_child_counts.entry(parent_path.clone()).or_insert(0) += 1;
        }
    }

    let mut nodes = Vec::new();
    for folder_path in &folder_keys {
        if !matches_root_path(folder_path, normalized_root.as_deref()) {
            continue;
        }
        let Some(record) = folders.get(folder_path) else {
            continue;
        };
        nodes.push(DashboardBrowseNode {
            kind: DashboardBrowseNodeKind::Folder,
            title: record.title.clone(),
            path: record.path.clone(),
            uid: record.uid.clone(),
            depth: folder_depth(folder_path, normalized_root.as_deref()),
            meta: format!(
                "{} folder(s) | {} dashboard(s)",
                folder_child_counts.get(folder_path).copied().unwrap_or(0),
                folder_dashboard_counts
                    .get(folder_path)
                    .copied()
                    .unwrap_or(0)
            ),
            details: vec![
                "Type: Folder".to_string(),
                format!("Org: {}", org_name),
                format!("Org ID: {}", org_id),
                format!("Title: {}", record.title),
                format!("Path: {}", record.path),
                format!("UID: {}", record.uid.as_deref().unwrap_or("-")),
                format!(
                    "Parent path: {}",
                    record.parent_path.as_deref().unwrap_or("-")
                ),
                format!(
                    "Child folders: {}",
                    folder_child_counts.get(folder_path).copied().unwrap_or(0)
                ),
                format!(
                    "Dashboards in subtree: {}",
                    folder_dashboard_counts
                        .get(folder_path)
                        .copied()
                        .unwrap_or(0)
                ),
                if local_mode {
                    "Local browse: read-only file tree.".to_string()
                } else {
                    "Delete: press d to remove dashboards in this subtree.".to_string()
                },
                if local_mode {
                    "Local browse: delete actions are unavailable.".to_string()
                } else {
                    "Delete folders: press D to remove dashboards and folders in this subtree."
                        .to_string()
                },
            ],
            url: None,
            org_name: org_name.to_string(),
            org_id: org_id.to_string(),
            child_count: folder_child_counts.get(folder_path).copied().unwrap_or(0),
        });

        let mut dashboards = filtered_summaries
            .iter()
            .filter(|summary| {
                normalize_folder_path(&string_field(
                    summary,
                    "folderPath",
                    &string_field(summary, "folderTitle", DEFAULT_FOLDER_TITLE),
                )) == *folder_path
            })
            .collect::<Vec<_>>();
        dashboards.sort_by(|left, right| {
            string_field(left, "title", DEFAULT_DASHBOARD_TITLE)
                .cmp(&string_field(right, "title", DEFAULT_DASHBOARD_TITLE))
                .then_with(|| string_field(left, "uid", "").cmp(&string_field(right, "uid", "")))
        });
        for summary in dashboards {
            let title = string_field(summary, "title", DEFAULT_DASHBOARD_TITLE);
            let uid = string_field(summary, "uid", "");
            let url = string_field(summary, "url", "");
            nodes.push(DashboardBrowseNode {
                kind: DashboardBrowseNodeKind::Dashboard,
                title: title.clone(),
                path: folder_path.clone(),
                uid: Some(uid.clone()),
                depth: folder_depth(folder_path, normalized_root.as_deref()) + 1,
                meta: format!("uid={uid}"),
                details: vec![
                    "Type: Dashboard".to_string(),
                    format!("Org: {}", org_name),
                    format!("Org ID: {}", org_id),
                    format!("Title: {title}"),
                    format!("UID: {uid}"),
                    format!("Folder path: {folder_path}"),
                    format!("Folder UID: {}", {
                        let value = string_field(summary, "folderUid", "");
                        if value.is_empty() {
                            "-".to_string()
                        } else {
                            value
                        }
                    }),
                    format!(
                        "URL: {}",
                        if url.is_empty() {
                            "-".to_string()
                        } else {
                            url.clone()
                        }
                    ),
                    if local_mode {
                        "Local browse: live details are unavailable.".to_string()
                    } else {
                        "View: press v to load live dashboard details.".to_string()
                    },
                    if local_mode {
                        format!("Source file: {}", string_field(summary, "sourceFile", "-"))
                    } else {
                        "Advanced edit: press E to open raw dashboard JSON, then review/apply/save it back in the TUI."
                            .to_string()
                    },
                    if local_mode {
                        "Local browse: delete actions are unavailable.".to_string()
                    } else {
                        "Delete: press d to delete this dashboard.".to_string()
                    },
                ],
                url: (!url.is_empty()).then_some(url),
                org_name: org_name.to_string(),
                org_id: org_id.to_string(),
                child_count: 0,
            });
        }
    }

    Ok(DashboardBrowseDocument {
        summary: DashboardBrowseSummary {
            root_path: normalized_root,
            dashboard_count: filtered_summaries.len(),
            folder_count: nodes
                .iter()
                .filter(|node| node.kind == DashboardBrowseNodeKind::Folder)
                .count(),
            org_count: 1,
            scope_label: format!("Org {} ({})", org_name, org_id),
        },
        nodes,
    })
}

pub(super) fn build_org_node(
    org_name: &str,
    org_id: &str,
    folder_count: usize,
    dashboard_count: usize,
) -> DashboardBrowseNode {
    DashboardBrowseNode {
        kind: DashboardBrowseNodeKind::Org,
        title: org_name.to_string(),
        path: org_name.to_string(),
        uid: None,
        depth: 0,
        meta: format!("{folder_count} folder(s) | {dashboard_count} dashboard(s)"),
        details: vec![
            "Type: Org".to_string(),
            format!("Org: {org_name}"),
            format!("Org ID: {org_id}"),
            format!("Folder count: {folder_count}"),
            format!("Dashboard count: {dashboard_count}"),
            "Browse: select folder or dashboard rows below this org.".to_string(),
        ],
        url: None,
        org_name: org_name.to_string(),
        org_id: org_id.to_string(),
        child_count: folder_count,
    }
}

fn ensure_folder_path(
    folders: &mut BTreeMap<String, FolderNodeRecord>,
    folder_path: &str,
    uid: Option<String>,
) {
    let normalized = normalize_folder_path(folder_path);
    if normalized.is_empty() {
        return;
    }
    let ancestors = folder_ancestors(&normalized);
    for path in ancestors {
        let title = path
            .split(" / ")
            .last()
            .unwrap_or(DEFAULT_FOLDER_TITLE)
            .to_string();
        let parent_path = parent_folder_path(&path);
        let record = folders
            .entry(path.clone())
            .or_insert_with(|| FolderNodeRecord {
                title: title.clone(),
                path: path.clone(),
                uid: None,
                parent_path: parent_path.clone(),
            });
        if path == normalized {
            if let Some(folder_uid) = uid.as_ref().filter(|value| !value.is_empty()) {
                record.uid = Some(folder_uid.clone());
            }
            record.title = title;
            record.parent_path = parent_path;
        }
    }
}

fn folder_ancestors(path: &str) -> Vec<String> {
    let mut ancestors = Vec::new();
    let mut parts = Vec::new();
    for part in path.split(" / ") {
        parts.push(part);
        ancestors.push(parts.join(" / "));
    }
    ancestors
}

fn parent_folder_path(path: &str) -> Option<String> {
    let mut parts = path.split(" / ").collect::<Vec<_>>();
    if parts.len() <= 1 {
        None
    } else {
        parts.pop();
        Some(parts.join(" / "))
    }
}

fn matches_root_path(path: &str, root_path: Option<&str>) -> bool {
    match root_path {
        Some(root) => path == root || path.starts_with(&format!("{root} / ")),
        None => true,
    }
}

fn folder_depth(path: &str, root_path: Option<&str>) -> usize {
    let depth = path.split(" / ").count().saturating_sub(1);
    match root_path {
        Some(root) => depth.saturating_sub(root.split(" / ").count().saturating_sub(1)),
        None => depth,
    }
}

#[cfg(test)]
#[cfg(test)]
#[path = "browse_support_rust_tests.rs"]
mod tests;
