//! Dashboard plan reconciliation and summary decisions.

use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::common::{message, string_field, value_as_object, Result};
use crate::review_contract::{
    REVIEW_ACTION_BLOCKED_TARGET, REVIEW_ACTION_EXTRA_REMOTE, REVIEW_ACTION_SAME,
    REVIEW_ACTION_WOULD_CREATE, REVIEW_ACTION_WOULD_DELETE, REVIEW_ACTION_WOULD_UPDATE,
    REVIEW_HINT_REMOTE_ONLY, REVIEW_REASON_TARGET_ORG_MISSING,
    REVIEW_REASON_TARGET_PROVISIONED_OR_MANAGED, REVIEW_STATUS_BLOCKED, REVIEW_STATUS_READY,
    REVIEW_STATUS_SAME, REVIEW_STATUS_WARNING,
};

use super::super::import_target::{
    build_dashboard_target_review, dashboard_target_evidence_blocks_direct_write,
    dashboard_target_evidence_warns_direct_write,
};
use super::super::{
    build_datasource_catalog, build_folder_path, collect_datasource_refs, extract_dashboard_object,
    lookup_datasource, FolderInventoryItem, DEFAULT_FOLDER_TITLE, DEFAULT_FOLDER_UID,
};
use super::types::{
    DashboardPlanAction, DashboardPlanChange, DashboardPlanOrgSummary, DashboardPlanSummary,
    LiveDashboard, LocalDashboard, OrgPlanInput,
};

fn summarize_value(value: &Value) -> Value {
    match value {
        Value::Object(object) if object.len() > 8 => {
            Value::String(format!("object({} keys)", object.len()))
        }
        Value::Array(items) if items.len() > 8 => {
            Value::String(format!("array({} items)", items.len()))
        }
        other => other.clone(),
    }
}

fn strip_dashboard_noise(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for key in ["id", "version", "iteration", "schemaVersion"] {
                object.remove(key);
            }
            for child in object.values_mut() {
                strip_dashboard_noise(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                strip_dashboard_noise(item);
            }
        }
        _ => {}
    }
}

fn normalize_dashboard_document(document: &Value) -> Result<Value> {
    let object = value_as_object(document, "Dashboard payload must be a JSON object.")?;
    let mut normalized = Value::Object(extract_dashboard_object(object)?.clone());
    strip_dashboard_noise(&mut normalized);
    Ok(normalized)
}

fn compare_dashboard_documents(
    left: &Value,
    right: &Value,
) -> (Vec<String>, Vec<DashboardPlanChange>) {
    let mut changed_fields = Vec::new();
    let mut changes = Vec::new();
    compare_json_values("dashboard", left, right, &mut changed_fields, &mut changes);
    (changed_fields, changes)
}

fn compare_json_values(
    prefix: &str,
    left: &Value,
    right: &Value,
    changed_fields: &mut Vec<String>,
    changes: &mut Vec<DashboardPlanChange>,
) {
    if left == right {
        return;
    }
    match (left, right) {
        (Value::Object(left_object), Value::Object(right_object)) => {
            let mut keys = BTreeSet::new();
            for key in left_object.keys().chain(right_object.keys()) {
                keys.insert(key.clone());
            }
            for key in keys {
                let before = left_object.get(&key).unwrap_or(&Value::Null);
                let after = right_object.get(&key).unwrap_or(&Value::Null);
                let field = format!("{prefix}.{key}");
                compare_json_values(&field, before, after, changed_fields, changes);
            }
        }
        (Value::Array(_), Value::Array(_)) => {
            changed_fields.push(prefix.to_string());
            changes.push(DashboardPlanChange {
                field: prefix.to_string(),
                before: summarize_value(left),
                after: summarize_value(right),
            });
        }
        _ => {
            changed_fields.push(prefix.to_string());
            changes.push(DashboardPlanChange {
                field: prefix.to_string(),
                before: summarize_value(left),
                after: summarize_value(right),
            });
        }
    }
}

fn folder_path_for_uid(folder_uid: &str, folder_inventory: &[FolderInventoryItem]) -> String {
    if folder_uid.trim().is_empty() || folder_uid == DEFAULT_FOLDER_UID {
        return DEFAULT_FOLDER_TITLE.to_string();
    }
    folder_inventory
        .iter()
        .find(|folder| folder.uid == folder_uid)
        .map(|folder| folder.path.clone())
        .unwrap_or_else(|| folder_uid.to_string())
}

pub(super) fn build_local_dashboard(
    document: &Value,
    file_path: &Path,
    folder_inventory: &[FolderInventoryItem],
) -> Result<LocalDashboard> {
    let object = value_as_object(document, "Dashboard plan input must be a JSON object.")?;
    let dashboard = normalize_dashboard_document(document)?;
    let dashboard_object = dashboard
        .as_object()
        .ok_or_else(|| message("Dashboard plan input must be a JSON object."))?;
    let dashboard_uid = string_field(dashboard_object, "uid", "");
    let title = string_field(
        dashboard_object,
        "title",
        file_path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("dashboard"),
    );
    let folder_uid = object
        .get("meta")
        .and_then(Value::as_object)
        .and_then(|meta| meta.get("folderUid"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let folder_path = folder_path_for_uid(&folder_uid, folder_inventory);
    Ok(LocalDashboard {
        file_path: file_path.display().to_string(),
        dashboard,
        dashboard_uid,
        title,
        folder_uid,
        folder_path,
    })
}

fn resolve_live_folder_path_with_request<F>(
    request_json: &mut F,
    folder_uid: &str,
) -> Result<String>
where
    F: FnMut(reqwest::Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if folder_uid.trim().is_empty() || folder_uid == DEFAULT_FOLDER_UID {
        return Ok(DEFAULT_FOLDER_TITLE.to_string());
    }
    match super::super::fetch_folder_if_exists_with_request(&mut *request_json, folder_uid)? {
        Some(folder) => Ok(build_folder_path(&folder, DEFAULT_FOLDER_TITLE)),
        None => Ok(folder_uid.to_string()),
    }
}

pub(super) fn build_live_dashboard_with_request<F>(
    request_json: &mut F,
    payload: &Value,
) -> Result<LiveDashboard>
where
    F: FnMut(reqwest::Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let dashboard = normalize_dashboard_document(payload)?;
    let dashboard_object = dashboard
        .as_object()
        .ok_or_else(|| message("Unexpected dashboard payload from Grafana."))?;
    let payload_object = value_as_object(payload, "Unexpected dashboard payload from Grafana.")?;
    let uid = string_field(dashboard_object, "uid", "");
    let title = string_field(dashboard_object, "title", "");
    let folder_uid = payload_object
        .get("meta")
        .and_then(Value::as_object)
        .and_then(|meta| meta.get("folderUid"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let folder_path = resolve_live_folder_path_with_request(request_json, &folder_uid)?;
    let review = build_dashboard_target_review(payload)?;
    Ok(LiveDashboard {
        uid,
        title,
        folder_uid,
        folder_path,
        version: dashboard_object.get("version").and_then(Value::as_i64),
        evidence: review.evidence,
        payload: dashboard,
    })
}

fn count_library_panels(node: &Value) -> usize {
    match node {
        Value::Object(object) => {
            usize::from(object.contains_key("libraryPanel"))
                + object.values().map(count_library_panels).sum::<usize>()
        }
        Value::Array(items) => items.iter().map(count_library_panels).sum(),
        _ => 0,
    }
}

fn build_dependency_hints(
    dashboard: &Value,
    live_datasources: &[Map<String, Value>],
) -> (Vec<String>, Vec<String>) {
    let catalog = build_datasource_catalog(live_datasources);
    let mut refs = Vec::new();
    collect_datasource_refs(dashboard, &mut refs);
    let mut missing = BTreeSet::new();
    for reference in refs {
        let resolved = lookup_datasource(
            &catalog,
            reference.get("uid").and_then(Value::as_str),
            reference.get("name").and_then(Value::as_str),
        );
        if resolved.is_none() {
            let label = reference
                .get("name")
                .and_then(Value::as_str)
                .or_else(|| reference.get("uid").and_then(Value::as_str))
                .unwrap_or("unknown");
            missing.insert(label.to_string());
        }
    }
    let mut dependency_hints = Vec::new();
    let mut review_hints = Vec::new();
    if !missing.is_empty() {
        let labels = missing.into_iter().collect::<Vec<String>>().join(", ");
        dependency_hints.push(format!("missing-datasources={labels}"));
        review_hints.push("dashboard references unresolved datasources".to_string());
    }
    let library_panel_count = count_library_panels(dashboard);
    if library_panel_count > 0 {
        review_hints.push(format!("library-panel-references={library_panel_count}"));
    }
    (dependency_hints, review_hints)
}

fn build_action_id(org_id: Option<&str>, uid: &str, seed: usize) -> String {
    let org = org_id.unwrap_or("unknown");
    let resource = if uid.is_empty() { "dashboard" } else { uid };
    format!("org:{org}/dashboard:{resource}:{seed}")
}

pub(super) fn build_org_actions(org: &OrgPlanInput, prune: bool) -> Vec<DashboardPlanAction> {
    let no_live_target = org.target_org_id.is_none();
    let missing_target = no_live_target && org.org_action == "missing";
    let would_create_target = no_live_target && org.org_action == REVIEW_ACTION_WOULD_CREATE;
    let mut live_by_uid = BTreeMap::new();
    let mut live_by_title: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    let mut live_matched = vec![false; org.live_dashboards.len()];
    for (index, live) in org.live_dashboards.iter().enumerate() {
        if !live.uid.trim().is_empty() {
            live_by_uid.insert(live.uid.clone(), index);
        }
        if !live.title.trim().is_empty() {
            live_by_title
                .entry(live.title.clone())
                .or_default()
                .push(index);
        }
    }

    let mut actions = Vec::new();
    for (index, local) in org.local_dashboards.iter().enumerate() {
        let mut match_basis = "none".to_string();
        let mut live_index = None;
        if !local.dashboard_uid.trim().is_empty() {
            if let Some(position) = live_by_uid.get(&local.dashboard_uid) {
                match_basis = "uid".to_string();
                live_index = Some(*position);
            }
        }
        if live_index.is_none() {
            let title = local.title.trim();
            if !title.is_empty() {
                if let Some(positions) = live_by_title.get(title) {
                    if positions.len() == 1 {
                        match_basis = "title".to_string();
                        live_index = Some(positions[0]);
                    }
                }
            }
        }
        if let Some(position) = live_index {
            live_matched[position] = true;
        }
        let live = live_index.map(|position| &org.live_dashboards[position]);

        let (changed_fields, changes) = if let Some(live) = live {
            compare_dashboard_documents(&local.dashboard, &live.payload)
        } else {
            (Vec::new(), Vec::new())
        };
        let (dependency_hints, mut review_hints) = if no_live_target {
            (Vec::new(), Vec::new())
        } else {
            build_dependency_hints(&local.dashboard, &org.live_datasources)
        };
        if !local.folder_uid.is_empty() && local.folder_uid != DEFAULT_FOLDER_UID {
            review_hints.push(format!("folder-uid={}", local.folder_uid));
        }
        if local.folder_uid.is_empty() || local.folder_uid == DEFAULT_FOLDER_UID {
            review_hints.push("folder=General".to_string());
        }
        if local.folder_uid != DEFAULT_FOLDER_UID
            && !org
                .folder_inventory
                .iter()
                .any(|folder| folder.uid == local.folder_uid)
        {
            review_hints.push(format!("missing-folder-uid={}", local.folder_uid));
        }

        if missing_target {
            review_hints.push(REVIEW_REASON_TARGET_ORG_MISSING.to_string());
        }
        if would_create_target {
            review_hints.push("target-org-would-create".to_string());
        }

        let target_review_blocked = live
            .as_ref()
            .map(|dashboard| dashboard_target_evidence_blocks_direct_write(&dashboard.evidence))
            .unwrap_or(false);
        let target_review_warning = live
            .as_ref()
            .map(|dashboard| dashboard_target_evidence_warns_direct_write(&dashboard.evidence))
            .unwrap_or(false);

        let mut action = if missing_target || would_create_target || live.is_none() {
            REVIEW_ACTION_WOULD_CREATE.to_string()
        } else if changed_fields.is_empty()
            && local.folder_uid == live.map(|item| item.folder_uid.clone()).unwrap_or_default()
        {
            REVIEW_ACTION_SAME.to_string()
        } else {
            REVIEW_ACTION_WOULD_UPDATE.to_string()
        };
        let mut status = if action == REVIEW_ACTION_SAME {
            REVIEW_STATUS_SAME.to_string()
        } else if missing_target {
            REVIEW_STATUS_BLOCKED.to_string()
        } else if would_create_target {
            REVIEW_STATUS_WARNING.to_string()
        } else {
            REVIEW_STATUS_READY.to_string()
        };
        let mut blocked_reason = None;
        if missing_target {
            blocked_reason = Some(REVIEW_REASON_TARGET_ORG_MISSING.to_string());
        } else if target_review_blocked
            && matches!(
                action.as_str(),
                REVIEW_ACTION_WOULD_UPDATE | REVIEW_ACTION_WOULD_DELETE
            )
        {
            action = REVIEW_ACTION_BLOCKED_TARGET.to_string();
            status = REVIEW_STATUS_BLOCKED.to_string();
            blocked_reason = Some(REVIEW_REASON_TARGET_PROVISIONED_OR_MANAGED.to_string());
        } else if action != REVIEW_ACTION_SAME
            && (target_review_warning
                || !dependency_hints.is_empty()
                || review_hints
                    .iter()
                    .any(|hint| hint.starts_with("missing-folder-uid=")))
        {
            status = REVIEW_STATUS_WARNING.to_string();
        }

        let target_uid = live.as_ref().map(|item| item.uid.clone());
        let target_version = live.as_ref().and_then(|item| item.version);
        let target_evidence = live
            .as_ref()
            .map(|item| item.evidence.clone())
            .unwrap_or_default();
        actions.push(DashboardPlanAction {
            action_id: build_action_id(
                org.target_org_id
                    .as_deref()
                    .or(org.source_org_id.as_deref()),
                &local.dashboard_uid,
                index,
            ),
            domain: "dashboard".to_string(),
            resource_kind: "dashboard".to_string(),
            dashboard_uid: local.dashboard_uid.clone(),
            title: local.title.clone(),
            folder_uid: local.folder_uid.clone(),
            folder_path: local.folder_path.clone(),
            source_org_id: org.source_org_id.clone(),
            source_org_name: org.source_org_name.clone(),
            target_org_id: org.target_org_id.clone(),
            target_org_name: org.target_org_name.clone(),
            match_basis,
            action,
            status,
            changed_fields,
            changes,
            source_file: Some(local.file_path.clone()),
            target_uid,
            target_version,
            target_evidence,
            dependency_hints,
            blocked_reason,
            review_hints,
            permission: None,
        });
    }

    for (index, live) in org.live_dashboards.iter().enumerate() {
        if live_matched.get(index).copied().unwrap_or(false) {
            continue;
        }
        let blocked = dashboard_target_evidence_blocks_direct_write(&live.evidence);
        let warned = dashboard_target_evidence_warns_direct_write(&live.evidence);
        actions.push(DashboardPlanAction {
            action_id: build_action_id(
                org.target_org_id
                    .as_deref()
                    .or(org.source_org_id.as_deref()),
                &live.uid,
                index,
            ),
            domain: "dashboard".to_string(),
            resource_kind: "dashboard".to_string(),
            dashboard_uid: live.uid.clone(),
            title: live.title.clone(),
            folder_uid: live.folder_uid.clone(),
            folder_path: live.folder_path.clone(),
            source_org_id: org.source_org_id.clone(),
            source_org_name: org.source_org_name.clone(),
            target_org_id: org.target_org_id.clone(),
            target_org_name: org.target_org_name.clone(),
            match_basis: "live-only".to_string(),
            action: if prune && blocked {
                REVIEW_ACTION_BLOCKED_TARGET.to_string()
            } else if prune {
                REVIEW_ACTION_WOULD_DELETE.to_string()
            } else {
                REVIEW_ACTION_EXTRA_REMOTE.to_string()
            },
            status: if blocked {
                REVIEW_STATUS_BLOCKED.to_string()
            } else if warned {
                REVIEW_STATUS_WARNING.to_string()
            } else if prune {
                REVIEW_STATUS_READY.to_string()
            } else {
                REVIEW_STATUS_WARNING.to_string()
            },
            changed_fields: Vec::new(),
            changes: Vec::new(),
            source_file: None,
            target_uid: Some(live.uid.clone()),
            target_version: live.version,
            target_evidence: live.evidence.clone(),
            dependency_hints: Vec::new(),
            blocked_reason: blocked
                .then_some(REVIEW_REASON_TARGET_PROVISIONED_OR_MANAGED.to_string()),
            review_hints: vec![format!("{REVIEW_HINT_REMOTE_ONLY} dashboard candidate")],
            permission: None,
        });
    }

    actions
}

pub(super) fn build_org_summary(
    org: &OrgPlanInput,
    actions: &[DashboardPlanAction],
) -> DashboardPlanOrgSummary {
    let mut summary = DashboardPlanOrgSummary {
        source_org_id: org.source_org_id.clone(),
        source_org_name: org.source_org_name.clone(),
        target_org_id: org.target_org_id.clone(),
        target_org_name: org.target_org_name.clone(),
        org_action: org.org_action.clone(),
        input_dir: org.input_dir.display().to_string(),
        checked: actions.len(),
        same: 0,
        create: 0,
        update: 0,
        extra: 0,
        delete: 0,
        blocked: 0,
        warning: 0,
    };
    for action in actions {
        match action.action.as_str() {
            REVIEW_ACTION_SAME => summary.same += 1,
            REVIEW_ACTION_WOULD_CREATE => summary.create += 1,
            REVIEW_ACTION_WOULD_UPDATE => summary.update += 1,
            REVIEW_ACTION_EXTRA_REMOTE => summary.extra += 1,
            REVIEW_ACTION_WOULD_DELETE => summary.delete += 1,
            _ => {}
        }
        match action.status.as_str() {
            REVIEW_STATUS_BLOCKED => summary.blocked += 1,
            REVIEW_STATUS_WARNING => summary.warning += 1,
            _ => {}
        }
    }
    summary
}

pub(super) fn build_summary(
    orgs: &[DashboardPlanOrgSummary],
    actions: &[DashboardPlanAction],
) -> DashboardPlanSummary {
    let mut summary = DashboardPlanSummary {
        checked: actions.len(),
        same: 0,
        create: 0,
        update: 0,
        extra: 0,
        delete: 0,
        blocked: 0,
        warning: 0,
        org_count: orgs.len(),
        would_create_org_count: 0,
    };
    for org in orgs {
        summary.same += org.same;
        summary.create += org.create;
        summary.update += org.update;
        summary.extra += org.extra;
        summary.delete += org.delete;
        summary.blocked += org.blocked;
        summary.warning += org.warning;
        if org.org_action == REVIEW_ACTION_WOULD_CREATE {
            summary.would_create_org_count += 1;
        }
    }
    summary
}
