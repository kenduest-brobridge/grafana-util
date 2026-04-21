//! Folder permission drift review for dashboard plans.

use reqwest::Method;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::common::{message, string_field, value_as_object, Result};
use crate::review_contract::{
    REVIEW_ACTION_EXTRA_REMOTE, REVIEW_ACTION_SAME, REVIEW_ACTION_WOULD_CREATE,
    REVIEW_ACTION_WOULD_UPDATE, REVIEW_REASON_AMBIGUOUS_LIVE_NAME_MATCH, REVIEW_STATUS_BLOCKED,
    REVIEW_STATUS_READY, REVIEW_STATUS_SAME, REVIEW_STATUS_WARNING,
};

use super::super::{
    fetch_folder_if_exists_with_request, fetch_folder_permissions_with_request,
    FolderInventoryItem, DASHBOARD_PERMISSION_BUNDLE_FILENAME,
};
use super::types::{
    DashboardPlanAction, DashboardPlanChange, FolderPermissionEntry, FolderPermissionPlanDetails,
    FolderPermissionResource, OrgPlanInput,
};

const DOMAIN_FOLDER_PERMISSION: &str = "folder-permission";
const RESOURCE_KIND_FOLDER_PERMISSION: &str = "folder-permission";
const MATCH_UID: &str = "uid";
const MATCH_UID_THEN_PATH: &str = "uid-then-path";
const STATUS_MISSING_LIVE_FOLDER: &str = "missing-live-folder";
const STATUS_AMBIGUOUS_LIVE_FOLDER_PATH: &str = "ambiguous-live-folder-path";

mod permission_bundle_keys {
    pub(super) const RESOURCES: &str = "resources";
    pub(super) const RESOURCE: &str = "resource";
    pub(super) const RESOURCE_KIND: &str = "kind";
    pub(super) const RESOURCE_UID: &str = "uid";
    pub(super) const RESOURCE_TITLE: &str = "title";
    pub(super) const PERMISSIONS: &str = "permissions";
    pub(super) const ORG: &str = "org";
    pub(super) const ORG_ID: &str = "orgId";
}

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

fn permission_entry_from_row(row: &Map<String, Value>) -> FolderPermissionEntry {
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

fn permission_entry_from_live_row(row: &Map<String, Value>) -> FolderPermissionEntry {
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

pub(super) fn load_folder_permission_resources(
    input_dir: &Path,
    permissions_file: Option<&str>,
    folder_inventory: &[FolderInventoryItem],
) -> Result<Vec<FolderPermissionResource>> {
    let permission_path =
        input_dir.join(permissions_file.unwrap_or(DASHBOARD_PERMISSION_BUNDLE_FILENAME));
    if !permission_path.is_file() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&permission_path)?;
    let document: Value = serde_json::from_str(&raw)?;
    let object = value_as_object(
        &document,
        "Dashboard permissions bundle must be a JSON object.",
    )?;
    let resources = object
        .get(permission_bundle_keys::RESOURCES)
        .and_then(Value::as_array)
        .ok_or_else(|| message("Dashboard permissions bundle must contain resources[]."))?;
    let folder_paths = folder_inventory
        .iter()
        .map(|folder| (folder.uid.as_str(), folder.path.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut parsed = Vec::new();
    for resource in resources {
        let resource_object = value_as_object(
            resource,
            "Permission resource document must be a JSON object.",
        )?;
        let Some(resource_info) = resource_object
            .get(permission_bundle_keys::RESOURCE)
            .and_then(Value::as_object)
        else {
            continue;
        };
        if resource_info
            .get(permission_bundle_keys::RESOURCE_KIND)
            .and_then(Value::as_str)
            != Some("folder")
        {
            continue;
        }
        let uid = string_field(resource_info, permission_bundle_keys::RESOURCE_UID, "");
        if uid.is_empty() {
            continue;
        }
        let permissions = resource_object
            .get(permission_bundle_keys::PERMISSIONS)
            .and_then(Value::as_array)
            .map(|rows| {
                rows.iter()
                    .filter_map(Value::as_object)
                    .map(permission_entry_from_row)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        parsed.push(FolderPermissionResource {
            path: folder_paths
                .get(uid.as_str())
                .copied()
                .unwrap_or("")
                .to_string(),
            uid,
            title: string_field(resource_info, permission_bundle_keys::RESOURCE_TITLE, ""),
            org: string_field(resource_object, permission_bundle_keys::ORG, ""),
            org_id: string_field(resource_object, permission_bundle_keys::ORG_ID, ""),
            permissions,
        });
    }
    Ok(parsed)
}

fn live_resource_from_rows(
    folder: &FolderPermissionResource,
    target_uid: &str,
    target_path: &str,
    rows: Vec<Map<String, Value>>,
) -> FolderPermissionResource {
    FolderPermissionResource {
        uid: target_uid.to_string(),
        title: folder.title.clone(),
        path: target_path.to_string(),
        org: folder.org.clone(),
        org_id: folder.org_id.clone(),
        permissions: rows.iter().map(permission_entry_from_live_row).collect(),
    }
}

fn live_folders_by_path(
    live_folders: &[FolderInventoryItem],
) -> BTreeMap<String, Vec<&FolderInventoryItem>> {
    let mut by_path: BTreeMap<String, Vec<&FolderInventoryItem>> = BTreeMap::new();
    for folder in live_folders {
        by_path.entry(folder.path.clone()).or_default().push(folder);
    }
    by_path
}

pub(super) fn collect_live_folder_permission_resources<F>(
    request_json: &mut F,
    source_folders: &[FolderPermissionResource],
    live_folders: &[FolderInventoryItem],
    match_mode: &str,
) -> Result<Vec<FolderPermissionResource>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if source_folders.is_empty() {
        return Ok(Vec::new());
    }
    let live_by_path = live_folders_by_path(live_folders);
    let mut seen = BTreeSet::new();
    let mut live_resources = Vec::new();
    for source in source_folders {
        let mut target_uid = source.uid.clone();
        let mut target_path = source.path.clone();
        if fetch_folder_if_exists_with_request(&mut *request_json, &target_uid)?.is_none()
            && match_mode == MATCH_UID_THEN_PATH
            && !source.path.is_empty()
        {
            if let Some(matches) = live_by_path.get(&source.path) {
                if matches.len() == 1 {
                    target_uid = matches[0].uid.clone();
                    target_path = matches[0].path.clone();
                }
            }
        }
        if !seen.insert(target_uid.clone()) {
            continue;
        }
        if fetch_folder_if_exists_with_request(&mut *request_json, &target_uid)?.is_none() {
            continue;
        }
        let rows = fetch_folder_permissions_with_request(&mut *request_json, &target_uid)?;
        live_resources.push(live_resource_from_rows(
            source,
            &target_uid,
            &target_path,
            rows,
        ));
    }
    Ok(live_resources)
}

fn permission_key(permission: &FolderPermissionEntry) -> String {
    format!(
        "{}:{}",
        permission.subject_type,
        if permission.subject_key.is_empty() {
            &permission.subject_name
        } else {
            &permission.subject_key
        }
    )
}

fn permission_details(permission: &FolderPermissionEntry) -> FolderPermissionPlanDetails {
    FolderPermissionPlanDetails {
        subject_type: permission.subject_type.clone(),
        subject_key: permission.subject_key.clone(),
        subject_name: permission.subject_name.clone(),
        permission: permission.permission,
        permission_name: permission.permission_name.clone(),
        inherited: permission.inherited,
    }
}

fn permission_change(
    field: &str,
    before: impl Into<Value>,
    after: impl Into<Value>,
) -> DashboardPlanChange {
    DashboardPlanChange {
        field: field.to_string(),
        before: before.into(),
        after: after.into(),
    }
}

fn build_action_id(
    org_id: Option<&str>,
    folder_uid: &str,
    permission: &str,
    seed: usize,
) -> String {
    let org = org_id.unwrap_or("unknown");
    format!("org:{org}/folder-permission:{folder_uid}:{permission}:{seed}")
}

fn build_permission_action(
    org: &OrgPlanInput,
    source: &FolderPermissionResource,
    live: Option<&FolderPermissionResource>,
    permission: Option<&FolderPermissionEntry>,
    action: &str,
    status: &str,
    match_basis: &str,
    changed_fields: Vec<String>,
    changes: Vec<DashboardPlanChange>,
    blocked_reason: Option<String>,
    review_hints: Vec<String>,
    seed: usize,
) -> DashboardPlanAction {
    let permission_key = permission
        .map(permission_key)
        .unwrap_or_else(|| "folder".to_string());
    DashboardPlanAction {
        action_id: build_action_id(
            org.target_org_id
                .as_deref()
                .or(org.source_org_id.as_deref()),
            &source.uid,
            &permission_key,
            seed,
        ),
        domain: DOMAIN_FOLDER_PERMISSION.to_string(),
        resource_kind: RESOURCE_KIND_FOLDER_PERMISSION.to_string(),
        dashboard_uid: String::new(),
        title: source.title.clone(),
        folder_uid: source.uid.clone(),
        folder_path: source.path.clone(),
        source_org_id: org.source_org_id.clone(),
        source_org_name: org.source_org_name.clone(),
        target_org_id: org.target_org_id.clone(),
        target_org_name: org.target_org_name.clone(),
        match_basis: match_basis.to_string(),
        action: action.to_string(),
        status: status.to_string(),
        changed_fields,
        changes,
        source_file: Some("permissions.json".to_string()),
        target_uid: live.map(|item| item.uid.clone()),
        target_version: None,
        target_evidence: Vec::new(),
        dependency_hints: Vec::new(),
        blocked_reason,
        review_hints,
        permission: permission.map(permission_details),
    }
}

fn match_live_folder<'a>(
    source: &FolderPermissionResource,
    live_by_uid: &'a BTreeMap<String, &'a FolderPermissionResource>,
    live_by_path: &'a BTreeMap<String, Vec<&'a FolderPermissionResource>>,
    match_mode: &str,
) -> (Option<&'a FolderPermissionResource>, String, Option<String>) {
    if let Some(live) = live_by_uid.get(&source.uid) {
        return (Some(*live), MATCH_UID.to_string(), None);
    }
    if match_mode == MATCH_UID_THEN_PATH && !source.path.is_empty() {
        if let Some(matches) = live_by_path.get(&source.path) {
            if matches.len() == 1 {
                return (Some(matches[0]), MATCH_UID_THEN_PATH.to_string(), None);
            }
            if matches.len() > 1 {
                return (
                    None,
                    MATCH_UID_THEN_PATH.to_string(),
                    Some(STATUS_AMBIGUOUS_LIVE_FOLDER_PATH.to_string()),
                );
            }
        }
    }
    (
        None,
        match_mode.to_string(),
        Some(STATUS_MISSING_LIVE_FOLDER.to_string()),
    )
}

pub(super) fn build_folder_permission_actions(
    org: &OrgPlanInput,
    match_mode: &str,
) -> Vec<DashboardPlanAction> {
    let live_by_uid = org
        .live_folder_permission_resources
        .iter()
        .map(|resource| (resource.uid.clone(), resource))
        .collect::<BTreeMap<_, _>>();
    let mut live_by_path: BTreeMap<String, Vec<&FolderPermissionResource>> = BTreeMap::new();
    for resource in &org.live_folder_permission_resources {
        if !resource.path.is_empty() {
            live_by_path
                .entry(resource.path.clone())
                .or_default()
                .push(resource);
        }
    }

    let mut actions = Vec::new();
    for source in &org.folder_permission_resources {
        let (live, match_basis, folder_block) =
            match_live_folder(source, &live_by_uid, &live_by_path, match_mode);
        if let Some(reason) = folder_block {
            actions.push(build_permission_action(
                org,
                source,
                None,
                None,
                REVIEW_ACTION_WOULD_UPDATE,
                REVIEW_STATUS_BLOCKED,
                &match_basis,
                Vec::new(),
                Vec::new(),
                Some(if reason == STATUS_AMBIGUOUS_LIVE_FOLDER_PATH {
                    REVIEW_REASON_AMBIGUOUS_LIVE_NAME_MATCH.to_string()
                } else {
                    reason.clone()
                }),
                vec![reason],
                actions.len(),
            ));
            continue;
        }
        let Some(live) = live else {
            continue;
        };
        let live_permissions = live
            .permissions
            .iter()
            .map(|permission| (permission_key(permission), permission))
            .collect::<BTreeMap<_, _>>();
        let source_permissions = source
            .permissions
            .iter()
            .map(|permission| (permission_key(permission), permission))
            .collect::<BTreeMap<_, _>>();

        for (key, source_permission) in &source_permissions {
            let mut review_hints = Vec::new();
            if source_permission.inherited {
                review_hints.push("inherited=true".to_string());
            }
            match live_permissions.get(key) {
                Some(live_permission) if *live_permission == *source_permission => {
                    actions.push(build_permission_action(
                        org,
                        source,
                        Some(live),
                        Some(source_permission),
                        REVIEW_ACTION_SAME,
                        REVIEW_STATUS_SAME,
                        &match_basis,
                        Vec::new(),
                        Vec::new(),
                        None,
                        review_hints,
                        actions.len(),
                    ));
                }
                Some(live_permission) => {
                    actions.push(build_permission_action(
                        org,
                        source,
                        Some(live),
                        Some(source_permission),
                        REVIEW_ACTION_WOULD_UPDATE,
                        REVIEW_STATUS_READY,
                        &match_basis,
                        vec!["permission".to_string()],
                        vec![permission_change(
                            "permission",
                            live_permission.permission,
                            source_permission.permission,
                        )],
                        None,
                        review_hints,
                        actions.len(),
                    ));
                }
                None => {
                    actions.push(build_permission_action(
                        org,
                        source,
                        Some(live),
                        Some(source_permission),
                        REVIEW_ACTION_WOULD_CREATE,
                        REVIEW_STATUS_READY,
                        &match_basis,
                        Vec::new(),
                        Vec::new(),
                        None,
                        review_hints,
                        actions.len(),
                    ));
                }
            }
        }
        for (key, live_permission) in live_permissions {
            if source_permissions.contains_key(&key) {
                continue;
            }
            let mut review_hints = vec!["remote-only folder permission".to_string()];
            if live_permission.inherited {
                review_hints.push("inherited=true".to_string());
            }
            actions.push(build_permission_action(
                org,
                source,
                Some(live),
                Some(live_permission),
                REVIEW_ACTION_EXTRA_REMOTE,
                REVIEW_STATUS_WARNING,
                &match_basis,
                Vec::new(),
                Vec::new(),
                None,
                review_hints,
                actions.len(),
            ));
        }
    }
    actions
}
