use reqwest::Method;
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

use crate::common::{message, render_json_value, string_field, value_as_object, Result};
use crate::http::JsonHttpClient;
use crate::tabular_output::render_table;

use super::{
    collect_folder_inventory_with_request, extract_dashboard_object,
    fetch_dashboard_if_exists_with_request, fetch_dashboard_with_request,
    fetch_folder_if_exists_with_request, list_dashboard_summaries_with_request, CloneFolderArgs,
    CloneFolderOutputFormat, FolderInventoryItem,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CloneFolderAction {
    pub(crate) action: String,
    pub(crate) source_uid: String,
    pub(crate) target_uid: String,
    pub(crate) target_title: String,
    #[serde(rename = "targetParentUid")]
    pub(crate) target_parent_uid: Option<String>,
    pub(crate) reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CloneFolderDashboardAction {
    pub(crate) action: String,
    #[serde(rename = "sourceUid")]
    pub(crate) source_uid: String,
    #[serde(rename = "sourceTitle")]
    pub(crate) source_title: String,
    #[serde(rename = "sourceFolderUid")]
    pub(crate) source_folder_uid: String,
    #[serde(rename = "targetUid")]
    pub(crate) target_uid: String,
    #[serde(rename = "targetTitle")]
    pub(crate) target_title: String,
    #[serde(rename = "targetFolderUid")]
    pub(crate) target_folder_uid: String,
    pub(crate) reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CloneFolderReport {
    #[serde(rename = "sourceFolderUid")]
    pub(crate) source_folder_uid: String,
    #[serde(rename = "targetFolderUid")]
    pub(crate) target_folder_uid: String,
    pub(crate) recursive: bool,
    #[serde(rename = "dryRun")]
    pub(crate) dry_run: bool,
    #[serde(rename = "folderActions")]
    pub(crate) folder_actions: Vec<CloneFolderAction>,
    #[serde(rename = "dashboardActions")]
    pub(crate) dashboard_actions: Vec<CloneFolderDashboardAction>,
}

#[derive(Debug, Clone)]
struct TargetFolderSpec {
    uid: String,
    title: String,
    parent_uid: Option<String>,
    source_uid: String,
    compare_title: bool,
    compare_parent: bool,
}

pub(crate) fn run_clone_folder_with_client(
    client: &JsonHttpClient,
    args: &CloneFolderArgs,
) -> Result<CloneFolderReport> {
    run_clone_folder_with_request(
        |method, path, params, payload| client.request_json(method, path, params, payload),
        args,
    )
}

pub(crate) fn run_clone_folder_with_request<F>(
    mut request_json: F,
    args: &CloneFolderArgs,
) -> Result<CloneFolderReport>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    validate_args(args)?;
    let summaries = list_dashboard_summaries_with_request(&mut request_json, args.page_size)?;
    let folders = collect_folder_inventory_with_request(&mut request_json, &summaries)?;
    let source_folder = resolve_source_folder(args, &folders)?;
    let target_specs = build_target_folder_specs(args, &source_folder, &folders)?;
    let folder_actions = plan_folder_actions(&mut request_json, args, &target_specs)?;
    let dashboard_actions = plan_dashboard_actions(
        &mut request_json,
        args,
        &summaries,
        &folders,
        &source_folder,
    )?;

    let report = CloneFolderReport {
        source_folder_uid: source_folder.uid.clone(),
        target_folder_uid: args.target_folder_uid.clone(),
        recursive: args.recursive,
        dry_run: args.dry_run,
        folder_actions,
        dashboard_actions,
    };

    if !args.dry_run {
        apply_clone_folder_plan(&mut request_json, args, &report)?;
    }
    Ok(report)
}

fn validate_args(args: &CloneFolderArgs) -> Result<()> {
    if args.source_folder_uid.is_none() && args.source_path.is_none() {
        return Err(message(
            "dashboard clone-folder requires --source-folder-uid or --source-path.",
        ));
    }
    if args.target_folder_uid.trim().is_empty() {
        return Err(message(
            "dashboard clone-folder requires a non-empty --target-folder-uid.",
        ));
    }
    if !args.dry_run && !args.yes {
        return Err(message(
            "Refusing to clone live folder without --yes. Use --dry-run to preview first.",
        ));
    }
    if args.create_target_folder
        && args
            .target_folder_title
            .as_deref()
            .map(str::trim)
            .unwrap_or("")
            .is_empty()
    {
        return Err(message(
            "dashboard clone-folder with --create-target-folder requires --target-folder-title.",
        ));
    }
    if args.uid_prefix.as_deref().unwrap_or("").is_empty() && args.uid_suffix.is_empty() {
        return Err(message(
            "dashboard clone-folder requires --uid-prefix or --uid-suffix to produce new dashboard UIDs.",
        ));
    }
    Ok(())
}

fn resolve_source_folder(
    args: &CloneFolderArgs,
    folders: &[FolderInventoryItem],
) -> Result<FolderInventoryItem> {
    if let Some(uid) = args.source_folder_uid.as_deref() {
        return folders
            .iter()
            .find(|folder| folder.uid == uid)
            .cloned()
            .ok_or_else(|| message(format!("Source folder UID not found: {uid}")));
    }
    let source_path = args.source_path.as_deref().unwrap_or("").trim();
    folders
        .iter()
        .find(|folder| folder.path == source_path)
        .cloned()
        .ok_or_else(|| message(format!("Source folder path not found: {source_path}")))
}

fn build_target_folder_specs(
    args: &CloneFolderArgs,
    source_folder: &FolderInventoryItem,
    folders: &[FolderInventoryItem],
) -> Result<Vec<TargetFolderSpec>> {
    let mut specs = Vec::new();
    specs.push(TargetFolderSpec {
        uid: args.target_folder_uid.clone(),
        title: args
            .target_folder_title
            .clone()
            .unwrap_or_else(|| source_folder.title.clone()),
        parent_uid: args.target_parent_folder_uid.clone(),
        source_uid: source_folder.uid.clone(),
        compare_title: args.target_folder_title.is_some(),
        compare_parent: args.target_parent_folder_uid.is_some(),
    });
    if !args.recursive {
        return Ok(specs);
    }
    let descendants = descendant_folder_uids(folders, &source_folder.uid);
    for folder in folders {
        if !descendants.contains(&folder.uid) {
            continue;
        }
        specs.push(TargetFolderSpec {
            uid: target_child_folder_uid(args, &folder.uid),
            title: folder.title.clone(),
            parent_uid: Some(match folder.parent_uid.as_deref() {
                Some(parent_uid) if parent_uid == source_folder.uid => {
                    args.target_folder_uid.clone()
                }
                Some(parent_uid) => target_child_folder_uid(args, parent_uid),
                None => args.target_folder_uid.clone(),
            }),
            source_uid: folder.uid.clone(),
            compare_title: true,
            compare_parent: true,
        });
    }
    Ok(specs)
}

fn descendant_folder_uids(
    folders: &[FolderInventoryItem],
    source_folder_uid: &str,
) -> BTreeSet<String> {
    let by_parent = folders.iter().fold(
        BTreeMap::<String, Vec<String>>::new(),
        |mut grouped, folder| {
            if let Some(parent_uid) = folder.parent_uid.as_ref() {
                grouped
                    .entry(parent_uid.clone())
                    .or_default()
                    .push(folder.uid.clone());
            }
            grouped
        },
    );
    let mut seen = BTreeSet::new();
    let mut stack = by_parent
        .get(source_folder_uid)
        .cloned()
        .unwrap_or_default();
    while let Some(uid) = stack.pop() {
        if !seen.insert(uid.clone()) {
            continue;
        }
        if let Some(children) = by_parent.get(&uid) {
            stack.extend(children.iter().cloned());
        }
    }
    seen
}

fn target_child_folder_uid(args: &CloneFolderArgs, source_uid: &str) -> String {
    format!("{}-{}", args.target_folder_uid, source_uid)
}

fn plan_folder_actions<F>(
    request_json: &mut F,
    args: &CloneFolderArgs,
    target_specs: &[TargetFolderSpec],
) -> Result<Vec<CloneFolderAction>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut actions = Vec::new();
    for spec in target_specs {
        match fetch_folder(request_json, &spec.uid)? {
            Some(existing) => {
                let actual_title = string_field(&existing, "title", "");
                let actual_parent_uid = parent_uid_from_folder(&existing);
                let title_mismatch =
                    spec.compare_title && !actual_title.is_empty() && actual_title != spec.title;
                let parent_mismatch = spec.compare_parent && actual_parent_uid != spec.parent_uid;
                if title_mismatch || parent_mismatch {
                    actions.push(CloneFolderAction {
                        action: "blocked".to_string(),
                        source_uid: spec.source_uid.clone(),
                        target_uid: spec.uid.clone(),
                        target_title: spec.title.clone(),
                        target_parent_uid: spec.parent_uid.clone(),
                        reason: Some("target-folder-mismatch".to_string()),
                    });
                }
            }
            None if args.create_target_folder => actions.push(CloneFolderAction {
                action: "create".to_string(),
                source_uid: spec.source_uid.clone(),
                target_uid: spec.uid.clone(),
                target_title: spec.title.clone(),
                target_parent_uid: spec.parent_uid.clone(),
                reason: None,
            }),
            None => actions.push(CloneFolderAction {
                action: "blocked".to_string(),
                source_uid: spec.source_uid.clone(),
                target_uid: spec.uid.clone(),
                target_title: spec.title.clone(),
                target_parent_uid: spec.parent_uid.clone(),
                reason: Some("target-folder-missing".to_string()),
            }),
        }
    }
    Ok(actions)
}

fn fetch_folder<F>(request_json: &mut F, uid: &str) -> Result<Option<Map<String, Value>>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    fetch_folder_if_exists_with_request(request_json, uid)
}

fn parent_uid_from_folder(folder: &Map<String, Value>) -> Option<String> {
    folder
        .get("parents")
        .and_then(Value::as_array)
        .and_then(|parents| parents.last())
        .and_then(Value::as_object)
        .map(|parent| string_field(parent, "uid", ""))
        .filter(|uid| !uid.is_empty())
}

fn plan_dashboard_actions<F>(
    request_json: &mut F,
    args: &CloneFolderArgs,
    summaries: &[Map<String, Value>],
    folders: &[FolderInventoryItem],
    source_folder: &FolderInventoryItem,
) -> Result<Vec<CloneFolderDashboardAction>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let descendants = if args.recursive {
        descendant_folder_uids(folders, &source_folder.uid)
    } else {
        BTreeSet::new()
    };
    let mut actions = Vec::new();
    for summary in summaries {
        let source_folder_uid = string_field(summary, "folderUid", "");
        let in_scope = source_folder_uid == source_folder.uid
            || (args.recursive && descendants.contains(&source_folder_uid));
        if !in_scope {
            continue;
        }
        let source_uid = string_field(summary, "uid", "");
        if source_uid.is_empty() {
            continue;
        }
        let target_uid = cloned_dashboard_uid(args, &source_uid);
        let target_title = cloned_dashboard_title(args, &string_field(summary, "title", ""));
        let target_folder_uid = if source_folder_uid == source_folder.uid {
            args.target_folder_uid.clone()
        } else {
            target_child_folder_uid(args, &source_folder_uid)
        };
        let action = match fetch_dashboard_exists(request_json, &target_uid)? {
            true if args.replace_existing => ("update".to_string(), None),
            true => (
                "blocked".to_string(),
                Some("target-dashboard-exists".to_string()),
            ),
            false => ("create".to_string(), None),
        };
        actions.push(CloneFolderDashboardAction {
            action: action.0,
            source_uid,
            source_title: string_field(summary, "title", ""),
            source_folder_uid,
            target_uid,
            target_title,
            target_folder_uid,
            reason: action.1,
        });
    }
    Ok(actions)
}

fn cloned_dashboard_uid(args: &CloneFolderArgs, source_uid: &str) -> String {
    format!(
        "{}{}{}",
        args.uid_prefix.as_deref().unwrap_or(""),
        source_uid,
        args.uid_suffix
    )
}

fn cloned_dashboard_title(args: &CloneFolderArgs, source_title: &str) -> String {
    format!(
        "{}{}{}",
        args.title_prefix.as_deref().unwrap_or(""),
        source_title,
        args.title_suffix.as_deref().unwrap_or("")
    )
}

fn fetch_dashboard_exists<F>(request_json: &mut F, uid: &str) -> Result<bool>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    fetch_dashboard_if_exists_with_request(request_json, uid).map(|value| value.is_some())
}

fn apply_clone_folder_plan<F>(
    request_json: &mut F,
    args: &CloneFolderArgs,
    report: &CloneFolderReport,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if let Some(blocked) = report
        .folder_actions
        .iter()
        .find(|action| action.action == "blocked")
    {
        return Err(message(format!(
            "Cannot clone folder: target folder {} is blocked: {}",
            blocked.target_uid,
            blocked.reason.as_deref().unwrap_or("unknown")
        )));
    }
    if let Some(blocked) = report
        .dashboard_actions
        .iter()
        .find(|action| action.action == "blocked")
    {
        return Err(message(format!(
            "Cannot clone folder: target dashboard {} is blocked: {}",
            blocked.target_uid,
            blocked.reason.as_deref().unwrap_or("unknown")
        )));
    }
    for action in report
        .folder_actions
        .iter()
        .filter(|action| action.action == "create")
    {
        let mut payload = Map::new();
        payload.insert("uid".to_string(), Value::String(action.target_uid.clone()));
        payload.insert(
            "title".to_string(),
            Value::String(action.target_title.clone()),
        );
        if let Some(parent_uid) = action.target_parent_uid.as_ref() {
            payload.insert("parentUid".to_string(), Value::String(parent_uid.clone()));
        }
        request_json(
            Method::POST,
            "/api/folders",
            &[],
            Some(&Value::Object(payload)),
        )?;
    }
    for action in &report.dashboard_actions {
        let source_payload = fetch_dashboard_with_request(&mut *request_json, &action.source_uid)?;
        let object = value_as_object(
            &source_payload,
            &format!(
                "Unexpected dashboard payload for UID {}.",
                action.source_uid
            ),
        )?;
        let mut dashboard = extract_dashboard_object(object)?.clone();
        dashboard.insert("id".to_string(), Value::Null);
        dashboard.insert("uid".to_string(), Value::String(action.target_uid.clone()));
        dashboard.insert(
            "title".to_string(),
            Value::String(action.target_title.clone()),
        );
        let mut payload = Map::new();
        payload.insert("dashboard".to_string(), Value::Object(dashboard));
        payload.insert("overwrite".to_string(), Value::Bool(args.replace_existing));
        payload.insert("message".to_string(), Value::String(args.message.clone()));
        payload.insert(
            "folderUid".to_string(),
            Value::String(action.target_folder_uid.clone()),
        );
        request_json(
            Method::POST,
            "/api/dashboards/db",
            &[],
            Some(&Value::Object(payload)),
        )?;
    }
    Ok(())
}

pub(crate) fn render_clone_folder_report(
    report: &CloneFolderReport,
    args: &CloneFolderArgs,
) -> Result<Vec<String>> {
    let output = args.output_format.unwrap_or({
        if args.json {
            CloneFolderOutputFormat::Json
        } else if args.table {
            CloneFolderOutputFormat::Table
        } else {
            CloneFolderOutputFormat::Text
        }
    });
    match output {
        CloneFolderOutputFormat::Json => {
            Ok(vec![render_json_value(&serde_json::to_value(report)?)?])
        }
        CloneFolderOutputFormat::Table => Ok(render_clone_folder_table(report, args.no_header)),
        CloneFolderOutputFormat::Text => Ok(render_clone_folder_text(report)),
    }
}

fn render_clone_folder_text(report: &CloneFolderReport) -> Vec<String> {
    let mut lines = vec![format!(
        "Clone folder {} -> {}",
        report.source_folder_uid, report.target_folder_uid
    )];
    for action in &report.folder_actions {
        lines.push(format!(
            "Folder {} uid={} title={} parentUid={}{}",
            action.action,
            action.target_uid,
            action.target_title,
            action.target_parent_uid.as_deref().unwrap_or("-"),
            reason_suffix(action.reason.as_deref())
        ));
    }
    for action in &report.dashboard_actions {
        lines.push(format!(
            "Dashboard {} {} -> {} folderUid={}{}",
            action.action,
            action.source_uid,
            action.target_uid,
            action.target_folder_uid,
            reason_suffix(action.reason.as_deref())
        ));
    }
    lines
}

fn reason_suffix(reason: Option<&str>) -> String {
    reason
        .map(|value| format!(" reason={value}"))
        .unwrap_or_default()
}

fn render_clone_folder_table(report: &CloneFolderReport, no_header: bool) -> Vec<String> {
    let mut rows = Vec::new();
    for action in &report.folder_actions {
        rows.push(vec![
            "folder".to_string(),
            action.action.clone(),
            action.source_uid.clone(),
            action.target_uid.clone(),
            action
                .target_parent_uid
                .clone()
                .unwrap_or_else(|| "-".to_string()),
            action.reason.clone().unwrap_or_else(|| "-".to_string()),
        ]);
    }
    for action in &report.dashboard_actions {
        rows.push(vec![
            "dashboard".to_string(),
            action.action.clone(),
            action.source_uid.clone(),
            action.target_uid.clone(),
            action.target_folder_uid.clone(),
            action.reason.clone().unwrap_or_else(|| "-".to_string()),
        ]);
    }
    let mut lines = render_table(
        &[
            "kind",
            "action",
            "source_uid",
            "target_uid",
            "folder_uid",
            "reason",
        ],
        &rows,
    );
    if no_header && lines.len() >= 2 {
        lines.drain(0..2);
    }
    lines
}
