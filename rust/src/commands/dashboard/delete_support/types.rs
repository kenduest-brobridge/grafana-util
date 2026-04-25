//! Stable dashboard delete plan models and pure target/path helpers.

use serde::Serialize;
use serde_json::{Map, Value};

use crate::common::string_field;

use super::super::{DEFAULT_DASHBOARD_TITLE, DEFAULT_FOLDER_TITLE};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct DashboardDeleteTarget {
    pub uid: String,
    pub title: String,
    pub folder_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct FolderDeleteTarget {
    pub uid: String,
    pub title: String,
    pub path: String,
    pub parent_uid: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct DeletePlan {
    pub selector_uid: Option<String>,
    pub selector_path: Option<String>,
    pub delete_folders: bool,
    pub dashboards: Vec<DashboardDeleteTarget>,
    pub folders: Vec<FolderDeleteTarget>,
}

pub(crate) fn normalize_folder_path(path: &str) -> String {
    let normalized = path.trim().replace('\\', "/");
    let parts: Vec<&str> = normalized
        .split('/')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    if parts.is_empty() {
        normalized.trim().to_string()
    } else {
        parts.join(" / ")
    }
}

pub(crate) fn build_dashboard_target(summary: &Map<String, Value>) -> DashboardDeleteTarget {
    DashboardDeleteTarget {
        uid: string_field(summary, "uid", "").to_string(),
        title: string_field(summary, "title", DEFAULT_DASHBOARD_TITLE).to_string(),
        folder_path: string_field(summary, "folderPath", DEFAULT_FOLDER_TITLE).to_string(),
    }
}

pub(crate) fn folder_path_matches(candidate: &str, root: &str) -> bool {
    candidate == root || candidate.starts_with(&format!("{root} / "))
}
