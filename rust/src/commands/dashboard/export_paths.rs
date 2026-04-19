use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::common::{sanitize_path_component, string_field};

use super::super::{
    FolderInventoryItem, DEFAULT_DASHBOARD_TITLE, DEFAULT_FOLDER_TITLE, DEFAULT_ORG_ID,
    DEFAULT_UNKNOWN_UID, PROMPT_EXPORT_SUBDIR, PROVISIONING_EXPORT_SUBDIR, RAW_EXPORT_SUBDIR,
};

pub fn build_output_path(output_dir: &Path, summary: &Map<String, Value>, flat: bool) -> PathBuf {
    let folder_title = string_field(summary, "folderTitle", DEFAULT_FOLDER_TITLE);
    let title = string_field(summary, "title", DEFAULT_DASHBOARD_TITLE);
    let uid = string_field(summary, "uid", DEFAULT_UNKNOWN_UID);
    let file_name = format!(
        "{}__{}.json",
        sanitize_path_component(&title),
        sanitize_path_component(&uid)
    );

    if flat {
        output_dir.join(file_name)
    } else {
        output_dir
            .join(sanitize_path_component(&folder_title))
            .join(file_name)
    }
}

pub(crate) fn build_output_path_with_folder_path(
    output_dir: &Path,
    summary: &Map<String, Value>,
    folder_path: Option<&str>,
    flat: bool,
) -> PathBuf {
    if flat {
        return build_output_path(output_dir, summary, true);
    }

    let title = string_field(summary, "title", DEFAULT_DASHBOARD_TITLE);
    let uid = string_field(summary, "uid", DEFAULT_UNKNOWN_UID);
    let file_name = format!(
        "{}__{}.json",
        sanitize_path_component(&title),
        sanitize_path_component(&uid)
    );
    let Some(folder_path) = folder_path.filter(|path| !path.trim().is_empty()) else {
        return build_output_path(output_dir, summary, false);
    };

    folder_path
        .split(" / ")
        .filter(|segment| !segment.trim().is_empty())
        .fold(output_dir.to_path_buf(), |path, segment| {
            path.join(sanitize_path_component(segment.trim()))
        })
        .join(file_name)
}

pub(crate) fn export_folder_inventory_key(summary: &Map<String, Value>) -> Option<String> {
    let folder_uid = string_field(summary, "folderUid", "");
    if folder_uid.is_empty() {
        return None;
    }
    let org_id = summary
        .get("orgId")
        .map(|value| match value {
            Value::String(text) => text.clone(),
            _ => value.to_string(),
        })
        .unwrap_or_else(|| DEFAULT_ORG_ID.to_string());
    Some(format!("{org_id}:{folder_uid}"))
}

pub(crate) fn build_folder_paths_by_inventory_key(
    folder_inventory: &[FolderInventoryItem],
) -> BTreeMap<String, String> {
    folder_inventory
        .iter()
        .map(|folder| {
            (
                format!("{}:{}", folder.org_id, folder.uid),
                folder.path.clone(),
            )
        })
        .collect()
}

pub fn build_export_variant_dirs(output_dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
    (
        output_dir.join(RAW_EXPORT_SUBDIR),
        output_dir.join(PROMPT_EXPORT_SUBDIR),
        output_dir.join(PROVISIONING_EXPORT_SUBDIR),
    )
}

pub(crate) fn build_history_output_path(history_dir: &Path, uid: &str) -> PathBuf {
    history_dir.join(format!("{}.history.json", sanitize_path_component(uid)))
}
