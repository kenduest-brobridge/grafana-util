#![cfg(feature = "tui")]
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use reqwest::Method;
use serde_json::{Map, Value};

use crate::common::{message, string_field, Result};
use crate::dashboard::files::{
    discover_dashboard_files, extract_dashboard_object, load_export_metadata,
    load_folder_inventory, load_json_file,
};
use crate::grafana_api::DashboardResourceClient;

use super::super::delete_support::normalize_folder_path;
use super::super::inspect::{resolve_export_folder_inventory_item, resolve_export_folder_path};
use super::super::list::{fetch_current_org_with_request, org_id_value};
use super::super::source_loader::{load_dashboard_source, LoadedDashboardSource};
use super::super::{
    build_auth_context, build_http_client, build_http_client_for_org,
    collect_folder_inventory_with_request, list_dashboard_summaries_with_request, BrowseArgs,
    DashboardImportInputFormat, DEFAULT_DASHBOARD_TITLE, DEFAULT_FOLDER_TITLE,
};
use super::browse_support::{
    build_dashboard_browse_document_for_org, build_org_node, DashboardBrowseDocument,
    DashboardBrowseSummary,
};

pub(crate) fn load_dashboard_browse_document_for_args<F>(
    request_json: &mut F,
    args: &BrowseArgs,
) -> Result<DashboardBrowseDocument>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if let Some(input_dir) = args.input_dir.as_deref().or(args.workspace.as_deref()) {
        return load_dashboard_browse_document_from_local_import_dir(
            input_dir,
            args.input_format,
            args.path.as_deref(),
            args.workspace.is_some(),
        );
    }
    if args.all_orgs {
        return load_dashboard_browse_document_all_orgs(request_json, args);
    }
    load_dashboard_browse_document_with_request(request_json, args.page_size, args.path.as_deref())
}

fn load_dashboard_browse_document_from_local_import_dir(
    input_dir: &Path,
    input_format: DashboardImportInputFormat,
    root_path: Option<&str>,
    strict_workspace: bool,
) -> Result<DashboardBrowseDocument> {
    let resolved = resolve_local_browse_source(input_dir, input_format, strict_workspace)?;
    let metadata = load_export_metadata(&resolved.resolved.metadata_dir, None)?;
    let folder_inventory =
        load_folder_inventory(&resolved.resolved.metadata_dir, metadata.as_ref())?;
    let dashboard_files = discover_dashboard_files(&resolved.resolved.dashboard_dir)?;
    let summaries = build_local_dashboard_summaries(
        &resolved.resolved.dashboard_dir,
        &dashboard_files,
        &folder_inventory,
        metadata.as_ref(),
    )?;
    let org_name = metadata
        .as_ref()
        .and_then(|item| item.org.as_deref())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Local export");
    let org_id = metadata
        .as_ref()
        .and_then(|item| item.org_id.as_deref())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(super::super::DEFAULT_ORG_ID);
    let mut document = build_dashboard_browse_document_for_org(
        &summaries,
        &folder_inventory,
        root_path,
        org_name,
        org_id,
        true,
        false,
    )?;
    let scope_prefix = match resolved.layout_kind {
        super::super::source_loader::DashboardWorkspaceLayoutKind::GitSync => {
            "Local Git Sync review tree"
        }
        super::super::source_loader::DashboardWorkspaceLayoutKind::Export => "Local export tree",
    };
    document.summary.scope_label = format!(
        "{scope_prefix} ({})",
        resolved.resolved.dashboard_dir.display()
    );
    Ok(document)
}

fn resolve_local_browse_source(
    input_dir: &Path,
    input_format: DashboardImportInputFormat,
    strict_workspace: bool,
) -> Result<LoadedDashboardSource> {
    load_dashboard_source(input_dir, input_format, None, strict_workspace)
}

fn build_local_dashboard_summaries(
    input_dir: &Path,
    dashboard_files: &[PathBuf],
    folder_inventory: &[super::super::FolderInventoryItem],
    metadata: Option<&crate::dashboard::models::ExportMetadata>,
) -> Result<Vec<Map<String, Value>>> {
    let mut summaries = Vec::new();
    let folder_inventory_by_uid = folder_inventory
        .iter()
        .cloned()
        .map(|item| (item.uid.clone(), item))
        .collect::<BTreeMap<String, super::super::FolderInventoryItem>>();
    for dashboard_file in dashboard_files {
        if dashboard_file
            .file_name()
            .and_then(|value| value.to_str())
            .map(|value| value.ends_with(".history.json"))
            .unwrap_or(false)
            || dashboard_file
                .components()
                .any(|component| component.as_os_str() == "history")
        {
            continue;
        }
        let document = load_json_file(dashboard_file)?;
        let document_object = document
            .as_object()
            .ok_or_else(|| message("Dashboard file must be a JSON object."))?;
        let dashboard = extract_dashboard_object(document_object)?;
        let uid = string_field(dashboard, "uid", "");
        if uid.is_empty() {
            continue;
        }
        let folder_item = resolve_export_folder_inventory_item(
            document_object,
            dashboard_file,
            input_dir,
            &folder_inventory_by_uid,
        );
        let folder_path = resolve_export_folder_path(
            document_object,
            dashboard_file,
            input_dir,
            &folder_inventory_by_uid,
        );
        let folder_uid = folder_item
            .as_ref()
            .map(|item| item.uid.clone())
            .unwrap_or_else(|| string_field(document_object, "folderUid", ""));
        let folder_title = folder_item
            .as_ref()
            .map(|item| item.title.clone())
            .unwrap_or_else(|| string_field(document_object, "folderTitle", DEFAULT_FOLDER_TITLE));
        let mut summary = Map::new();
        summary.insert("uid".to_string(), Value::String(uid));
        summary.insert(
            "title".to_string(),
            Value::String(string_field(dashboard, "title", DEFAULT_DASHBOARD_TITLE)),
        );
        summary.insert("folderUid".to_string(), Value::String(folder_uid));
        summary.insert("folderTitle".to_string(), Value::String(folder_title));
        summary.insert("folderPath".to_string(), Value::String(folder_path));
        summary.insert("url".to_string(), Value::String(String::new()));
        summary.insert(
            "sourceFile".to_string(),
            Value::String(dashboard_file.display().to_string()),
        );
        summary.insert(
            "orgName".to_string(),
            Value::String(
                metadata
                    .and_then(|item| item.org.clone())
                    .unwrap_or_else(|| "Local export".to_string()),
            ),
        );
        summary.insert(
            "orgId".to_string(),
            Value::String(
                metadata
                    .and_then(|item| item.org_id.clone())
                    .unwrap_or_else(|| super::super::DEFAULT_ORG_ID.to_string()),
            ),
        );
        summaries.push(summary);
    }
    Ok(summaries)
}

pub(crate) fn load_dashboard_browse_document_with_request<F>(
    mut request_json: F,
    page_size: usize,
    root_path: Option<&str>,
) -> Result<DashboardBrowseDocument>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let org = fetch_current_org_with_request(&mut request_json)?;
    let org_name = string_field(&org, "name", "");
    let org_id = org
        .get("id")
        .map(Value::to_string)
        .unwrap_or_else(|| super::super::DEFAULT_ORG_ID.to_string());
    let dashboard_summaries = list_dashboard_summaries_with_request(&mut request_json, page_size)?;
    let summaries = super::super::list::attach_dashboard_folder_paths_with_request(
        &mut request_json,
        &dashboard_summaries,
    )?;
    let folder_inventory = collect_folder_inventory_with_request(&mut request_json, &summaries)?;
    build_dashboard_browse_document_for_org(
        &summaries,
        &folder_inventory,
        root_path,
        &org_name,
        &org_id,
        false,
        false,
    )
}

fn load_dashboard_browse_document_all_orgs<F>(
    _request_json: &mut F,
    args: &BrowseArgs,
) -> Result<DashboardBrowseDocument>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let context = build_auth_context(&args.common)?;
    if context.auth_mode != "basic" {
        return Err(message(
            "Dashboard browse with --all-orgs requires Basic auth (--basic-user / --basic-password).",
        ));
    }

    let client = build_http_client(&args.common)?;
    let dashboard = DashboardResourceClient::new(&client);
    let mut orgs = dashboard.list_orgs()?;
    orgs.sort_by(|left, right| {
        string_field(left, "name", "")
            .to_ascii_lowercase()
            .cmp(&string_field(right, "name", "").to_ascii_lowercase())
            .then_with(|| {
                left.get("id")
                    .map(Value::to_string)
                    .cmp(&right.get("id").map(Value::to_string))
            })
    });

    let mut nodes = Vec::new();
    let mut folder_count = 0usize;
    let mut dashboard_count = 0usize;
    let mut matched_orgs = 0usize;

    for org in &orgs {
        let org_name = string_field(org, "name", "");
        let org_id = org_id_value(org)?;
        let org_id_text = org_id.to_string();
        let client = build_http_client_for_org(&args.common, org_id)?;
        let dashboard = DashboardResourceClient::new(&client);
        let dashboard_summaries = dashboard.list_dashboard_summaries(args.page_size)?;
        let summaries = super::super::list::attach_dashboard_folder_paths_with_request(
            |method, path, params, payload| dashboard.request_json(method, path, params, payload),
            &dashboard_summaries,
        )?;
        let folder_inventory = collect_folder_inventory_with_request(
            |method, path, params, payload| dashboard.request_json(method, path, params, payload),
            &summaries,
        )?;
        let scoped = build_dashboard_browse_document_for_org(
            &summaries,
            &folder_inventory,
            args.path.as_deref(),
            &org_name,
            &org_id_text,
            false,
            true,
        )?;
        if scoped.summary.dashboard_count == 0 && scoped.summary.folder_count == 0 {
            continue;
        }
        matched_orgs += 1;
        folder_count += scoped.summary.folder_count;
        dashboard_count += scoped.summary.dashboard_count;
        nodes.push(build_org_node(
            &org_name,
            &org_id_text,
            scoped.summary.folder_count,
            scoped.summary.dashboard_count,
        ));
        nodes.extend(scoped.nodes.into_iter().map(|mut node| {
            node.depth += 1;
            node
        }));
    }

    if matched_orgs == 0 {
        if let Some(root) = args.path.as_deref() {
            return Err(message(format!(
                "Dashboard browser folder path did not match any dashboards across all visible orgs: {}",
                normalize_folder_path(root)
            )));
        }
    }

    Ok(DashboardBrowseDocument {
        summary: DashboardBrowseSummary {
            root_path: args.path.as_ref().map(|value| normalize_folder_path(value)),
            dashboard_count,
            folder_count,
            org_count: matched_orgs,
            scope_label: if matched_orgs > 0 {
                "All visible orgs".to_string()
            } else {
                "No matching orgs".to_string()
            },
        },
        nodes,
    })
}
