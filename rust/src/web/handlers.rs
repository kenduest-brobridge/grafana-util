use axum::extract::State;
use axum::response::Html;
use axum::Json;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;

use super::contracts::{
    ActionDescriptor, ActionRunResponse, CapabilityDescriptor, CommonConnectionRequest,
    ConnectionTestResponse, DashboardBrowseRequest, DashboardInspectExportRequest,
    DashboardInspectLiveRequest, DashboardInspectVarsRequest, DashboardListRequest,
    OverviewStagedRequest, ProjectStatusLiveRequest, ProjectStatusStagedRequest,
    SyncBundlePreflightRequest, SyncPlanRequest, SyncPreflightRequest,
    SyncPromotionPreflightRequest, SyncSummaryRequest, WebError, WebState, WorkspaceDescriptor,
};
use super::page::html_page;
use super::registry::{action, capabilities, workspaces};
use crate::common::{message, string_field, Result};
use crate::dashboard::{
    build_auth_context, build_http_client, collect_dashboard_list_summaries,
    execute_dashboard_inspect_export, execute_dashboard_inspect_live,
    execute_dashboard_inspect_vars, execute_dashboard_list, CommonCliArgs, InspectExportArgs,
    InspectExportReportFormat, InspectLiveArgs, InspectOutputFormat, InspectVarsArgs, ListArgs,
    SimpleOutputFormat, DEFAULT_PAGE_SIZE, DEFAULT_TIMEOUT, DEFAULT_URL,
};
use crate::overview::{execute_overview, render_overview_text, OverviewArgs};
use crate::project_status_command::{
    execute_project_status_live, execute_project_status_staged, render_project_status_text,
    ProjectStatusLiveArgs, ProjectStatusStagedArgs,
};
use crate::sync::{
    execute_sync_command, SyncBundlePreflightArgs, SyncGroupCommand, SyncOutputFormat,
    SyncPlanArgs, SyncPreflightArgs, SyncPromotionPreflightArgs, SyncSummaryArgs,
};

fn parse_csv_list(input: Option<&str>) -> Vec<String> {
    input
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn optional_path(input: &Option<String>) -> Option<PathBuf> {
    input
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn required_path(input: &str, label: &str) -> Result<PathBuf> {
    let value = input.trim();
    if value.is_empty() {
        return Err(message(format!("{label} is required.")));
    }
    Ok(PathBuf::from(value))
}

fn normalize_url(input: Option<&str>) -> String {
    input
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_URL)
        .to_string()
}

fn optional_string(input: Option<&str>) -> Option<String> {
    input
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn common_cli_args_from_request(request: &CommonConnectionRequest) -> CommonCliArgs {
    CommonCliArgs {
        url: normalize_url(request.url.as_deref()),
        api_token: optional_string(request.api_token.as_deref()),
        username: optional_string(request.username.as_deref()),
        password: optional_string(request.password.as_deref()),
        prompt_password: false,
        prompt_token: false,
        timeout: request.timeout.unwrap_or(DEFAULT_TIMEOUT),
        verify_ssl: request.verify_ssl,
    }
}

pub(crate) fn test_connection(request: &CommonConnectionRequest) -> Result<ConnectionTestResponse> {
    let common = common_cli_args_from_request(request);
    let auth = build_auth_context(&common)?;
    let client = build_http_client(&common)?;
    let org_value = client
        .request_json(reqwest::Method::GET, "/api/org", &[], None)?
        .ok_or_else(|| message("Grafana did not return current-org metadata."))?;
    let org = org_value
        .as_object()
        .ok_or_else(|| message("Unexpected current-org payload from Grafana."))?;
    let org_id = org.get("id").and_then(Value::as_i64);
    let org_name = org.get("name").and_then(Value::as_str).map(str::to_string);
    let org_label = org_name
        .clone()
        .unwrap_or_else(|| "unknown org".to_string());
    Ok(ConnectionTestResponse {
        ok: true,
        url: common.url,
        auth_mode: auth.auth_mode,
        org_id,
        org_name,
        message: format!("Connected to {org_label}."),
    })
}

fn list_output_from_mode(mode: Option<&str>) -> Result<(bool, bool, Option<SimpleOutputFormat>)> {
    match mode.map(str::trim).filter(|value| !value.is_empty()) {
        None | Some("table") => Ok((true, false, None)),
        Some("csv") => Ok((false, true, None)),
        Some("json") => Ok((false, false, Some(SimpleOutputFormat::Json))),
        Some(other) => Err(message(format!(
            "Unsupported dashboard list outputMode {other:?}. Use table, csv, or json."
        ))),
    }
}

fn inspect_output_from_mode(
    mode: Option<&str>,
) -> Result<(
    bool,
    bool,
    Option<InspectExportReportFormat>,
    Option<InspectOutputFormat>,
)> {
    match mode.map(str::trim).filter(|value| !value.is_empty()) {
        None | Some("summary") => Ok((false, false, None, None)),
        Some("table") => Ok((false, true, None, None)),
        Some("json") => Ok((true, false, None, None)),
        Some("report-table") => Ok((false, false, None, Some(InspectOutputFormat::ReportTable))),
        Some("report-csv") => Ok((false, false, None, Some(InspectOutputFormat::ReportCsv))),
        Some("report-json") => Ok((false, false, None, Some(InspectOutputFormat::ReportJson))),
        Some("report-tree") => Ok((false, false, None, Some(InspectOutputFormat::ReportTree))),
        Some("report-tree-table") => Ok((
            false,
            false,
            None,
            Some(InspectOutputFormat::ReportTreeTable),
        )),
        Some("dependency") => Ok((
            false,
            false,
            Some(InspectExportReportFormat::Dependency),
            None,
        )),
        Some("dependency-json") => Ok((
            false,
            false,
            Some(InspectExportReportFormat::DependencyJson),
            None,
        )),
        Some("governance") => Ok((false, false, None, Some(InspectOutputFormat::Governance))),
        Some("governance-json") => {
            Ok((false, false, None, Some(InspectOutputFormat::GovernanceJson)))
        }
        Some(other) => Err(message(format!(
            "Unsupported inspect outputMode {other:?}. Use summary, table, json, report-table, report-csv, report-json, report-tree, report-tree-table, dependency, dependency-json, governance, or governance-json."
        ))),
    }
}

fn variable_output_from_mode(mode: Option<&str>) -> Result<Option<SimpleOutputFormat>> {
    match mode.map(str::trim).filter(|value| !value.is_empty()) {
        None | Some("table") => Ok(None),
        Some("csv") => Ok(Some(SimpleOutputFormat::Csv)),
        Some("json") => Ok(Some(SimpleOutputFormat::Json)),
        Some(other) => Err(message(format!(
            "Unsupported inspect-vars outputMode {other:?}. Use table, csv, or json."
        ))),
    }
}

fn action_response(
    descriptor: ActionDescriptor,
    text_lines: Vec<String>,
    document: Value,
) -> Json<ActionRunResponse> {
    Json(ActionRunResponse {
        action: descriptor.id,
        title: descriptor.title,
        ui_mode: descriptor.ui_mode,
        read_only: descriptor.read_only,
        text_lines,
        document,
    })
}

fn dashboard_row_value(summary: &Map<String, Value>, key: &str) -> String {
    string_field(summary, key, "")
}

fn dashboard_row_array(summary: &Map<String, Value>, key: &str) -> Vec<String> {
    summary
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<String>>()
        })
        .unwrap_or_default()
}

fn dashboard_row_document(summary: &Map<String, Value>) -> Value {
    json!({
        "uid": dashboard_row_value(summary, "uid"),
        "title": dashboard_row_value(summary, "title"),
        "folderTitle": dashboard_row_value(summary, "folderTitle"),
        "folderUid": dashboard_row_value(summary, "folderUid"),
        "folderPath": dashboard_row_value(summary, "folderPath"),
        "orgName": dashboard_row_value(summary, "orgName"),
        "orgId": summary.get("orgId").cloned().unwrap_or(Value::Null),
        "sources": dashboard_row_array(summary, "sources"),
        "sourceUids": dashboard_row_array(summary, "sourceUids"),
    })
}

pub(crate) fn build_dashboard_browse_document(
    summaries: &[Map<String, Value>],
    query: Option<&str>,
    path: Option<&str>,
    selected_uid: Option<&str>,
) -> Value {
    let query = query.map(str::trim).filter(|value| !value.is_empty());
    let path = path.map(str::trim).filter(|value| !value.is_empty());
    let rows = summaries
        .iter()
        .filter(|summary| {
            let folder_path = dashboard_row_value(summary, "folderPath");
            let title = dashboard_row_value(summary, "title");
            let uid = dashboard_row_value(summary, "uid");
            let org_name = dashboard_row_value(summary, "orgName");
            let sources = dashboard_row_array(summary, "sources").join(" ");
            let query_ok = query
                .map(|value| {
                    let value = value.to_ascii_lowercase();
                    [
                        uid.to_ascii_lowercase(),
                        title.to_ascii_lowercase(),
                        folder_path.to_ascii_lowercase(),
                        org_name.to_ascii_lowercase(),
                        sources.to_ascii_lowercase(),
                    ]
                    .iter()
                    .any(|candidate| candidate.contains(&value))
                })
                .unwrap_or(true);
            let path_ok = path
                .map(|value| folder_path == value || folder_path.starts_with(&format!("{value} /")))
                .unwrap_or(true);
            query_ok && path_ok
        })
        .map(dashboard_row_document)
        .collect::<Vec<Value>>();

    let selected_uid = selected_uid
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            rows.first()
                .and_then(|row| row.get("uid"))
                .and_then(Value::as_str)
                .map(str::to_string)
        });

    let detail = selected_uid.as_deref().and_then(|uid| {
        rows.iter()
            .find(|row| row.get("uid").and_then(Value::as_str) == Some(uid))
            .cloned()
    });

    let mut orgs = BTreeMap::<String, BTreeMap<String, usize>>::new();
    for row in &rows {
        let org_name = row
            .get("orgName")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or("Current Org")
            .to_string();
        let folder_path = row
            .get("folderPath")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or("General")
            .to_string();
        *orgs
            .entry(org_name)
            .or_default()
            .entry(folder_path)
            .or_insert(0) += 1;
    }

    let tree = orgs
        .into_iter()
        .map(|(org_name, folders)| {
            json!({
                "id": org_name,
                "label": org_name,
                "dashboardCount": folders.values().sum::<usize>(),
                "children": folders
                    .into_iter()
                    .map(|(folder_path, dashboard_count)| {
                        json!({
                            "id": format!("{org_name}::{folder_path}"),
                            "label": folder_path,
                            "folderPath": folder_path,
                            "dashboardCount": dashboard_count,
                        })
                    })
                    .collect::<Vec<Value>>()
            })
        })
        .collect::<Vec<Value>>();

    json!({
        "kind": "grafana-utils-dashboard-browser",
        "summary": {
            "dashboardCount": rows.len(),
            "orgCount": tree.len(),
            "folderCount": tree.iter().map(|org| {
                org.get("children")
                    .and_then(Value::as_array)
                    .map(|items| items.len())
                    .unwrap_or(0)
            }).sum::<usize>(),
            "selectedUid": selected_uid,
        },
        "filters": {
            "query": query,
            "path": path,
        },
        "tree": tree,
        "rows": rows,
        "detail": detail,
    })
}

pub(crate) async fn index_handler(State(_state): State<WebState>) -> Html<&'static str> {
    Html(html_page())
}

pub(crate) async fn workspaces_handler() -> Json<Vec<WorkspaceDescriptor>> {
    Json(workspaces())
}

pub(crate) async fn capabilities_handler() -> Json<Vec<CapabilityDescriptor>> {
    Json(capabilities())
}

pub(crate) async fn test_connection_handler(
    State(_state): State<WebState>,
    Json(request): Json<CommonConnectionRequest>,
) -> std::result::Result<Json<ConnectionTestResponse>, WebError> {
    Ok(Json(test_connection(&request)?))
}

pub(crate) async fn run_dashboard_browse_handler(
    State(_state): State<WebState>,
    Json(request): Json<DashboardBrowseRequest>,
) -> std::result::Result<Json<ActionRunResponse>, WebError> {
    let descriptor = action("dashboard-browse");
    let common = common_cli_args_from_request(&CommonConnectionRequest {
        url: request.url.clone(),
        api_token: request.api_token.clone(),
        username: request.username.clone(),
        password: request.password.clone(),
        timeout: request.timeout,
        verify_ssl: request.verify_ssl,
    });
    let list_args = ListArgs {
        common,
        page_size: request.page_size.unwrap_or(DEFAULT_PAGE_SIZE),
        org_id: request.org_id,
        all_orgs: request.all_orgs,
        with_sources: request.with_sources,
        output_columns: Vec::new(),
        table: true,
        csv: false,
        json: false,
        output_format: None,
        no_header: false,
    };
    let summaries = collect_dashboard_list_summaries(&list_args)?;
    let document = build_dashboard_browse_document(
        &summaries,
        request.query.as_deref(),
        request.path.as_deref(),
        request.selected_uid.as_deref(),
    );
    let text_lines = vec![
        format!(
            "Dashboard browse rows={} orgs={} folders={}",
            document["summary"]["dashboardCount"].as_u64().unwrap_or(0),
            document["summary"]["orgCount"].as_u64().unwrap_or(0),
            document["summary"]["folderCount"].as_u64().unwrap_or(0)
        ),
        format!(
            "Filters: query={} path={}",
            request.query.clone().unwrap_or_default(),
            request.path.clone().unwrap_or_default()
        ),
    ];
    Ok(action_response(descriptor, text_lines, document))
}

pub(crate) async fn run_dashboard_list_handler(
    State(_state): State<WebState>,
    Json(request): Json<DashboardListRequest>,
) -> std::result::Result<Json<ActionRunResponse>, WebError> {
    let descriptor = action("dashboard-list");
    let common = common_cli_args_from_request(&CommonConnectionRequest {
        url: request.url.clone(),
        api_token: request.api_token.clone(),
        username: request.username.clone(),
        password: request.password.clone(),
        timeout: request.timeout,
        verify_ssl: request.verify_ssl,
    });
    let (table, csv, output_format) = list_output_from_mode(request.output_mode.as_deref())?;
    let args = ListArgs {
        common,
        page_size: request.page_size.unwrap_or(DEFAULT_PAGE_SIZE),
        org_id: request.org_id,
        all_orgs: request.all_orgs,
        with_sources: request.with_sources,
        output_columns: parse_csv_list(request.output_columns.as_deref()),
        table,
        csv,
        json: matches!(output_format, Some(SimpleOutputFormat::Json)),
        output_format,
        no_header: false,
    };
    let output = execute_dashboard_list(&args)?;
    Ok(action_response(
        descriptor,
        output.text_lines,
        output.document,
    ))
}

pub(crate) async fn run_dashboard_inspect_live_handler(
    State(_state): State<WebState>,
    Json(request): Json<DashboardInspectLiveRequest>,
) -> std::result::Result<Json<ActionRunResponse>, WebError> {
    let descriptor = action("dashboard-inspect-live");
    let common = common_cli_args_from_request(&CommonConnectionRequest {
        url: request.url.clone(),
        api_token: request.api_token.clone(),
        username: request.username.clone(),
        password: request.password.clone(),
        timeout: request.timeout,
        verify_ssl: request.verify_ssl,
    });
    let (json, table, report, output_format) =
        inspect_output_from_mode(request.output_mode.as_deref())?;
    let args = InspectLiveArgs {
        common,
        page_size: request.page_size.unwrap_or(DEFAULT_PAGE_SIZE),
        concurrency: request.concurrency.unwrap_or(8),
        org_id: request.org_id,
        all_orgs: request.all_orgs,
        json,
        table,
        report,
        output_format,
        report_columns: parse_csv_list(request.report_columns.as_deref()),
        report_filter_datasource: optional_string(request.report_filter_datasource.as_deref()),
        report_filter_panel_id: optional_string(request.report_filter_panel_id.as_deref()),
        progress: request.progress,
        help_full: false,
        no_header: false,
        output_file: None,
        interactive: false,
    };
    let output = execute_dashboard_inspect_live(&args)?;
    Ok(action_response(
        descriptor,
        output.text_lines,
        output.document,
    ))
}

pub(crate) async fn run_dashboard_inspect_export_handler(
    State(_state): State<WebState>,
    Json(request): Json<DashboardInspectExportRequest>,
) -> std::result::Result<Json<ActionRunResponse>, WebError> {
    let descriptor = action("dashboard-inspect-export");
    let (json, table, report, output_format) =
        inspect_output_from_mode(request.output_mode.as_deref())?;
    let args = InspectExportArgs {
        import_dir: required_path(&request.import_dir, "importDir")?,
        json,
        table,
        report,
        output_format,
        report_columns: parse_csv_list(request.report_columns.as_deref()),
        report_filter_datasource: optional_string(request.report_filter_datasource.as_deref()),
        report_filter_panel_id: optional_string(request.report_filter_panel_id.as_deref()),
        help_full: false,
        no_header: false,
        output_file: None,
        interactive: false,
    };
    let output = execute_dashboard_inspect_export(&args)?;
    Ok(action_response(
        descriptor,
        output.text_lines,
        output.document,
    ))
}

pub(crate) async fn run_dashboard_inspect_vars_handler(
    State(_state): State<WebState>,
    Json(request): Json<DashboardInspectVarsRequest>,
) -> std::result::Result<Json<ActionRunResponse>, WebError> {
    let descriptor = action("dashboard-inspect-vars");
    let common = common_cli_args_from_request(&CommonConnectionRequest {
        url: request.url.clone(),
        api_token: request.api_token.clone(),
        username: request.username.clone(),
        password: request.password.clone(),
        timeout: request.timeout,
        verify_ssl: request.verify_ssl,
    });
    let args = InspectVarsArgs {
        common,
        dashboard_uid: optional_string(request.dashboard_uid.as_deref()),
        dashboard_url: optional_string(request.dashboard_url.as_deref()),
        vars_query: optional_string(request.vars_query.as_deref()),
        org_id: request.org_id,
        output_format: variable_output_from_mode(request.output_mode.as_deref())?,
        no_header: false,
        output_file: None,
    };
    let output = execute_dashboard_inspect_vars(&args)?;
    Ok(action_response(
        descriptor,
        output.text_lines,
        output.document,
    ))
}

pub(crate) async fn run_overview_staged_handler(
    State(_state): State<WebState>,
    Json(request): Json<OverviewStagedRequest>,
) -> std::result::Result<Json<ActionRunResponse>, WebError> {
    let descriptor = action("overview-staged");
    let args = OverviewArgs {
        dashboard_export_dir: optional_path(&request.dashboard_export_dir),
        datasource_export_dir: optional_path(&request.datasource_export_dir),
        access_user_export_dir: optional_path(&request.access_user_export_dir),
        access_team_export_dir: optional_path(&request.access_team_export_dir),
        access_org_export_dir: optional_path(&request.access_org_export_dir),
        access_service_account_export_dir: optional_path(
            &request.access_service_account_export_dir,
        ),
        desired_file: optional_path(&request.desired_file),
        source_bundle: optional_path(&request.source_bundle),
        target_inventory: optional_path(&request.target_inventory),
        alert_export_dir: optional_path(&request.alert_export_dir),
        availability_file: optional_path(&request.availability_file),
        mapping_file: optional_path(&request.mapping_file),
        output: crate::overview::OverviewOutputFormat::Json,
    };
    let document = execute_overview(&args)?;
    let text_lines = render_overview_text(&document)?;
    Ok(action_response(
        descriptor,
        text_lines,
        serde_json::to_value(document)?,
    ))
}

pub(crate) async fn run_project_status_staged_handler(
    State(_state): State<WebState>,
    Json(request): Json<ProjectStatusStagedRequest>,
) -> std::result::Result<Json<ActionRunResponse>, WebError> {
    let descriptor = action("project-status-staged");
    let args = ProjectStatusStagedArgs {
        dashboard_export_dir: optional_path(&request.dashboard_export_dir),
        datasource_export_dir: optional_path(&request.datasource_export_dir),
        access_user_export_dir: optional_path(&request.access_user_export_dir),
        access_team_export_dir: optional_path(&request.access_team_export_dir),
        access_org_export_dir: optional_path(&request.access_org_export_dir),
        access_service_account_export_dir: optional_path(
            &request.access_service_account_export_dir,
        ),
        desired_file: optional_path(&request.desired_file),
        source_bundle: optional_path(&request.source_bundle),
        target_inventory: optional_path(&request.target_inventory),
        alert_export_dir: optional_path(&request.alert_export_dir),
        availability_file: optional_path(&request.availability_file),
        mapping_file: optional_path(&request.mapping_file),
        output: crate::project_status_command::ProjectStatusOutputFormat::Json,
    };
    let status = execute_project_status_staged(&args)?;
    Ok(action_response(
        descriptor,
        render_project_status_text(&status),
        serde_json::to_value(status)?,
    ))
}

pub(crate) async fn run_project_status_live_handler(
    State(_state): State<WebState>,
    Json(request): Json<ProjectStatusLiveRequest>,
) -> std::result::Result<Json<ActionRunResponse>, WebError> {
    let descriptor = action("project-status-live");
    let args = ProjectStatusLiveArgs {
        url: normalize_url(request.url.as_deref()),
        api_token: optional_string(request.api_token.as_deref()),
        username: optional_string(request.username.as_deref()),
        password: optional_string(request.password.as_deref()),
        prompt_password: false,
        prompt_token: false,
        timeout: request.timeout.unwrap_or(DEFAULT_TIMEOUT),
        verify_ssl: request.verify_ssl,
        insecure: request.insecure,
        ca_cert: optional_path(&request.ca_cert),
        all_orgs: request.all_orgs,
        org_id: request.org_id,
        sync_summary_file: optional_path(&request.sync_summary_file),
        bundle_preflight_file: optional_path(&request.bundle_preflight_file),
        promotion_summary_file: optional_path(&request.promotion_summary_file),
        mapping_file: optional_path(&request.mapping_file),
        availability_file: optional_path(&request.availability_file),
        output: crate::project_status_command::ProjectStatusOutputFormat::Json,
    };
    let status = execute_project_status_live(&args)?;
    Ok(action_response(
        descriptor,
        render_project_status_text(&status),
        serde_json::to_value(status)?,
    ))
}

pub(crate) async fn run_sync_summary_handler(
    State(_state): State<WebState>,
    Json(request): Json<SyncSummaryRequest>,
) -> std::result::Result<Json<ActionRunResponse>, WebError> {
    let descriptor = action("sync-summary");
    let command = SyncGroupCommand::Summary(SyncSummaryArgs {
        desired_file: required_path(&request.desired_file, "desiredFile")?,
        output: SyncOutputFormat::Json,
    });
    let output = execute_sync_command(&command)?;
    Ok(action_response(
        descriptor,
        output.text_lines,
        output.document,
    ))
}

pub(crate) async fn run_sync_plan_handler(
    State(_state): State<WebState>,
    Json(request): Json<SyncPlanRequest>,
) -> std::result::Result<Json<ActionRunResponse>, WebError> {
    let descriptor = action("sync-plan");
    let common = common_cli_args_from_request(&CommonConnectionRequest {
        url: request.url.clone(),
        api_token: request.api_token.clone(),
        username: request.username.clone(),
        password: request.password.clone(),
        timeout: request.timeout,
        verify_ssl: request.verify_ssl,
    });
    let command = SyncGroupCommand::Plan(SyncPlanArgs {
        desired_file: required_path(&request.desired_file, "desiredFile")?,
        live_file: optional_path(&request.live_file),
        fetch_live: request.fetch_live,
        common,
        org_id: request.org_id,
        page_size: request.page_size.unwrap_or(DEFAULT_PAGE_SIZE),
        allow_prune: request.allow_prune,
        output: SyncOutputFormat::Json,
        trace_id: optional_string(request.trace_id.as_deref()),
    });
    let output = execute_sync_command(&command)?;
    Ok(action_response(
        descriptor,
        output.text_lines,
        output.document,
    ))
}

pub(crate) async fn run_sync_preflight_handler(
    State(_state): State<WebState>,
    Json(request): Json<SyncPreflightRequest>,
) -> std::result::Result<Json<ActionRunResponse>, WebError> {
    let descriptor = action("sync-preflight");
    let common = common_cli_args_from_request(&CommonConnectionRequest {
        url: request.url.clone(),
        api_token: request.api_token.clone(),
        username: request.username.clone(),
        password: request.password.clone(),
        timeout: request.timeout,
        verify_ssl: request.verify_ssl,
    });
    let command = SyncGroupCommand::Preflight(SyncPreflightArgs {
        desired_file: required_path(&request.desired_file, "desiredFile")?,
        availability_file: optional_path(&request.availability_file),
        fetch_live: request.fetch_live,
        common,
        org_id: request.org_id,
        output: SyncOutputFormat::Json,
    });
    let output = execute_sync_command(&command)?;
    Ok(action_response(
        descriptor,
        output.text_lines,
        output.document,
    ))
}

pub(crate) async fn run_sync_bundle_preflight_handler(
    State(_state): State<WebState>,
    Json(request): Json<SyncBundlePreflightRequest>,
) -> std::result::Result<Json<ActionRunResponse>, WebError> {
    let descriptor = action("sync-bundle-preflight");
    let common = common_cli_args_from_request(&CommonConnectionRequest {
        url: request.url.clone(),
        api_token: request.api_token.clone(),
        username: request.username.clone(),
        password: request.password.clone(),
        timeout: request.timeout,
        verify_ssl: request.verify_ssl,
    });
    let command = SyncGroupCommand::BundlePreflight(SyncBundlePreflightArgs {
        source_bundle: required_path(&request.source_bundle, "sourceBundle")?,
        target_inventory: required_path(&request.target_inventory, "targetInventory")?,
        availability_file: optional_path(&request.availability_file),
        fetch_live: request.fetch_live,
        common,
        org_id: request.org_id,
        output: SyncOutputFormat::Json,
    });
    let output = execute_sync_command(&command)?;
    Ok(action_response(
        descriptor,
        output.text_lines,
        output.document,
    ))
}

pub(crate) async fn run_sync_promotion_preflight_handler(
    State(_state): State<WebState>,
    Json(request): Json<SyncPromotionPreflightRequest>,
) -> std::result::Result<Json<ActionRunResponse>, WebError> {
    let descriptor = action("sync-promotion-preflight");
    let common = common_cli_args_from_request(&CommonConnectionRequest {
        url: request.url.clone(),
        api_token: request.api_token.clone(),
        username: request.username.clone(),
        password: request.password.clone(),
        timeout: request.timeout,
        verify_ssl: request.verify_ssl,
    });
    let command = SyncGroupCommand::PromotionPreflight(SyncPromotionPreflightArgs {
        source_bundle: required_path(&request.source_bundle, "sourceBundle")?,
        target_inventory: required_path(&request.target_inventory, "targetInventory")?,
        mapping_file: optional_path(&request.mapping_file),
        availability_file: optional_path(&request.availability_file),
        fetch_live: request.fetch_live,
        common,
        org_id: request.org_id,
        output: SyncOutputFormat::Json,
    });
    let output = execute_sync_command(&command)?;
    Ok(action_response(
        descriptor,
        output.text_lines,
        output.document,
    ))
}
