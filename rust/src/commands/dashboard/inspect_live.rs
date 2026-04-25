//! Live dashboard inspection staging and export orchestration.
//!
//! This module turns live Grafana reads into the same raw export contract consumed by the
//! offline inspect pipeline, so the exported files and the live request path can share the
//! same summary/report/governance builders.
use reqwest::Method;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

use crate::common::{string_field, value_as_object, Result};
use crate::grafana_api::{DashboardResourceClient, DatasourceResourceClient};
use crate::http::JsonHttpClient;

mod fetch;
mod staging;
pub(crate) use fetch::snapshot_live_dashboard_export_with_fetcher;
#[cfg(test)]
pub(crate) use staging::prepare_inspect_export_import_dir;
pub(crate) use staging::{
    load_variant_index_entries, prepare_inspect_export_import_dir_for_variant,
    prepare_inspect_live_import_dir, prepare_live_review_import_dir, TempInspectDir,
};

use super::cli_defs::{ExportArgs, InspectExportArgs, InspectLiveArgs};
use super::export::export_dashboards_with_request;
use super::files::write_json_document;
use super::inspect::analyze_export_dir;
#[cfg(feature = "tui")]
use super::inspect::build_export_inspection_query_report;
#[cfg(feature = "tui")]
use super::inspect::build_export_inspection_summary;
#[cfg(feature = "tui")]
use super::inspect_governance::build_export_inspection_governance_document;
#[cfg(feature = "tui")]
use super::inspect_live_tui::run_inspect_live_interactive as run_inspect_live_tui;
use super::live::build_datasource_inventory_record;
use super::{
    FolderInventoryItem, DATASOURCE_INVENTORY_FILENAME, DEFAULT_FOLDER_TITLE, DEFAULT_ORG_ID,
    DEFAULT_ORG_NAME, FOLDER_INVENTORY_FILENAME, RAW_EXPORT_SUBDIR,
};

pub(crate) fn build_review_live_export_args(
    common: &crate::dashboard::CommonCliArgs,
    output_dir: PathBuf,
    page_size: usize,
    org_id: Option<i64>,
    all_orgs: bool,
) -> ExportArgs {
    ExportArgs {
        common: common.clone(),
        output_dir,
        page_size,
        org_id,
        all_orgs,
        flat: false,
        overwrite: false,
        without_dashboard_raw: false,
        without_dashboard_prompt: true,
        without_dashboard_provisioning: true,
        include_history: false,
        provisioning_provider_name: "grafana-utils-dashboards".to_string(),
        provisioning_provider_org_id: None,
        provisioning_provider_path: None,
        provisioning_provider_disable_deletion: false,
        provisioning_provider_allow_ui_updates: false,
        provisioning_provider_update_interval_seconds: 30,
        dry_run: false,
        progress: false,
        verbose: false,
        run: None,
        run_id: None,
    }
}

fn build_live_export_args(args: &InspectLiveArgs, output_dir: PathBuf) -> ExportArgs {
    let mut export_args = build_review_live_export_args(
        &args.common,
        output_dir,
        args.page_size,
        args.org_id,
        args.all_orgs,
    );
    export_args.progress = args.progress;
    export_args
}

fn collect_folder_inventory_from_summaries(
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

#[cfg(feature = "tui")]
// Build live inspection artifacts and open the interactive export workbench.
fn run_interactive_inspect_live_tui_from_dir(input_dir: &Path) -> Result<usize> {
    eprintln!(
        "[summary-live --interactive] building summary: {}",
        input_dir.display()
    );
    let summary = build_export_inspection_summary(input_dir)?;
    eprintln!(
        "[summary-live --interactive] building query report: {}",
        input_dir.display()
    );
    let report = build_export_inspection_query_report(input_dir)?;
    eprintln!(
        "[summary-live --interactive] building governance review: {}",
        input_dir.display()
    );
    let governance = build_export_inspection_governance_document(&summary, &report);
    eprintln!(
        "[summary-live --interactive] launching review workbench: {}",
        input_dir.display()
    );
    run_inspect_live_tui(&summary, &governance, &report)?;
    Ok(summary.dashboard_count)
}

#[cfg(not(feature = "tui"))]
// Non-TUI path keeps command behavior explicit for --interactive usage.
fn run_interactive_inspect_live_tui_from_dir(_import_dir: &Path) -> Result<usize> {
    super::tui_not_built("summary-live --interactive")
}

pub(crate) fn inspect_live_dashboards_with_client(
    client: &JsonHttpClient,
    args: &InspectLiveArgs,
) -> Result<usize> {
    // Small or multi-org scans take the request-parity path; the threaded client path is
    // only for the single-org case where live fetch fan-out is safe and useful.
    if args.all_orgs || args.concurrency <= 1 {
        return inspect_live_dashboards_with_request(
            |method, path, params, payload| client.request_json(method, path, params, payload),
            args,
        );
    }

    let temp_dir = TempInspectDir::new("summary-live")?;
    let raw_dir = temp_dir.path.join(RAW_EXPORT_SUBDIR);
    let dashboard = DashboardResourceClient::new(client);
    let datasource = DatasourceResourceClient::new(client);
    let summaries = dashboard.list_dashboard_summaries(args.page_size)?;
    let datasource_items = datasource.list_datasources()?;
    let org_value = dashboard
        .fetch_current_org()
        .map(Value::Object)
        .unwrap_or_else(|_| serde_json::json!({"id": 1, "name": "Main Org."}));
    let org = value_as_object(&org_value, "Unexpected current org payload from Grafana.")?;
    let datasource_inventory = datasource_items
        .iter()
        .map(|item| build_datasource_inventory_record(item, org))
        .collect::<Vec<_>>();
    snapshot_live_dashboard_export_with_fetcher(
        &raw_dir,
        &summaries,
        args.concurrency,
        args.progress,
        |uid| dashboard.fetch_dashboard(uid),
    )?;
    write_json_document(
        &datasource_inventory,
        &raw_dir.join(DATASOURCE_INVENTORY_FILENAME),
    )?;
    let folder_inventory = collect_folder_inventory_from_summaries(&dashboard, &summaries)?;
    write_json_document(&folder_inventory, &raw_dir.join(FOLDER_INVENTORY_FILENAME))?;
    if args.interactive {
        return run_interactive_inspect_live_tui_from_dir(&raw_dir);
    }
    let inspect_args = build_export_inspect_args_from_live(args, raw_dir);
    analyze_export_dir(&inspect_args)
}

fn build_export_inspect_args_from_live(
    args: &InspectLiveArgs,
    input_dir: PathBuf,
) -> InspectExportArgs {
    InspectExportArgs {
        input_dir,
        input_type: None,
        input_format: super::DashboardImportInputFormat::Raw,
        text: args.text,
        csv: args.csv,
        json: args.json,
        table: args.table,
        yaml: args.yaml,
        output_format: args.output_format,
        output_file: args.output_file.clone(),
        also_stdout: args.also_stdout,
        report_columns: args.report_columns.clone(),
        list_columns: args.list_columns,
        report_filter_datasource: args.report_filter_datasource.clone(),
        report_filter_panel_id: args.report_filter_panel_id.clone(),
        help_full: args.help_full,
        no_header: args.no_header,
        interactive: args.interactive,
    }
}

pub(crate) fn inspect_live_dashboards_with_request<F>(
    mut request_json: F,
    args: &InspectLiveArgs,
) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    // Live inspect first stages a raw export tree, then reuses the offline analyzer so the
    // live and export code paths stay behaviorally aligned.
    let temp_dir = TempInspectDir::new("summary-live")?;
    let export_args = build_live_export_args(args, temp_dir.path.clone());
    let _ = export_dashboards_with_request(&mut request_json, &export_args)?;
    let inspect_import_dir = prepare_inspect_live_import_dir(&temp_dir.path, args)?;
    if args.interactive {
        return run_interactive_inspect_live_tui_from_dir(&inspect_import_dir);
    }
    let inspect_args = build_export_inspect_args_from_live(args, inspect_import_dir);
    analyze_export_dir(&inspect_args)
}
