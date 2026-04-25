use serde_json::{Map, Value};

use crate::common::string_field;

use super::super::types::FolderPermissionEntry;

mod permission_row_keys {
    pub(super) const SUBJECT_TYPE: &str = "subjectType";
    pub(super) const SUBJECT_KEY: &str = "subjectKey";
    pub(super) const SUBJECT_ID: &str = "subjectId";
    pub(super) const SUBJECT_NAME: &str = "subjectName";
    pub(super) const PERMISSION: &str = "permission";
    pub(super) const PERMISSION_NAME: &str = "permissionName";
    pub(super) const INHERITED: &str = "inherited";
}

fn value_text(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(text)) => text.to_string(),
        Some(Value::Number(number)) => number.to_string(),
        Some(Value::Bool(flag)) => flag.to_string(),
        _ => String::new(),
    }
}

fn normalize_permission_level(record: &Map<String, Value>) -> (i64, String) {
    let permission = record
        .get("permission")
        .or_else(|| record.get("permissionCode"))
        .and_then(Value::as_i64)
        .unwrap_or_default();
    let permission_name = record
        .get("permissionName")
        .or_else(|| record.get("permission"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| permission.to_string());
    (permission, permission_name)
}

fn normalize_permission_subject(record: &Map<String, Value>) -> (String, String, String, String) {
    if let Some(role) = record.get("role").and_then(Value::as_str) {
        return (
            "role".to_string(),
            format!("role:{role}"),
            role.to_string(),
            role.to_string(),
        );
    }
    for (subject_type, id_key, name_key) in [
        ("user", "userId", "userLogin"),
        ("team", "teamId", "team"),
        ("service-account", "serviceAccountId", "serviceAccount"),
    ] {
        let id = value_text(record.get(id_key));
        if id.is_empty() {
            continue;
        }
        let name = record
            .get(name_key)
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| id.clone());
        return (
            subject_type.to_string(),
            format!("{subject_type}:{name}"),
            id,
            name,
        );
    }
    (
        "unknown".to_string(),
        "unknown".to_string(),
        String::new(),
        "unknown".to_string(),
    )
}

pub(super) fn permission_entry_from_row(row: &Map<String, Value>) -> FolderPermissionEntry {
    let subject_type = string_field(row, permission_row_keys::SUBJECT_TYPE, "");
    let subject_key = string_field(row, permission_row_keys::SUBJECT_KEY, "");
    let subject_id = string_field(row, permission_row_keys::SUBJECT_ID, "");
    let subject_name = string_field(row, permission_row_keys::SUBJECT_NAME, "");
    let permission = row
        .get(permission_row_keys::PERMISSION)
        .and_then(Value::as_i64)
        .unwrap_or_default();
    let permission_name = string_field(row, permission_row_keys::PERMISSION_NAME, "");
    let inherited = row
        .get(permission_row_keys::INHERITED)
        .and_then(Value::as_bool)
        .unwrap_or(false);
    FolderPermissionEntry {
        subject_type,
        subject_key,
        subject_id,
        subject_name,
        permission,
        permission_name,
        inherited,
    }
}

pub(super) fn permission_entry_from_live_row(row: &Map<String, Value>) -> FolderPermissionEntry {
    let (subject_type, subject_key, subject_id, subject_name) = normalize_permission_subject(row);
    let (permission, permission_name) = normalize_permission_level(row);
    let inherited = row
        .get("inherited")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    FolderPermissionEntry {
        subject_type,
        subject_key,
        subject_id,
        subject_name,
        permission,
        permission_name,
        inherited,
    }
}
