use serde_json::{Map, Value};

use crate::common::{string_field, Result};
use crate::grafana_api::DashboardResourceClient;

use super::super::{FolderInventoryItem, DEFAULT_FOLDER_TITLE, DEFAULT_ORG_ID, DEFAULT_ORG_NAME};

pub(super) fn collect_folder_inventory_from_summaries(
    dashboard: &DashboardResourceClient<'_>,
    summaries: &[Map<String, Value>],
) -> Result<Vec<FolderInventoryItem>> {
    let mut seen = std::collections::BTreeSet::new();
    let mut folders = Vec::new();
    for summary in summaries {
        let folder_uid = string_field(summary, "folderUid", "");
        if folder_uid.is_empty() {
            continue;
        }
        let org_id = summary
            .get("orgId")
            .map(|value| match value {
                Value::String(text) => text.clone(),
                _ => value.to_string(),
            })
            .unwrap_or_else(|| DEFAULT_ORG_ID.to_string());
        let key = format!("{org_id}:{folder_uid}");
        if seen.contains(&key) {
            continue;
        }
        let Some(folder) = dashboard.fetch_folder_if_exists(&folder_uid)? else {
            continue;
        };
        let org = string_field(summary, "orgName", DEFAULT_ORG_NAME);
        let mut parent_path = Vec::new();
        let mut previous_parent_uid = None;
        if let Some(parents) = folder.get("parents").and_then(Value::as_array) {
            for parent in parents {
                let Some(parent_object) = parent.as_object() else {
                    continue;
                };
                let parent_uid = string_field(parent_object, "uid", "");
                let parent_title = string_field(parent_object, "title", "");
                if parent_uid.is_empty() || parent_title.is_empty() {
                    continue;
                }
                parent_path.push(parent_title.clone());
                let parent_key = format!("{org_id}:{parent_uid}");
                if !seen.contains(&parent_key) {
                    folders.push(FolderInventoryItem {
                        uid: parent_uid.clone(),
                        title: parent_title,
                        path: parent_path.join(" / "),
                        parent_uid: previous_parent_uid.clone(),
                        org: org.clone(),
                        org_id: org_id.clone(),
                    });
                    seen.insert(parent_key);
                }
                previous_parent_uid = Some(parent_uid);
            }
        }
        let folder_title = string_field(&folder, "title", DEFAULT_FOLDER_TITLE);
        parent_path.push(folder_title.clone());
        folders.push(FolderInventoryItem {
            uid: folder_uid.clone(),
            title: folder_title,
            path: parent_path.join(" / "),
            parent_uid: previous_parent_uid,
            org,
            org_id: org_id.clone(),
        });
        seen.insert(key);
    }
    folders.sort_by(|left, right| {
        left.org_id
            .cmp(&right.org_id)
            .then(left.path.cmp(&right.path))
            .then(left.uid.cmp(&right.uid))
    });
    Ok(folders)
}
