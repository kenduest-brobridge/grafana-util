//! Dashboard domain test suite.
//! Covers parser surfaces, formatter/output contracts, and export/import/inspect/list/diff
//! behavior with in-memory/mocked request fixtures.
#![allow(unused_imports)]

use super::test_support;
use super::test_support::{
    attach_dashboard_folder_paths_with_request, build_export_metadata,
    build_export_variant_dirs, build_external_export_document, build_folder_inventory_status,
    build_folder_path, build_governance_gate_tui_groups, build_governance_gate_tui_items,
    build_impact_browser_items, build_impact_document, build_impact_tui_groups,
    build_import_auth_context, build_import_payload, build_output_path,
    build_preserved_web_import_document, build_root_export_index, build_topology_document,
    build_topology_tui_groups, diff_dashboards_with_request, discover_dashboard_files,
    export_dashboards_with_request, extract_dashboard_variables, filter_impact_tui_items,
    filter_topology_tui_items, format_dashboard_summary_line, format_export_progress_line,
    format_export_verbose_line, format_folder_inventory_status_line, format_import_progress_line,
    format_import_verbose_line, import_dashboards_with_org_clients, import_dashboards_with_request,
    list_dashboards_with_request, parse_cli_from,
    render_dashboard_governance_gate_result, render_dashboard_summary_csv,
    render_dashboard_summary_json, render_dashboard_summary_table, render_impact_text,
    render_import_dry_run_json, render_import_dry_run_table, render_topology_dot,
    render_topology_mermaid, CommonCliArgs,
    DashboardCliArgs, DashboardCommand, DashboardGovernanceGateFinding,
    DashboardGovernanceGateResult, DashboardGovernanceGateSummary, DiffArgs, ExportArgs,
    FolderInventoryStatusKind, GovernanceGateArgs, GovernanceGateOutputFormat, ImpactAlertResource,
    ImpactDashboard, ImpactDocument, ImpactOutputFormat, ImpactSummary, ImportArgs,
    InspectExportArgs, InspectExportReportFormat, InspectLiveArgs, InspectOutputFormat, ListArgs,
    SimpleOutputFormat, TopologyDocument, TopologyOutputFormat, ValidationOutputFormat,
    DASHBOARD_PERMISSION_BUNDLE_FILENAME, DATASOURCE_INVENTORY_FILENAME, EXPORT_METADATA_FILENAME,
    FOLDER_INVENTORY_FILENAME, TOOL_SCHEMA_VERSION,
};
use crate::common::api_response;
use crate::dashboard::inspect::{
    dispatch_query_analysis, extract_query_field_and_text, resolve_query_analyzer_family,
    QueryAnalysis, QueryExtractionContext,
};
use crate::dashboard::inspect_governance::governance_risk_spec;
use clap::{CommandFactory, Parser};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

type TestRequestResult = crate::common::Result<Option<Value>>;

fn make_common_args(base_url: String) -> CommonCliArgs {
    CommonCliArgs {
        url: base_url,
        api_token: Some("token".to_string()),
        username: None,
        password: None,
        prompt_password: false,
        prompt_token: false,
        timeout: 30,
        verify_ssl: false,
    }
}

fn make_basic_common_args(base_url: String) -> CommonCliArgs {
    CommonCliArgs {
        url: base_url,
        api_token: None,
        username: Some("admin".to_string()),
        password: Some("admin".to_string()),
        prompt_password: false,
        prompt_token: false,
        timeout: 30,
        verify_ssl: false,
    }
}

fn load_prompt_export_cases() -> Vec<Value> {
    serde_json::from_str(include_str!(
        "../../../fixtures/dashboard_prompt_export_cases.json"
    ))
    .unwrap()
}

fn load_inspection_analyzer_cases() -> Vec<Value> {
    serde_json::from_str(include_str!(
        "../../../fixtures/dashboard_inspection_analyzer_cases.json"
    ))
    .unwrap()
}

fn sample_topology_tui_document() -> TopologyDocument {
    let governance = json!({
        "dashboardGovernance": [
            {
                "dashboardUid": "cpu-main",
                "dashboardTitle": "CPU Main"
            }
        ],
        "dashboardDatasourceEdges": [
            {
                "dashboardUid": "cpu-main",
                "dashboardTitle": "CPU Main",
                "datasourceUid": "prom-main",
                "datasource": "Prometheus Main",
                "panelCount": 1,
                "queryCount": 1,
                "queryFields": ["expr"],
                "queryVariables": ["cluster"],
                "metrics": ["up"],
                "functions": [],
                "measurements": [],
                "buckets": []
            }
        ],
        "dashboardDependencies": [
            {
                "dashboardUid": "cpu-main",
                "panelIds": ["7"],
                "panelVariables": ["cluster"],
                "queryVariables": ["cluster"]
            }
        ]
    });
    let alert_contract = json!({
        "kind": "grafana-utils-sync-alert-contract",
        "resources": [
            {
                "kind": "grafana-alert-rule",
                "identity": "cpu-high",
                "title": "CPU High",
                "references": ["prom-main", "cpu-main"]
            }
        ]
    });

    build_topology_document(&governance, Some(&alert_contract)).unwrap()
}

#[allow(clippy::type_complexity)]
fn with_dashboard_import_live_preflight<F>(
    preflight_datasources: Value,
    preflight_plugins: Value,
    mut handler: F,
) -> impl FnMut(reqwest::Method, &str, &[(String, String)], Option<&Value>) -> TestRequestResult
where
    F: FnMut(reqwest::Method, &str, &[(String, String)], Option<&Value>) -> TestRequestResult,
{
    move |method, path, params, payload| {
        if method == reqwest::Method::GET && path == "/api/datasources" {
            return Ok(Some(preflight_datasources.clone()));
        }
        if method == reqwest::Method::GET && path == "/api/plugins" {
            return Ok(Some(preflight_plugins.clone()));
        }
        if method == reqwest::Method::GET && path == "/api/search" {
            return Ok(Some(json!([])));
        }
        handler(method, path, params, payload)
    }
}

fn make_import_args(import_dir: PathBuf) -> ImportArgs {
    ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir,
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: false,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: true,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    }
}

#[allow(clippy::too_many_arguments)]
fn write_basic_raw_export(
    raw_dir: &Path,
    org_id: &str,
    org_name: &str,
    dashboard_uid: &str,
    dashboard_title: &str,
    datasource_uid: &str,
    datasource_type: &str,
    panel_type: &str,
    folder_uid: &str,
    folder_title: &str,
    query_field: &str,
    query_text: &str,
) {
    fs::create_dir_all(raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid",
            "foldersFile": FOLDER_INVENTORY_FILENAME,
            "datasourcesFile": DATASOURCE_INVENTORY_FILENAME,
            "org": org_name,
            "orgId": org_id
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join(FOLDER_INVENTORY_FILENAME),
        serde_json::to_string_pretty(&json!([
            {
                "uid": folder_uid,
                "title": folder_title,
                "path": folder_title,
                "org": org_name,
                "orgId": org_id
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join(DATASOURCE_INVENTORY_FILENAME),
        serde_json::to_string_pretty(&json!([
            {
                "uid": datasource_uid,
                "name": datasource_uid,
                "type": datasource_type,
                "access": "proxy",
                "url": "http://grafana.example.internal",
                "isDefault": "true",
                "org": org_name,
                "orgId": org_id
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("index.json"),
        serde_json::to_string_pretty(&json!([
            {
                "uid": dashboard_uid,
                "title": dashboard_title,
                "path": "dash.json",
                "format": "grafana-web-import-preserve-uid",
                "org": org_name,
                "orgId": org_id
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "id": null,
                "uid": dashboard_uid,
                "title": dashboard_title,
                "schemaVersion": 38,
                "panels": [{
                    "id": 7,
                    "title": dashboard_title,
                    "type": panel_type,
                    "datasource": {"uid": datasource_uid, "type": datasource_type},
                    "targets": [{
                        "refId": "A",
                        query_field: query_text
                    }]
                }]
            },
            "meta": {
                "folderUid": folder_uid,
                "folderTitle": folder_title
            }
        }))
        .unwrap(),
    )
    .unwrap();
}

fn write_combined_export_root_metadata(export_root: &Path, orgs: &[(&str, &str, &str)]) {
    fs::create_dir_all(export_root).unwrap();
    let org_entries: Vec<Value> = orgs
        .iter()
        .map(|(org_id, org_name, export_dir)| {
            json!({
                "org": org_name,
                "orgId": org_id,
                "dashboardCount": 1,
                "exportDir": export_dir
            })
        })
        .collect();
    fs::write(
        export_root.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "root",
            "dashboardCount": orgs.len(),
            "indexFile": "index.json",
            "orgCount": orgs.len(),
            "orgs": org_entries
        }))
        .unwrap(),
    )
    .unwrap();
}

fn read_json_output_file(path: &Path) -> Value {
    let raw = fs::read_to_string(path).unwrap();
    assert!(
        raw.ends_with('\n'),
        "expected output file {} to end with a newline",
        path.display()
    );
    serde_json::from_str(&raw).unwrap()
}

fn json_query_report_row<'a>(document: &'a Value, ref_id: &str) -> &'a Value {
    document["queries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["refId"] == Value::String(ref_id.to_string()))
        .unwrap()
}

fn assert_json_query_report_row_parity(
    export_document: &Value,
    live_document: &Value,
    ref_id: &str,
) {
    let export_row = json_query_report_row(export_document, ref_id);
    let live_row = json_query_report_row(live_document, ref_id);
    for field in [
        "org",
        "orgId",
        "dashboardUid",
        "dashboardTitle",
        "dashboardTags",
        "folderPath",
        "folderFullPath",
        "folderLevel",
        "folderUid",
        "parentFolderUid",
        "panelId",
        "panelTitle",
        "panelType",
        "panelTargetCount",
        "panelQueryCount",
        "panelDatasourceCount",
        "panelVariables",
        "refId",
        "datasource",
        "datasourceName",
        "datasourceUid",
        "datasourceType",
        "datasourceFamily",
        "queryField",
        "targetHidden",
        "targetDisabled",
        "queryVariables",
        "metrics",
        "functions",
        "measurements",
        "buckets",
        "query",
    ] {
        assert_eq!(
            export_row[field], live_row[field],
            "field={field}, refId={ref_id}"
        );
    }
}

fn normalize_governance_document_for_compare(document: &Value) -> Value {
    let mut normalized = document.clone();
    if let Some(rows) = normalized
        .get_mut("dashboardDependencies")
        .and_then(|value| value.as_array_mut())
    {
        for row in rows {
            if let Some(object) = row.as_object_mut() {
                object.remove("file");
            }
        }
    }
    normalized
}

fn normalize_queries_document_for_compare(document: &Value) -> Value {
    let mut normalized = document.clone();
    if let Some(rows) = normalized
        .get_mut("queries")
        .and_then(|value| value.as_array_mut())
    {
        for row in rows {
            if let Some(object) = row.as_object_mut() {
                object.remove("file");
                object.remove("datasourceOrg");
                object.remove("datasourceOrgId");
                object.remove("datasourceDatabase");
                object.remove("datasourceBucket");
                object.remove("datasourceOrganization");
                object.remove("datasourceIndexPattern");
            }
        }
    }
    normalized
}

fn assert_governance_documents_match(export_document: &Value, live_document: &Value) {
    assert_eq!(
        normalize_governance_document_for_compare(export_document),
        normalize_governance_document_for_compare(live_document)
    );
}

fn assert_all_orgs_export_live_documents_match(
    export_report_document: &Value,
    live_report_document: &Value,
    export_dependency_document: &Value,
    live_dependency_document: &Value,
    export_governance_document: &Value,
    live_governance_document: &Value,
) {
    assert_eq!(
        normalize_queries_document_for_compare(export_report_document),
        normalize_queries_document_for_compare(live_report_document)
    );
    assert_eq!(
        normalize_queries_document_for_compare(export_dependency_document),
        normalize_queries_document_for_compare(live_dependency_document)
    );
    assert_governance_documents_match(export_governance_document, live_governance_document);
}

#[allow(clippy::type_complexity)]
fn core_family_inspect_live_request_fixture(
    datasource_inventory: Value,
    dashboard_payload: Value,
) -> impl FnMut(reqwest::Method, &str, &[(String, String)], Option<&Value>) -> TestRequestResult {
    move |method, path, params, _payload| {
        let method_name = method.to_string();
        match (method, path) {
            (reqwest::Method::GET, "/api/org") => Ok(Some(json!({
                "id": 1,
                "name": "Main Org."
            }))),
            (reqwest::Method::GET, "/api/datasources") => Ok(Some(datasource_inventory.clone())),
            (reqwest::Method::GET, "/api/search") => Ok(Some(json!([
                {
                    "uid": "core-main",
                    "title": "Core Main",
                    "type": "dash-db",
                    "folderUid": "general",
                    "folderTitle": "General"
                }
            ]))),
            (reqwest::Method::GET, "/api/folders/general") => Ok(Some(json!({
                "uid": "general",
                "title": "General"
            }))),
            (reqwest::Method::GET, "/api/folders/general/permissions") => Ok(Some(json!([]))),
            (reqwest::Method::GET, "/api/dashboards/uid/core-main") => {
                Ok(Some(dashboard_payload.clone()))
            }
            (reqwest::Method::GET, "/api/dashboards/uid/core-main/permissions") => {
                Ok(Some(json!([])))
            }
            _ => Err(test_support::message(format!(
                "unexpected request {method_name} {path} {params:?}"
            ))),
        }
    }
}

#[allow(clippy::type_complexity)]
fn all_orgs_inspect_live_request_fixture(
) -> impl FnMut(reqwest::Method, &str, &[(String, String)], Option<&Value>) -> TestRequestResult {
    move |method, path, params, _payload| {
        let method_name = method.to_string();
        match (method, path) {
            (reqwest::Method::GET, "/api/orgs") => Ok(Some(json!([
                {"id": 1, "name": "Main Org."},
                {"id": 2, "name": "Ops Org"}
            ]))),
            (reqwest::Method::GET, "/api/org") => {
                let scoped_org = params
                    .iter()
                    .find(|(key, _)| key == "orgId")
                    .map(|(_, value)| value.as_str())
                    .unwrap_or("1");
                match scoped_org {
                    "1" => Ok(Some(json!({"id": 1, "name": "Main Org."}))),
                    "2" => Ok(Some(json!({"id": 2, "name": "Ops Org"}))),
                    other => panic!("unexpected org context {other}"),
                }
            }
            (reqwest::Method::GET, "/api/datasources") => {
                let scoped_org = params
                    .iter()
                    .find(|(key, _)| key == "orgId")
                    .map(|(_, value)| value.as_str())
                    .unwrap_or("1");
                match scoped_org {
                    "1" => Ok(Some(json!([
                        {
                            "uid": "prom-main",
                            "name": "Prometheus Main",
                            "type": "prometheus",
                            "access": "proxy",
                            "url": "http://prometheus:9090",
                            "isDefault": true
                        }
                    ]))),
                    "2" => Ok(Some(json!([
                        {
                            "uid": "prom-two",
                            "name": "Prometheus Two",
                            "type": "prometheus",
                            "access": "proxy",
                            "url": "http://prometheus-two:9090",
                            "isDefault": true
                        }
                    ]))),
                    other => panic!("unexpected org context {other}"),
                }
            }
            (reqwest::Method::GET, "/api/search") => {
                let scoped_org = params
                    .iter()
                    .find(|(key, _)| key == "orgId")
                    .map(|(_, value)| value.as_str())
                    .unwrap_or("1");
                match scoped_org {
                    "1" => Ok(Some(json!([
                        {
                            "uid": "cpu-main",
                            "title": "CPU Main",
                            "type": "dash-db",
                            "folderUid": "general",
                            "folderTitle": "General"
                        }
                    ]))),
                    "2" => Ok(Some(json!([
                        {
                            "uid": "latency-main",
                            "title": "Latency Main",
                            "type": "dash-db",
                            "folderUid": "ops",
                            "folderTitle": "Ops"
                        }
                    ]))),
                    other => panic!("unexpected org context {other}"),
                }
            }
            (reqwest::Method::GET, "/api/folders/general") => {
                Ok(Some(json!({"uid": "general", "title": "General"})))
            }
            (reqwest::Method::GET, "/api/folders/general/permissions") => Ok(Some(json!([]))),
            (reqwest::Method::GET, "/api/folders/ops") => {
                Ok(Some(json!({"uid": "ops", "title": "Ops"})))
            }
            (reqwest::Method::GET, "/api/folders/ops/permissions") => Ok(Some(json!([]))),
            (reqwest::Method::GET, "/api/dashboards/uid/cpu-main") => Ok(Some(json!({
                "dashboard": {
                    "id": 11,
                    "uid": "cpu-main",
                    "title": "CPU Main",
                    "panels": [{
                        "id": 7,
                        "title": "CPU Query",
                        "type": "timeseries",
                        "datasource": {"uid": "prom-main", "type": "prometheus"},
                        "targets": [{"refId": "A", "expr": "up"}]
                    }]
                },
                "meta": {"folderUid": "general", "folderTitle": "General"}
            }))),
            (reqwest::Method::GET, "/api/dashboards/uid/cpu-main/permissions") => {
                Ok(Some(json!([])))
            }
            (reqwest::Method::GET, "/api/dashboards/uid/latency-main") => Ok(Some(json!({
                "dashboard": {
                    "id": 12,
                    "uid": "latency-main",
                    "title": "Latency Main",
                    "panels": [{
                        "id": 8,
                        "title": "Latency Query",
                        "type": "timeseries",
                        "datasource": {"uid": "prom-two", "type": "prometheus"},
                        "targets": [{"refId": "A", "expr": "rate(http_requests_total[5m])"}]
                    }]
                },
                "meta": {"folderUid": "ops", "folderTitle": "Ops"}
            }))),
            (reqwest::Method::GET, "/api/dashboards/uid/latency-main/permissions") => {
                Ok(Some(json!([])))
            }
            (_method, path) => Err(test_support::message(format!(
                "unexpected request {method_name} {path} {params:?}"
            ))),
        }
    }
}

fn export_query_row<'a>(
    report: &'a test_support::ExportInspectionQueryReport,
    dashboard_uid: &str,
) -> &'a test_support::ExportInspectionQueryRow {
    report
        .queries
        .iter()
        .find(|query| query.dashboard_uid == dashboard_uid)
        .unwrap()
}

#[derive(Clone, Copy, Default)]
struct CoreFamilyQueryRowExpectation<'a> {
    dashboard_uid: &'a str,
    dashboard_title: &'a str,
    panel_id: &'a str,
    panel_title: &'a str,
    panel_type: &'a str,
    ref_id: &'a str,
    datasource: &'a str,
    datasource_name: &'a str,
    datasource_uid: &'a str,
    datasource_type: &'a str,
    datasource_family: &'a str,
    query_field: &'a str,
    query_text: &'a str,
    folder_path: &'a str,
    folder_full_path: &'a str,
    folder_level: &'a str,
    folder_uid: &'a str,
    parent_folder_uid: &'a str,
    datasource_org: &'a str,
    datasource_org_id: &'a str,
    datasource_database: &'a str,
    datasource_bucket: &'a str,
    datasource_organization: &'a str,
    datasource_index_pattern: &'a str,
    metrics: &'a [&'a str],
    functions: &'a [&'a str],
    measurements: &'a [&'a str],
    buckets: &'a [&'a str],
}

fn assert_core_family_query_row(
    report: &test_support::ExportInspectionQueryReport,
    expected: CoreFamilyQueryRowExpectation<'_>,
) {
    let row = export_query_row(report, expected.dashboard_uid);
    if !expected.dashboard_uid.is_empty() {
        assert_eq!(row.dashboard_uid, expected.dashboard_uid);
    }
    if !expected.dashboard_title.is_empty() {
        assert_eq!(row.dashboard_title, expected.dashboard_title);
    }
    if !expected.panel_id.is_empty() {
        assert_eq!(row.panel_id, expected.panel_id);
    }
    if !expected.panel_title.is_empty() {
        assert_eq!(row.panel_title, expected.panel_title);
    }
    if !expected.panel_type.is_empty() {
        assert_eq!(row.panel_type, expected.panel_type);
    }
    if !expected.ref_id.is_empty() {
        assert_eq!(row.ref_id, expected.ref_id);
    }
    if !expected.datasource.is_empty() {
        assert_eq!(row.datasource, expected.datasource);
    }
    if !expected.datasource_name.is_empty() {
        assert_eq!(row.datasource_name, expected.datasource_name);
    }
    if !expected.datasource_uid.is_empty() {
        assert_eq!(row.datasource_uid, expected.datasource_uid);
    }
    if !expected.datasource_type.is_empty() {
        assert_eq!(row.datasource_type, expected.datasource_type);
    }
    if !expected.datasource_family.is_empty() {
        assert_eq!(row.datasource_family, expected.datasource_family);
    }
    if !expected.query_field.is_empty() {
        assert_eq!(row.query_field, expected.query_field);
    }
    if !expected.query_text.is_empty() {
        assert_eq!(row.query_text, expected.query_text);
    }
    if !expected.folder_path.is_empty() {
        assert_eq!(row.folder_path, expected.folder_path);
    }
    if !expected.folder_full_path.is_empty() {
        assert_eq!(row.folder_full_path, expected.folder_full_path);
    }
    if !expected.folder_level.is_empty() {
        assert_eq!(row.folder_level, expected.folder_level);
    }
    if !expected.folder_uid.is_empty() {
        assert_eq!(row.folder_uid, expected.folder_uid);
    }
    if !expected.parent_folder_uid.is_empty() {
        assert_eq!(row.parent_folder_uid, expected.parent_folder_uid);
    }
    if !expected.datasource_org.is_empty() {
        assert_eq!(row.datasource_org, expected.datasource_org);
    }
    if !expected.datasource_org_id.is_empty() {
        assert_eq!(row.datasource_org_id, expected.datasource_org_id);
    }
    if !expected.datasource_database.is_empty() {
        assert_eq!(row.datasource_database, expected.datasource_database);
    }
    if !expected.datasource_bucket.is_empty() {
        assert_eq!(row.datasource_bucket, expected.datasource_bucket);
    }
    if !expected.datasource_organization.is_empty() {
        assert_eq!(
            row.datasource_organization,
            expected.datasource_organization
        );
    }
    if !expected.datasource_index_pattern.is_empty() {
        assert_eq!(
            row.datasource_index_pattern,
            expected.datasource_index_pattern
        );
    }
    assert_eq!(row.dashboard_tags, Vec::<String>::new());
    assert_eq!(row.panel_target_count, 1);
    assert_eq!(row.panel_query_count, 1);
    assert_eq!(row.panel_datasource_count, 1);
    assert_eq!(row.panel_variables, Vec::<String>::new());
    assert_eq!(row.query_variables, Vec::<String>::new());
    assert_eq!(row.target_hidden, "false");
    assert_eq!(row.target_disabled, "false");
    assert_eq!(
        row.metrics,
        expected
            .metrics
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<String>>()
    );
    assert_eq!(
        row.functions,
        expected
            .functions
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<String>>()
    );
    assert_eq!(
        row.measurements,
        expected
            .measurements
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<String>>()
    );
    assert_eq!(
        row.buckets,
        expected
            .buckets
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<String>>()
    );
}

#[cfg(test)]
#[path = "inspect_query_rust_tests.rs"]
mod inspect_query_rust_tests;


#[test]
fn build_export_metadata_serializes_expected_shape() {
    let value = serde_json::to_value(build_export_metadata(
        "raw",
        2,
        Some("grafana-web-import-preserve-uid"),
        Some(FOLDER_INVENTORY_FILENAME),
        Some(DATASOURCE_INVENTORY_FILENAME),
        Some(DASHBOARD_PERMISSION_BUNDLE_FILENAME),
        Some("Main Org."),
        Some("1"),
        None,
    ))
    .unwrap();

    assert_eq!(
        value,
        json!({
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "kind": "grafana-utils-dashboard-export-index",
            "variant": "raw",
            "dashboardCount": 2,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid",
            "foldersFile": "folders.json",
            "datasourcesFile": "datasources.json",
            "permissionsFile": "permissions.json",
            "org": "Main Org.",
            "orgId": "1"
        })
    );
}

#[test]
fn build_root_export_index_serializes_expected_shape() {
    let summary = serde_json::from_value(json!({
        "uid": "cpu-main",
        "title": "CPU Overview",
        "folderTitle": "Infra",
        "orgName": "Main Org.",
        "orgId": 1
    }))
    .unwrap();
    let mut item = test_support::build_dashboard_index_item(&summary, "cpu-main");
    item.raw_path = Some("/tmp/raw/cpu-main.json".to_string());

    let value = serde_json::to_value(build_root_export_index(
        &[item],
        Some(Path::new("/tmp/raw/index.json")),
        None,
        &[],
    ))
    .unwrap();

    assert_eq!(
        value,
        json!({
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "kind": "grafana-utils-dashboard-export-index",
            "items": [
                {
                    "uid": "cpu-main",
                    "title": "CPU Overview",
                    "folderTitle": "Infra",
                    "org": "Main Org.",
                    "orgId": "1",
                    "raw_path": "/tmp/raw/cpu-main.json"
                }
            ],
            "variants": {
                "raw": "/tmp/raw/index.json",
                "prompt": null
            },
            "folders": []
        })
    );
}

#[test]
fn collect_folder_inventory_with_request_records_parent_chain() {
    let summaries = vec![json!({
        "uid": "cpu-main",
        "title": "CPU Overview",
        "folderTitle": "Infra",
        "folderUid": "infra",
        "orgName": "Main Org.",
        "orgId": 1
    })
    .as_object()
    .unwrap()
    .clone()];

    let folders = test_support::collect_folder_inventory_with_request(
        |_method, path, _params, _payload| match path {
            "/api/folders/infra" => Ok(Some(json!({
                "uid": "infra",
                "title": "Infra",
                "parents": [
                    {"uid": "platform", "title": "Platform"},
                    {"uid": "team", "title": "Team"}
                ]
            }))),
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &summaries,
    )
    .unwrap();

    assert_eq!(
        serde_json::to_value(folders).unwrap(),
        json!([
            {
                "uid": "platform",
                "title": "Platform",
                "path": "Platform",
                "org": "Main Org.",
                "orgId": "1"
            },
            {
                "uid": "team",
                "title": "Team",
                "path": "Platform / Team",
                "parentUid": "platform",
                "org": "Main Org.",
                "orgId": "1"
            },
            {
                "uid": "infra",
                "title": "Infra",
                "path": "Platform / Team / Infra",
                "parentUid": "team",
                "org": "Main Org.",
                "orgId": "1"
            }
        ])
    );
}

#[test]
fn build_datasource_inventory_record_keeps_datasource_config_fields() {
    let datasource = json!({
        "uid": "influx-main",
        "name": "Influx Main",
        "type": "influxdb",
        "access": "proxy",
        "url": "http://influxdb:8086",
        "jsonData": {
            "dbName": "metrics_v1",
            "defaultBucket": "prod-default",
            "organization": "acme-observability"
        }
    })
    .as_object()
    .unwrap()
    .clone();
    let org = json!({
        "id": 1,
        "name": "Main Org."
    })
    .as_object()
    .unwrap()
    .clone();

    let record = test_support::build_datasource_inventory_record(&datasource, &org);
    assert_eq!(record.database, "metrics_v1");
    assert_eq!(record.default_bucket, "prod-default");
    assert_eq!(record.organization, "acme-observability");

    let elastic = json!({
        "uid": "elastic-main",
        "name": "Elastic Main",
        "type": "elasticsearch",
        "access": "proxy",
        "url": "http://elasticsearch:9200",
        "jsonData": {
            "indexPattern": "[logs-]YYYY.MM.DD"
        }
    })
    .as_object()
    .unwrap()
    .clone();
    let elastic_record = test_support::build_datasource_inventory_record(&elastic, &org);
    assert_eq!(elastic_record.index_pattern, "[logs-]YYYY.MM.DD");
}

fn make_inspect_live_tui_fixture() -> (
    test_support::ExportInspectionSummary,
    test_support::inspect_governance::ExportInspectionGovernanceDocument,
    test_support::ExportInspectionQueryReport,
) {
    let summary = test_support::ExportInspectionSummary {
        import_dir: "/tmp/raw".to_string(),
        export_org: Some("Main Org.".to_string()),
        export_org_id: Some("1".to_string()),
        dashboard_count: 1,
        folder_count: 1,
        panel_count: 1,
        query_count: 1,
        datasource_inventory_count: 1,
        orphaned_datasource_count: 0,
        mixed_dashboard_count: 0,
        folder_paths: Vec::new(),
        datasource_usage: Vec::new(),
        datasource_inventory: Vec::new(),
        orphaned_datasources: Vec::new(),
        mixed_dashboards: Vec::new(),
    };
    let query = make_core_family_report_row(
        "cpu-main",
        "7",
        "A",
        "prom-main",
        "Prometheus Main",
        "prometheus",
        "prometheus",
        "up",
        &[],
    );
    let report = test_support::ExportInspectionQueryReport {
        import_dir: "/tmp/raw".to_string(),
        summary: test_support::QueryReportSummary {
            dashboard_count: 1,
            panel_count: 1,
            query_count: 1,
            report_row_count: 1,
        },
        queries: vec![query],
    };
    let governance = test_support::inspect_governance::ExportInspectionGovernanceDocument {
        summary: test_support::inspect_governance::GovernanceSummary {
            dashboard_count: 1,
            query_record_count: 1,
            datasource_inventory_count: 1,
            datasource_family_count: 1,
            datasource_coverage_count: 1,
            dashboard_datasource_edge_count: 1,
            datasource_risk_coverage_count: 1,
            dashboard_risk_coverage_count: 1,
            mixed_datasource_dashboard_count: 0,
            orphaned_datasource_count: 0,
            risk_record_count: 2,
            query_audit_count: 1,
            dashboard_audit_count: 0,
        },
        datasource_families: Vec::new(),
        dashboard_dependencies: Vec::new(),
        dashboard_governance: vec![test_support::inspect_governance::DashboardGovernanceRow {
            dashboard_uid: "cpu-main".to_string(),
            dashboard_title: "CPU Main".to_string(),
            folder_path: "General".to_string(),
            panel_count: 1,
            query_count: 1,
            datasource_count: 1,
            datasource_family_count: 1,
            datasources: vec!["prom-main".to_string()],
            datasource_families: vec!["prometheus".to_string()],
            mixed_datasource: false,
            risk_count: 1,
            risk_kinds: vec!["prometheus-query-cost-score".to_string()],
        }],
        dashboard_datasource_edges: Vec::new(),
        datasource_governance: Vec::new(),
        datasources: Vec::new(),
        risk_records: vec![test_support::inspect_governance::GovernanceRiskRow {
            kind: "prometheus-query-cost-score".to_string(),
            severity: "high".to_string(),
            category: "cost".to_string(),
            dashboard_uid: "cpu-main".to_string(),
            panel_id: "7".to_string(),
            datasource: "Prometheus Main".to_string(),
            detail: "cost=3".to_string(),
            recommendation: "Reduce expensive Prometheus query shapes before broad rollout."
                .to_string(),
        }],
        query_audits: vec![test_support::inspect_governance::QueryAuditRow {
            dashboard_uid: "cpu-main".to_string(),
            dashboard_title: "CPU Main".to_string(),
            folder_path: "General".to_string(),
            panel_id: "7".to_string(),
            panel_title: "CPU".to_string(),
            ref_id: "A".to_string(),
            datasource: "Prometheus Main".to_string(),
            datasource_uid: "prom-main".to_string(),
            datasource_family: "prometheus".to_string(),
            aggregation_depth: 0,
            regex_matcher_count: 0,
            estimated_series_risk: "low".to_string(),
            query_cost_score: 3,
            score: 2,
            severity: "medium".to_string(),
            reasons: vec![
                "broad-prometheus-selector".to_string(),
                "prometheus-query-cost-score".to_string(),
            ],
            recommendations: vec![
                "Add label filters to the Prometheus selector.".to_string(),
                "Trim costly aggregation and range windows.".to_string(),
            ],
        }],
        dashboard_audits: Vec::new(),
    };

    (summary, governance, report)
}

#[test]
fn build_inspect_live_tui_groups_summarizes_dashboard_query_and_risk_sections() {
    let (summary, governance, report) = make_inspect_live_tui_fixture();
    let groups = test_support::build_inspect_live_tui_groups(&summary, &governance, &report);

    assert_eq!(groups.len(), 4);
    assert_eq!(groups[0].label, "All");
    assert_eq!(groups[0].count, 5);
    assert_eq!(groups[1].label, "Dashboards");
    assert_eq!(groups[1].count, 1);
    assert_eq!(groups[2].label, "Queries");
    assert_eq!(groups[2].count, 1);
    assert_eq!(groups[3].label, "Risks");
    assert_eq!(groups[3].count, 3);
}

#[test]
fn filter_inspect_live_tui_items_limits_items_to_selected_group() {
    let (summary, governance, report) = make_inspect_live_tui_fixture();
    let dashboard_items =
        test_support::filter_inspect_live_tui_items(&summary, &governance, &report, "dashboards");
    let query_items =
        test_support::filter_inspect_live_tui_items(&summary, &governance, &report, "queries");
    let risk_items =
        test_support::filter_inspect_live_tui_items(&summary, &governance, &report, "risks");
    let all_items =
        test_support::filter_inspect_live_tui_items(&summary, &governance, &report, "all");

    assert_eq!(dashboard_items.len(), 1);
    assert!(dashboard_items.iter().all(|item| item.kind == "dashboard"));
    assert_eq!(query_items.len(), 1);
    assert!(query_items.iter().all(|item| item.kind == "query"));
    assert_eq!(risk_items.len(), 3);
    assert!(risk_items.iter().any(|item| item.kind == "dashboard-risk"));
    assert!(risk_items.iter().any(|item| item.kind == "risk-record"));
    assert!(risk_items.iter().any(|item| item.kind == "query-audit"));
    assert_eq!(all_items.len(), 5);
    assert!(all_items.iter().any(|item| item.kind == "dashboard"));
    assert!(all_items.iter().any(|item| item.kind == "query"));
    assert!(all_items.iter().any(|item| item.kind == "risk-record"));
}

#[test]
fn build_topology_tui_groups_summarize_node_kinds() {
    let document = sample_topology_tui_document();
    let groups = build_topology_tui_groups(&document);

    let counts = groups
        .iter()
        .map(|group| (group.label.as_str(), group.count))
        .collect::<Vec<_>>();
    assert_eq!(
        counts,
        vec![
            ("All", 5),
            ("Datasources", 1),
            ("Dashboards", 1),
            ("Panels", 1),
            ("Variables", 1),
            ("Alert Rules", 1),
            ("Contact Points", 0),
            ("Mute Timings", 0),
            ("Policies", 0),
            ("Templates", 0),
            ("Alert Resources", 0),
        ]
    );
}

#[test]
fn filter_topology_tui_items_limits_items_to_selected_group() {
    let document = sample_topology_tui_document();

    let variables = filter_topology_tui_items(&document, "variable");
    assert_eq!(variables.len(), 1);
    assert_eq!(variables[0].kind, "variable");
    assert_eq!(variables[0].title, "cluster");

    let panels = filter_topology_tui_items(&document, "panel");
    assert_eq!(panels.len(), 1);
    assert_eq!(panels[0].kind, "panel");
    assert_eq!(panels[0].title, "Panel 7");

    let all = filter_topology_tui_items(&document, "all");
    assert_eq!(all.len(), document.nodes.len());
}


#[test]
fn validate_dashboard_export_dir_detects_custom_plugin_legacy_layout_and_schema_migration() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join("legacy.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "uid": "legacy-main",
                "title": "Legacy Main",
                "schemaVersion": 30,
                "rows": [],
                "panels": [
                    {"id": 7, "type": "acme-panel", "datasource": {"type": "acme-ds"}}
                ]
            },
            "__inputs": [{"name": "DS_PROM"}]
        }))
        .unwrap(),
    )
    .unwrap();

    let result =
        test_support::validate_dashboard_export_dir(&raw_dir, true, true, Some(39)).unwrap();
    let output = test_support::render_validation_result_json(&result).unwrap();

    assert_eq!(result.dashboard_count, 1);
    assert!(result.error_count >= 4);
    assert!(output.contains("custom-panel-plugin"));
    assert!(output.contains("custom-datasource-plugin"));
    assert!(output.contains("legacy-row-layout"));
    assert!(output.contains("schema-migration-required"));
}

#[test]
fn snapshot_live_dashboard_export_with_fetcher_writes_dashboards_in_parallel() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    let summaries = vec![
        json!({"uid": "cpu-main", "title": "CPU Main", "folderTitle": "Infra"})
            .as_object()
            .unwrap()
            .clone(),
        json!({"uid": "logs-main", "title": "Logs Main", "folderTitle": "Ops"})
            .as_object()
            .unwrap()
            .clone(),
    ];

    let count = test_support::snapshot_live_dashboard_export_with_fetcher(
        &raw_dir,
        &summaries,
        4,
        false,
        |uid| {
            Ok(json!({
                "dashboard": {
                    "uid": uid,
                    "title": uid,
                    "schemaVersion": 39,
                    "panels": []
                },
                "meta": {}
            }))
        },
    )
    .unwrap();

    assert_eq!(count, 2);
    assert!(raw_dir.join("Infra/CPU_Main__cpu-main.json").is_file());
    assert!(raw_dir.join("Ops/Logs_Main__logs-main.json").is_file());
}

#[test]
fn import_dashboards_with_strict_schema_rejects_custom_plugins_before_live_write() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("custom.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "uid": "custom-main",
                "title": "Custom Main",
                "schemaVersion": 39,
                "panels": [
                    {"id": 7, "type": "acme-panel", "datasource": {"type": "prometheus"}}
                ]
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let mut args = make_import_args(raw_dir);
    args.strict_schema = true;
    args.dry_run = true;
    let error = test_support::import_dashboards_with_request(
        |_method, _path, _params, _payload| Ok(None),
        &args,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("custom-panel-plugin"));
    assert!(error.contains("unsupported custom panel plugin type"));
}


#[test]
fn build_output_path_keeps_folder_structure() {
    let summary = json!({
        "folderTitle": "Infra Team",
        "title": "Cluster Health",
        "uid": "abc",
    });
    let path = build_output_path(Path::new("out"), summary.as_object().unwrap(), false);
    assert_eq!(path, Path::new("out/Infra_Team/Cluster_Health__abc.json"));
}

#[test]
fn build_folder_inventory_status_reports_missing_folder() {
    let folder = test_support::FolderInventoryItem {
        uid: "child".to_string(),
        title: "Child".to_string(),
        path: "Platform / Child".to_string(),
        parent_uid: Some("platform".to_string()),
        org: "Main Org.".to_string(),
        org_id: "1".to_string(),
    };

    let status = build_folder_inventory_status(&folder, None);

    assert_eq!(status.kind, FolderInventoryStatusKind::Missing);
    assert_eq!(
        format_folder_inventory_status_line(&status),
        "Folder inventory missing uid=child title=Child parentUid=platform path=Platform / Child"
    );
}

#[test]
fn build_folder_inventory_status_reports_matching_folder() {
    let folder = test_support::FolderInventoryItem {
        uid: "child".to_string(),
        title: "Child".to_string(),
        path: "Platform / Child".to_string(),
        parent_uid: Some("platform".to_string()),
        org: "Main Org.".to_string(),
        org_id: "1".to_string(),
    };
    let destination_folder = json!({
        "uid": "child",
        "title": "Child",
        "parents": [{"uid": "platform", "title": "Platform"}]
    })
    .as_object()
    .unwrap()
    .clone();

    let status = build_folder_inventory_status(&folder, Some(&destination_folder));

    assert_eq!(status.kind, FolderInventoryStatusKind::Matches);
    assert_eq!(
        format_folder_inventory_status_line(&status),
        "Folder inventory matches uid=child title=Child parentUid=platform path=Platform / Child"
    );
}

#[test]
fn build_folder_inventory_status_reports_mismatch_details() {
    let folder = test_support::FolderInventoryItem {
        uid: "child".to_string(),
        title: "Child".to_string(),
        path: "Platform / Child".to_string(),
        parent_uid: Some("platform".to_string()),
        org: "Main Org.".to_string(),
        org_id: "1".to_string(),
    };
    let destination_folder = json!({
        "uid": "child",
        "title": "Ops Child",
        "parents": [{"uid": "ops", "title": "Ops"}]
    })
    .as_object()
    .unwrap()
    .clone();

    let status = build_folder_inventory_status(&folder, Some(&destination_folder));

    assert_eq!(status.kind, FolderInventoryStatusKind::Mismatch);
    assert_eq!(
        format_folder_inventory_status_line(&status),
        "Folder inventory mismatch uid=child expected(title=Child, parentUid=platform, path=Platform / Child) actual(title=Ops Child, parentUid=ops, path=Ops / Ops Child)"
    );
}

#[test]
fn render_folder_inventory_dry_run_table_supports_expected_columns() {
    let rows = vec![[
        "child".to_string(),
        "exists".to_string(),
        "mismatch".to_string(),
        "path".to_string(),
        "Platform / Child".to_string(),
        "Legacy / Child".to_string(),
    ]];

    let with_header = test_support::render_folder_inventory_dry_run_table(&rows, true);

    assert!(with_header[0].contains("EXPECTED_PATH"));
    assert!(with_header[0].contains("ACTUAL_PATH"));
    assert!(with_header[2].contains("Legacy / Child"));
}

#[test]
fn export_progress_line_uses_concise_counter_format() {
    assert_eq!(
        format_export_progress_line(2, 5, "cpu-main", false),
        "Exporting dashboard 2/5: cpu-main"
    );
    assert_eq!(
        format_export_progress_line(2, 5, "cpu-main", true),
        "Would export dashboard 2/5: cpu-main"
    );
}

#[test]
fn export_verbose_line_includes_variant_and_path() {
    assert_eq!(
        format_export_verbose_line("prompt", "cpu-main", Path::new("/tmp/out.json"), false),
        "Exported prompt cpu-main -> /tmp/out.json"
    );
    assert_eq!(
        format_export_verbose_line("raw", "cpu-main", Path::new("/tmp/out.json"), true),
        "Would export raw    cpu-main -> /tmp/out.json"
    );
}

#[test]
fn import_progress_line_uses_concise_counter_format() {
    assert_eq!(
        format_import_progress_line(3, 7, "/tmp/raw/cpu.json", false, None, None),
        "Importing dashboard 3/7: /tmp/raw/cpu.json"
    );
    assert_eq!(
        format_import_progress_line(
            3,
            7,
            "cpu-main",
            true,
            Some("would-update"),
            Some("General")
        ),
        "Dry-run dashboard 3/7: cpu-main dest=exists action=update folderPath=General"
    );
    assert_eq!(
        format_import_progress_line(3, 7, "cpu-main", true, Some("would-skip-missing"), Some("Platform / Infra")),
        "Dry-run dashboard 3/7: cpu-main dest=missing action=skip-missing folderPath=Platform / Infra"
    );
}

#[test]
fn render_import_dry_run_table_supports_optional_header() {
    let rows = vec![
        [
            "abc".to_string(),
            "exists".to_string(),
            "update".to_string(),
            "General".to_string(),
            "General".to_string(),
            "General".to_string(),
            "".to_string(),
            "/tmp/a.json".to_string(),
        ],
        [
            "xyz".to_string(),
            "missing".to_string(),
            "create".to_string(),
            "Platform / Infra".to_string(),
            "Platform / Infra".to_string(),
            "".to_string(),
            "".to_string(),
            "/tmp/b.json".to_string(),
        ],
    ];
    let with_header = test_support::render_import_dry_run_table(&rows, true, None);
    assert!(with_header[0].contains("UID"));
    assert!(with_header[0].contains("DESTINATION"));
    assert!(with_header[0].contains("ACTION"));
    assert!(with_header[0].contains("FOLDER_PATH"));
    assert!(with_header[0].contains("FILE"));
    assert!(with_header[2].contains("abc"));
    assert!(with_header[2].contains("exists"));
    assert!(with_header[2].contains("update"));
    assert!(with_header[2].contains("General"));
    assert!(with_header[2].contains("/tmp/a.json"));
    let without_header = test_support::render_import_dry_run_table(&rows, false, None);
    assert_eq!(without_header.len(), 2);
    assert!(without_header[0].contains("abc"));
    assert!(without_header[0].contains("exists"));
    assert!(without_header[0].contains("update"));
    assert!(without_header[0].contains("General"));
    assert!(without_header[0].contains("/tmp/a.json"));
}

#[test]
fn render_import_dry_run_table_honors_selected_columns() {
    let rows = vec![[
        "abc".to_string(),
        "exists".to_string(),
        "skip-folder-mismatch".to_string(),
        "Platform / Ops".to_string(),
        "Platform / Source".to_string(),
        "Platform / Dest".to_string(),
        "path".to_string(),
        "/tmp/a.json".to_string(),
    ]];

    let lines = test_support::render_import_dry_run_table(
        &rows,
        true,
        Some(&["uid".to_string(), "reason".to_string(), "file".to_string()]),
    );

    assert!(lines[0].contains("UID"));
    assert!(lines[0].contains("REASON"));
    assert!(lines[0].contains("FILE"));
    assert!(!lines[0].contains("DESTINATION"));
    assert!(lines[2].contains("abc"));
    assert!(lines[2].contains("path"));
    assert!(lines[2].contains("/tmp/a.json"));
}

#[test]
fn render_import_dry_run_json_returns_structured_document() {
    let folder_status = test_support::FolderInventoryStatus {
        uid: "infra".to_string(),
        expected_title: "Infra".to_string(),
        expected_parent_uid: Some("platform".to_string()),
        expected_path: "Platform / Infra".to_string(),
        actual_title: Some("Infra".to_string()),
        actual_parent_uid: Some("platform".to_string()),
        actual_path: Some("Platform / Infra".to_string()),
        kind: FolderInventoryStatusKind::Matches,
    };
    let rows = vec![[
        "abc".to_string(),
        "exists".to_string(),
        "update".to_string(),
        "Platform / Infra".to_string(),
        "Platform / Infra".to_string(),
        "Platform / Infra".to_string(),
        "".to_string(),
        "/tmp/a.json".to_string(),
    ]];

    let value: Value = serde_json::from_str(
        &test_support::render_import_dry_run_json(
            "create-or-update",
            &[folder_status],
            &rows,
            Path::new("/tmp/raw"),
            0,
            0,
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(value["mode"], "create-or-update");
    assert_eq!(value["folders"][0]["uid"], "infra");
    assert_eq!(value["dashboards"][0]["folderPath"], "Platform / Infra");
    assert_eq!(
        value["dashboards"][0]["sourceFolderPath"],
        "Platform / Infra"
    );
    assert_eq!(
        value["dashboards"][0]["destinationFolderPath"],
        "Platform / Infra"
    );
    assert_eq!(value["summary"]["dashboardCount"], 1);
}

#[test]
fn render_routed_import_org_table_includes_org_level_columns() {
    let rows = vec![
        [
            "2".to_string(),
            "Org Two".to_string(),
            "exists".to_string(),
            "2".to_string(),
            "3".to_string(),
        ],
        [
            "9".to_string(),
            "Ops Org".to_string(),
            "would-create".to_string(),
            "<new>".to_string(),
            "1".to_string(),
        ],
    ];

    let lines = test_support::import::render_routed_import_org_table(&rows, true);

    assert!(lines[0].contains("SOURCE_ORG_ID"));
    assert!(lines[0].contains("ORG_ACTION"));
    assert!(lines[2].contains("Org Two"));
    assert!(lines[3].contains("would-create"));
}

#[test]
fn routed_import_scope_identity_matches_table_json_and_progress_surfaces() {
    let temp = tempdir().unwrap();
    let export_root = temp.path().join("exports");
    let org_two_raw = export_root.join("org_2_Org_Two").join("raw");
    let org_nine_raw = export_root.join("org_9_Ops_Org").join("raw");
    write_combined_export_root_metadata(
        &export_root,
        &[
            ("2", "Org Two", "org_2_Org_Two"),
            ("9", "Ops Org", "org_9_Ops_Org"),
        ],
    );
    write_basic_raw_export(
        &org_two_raw,
        "2",
        "Org Two",
        "cpu-two",
        "CPU Two",
        "prom-two",
        "prometheus",
        "timeseries",
        "general",
        "General",
        "expr",
        "up",
    );
    write_basic_raw_export(
        &org_nine_raw,
        "9",
        "Ops Org",
        "ops-main",
        "Ops Main",
        "loki-nine",
        "loki",
        "logs",
        "ops",
        "Ops",
        "expr",
        "{job=\"grafana\"}",
    );

    let mut args = make_import_args(export_root);
    args.common = make_basic_common_args("http://127.0.0.1:3000".to_string());
    args.use_export_org = true;
    args.create_missing_orgs = true;
    args.dry_run = true;
    args.json = true;

    let payload: Value = serde_json::from_str(
        &test_support::import::build_routed_import_dry_run_json_with_request(
            |method, path, _params, _payload| match (method, path) {
                (reqwest::Method::GET, "/api/orgs") => Ok(Some(json!([
                    {"id": 2, "name": "Org Two"}
                ]))),
                _ => Err(test_support::message(format!("unexpected request {path}"))),
            },
            |_target_org_id, scoped_args| {
                Ok(test_support::import::ImportDryRunReport {
                    mode: "create-only".to_string(),
                    import_dir: scoped_args.import_dir.clone(),
                    folder_statuses: Vec::new(),
                    dashboard_records: Vec::new(),
                    skipped_missing_count: 0,
                    skipped_folder_mismatch_count: 0,
                })
            },
            &args,
        )
        .unwrap(),
    )
    .unwrap();

    let org_entries = payload["orgs"].as_array().unwrap();
    let rows: Vec<[String; 5]> = org_entries
        .iter()
        .map(|entry| {
            [
                entry["sourceOrgId"].as_i64().unwrap().to_string(),
                entry["sourceOrgName"].as_str().unwrap().to_string(),
                entry["orgAction"].as_str().unwrap().to_string(),
                test_support::import::format_routed_import_target_org_label(
                    entry["targetOrgId"].as_i64(),
                ),
                entry["dashboardCount"].as_u64().unwrap().to_string(),
            ]
        })
        .collect();
    let table_lines = test_support::import::render_routed_import_org_table(&rows, true);

    let org_two = org_entries
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(2))
        .unwrap();
    let org_nine = org_entries
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(9))
        .unwrap();

    let existing_summary = test_support::import::format_routed_import_scope_summary_fields(
        2,
        "Org Two",
        "exists",
        Some(2),
        Path::new(org_two["importDir"].as_str().unwrap()),
    );
    let would_create_summary = test_support::import::format_routed_import_scope_summary_fields(
        9,
        "Ops Org",
        "would-create",
        None,
        Path::new(org_nine["importDir"].as_str().unwrap()),
    );

    assert_eq!(org_two["targetOrgId"], json!(2));
    assert_eq!(org_nine["targetOrgId"], Value::Null);
    assert!(table_lines[2].contains("Org Two"));
    assert!(table_lines[2].contains("2"));
    assert!(table_lines[3].contains("Ops Org"));
    assert!(table_lines[3].contains("<new>"));
    assert!(existing_summary.contains("export orgId=2"));
    assert!(existing_summary.contains("name=Org Two"));
    assert!(existing_summary.contains("orgAction=exists"));
    assert!(existing_summary.contains("targetOrgId=2"));
    assert!(existing_summary.contains(org_two["importDir"].as_str().unwrap()));
    assert!(would_create_summary.contains("export orgId=9"));
    assert!(would_create_summary.contains("name=Ops Org"));
    assert!(would_create_summary.contains("orgAction=would-create"));
    assert!(would_create_summary.contains("targetOrgId=<new>"));
    assert!(would_create_summary.contains(org_nine["importDir"].as_str().unwrap()));
}

#[test]
fn routed_import_selected_scope_statuses_match_json_table_and_summary_contract() {
    let temp = tempdir().unwrap();
    let export_root = temp.path().join("exports");
    let org_two_raw = export_root.join("org_2_Org_Two").join("raw");
    let org_five_raw = export_root.join("org_5_Org_Five").join("raw");
    let org_nine_raw = export_root.join("org_9_Ops_Org").join("raw");
    write_combined_export_root_metadata(
        &export_root,
        &[
            ("2", "Org Two", "org_2_Org_Two"),
            ("5", "Org Five", "org_5_Org_Five"),
            ("9", "Ops Org", "org_9_Ops_Org"),
        ],
    );
    write_basic_raw_export(
        &org_two_raw,
        "2",
        "Org Two",
        "cpu-two",
        "CPU Two",
        "prom-two",
        "prometheus",
        "timeseries",
        "general",
        "General",
        "expr",
        "up",
    );
    write_basic_raw_export(
        &org_five_raw,
        "5",
        "Org Five",
        "cpu-five",
        "CPU Five",
        "prom-five",
        "prometheus",
        "timeseries",
        "general",
        "General",
        "expr",
        "up",
    );
    write_basic_raw_export(
        &org_nine_raw,
        "9",
        "Ops Org",
        "ops-main",
        "Ops Main",
        "loki-nine",
        "loki",
        "logs",
        "ops",
        "Ops",
        "expr",
        "{job=\"grafana\"}",
    );

    let mut args = make_import_args(export_root);
    args.common = make_basic_common_args("http://127.0.0.1:3000".to_string());
    args.use_export_org = true;
    args.only_org_id = vec![2, 9];
    args.create_missing_orgs = false;
    args.dry_run = true;
    args.json = true;

    let payload: Value = serde_json::from_str(
        &test_support::import::build_routed_import_dry_run_json_with_request(
            |method, path, _params, _payload| match (method, path) {
                (reqwest::Method::GET, "/api/orgs") => Ok(Some(json!([
                    {"id": 2, "name": "Org Two"}
                ]))),
                _ => Err(test_support::message(format!("unexpected request {path}"))),
            },
            |_target_org_id, scoped_args| {
                Ok(test_support::import::ImportDryRunReport {
                    mode: "create-only".to_string(),
                    import_dir: scoped_args.import_dir.clone(),
                    folder_statuses: Vec::new(),
                    dashboard_records: Vec::new(),
                    skipped_missing_count: 0,
                    skipped_folder_mismatch_count: 0,
                })
            },
            &args,
        )
        .unwrap(),
    )
    .unwrap();

    let org_entries = payload["orgs"].as_array().unwrap();
    let import_entries = payload["imports"].as_array().unwrap();
    assert_eq!(org_entries.len(), 2);
    assert_eq!(import_entries.len(), 2);
    assert_eq!(payload["summary"]["orgCount"], json!(2));
    assert_eq!(payload["summary"]["existingOrgCount"], json!(1));
    assert_eq!(payload["summary"]["missingOrgCount"], json!(1));
    assert_eq!(payload["summary"]["wouldCreateOrgCount"], json!(0));

    let org_two = org_entries
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(2))
        .unwrap();
    let org_nine = org_entries
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(9))
        .unwrap();
    assert!(org_entries
        .iter()
        .all(|entry| entry["sourceOrgId"] != json!(5)));

    assert_eq!(org_two["orgAction"], json!("exists"));
    assert_eq!(org_two["targetOrgId"], json!(2));
    assert_eq!(org_nine["orgAction"], json!("missing"));
    assert_eq!(org_nine["targetOrgId"], Value::Null);

    let org_nine_import = import_entries
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(9))
        .unwrap();
    assert_eq!(org_nine_import["orgAction"], json!("missing"));
    assert_eq!(org_nine_import["dashboards"], json!([]));
    assert_eq!(org_nine_import["summary"]["dashboardCount"], json!(1));

    let rows: Vec<[String; 5]> = org_entries
        .iter()
        .map(|entry| {
            [
                entry["sourceOrgId"].as_i64().unwrap().to_string(),
                entry["sourceOrgName"].as_str().unwrap().to_string(),
                entry["orgAction"].as_str().unwrap().to_string(),
                test_support::import::format_routed_import_target_org_label(
                    entry["targetOrgId"].as_i64(),
                ),
                entry["dashboardCount"].as_u64().unwrap().to_string(),
            ]
        })
        .collect();
    let table_lines = test_support::import::render_routed_import_org_table(&rows, true);
    assert!(table_lines[2].contains("Org Two"));
    assert!(table_lines[2].contains("exists"));
    assert!(table_lines[2].contains("2"));
    assert!(table_lines[3].contains("Ops Org"));
    assert!(table_lines[3].contains("missing"));
    assert!(table_lines[3].contains("<new>"));

    let missing_summary = test_support::import::format_routed_import_scope_summary_fields(
        9,
        "Ops Org",
        "missing",
        None,
        Path::new(org_nine["importDir"].as_str().unwrap()),
    );
    assert!(missing_summary.contains("export orgId=9"));
    assert!(missing_summary.contains("name=Ops Org"));
    assert!(missing_summary.contains("orgAction=missing"));
    assert!(missing_summary.contains("targetOrgId=<new>"));
    assert!(missing_summary.contains(org_nine["importDir"].as_str().unwrap()));
}

#[test]
fn describe_dashboard_import_mode_uses_expected_labels() {
    assert_eq!(
        test_support::describe_dashboard_import_mode(false, false),
        "create-only"
    );
    assert_eq!(
        test_support::describe_dashboard_import_mode(true, false),
        "create-or-update"
    );
    assert_eq!(
        test_support::describe_dashboard_import_mode(false, true),
        "update-or-skip-missing"
    );
}

#[test]
fn import_verbose_line_includes_dry_run_action() {
    assert_eq!(
        format_import_verbose_line(Path::new("/tmp/raw/cpu.json"), false, None, None, None),
        "Imported /tmp/raw/cpu.json"
    );
    assert_eq!(
        format_import_verbose_line(
            Path::new("/tmp/raw/cpu.json"),
            true,
            Some("cpu-main"),
            Some("would-update"),
            Some("General")
        ),
        "Dry-run import uid=cpu-main dest=exists action=update folderPath=General file=/tmp/raw/cpu.json"
    );
    assert_eq!(
        format_import_verbose_line(
            Path::new("/tmp/raw/cpu.json"),
            true,
            Some("cpu-main"),
            Some("would-skip-missing"),
            Some("Platform / Infra")
        ),
        "Dry-run import uid=cpu-main dest=missing action=skip-missing folderPath=Platform / Infra file=/tmp/raw/cpu.json"
    );
}

#[test]
fn build_export_variant_dirs_returns_raw_and_prompt_dirs() {
    let (raw_dir, prompt_dir) = build_export_variant_dirs(Path::new("dashboards"));
    assert_eq!(raw_dir, Path::new("dashboards/raw"));
    assert_eq!(prompt_dir, Path::new("dashboards/prompt"));
}

#[test]
fn discover_dashboard_files_rejects_combined_export_root() {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join("raw")).unwrap();
    fs::create_dir_all(temp.path().join("prompt")).unwrap();
    let error = discover_dashboard_files(temp.path()).unwrap_err();
    assert!(error.to_string().contains("combined export root"));
}

#[test]
fn discover_dashboard_files_ignores_export_metadata() {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join("raw/subdir")).unwrap();
    fs::write(
        temp.path().join("raw/subdir/dashboard.json"),
        serde_json::to_string_pretty(&json!({"uid": "abc", "title": "CPU"})).unwrap(),
    )
    .unwrap();
    fs::write(
        temp.path().join("raw").join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();

    let files = discover_dashboard_files(&temp.path().join("raw")).unwrap();
    assert_eq!(files, vec![temp.path().join("raw/subdir/dashboard.json")]);
}

#[test]
fn discover_dashboard_files_ignores_folder_inventory() {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join("raw/subdir")).unwrap();
    fs::write(
        temp.path().join("raw/subdir/dashboard.json"),
        serde_json::to_string_pretty(&json!({"uid": "abc", "title": "CPU"})).unwrap(),
    )
    .unwrap();
    fs::write(
        temp.path().join("raw").join(FOLDER_INVENTORY_FILENAME),
        serde_json::to_string_pretty(&json!([
            {"uid": "infra", "title": "Infra", "path": "Infra", "org": "Main Org.", "orgId": "1"}
        ]))
        .unwrap(),
    )
    .unwrap();

    let files = discover_dashboard_files(&temp.path().join("raw")).unwrap();
    assert_eq!(files, vec![temp.path().join("raw/subdir/dashboard.json")]);
}

#[test]
fn discover_dashboard_files_ignores_permission_bundle() {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join("raw/subdir")).unwrap();
    fs::write(
        temp.path().join("raw/subdir/dashboard.json"),
        serde_json::to_string_pretty(&json!({"uid": "abc", "title": "CPU"})).unwrap(),
    )
    .unwrap();
    fs::write(
        temp.path().join("raw").join(DASHBOARD_PERMISSION_BUNDLE_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-permission-bundle",
            "schemaVersion": 1,
            "summary": {"resourceCount": 0, "dashboardCount": 0, "folderCount": 0, "permissionCount": 0},
            "resources": []
        }))
        .unwrap(),
    )
    .unwrap();

    let files = discover_dashboard_files(&temp.path().join("raw")).unwrap();
    assert_eq!(files, vec![temp.path().join("raw/subdir/dashboard.json")]);
}

#[test]
fn build_import_payload_accepts_wrapped_document() {
    let payload = build_import_payload(
        &json!({
            "dashboard": {"id": 7, "uid": "abc", "title": "CPU"},
            "meta": {"folderUid": "old-folder"}
        }),
        Some("new-folder"),
        true,
        "sync dashboards",
    )
    .unwrap();

    assert_eq!(payload["dashboard"]["id"], Value::Null);
    assert_eq!(payload["folderUid"], "new-folder");
    assert_eq!(payload["overwrite"], true);
    assert_eq!(payload["message"], "sync dashboards");
}

#[test]
fn build_preserved_web_import_document_clears_numeric_id() {
    let document = build_preserved_web_import_document(&json!({
        "dashboard": {"id": 7, "uid": "abc", "title": "CPU"}
    }))
    .unwrap();

    assert_eq!(document["id"], Value::Null);
    assert_eq!(document["uid"], "abc");
}

#[test]
fn format_dashboard_summary_line_uses_uid_name_and_folder_details() {
    let summary = json!({
        "uid": "abc",
        "folderUid": "infra",
        "folderPath": "Platform / Infra",
        "folderTitle": "Infra",
        "orgId": 1,
        "orgName": "Main Org",
        "title": "CPU"
    });

    let line = format_dashboard_summary_line(summary.as_object().unwrap());
    assert_eq!(
        line,
        "uid=abc name=CPU folder=Infra folderUid=infra path=Platform / Infra org=Main Org orgId=1"
    );
}

#[test]
fn format_dashboard_summary_line_appends_sources_when_present() {
    let summary = json!({
        "uid": "abc",
        "folderUid": "infra",
        "folderPath": "Platform / Infra",
        "folderTitle": "Infra",
        "orgId": 1,
        "orgName": "Main Org",
        "title": "CPU",
        "sources": ["Loki Logs", "Prom Main"]
    });

    let line = format_dashboard_summary_line(summary.as_object().unwrap());
    assert_eq!(
        line,
        "uid=abc name=CPU folder=Infra folderUid=infra path=Platform / Infra org=Main Org orgId=1 sources=Loki Logs,Prom Main"
    );
}

#[test]
fn render_dashboard_summary_table_uses_headers_and_defaults() {
    let summaries = vec![
        json!({
            "uid": "abc",
            "folderUid": "infra",
            "folderPath": "Platform / Infra",
            "folderTitle": "Infra",
            "orgId": 1,
            "orgName": "Main Org",
            "title": "CPU"
        })
        .as_object()
        .unwrap()
        .clone(),
        json!({
            "orgId": 1,
            "orgName": "Main Org",
            "uid": "xyz",
            "title": "Overview"
        })
        .as_object()
        .unwrap()
        .clone(),
    ];

    let lines = render_dashboard_summary_table(&summaries, &[], true);
    assert!(lines[0].contains("ORG"));
    assert!(lines[0].contains("ORG_ID"));
    assert!(lines[2].contains("Main Org"));
    assert!(lines[2].contains("  1"));
    assert!(lines[3].contains("Main Org"));
}

#[test]
fn render_dashboard_summary_table_includes_sources_column_when_present() {
    let summaries = vec![json!({
        "uid": "abc",
        "folderUid": "infra",
        "folderPath": "Platform / Infra",
        "folderTitle": "Infra",
        "orgId": 1,
        "orgName": "Main Org",
        "title": "CPU",
        "sources": ["Prom Main", "Loki Logs"]
    })
    .as_object()
    .unwrap()
    .clone()];

    let lines = render_dashboard_summary_table(&summaries, &[], true);
    assert!(lines[0].contains("ORG"));
    assert!(lines[0].contains("SOURCES"));
    assert!(lines[2].starts_with("abc  CPU   Infra   infra"));
    assert!(lines[2].contains("Main Org"));
    assert!(lines[2].ends_with("Prom Main,Loki Logs"));
}

#[test]
fn render_dashboard_summary_table_can_omit_header() {
    let summaries = vec![json!({
        "uid": "abc",
        "folderUid": "infra",
        "folderPath": "Platform / Infra",
        "folderTitle": "Infra",
        "orgId": 1,
        "orgName": "Main Org",
        "title": "CPU"
    })
    .as_object()
    .unwrap()
    .clone()];

    let lines = render_dashboard_summary_table(&summaries, &[], false);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].starts_with("abc"));
}

#[test]
fn render_dashboard_summary_csv_uses_headers_and_escaping() {
    let summaries = vec![
        json!({
            "uid": "abc",
            "folderUid": "infra",
            "folderPath": "Platform / Infra",
            "folderTitle": "Infra",
            "orgId": 1,
            "orgName": "Main Org",
            "title": "CPU"
        })
        .as_object()
        .unwrap()
        .clone(),
        json!({
            "uid": "xyz",
            "folderUid": "ops",
            "folderPath": "Root / Ops",
            "folderTitle": "Ops",
            "orgId": 1,
            "orgName": "Main Org",
            "title": "CPU, \"critical\""
        })
        .as_object()
        .unwrap()
        .clone(),
    ];

    let lines = render_dashboard_summary_csv(&summaries, &[]);
    assert_eq!(lines[0], "uid,name,folder,folderUid,path,org,orgId");
    assert_eq!(lines[1], "abc,CPU,Infra,infra,Platform / Infra,Main Org,1");
    assert_eq!(
        lines[2],
        "xyz,\"CPU, \"\"critical\"\"\",Ops,ops,Root / Ops,Main Org,1"
    );
}

#[test]
fn render_dashboard_summary_csv_includes_sources_column_when_present() {
    let summaries = vec![json!({
        "uid": "abc",
        "folderUid": "infra",
        "folderPath": "Platform / Infra",
        "folderTitle": "Infra",
        "orgId": 1,
        "orgName": "Main Org",
        "title": "CPU",
        "sources": ["Prom Main", "Loki Logs"],
        "sourceUids": ["loki_uid", "prom_uid"]
    })
    .as_object()
    .unwrap()
    .clone()];

    let lines = render_dashboard_summary_csv(&summaries, &[]);
    assert_eq!(
        lines[0],
        "uid,name,folder,folderUid,path,org,orgId,sources,sourceUids"
    );
    assert_eq!(
        lines[1],
        "abc,CPU,Infra,infra,Platform / Infra,Main Org,1,\"Prom Main,Loki Logs\",\"loki_uid,prom_uid\""
    );
}

#[test]
fn render_dashboard_summary_json_returns_objects() {
    let summaries = vec![
        json!({
            "uid": "abc",
            "folderUid": "infra",
            "folderPath": "Platform / Infra",
            "folderTitle": "Infra",
            "orgId": 1,
            "orgName": "Main Org",
            "title": "CPU"
        })
        .as_object()
        .unwrap()
        .clone(),
        json!({
            "orgId": 1,
            "orgName": "Main Org",
            "uid": "xyz",
            "title": "Overview"
        })
        .as_object()
        .unwrap()
        .clone(),
    ];

    let value = render_dashboard_summary_json(&summaries, &[]);
    assert_eq!(
        value,
        json!([
            {
                "uid": "abc",
                "name": "CPU",
                "folder": "Infra",
                "folderUid": "infra",
                "path": "Platform / Infra",
                "org": "Main Org",
                "orgId": "1"
            },
            {
                "uid": "xyz",
                "name": "Overview",
                "folder": "General",
                "folderUid": "general",
                "path": "General",
                "org": "Main Org",
                "orgId": "1"
            }
        ])
    );
}

#[test]
fn render_dashboard_summary_json_includes_sources_when_present() {
    let summaries = vec![json!({
        "uid": "abc",
        "folderUid": "infra",
        "folderPath": "Platform / Infra",
        "folderTitle": "Infra",
        "orgId": 1,
        "orgName": "Main Org",
        "title": "CPU",
        "sources": ["Loki Logs", "Prom Main"],
        "sourceUids": ["loki_uid", "prom_uid"]
    })
    .as_object()
    .unwrap()
    .clone()];

    let value = render_dashboard_summary_json(&summaries, &[]);
    assert_eq!(
        value,
        json!([
            {
                "uid": "abc",
                "name": "CPU",
                "folder": "Infra",
                "folderUid": "infra",
                "path": "Platform / Infra",
                "org": "Main Org",
                "orgId": "1",
                "sources": ["Loki Logs", "Prom Main"],
                "sourceUids": ["loki_uid", "prom_uid"]
            }
        ])
    );
}

#[test]
fn render_dashboard_summary_table_respects_selected_columns() {
    let summaries = vec![json!({
        "uid": "abc",
        "folderUid": "infra",
        "folderPath": "Platform / Infra",
        "folderTitle": "Infra",
        "orgId": 1,
        "orgName": "Main Org",
        "title": "CPU",
        "sources": ["Prom Main"]
    })
    .as_object()
    .unwrap()
    .clone()];

    let lines = render_dashboard_summary_table(
        &summaries,
        &["uid".to_string(), "name".to_string(), "sources".to_string()],
        true,
    );
    assert!(lines[0].contains("UID"));
    assert!(lines[0].contains("NAME"));
    assert!(lines[0].contains("SOURCES"));
    assert!(lines[2].contains("abc"));
    assert!(lines[2].contains("CPU"));
    assert!(lines[2].contains("Prom Main"));
}

#[test]
fn render_dashboard_summary_json_respects_selected_columns() {
    let summaries = vec![json!({
        "uid": "abc",
        "folderUid": "infra",
        "folderPath": "Platform / Infra",
        "folderTitle": "Infra",
        "orgId": 1,
        "orgName": "Main Org",
        "title": "CPU",
        "sources": ["Loki Logs", "Prom Main"],
        "sourceUids": ["loki_uid", "prom_uid"]
    })
    .as_object()
    .unwrap()
    .clone()];

    let value = render_dashboard_summary_json(
        &summaries,
        &[
            "uid".to_string(),
            "org_id".to_string(),
            "source_uids".to_string(),
        ],
    );
    assert_eq!(
        value,
        json!([
            {
                "uid": "abc",
                "orgId": "1",
                "sourceUids": ["loki_uid", "prom_uid"]
            }
        ])
    );
}

#[test]
fn build_folder_path_joins_parents_and_title() {
    let folder = json!({
        "title": "Child",
        "parents": [{"title": "Root"}, {"title": "Team"}]
    });
    let path = build_folder_path(folder.as_object().unwrap(), "Child");
    assert_eq!(path, "Root / Team / Child");
}

#[test]
fn attach_dashboard_folder_paths_with_request_uses_folder_hierarchy() {
    let summaries = vec![
        json!({
            "uid": "abc",
            "folderUid": "child",
            "folderTitle": "Child",
            "title": "CPU"
        })
        .as_object()
        .unwrap()
        .clone(),
        json!({
            "uid": "xyz",
            "title": "Overview"
        })
        .as_object()
        .unwrap()
        .clone(),
    ];

    let enriched = attach_dashboard_folder_paths_with_request(
        |_method, path, _params, _payload| match path {
            "/api/folders/child" => Ok(Some(json!({
                "title": "Child",
                "parents": [{"title": "Root"}]
            }))),
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &summaries,
    )
    .unwrap();

    assert_eq!(enriched[0]["folderPath"], json!("Root / Child"));
    assert_eq!(enriched[1]["folderPath"], json!("General"));
}

#[test]
fn list_dashboards_with_request_returns_dashboard_count() {
    let args = ListArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        page_size: 500,
        org_id: None,
        all_orgs: false,
        with_sources: false,
        output_columns: Vec::new(),
        table: false,
        csv: false,
        json: false,
        output_format: None,
        no_header: false,
    };

    let mut calls = Vec::new();
    let count = list_dashboards_with_request(
        |method, path, _params, _payload| {
            calls.push((method.to_string(), path.to_string()));
            match path {
                "/api/search" => Ok(Some(json!([
                    {"uid": "abc", "title": "CPU", "folderTitle": "Infra", "folderUid": "infra"},
                    {"uid": "def", "title": "Memory", "folderTitle": "Infra"},
                ]))),
                "/api/org" => Ok(Some(json!({
                    "id": 1,
                    "name": "Main Org"
                }))),
                "/api/folders/infra" => Ok(Some(json!({
                    "title": "Infra",
                    "parents": [{"title": "Platform"}]
                }))),
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            }
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 2);
    assert_eq!(
        calls.iter().filter(|(_, path)| path == "/api/org").count(),
        1
    );
    assert!(!calls.iter().any(|(_, path)| path == "/api/datasources"));
    assert!(!calls
        .iter()
        .any(|(_, path)| path.starts_with("/api/dashboards/uid/")));
}

#[test]
fn collect_dashboard_source_names_prefers_datasource_names() {
    let payload = json!({
        "dashboard": {
            "uid": "abc",
            "title": "CPU",
            "panels": [
                {"datasource": {"uid": "prom_uid", "type": "prometheus"}},
                {"datasource": "Loki Logs"},
                {"datasource": "prometheus"},
                {"datasource": "-- Mixed --"}
            ]
        }
    });
    let catalog = test_support::build_datasource_catalog(&[
        json!({
            "uid": "prom_uid",
            "name": "Prom Main",
            "type": "prometheus",
            "pluginVersion": "11.0.0"
        })
        .as_object()
        .unwrap()
        .clone(),
        json!({
            "uid": "loki_uid",
            "name": "Loki Logs",
            "type": "loki",
            "meta": {"info": {"version": "3.1.0"}}
        })
        .as_object()
        .unwrap()
        .clone(),
    ]);

    let (sources, source_uids) =
        test_support::collect_dashboard_source_metadata(&payload, &catalog).unwrap();
    assert_eq!(
        sources,
        vec!["Loki Logs".to_string(), "Prom Main".to_string()]
    );
    assert_eq!(
        source_uids,
        vec!["loki_uid".to_string(), "prom_uid".to_string()]
    );
}

#[test]
fn collect_dashboard_source_names_accepts_preserved_raw_dashboard_documents() {
    let payload = json!({
        "uid": "abc",
        "title": "CPU",
        "panels": [
            {"datasource": {"uid": "prom_uid", "type": "prometheus"}},
            {"datasource": "Loki Logs"}
        ]
    });
    let catalog = test_support::build_datasource_catalog(&[
        json!({
            "uid": "prom_uid",
            "name": "Prom Main",
            "type": "prometheus"
        })
        .as_object()
        .unwrap()
        .clone(),
        json!({
            "uid": "loki_uid",
            "name": "Loki Logs",
            "type": "loki"
        })
        .as_object()
        .unwrap()
        .clone(),
    ]);

    let (sources, source_uids) =
        test_support::collect_dashboard_source_metadata(&payload, &catalog).unwrap();
    assert_eq!(
        sources,
        vec!["Loki Logs".to_string(), "Prom Main".to_string()]
    );
    assert_eq!(
        source_uids,
        vec!["loki_uid".to_string(), "prom_uid".to_string()]
    );
}

#[test]
fn list_dashboards_with_request_json_fetches_dashboards_and_datasources_by_default() {
    let args = ListArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        page_size: 500,
        org_id: None,
        all_orgs: false,
        with_sources: false,
        output_columns: Vec::new(),
        table: false,
        csv: false,
        json: true,
        output_format: None,
        no_header: false,
    };
    let mut calls = Vec::new();

    let count = list_dashboards_with_request(
        |method, path, _params, _payload| {
            calls.push((method.to_string(), path.to_string()));
            match path {
                "/api/search" => Ok(Some(json!([
                    {"uid": "abc", "title": "CPU", "folderTitle": "Infra", "folderUid": "infra"}
                ]))),
                "/api/org" => Ok(Some(json!({
                    "id": 1,
                    "name": "Main Org"
                }))),
                "/api/folders/infra" => Ok(Some(json!({
                    "title": "Infra",
                    "parents": [{"title": "Platform"}]
                }))),
                "/api/datasources" => Ok(Some(json!([
                    {"uid": "prom_uid", "name": "Prom Main", "type": "prometheus"}
                ]))),
                "/api/dashboards/uid/abc" => Ok(Some(json!({
                    "dashboard": {
                        "uid": "abc",
                        "title": "CPU",
                        "panels": [
                            {"datasource": {"uid": "prom_uid", "type": "prometheus"}}
                        ]
                    }
                }))),
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            }
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert_eq!(
        calls.iter().filter(|(_, path)| path == "/api/org").count(),
        1
    );
    assert!(calls.iter().any(|(_, path)| path == "/api/datasources"));
    assert!(calls
        .iter()
        .any(|(_, path)| path == "/api/dashboards/uid/abc"));
}

#[test]
fn list_dashboards_with_request_output_columns_sources_fetches_dashboard_sources() {
    let args = ListArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        page_size: 500,
        org_id: None,
        all_orgs: false,
        with_sources: false,
        output_columns: vec!["uid".to_string(), "sources".to_string()],
        table: true,
        csv: false,
        json: false,
        output_format: None,
        no_header: false,
    };
    let mut calls = Vec::new();

    let count = list_dashboards_with_request(
        |method, path, _params, _payload| {
            calls.push((method.to_string(), path.to_string()));
            match path {
                "/api/search" => Ok(Some(json!([
                    {"uid": "abc", "title": "CPU", "folderTitle": "Infra", "folderUid": "infra"}
                ]))),
                "/api/org" => Ok(Some(json!({
                    "id": 1,
                    "name": "Main Org"
                }))),
                "/api/folders/infra" => Ok(Some(json!({
                    "title": "Infra",
                    "parents": [{"title": "Platform"}]
                }))),
                "/api/datasources" => Ok(Some(json!([
                    {"uid": "prom_uid", "name": "Prom Main", "type": "prometheus"}
                ]))),
                "/api/dashboards/uid/abc" => Ok(Some(json!({
                    "dashboard": {
                        "uid": "abc",
                        "title": "CPU",
                        "panels": [
                            {"datasource": {"uid": "prom_uid", "type": "prometheus"}}
                        ]
                    }
                }))),
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            }
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert!(calls.iter().any(|(_, path)| path == "/api/datasources"));
    assert!(calls
        .iter()
        .any(|(_, path)| path == "/api/dashboards/uid/abc"));
}

#[test]
fn list_dashboards_with_request_with_org_id_scopes_requests() {
    let args = ListArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        page_size: 500,
        org_id: Some(7),
        all_orgs: false,
        with_sources: false,
        output_columns: Vec::new(),
        table: false,
        csv: false,
        json: true,
        output_format: None,
        no_header: false,
    };
    let mut calls = Vec::new();

    let count = list_dashboards_with_request(
        |method, path, params, _payload| {
            calls.push((method.to_string(), path.to_string(), params.to_vec()));
            let scoped_org = params
                .iter()
                .find(|(key, _)| key == "orgId")
                .map(|(_, value)| value.as_str());
            match (path, scoped_org) {
                ("/api/search", Some("7")) => Ok(Some(json!([
                    {"uid": "abc", "title": "CPU", "folderTitle": "Infra", "folderUid": "infra"}
                ]))),
                ("/api/org", Some("7")) => Ok(Some(json!({
                    "id": 7,
                    "name": "Scoped Org"
                }))),
                ("/api/folders/infra", Some("7")) => Ok(Some(json!({
                    "title": "Infra",
                    "parents": [{"title": "Platform"}]
                }))),
                ("/api/datasources", Some("7")) => Ok(Some(json!([
                    {"uid": "prom_uid", "name": "Prom Main", "type": "prometheus"}
                ]))),
                ("/api/dashboards/uid/abc", Some("7")) => Ok(Some(json!({
                    "dashboard": {
                        "uid": "abc",
                        "title": "CPU",
                        "panels": [
                            {"datasource": {"uid": "prom_uid", "type": "prometheus"}}
                        ]
                    }
                }))),
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            }
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert_eq!(
        calls
            .iter()
            .filter(|(_, path, params)| path == "/api/search"
                && params
                    .iter()
                    .any(|(key, value)| key == "orgId" && value == "7"))
            .count(),
        1
    );
    assert_eq!(
        calls
            .iter()
            .filter(|(_, path, params)| path == "/api/datasources"
                && params
                    .iter()
                    .any(|(key, value)| key == "orgId" && value == "7"))
            .count(),
        1
    );
    assert_eq!(
        calls
            .iter()
            .filter(|(_, path, params)| path == "/api/dashboards/uid/abc"
                && params
                    .iter()
                    .any(|(key, value)| key == "orgId" && value == "7"))
            .count(),
        1
    );
}

#[test]
fn list_dashboards_with_request_all_orgs_aggregates_results() {
    let args = ListArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        page_size: 500,
        org_id: None,
        all_orgs: true,
        with_sources: false,
        output_columns: Vec::new(),
        table: false,
        csv: false,
        json: true,
        output_format: None,
        no_header: false,
    };
    let mut calls = Vec::new();

    let count = list_dashboards_with_request(
        |method, path, params, _payload| {
            calls.push((method.to_string(), path.to_string(), params.to_vec()));
            let scoped_org = params
                .iter()
                .find(|(key, _)| key == "orgId")
                .map(|(_, value)| value.as_str());
            match (path, scoped_org) {
                ("/api/orgs", None) => Ok(Some(json!([
                    {"id": 1, "name": "Main Org"},
                    {"id": 2, "name": "Ops Org"}
                ]))),
                ("/api/search", Some("1")) => Ok(Some(json!([
                    {"uid": "abc", "title": "CPU", "folderTitle": "Infra", "folderUid": "infra"}
                ]))),
                ("/api/datasources", Some("1")) => Ok(Some(json!([
                    {"uid": "prom_uid", "name": "Prom Main", "type": "prometheus"}
                ]))),
                ("/api/search", Some("2")) => Ok(Some(json!([
                    {"uid": "xyz", "title": "Logs", "folderTitle": "Ops", "folderUid": "ops"}
                ]))),
                ("/api/datasources", Some("2")) => Ok(Some(json!([
                    {"uid": "loki_uid", "name": "Loki Logs", "type": "loki"}
                ]))),
                ("/api/folders/infra", Some("1")) => Ok(Some(json!({
                    "title": "Infra",
                    "parents": [{"title": "Platform"}]
                }))),
                ("/api/folders/ops", Some("2")) => Ok(Some(json!({
                    "title": "Ops",
                    "parents": [{"title": "Platform"}]
                }))),
                ("/api/dashboards/uid/abc", Some("1")) => Ok(Some(json!({
                    "dashboard": {
                        "uid": "abc",
                        "title": "CPU",
                        "panels": [
                            {"datasource": {"uid": "prom_uid", "type": "prometheus"}}
                        ]
                    }
                }))),
                ("/api/dashboards/uid/xyz", Some("2")) => Ok(Some(json!({
                    "dashboard": {
                        "uid": "xyz",
                        "title": "Logs",
                        "panels": [
                            {"datasource": {"uid": "loki_uid", "type": "loki"}}
                        ]
                    }
                }))),
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            }
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 2);
    assert_eq!(
        calls
            .iter()
            .filter(|(_, path, _)| path == "/api/orgs")
            .count(),
        1
    );
    assert_eq!(
        calls
            .iter()
            .filter(|(_, path, params)| path == "/api/search"
                && params
                    .iter()
                    .any(|(key, value)| key == "orgId" && value == "1"))
            .count(),
        1
    );
    assert_eq!(
        calls
            .iter()
            .filter(|(_, path, params)| path == "/api/search"
                && params
                    .iter()
                    .any(|(key, value)| key == "orgId" && value == "2"))
            .count(),
        1
    );
    assert_eq!(
        calls
            .iter()
            .filter(|(_, path, params)| path == "/api/datasources"
                && params
                    .iter()
                    .any(|(key, value)| key == "orgId" && value == "1"))
            .count(),
        1
    );
    assert_eq!(
        calls
            .iter()
            .filter(|(_, path, params)| path == "/api/datasources"
                && params
                    .iter()
                    .any(|(key, value)| key == "orgId" && value == "2"))
            .count(),
        1
    );
}

#[test]
fn export_dashboards_with_client_writes_raw_variant_and_indexes() {
    let temp = tempdir().unwrap();
    let args = ExportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        export_dir: temp.path().join("dashboards"),
        page_size: 500,
        org_id: None,
        all_orgs: false,
        flat: false,
        overwrite: true,
        without_dashboard_raw: false,
        without_dashboard_prompt: true,
        dry_run: false,
        progress: false,
        verbose: false,
    };
    let mut calls = Vec::new();
    let count = export_dashboards_with_request(
        |method, path, params, payload| {
            calls.push((
                method.to_string(),
                path.to_string(),
                params.to_vec(),
                payload.cloned(),
            ));
            if path == "/api/org" {
                return Ok(Some(json!({"id": 1, "name": "Main Org."})));
            }
            if path == "/api/search" {
                return Ok(Some(
                    json!([{ "uid": "abc", "title": "CPU", "folderTitle": "Infra" }]),
                ));
            }
            if path == "/api/datasources" {
                return Ok(Some(json!([
                    {"uid": "prom-main", "name": "Prometheus Main", "type": "prometheus", "url": "http://prometheus:9090", "access": "proxy", "isDefault": true}
                ])));
            }
            if path == "/api/dashboards/uid/abc" {
                return Ok(Some(
                    json!({
                        "dashboard": {
                            "id": 7,
                            "uid": "abc",
                            "title": "CPU",
                            "schemaVersion": 38,
                            "panels": [
                                {
                                    "id": 7,
                                    "title": "CPU Query",
                                    "type": "timeseries",
                                    "datasource": {"uid": "prom-main", "type": "prometheus"},
                                    "targets": [{"refId": "A", "expr": "up"}]
                                }
                            ]
                        }
                    }),
                ));
            }
            if path == "/api/dashboards/uid/abc/permissions" {
                return Ok(Some(json!([
                    {"userId": 11, "userLogin": "ops", "permission": 4, "inherited": false},
                    {"teamId": 21, "team": "SRE", "permission": 2, "inherited": false}
                ])));
            }
            Err(test_support::message(format!("unexpected path {path}")))
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert!(args.export_dir.join("raw/Infra/CPU__abc.json").is_file());
    assert!(args.export_dir.join("raw/index.json").is_file());
    assert!(args.export_dir.join("raw/export-metadata.json").is_file());
    assert!(args.export_dir.join("raw/permissions.json").is_file());
    assert!(args.export_dir.join("index.json").is_file());
    assert!(args.export_dir.join("export-metadata.json").is_file());
    let raw_metadata: Value = serde_json::from_str(
        &fs::read_to_string(args.export_dir.join("raw/export-metadata.json")).unwrap(),
    )
    .unwrap();
    let root_metadata: Value = serde_json::from_str(
        &fs::read_to_string(args.export_dir.join("export-metadata.json")).unwrap(),
    )
    .unwrap();
    let permission_bundle: Value = serde_json::from_str(
        &fs::read_to_string(
            args.export_dir
                .join("raw")
                .join(DASHBOARD_PERMISSION_BUNDLE_FILENAME),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(raw_metadata["org"], Value::String("Main Org.".to_string()));
    assert_eq!(raw_metadata["orgId"], Value::String("1".to_string()));
    assert_eq!(
        raw_metadata["permissionsFile"],
        Value::String("permissions.json".to_string())
    );
    assert_eq!(root_metadata["org"], Value::String("Main Org.".to_string()));
    assert_eq!(root_metadata["orgId"], Value::String("1".to_string()));
    assert_eq!(
        permission_bundle["summary"],
        json!({
            "resourceCount": 1,
            "dashboardCount": 1,
            "folderCount": 0,
            "permissionCount": 2
        })
    );
    assert_eq!(calls.len(), 5);

    let raw_dir = args.export_dir.join("raw");
    let report = test_support::build_export_inspection_query_report(&raw_dir).unwrap();
    assert_eq!(report.summary.dashboard_count, 1);
    assert_eq!(report.summary.query_count, 1);
    assert_eq!(report.queries[0].dashboard_uid, "abc");
    assert_eq!(report.queries[0].datasource_family, "prometheus");

    let mut import_args = make_import_args(raw_dir.clone());
    import_args.dry_run = false;
    let mut posted_payloads = Vec::new();
    let import_count = import_dashboards_with_request(
        with_dashboard_import_live_preflight(
            json!([
                {
                    "uid": "prom-main",
                    "name": "Prometheus Main",
                    "type": "prometheus",
                    "access": "proxy",
                    "url": "http://prometheus:9090",
                    "isDefault": true
                }
            ]),
            json!([
                {"id": "row"},
                {"id": "timeseries"}
            ]),
            |method, path, _params, payload| match (method, path) {
                (reqwest::Method::GET, "/api/org") => {
                    Ok(Some(json!({"id": 1, "name": "Main Org."})))
                }
                (reqwest::Method::GET, "/api/dashboards/uid/abc") => Err(api_response(
                    404,
                    "http://127.0.0.1:3000/api/dashboards/uid/abc",
                    "{\"message\":\"not found\"}",
                )),
                (reqwest::Method::POST, "/api/dashboards/db") => {
                    posted_payloads.push(payload.cloned().unwrap());
                    Ok(Some(json!({"status": "success"})))
                }
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            },
        ),
        &import_args,
    )
    .unwrap();
    assert_eq!(import_count, 1);
    assert_eq!(posted_payloads.len(), 1);
    assert_eq!(posted_payloads[0]["dashboard"]["uid"], "abc");

    let diff_args = DiffArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        import_dir: raw_dir,
        import_folder_uid: None,
        context_lines: 3,
    };
    let diff_count = diff_dashboards_with_request(
        |method, path, _params, _payload| match (method, path) {
            (reqwest::Method::GET, "/api/dashboards/uid/abc") => Ok(Some(json!({
                "dashboard": {
                    "id": 7,
                    "uid": "abc",
                    "title": "CPU",
                    "schemaVersion": 38,
                    "panels": [
                        {
                            "id": 7,
                            "title": "CPU Query",
                            "type": "timeseries",
                            "datasource": {"uid": "prom-main", "type": "prometheus"},
                            "targets": [{"refId": "A", "expr": "up"}]
                        }
                    ]
                }
            }))),
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &diff_args,
    )
    .unwrap();
    assert_eq!(diff_count, 0);
}

#[test]
fn export_dashboards_with_request_with_org_id_scopes_requests() {
    let temp = tempdir().unwrap();
    let args = ExportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        export_dir: temp.path().join("dashboards"),
        page_size: 500,
        org_id: Some(7),
        all_orgs: false,
        flat: false,
        overwrite: true,
        without_dashboard_raw: false,
        without_dashboard_prompt: true,
        dry_run: false,
        progress: false,
        verbose: false,
    };
    let mut calls = Vec::new();

    let count = export_dashboards_with_request(
        |method, path, params, payload| {
            calls.push((
                method.to_string(),
                path.to_string(),
                params.to_vec(),
                payload.cloned(),
            ));
            let scoped_org = params
                .iter()
                .find(|(key, _)| key == "orgId")
                .map(|(_, value)| value.as_str());
            match (path, scoped_org) {
                ("/api/org", Some("7")) => Ok(Some(json!({"id": 7, "name": "Scoped Org"}))),
                ("/api/search", Some("7")) => Ok(Some(
                    json!([{ "uid": "abc", "title": "CPU", "folderTitle": "Infra" }]),
                )),
                ("/api/datasources", Some("7")) => Ok(Some(json!([
                    {"uid": "prom-main", "name": "Prometheus Main", "type": "prometheus", "url": "http://prometheus:9090", "access": "proxy", "isDefault": true}
                ]))),
                ("/api/dashboards/uid/abc", Some("7")) => Ok(Some(
                    json!({"dashboard": {"id": 7, "uid": "abc", "title": "CPU"}}),
                )),
                ("/api/dashboards/uid/abc/permissions", Some("7")) => Ok(Some(json!([
                    {"userId": 11, "userLogin": "ops", "permission": 4}
                ]))),
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            }
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert!(args.export_dir.join("raw/Infra/CPU__abc.json").is_file());
    assert_eq!(
        calls
            .iter()
            .filter(|(_, path, params, _)| path == "/api/search"
                && params
                    .iter()
                    .any(|(key, value)| key == "orgId" && value == "7"))
            .count(),
        1
    );
}

#[test]
fn build_external_export_document_adds_datasource_inputs() {
    let payload = json!({
        "dashboard": {
            "id": 9,
            "title": "Infra",
            "panels": [
                {
                    "type": "timeseries",
                    "datasource": {"type": "prometheus", "uid": "prom_uid"},
                    "targets": [
                        {
                            "datasource": {"type": "prometheus", "uid": "prom_uid"},
                            "expr": "up"
                        }
                    ]
                },
                {
                    "type": "stat",
                    "datasource": "Loki Logs"
                }
            ]
        }
    });
    let catalog = test_support::build_datasource_catalog(&[
        json!({
            "uid": "prom_uid",
            "name": "Prom Main",
            "type": "prometheus",
            "pluginVersion": "11.0.0"
        })
        .as_object()
        .unwrap()
        .clone(),
        json!({
            "uid": "loki_uid",
            "name": "Loki Logs",
            "type": "loki",
            "meta": {"info": {"version": "3.1.0"}}
        })
        .as_object()
        .unwrap()
        .clone(),
    ]);

    let document = build_external_export_document(&payload, &catalog).unwrap();

    assert_eq!(
        document["panels"][0]["datasource"]["uid"],
        "${DS_PROM_MAIN}"
    );
    assert_eq!(
        document["panels"][0]["targets"][0]["datasource"]["uid"],
        "${DS_PROM_MAIN}"
    );
    assert_eq!(document["panels"][1]["datasource"], "${DS_LOKI_LOGS}");
    assert_eq!(document["__inputs"][0]["name"], "DS_LOKI_LOGS");
    assert_eq!(document["__inputs"][1]["name"], "DS_PROM_MAIN");
    assert_eq!(document["__inputs"][0]["label"], "Loki Logs");
    assert_eq!(document["__inputs"][1]["label"], "Prom Main");
    assert_eq!(document["__inputs"][0]["pluginName"], "Loki");
    assert_eq!(document["__inputs"][1]["pluginName"], "Prometheus");
    assert_eq!(
        document["__requires"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|item| item["type"] == "datasource")
            .map(|item| (
                item["id"].as_str().unwrap(),
                item["name"].as_str().unwrap(),
                item["version"].as_str().unwrap()
            ))
            .collect::<Vec<_>>(),
        vec![
            ("loki", "Loki", "3.1.0"),
            ("prometheus", "Prometheus", "11.0.0"),
        ]
    );
    assert_eq!(document["__elements"], json!({}));
}

#[test]
fn build_external_export_document_creates_input_from_datasource_template_variable() {
    let payload = json!({
        "dashboard": {
            "id": 15,
            "title": "Prometheus / Overview",
            "templating": {
                "list": [
                    {
                        "current": {"text": "default", "value": "default"},
                        "hide": 0,
                        "label": "Data source",
                        "name": "datasource",
                        "options": [],
                        "query": "prometheus",
                        "refresh": 1,
                        "regex": "",
                        "type": "datasource"
                    },
                    {
                        "allValue": ".+",
                        "current": {"selected": true, "text": "All", "value": "$__all"},
                        "datasource": "$datasource",
                        "includeAll": true,
                        "label": "job",
                        "multi": true,
                        "name": "job",
                        "options": [],
                        "query": "label_values(prometheus_build_info, job)",
                        "refresh": 1,
                        "regex": "",
                        "sort": 2,
                        "type": "query"
                    }
                ]
            },
            "panels": [
                {
                    "type": "timeseries",
                    "datasource": "$datasource",
                    "targets": [{"refId": "A", "expr": "up"}]
                }
            ]
        }
    });

    let catalog = test_support::build_datasource_catalog(&[]);
    let document = build_external_export_document(&payload, &catalog).unwrap();
    assert_eq!(document["__inputs"][0]["name"], "DS_PROMETHEUS");
    assert_eq!(document["templating"]["list"][0]["current"], json!({}));
    assert_eq!(document["templating"]["list"][0]["query"], "prometheus");
    assert_eq!(
        document["templating"]["list"][1]["datasource"]["uid"],
        "${DS_PROMETHEUS}"
    );
    assert_eq!(document["panels"][0]["datasource"]["uid"], "$datasource");
}

#[test]
fn build_external_export_document_deduplicates_same_type_datasource_requires() {
    let payload = json!({
        "dashboard": {
            "id": 18,
            "title": "Two Prometheus Query Dashboard",
            "panels": [
                {
                    "id": 1,
                    "type": "timeseries",
                    "title": "Two Prometheus Panel",
                    "datasource": {"type": "datasource", "uid": "-- Mixed --"},
                    "targets": [
                        {
                            "refId": "A",
                            "datasource": {"type": "prometheus", "uid": "prom_uid_1"},
                            "expr": "up"
                        },
                        {
                            "refId": "B",
                            "datasource": {"type": "prometheus", "uid": "prom_uid_2"},
                            "expr": "up"
                        }
                    ]
                }
            ]
        }
    });
    let catalog = test_support::build_datasource_catalog(&[
        json!({
            "uid": "prom_uid_1",
            "name": "Smoke Prometheus",
            "type": "prometheus",
            "pluginVersion": "11.0.0"
        })
        .as_object()
        .unwrap()
        .clone(),
        json!({
            "uid": "prom_uid_2",
            "name": "Smoke Prometheus 2",
            "type": "prometheus"
        })
        .as_object()
        .unwrap()
        .clone(),
    ]);

    let document = build_external_export_document(&payload, &catalog).unwrap();

    assert_eq!(
        document["panels"][0]["targets"][0]["datasource"]["uid"],
        "${DS_SMOKE_PROMETHEUS}"
    );
    assert_eq!(
        document["panels"][0]["targets"][1]["datasource"]["uid"],
        "${DS_SMOKE_PROMETHEUS_2}"
    );
    assert_eq!(
        document["__requires"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|item| item["type"] == "datasource")
            .map(|item| (
                item["id"].as_str().unwrap(),
                item["name"].as_str().unwrap(),
                item["version"].as_str().unwrap()
            ))
            .collect::<Vec<_>>(),
        vec![("prometheus", "Prometheus", "11.0.0")]
    );
}

#[test]
fn build_external_export_document_uses_grafana_style_plugin_names_for_inputs_and_requires() {
    let payload = json!({
        "dashboard": {
            "id": 19,
            "title": "Search And SQL",
            "panels": [
                {"type": "table", "datasource": "logs-search"},
                {"type": "stat", "datasource": "reporting-db"}
            ]
        }
    });
    let catalog = test_support::build_datasource_catalog(&[
        json!({
            "uid": "search_uid",
            "name": "logs-search",
            "type": "grafana-opensearch-datasource",
            "pluginVersion": "2.25.0"
        })
        .as_object()
        .unwrap()
        .clone(),
        json!({
            "uid": "sql_uid",
            "name": "reporting-db",
            "type": "postgres",
            "meta": {"info": {"version": "1.0.0"}}
        })
        .as_object()
        .unwrap()
        .clone(),
    ]);

    let document = build_external_export_document(&payload, &catalog).unwrap();

    assert_eq!(document["__inputs"][0]["label"], "logs-search");
    assert_eq!(document["__inputs"][0]["pluginName"], "OpenSearch");
    assert_eq!(document["__inputs"][1]["label"], "reporting-db");
    assert_eq!(document["__inputs"][1]["pluginName"], "PostgreSQL");
    assert_eq!(
        document["__requires"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|item| item["type"] == "datasource")
            .map(|item| (
                item["id"].as_str().unwrap(),
                item["name"].as_str().unwrap(),
                item["version"].as_str().unwrap()
            ))
            .collect::<Vec<_>>(),
        vec![
            ("grafana-opensearch-datasource", "OpenSearch", "2.25.0"),
            ("postgres", "PostgreSQL", "1.0.0"),
        ]
    );
}

#[test]
fn build_external_export_document_keeps_distinct_same_type_object_reference_names() {
    let payload = json!({
        "dashboard": {
            "id": 20,
            "title": "Same Type Object References",
            "panels": [
                {
                    "type": "timeseries",
                    "datasource": {"type": "datasource", "uid": "-- Mixed --"},
                    "targets": [
                        {
                            "refId": "A",
                            "datasource": {
                                "type": "prometheus",
                                "uid": "prom_uid_1",
                                "name": "Regional Prometheus"
                            },
                            "expr": "up"
                        },
                        {
                            "refId": "B",
                            "datasource": {
                                "type": "prometheus",
                                "uid": "prom_uid_2",
                                "name": "Infra Prometheus"
                            },
                            "expr": "up"
                        }
                    ]
                }
            ]
        }
    });
    let catalog = test_support::build_datasource_catalog(&[
        json!({
            "uid": "prom_uid_1",
            "name": "Regional Prometheus",
            "type": "prometheus",
            "pluginVersion": "11.0.0"
        })
        .as_object()
        .unwrap()
        .clone(),
        json!({
            "uid": "prom_uid_2",
            "name": "Infra Prometheus",
            "type": "prometheus"
        })
        .as_object()
        .unwrap()
        .clone(),
    ]);

    let document = build_external_export_document(&payload, &catalog).unwrap();

    assert_eq!(document["__inputs"][0]["label"], "Infra Prometheus");
    assert_eq!(document["__inputs"][0]["name"], "DS_INFRA_PROMETHEUS");
    assert_eq!(document["__inputs"][1]["label"], "Regional Prometheus");
    assert_eq!(document["__inputs"][1]["name"], "DS_REGIONAL_PROMETHEUS");
    assert_eq!(
        document["panels"][0]["targets"][0]["datasource"]["uid"],
        "${DS_REGIONAL_PROMETHEUS}"
    );
    assert_eq!(
        document["panels"][0]["targets"][1]["datasource"]["uid"],
        "${DS_INFRA_PROMETHEUS}"
    );
}

#[test]
fn build_external_export_document_matches_shared_prompt_export_fixture_cases() {
    for case in load_prompt_export_cases() {
        let case_name = case["name"].as_str().unwrap();
        let payload = case["payload"].clone();
        let catalog_items = case["catalog"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_object().unwrap().clone())
            .collect::<Vec<_>>();
        let catalog = test_support::build_datasource_catalog(&catalog_items);
        let document = build_external_export_document(&payload, &catalog).unwrap();

        assert_eq!(
            document["__inputs"], case["expectedInputs"],
            "case={case_name}"
        );
        assert_eq!(
            document["__requires"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|item| item["type"] == "datasource")
                .cloned()
                .collect::<Vec<Value>>(),
            case["expectedDatasourceRequires"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| {
                    json!({
                        "type": "datasource",
                        "id": item["id"],
                        "name": item["name"],
                        "version": item["version"],
                    })
                })
                .collect::<Vec<Value>>(),
            "case={case_name}"
        );
        assert_eq!(
            document["__requires"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|item| item["type"] == "panel")
                .cloned()
                .collect::<Vec<Value>>(),
            case["expectedPanelRequires"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| {
                    json!({
                        "type": "panel",
                        "id": item["id"],
                        "name": item["name"],
                        "version": item["version"],
                    })
                })
                .collect::<Vec<Value>>(),
            "case={case_name}"
        );
    }
}

#[test]
fn resolve_query_analyzer_family_uses_string_prometheus_datasource_refs() {
    let panel = Map::from_iter(vec![(
        "datasource".to_string(),
        Value::String("prometheus".to_string()),
    )]);
    let target = Map::from_iter(vec![
        (
            "datasource".to_string(),
            Value::String("prometheus".to_string()),
        ),
        ("query".to_string(), Value::String("up".to_string())),
    ]);
    let context = test_support::QueryExtractionContext {
        panel: &panel,
        target: &target,
        query_field: "query",
        query_text: "up",
        resolved_datasource_type: "",
    };

    assert_eq!(
        test_support::resolve_query_analyzer_family(&context),
        test_support::DATASOURCE_FAMILY_PROMETHEUS
    );
    assert_eq!(
        test_support::dispatch_query_analysis(&context).metrics,
        vec!["up".to_string()]
    );
}

#[test]
fn resolve_query_analyzer_family_uses_string_loki_datasource_refs() {
    let panel = Map::from_iter(vec![(
        "datasource".to_string(),
        Value::String("loki".to_string()),
    )]);
    let target = Map::from_iter(vec![
        ("datasource".to_string(), Value::String("loki".to_string())),
        (
            "query".to_string(),
            Value::String("{job=\"grafana\"} |= \"error\"".to_string()),
        ),
    ]);
    let context = test_support::QueryExtractionContext {
        panel: &panel,
        target: &target,
        query_field: "query",
        query_text: "{job=\"grafana\"} |= \"error\"",
        resolved_datasource_type: "",
    };

    assert_eq!(
        test_support::resolve_query_analyzer_family(&context),
        test_support::DATASOURCE_FAMILY_LOKI
    );
    assert_eq!(
        test_support::dispatch_query_analysis(&context).measurements,
        vec![
            "{job=\"grafana\"}".to_string(),
            "job=\"grafana\"".to_string()
        ]
    );
}

#[test]
fn resolve_query_analyzer_family_uses_plugin_id_datasource_refs() {
    let panel = Map::from_iter(vec![(
        "datasource".to_string(),
        json!({"pluginId": "grafana-postgresql-datasource"}),
    )]);
    let target = Map::from_iter(vec![
        (
            "datasource".to_string(),
            json!({"pluginId": "grafana-postgresql-datasource"}),
        ),
        (
            "query".to_string(),
            Value::String("custom.metric".to_string()),
        ),
    ]);
    let context = test_support::QueryExtractionContext {
        panel: &panel,
        target: &target,
        query_field: "query",
        query_text: "custom.metric",
        resolved_datasource_type: "",
    };

    assert_eq!(
        test_support::resolve_query_analyzer_family(&context),
        test_support::DATASOURCE_FAMILY_SQL
    );
}

#[test]
fn resolve_query_analyzer_family_ignores_placeholder_string_datasource_refs() {
    let panel = Map::from_iter(vec![(
        "datasource".to_string(),
        Value::String("${DS_PROM_MAIN}".to_string()),
    )]);
    let target = Map::from_iter(vec![
        (
            "datasource".to_string(),
            Value::String("${DS_PROM_MAIN}".to_string()),
        ),
        (
            "query".to_string(),
            Value::String("custom.metric".to_string()),
        ),
    ]);
    let context = test_support::QueryExtractionContext {
        panel: &panel,
        target: &target,
        query_field: "query",
        query_text: "custom.metric",
        resolved_datasource_type: "",
    };

    assert_eq!(
        test_support::resolve_query_analyzer_family(&context),
        test_support::DATASOURCE_FAMILY_UNKNOWN
    );
}

#[test]
fn export_dashboards_with_client_writes_prompt_variant_and_indexes() {
    let temp = tempdir().unwrap();
    let args = ExportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        export_dir: temp.path().join("dashboards"),
        page_size: 500,
        org_id: None,
        all_orgs: false,
        flat: false,
        overwrite: true,
        without_dashboard_raw: false,
        without_dashboard_prompt: false,
        dry_run: false,
        progress: false,
        verbose: false,
    };

    let count = export_dashboards_with_request(
        |_method, path, _params, _payload| match path {
            "/api/datasources" => Ok(Some(json!([
                {"uid": "prom_uid", "name": "Prom Main", "type": "prometheus"}
            ]))),
            "/api/org" => Ok(Some(json!({"id": 1, "name": "Main Org."}))),
            "/api/search" => Ok(Some(json!([{ "uid": "abc", "title": "CPU", "folderTitle": "Infra" }]))),
            "/api/dashboards/uid/abc" => Ok(Some(json!({
                "dashboard": {
                    "id": 7,
                    "uid": "abc",
                    "title": "CPU",
                    "panels": [
                        {"type": "timeseries", "datasource": {"type": "prometheus", "uid": "prom_uid"}}
                    ]
                }
            }))),
            "/api/dashboards/uid/abc/permissions" => Ok(Some(json!([
                {"userId": 11, "userLogin": "ops", "permission": 4}
            ]))),
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert!(args.export_dir.join("prompt/Infra/CPU__abc.json").is_file());
    assert!(args.export_dir.join("prompt/index.json").is_file());
    assert!(args
        .export_dir
        .join("prompt/export-metadata.json")
        .is_file());
}

#[test]
fn export_dashboards_with_request_all_orgs_aggregates_results() {
    let temp = tempdir().unwrap();
    let args = ExportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        export_dir: temp.path().join("dashboards"),
        page_size: 500,
        org_id: None,
        all_orgs: true,
        flat: false,
        overwrite: true,
        without_dashboard_raw: false,
        without_dashboard_prompt: true,
        dry_run: false,
        progress: false,
        verbose: false,
    };
    let mut calls = Vec::new();

    let count = export_dashboards_with_request(
        |method, path, params, payload| {
            calls.push((
                method.to_string(),
                path.to_string(),
                params.to_vec(),
                payload.cloned(),
            ));
            let scoped_org = params
                .iter()
                .find(|(key, _)| key == "orgId")
                .map(|(_, value)| value.as_str());
            match (path, scoped_org) {
                ("/api/orgs", None) => Ok(Some(json!([
                    {"id": 1, "name": "Main Org"},
                    {"id": 2, "name": "Ops Org"}
                ]))),
                ("/api/org", Some("1")) => Ok(Some(json!({"id": 1, "name": "Main Org"}))),
                ("/api/org", Some("2")) => Ok(Some(json!({"id": 2, "name": "Ops Org"}))),
                ("/api/search", Some("1")) => Ok(Some(
                    json!([{ "uid": "abc", "title": "CPU", "folderTitle": "Infra" }]),
                )),
                ("/api/datasources", Some("1")) => Ok(Some(json!([
                    {"uid": "prom-main", "name": "Prometheus Main", "type": "prometheus", "url": "http://prometheus:9090", "access": "proxy", "isDefault": true}
                ]))),
                ("/api/search", Some("2")) => Ok(Some(
                    json!([{ "uid": "xyz", "title": "Logs", "folderTitle": "Ops" }]),
                )),
                ("/api/datasources", Some("2")) => Ok(Some(json!([
                    {"uid": "logs-main", "name": "Logs Main", "type": "loki", "url": "http://loki:3100", "access": "proxy", "isDefault": false}
                ]))),
                ("/api/dashboards/uid/abc", Some("1")) => Ok(Some(
                    json!({"dashboard": {"id": 7, "uid": "abc", "title": "CPU", "panels": [
                        {"datasource": {"uid": "prom-main", "type": "prometheus"}}
                    ]}}),
                )),
                ("/api/dashboards/uid/xyz", Some("2")) => Ok(Some(
                    json!({"dashboard": {"id": 8, "uid": "xyz", "title": "Logs", "panels": [
                        {"datasource": {"uid": "logs-main", "type": "loki"}}
                    ]}}),
                )),
                ("/api/dashboards/uid/abc/permissions", Some("1")) => Ok(Some(json!([
                    {"userId": 11, "userLogin": "ops", "permission": 4}
                ]))),
                ("/api/dashboards/uid/xyz/permissions", Some("2")) => Ok(Some(json!([
                    {"teamId": 21, "team": "SRE", "permission": 2}
                ]))),
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            }
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 2);
    assert!(args
        .export_dir
        .join("org_1_Main_Org/raw/Infra/CPU__abc.json")
        .is_file());
    assert!(args
        .export_dir
        .join("org_1_Main_Org/raw/index.json")
        .is_file());
    assert!(args
        .export_dir
        .join("org_1_Main_Org/raw/export-metadata.json")
        .is_file());
    assert!(args
        .export_dir
        .join("org_1_Main_Org/raw/folders.json")
        .is_file());
    assert!(args
        .export_dir
        .join("org_1_Main_Org/raw/datasources.json")
        .is_file());
    assert!(args
        .export_dir
        .join("org_1_Main_Org/raw/permissions.json")
        .is_file());
    assert!(args
        .export_dir
        .join("org_2_Ops_Org/raw/Ops/Logs__xyz.json")
        .is_file());
    assert!(args
        .export_dir
        .join("org_2_Ops_Org/raw/index.json")
        .is_file());
    assert!(args
        .export_dir
        .join("org_2_Ops_Org/raw/permissions.json")
        .is_file());
    let aggregate_root_index: Value =
        serde_json::from_str(&fs::read_to_string(args.export_dir.join("index.json")).unwrap())
            .unwrap();
    let aggregate_root_metadata: Value = serde_json::from_str(
        &fs::read_to_string(args.export_dir.join("export-metadata.json")).unwrap(),
    )
    .unwrap();
    let org_one_metadata: Value = serde_json::from_str(
        &fs::read_to_string(
            args.export_dir
                .join("org_1_Main_Org/raw/export-metadata.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let org_two_metadata: Value = serde_json::from_str(
        &fs::read_to_string(
            args.export_dir
                .join("org_2_Ops_Org/raw/export-metadata.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        org_one_metadata["org"],
        Value::String("Main Org".to_string())
    );
    assert_eq!(org_one_metadata["orgId"], Value::String("1".to_string()));
    assert_eq!(
        org_one_metadata["permissionsFile"],
        Value::String("permissions.json".to_string())
    );
    assert_eq!(
        org_two_metadata["org"],
        Value::String("Ops Org".to_string())
    );
    assert_eq!(org_two_metadata["orgId"], Value::String("2".to_string()));
    assert_eq!(
        org_two_metadata["permissionsFile"],
        Value::String("permissions.json".to_string())
    );
    assert_eq!(aggregate_root_index["items"].as_array().unwrap().len(), 2);
    assert!(aggregate_root_index["variants"]["raw"].is_null());
    assert_eq!(
        aggregate_root_index["items"][0]["raw_path"],
        Value::String(
            args.export_dir
                .join("org_1_Main_Org/raw/Infra/CPU__abc.json")
                .display()
                .to_string()
        )
    );
    assert_eq!(
        aggregate_root_index["items"][1]["raw_path"],
        Value::String(
            args.export_dir
                .join("org_2_Ops_Org/raw/Ops/Logs__xyz.json")
                .display()
                .to_string()
        )
    );
    assert_eq!(
        aggregate_root_metadata["variant"],
        Value::String("root".to_string())
    );
    assert_eq!(
        aggregate_root_metadata["indexFile"],
        Value::String("index.json".to_string())
    );
    assert_eq!(aggregate_root_metadata["orgCount"], Value::Number(2.into()));
    assert_eq!(aggregate_root_metadata["orgs"].as_array().unwrap().len(), 2);
    let org_entries = aggregate_root_metadata["orgs"].as_array().unwrap();
    let org_one_entry = org_entries
        .iter()
        .find(|entry| entry["orgId"] == Value::String("1".to_string()))
        .unwrap();
    let org_two_entry = org_entries
        .iter()
        .find(|entry| entry["orgId"] == Value::String("2".to_string()))
        .unwrap();
    assert_eq!(
        org_one_entry["usedDatasourceCount"],
        Value::Number(1.into())
    );
    assert_eq!(
        org_one_entry["exportDir"],
        Value::String(args.export_dir.join("org_1_Main_Org").display().to_string())
    );
    assert_eq!(
        org_one_entry["usedDatasources"][0]["uid"],
        Value::String("prom-main".to_string())
    );
    assert_eq!(
        org_two_entry["usedDatasourceCount"],
        Value::Number(1.into())
    );
    assert_eq!(
        org_two_entry["exportDir"],
        Value::String(args.export_dir.join("org_2_Ops_Org").display().to_string())
    );
    assert_eq!(
        org_two_entry["usedDatasources"][0]["uid"],
        Value::String("logs-main".to_string())
    );
    assert_eq!(
        calls
            .iter()
            .filter(|(_, path, _, _)| path == "/api/orgs")
            .count(),
        1
    );
    assert_eq!(
        calls
            .iter()
            .filter(|(_, path, params, _)| path == "/api/search"
                && params
                    .iter()
                    .any(|(key, value)| key == "orgId" && value == "1"))
            .count(),
        1
    );
    assert_eq!(
        calls
            .iter()
            .filter(|(_, path, params, _)| path == "/api/search"
                && params
                    .iter()
                    .any(|(key, value)| key == "orgId" && value == "2"))
            .count(),
        1
    );
}

#[test]
fn export_dashboards_with_dry_run_keeps_output_dir_empty() {
    let temp = tempdir().unwrap();
    let args = ExportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        export_dir: temp.path().join("dashboards"),
        page_size: 500,
        org_id: None,
        all_orgs: false,
        flat: false,
        overwrite: true,
        without_dashboard_raw: false,
        without_dashboard_prompt: true,
        dry_run: true,
        progress: false,
        verbose: false,
    };

    let count = export_dashboards_with_request(
        |_method, path, _params, _payload| match path {
            "/api/org" => Ok(Some(json!({"id": 1, "name": "Main Org."}))),
            "/api/search" => Ok(Some(
                json!([{ "uid": "abc", "title": "CPU", "folderTitle": "Infra" }]),
            )),
            "/api/datasources" => Ok(Some(json!([
                {"uid": "prom-main", "name": "Prometheus Main", "type": "prometheus", "url": "http://prometheus:9090", "access": "proxy", "isDefault": true}
            ]))),
            "/api/dashboards/uid/abc" => Ok(Some(
                json!({"dashboard": {"id": 7, "uid": "abc", "title": "CPU"}}),
            )),
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert!(!args.export_dir.exists());
}

#[test]
fn build_export_inspection_summary_reports_structure_and_datasources() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(raw_dir.join("General")).unwrap();
    fs::create_dir_all(raw_dir.join("Prod")).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 2,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid",
            "foldersFile": FOLDER_INVENTORY_FILENAME,
            "datasourcesFile": DATASOURCE_INVENTORY_FILENAME,
            "org": "Main Org.",
            "orgId": "1"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join(FOLDER_INVENTORY_FILENAME),
        serde_json::to_string_pretty(&json!([
            {
                "uid": "prod",
                "title": "Prod",
                "parentUid": "apps",
                "path": "Platform / Team / Apps / Prod",
                "org": "Main Org.",
                "orgId": "1"
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join(DATASOURCE_INVENTORY_FILENAME),
        serde_json::to_string_pretty(&json!([
            {
                "uid": "loki-a",
                "name": "Logs Main",
                "type": "loki",
                "access": "proxy",
                "url": "http://loki:3100",
                "isDefault": "false",
                "org": "Main Org.",
                "orgId": "1"
            },
            {
                "uid": "prom-a",
                "name": "Prometheus Main",
                "type": "prometheus",
                "access": "proxy",
                "url": "http://prometheus:9090",
                "isDefault": "true",
                "org": "Main Org.",
                "orgId": "1"
            },
            {
                "uid": "unused-main",
                "name": "Unused Main",
                "type": "tempo",
                "access": "proxy",
                "url": "http://tempo:3200",
                "isDefault": "false",
                "org": "Main Org.",
                "orgId": "1"
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("General").join("main.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "uid": "main",
                "title": "Main",
                "panels": [
                    {
                        "id": 1,
                        "type": "timeseries",
                        "datasource": {"uid": "prom-a", "type": "prometheus"},
                        "targets": [
                            {"refId": "A", "datasource": {"uid": "prom-a", "type": "prometheus"}}
                        ]
                    }
                ]
            }
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("Prod").join("mixed.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "uid": "mixed",
                "title": "Mixed",
                "panels": [
                    {
                        "id": 2,
                        "type": "timeseries",
                        "targets": [
                            {"refId": "A", "datasource": {"uid": "prom-a", "type": "prometheus"}},
                            {"refId": "B", "datasource": {"uid": "loki-a", "type": "loki"}}
                        ]
                    }
                ]
            },
            "meta": {"folderUid": "prod"}
        }))
        .unwrap(),
    )
    .unwrap();

    let summary = test_support::build_export_inspection_summary(&raw_dir).unwrap();

    assert_eq!(summary.export_org, Some("Main Org.".to_string()));
    assert_eq!(summary.export_org_id, Some("1".to_string()));
    assert_eq!(summary.dashboard_count, 2);
    assert_eq!(summary.folder_count, 2);
    assert_eq!(summary.panel_count, 2);
    assert_eq!(summary.query_count, 3);
    assert_eq!(summary.datasource_inventory_count, 3);
    assert_eq!(summary.orphaned_datasource_count, 1);
    assert_eq!(summary.mixed_dashboard_count, 1);
    assert!(summary
        .folder_paths
        .iter()
        .any(|item| item.path == "General" && item.dashboards == 1));
    assert!(summary
        .folder_paths
        .iter()
        .any(|item| item.path == "Platform / Team / Apps / Prod"));
    let prom_usage = summary
        .datasource_usage
        .iter()
        .find(|item| item.datasource == "prom-a")
        .unwrap();
    assert_eq!(prom_usage.reference_count, 3);
    assert_eq!(prom_usage.dashboard_count, 2);
    assert_eq!(summary.datasource_inventory[0].dashboard_count, 1);
    assert_eq!(summary.datasource_inventory[1].reference_count, 3);
    assert_eq!(summary.orphaned_datasources.len(), 1);
    assert_eq!(summary.orphaned_datasources[0].uid, "unused-main");
    assert_eq!(summary.mixed_dashboards[0].uid, "mixed");

    let summary_document = test_support::build_export_inspection_summary_document(&summary);
    assert_eq!(
        summary_document.summary.export_org.as_deref(),
        Some("Main Org.")
    );
    assert_eq!(summary_document.summary.export_org_id.as_deref(), Some("1"));
    assert_eq!(summary_document.summary.dashboard_count, 2);
    assert_eq!(summary_document.summary.folder_count, 2);
    assert_eq!(summary_document.summary.query_count, 3);
    assert_eq!(summary_document.datasource_inventory[1].reference_count, 3);
    assert_eq!(summary_document.orphaned_datasources[0].uid, "unused-main");
    assert_eq!(
        summary_document.mixed_datasource_dashboards[0].folder_path,
        "Platform / Team / Apps / Prod"
    );
}

#[test]
fn build_export_inspection_summary_rows_include_export_org_metadata() {
    let summary = test_support::ExportInspectionSummary {
        import_dir: "/tmp/raw".to_string(),
        export_org: Some("Main Org.".to_string()),
        export_org_id: Some("1".to_string()),
        dashboard_count: 2,
        folder_count: 2,
        panel_count: 3,
        query_count: 4,
        datasource_inventory_count: 3,
        orphaned_datasource_count: 1,
        mixed_dashboard_count: 1,
        folder_paths: Vec::new(),
        datasource_usage: Vec::new(),
        datasource_inventory: Vec::new(),
        orphaned_datasources: Vec::new(),
        mixed_dashboards: Vec::new(),
    };

    let rows = test_support::build_export_inspection_summary_rows(&summary);

    assert!(rows.contains(&vec!["export_org".to_string(), "Main Org.".to_string()]));
    assert!(rows.contains(&vec!["export_org_id".to_string(), "1".to_string()]));
    assert!(rows.contains(&vec!["dashboard_count".to_string(), "2".to_string()]));
}

#[test]
fn build_export_inspection_summary_uses_unique_folder_title_fallback_for_full_path() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(raw_dir.join("Infra")).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid",
            "foldersFile": FOLDER_INVENTORY_FILENAME
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join(FOLDER_INVENTORY_FILENAME),
        serde_json::to_string_pretty(&json!([
            {
                "uid": "infra",
                "title": "Infra",
                "parentUid": "platform",
                "path": "Platform / Infra",
                "org": "Main Org.",
                "orgId": "1"
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("Infra").join("sub.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "uid": "sub",
                "title": "Sub",
                "panels": []
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let summary = test_support::build_export_inspection_summary(&raw_dir).unwrap();

    assert_eq!(summary.folder_paths[0].path, "Platform / Infra");
}

#[test]
fn build_export_inspection_summary_includes_zero_dashboard_ancestor_paths() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(raw_dir.join("Prod")).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid",
            "foldersFile": FOLDER_INVENTORY_FILENAME
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join(FOLDER_INVENTORY_FILENAME),
        serde_json::to_string_pretty(&json!([
            {
                "uid": "platform",
                "title": "Platform",
                "parentUid": null,
                "path": "Platform",
                "org": "Main Org.",
                "orgId": "1"
            },
            {
                "uid": "team",
                "title": "Team",
                "parentUid": "platform",
                "path": "Platform / Team",
                "org": "Main Org.",
                "orgId": "1"
            },
            {
                "uid": "apps",
                "title": "Apps",
                "parentUid": "team",
                "path": "Platform / Team / Apps",
                "org": "Main Org.",
                "orgId": "1"
            },
            {
                "uid": "prod",
                "title": "Prod",
                "parentUid": "apps",
                "path": "Platform / Team / Apps / Prod",
                "org": "Main Org.",
                "orgId": "1"
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("Prod").join("prod.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "uid": "prod-main",
                "title": "Prod Main",
                "panels": []
            },
            "meta": {"folderUid": "prod"}
        }))
        .unwrap(),
    )
    .unwrap();

    let summary = test_support::build_export_inspection_summary(&raw_dir).unwrap();
    let paths = summary
        .folder_paths
        .iter()
        .map(|item| (item.path.clone(), item.dashboards))
        .collect::<Vec<(String, usize)>>();

    assert_eq!(
        paths,
        vec![
            ("Platform".to_string(), 0),
            ("Platform / Team".to_string(), 0),
            ("Platform / Team / Apps".to_string(), 0),
            ("Platform / Team / Apps / Prod".to_string(), 1),
        ]
    );
}

#[test]
fn build_export_inspection_query_report_emits_search_family_for_inventory_backed_elasticsearch_and_opensearch_rows(
) {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(raw_dir.join("General")).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid",
            "datasourcesFile": DATASOURCE_INVENTORY_FILENAME
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join(DATASOURCE_INVENTORY_FILENAME),
        serde_json::to_string_pretty(&json!([
            {
                "uid": "elastic-main",
                "name": "Elastic Main",
                "type": "elasticsearch",
                "access": "proxy",
                "url": "http://elasticsearch:9200",
                "indexPattern": "[logs-]YYYY.MM.DD",
                "isDefault": "false",
                "org": "Main Org.",
                "orgId": "1"
            },
            {
                "uid": "opensearch-main",
                "name": "OpenSearch Main",
                "type": "grafana-opensearch-datasource",
                "access": "proxy",
                "url": "http://opensearch:9200",
                "indexPattern": "logs-*",
                "isDefault": "false",
                "org": "Main Org.",
                "orgId": "1"
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("General").join("search.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "uid": "search-main",
                "title": "Search Main",
                "panels": [
                    {
                        "id": 7,
                        "title": "Elastic Query",
                        "type": "table",
                        "targets": [
                            {
                                "refId": "A",
                                "datasource": {"uid": "elastic-main"},
                                "query": "status:500 AND _exists_:trace.id AND service.name:\"api\""
                            }
                        ]
                    },
                    {
                        "id": 8,
                        "title": "OpenSearch Query",
                        "type": "table",
                        "targets": [
                            {
                                "refId": "B",
                                "datasource": {"uid": "opensearch-main"},
                                "query": "_exists_:host.name AND host.name:api AND response.status:404 AND category:\"auth\""
                            }
                        ]
                    }
                ]
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let report = test_support::build_export_inspection_query_report(&raw_dir).unwrap();
    let row = |ref_id: &str| {
        report
            .queries
            .iter()
            .find(|query| query.ref_id == ref_id)
            .unwrap()
    };

    assert_eq!(report.summary.dashboard_count, 1);
    assert_eq!(report.summary.panel_count, 2);
    assert_eq!(report.summary.query_count, 2);
    assert_eq!(report.summary.report_row_count, 2);

    let elastic = row("A");
    assert_eq!(elastic.datasource, "elastic-main");
    assert_eq!(elastic.datasource_name, "Elastic Main");
    assert_eq!(elastic.datasource_uid, "elastic-main");
    assert_eq!(elastic.datasource_type, "elasticsearch");
    assert_eq!(elastic.datasource_family, "search");
    assert_eq!(elastic.datasource_org, "Main Org.");
    assert_eq!(elastic.datasource_org_id, "1");
    assert_eq!(elastic.datasource_index_pattern, "[logs-]YYYY.MM.DD");
    assert_eq!(elastic.query_field, "query");
    assert_eq!(elastic.metrics, Vec::<String>::new());
    assert_eq!(elastic.functions, Vec::<String>::new());
    assert_eq!(elastic.buckets, Vec::<String>::new());
    assert_eq!(
        elastic.measurements,
        vec![
            "trace.id".to_string(),
            "status".to_string(),
            "service.name".to_string(),
        ]
    );

    let opensearch = row("B");
    assert_eq!(opensearch.datasource, "opensearch-main");
    assert_eq!(opensearch.datasource_name, "OpenSearch Main");
    assert_eq!(opensearch.datasource_uid, "opensearch-main");
    assert_eq!(opensearch.datasource_type, "grafana-opensearch-datasource");
    assert_eq!(opensearch.datasource_family, "search");
    assert_eq!(opensearch.datasource_org, "Main Org.");
    assert_eq!(opensearch.datasource_org_id, "1");
    assert_eq!(opensearch.datasource_index_pattern, "logs-*");
    assert_eq!(opensearch.query_field, "query");
    assert_eq!(opensearch.metrics, Vec::<String>::new());
    assert_eq!(opensearch.functions, Vec::<String>::new());
    assert_eq!(opensearch.buckets, Vec::<String>::new());
    assert_eq!(
        opensearch.measurements,
        vec![
            "host.name".to_string(),
            "response.status".to_string(),
            "category".to_string(),
        ]
    );

    let filtered = test_support::apply_query_report_filters(report.clone(), Some("search"), None);
    assert_eq!(filtered.summary.dashboard_count, 1);
    assert_eq!(filtered.summary.panel_count, 2);
    assert_eq!(filtered.summary.query_count, 2);
    assert_eq!(filtered.summary.report_row_count, 2);
    assert_eq!(filtered.queries.len(), 2);
    assert!(filtered
        .queries
        .iter()
        .all(|query| query.datasource_family == "search"));
    assert_eq!(
        filtered
            .queries
            .iter()
            .map(|query| query.datasource_type.as_str())
            .collect::<Vec<&str>>(),
        vec!["elasticsearch", "grafana-opensearch-datasource"]
    );
}

#[test]
fn build_export_inspection_query_report_emits_prometheus_and_loki_families_for_inventory_backed_rows(
) {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(raw_dir.join("General")).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid",
            "datasourcesFile": DATASOURCE_INVENTORY_FILENAME
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join(DATASOURCE_INVENTORY_FILENAME),
        serde_json::to_string_pretty(&json!([
            {
                "uid": "prom-main",
                "name": "Prometheus Main",
                "type": "prometheus",
                "access": "proxy",
                "url": "http://prometheus:9090",
                "isDefault": "false",
                "org": "Main Org.",
                "orgId": "1"
            },
            {
                "uid": "loki-main",
                "name": "Loki Main",
                "type": "loki",
                "access": "proxy",
                "url": "http://loki:3100",
                "isDefault": "false",
                "org": "Main Org.",
                "orgId": "1"
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("General").join("core.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "uid": "core-main",
                "title": "Core Main",
                "panels": [
                    {
                        "id": 7,
                        "title": "HTTP Requests",
                        "type": "timeseries",
                        "targets": [
                            {
                                "refId": "A",
                                "datasource": {"uid": "prom-main"},
                                "expr": "sum by(instance) (rate(http_requests_total{job=\"api\", instance=~\"web-.+\", __name__=\"http_requests_total\"}[5m])) / ignoring(pod) group_left(namespace) kube_pod_info{namespace=\"prod\", pod=~\"api-.+\"}"
                            }
                        ]
                    },
                    {
                        "id": 11,
                        "title": "Errors",
                        "type": "logs",
                        "targets": [
                            {
                                "refId": "B",
                                "datasource": {"uid": "loki-main"},
                                "expr": "sum by (level) (count_over_time({job=\"grafana\",level=~\"error|warn\"} |= \"timeout\" | json | level=\"error\" [5m]))"
                            }
                        ]
                    }
                ]
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let report = test_support::build_export_inspection_query_report(&raw_dir).unwrap();
    assert_eq!(report.summary.dashboard_count, 1);
    assert_eq!(report.summary.panel_count, 2);
    assert_eq!(report.summary.query_count, 2);
    assert_eq!(report.summary.report_row_count, 2);

    let row = |ref_id: &str| {
        report
            .queries
            .iter()
            .find(|query| query.ref_id == ref_id)
            .unwrap()
    };

    let prometheus = row("A");
    assert_eq!(prometheus.datasource, "prom-main");
    assert_eq!(prometheus.datasource_name, "Prometheus Main");
    assert_eq!(prometheus.datasource_uid, "prom-main");
    assert_eq!(prometheus.datasource_type, "prometheus");
    assert_eq!(prometheus.datasource_family, "prometheus");
    assert_eq!(
        prometheus.metrics,
        vec![
            "http_requests_total".to_string(),
            "kube_pod_info".to_string(),
        ]
    );
    assert_eq!(prometheus.functions, vec!["rate".to_string()]);
    assert_eq!(prometheus.measurements, Vec::<String>::new());
    assert_eq!(prometheus.buckets, vec!["5m".to_string()]);

    let loki = row("B");
    assert_eq!(loki.datasource, "loki-main");
    assert_eq!(loki.datasource_name, "Loki Main");
    assert_eq!(loki.datasource_uid, "loki-main");
    assert_eq!(loki.datasource_type, "loki");
    assert_eq!(loki.datasource_family, "loki");
    assert_eq!(loki.metrics, Vec::<String>::new());
    assert_eq!(
        loki.functions,
        vec![
            "sum".to_string(),
            "count_over_time".to_string(),
            "json".to_string(),
            "line_filter_contains".to_string(),
            "line_filter_contains:timeout".to_string(),
        ]
    );
    assert_eq!(
        loki.measurements,
        vec![
            "{job=\"grafana\",level=~\"error|warn\"}".to_string(),
            "job=\"grafana\"".to_string(),
            "level=~\"error|warn\"".to_string(),
            "level=\"error\"".to_string(),
        ]
    );
    assert_eq!(loki.buckets, vec!["5m".to_string()]);
}

#[test]
fn prepare_inspect_export_import_dir_merges_multi_org_root_for_inspection() {
    let export_root_temp = tempdir().unwrap();
    let export_root = export_root_temp.path().join("dashboard");
    let org_one_raw = export_root.join("org_1_Main_Org").join("raw");
    let org_two_raw = export_root.join("org_2_Ops_Org").join("raw");
    fs::create_dir_all(&org_one_raw).unwrap();
    fs::create_dir_all(&org_two_raw).unwrap();
    fs::write(
        export_root.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "root",
            "dashboardCount": 2,
            "indexFile": "index.json",
            "orgCount": 2,
            "orgs": [
                {"org": "Main Org.", "orgId": "1", "dashboardCount": 1},
                {"org": "Ops Org", "orgId": "2", "dashboardCount": 1}
            ]
        }))
        .unwrap(),
    )
    .unwrap();

    for (raw_dir, org_id, org_name, uid) in [
        (&org_one_raw, "1", "Main Org.", "cpu-main"),
        (&org_two_raw, "2", "Ops Org", "logs-main"),
    ] {
        fs::write(
            raw_dir.join(EXPORT_METADATA_FILENAME),
            serde_json::to_string_pretty(&json!({
                "kind": "grafana-utils-dashboard-export-index",
                "schemaVersion": TOOL_SCHEMA_VERSION,
                "variant": "raw",
                "dashboardCount": 1,
                "indexFile": "index.json",
                "format": "grafana-web-import-preserve-uid",
                "foldersFile": FOLDER_INVENTORY_FILENAME,
                "datasourcesFile": DATASOURCE_INVENTORY_FILENAME
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join("index.json"),
            serde_json::to_string_pretty(&json!([
                {
                    "uid": uid,
                    "title": uid,
                    "path": format!("General/{uid}.json"),
                    "format": "grafana-web-import-preserve-uid",
                    "org": org_name,
                    "orgId": org_id
                }
            ]))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join(FOLDER_INVENTORY_FILENAME),
            serde_json::to_string_pretty(&json!([
                {"uid": "general", "title": "General", "path": "General", "org": org_name, "orgId": org_id}
            ]))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join(DATASOURCE_INVENTORY_FILENAME),
            serde_json::to_string_pretty(&json!([
                {
                    "uid": format!("ds-{org_id}"),
                    "name": format!("ds-{org_id}"),
                    "type": "prometheus",
                    "access": "proxy",
                    "url": "http://prometheus:9090",
                    "isDefault": "true",
                    "org": org_name,
                    "orgId": org_id
                }
            ]))
            .unwrap(),
        )
        .unwrap();
        let dashboard_dir = raw_dir.join("General");
        fs::create_dir_all(&dashboard_dir).unwrap();
        fs::write(
            dashboard_dir.join(format!("{uid}.json")),
            serde_json::to_string_pretty(&json!({
                "dashboard": {
                    "uid": uid,
                    "title": uid,
                    "panels": [
                        {
                            "id": 7,
                            "title": "CPU",
                            "type": "timeseries",
                            "datasource": {"uid": format!("ds-{org_id}"), "type": "prometheus"},
                            "targets": [{"refId": "A", "expr": "up"}]
                        }
                    ]
                }
            }))
            .unwrap(),
        )
        .unwrap();
    }

    let inspect_temp = tempdir().unwrap();
    let merged_raw_dir =
        test_support::prepare_inspect_export_import_dir(inspect_temp.path(), &export_root).unwrap();
    let merged_metadata: Value = serde_json::from_str(
        &fs::read_to_string(merged_raw_dir.join(EXPORT_METADATA_FILENAME)).unwrap(),
    )
    .unwrap();
    let merged_index: Value =
        serde_json::from_str(&fs::read_to_string(merged_raw_dir.join("index.json")).unwrap())
            .unwrap();
    let report = test_support::build_export_inspection_query_report(&merged_raw_dir).unwrap();

    assert_eq!(report.summary.dashboard_count, 2);
    assert_eq!(report.queries.len(), 2);
    assert_eq!(merged_metadata["variant"], Value::String("raw".to_string()));
    assert_eq!(merged_metadata["dashboardCount"], Value::Number(2.into()));
    assert_eq!(
        merged_metadata["foldersFile"],
        Value::String(FOLDER_INVENTORY_FILENAME.to_string())
    );
    assert_eq!(
        merged_metadata["datasourcesFile"],
        Value::String(DATASOURCE_INVENTORY_FILENAME.to_string())
    );
    assert!(merged_metadata.get("orgCount").is_none());
    assert!(merged_metadata.get("orgs").is_none());
    assert_eq!(
        fs::read_to_string(merged_raw_dir.join(".inspect-source-root"))
            .unwrap()
            .trim(),
        export_root.display().to_string()
    );
    let merged_index_items = merged_index.as_array().unwrap();
    assert_eq!(
        merged_index_items[0]["path"],
        Value::String("org_1_Main_Org/General/cpu-main.json".to_string())
    );
    assert_eq!(
        merged_index_items[1]["path"],
        Value::String("org_2_Ops_Org/General/logs-main.json".to_string())
    );
    assert_eq!(report.queries[0].org, "Main Org.");
    assert_eq!(report.queries[0].org_id, "1");
    assert_eq!(report.queries[0].folder_path, "General");
    assert_eq!(
        report.queries[0].file_path,
        export_root
            .join("org_1_Main_Org")
            .join("raw")
            .join("General")
            .join("cpu-main.json")
            .display()
            .to_string()
    );
    assert_eq!(report.queries[1].org, "Ops Org");
    assert_eq!(report.queries[1].org_id, "2");
    assert_eq!(report.queries[1].folder_path, "General");
}

#[test]
fn build_export_inspection_query_report_extracts_loki_query_details() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(raw_dir.join("Logs")).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("Logs").join("loki.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "uid": "logs-main",
                "title": "Logs Main",
                "panels": [
                    {
                        "id": 11,
                        "title": "Errors",
                        "type": "logs",
                        "targets": [
                            {
                                "refId": "A",
                                "datasource": {"uid": "loki-main", "type": "loki"},
                                "expr": "sum by (level) (count_over_time({job=\"grafana\",level=~\"error|warn\"} |= \"timeout\" |~ \"panic|fatal\" | json | line_format \"{{.msg}}\" [5m]))"
                            }
                        ]
                    }
                ]
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let report = test_support::build_export_inspection_query_report(&raw_dir).unwrap();

    assert_eq!(report.queries.len(), 1);
    assert_eq!(report.queries[0].metrics, Vec::<String>::new());
    assert_eq!(
        report.queries[0].functions,
        vec![
            "sum".to_string(),
            "count_over_time".to_string(),
            "json".to_string(),
            "line_format".to_string(),
            "line_filter_contains".to_string(),
            "line_filter_contains:timeout".to_string(),
            "line_filter_regex".to_string(),
            "line_filter_regex:panic|fatal".to_string(),
        ]
    );
    assert_eq!(
        report.queries[0].measurements,
        vec![
            "{job=\"grafana\",level=~\"error|warn\"}".to_string(),
            "job=\"grafana\"".to_string(),
            "level=~\"error|warn\"".to_string(),
        ]
    );
    assert_eq!(report.queries[0].buckets, vec!["5m".to_string()]);
}

#[test]
fn build_export_inspection_query_report_ignores_loki_line_format_templates_when_extracting_filters()
{
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    write_basic_raw_export(
        &raw_dir,
        "1",
        "Main Org.",
        "loki-main",
        "Logs Main",
        "loki-main",
        "loki",
        "logs",
        "folder-1",
        "Logs",
        "expr",
        "sum by (namespace) (count_over_time({job=\"grafana\",namespace!~\"kube-system\"} | line_format \"{{.msg}} |= {{.status}} |~ {{.level}}\" |= \"timeout\" |~ \"panic|fatal\" [10m]))",
    );

    let report = test_support::build_export_inspection_query_report(&raw_dir).unwrap();

    assert_eq!(report.queries.len(), 1);
    assert_eq!(
        report.queries[0].functions,
        vec![
            "sum".to_string(),
            "count_over_time".to_string(),
            "line_format".to_string(),
            "line_filter_contains".to_string(),
            "line_filter_contains:timeout".to_string(),
            "line_filter_regex".to_string(),
            "line_filter_regex:panic|fatal".to_string(),
        ]
    );
    assert_eq!(
        report.queries[0].measurements,
        vec![
            "{job=\"grafana\",namespace!~\"kube-system\"}".to_string(),
            "job=\"grafana\"".to_string(),
            "namespace!~\"kube-system\"".to_string(),
        ]
    );
    assert_eq!(report.queries[0].buckets, vec!["10m".to_string()]);
}

#[test]
fn build_export_inspection_query_report_extracts_negative_loki_line_filters() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    write_basic_raw_export(
        &raw_dir,
        "1",
        "Main Org.",
        "loki-main",
        "Logs Main",
        "loki-main",
        "loki",
        "logs",
        "folder-1",
        "Logs",
        "expr",
        "sum by (namespace) (count_over_time({job=\"grafana\",namespace!=\"kube-system\"} != \"debug\" !~ \"health|metrics\" | json [15m]))",
    );

    let report = test_support::build_export_inspection_query_report(&raw_dir).unwrap();

    assert_eq!(report.queries.len(), 1);
    assert_eq!(
        report.queries[0].functions,
        vec![
            "sum".to_string(),
            "count_over_time".to_string(),
            "json".to_string(),
            "line_filter_not_contains".to_string(),
            "line_filter_not_contains:debug".to_string(),
            "line_filter_not_regex".to_string(),
            "line_filter_not_regex:health|metrics".to_string(),
        ]
    );
    assert_eq!(
        report.queries[0].measurements,
        vec![
            "{job=\"grafana\",namespace!=\"kube-system\"}".to_string(),
            "job=\"grafana\"".to_string(),
            "namespace!=\"kube-system\"".to_string(),
        ]
    );
    assert_eq!(report.queries[0].buckets, vec!["15m".to_string()]);
}

#[test]
fn build_export_inspection_query_report_ignores_loki_regex_character_classes_when_extracting_buckets(
) {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    write_basic_raw_export(
        &raw_dir,
        "1",
        "Main Org.",
        "loki-main",
        "Logs Main",
        "loki-main",
        "loki",
        "logs",
        "folder-1",
        "Logs",
        "expr",
        "sum(count_over_time({job=\"grafana\"} |~ \"panic[0-9]+\" [5m]))",
    );

    let report = test_support::build_export_inspection_query_report(&raw_dir).unwrap();

    assert_eq!(report.queries.len(), 1);
    assert_eq!(
        report.queries[0].functions,
        vec![
            "sum".to_string(),
            "count_over_time".to_string(),
            "line_filter_regex".to_string(),
            "line_filter_regex:panic[0-9]+".to_string(),
        ]
    );
    assert_eq!(
        report.queries[0].measurements,
        vec![
            "{job=\"grafana\"}".to_string(),
            "job=\"grafana\"".to_string(),
        ]
    );
    assert_eq!(report.queries[0].buckets, vec!["5m".to_string()]);
}

#[test]
fn build_export_inspection_query_report_keeps_prometheus_metrics_and_skips_label_tokens() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(raw_dir.join("General")).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("General").join("prometheus.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "uid": "prom-main",
                "title": "Prom Main",
                "panels": [
                    {
                        "id": 7,
                        "title": "HTTP Requests",
                        "type": "timeseries",
                        "datasource": {"uid": "prom-main", "type": "prometheus"},
                        "targets": [
                            {
                                "refId": "A",
                                "expr": "sum by(instance) (rate(http_requests_total{job=\"api\", instance=~\"web-.+\", __name__=\"http_requests_total\"}[5m])) / ignoring(pod) group_left(namespace) kube_pod_info{namespace=\"prod\", pod=~\"api-.+\"}"
                            }
                        ]
                    }
                ]
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let report = test_support::build_export_inspection_query_report(&raw_dir).unwrap();

    assert_eq!(report.queries.len(), 1);
    assert_eq!(
        report.queries[0].metrics,
        vec![
            "http_requests_total".to_string(),
            "kube_pod_info".to_string(),
        ]
    );
    assert_eq!(report.queries[0].functions, vec!["rate".to_string()]);
}

#[test]
fn resolve_query_analyzer_family_from_datasource_type_maps_supported_aliases_to_families() {
    let cases = [
        (
            "prometheus",
            Some(test_support::DATASOURCE_FAMILY_PROMETHEUS),
        ),
        (
            "grafana-prometheus-datasource",
            Some(test_support::DATASOURCE_FAMILY_PROMETHEUS),
        ),
        ("loki", Some(test_support::DATASOURCE_FAMILY_LOKI)),
        (
            "grafana-loki-datasource",
            Some(test_support::DATASOURCE_FAMILY_LOKI),
        ),
        ("tempo", Some(test_support::DATASOURCE_FAMILY_TRACING)),
        ("jaeger", Some(test_support::DATASOURCE_FAMILY_TRACING)),
        ("zipkin", Some(test_support::DATASOURCE_FAMILY_TRACING)),
        ("influxdb", Some(test_support::DATASOURCE_FAMILY_FLUX)),
        (
            "grafana-influxdb-datasource",
            Some(test_support::DATASOURCE_FAMILY_FLUX),
        ),
        ("flux", Some(test_support::DATASOURCE_FAMILY_FLUX)),
        ("mysql", Some(test_support::DATASOURCE_FAMILY_SQL)),
        (
            "grafana-mysql-datasource",
            Some(test_support::DATASOURCE_FAMILY_SQL),
        ),
        ("postgres", Some(test_support::DATASOURCE_FAMILY_SQL)),
        ("postgresql", Some(test_support::DATASOURCE_FAMILY_SQL)),
        (
            "grafana-postgresql-datasource",
            Some(test_support::DATASOURCE_FAMILY_SQL),
        ),
        ("mssql", Some(test_support::DATASOURCE_FAMILY_SQL)),
        (
            "elasticsearch",
            Some(test_support::DATASOURCE_FAMILY_SEARCH),
        ),
        (
            "grafana-opensearch-datasource",
            Some(test_support::DATASOURCE_FAMILY_SEARCH),
        ),
        ("opensearch", Some(test_support::DATASOURCE_FAMILY_SEARCH)),
        ("sqlite", None),
        ("custom", None),
    ];

    for (datasource_type, expected) in cases {
        assert_eq!(
            test_support::resolve_query_analyzer_family_from_datasource_type(datasource_type),
            expected,
            "unexpected family for datasource type {datasource_type}"
        );
    }
}

#[test]
fn resolve_query_analyzer_family_from_query_signature_maps_fallback_and_search_shapes() {
    let cases = [
        (
            "rawSql",
            "SELECT * FROM cpu",
            Some(test_support::DATASOURCE_FAMILY_SQL),
        ),
        (
            "sql",
            "SELECT * FROM cpu",
            Some(test_support::DATASOURCE_FAMILY_SQL),
        ),
        (
            "logql",
            "{job=\"grafana\"}",
            Some(test_support::DATASOURCE_FAMILY_LOKI),
        ),
        (
            "expr",
            "up",
            Some(test_support::DATASOURCE_FAMILY_PROMETHEUS),
        ),
        (
            "query",
            "from(bucket: \"prod\") |> range(start: -1h)",
            Some(test_support::DATASOURCE_FAMILY_FLUX),
        ),
        (
            "query",
            "SELECT mean(value) FROM cpu",
            Some(test_support::DATASOURCE_FAMILY_SQL),
        ),
        (
            "query",
            "update cpu set value = 1",
            Some(test_support::DATASOURCE_FAMILY_SQL),
        ),
        (
            "query",
            "_exists_:host.name AND host.name:api AND response.status:404",
            Some(test_support::DATASOURCE_FAMILY_SEARCH),
        ),
        ("query", "service.name:checkout AND trace.id=abc123", None),
        ("query", "up", None),
    ];

    for (query_field, query_text, expected) in cases {
        assert_eq!(
            test_support::resolve_query_analyzer_family_from_query_signature(
                query_field,
                query_text
            ),
            expected,
            "unexpected family for query_field={query_field} query_text={query_text}"
        );
    }
}

#[test]
fn dispatch_query_analysis_extracts_flux_every_window_hints() {
    let panel = Map::new();
    let target = Map::new();
    let context = test_support::QueryExtractionContext {
        panel: &panel,
        target: &target,
        query_field: "query",
        query_text:
            "from(bucket: \"prod\") |> range(start: -1h) |> aggregateWindow(every: 5m, fn: mean)",
        resolved_datasource_type: "flux",
    };

    assert_eq!(
        test_support::resolve_query_analyzer_family(&context),
        test_support::DATASOURCE_FAMILY_FLUX
    );
    assert_eq!(
        test_support::dispatch_query_analysis(&context).buckets,
        vec!["prod".to_string(), "5m".to_string()]
    );
}

#[test]
fn dispatch_query_analysis_ignores_flux_every_outside_window_calls() {
    let panel = Map::new();
    let target = Map::new();
    let context = test_support::QueryExtractionContext {
        panel: &panel,
        target: &target,
        query_field: "query",
        query_text:
            "option task = {name: \"cpu\", every: 1h}\nfrom(bucket: \"prod\") |> range(start: -1h)",
        resolved_datasource_type: "flux",
    };

    assert_eq!(
        test_support::dispatch_query_analysis(&context).buckets,
        vec!["prod".to_string()]
    );
}

#[test]
fn resolve_query_analyzer_family_prefers_target_datasource_then_panel_datasource() {
    let resolve = |panel_type: Option<&str>,
                   target_type: Option<&str>,
                   query_field: &'static str,
                   query_text: &'static str| {
        let panel_value = match panel_type {
            Some(value) => json!({"datasource": {"type": value}}),
            None => json!({}),
        };
        let target_value = match target_type {
            Some(value) => json!({"datasource": {"type": value}}),
            None => json!({}),
        };
        let panel = panel_value.as_object().unwrap();
        let target = target_value.as_object().unwrap();
        test_support::resolve_query_analyzer_family(&test_support::QueryExtractionContext {
            panel,
            target,
            query_field,
            query_text,
            resolved_datasource_type: "",
        })
    };

    let cases = [
        (
            Some("loki"),
            Some("prometheus"),
            "rawSql",
            "SELECT 1",
            test_support::DATASOURCE_FAMILY_PROMETHEUS,
        ),
        (
            Some("mssql"),
            Some("custom"),
            "expr",
            "up",
            test_support::DATASOURCE_FAMILY_SQL,
        ),
        (
            Some("loki"),
            None,
            "expr",
            "up",
            test_support::DATASOURCE_FAMILY_LOKI,
        ),
        (
            None,
            Some("grafana-postgresql-datasource"),
            "query",
            "up",
            test_support::DATASOURCE_FAMILY_SQL,
        ),
        (
            None,
            Some("flux"),
            "query",
            "SELECT * FROM cpu",
            test_support::DATASOURCE_FAMILY_FLUX,
        ),
        (
            Some("zipkin"),
            None,
            "query",
            "service.name:checkout",
            test_support::DATASOURCE_FAMILY_TRACING,
        ),
    ];

    for (panel_type, target_type, query_field, query_text, expected) in cases {
        assert_eq!(
            resolve(panel_type, target_type, query_field, query_text),
            expected,
            "unexpected family for panel={panel_type:?} target={target_type:?} query_field={query_field} query_text={query_text}"
        );
    }
}

#[test]
fn resolve_query_analyzer_family_prefers_inventory_resolved_datasource_type() {
    let panel = Map::new();
    let target = Map::from_iter(vec![
        (
            "datasource".to_string(),
            Value::Object(Map::from_iter(vec![(
                "uid".to_string(),
                Value::String("prom-main".to_string()),
            )])),
        ),
        ("query".to_string(), Value::String("up".to_string())),
    ]);
    let context = test_support::QueryExtractionContext {
        panel: &panel,
        target: &target,
        query_field: "query",
        query_text: "up",
        resolved_datasource_type: "prometheus",
    };

    assert_eq!(
        test_support::resolve_query_analyzer_family(&context),
        test_support::DATASOURCE_FAMILY_PROMETHEUS
    );
    assert_eq!(
        test_support::dispatch_query_analysis(&context).metrics,
        vec!["up".to_string()]
    );
}

#[test]
fn dispatch_query_analysis_extracts_obvious_tracing_field_hints() {
    let panel_value = json!({
        "datasource": {
            "type": "tempo"
        }
    });
    let target_value = json!({});
    let panel = panel_value.as_object().unwrap();
    let target = target_value.as_object().unwrap();
    let context = test_support::QueryExtractionContext {
        panel,
        target,
        query_field: "query",
        query_text: "service.name:checkout AND traceID=abc123 AND span.name:\"GET /orders\"",
        resolved_datasource_type: "tempo",
    };

    assert_eq!(
        test_support::resolve_query_analyzer_family(&context),
        test_support::DATASOURCE_FAMILY_TRACING
    );
    assert_eq!(
        test_support::dispatch_query_analysis(&context),
        QueryAnalysis {
            metrics: Vec::new(),
            functions: Vec::new(),
            measurements: vec![
                "service.name".to_string(),
                "traceID".to_string(),
                "span.name".to_string(),
            ],
            buckets: Vec::new(),
        }
    );
}

#[test]
fn dispatch_query_analysis_keeps_tracing_family_conservative_for_plain_text() {
    let panel_value = json!({
        "datasource": {
            "type": "zipkin"
        }
    });
    let target_value = json!({});
    let panel = panel_value.as_object().unwrap();
    let target = target_value.as_object().unwrap();
    let context = test_support::QueryExtractionContext {
        panel,
        target,
        query_field: "query",
        query_text: "trace workflow text with no obvious fields",
        resolved_datasource_type: "zipkin",
    };

    assert_eq!(
        test_support::resolve_query_analyzer_family(&context),
        test_support::DATASOURCE_FAMILY_TRACING
    );
    assert_eq!(
        test_support::dispatch_query_analysis(&context),
        QueryAnalysis::default()
    );
}

#[test]
fn resolve_query_analyzer_family_routes_elasticsearch_and_opensearch_to_search_family() {
    let cases = [
        ("elasticsearch", Some("prometheus"), Some("loki")),
        ("opensearch", Some("prometheus"), Some("loki")),
    ];

    for (resolved_datasource_type, panel_type, target_type) in cases {
        let panel_value = match panel_type {
            Some(value) => json!({"datasource": {"type": value}}),
            None => json!({}),
        };
        let target_value = match target_type {
            Some(value) => json!({"datasource": {"type": value}}),
            None => json!({}),
        };
        let panel = panel_value.as_object().unwrap();
        let target = target_value.as_object().unwrap();
        let context = test_support::QueryExtractionContext {
            panel,
            target,
            query_field: "query",
            query_text: "status:500",
            resolved_datasource_type,
        };

        assert_eq!(
            test_support::resolve_query_analyzer_family(&context),
            test_support::DATASOURCE_FAMILY_SEARCH,
            "unexpected family for resolved_datasource_type={resolved_datasource_type}"
        );
        assert_eq!(
            test_support::dispatch_query_analysis(&context),
            QueryAnalysis {
                metrics: Vec::new(),
                functions: Vec::new(),
                measurements: vec!["status".to_string()],
                buckets: Vec::new(),
            }
        );
    }
}

#[test]
fn dispatch_query_analysis_for_search_family_stays_conservative() {
    let cases = [
        (
            "elasticsearch",
            "status:500 AND status:500 AND _exists_:trace.id AND service.name:count AND category:rate",
            vec![
                "trace.id".to_string(),
                "status".to_string(),
                "service.name".to_string(),
                "category".to_string(),
            ],
        ),
        (
            "opensearch",
            "_exists_:host.name AND host.name:api AND response.status:404 AND response.status:404 AND level:error",
            vec![
                "host.name".to_string(),
                "response.status".to_string(),
                "level".to_string(),
            ],
        ),
    ];

    for (resolved_datasource_type, query_text, expected_measurements) in cases {
        let panel = Map::new();
        let target = Map::new();
        let context = test_support::QueryExtractionContext {
            panel: &panel,
            target: &target,
            query_field: "query",
            query_text,
            resolved_datasource_type,
        };

        assert_eq!(
            test_support::resolve_query_analyzer_family(&context),
            test_support::DATASOURCE_FAMILY_SEARCH,
            "unexpected family for resolved_datasource_type={resolved_datasource_type}"
        );
        assert_eq!(
            test_support::dispatch_query_analysis(&context),
            QueryAnalysis {
                metrics: Vec::new(),
                functions: Vec::new(),
                measurements: expected_measurements,
                buckets: Vec::new(),
            },
            "unexpected analysis for resolved_datasource_type={resolved_datasource_type}"
        );
    }
}

#[test]
fn dispatch_query_analysis_extracts_search_field_references_from_lucene_queries() {
    let panel_value = json!({
        "datasource": {
            "type": "elasticsearch"
        }
    });
    let target_value = json!({});
    let panel = panel_value.as_object().unwrap();
    let target = target_value.as_object().unwrap();

    let cases = [
        ("status:500", vec!["status"]),
        ("service.name:\"api\"", vec!["service.name"]),
        ("_exists_:traceId", vec!["traceId"]),
        (
            "@timestamp:[now-15m TO now] AND level:error",
            vec!["@timestamp", "level"],
        ),
        (
            "_exists_:@timestamp AND service.name:\"api\"",
            vec!["@timestamp", "service.name"],
        ),
        (
            "status:500 AND service.name:\"api\"",
            vec!["status", "service.name"],
        ),
    ];

    for (query_text, expected_measurements) in cases {
        let context = test_support::QueryExtractionContext {
            panel,
            target,
            query_field: "query",
            query_text,
            resolved_datasource_type: "elasticsearch",
        };

        assert_eq!(
            test_support::resolve_query_analyzer_family(&context),
            test_support::DATASOURCE_FAMILY_SEARCH,
            "unexpected family for query_text={query_text}"
        );
        assert_eq!(
            test_support::dispatch_query_analysis(&context),
            QueryAnalysis {
                metrics: Vec::new(),
                functions: Vec::new(),
                measurements: expected_measurements
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<String>>(),
                buckets: Vec::new(),
            },
            "unexpected analysis for query_text={query_text}"
        );
    }
}

#[test]
fn dispatch_query_analysis_keeps_search_family_conservative_for_non_lucene_shapes() {
    let panel_value = json!({
        "datasource": {
            "type": "elasticsearch"
        }
    });
    let target_value = json!({});
    let panel = panel_value.as_object().unwrap();
    let target = target_value.as_object().unwrap();

    let cases = [
        "{\"query\":{\"match\":{\"status\":\"500\"}}}",
        "source=logs | where status=500",
    ];

    for query_text in cases {
        let context = test_support::QueryExtractionContext {
            panel,
            target,
            query_field: "query",
            query_text,
            resolved_datasource_type: "elasticsearch",
        };

        assert_eq!(
            test_support::dispatch_query_analysis(&context),
            QueryAnalysis::default(),
            "unexpected analysis for query_text={query_text}"
        );
    }
}

#[test]
fn normalize_family_name_covers_core_family_aliases() {
    let cases = [
        ("prometheus", "prometheus"),
        ("grafana-prometheus-datasource", "prometheus"),
        ("loki", "loki"),
        ("grafana-loki-datasource", "loki"),
        ("influxdb", "flux"),
        ("grafana-influxdb-datasource", "flux"),
        ("flux", "flux"),
        ("mysql", "sql"),
        ("grafana-mysql-datasource", "sql"),
        ("postgres", "sql"),
        ("grafana-postgresql-datasource", "sql"),
        ("mssql", "sql"),
        ("elasticsearch", "search"),
        ("opensearch", "search"),
        ("grafana-opensearch-datasource", "search"),
        ("tempo", "tracing"),
        ("grafana-tempo-datasource", "tracing"),
        ("jaeger", "tracing"),
        ("zipkin", "tracing"),
        ("custom", "custom"),
    ];

    for (datasource_type, expected) in cases {
        assert_eq!(
            test_support::normalize_family_name(datasource_type),
            expected,
            "unexpected normalized family for datasource_type={datasource_type}"
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn make_core_family_report_row(
    dashboard_uid: &str,
    panel_id: &str,
    ref_id: &str,
    datasource_uid: &str,
    datasource_name: &str,
    datasource_type: &str,
    datasource_family: &str,
    query_text: &str,
    measurements: &[&str],
) -> test_support::ExportInspectionQueryRow {
    test_support::ExportInspectionQueryRow {
        org: "Main Org.".to_string(),
        org_id: "1".to_string(),
        dashboard_uid: dashboard_uid.to_string(),
        dashboard_title: format!("{dashboard_uid} Dashboard"),
        dashboard_tags: Vec::new(),
        folder_path: "General".to_string(),
        folder_full_path: "/".to_string(),
        folder_level: "1".to_string(),
        folder_uid: "general".to_string(),
        parent_folder_uid: String::new(),
        panel_id: panel_id.to_string(),
        panel_title: "Query".to_string(),
        panel_type: "table".to_string(),
        panel_target_count: 1,
        panel_query_count: 1,
        panel_datasource_count: 0,
        panel_variables: Vec::new(),
        ref_id: ref_id.to_string(),
        datasource: datasource_name.to_string(),
        datasource_name: datasource_name.to_string(),
        datasource_uid: datasource_uid.to_string(),
        datasource_org: String::new(),
        datasource_org_id: String::new(),
        datasource_database: String::new(),
        datasource_bucket: String::new(),
        datasource_organization: String::new(),
        datasource_index_pattern: String::new(),
        datasource_type: datasource_type.to_string(),
        datasource_family: datasource_family.to_string(),
        query_field: "query".to_string(),
        target_hidden: "false".to_string(),
        target_disabled: "false".to_string(),
        query_text: query_text.to_string(),
        query_variables: Vec::new(),
        metrics: Vec::new(),
        functions: Vec::new(),
        measurements: measurements.iter().map(|value| value.to_string()).collect(),
        buckets: Vec::new(),
        file_path: format!("/tmp/raw/{dashboard_uid}.json"),
    }
}

#[test]
fn apply_query_report_filters_matches_core_family_aliases() {
    let make_row = |dashboard_uid: &str,
                    panel_id: &str,
                    ref_id: &str,
                    datasource_uid: &str,
                    datasource_name: &str,
                    datasource_type: &str,
                    datasource_family: &str| {
        make_core_family_report_row(
            dashboard_uid,
            panel_id,
            ref_id,
            datasource_uid,
            datasource_name,
            datasource_type,
            datasource_family,
            "placeholder",
            &[],
        )
    };
    let report = test_support::ExportInspectionQueryReport {
        import_dir: "/tmp/raw".to_string(),
        summary: test_support::QueryReportSummary {
            dashboard_count: 6,
            panel_count: 6,
            query_count: 6,
            report_row_count: 6,
        },
        queries: vec![
            make_row(
                "prom-main",
                "1",
                "A",
                "prom-main",
                "Prometheus Main",
                "prometheus",
                "prometheus",
            ),
            make_row(
                "logs-main",
                "2",
                "A",
                "logs-main",
                "Logs Main",
                "loki",
                "loki",
            ),
            make_row(
                "flux-main",
                "3",
                "A",
                "flux-main",
                "Influx Main",
                "influxdb",
                "flux",
            ),
            make_row(
                "sql-main",
                "4",
                "A",
                "sql-main",
                "Postgres Main",
                "postgres",
                "postgres",
            ),
            make_row(
                "search-main",
                "5",
                "A",
                "search-main",
                "Elastic Main",
                "elasticsearch",
                "search",
            ),
            make_row(
                "trace-main",
                "6",
                "A",
                "trace-main",
                "Tempo Main",
                "tempo",
                "tracing",
            ),
        ],
    };
    let cases = [
        ("prometheus", "prom-main"),
        ("loki", "logs-main"),
        ("flux", "flux-main"),
        ("postgres", "sql-main"),
        ("search", "search-main"),
        ("tracing", "trace-main"),
    ];

    for (filter_value, expected_dashboard_uid) in cases {
        let filtered =
            test_support::apply_query_report_filters(report.clone(), Some(filter_value), None);
        assert_eq!(filtered.queries.len(), 1);
        assert_eq!(filtered.queries[0].dashboard_uid, expected_dashboard_uid);
    }

    let rendered = test_support::render_grouped_query_report(&report).join("\n");
    assert!(rendered.contains("datasourceFamily=search"));
    assert!(rendered.contains("datasourceFamily=tracing"));
}

#[test]
fn dispatch_query_analysis_matches_shared_analyzer_fixture_cases() {
    for case in load_inspection_analyzer_cases() {
        let case_name = case["name"].as_str().unwrap();
        let expected_family = case["expectedFamily"].as_str().unwrap();
        let expected_analysis = &case["expectedAnalysis"];
        let panel = case["panel"].as_object().unwrap().clone();
        let target = case["target"].as_object().unwrap().clone();
        let query_field = case["queryField"].as_str().unwrap();
        let query_text = case["queryText"].as_str().unwrap();
        let context = QueryExtractionContext {
            panel: &panel,
            target: &target,
            query_field,
            query_text,
            resolved_datasource_type: "",
        };

        assert_eq!(
            resolve_query_analyzer_family(&context),
            expected_family,
            "case={case_name}"
        );
        assert_eq!(
            dispatch_query_analysis(&context),
            QueryAnalysis {
                metrics: expected_analysis["metrics"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| value.as_str().unwrap().to_string())
                    .collect::<Vec<String>>(),
                functions: expected_analysis["functions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| value.as_str().unwrap().to_string())
                    .collect::<Vec<String>>(),
                measurements: expected_analysis["measurements"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| value.as_str().unwrap().to_string())
                    .collect::<Vec<String>>(),
                buckets: expected_analysis["buckets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| value.as_str().unwrap().to_string())
                    .collect::<Vec<String>>(),
            },
            "case={case_name}"
        );
    }
}

#[test]
fn resolve_report_column_ids_include_file_by_default_and_allow_datasource_uid() {
    let default_columns = test_support::resolve_report_column_ids(&[]).unwrap();
    assert!(default_columns.iter().any(|value| value == "file"));
    assert!(!default_columns
        .iter()
        .any(|value| value == "datasource_uid"));
    assert!(default_columns
        .iter()
        .any(|value| value == "datasource_type"));
    assert!(default_columns
        .iter()
        .any(|value| value == "datasource_family"));
    assert!(default_columns
        .iter()
        .any(|value| value == "dashboard_tags"));
    assert!(default_columns
        .iter()
        .any(|value| value == "panel_query_count"));
    assert!(default_columns
        .iter()
        .any(|value| value == "panel_datasource_count"));
    assert!(default_columns
        .iter()
        .any(|value| value == "panel_variables"));
    assert!(default_columns
        .iter()
        .any(|value| value == "query_variables"));

    let selected = test_support::resolve_report_column_ids(&[
        "dashboard_uid".to_string(),
        "datasource_uid".to_string(),
        "datasource_type".to_string(),
        "datasource_family".to_string(),
        "file".to_string(),
        "query".to_string(),
    ])
    .unwrap();
    assert_eq!(
        selected,
        vec![
            "dashboard_uid".to_string(),
            "datasource_uid".to_string(),
            "datasource_type".to_string(),
            "datasource_family".to_string(),
            "file".to_string(),
            "query".to_string(),
        ]
    );
}

#[test]
fn resolve_report_column_ids_for_format_defaults_csv_to_supported_columns() {
    let csv_columns = test_support::resolve_report_column_ids_for_format(
        Some(InspectExportReportFormat::Csv),
        &[],
    )
    .unwrap();
    assert!(csv_columns.iter().any(|value| value == "datasource_uid"));
    assert!(csv_columns
        .iter()
        .any(|value| value == "panel_target_count"));
    assert!(csv_columns.iter().any(|value| value == "target_hidden"));
    assert!(csv_columns.iter().any(|value| value == "target_disabled"));
    assert_eq!(
        csv_columns.len(),
        test_support::SUPPORTED_REPORT_COLUMN_IDS.len()
    );

    let table_columns = test_support::resolve_report_column_ids_for_format(
        Some(InspectExportReportFormat::Table),
        &[],
    )
    .unwrap();
    assert_eq!(
        table_columns,
        test_support::DEFAULT_REPORT_COLUMN_IDS
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<String>>()
    );
}

#[test]
fn resolve_report_column_ids_accepts_json_style_aliases() {
    let selected = test_support::resolve_report_column_ids(&[
        "dashboardUid".to_string(),
        "dashboardTags".to_string(),
        "datasourceUid".to_string(),
        "datasourceType".to_string(),
        "datasourceFamily".to_string(),
        "panelQueryCount".to_string(),
        "panelDatasourceCount".to_string(),
        "panelVariables".to_string(),
        "queryField".to_string(),
        "queryVariables".to_string(),
        "file".to_string(),
    ])
    .unwrap();
    assert_eq!(
        selected,
        vec![
            "dashboard_uid".to_string(),
            "dashboard_tags".to_string(),
            "datasource_uid".to_string(),
            "datasource_type".to_string(),
            "datasource_family".to_string(),
            "panel_query_count".to_string(),
            "panel_datasource_count".to_string(),
            "panel_variables".to_string(),
            "query_field".to_string(),
            "query_variables".to_string(),
            "file".to_string(),
        ]
    );
}

#[test]
fn export_inspection_query_row_json_keeps_datasource_uid_and_file_fields() {
    let row = test_support::ExportInspectionQueryRow {
        org: "Main Org.".to_string(),
        org_id: "1".to_string(),
        dashboard_uid: "main".to_string(),
        dashboard_title: "Main".to_string(),
        dashboard_tags: Vec::new(),
        folder_path: "General".to_string(),
        folder_full_path: "/".to_string(),
        folder_level: "1".to_string(),
        folder_uid: "general".to_string(),
        parent_folder_uid: String::new(),
        panel_id: "1".to_string(),
        panel_title: "CPU".to_string(),
        panel_type: "timeseries".to_string(),
        panel_target_count: 0,
        panel_query_count: 0,
        panel_datasource_count: 0,
        panel_variables: Vec::new(),
        ref_id: "A".to_string(),
        datasource: "prom-main".to_string(),
        datasource_name: "prom-main".to_string(),
        datasource_uid: String::new(),
        datasource_org: String::new(),
        datasource_org_id: String::new(),
        datasource_database: String::new(),
        datasource_bucket: String::new(),
        datasource_organization: String::new(),
        datasource_index_pattern: String::new(),
        datasource_type: "prometheus".to_string(),
        datasource_family: "prometheus".to_string(),
        query_field: "expr".to_string(),
        target_hidden: "false".to_string(),
        target_disabled: "false".to_string(),
        query_text: "up".to_string(),
        query_variables: Vec::new(),
        metrics: vec!["up".to_string()],
        functions: Vec::new(),
        measurements: Vec::new(),
        buckets: Vec::new(),
        file_path: "/tmp/raw/main.json".to_string(),
    };

    let value = serde_json::to_value(&row).unwrap();

    assert_eq!(value["org"], Value::String("Main Org.".to_string()));
    assert_eq!(value["orgId"], Value::String("1".to_string()));
    assert_eq!(value["folderFullPath"], Value::String("/".to_string()));
    assert_eq!(value["folderLevel"], Value::String("1".to_string()));
    assert_eq!(value["datasourceUid"], Value::String(String::new()));
    assert_eq!(
        value["datasourceType"],
        Value::String("prometheus".to_string())
    );
    assert_eq!(
        value["datasourceFamily"],
        Value::String("prometheus".to_string())
    );
    assert_eq!(
        value["file"],
        Value::String("/tmp/raw/main.json".to_string())
    );
}

#[test]
fn resolve_report_column_ids_rejects_unknown_columns() {
    let error = test_support::resolve_report_column_ids(&["unknown".to_string()]).unwrap_err();
    assert!(error
        .to_string()
        .contains("Unsupported --report-columns value"));
}

#[test]
fn resolve_report_column_ids_supports_all() {
    let columns = test_support::resolve_report_column_ids(&["all".to_string()]).unwrap();
    assert!(columns.contains(&"folder_full_path".to_string()));
    assert!(columns.contains(&"folder_level".to_string()));
    assert!(columns.contains(&"datasource_uid".to_string()));
    assert!(columns.contains(&"dashboard_tags".to_string()));
    assert!(columns.contains(&"panel_query_count".to_string()));
    assert!(columns.contains(&"panel_datasource_count".to_string()));
    assert!(columns.contains(&"panel_variables".to_string()));
    assert!(columns.contains(&"query_variables".to_string()));
    assert!(columns.contains(&"file".to_string()));
}

#[test]
fn report_format_supports_columns_matches_inspection_contract() {
    assert!(test_support::report_format_supports_columns(
        InspectExportReportFormat::Table
    ));
    assert!(test_support::report_format_supports_columns(
        InspectExportReportFormat::Csv
    ));
    assert!(test_support::report_format_supports_columns(
        InspectExportReportFormat::TreeTable
    ));
    assert!(!test_support::report_format_supports_columns(
        InspectExportReportFormat::Json
    ));
    assert!(!test_support::report_format_supports_columns(
        InspectExportReportFormat::Tree
    ));
}

#[test]
fn apply_query_report_filters_keep_matching_rows_only() {
    let report = test_support::ExportInspectionQueryReport {
        import_dir: "/tmp/raw".to_string(),
        summary: test_support::QueryReportSummary {
            dashboard_count: 2,
            panel_count: 2,
            query_count: 2,
            report_row_count: 2,
        },
        queries: vec![
            test_support::ExportInspectionQueryRow {
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
                dashboard_uid: "main".to_string(),
                dashboard_title: "Main".to_string(),
                dashboard_tags: Vec::new(),
                folder_path: "General".to_string(),
                folder_full_path: "/".to_string(),
                folder_level: "1".to_string(),
                folder_uid: "general".to_string(),
                parent_folder_uid: String::new(),
                panel_id: "1".to_string(),
                panel_title: "CPU".to_string(),
                panel_type: "timeseries".to_string(),
                panel_target_count: 1,
                panel_query_count: 1,
                panel_datasource_count: 0,
                panel_variables: Vec::new(),
                ref_id: "A".to_string(),
                datasource: "prom-main".to_string(),
                datasource_name: "prom-main".to_string(),
                datasource_uid: "prom-uid".to_string(),
                datasource_org: String::new(),
                datasource_org_id: String::new(),
                datasource_database: String::new(),
                datasource_bucket: String::new(),
                datasource_organization: String::new(),
                datasource_index_pattern: String::new(),
                datasource_type: "prometheus".to_string(),
                datasource_family: "prometheus".to_string(),
                query_field: "expr".to_string(),
                target_hidden: "false".to_string(),
                target_disabled: "false".to_string(),
                query_text: "up{job=\"grafana\"}".to_string(),
                query_variables: Vec::new(),
                metrics: vec!["up".to_string()],
                functions: Vec::new(),
                measurements: Vec::new(),
                buckets: Vec::new(),
                file_path: "/tmp/raw/main.json".to_string(),
            },
            test_support::ExportInspectionQueryRow {
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
                dashboard_uid: "logs".to_string(),
                dashboard_title: "Logs".to_string(),
                dashboard_tags: Vec::new(),
                folder_path: "General".to_string(),
                folder_full_path: "/".to_string(),
                folder_level: "1".to_string(),
                folder_uid: "general".to_string(),
                parent_folder_uid: String::new(),
                panel_id: "2".to_string(),
                panel_title: "Logs".to_string(),
                panel_type: "logs".to_string(),
                panel_target_count: 1,
                panel_query_count: 1,
                panel_datasource_count: 0,
                panel_variables: Vec::new(),
                ref_id: "A".to_string(),
                datasource: "logs-main".to_string(),
                datasource_name: "logs-main".to_string(),
                datasource_uid: "logs-uid".to_string(),
                datasource_org: String::new(),
                datasource_org_id: String::new(),
                datasource_database: String::new(),
                datasource_bucket: String::new(),
                datasource_organization: String::new(),
                datasource_index_pattern: String::new(),
                datasource_type: "loki".to_string(),
                datasource_family: "loki".to_string(),
                query_field: "expr".to_string(),
                target_hidden: "false".to_string(),
                target_disabled: "false".to_string(),
                query_text: "{job=\"grafana\"}".to_string(),
                query_variables: Vec::new(),
                metrics: Vec::new(),
                functions: Vec::new(),
                measurements: Vec::new(),
                buckets: Vec::new(),
                file_path: "/tmp/raw/logs.json".to_string(),
            },
        ],
    };

    let filtered = test_support::apply_query_report_filters(report, Some("prom-main"), Some("1"));

    assert_eq!(filtered.summary.dashboard_count, 1);
    assert_eq!(filtered.summary.panel_count, 1);
    assert_eq!(filtered.summary.query_count, 1);
    assert_eq!(filtered.summary.report_row_count, 1);
    assert_eq!(filtered.queries.len(), 1);
    assert_eq!(filtered.queries[0].datasource, "prom-main");
    assert_eq!(filtered.queries[0].panel_id, "1");
}

#[test]
fn apply_query_report_filters_match_datasource_uid_type_and_family() {
    let report = test_support::ExportInspectionQueryReport {
        import_dir: "/tmp/raw".to_string(),
        summary: test_support::QueryReportSummary {
            dashboard_count: 2,
            panel_count: 2,
            query_count: 2,
            report_row_count: 2,
        },
        queries: vec![
            test_support::ExportInspectionQueryRow {
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
                dashboard_uid: "main".to_string(),
                dashboard_title: "Main".to_string(),
                dashboard_tags: Vec::new(),
                folder_path: "General".to_string(),
                folder_full_path: "/".to_string(),
                folder_level: "1".to_string(),
                folder_uid: "general".to_string(),
                parent_folder_uid: String::new(),
                panel_id: "1".to_string(),
                panel_title: "CPU".to_string(),
                panel_type: "timeseries".to_string(),
                panel_target_count: 1,
                panel_query_count: 1,
                panel_datasource_count: 0,
                panel_variables: Vec::new(),
                ref_id: "A".to_string(),
                datasource: "prom-main".to_string(),
                datasource_name: "prom-main".to_string(),
                datasource_uid: "prom-uid".to_string(),
                datasource_org: String::new(),
                datasource_org_id: String::new(),
                datasource_database: String::new(),
                datasource_bucket: String::new(),
                datasource_organization: String::new(),
                datasource_index_pattern: String::new(),
                datasource_type: "prometheus".to_string(),
                datasource_family: "prometheus".to_string(),
                query_field: "expr".to_string(),
                target_hidden: "false".to_string(),
                target_disabled: "false".to_string(),
                query_text: "up".to_string(),
                query_variables: Vec::new(),
                metrics: vec!["up".to_string()],
                functions: Vec::new(),
                measurements: Vec::new(),
                buckets: Vec::new(),
                file_path: "/tmp/raw/main.json".to_string(),
            },
            test_support::ExportInspectionQueryRow {
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
                dashboard_uid: "logs".to_string(),
                dashboard_title: "Logs".to_string(),
                dashboard_tags: Vec::new(),
                folder_path: "General".to_string(),
                folder_full_path: "/".to_string(),
                folder_level: "1".to_string(),
                folder_uid: "general".to_string(),
                parent_folder_uid: String::new(),
                panel_id: "2".to_string(),
                panel_title: "Logs".to_string(),
                panel_type: "logs".to_string(),
                panel_target_count: 1,
                panel_query_count: 1,
                panel_datasource_count: 0,
                panel_variables: Vec::new(),
                ref_id: "A".to_string(),
                datasource: "logs-main".to_string(),
                datasource_name: "logs-main".to_string(),
                datasource_uid: "logs-uid".to_string(),
                datasource_org: String::new(),
                datasource_org_id: String::new(),
                datasource_database: String::new(),
                datasource_bucket: String::new(),
                datasource_organization: String::new(),
                datasource_index_pattern: String::new(),
                datasource_type: "loki".to_string(),
                datasource_family: "loki".to_string(),
                query_field: "expr".to_string(),
                target_hidden: "false".to_string(),
                target_disabled: "false".to_string(),
                query_text: "{job=\"grafana\"}".to_string(),
                query_variables: Vec::new(),
                metrics: Vec::new(),
                functions: Vec::new(),
                measurements: Vec::new(),
                buckets: Vec::new(),
                file_path: "/tmp/raw/logs.json".to_string(),
            },
        ],
    };

    let filtered_uid =
        test_support::apply_query_report_filters(report.clone(), Some("prom-uid"), None);
    assert_eq!(filtered_uid.queries.len(), 1);
    assert_eq!(filtered_uid.queries[0].dashboard_uid, "main");

    let filtered_type =
        test_support::apply_query_report_filters(report.clone(), Some("loki"), None);
    assert_eq!(filtered_type.queries.len(), 1);
    assert_eq!(filtered_type.queries[0].dashboard_uid, "logs");

    let filtered_family =
        test_support::apply_query_report_filters(report, Some("prometheus"), None);
    assert_eq!(filtered_family.queries.len(), 1);
    assert_eq!(filtered_family.queries[0].dashboard_uid, "main");
}

#[test]
fn apply_query_report_filters_matches_normalized_search_family() {
    let report = test_support::ExportInspectionQueryReport {
        import_dir: "/tmp/raw".to_string(),
        summary: test_support::QueryReportSummary {
            dashboard_count: 1,
            panel_count: 1,
            query_count: 1,
            report_row_count: 1,
        },
        queries: vec![test_support::ExportInspectionQueryRow {
            org: "Main Org.".to_string(),
            org_id: "1".to_string(),
            dashboard_uid: "search-main".to_string(),
            dashboard_title: "Search Main".to_string(),
            dashboard_tags: Vec::new(),
            folder_path: "General".to_string(),
            folder_full_path: "/".to_string(),
            folder_level: "1".to_string(),
            folder_uid: "general".to_string(),
            parent_folder_uid: String::new(),
            panel_id: "11".to_string(),
            panel_title: "Errors".to_string(),
            panel_type: "table".to_string(),
            panel_target_count: 1,
            panel_query_count: 1,
            panel_datasource_count: 0,
            panel_variables: Vec::new(),
            ref_id: "E".to_string(),
            datasource: "elastic-main".to_string(),
            datasource_name: "Elastic Main".to_string(),
            datasource_uid: "elastic-main".to_string(),
            datasource_org: "Main Org.".to_string(),
            datasource_org_id: "1".to_string(),
            datasource_database: String::new(),
            datasource_bucket: String::new(),
            datasource_organization: String::new(),
            datasource_index_pattern: "[logs-]YYYY.MM.DD".to_string(),
            datasource_type: "elasticsearch".to_string(),
            datasource_family: "search".to_string(),
            query_field: "query".to_string(),
            target_hidden: "false".to_string(),
            target_disabled: "false".to_string(),
            query_text: "status:500".to_string(),
            query_variables: Vec::new(),
            metrics: Vec::new(),
            functions: Vec::new(),
            measurements: vec!["status".to_string()],
            buckets: Vec::new(),
            file_path: "/tmp/raw/search.json".to_string(),
        }],
    };

    let filtered_family =
        test_support::apply_query_report_filters(report.clone(), Some("search"), None);
    assert_eq!(filtered_family.queries.len(), 1);
    assert_eq!(filtered_family.queries[0].dashboard_uid, "search-main");

    let filtered_type =
        test_support::apply_query_report_filters(report, Some("elasticsearch"), None);
    assert_eq!(filtered_type.queries.len(), 1);
    assert_eq!(filtered_type.queries[0].dashboard_uid, "search-main");
}

#[test]
fn normalize_query_report_groups_rows_by_dashboard_then_panel() {
    let report = test_support::ExportInspectionQueryReport {
        import_dir: "/tmp/raw".to_string(),
        summary: test_support::QueryReportSummary {
            dashboard_count: 2,
            panel_count: 3,
            query_count: 3,
            report_row_count: 3,
        },
        queries: vec![
            test_support::ExportInspectionQueryRow {
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
                dashboard_uid: "main".to_string(),
                dashboard_title: "Main".to_string(),
                dashboard_tags: Vec::new(),
                folder_path: "General".to_string(),
                folder_full_path: "/".to_string(),
                folder_level: "1".to_string(),
                folder_uid: "general".to_string(),
                parent_folder_uid: String::new(),
                panel_id: "1".to_string(),
                panel_title: "CPU".to_string(),
                panel_type: "timeseries".to_string(),
                panel_target_count: 1,
                panel_query_count: 1,
                panel_datasource_count: 0,
                panel_variables: Vec::new(),
                ref_id: "A".to_string(),
                datasource: "prom-main".to_string(),
                datasource_name: "prom-main".to_string(),
                datasource_uid: "prom-main".to_string(),
                datasource_org: String::new(),
                datasource_org_id: String::new(),
                datasource_database: String::new(),
                datasource_bucket: String::new(),
                datasource_organization: String::new(),
                datasource_index_pattern: String::new(),
                datasource_type: "prometheus".to_string(),
                datasource_family: "prometheus".to_string(),
                query_field: "expr".to_string(),
                target_hidden: "false".to_string(),
                target_disabled: "false".to_string(),
                query_text: "up".to_string(),
                query_variables: Vec::new(),
                metrics: vec!["up".to_string()],
                functions: Vec::new(),
                measurements: Vec::new(),
                buckets: Vec::new(),
                file_path: "/tmp/raw/main.json".to_string(),
            },
            test_support::ExportInspectionQueryRow {
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
                dashboard_uid: "main".to_string(),
                dashboard_title: "Main".to_string(),
                dashboard_tags: Vec::new(),
                folder_path: "General".to_string(),
                folder_full_path: "/".to_string(),
                folder_level: "1".to_string(),
                folder_uid: "general".to_string(),
                parent_folder_uid: String::new(),
                panel_id: "2".to_string(),
                panel_title: "Memory".to_string(),
                panel_type: "timeseries".to_string(),
                panel_target_count: 1,
                panel_query_count: 1,
                panel_datasource_count: 0,
                panel_variables: Vec::new(),
                ref_id: "B".to_string(),
                datasource: "prom-main".to_string(),
                datasource_name: "prom-main".to_string(),
                datasource_uid: "prom-main".to_string(),
                datasource_org: String::new(),
                datasource_org_id: String::new(),
                datasource_database: String::new(),
                datasource_bucket: String::new(),
                datasource_organization: String::new(),
                datasource_index_pattern: String::new(),
                datasource_type: "prometheus".to_string(),
                datasource_family: "prometheus".to_string(),
                query_field: "expr".to_string(),
                target_hidden: "false".to_string(),
                target_disabled: "false".to_string(),
                query_text: "process_resident_memory_bytes".to_string(),
                query_variables: Vec::new(),
                metrics: vec!["process_resident_memory_bytes".to_string()],
                functions: Vec::new(),
                measurements: Vec::new(),
                buckets: Vec::new(),
                file_path: "/tmp/raw/main.json".to_string(),
            },
            test_support::ExportInspectionQueryRow {
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
                dashboard_uid: "logs".to_string(),
                dashboard_title: "Logs".to_string(),
                dashboard_tags: Vec::new(),
                folder_path: "Platform / Logs".to_string(),
                folder_full_path: "/Platform/Logs".to_string(),
                folder_level: "2".to_string(),
                folder_uid: "logs".to_string(),
                parent_folder_uid: "platform".to_string(),
                panel_id: "7".to_string(),
                panel_title: "Errors".to_string(),
                panel_type: "logs".to_string(),
                panel_target_count: 1,
                panel_query_count: 1,
                panel_datasource_count: 0,
                panel_variables: Vec::new(),
                ref_id: "A".to_string(),
                datasource: "loki-main".to_string(),
                datasource_name: "loki-main".to_string(),
                datasource_uid: "loki-main".to_string(),
                datasource_org: String::new(),
                datasource_org_id: String::new(),
                datasource_database: String::new(),
                datasource_bucket: String::new(),
                datasource_organization: String::new(),
                datasource_index_pattern: String::new(),
                datasource_type: "loki".to_string(),
                datasource_family: "loki".to_string(),
                query_field: "expr".to_string(),
                target_hidden: "false".to_string(),
                target_disabled: "false".to_string(),
                query_text: "{job=\"grafana\"}".to_string(),
                query_variables: Vec::new(),
                metrics: Vec::new(),
                functions: Vec::new(),
                measurements: Vec::new(),
                buckets: Vec::new(),
                file_path: "/tmp/raw/logs.json".to_string(),
            },
        ],
    };

    let normalized = test_support::normalize_query_report(&report);

    assert_eq!(normalized.import_dir, "/tmp/raw");
    assert_eq!(normalized.summary, report.summary);
    assert_eq!(normalized.dashboards.len(), 2);
    assert_eq!(normalized.dashboards[0].org, "Main Org.");
    assert_eq!(normalized.dashboards[0].org_id, "1");
    assert_eq!(normalized.dashboards[0].dashboard_uid, "main");
    assert_eq!(normalized.dashboards[0].file_path, "/tmp/raw/main.json");
    assert_eq!(
        normalized.dashboards[0].datasources,
        vec!["prom-main".to_string()]
    );
    assert_eq!(
        normalized.dashboards[0].datasource_families,
        vec!["prometheus".to_string()]
    );
    assert_eq!(normalized.dashboards[0].panels.len(), 2);
    assert_eq!(normalized.dashboards[0].panels[0].panel_id, "1");
    assert_eq!(
        normalized.dashboards[0].panels[0].datasources,
        vec!["prom-main".to_string()]
    );
    assert_eq!(
        normalized.dashboards[0].panels[0].datasource_families,
        vec!["prometheus".to_string()]
    );
    assert_eq!(
        normalized.dashboards[0].panels[0].query_fields,
        vec!["expr".to_string()]
    );
    assert_eq!(normalized.dashboards[0].panels[0].queries.len(), 1);
    assert_eq!(normalized.dashboards[1].dashboard_uid, "logs");
    assert_eq!(normalized.dashboards[1].panels[0].panel_title, "Errors");
}

#[test]
fn validate_inspect_export_report_args_rejects_report_columns_without_report() {
    let args = InspectExportArgs {
        import_dir: PathBuf::from("./dashboards/raw"),
        json: false,
        table: false,
        report: None,
        output_format: None,
        report_columns: vec!["dashboard_uid".to_string()],
        report_filter_datasource: None,
        report_filter_panel_id: None,
        help_full: false,
        no_header: false,
        output_file: None,
    };

    let error = test_support::validate_inspect_export_report_args(&args).unwrap_err();
    assert!(error.to_string().contains(
        "--report-columns is only supported together with --report or report-like --output-format"
    ));
}

#[test]
fn validate_inspect_export_report_args_rejects_report_columns_for_json_report() {
    let args = InspectExportArgs {
        import_dir: PathBuf::from("./dashboards/raw"),
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::Json),
        output_format: None,
        report_columns: vec!["dashboard_uid".to_string()],
        report_filter_datasource: None,
        report_filter_panel_id: None,
        help_full: false,
        no_header: false,
        output_file: None,
    };

    let error = test_support::validate_inspect_export_report_args(&args).unwrap_err();
    assert!(error.to_string().contains(
        "--report-columns is only supported with report-table, report-csv, report-tree-table, or the equivalent --report modes"
    ));
}

#[test]
fn validate_inspect_export_report_args_rejects_report_columns_for_dependency_report() {
    let args = InspectExportArgs {
        import_dir: PathBuf::from("./dashboards/raw"),
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::Dependency),
        output_format: None,
        report_columns: vec!["dashboard_uid".to_string()],
        report_filter_datasource: None,
        report_filter_panel_id: None,
        help_full: false,
        no_header: false,
        output_file: None,
    };

    let error = test_support::validate_inspect_export_report_args(&args).unwrap_err();
    assert!(error.to_string().contains(
        "--report-columns is only supported with report-table, report-csv, report-tree-table, or the equivalent --report modes"
    ));
}

#[test]
fn validate_inspect_export_report_args_rejects_report_columns_for_tree_report() {
    let args = InspectExportArgs {
        import_dir: PathBuf::from("./dashboards/raw"),
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::Tree),
        output_format: None,
        report_columns: vec!["dashboard_uid".to_string()],
        report_filter_datasource: None,
        report_filter_panel_id: None,
        help_full: false,
        no_header: false,
        output_file: None,
    };

    let error = test_support::validate_inspect_export_report_args(&args).unwrap_err();
    assert!(error.to_string().contains(
        "--report-columns is only supported with report-table, report-csv, report-tree-table, or the equivalent --report modes"
    ));
}

#[test]
fn validate_inspect_export_report_args_rejects_report_columns_for_governance_report() {
    let args = InspectExportArgs {
        import_dir: PathBuf::from("./dashboards/raw"),
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::Governance),
        output_format: None,
        report_columns: vec!["dashboard_uid".to_string()],
        report_filter_datasource: None,
        report_filter_panel_id: None,
        help_full: false,
        no_header: false,
        output_file: None,
    };

    let error = test_support::validate_inspect_export_report_args(&args).unwrap_err();
    assert!(error
        .to_string()
        .contains("--report-columns is not supported with governance output"));
}

#[test]
fn validate_inspect_export_report_args_allows_report_columns_for_tree_table_report() {
    let args = InspectExportArgs {
        import_dir: PathBuf::from("./dashboards/raw"),
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::TreeTable),
        output_format: None,
        report_columns: vec!["panel_id".to_string(), "query".to_string()],
        report_filter_datasource: None,
        report_filter_panel_id: None,
        help_full: false,
        no_header: false,
        output_file: None,
    };

    test_support::validate_inspect_export_report_args(&args).unwrap();
}

#[test]
fn render_csv_uses_headers_and_escaping() {
    let lines = test_support::render_csv(
        &["DASHBOARD_UID", "QUERY"],
        &[vec![
            "mixed-main".to_string(),
            "{job=\"grafana\"},error".to_string(),
        ]],
    );

    assert_eq!(lines[0], "DASHBOARD_UID,QUERY");
    assert_eq!(lines[1], "mixed-main,\"{job=\"\"grafana\"\"},error\"");
}

#[test]
fn render_grouped_query_report_displays_dashboard_panel_and_query_tree() {
    let report = test_support::ExportInspectionQueryReport {
        import_dir: "/tmp/raw".to_string(),
        summary: test_support::QueryReportSummary {
            dashboard_count: 1,
            panel_count: 2,
            query_count: 2,
            report_row_count: 2,
        },
        queries: vec![
            test_support::ExportInspectionQueryRow {
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
                dashboard_uid: "main".to_string(),
                dashboard_title: "Main".to_string(),
                dashboard_tags: Vec::new(),
                folder_path: "General".to_string(),
                folder_full_path: "/".to_string(),
                folder_level: "1".to_string(),
                folder_uid: "general".to_string(),
                parent_folder_uid: String::new(),
                panel_id: "7".to_string(),
                panel_title: "CPU".to_string(),
                panel_type: "timeseries".to_string(),
                panel_target_count: 1,
                panel_query_count: 1,
                panel_datasource_count: 0,
                panel_variables: Vec::new(),
                ref_id: "A".to_string(),
                datasource: "prom-main".to_string(),
                datasource_name: "prom-main".to_string(),
                datasource_uid: "prom-main".to_string(),
                datasource_org: String::new(),
                datasource_org_id: String::new(),
                datasource_database: String::new(),
                datasource_bucket: String::new(),
                datasource_organization: String::new(),
                datasource_index_pattern: String::new(),
                datasource_type: "prometheus".to_string(),
                datasource_family: "prometheus".to_string(),
                query_field: "expr".to_string(),
                target_hidden: "false".to_string(),
                target_disabled: "false".to_string(),
                query_text: "up".to_string(),
                query_variables: Vec::new(),
                metrics: vec!["up".to_string()],
                functions: Vec::new(),
                measurements: Vec::new(),
                buckets: Vec::new(),
                file_path: "/tmp/raw/main.json".to_string(),
            },
            test_support::ExportInspectionQueryRow {
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
                dashboard_uid: "main".to_string(),
                dashboard_title: "Main".to_string(),
                dashboard_tags: Vec::new(),
                folder_path: "General".to_string(),
                folder_full_path: "/".to_string(),
                folder_level: "1".to_string(),
                folder_uid: "general".to_string(),
                parent_folder_uid: String::new(),
                panel_id: "8".to_string(),
                panel_title: "Logs".to_string(),
                panel_type: "logs".to_string(),
                panel_target_count: 1,
                panel_query_count: 1,
                panel_datasource_count: 0,
                panel_variables: Vec::new(),
                ref_id: "B".to_string(),
                datasource: "loki-main".to_string(),
                datasource_name: "loki-main".to_string(),
                datasource_uid: "loki-main".to_string(),
                datasource_org: String::new(),
                datasource_org_id: String::new(),
                datasource_database: String::new(),
                datasource_bucket: String::new(),
                datasource_organization: String::new(),
                datasource_index_pattern: String::new(),
                datasource_type: "loki".to_string(),
                datasource_family: "loki".to_string(),
                query_field: "expr".to_string(),
                target_hidden: "false".to_string(),
                target_disabled: "false".to_string(),
                query_text: "{job=\"grafana\"}".to_string(),
                query_variables: Vec::new(),
                metrics: Vec::new(),
                functions: Vec::new(),
                measurements: Vec::new(),
                buckets: Vec::new(),
                file_path: "/tmp/raw/main.json".to_string(),
            },
        ],
    };

    let lines = test_support::render_grouped_query_report(&report);
    let output = lines.join("\n");

    assert!(output.contains("Export inspection report: /tmp/raw"));
    assert!(output.contains("# Dashboard tree"));
    assert!(output.contains("[1] Dashboard: Main (uid=main, folder=General"));
    assert!(output.contains("datasources=prom-main,loki-main"));
    assert!(output.contains("families=prometheus,loki"));
    assert!(output.contains("folderUid=general"));
    assert!(output.contains("org=Main Org., orgId=1"));
    assert!(output.contains("  File: /tmp/raw/main.json"));
    assert!(output.contains("  Panel: CPU (id=7, type=timeseries, targets=1, queries=1, datasources=prom-main, families=prometheus, fields=expr)"));
    assert!(output.contains("  Panel: Logs (id=8, type=logs, targets=1, queries=1, datasources=loki-main, families=loki, fields=expr)"));
    assert!(output.contains(
        "    Query: refId=A datasource=prom-main datasourceName=prom-main datasourceUid=prom-main datasourceType=prometheus datasourceFamily=prometheus field=expr metrics=up"
    ));
    assert!(output.contains("      up"));
    assert!(output.contains(
        "    Query: refId=B datasource=loki-main datasourceName=loki-main datasourceUid=loki-main datasourceType=loki datasourceFamily=loki field=expr"
    ));
    assert!(output.contains("      {job=\"grafana\"}"));
}

#[test]
fn render_grouped_query_table_report_displays_dashboard_sections_with_tables() {
    let report = test_support::ExportInspectionQueryReport {
        import_dir: "/tmp/raw".to_string(),
        summary: test_support::QueryReportSummary {
            dashboard_count: 1,
            panel_count: 2,
            query_count: 2,
            report_row_count: 2,
        },
        queries: vec![
            test_support::ExportInspectionQueryRow {
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
                dashboard_uid: "main".to_string(),
                dashboard_title: "Main".to_string(),
                dashboard_tags: Vec::new(),
                folder_path: "General".to_string(),
                folder_full_path: "/".to_string(),
                folder_level: "1".to_string(),
                folder_uid: "general".to_string(),
                parent_folder_uid: String::new(),
                panel_id: "7".to_string(),
                panel_title: "CPU".to_string(),
                panel_type: "timeseries".to_string(),
                panel_target_count: 1,
                panel_query_count: 1,
                panel_datasource_count: 0,
                panel_variables: Vec::new(),
                ref_id: "A".to_string(),
                datasource: "prom-main".to_string(),
                datasource_name: "prom-main".to_string(),
                datasource_uid: "prom-main".to_string(),
                datasource_org: String::new(),
                datasource_org_id: String::new(),
                datasource_database: String::new(),
                datasource_bucket: String::new(),
                datasource_organization: String::new(),
                datasource_index_pattern: String::new(),
                datasource_type: "prometheus".to_string(),
                datasource_family: "prometheus".to_string(),
                query_field: "expr".to_string(),
                target_hidden: "false".to_string(),
                target_disabled: "false".to_string(),
                query_text: "up".to_string(),
                query_variables: Vec::new(),
                metrics: vec!["up".to_string()],
                functions: Vec::new(),
                measurements: Vec::new(),
                buckets: Vec::new(),
                file_path: "/tmp/raw/main.json".to_string(),
            },
            test_support::ExportInspectionQueryRow {
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
                dashboard_uid: "main".to_string(),
                dashboard_title: "Main".to_string(),
                dashboard_tags: Vec::new(),
                folder_path: "General".to_string(),
                folder_full_path: "/".to_string(),
                folder_level: "1".to_string(),
                folder_uid: "general".to_string(),
                parent_folder_uid: String::new(),
                panel_id: "8".to_string(),
                panel_title: "Logs".to_string(),
                panel_type: "logs".to_string(),
                panel_target_count: 1,
                panel_query_count: 1,
                panel_datasource_count: 0,
                panel_variables: Vec::new(),
                ref_id: "B".to_string(),
                datasource: "loki-main".to_string(),
                datasource_name: "loki-main".to_string(),
                datasource_uid: "loki-main".to_string(),
                datasource_org: String::new(),
                datasource_org_id: String::new(),
                datasource_database: String::new(),
                datasource_bucket: String::new(),
                datasource_organization: String::new(),
                datasource_index_pattern: String::new(),
                datasource_type: "loki".to_string(),
                datasource_family: "loki".to_string(),
                query_field: "expr".to_string(),
                target_hidden: "false".to_string(),
                target_disabled: "false".to_string(),
                query_text: "{job=\"grafana\"}".to_string(),
                query_variables: Vec::new(),
                metrics: Vec::new(),
                functions: Vec::new(),
                measurements: Vec::new(),
                buckets: Vec::new(),
                file_path: "/tmp/raw/main.json".to_string(),
            },
        ],
    };

    let lines = test_support::render_grouped_query_table_report(
        &report,
        &[
            "panel_id".to_string(),
            "panel_title".to_string(),
            "datasource".to_string(),
            "query".to_string(),
        ],
        true,
    );
    let output = lines.join("\n");

    assert!(output.contains("# Dashboard sections"));
    assert!(output.contains("[1] Dashboard: Main (uid=main, folder=General"));
    assert!(output.contains("datasources=prom-main,loki-main"));
    assert!(output.contains("families=prometheus,loki"));
    assert!(output.contains("folderUid=general"));
    assert!(output.contains("org=Main Org., orgId=1"));
    assert!(output.contains("File: /tmp/raw/main.json"));
    assert!(output.contains("Panel: CPU (id=7, type=timeseries, targets=1, queries=1, datasources=prom-main, families=prometheus, fields=expr)"));
    assert!(output.contains("Panel: Logs (id=8, type=logs, targets=1, queries=1, datasources=loki-main, families=loki, fields=expr)"));
    assert!(output.contains("PANEL_ID  PANEL_TITLE  DATASOURCE  QUERY"));
    assert!(output.contains("7         CPU          prom-main   up"));
    assert!(output.contains("8         Logs         loki-main   {job=\"grafana\"}"));
}

#[test]
fn render_query_report_column_supports_org_columns() {
    let row = test_support::ExportInspectionQueryRow {
        org: "Main Org.".to_string(),
        org_id: "1".to_string(),
        dashboard_uid: "main".to_string(),
        dashboard_title: "Main".to_string(),
        dashboard_tags: Vec::new(),
        folder_path: "General".to_string(),
        folder_full_path: "/".to_string(),
        folder_level: "1".to_string(),
        folder_uid: "general".to_string(),
        parent_folder_uid: String::new(),
        panel_id: "1".to_string(),
        panel_title: "CPU".to_string(),
        panel_type: "timeseries".to_string(),
        panel_target_count: 0,
        panel_query_count: 0,
        panel_datasource_count: 0,
        panel_variables: Vec::new(),
        ref_id: "A".to_string(),
        datasource: "prom-main".to_string(),
        datasource_name: "prom-main".to_string(),
        datasource_uid: "prom-main".to_string(),
        datasource_org: String::new(),
        datasource_org_id: String::new(),
        datasource_database: String::new(),
        datasource_bucket: String::new(),
        datasource_organization: String::new(),
        datasource_index_pattern: String::new(),
        datasource_type: "prometheus".to_string(),
        datasource_family: "prometheus".to_string(),
        query_field: "expr".to_string(),
        target_hidden: "false".to_string(),
        target_disabled: "false".to_string(),
        query_text: "up".to_string(),
        query_variables: Vec::new(),
        metrics: vec!["up".to_string()],
        functions: Vec::new(),
        measurements: Vec::new(),
        buckets: Vec::new(),
        file_path: "/tmp/raw/main.json".to_string(),
    };

    assert_eq!(test_support::report_column_header("org"), "ORG");
    assert_eq!(test_support::report_column_header("org_id"), "ORG_ID");
    assert_eq!(
        test_support::render_query_report_column(&row, "org"),
        "Main Org."
    );
    assert_eq!(
        test_support::render_query_report_column(&row, "org_id"),
        "1"
    );
}

#[test]
fn render_query_report_column_supports_variable_and_count_columns() {
    let row = test_support::ExportInspectionQueryRow {
        dashboard_tags: vec!["prod".to_string(), "infra".to_string()],
        query_variables: vec!["host".to_string(), "job".to_string()],
        panel_variables: vec!["cluster".to_string(), "team".to_string()],
        panel_target_count: 3,
        panel_query_count: 2,
        panel_datasource_count: 1,
        target_hidden: "true".to_string(),
        target_disabled: "false".to_string(),
        ..Default::default()
    };

    assert_eq!(
        test_support::report_column_header("dashboard_tags"),
        "DASHBOARD_TAGS"
    );
    assert_eq!(
        test_support::report_column_header("query_variables"),
        "QUERY_VARIABLES"
    );
    assert_eq!(
        test_support::report_column_header("panel_variables"),
        "PANEL_VARIABLES"
    );
    assert_eq!(
        test_support::report_column_header("panel_target_count"),
        "PANEL_TARGET_COUNT"
    );
    assert_eq!(
        test_support::report_column_header("panel_query_count"),
        "PANEL_EFFECTIVE_QUERY_COUNT"
    );
    assert_eq!(
        test_support::report_column_header("panel_datasource_count"),
        "PANEL_TOTAL_DATASOURCE_COUNT"
    );
    assert_eq!(
        test_support::report_column_header("target_hidden"),
        "TARGET_HIDDEN"
    );
    assert_eq!(
        test_support::report_column_header("target_disabled"),
        "TARGET_DISABLED"
    );
    assert_eq!(
        test_support::render_query_report_column(&row, "dashboard_tags"),
        "prod,infra"
    );
    assert_eq!(
        test_support::render_query_report_column(&row, "query_variables"),
        "host,job"
    );
    assert_eq!(
        test_support::render_query_report_column(&row, "panel_variables"),
        "cluster,team"
    );
    assert_eq!(
        test_support::render_query_report_column(&row, "panel_target_count"),
        "3"
    );
    assert_eq!(
        test_support::render_query_report_column(&row, "panel_query_count"),
        "2"
    );
    assert_eq!(
        test_support::render_query_report_column(&row, "panel_datasource_count"),
        "1"
    );
    assert_eq!(
        test_support::render_query_report_column(&row, "target_hidden"),
        "true"
    );
    assert_eq!(
        test_support::render_query_report_column(&row, "target_disabled"),
        "false"
    );
}

#[test]
fn render_grouped_query_table_report_includes_loki_analysis_columns() {
    let report = test_support::ExportInspectionQueryReport {
        import_dir: "/tmp/raw".to_string(),
        summary: test_support::QueryReportSummary {
            dashboard_count: 1,
            panel_count: 1,
            query_count: 1,
            report_row_count: 1,
        },
        queries: vec![test_support::ExportInspectionQueryRow {
            org: "Main Org.".to_string(),
            org_id: "1".to_string(),
            dashboard_uid: "logs-main".to_string(),
            dashboard_title: "Logs Main".to_string(),
            dashboard_tags: Vec::new(),
            folder_path: "Logs".to_string(),
            folder_full_path: "/Logs".to_string(),
            folder_level: "1".to_string(),
            folder_uid: "logs".to_string(),
            parent_folder_uid: String::new(),
            panel_id: "11".to_string(),
            panel_title: "Errors".to_string(),
            panel_type: "logs".to_string(),
            panel_target_count: 0,
            panel_query_count: 0,
            panel_datasource_count: 0,
            panel_variables: Vec::new(),
            ref_id: "A".to_string(),
            datasource: "loki-main".to_string(),
            datasource_name: "loki-main".to_string(),
            datasource_uid: "loki-main".to_string(),
            datasource_org: String::new(),
            datasource_org_id: String::new(),
            datasource_database: String::new(),
            datasource_bucket: String::new(),
            datasource_organization: String::new(),
            datasource_index_pattern: String::new(),
            datasource_type: "loki".to_string(),
            datasource_family: "loki".to_string(),
            query_field: "expr".to_string(),
            target_hidden: "false".to_string(),
            target_disabled: "false".to_string(),
            query_text: "{job=\"varlogs\",app=~\"api|web\"} |= \"error\" | json [5m]".to_string(),
            query_variables: Vec::new(),
            metrics: Vec::new(),
            functions: vec![
                "sum".to_string(),
                "count_over_time".to_string(),
                "filter_eq".to_string(),
                "json".to_string(),
            ],
            measurements: vec![
                "job=\"varlogs\"".to_string(),
                "app=~\"api|web\"".to_string(),
            ],
            buckets: vec!["5m".to_string()],
            file_path: "/tmp/raw/logs-main.json".to_string(),
        }],
    };

    let lines = test_support::render_grouped_query_table_report(
        &report,
        &[
            "panel_id".to_string(),
            "datasource".to_string(),
            "functions".to_string(),
            "measurements".to_string(),
            "buckets".to_string(),
            "query".to_string(),
        ],
        true,
    );
    let output = lines.join("\n");

    assert!(output.contains("PANEL_ID  DATASOURCE  FUNCTIONS"));
    assert!(output.contains("11"));
    assert!(output.contains("loki-main"));
    assert!(output.contains("sum,count_over_time,filter_eq,json"));
    assert!(output.contains("job=\"varlogs\",app=~\"api|web\""));
    assert!(output.contains("5m"));
    assert!(output.contains("{job=\"varlogs\",app=~\"api|web\"} |= \"error\" | json [5m]"));
}

#[test]
fn render_grouped_query_table_report_uses_default_column_set_when_requested() {
    let columns = test_support::resolve_report_column_ids(&[]).unwrap();
    assert_eq!(
        columns,
        test_support::DEFAULT_REPORT_COLUMN_IDS
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<String>>()
    );
}

#[test]
fn evaluate_dashboard_governance_gate_enforces_query_thresholds_and_warning_policy() {
    let policy = json!({
        "version": 1,
        "queries": {
            "maxQueriesPerDashboard": 1,
            "maxQueriesPerPanel": 1
        },
        "enforcement": {
            "failOnWarnings": true
        }
    });
    let governance = json!({
        "summary": {
            "dashboardCount": 1,
            "queryRecordCount": 2
        },
        "riskRecords": [
            {
                "kind": "broad-loki-selector",
                "dashboardUid": "core-main",
                "panelId": "7",
                "datasource": "Logs Main",
                "detail": "{}",
                "recommendation": "Narrow the Loki stream selector."
            }
        ]
    });
    let queries = json!({
        "summary": {
            "dashboardCount": 1,
            "queryRecordCount": 2
        },
        "queries": [
            {
                "dashboardUid": "core-main",
                "dashboardTitle": "Core Main",
                "panelId": "7",
                "panelTitle": "Errors",
                "refId": "A",
                "datasource": "Logs Main",
                "datasourceUid": "logs-main",
                "datasourceFamily": "loki"
            },
            {
                "dashboardUid": "core-main",
                "dashboardTitle": "Core Main",
                "panelId": "7",
                "panelTitle": "Errors",
                "refId": "B",
                "datasource": "Logs Main",
                "datasourceUid": "logs-main",
                "datasourceFamily": "loki"
            }
        ]
    });

    let result =
        test_support::evaluate_dashboard_governance_gate(&policy, &governance, &queries).unwrap();

    assert!(!result.ok);
    assert_eq!(result.summary.dashboard_count, 1);
    assert_eq!(result.summary.query_record_count, 2);
    assert_eq!(result.summary.violation_count, 2);
    assert_eq!(result.summary.warning_count, 1);
    assert_eq!(
        result.summary.checked_rules,
        json!({
            "datasourceAllowedFamilies": [],
            "datasourceAllowedUids": [],
            "allowedFolderPrefixes": [],
            "forbidUnknown": false,
            "forbidMixedFamilies": false,
            "forbidSelectStar": false,
            "requireSqlTimeFilter": false,
            "forbidBroadLokiRegex": false,
            "forbidBroadPrometheusSelectors": false,
            "forbidRegexHeavyPrometheus": false,
            "forbidHighCardinalityRegex": false,
            "maxPrometheusRangeWindowSeconds": null,
            "maxPrometheusAggregationDepth": null,
            "maxPrometheusCostScore": null,
            "forbidUnscopedLokiSearch": false,
            "maxPanelsPerDashboard": null,
            "minRefreshIntervalSeconds": null,
            "maxAuditScore": null,
            "maxReasonCount": null,
            "blockReasons": [],
            "maxDashboardLoadScore": null,
            "maxQueryComplexityScore": null,
            "maxDashboardComplexityScore": null,
            "maxQueriesPerDashboard": 1,
            "maxQueriesPerPanel": 1,
            "failOnWarnings": true
        })
    );
    assert_eq!(result.violations[0].code, "max-queries-per-dashboard");
    assert_eq!(result.violations[1].code, "max-queries-per-panel");
    assert_eq!(result.warnings[0].risk_kind, "broad-loki-selector");
}

#[test]
fn evaluate_dashboard_governance_gate_enforces_perf_and_dashboard_pressure_rules() {
    let policy = json!({
        "version": 1,
        "queries": {
            "forbidBroadPrometheusSelectors": true,
            "forbidRegexHeavyPrometheus": true,
            "forbidHighCardinalityRegex": false,
            "maxPrometheusRangeWindowSeconds": 3600,
            "maxPrometheusAggregationDepth": null,
            "maxPrometheusCostScore": null,
            "forbidUnscopedLokiSearch": true
        },
        "dashboards": {
            "maxPanelsPerDashboard": 30,
            "minRefreshIntervalSeconds": 30
        }
    });
    let governance = json!({
        "dashboardGovernance": [
            {
                "dashboardUid": "cpu-main",
                "dashboardTitle": "CPU Main",
                "panelCount": 31,
                "queryCount": 4,
                "datasourceFamilies": ["prometheus", "loki"],
                "mixedDatasource": false
            }
        ],
        "riskRecords": [
            {
                "kind": "dashboard-refresh-pressure",
                "dashboardUid": "cpu-main",
                "panelId": "",
                "datasource": "CPU Main",
                "detail": "5s",
                "recommendation": "Increase refresh."
            }
        ]
    });
    let queries = json!({
        "summary": {
            "dashboardCount": 1,
            "queryRecordCount": 4
        },
        "queries": [
            {
                "dashboardUid": "cpu-main",
                "dashboardTitle": "CPU Main",
                "panelId": "7",
                "panelTitle": "CPU",
                "refId": "A",
                "datasource": "Prometheus Main",
                "datasourceUid": "prom-main",
                "datasourceFamily": "prometheus",
                "query": "up",
                "refresh": "5s",
                "metrics": ["up"],
                "functions": [],
                "measurements": [],
                "buckets": []
            },
            {
                "dashboardUid": "cpu-main",
                "dashboardTitle": "CPU Main",
                "panelId": "8",
                "panelTitle": "Regex",
                "refId": "B",
                "datasource": "Prometheus Main",
                "datasourceUid": "prom-main",
                "datasourceFamily": "prometheus",
                "query": "sum(rate(http_requests_total{job=~\"api|web\"}[5m]))",
                "metrics": ["http_requests_total"],
                "functions": ["sum", "rate"],
                "measurements": ["job=~\"api|web\""],
                "buckets": ["5m"]
            },
            {
                "dashboardUid": "cpu-main",
                "dashboardTitle": "CPU Main",
                "panelId": "9",
                "panelTitle": "Long",
                "refId": "C",
                "datasource": "Prometheus Main",
                "datasourceUid": "prom-main",
                "datasourceFamily": "prometheus",
                "query": "sum(rate(process_cpu_seconds_total[6h]))",
                "metrics": ["process_cpu_seconds_total"],
                "functions": ["sum", "rate"],
                "measurements": [],
                "buckets": ["6h"]
            },
            {
                "dashboardUid": "cpu-main",
                "dashboardTitle": "CPU Main",
                "panelId": "10",
                "panelTitle": "Logs",
                "refId": "D",
                "datasource": "Logs Main",
                "datasourceUid": "logs-main",
                "datasourceFamily": "loki",
                "query": "{} |= \"error\"",
                "metrics": [],
                "functions": ["line_filter_contains"],
                "measurements": ["{}"],
                "buckets": []
            }
        ]
    });

    let result =
        test_support::evaluate_dashboard_governance_gate(&policy, &governance, &queries).unwrap();
    let codes = result
        .violations
        .iter()
        .map(|item| item.code.as_str())
        .collect::<Vec<_>>();
    assert!(codes.contains(&"prometheus-broad-selector"));
    assert!(codes.contains(&"prometheus-regex-heavy"));
    assert!(codes.contains(&"prometheus-range-window-too-large"));
    assert!(codes.contains(&"loki-unscoped-search"));
    assert!(codes.contains(&"max-panels-per-dashboard"));
    assert!(codes.contains(&"min-refresh-interval-seconds"));
}

#[test]
fn evaluate_dashboard_governance_gate_enforces_query_audit_contract_rules() {
    let policy = json!({
        "version": 1,
        "queries": {
            "maxAuditScore": 2,
            "maxReasonCount": 1,
            "blockReasons": ["unscoped-loki-search"]
        },
        "dashboards": {
            "maxDashboardLoadScore": 2
        }
    });
    let governance = json!({
        "summary": {
            "dashboardCount": 1,
            "queryRecordCount": 2
        },
        "dashboardGovernance": [
            {
                "dashboardUid": "cpu-main",
                "dashboardTitle": "CPU Main",
                "panelCount": 31,
                "queryCount": 2
            }
        ],
        "riskRecords": [],
        "queryAudits": [
            {
                "dashboardUid": "cpu-main",
                "dashboardTitle": "CPU Main",
                "folderPath": "Platform",
                "panelId": "7",
                "panelTitle": "CPU",
                "refId": "A",
                "datasource": "Prometheus Main",
                "datasourceUid": "prom-main",
                "datasourceFamily": "prometheus",
                "aggregationDepth": 1,
                "regexMatcherCount": 1,
                "estimatedSeriesRisk": "medium",
                "queryCostScore": 3,
                "score": 3,
                "severity": "medium",
                "reasons": ["broad-prometheus-selector", "prometheus-regex-heavy"],
                "recommendations": ["scope it"]
            },
            {
                "dashboardUid": "cpu-main",
                "dashboardTitle": "CPU Main",
                "folderPath": "Platform",
                "panelId": "8",
                "panelTitle": "Logs",
                "refId": "B",
                "datasource": "Logs Main",
                "datasourceUid": "logs-main",
                "datasourceFamily": "loki",
                "aggregationDepth": 0,
                "regexMatcherCount": 0,
                "estimatedSeriesRisk": "low",
                "queryCostScore": 4,
                "score": 4,
                "severity": "high",
                "reasons": ["unscoped-loki-search"],
                "recommendations": ["scope labels"]
            }
        ],
        "dashboardAudits": [
            {
                "dashboardUid": "cpu-main",
                "dashboardTitle": "CPU Main",
                "folderPath": "Platform",
                "panelCount": 31,
                "queryCount": 2,
                "refreshIntervalSeconds": 5,
                "score": 4,
                "severity": "high",
                "reasons": ["dashboard-panel-pressure", "dashboard-refresh-pressure"],
                "recommendations": ["split it"]
            }
        ]
    });
    let queries = json!({
        "summary": {
            "dashboardCount": 1,
            "queryRecordCount": 2
        },
        "queries": []
    });

    let result =
        test_support::evaluate_dashboard_governance_gate(&policy, &governance, &queries).unwrap();
    let codes = result
        .violations
        .iter()
        .map(|item| item.code.as_str())
        .collect::<Vec<_>>();
    assert!(codes.contains(&"query-audit-score-too-high"));
    assert!(codes.contains(&"query-audit-reason-count-too-high"));
    assert!(codes.contains(&"query-audit-blocked-reason"));
    assert!(codes.contains(&"dashboard-load-score-too-high"));
}

#[test]
fn evaluate_dashboard_governance_gate_enforces_prometheus_cost_policy_rules() {
    let policy = json!({
        "version": 1,
        "queries": {
            "forbidHighCardinalityRegex": true,
            "maxPrometheusAggregationDepth": 1,
            "maxPrometheusCostScore": 3
        }
    });
    let governance = json!({
        "summary": {
            "dashboardCount": 1,
            "queryRecordCount": 1
        },
        "riskRecords": [],
        "queryAudits": [
            {
                "dashboardUid": "cpu-main",
                "dashboardTitle": "CPU Main",
                "folderPath": "Platform",
                "panelId": "7",
                "panelTitle": "CPU",
                "refId": "A",
                "datasource": "Prometheus Main",
                "datasourceUid": "prom-main",
                "datasourceFamily": "prometheus",
                "aggregationDepth": 2,
                "regexMatcherCount": 2,
                "estimatedSeriesRisk": "high",
                "queryCostScore": 5,
                "score": 5,
                "severity": "high",
                "reasons": ["prometheus-high-cardinality-regex", "prometheus-deep-aggregation"],
                "recommendations": ["scope it"]
            }
        ],
        "dashboardAudits": []
    });
    let queries = json!({
        "summary": {
            "dashboardCount": 1,
            "queryRecordCount": 1
        },
        "queries": []
    });

    let result =
        test_support::evaluate_dashboard_governance_gate(&policy, &governance, &queries).unwrap();
    let codes = result
        .violations
        .iter()
        .map(|item| item.code.as_str())
        .collect::<Vec<_>>();
    assert!(codes.contains(&"prometheus-high-cardinality-regex"));
    assert!(codes.contains(&"prometheus-aggregation-depth-too-high"));
    assert!(codes.contains(&"prometheus-cost-score-too-high"));
}

#[test]
fn render_dashboard_governance_gate_result_lists_violations_and_warnings() {
    let result = test_support::DashboardGovernanceGateResult {
        ok: false,
        summary: test_support::DashboardGovernanceGateSummary {
            dashboard_count: 1,
            query_record_count: 2,
            violation_count: 1,
            warning_count: 1,
            checked_rules: json!({
                "datasourceAllowedFamilies": [],
                "datasourceAllowedUids": [],
                "allowedFolderPrefixes": [],
                "forbidUnknown": false,
                "forbidMixedFamilies": false,
                "forbidSelectStar": false,
                "requireSqlTimeFilter": false,
                "forbidBroadLokiRegex": false,
                "forbidBroadPrometheusSelectors": false,
                "forbidRegexHeavyPrometheus": false,
                "forbidHighCardinalityRegex": false,
                "maxPrometheusRangeWindowSeconds": null,
                "maxPrometheusAggregationDepth": null,
                "maxPrometheusCostScore": null,
                "forbidUnscopedLokiSearch": false,
                "maxPanelsPerDashboard": null,
                "minRefreshIntervalSeconds": null,
                "maxAuditScore": null,
                "maxReasonCount": null,
                "blockReasons": [],
                "maxDashboardLoadScore": null,
                "maxQueryComplexityScore": null,
                "maxDashboardComplexityScore": null,
                "maxQueriesPerDashboard": 1,
                "maxQueriesPerPanel": null,
                "failOnWarnings": false
            }),
        },
        violations: vec![test_support::DashboardGovernanceGateFinding {
            severity: "error".to_string(),
            code: "max-queries-per-dashboard".to_string(),
            message: "Dashboard query count 2 exceeds policy maxQueriesPerDashboard=1.".to_string(),
            dashboard_uid: "core-main".to_string(),
            dashboard_title: "Core Main".to_string(),
            panel_id: String::new(),
            panel_title: String::new(),
            ref_id: String::new(),
            datasource: String::new(),
            datasource_uid: String::new(),
            datasource_family: String::new(),
            risk_kind: String::new(),
        }],
        warnings: vec![test_support::DashboardGovernanceGateFinding {
            severity: "warning".to_string(),
            code: "broad-loki-selector".to_string(),
            message: "Narrow the Loki stream selector.".to_string(),
            dashboard_uid: "core-main".to_string(),
            dashboard_title: String::new(),
            panel_id: "7".to_string(),
            panel_title: String::new(),
            ref_id: String::new(),
            datasource: "Logs Main".to_string(),
            datasource_uid: String::new(),
            datasource_family: String::new(),
            risk_kind: "broad-loki-selector".to_string(),
        }],
    };

    let output = render_dashboard_governance_gate_result(&result);
    assert!(output.contains("Dashboard governance gate: FAIL"));
    assert!(output.contains("Violations:"));
    assert!(output.contains("Warnings:"));
    assert!(output.contains("max-queries-per-dashboard"));
    assert!(output.contains("broad-loki-selector"));
}

#[test]
fn run_dashboard_governance_gate_writes_json_output_file() {
    let temp = tempdir().unwrap();
    let policy_path = temp.path().join("policy.json");
    let governance_path = temp.path().join("governance.json");
    let queries_path = temp.path().join("queries.json");
    let json_output = temp.path().join("governance-check.json");

    fs::write(
        &policy_path,
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "datasources": {
                "allowedFamilies": [],
                "allowedUids": []
            },
            "queries": {
                "maxQueriesPerDashboard": 4,
                "maxQueriesPerPanel": 2
            },
            "enforcement": {
                "failOnWarnings": false
            }
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        &governance_path,
        serde_json::to_string_pretty(&json!({
            "summary": {
                "dashboardCount": 1,
                "queryRecordCount": 2
            },
            "riskRecords": []
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        &queries_path,
        serde_json::to_string_pretty(&json!({
            "summary": {
                "dashboardCount": 1,
                "queryRecordCount": 2
            },
            "queries": [
                {
                    "dashboardUid": "core-main",
                    "dashboardTitle": "Core Main",
                    "panelId": "7",
                    "panelTitle": "Errors",
                    "refId": "A"
                },
                {
                    "dashboardUid": "core-main",
                    "dashboardTitle": "Core Main",
                    "panelId": "8",
                    "panelTitle": "Latency",
                    "refId": "B"
                }
            ]
        }))
        .unwrap(),
    )
    .unwrap();

    let args = GovernanceGateArgs {
        policy: policy_path,
        governance: governance_path,
        queries: queries_path,
        output_format: GovernanceGateOutputFormat::Json,
        json_output: Some(json_output.clone()),
        interactive: false,
    };

    test_support::run_dashboard_governance_gate(&args).unwrap();
    let output = read_json_output_file(&json_output);
    assert_eq!(output["ok"], json!(true));
    assert_eq!(output["summary"]["violationCount"], json!(0));
    assert_eq!(output["summary"]["warningCount"], json!(0));
    assert_eq!(
        output["summary"]["checkedRules"],
        json!({
            "datasourceAllowedFamilies": [],
            "datasourceAllowedUids": [],
            "allowedFolderPrefixes": [],
            "forbidUnknown": false,
            "forbidMixedFamilies": false,
            "forbidSelectStar": false,
            "requireSqlTimeFilter": false,
            "forbidBroadLokiRegex": false,
            "forbidBroadPrometheusSelectors": false,
            "forbidRegexHeavyPrometheus": false,
            "forbidHighCardinalityRegex": false,
            "maxPrometheusRangeWindowSeconds": null,
            "maxPrometheusAggregationDepth": null,
            "maxPrometheusCostScore": null,
            "forbidUnscopedLokiSearch": false,
            "maxPanelsPerDashboard": null,
            "minRefreshIntervalSeconds": null,
            "maxAuditScore": null,
            "maxReasonCount": null,
            "blockReasons": [],
            "maxDashboardLoadScore": null,
            "maxQueryComplexityScore": null,
            "maxDashboardComplexityScore": null,
            "maxQueriesPerDashboard": 4,
            "maxQueriesPerPanel": 2,
            "failOnWarnings": false
        })
    );
    assert_eq!(output["violations"], json!([]));
    assert_eq!(output["warnings"], json!([]));
}

#[test]
fn evaluate_dashboard_governance_gate_enforces_datasource_policy_rules() {
    let policy = json!({
        "version": 1,
        "datasources": {
            "allowedFamilies": ["prometheus"],
            "allowedUids": ["prom-main"],
            "forbidUnknown": true,
            "forbidMixedFamilies": true
        },
        "enforcement": {
            "failOnWarnings": false
        }
    });
    let governance = json!({
        "summary": {
            "dashboardCount": 1,
            "queryRecordCount": 2
        },
        "dashboardGovernance": [
            {
                "dashboardUid": "mixed-main",
                "dashboardTitle": "Mixed Main",
                "datasourceFamilies": ["prometheus", "unknown"],
                "mixedDatasource": true
            }
        ],
        "riskRecords": []
    });
    let queries = json!({
        "summary": {
            "dashboardCount": 1,
            "queryRecordCount": 2
        },
        "queries": [
            {
                "dashboardUid": "mixed-main",
                "dashboardTitle": "Mixed Main",
                "panelId": "7",
                "panelTitle": "CPU",
                "refId": "A",
                "datasource": "Prometheus Main",
                "datasourceUid": "prom-main",
                "datasourceFamily": "prometheus"
            },
            {
                "dashboardUid": "mixed-main",
                "dashboardTitle": "Mixed Main",
                "panelId": "8",
                "panelTitle": "Custom",
                "refId": "B",
                "datasource": "",
                "datasourceUid": "custom-main",
                "datasourceFamily": "unknown"
            }
        ]
    });

    let result =
        test_support::evaluate_dashboard_governance_gate(&policy, &governance, &queries).unwrap();
    let codes = result
        .violations
        .iter()
        .map(|item| item.code.as_str())
        .collect::<Vec<&str>>();

    assert!(!result.ok);
    assert_eq!(result.summary.violation_count, 4);
    assert!(codes.contains(&"datasource-unknown"));
    assert!(codes.contains(&"datasource-family-not-allowed"));
    assert!(codes.contains(&"datasource-uid-not-allowed"));
    assert!(codes.contains(&"mixed-datasource-families-not-allowed"));
    assert_eq!(
        result.summary.checked_rules,
        json!({
            "datasourceAllowedFamilies": ["prometheus"],
            "datasourceAllowedUids": ["prom-main"],
            "allowedFolderPrefixes": [],
            "forbidUnknown": true,
            "forbidMixedFamilies": true,
            "forbidSelectStar": false,
            "requireSqlTimeFilter": false,
            "forbidBroadLokiRegex": false,
            "forbidBroadPrometheusSelectors": false,
            "forbidRegexHeavyPrometheus": false,
            "forbidHighCardinalityRegex": false,
            "maxPrometheusRangeWindowSeconds": null,
            "maxPrometheusAggregationDepth": null,
            "maxPrometheusCostScore": null,
            "forbidUnscopedLokiSearch": false,
            "maxPanelsPerDashboard": null,
            "minRefreshIntervalSeconds": null,
            "maxAuditScore": null,
            "maxReasonCount": null,
            "blockReasons": [],
            "maxDashboardLoadScore": null,
            "maxQueryComplexityScore": null,
            "maxDashboardComplexityScore": null,
            "maxQueriesPerDashboard": null,
            "maxQueriesPerPanel": null,
            "failOnWarnings": false
        })
    );
}

#[test]
fn evaluate_dashboard_governance_gate_enforces_routing_sql_and_loki_policy_rules() {
    let policy = json!({
        "version": 1,
        "routing": {
            "allowedFolderPrefixes": ["Platform"]
        },
        "queries": {
            "forbidSelectStar": true,
            "requireSqlTimeFilter": true,
            "forbidBroadLokiRegex": true
        }
    });
    let governance = json!({
        "summary": {
            "dashboardCount": 2,
            "queryRecordCount": 3
        },
        "dashboardGovernance": [],
        "riskRecords": []
    });
    let queries = json!({
        "summary": {
            "dashboardCount": 2,
            "queryRecordCount": 3
        },
        "queries": [
            {
                "dashboardUid": "sql-main",
                "dashboardTitle": "SQL Main",
                "folderPath": "Operations",
                "panelId": "7",
                "panelTitle": "Rows",
                "refId": "A",
                "datasource": "Warehouse",
                "datasourceUid": "sql-main",
                "datasourceFamily": "sql",
                "query": "SELECT * FROM metrics"
            },
            {
                "dashboardUid": "sql-main",
                "dashboardTitle": "SQL Main",
                "folderPath": "Operations",
                "panelId": "8",
                "panelTitle": "Latency",
                "refId": "B",
                "datasource": "Warehouse",
                "datasourceUid": "sql-main",
                "datasourceFamily": "sql",
                "query": "SELECT count(*) FROM metrics"
            },
            {
                "dashboardUid": "logs-main",
                "dashboardTitle": "Logs Main",
                "folderPath": "Platform / Logs",
                "panelId": "9",
                "panelTitle": "Errors",
                "refId": "C",
                "datasource": "Logs Main",
                "datasourceUid": "logs-main",
                "datasourceFamily": "loki",
                "query": "{namespace=~\".*\"} |~ \".*\""
            }
        ]
    });

    let result =
        test_support::evaluate_dashboard_governance_gate(&policy, &governance, &queries).unwrap();
    let codes = result
        .violations
        .iter()
        .map(|item| item.code.as_str())
        .collect::<Vec<&str>>();

    assert!(!result.ok);
    assert_eq!(result.summary.violation_count, 6);
    assert!(codes.contains(&"routing-folder-not-allowed"));
    assert!(codes.contains(&"sql-select-star"));
    assert!(codes.contains(&"sql-missing-time-filter"));
    assert!(codes.contains(&"loki-broad-regex"));
    assert_eq!(
        result.summary.checked_rules,
        json!({
            "datasourceAllowedFamilies": [],
            "datasourceAllowedUids": [],
            "allowedFolderPrefixes": ["Platform"],
            "forbidUnknown": false,
            "forbidMixedFamilies": false,
            "forbidSelectStar": true,
            "requireSqlTimeFilter": true,
            "forbidBroadLokiRegex": true,
            "forbidBroadPrometheusSelectors": false,
            "forbidRegexHeavyPrometheus": false,
            "forbidHighCardinalityRegex": false,
            "maxPrometheusRangeWindowSeconds": null,
            "maxPrometheusAggregationDepth": null,
            "maxPrometheusCostScore": null,
            "forbidUnscopedLokiSearch": false,
            "maxPanelsPerDashboard": null,
            "minRefreshIntervalSeconds": null,
            "maxAuditScore": null,
            "maxReasonCount": null,
            "blockReasons": [],
            "maxDashboardLoadScore": null,
            "maxQueryComplexityScore": null,
            "maxDashboardComplexityScore": null,
            "maxQueriesPerDashboard": null,
            "maxQueriesPerPanel": null,
            "failOnWarnings": false
        })
    );
}

#[test]
fn evaluate_dashboard_governance_gate_enforces_query_and_dashboard_complexity_rules() {
    let policy = json!({
        "version": 1,
        "queries": {
            "maxQueryComplexityScore": 3,
            "maxDashboardComplexityScore": 6
        }
    });
    let governance = json!({
        "summary": {
            "dashboardCount": 1,
            "queryRecordCount": 2
        },
        "dashboardGovernance": [],
        "riskRecords": []
    });
    let queries = json!({
        "summary": {
            "dashboardCount": 1,
            "queryRecordCount": 2
        },
        "queries": [
            {
                "dashboardUid": "core-main",
                "dashboardTitle": "Core Main",
                "folderPath": "Platform",
                "panelId": "7",
                "panelTitle": "CPU",
                "refId": "A",
                "datasource": "Prometheus Main",
                "datasourceUid": "prom-main",
                "datasourceFamily": "prometheus",
                "metrics": ["http_requests_total", "process_cpu_seconds_total"],
                "measurements": ["job=\"grafana\""],
                "buckets": ["5m"],
                "query": "sum(rate(http_requests_total{job=~\"grafana\"}[5m]))"
            },
            {
                "dashboardUid": "core-main",
                "dashboardTitle": "Core Main",
                "folderPath": "Platform",
                "panelId": "8",
                "panelTitle": "Memory",
                "refId": "B",
                "datasource": "Prometheus Main",
                "datasourceUid": "prom-main",
                "datasourceFamily": "prometheus",
                "metrics": ["node_memory_MemAvailable_bytes"],
                "measurements": [],
                "buckets": [],
                "query": "max(node_memory_MemAvailable_bytes)"
            }
        ]
    });

    let result =
        test_support::evaluate_dashboard_governance_gate(&policy, &governance, &queries).unwrap();
    let codes = result
        .violations
        .iter()
        .map(|item| item.code.as_str())
        .collect::<Vec<&str>>();

    assert!(!result.ok);
    assert!(codes.contains(&"query-complexity-too-high"));
    assert!(codes.contains(&"dashboard-complexity-too-high"));
    assert_eq!(
        result.summary.checked_rules,
        json!({
            "datasourceAllowedFamilies": [],
            "datasourceAllowedUids": [],
            "allowedFolderPrefixes": [],
            "forbidUnknown": false,
            "forbidMixedFamilies": false,
            "forbidSelectStar": false,
            "requireSqlTimeFilter": false,
            "forbidBroadLokiRegex": false,
            "forbidBroadPrometheusSelectors": false,
            "forbidRegexHeavyPrometheus": false,
            "forbidHighCardinalityRegex": false,
            "maxPrometheusRangeWindowSeconds": null,
            "maxPrometheusAggregationDepth": null,
            "maxPrometheusCostScore": null,
            "forbidUnscopedLokiSearch": false,
            "maxPanelsPerDashboard": null,
            "minRefreshIntervalSeconds": null,
            "maxAuditScore": null,
            "maxReasonCount": null,
            "blockReasons": [],
            "maxDashboardLoadScore": null,
            "maxQueryComplexityScore": 3,
            "maxDashboardComplexityScore": 6,
            "maxQueriesPerDashboard": null,
            "maxQueriesPerPanel": null,
            "failOnWarnings": false
        })
    );
}

#[test]
fn build_export_inspection_governance_document_groups_core_family_dependency_rows() {
    let cases = [
        (
            "search",
            &["elasticsearch", "opensearch"][..],
            "status:500 AND service.name:\"api\"",
            &["service.name", "status"][..],
        ),
        (
            "tracing",
            &["jaeger", "tempo", "zipkin"][..],
            "service.name:api AND span.name:checkout AND traceID:abc123",
            &["service.name", "span.name", "traceID"][..],
        ),
    ];

    for (family, datasource_types, query_text, measurements) in cases {
        let dashboard_uid = format!("{family}-main");
        let queries = datasource_types
            .iter()
            .enumerate()
            .map(|(index, datasource_type)| {
                let panel_id = (index + 1).to_string();
                let ref_id = ((b'A' + index as u8) as char).to_string();
                let datasource_name = format!("{datasource_type}-main");
                let datasource_uid = format!("{datasource_type}-uid");
                make_core_family_report_row(
                    dashboard_uid.as_str(),
                    &panel_id,
                    &ref_id,
                    &datasource_uid,
                    &datasource_name,
                    datasource_type,
                    family,
                    query_text,
                    measurements,
                )
            })
            .collect::<Vec<test_support::ExportInspectionQueryRow>>();
        let summary = test_support::ExportInspectionSummary {
            import_dir: "/tmp/raw".to_string(),
            export_org: None,
            export_org_id: None,
            dashboard_count: 1,
            folder_count: 1,
            panel_count: datasource_types.len(),
            query_count: datasource_types.len(),
            datasource_inventory_count: 0,
            orphaned_datasource_count: 0,
            mixed_dashboard_count: 0,
            folder_paths: Vec::new(),
            datasource_usage: Vec::new(),
            datasource_inventory: Vec::new(),
            orphaned_datasources: Vec::new(),
            mixed_dashboards: Vec::new(),
        };
        let report = test_support::ExportInspectionQueryReport {
            import_dir: "/tmp/raw".to_string(),
            summary: test_support::QueryReportSummary {
                dashboard_count: 1,
                panel_count: datasource_types.len(),
                query_count: datasource_types.len(),
                report_row_count: datasource_types.len(),
            },
            queries,
        };

        let document = test_support::build_export_inspection_governance_document(&summary, &report);

        assert_eq!(document.summary.dashboard_count, 1);
        assert_eq!(document.summary.query_record_count, datasource_types.len());
        assert_eq!(document.summary.datasource_family_count, 1);
        assert_eq!(document.summary.risk_record_count, 0);
        assert_eq!(document.datasource_families.len(), 1);
        assert_eq!(document.datasource_families[0].family, family);
        assert_eq!(
            document.datasource_families[0].datasource_types,
            datasource_types
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<String>>()
        );
        assert_eq!(document.dashboard_dependencies.len(), 1);
        assert_eq!(
            document.dashboard_dependencies[0].datasource_families,
            vec![family.to_string()]
        );
        assert_eq!(
            document.dashboard_dependencies[0].query_fields,
            vec!["query".to_string()]
        );
        assert!(document.dashboard_dependencies[0].metrics.is_empty());
        assert!(document.dashboard_dependencies[0].functions.is_empty());
        assert!(document.risk_records.is_empty());
    }
}

#[test]
fn build_export_inspection_governance_document_rolls_up_dashboard_dependency_analysis() {
    let summary = test_support::ExportInspectionSummary {
        import_dir: "/tmp/raw".to_string(),
        export_org: Some("Main Org.".to_string()),
        export_org_id: Some("1".to_string()),
        dashboard_count: 1,
        folder_count: 1,
        panel_count: 2,
        query_count: 2,
        datasource_inventory_count: 1,
        orphaned_datasource_count: 0,
        mixed_dashboard_count: 0,
        folder_paths: Vec::new(),
        datasource_usage: Vec::new(),
        datasource_inventory: vec![test_support::DatasourceInventorySummary {
            uid: "prom-main".to_string(),
            name: "Prometheus Main".to_string(),
            datasource_type: "prometheus".to_string(),
            access: "proxy".to_string(),
            url: "http://prometheus:9090".to_string(),
            is_default: "true".to_string(),
            org: "Main Org.".to_string(),
            org_id: "1".to_string(),
            reference_count: 2,
            dashboard_count: 1,
        }],
        orphaned_datasources: Vec::new(),
        mixed_dashboards: Vec::new(),
    };
    let mut query_a = make_core_family_report_row(
        "core-main",
        "7",
        "A",
        "prom-main",
        "Prometheus Main",
        "prometheus",
        "prometheus",
        "sum(rate(http_requests_total[5m]))",
        &["job=\"grafana\""],
    );
    query_a.query_field = "expr".to_string();
    query_a.metrics = vec!["http_requests_total".to_string()];
    query_a.functions = vec!["rate".to_string()];
    query_a.measurements = vec!["job=\"grafana\"".to_string()];
    query_a.buckets = vec!["5m".to_string()];

    let mut query_b = make_core_family_report_row(
        "core-main",
        "8",
        "B",
        "prom-main",
        "Prometheus Main",
        "prometheus",
        "prometheus",
        "sum(rate(process_cpu_seconds_total[1h]))",
        &["service.name"],
    );
    query_b.query_field = "query".to_string();
    query_b.metrics = vec![
        "http_requests_total".to_string(),
        "process_cpu_seconds_total".to_string(),
    ];
    query_b.functions = vec!["rate".to_string(), "sum".to_string()];
    query_b.measurements = vec!["service.name".to_string(), "job=\"grafana\"".to_string()];
    query_b.buckets = vec!["1h".to_string(), "5m".to_string()];

    let report = test_support::ExportInspectionQueryReport {
        import_dir: "/tmp/raw".to_string(),
        summary: test_support::QueryReportSummary {
            dashboard_count: 1,
            panel_count: 2,
            query_count: 2,
            report_row_count: 2,
        },
        queries: vec![query_a, query_b],
    };

    let document = test_support::build_export_inspection_governance_document(&summary, &report);
    let document_json = serde_json::to_value(&document).unwrap();
    let dependency_row = &document_json["dashboardDependencies"][0];

    assert_eq!(document.summary.dashboard_count, 1);
    assert_eq!(document.summary.query_record_count, 2);
    assert_eq!(document.summary.datasource_family_count, 1);
    assert_eq!(document.summary.datasource_coverage_count, 1);
    assert_eq!(document.summary.dashboard_datasource_edge_count, 1);
    assert_eq!(document.summary.datasource_risk_coverage_count, 0);
    assert_eq!(document.summary.dashboard_risk_coverage_count, 1);
    assert_eq!(document.summary.risk_record_count, 1);
    assert_eq!(dependency_row["queryFields"], json!(["expr", "query"]));
    assert_eq!(
        dependency_row["metrics"],
        json!(["http_requests_total", "process_cpu_seconds_total"])
    );
    assert_eq!(dependency_row["functions"], json!(["rate", "sum"]));
    assert_eq!(
        dependency_row["measurements"],
        json!(["job=\"grafana\"", "service.name"])
    );
    assert_eq!(dependency_row["buckets"], json!(["1h", "5m"]));
    assert_eq!(dependency_row["datasourceCount"], Value::from(1));
    assert_eq!(dependency_row["datasourceFamilyCount"], Value::from(1));
    assert_eq!(dependency_row["datasourceFamilies"], json!(["prometheus"]));
}

#[test]
fn build_export_inspection_governance_document_surfaces_datasource_blast_radius_dashboards() {
    let summary = test_support::ExportInspectionSummary {
        import_dir: "/tmp/raw".to_string(),
        export_org: Some("Main Org.".to_string()),
        export_org_id: Some("1".to_string()),
        dashboard_count: 1,
        folder_count: 1,
        panel_count: 2,
        query_count: 2,
        datasource_inventory_count: 1,
        orphaned_datasource_count: 0,
        mixed_dashboard_count: 0,
        folder_paths: Vec::new(),
        datasource_usage: Vec::new(),
        datasource_inventory: vec![test_support::DatasourceInventorySummary {
            uid: "prom-main".to_string(),
            name: "Prometheus Main".to_string(),
            datasource_type: "prometheus".to_string(),
            access: "proxy".to_string(),
            url: "http://prometheus:9090".to_string(),
            is_default: "true".to_string(),
            org: "Main Org.".to_string(),
            org_id: "1".to_string(),
            reference_count: 2,
            dashboard_count: 1,
        }],
        orphaned_datasources: Vec::new(),
        mixed_dashboards: Vec::new(),
    };
    let query_a = make_core_family_report_row(
        "core-main",
        "7",
        "A",
        "prom-main",
        "Prometheus Main",
        "prometheus",
        "prometheus",
        "sum(rate(http_requests_total[5m]))",
        &["job=\"grafana\""],
    );
    let mut query_b = make_core_family_report_row(
        "core-main",
        "8",
        "B",
        "prom-main",
        "Prometheus Main",
        "prometheus",
        "prometheus",
        "sum(rate(process_cpu_seconds_total[1h]))",
        &["job=\"grafana\""],
    );
    query_b.query_field = "expr".to_string();

    let report = test_support::ExportInspectionQueryReport {
        import_dir: "/tmp/raw".to_string(),
        summary: test_support::QueryReportSummary {
            dashboard_count: 1,
            panel_count: 2,
            query_count: 2,
            report_row_count: 2,
        },
        queries: vec![query_a, query_b],
    };

    let document = test_support::build_export_inspection_governance_document(&summary, &report);
    let document_json = serde_json::to_value(&document).unwrap();
    let datasource_row = &document_json["datasources"][0];
    let datasource_families = document_json["datasourceFamilies"].as_array().unwrap();

    assert_eq!(document.summary.dashboard_count, 1);
    assert_eq!(document.summary.query_record_count, 2);
    assert_eq!(document.summary.datasource_family_count, 1);
    assert_eq!(document.summary.datasource_coverage_count, 1);
    assert_eq!(document.summary.dashboard_datasource_edge_count, 1);
    assert_eq!(document.summary.datasource_risk_coverage_count, 0);
    assert_eq!(document.summary.dashboard_risk_coverage_count, 0);
    assert_eq!(document.summary.risk_record_count, 0);
    assert_eq!(datasource_row["dashboardUids"], json!(["core-main"]));
    assert_eq!(datasource_row["dashboardCount"], Value::from(1));
    assert_eq!(datasource_row["panelCount"], Value::from(2));
    assert_eq!(datasource_row["queryCount"], Value::from(2));
    assert_eq!(datasource_row["queryFields"], json!(["expr", "query"]));
    assert_eq!(datasource_families.len(), 1);
    let datasource_governance_row = &document_json["datasourceGovernance"][0];
    assert_eq!(
        datasource_governance_row["datasourceUid"],
        json!("prom-main")
    );
    assert_eq!(datasource_governance_row["riskKinds"], json!([]));
    assert_eq!(datasource_governance_row["mixedDashboardCount"], json!(0));

    let lines = test_support::render_governance_table_report("/tmp/raw", &document);
    let output = lines.join("\n");
    assert!(output.contains("DATASOURCES_WITH_RISKS"));
    assert!(output.contains("# Datasource Governance"));
    assert!(output.contains("RISK_KINDS"));
    assert!(output.contains("MIXED_DASHBOARDS"));
    assert!(output.contains("ORPHANED_DATASOURCES"));
    assert!(output.contains("DASHBOARD_UIDS"));
    assert!(output.contains("PANELS"));
    assert!(output.contains("core-main"));
}

#[test]
fn render_governance_table_report_displays_sections() {
    let summary = test_support::ExportInspectionSummary {
        import_dir: "/tmp/raw".to_string(),
        export_org: None,
        export_org_id: None,
        dashboard_count: 1,
        folder_count: 1,
        panel_count: 1,
        query_count: 1,
        datasource_inventory_count: 2,
        orphaned_datasource_count: 1,
        mixed_dashboard_count: 0,
        folder_paths: Vec::new(),
        datasource_usage: Vec::new(),
        datasource_inventory: vec![
            test_support::DatasourceInventorySummary {
                uid: "logs-main".to_string(),
                name: "Logs Main".to_string(),
                datasource_type: "loki".to_string(),
                access: "proxy".to_string(),
                url: "http://loki:3100".to_string(),
                is_default: "false".to_string(),
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
                reference_count: 1,
                dashboard_count: 1,
            },
            test_support::DatasourceInventorySummary {
                uid: "unused-main".to_string(),
                name: "Unused Main".to_string(),
                datasource_type: "tempo".to_string(),
                access: "proxy".to_string(),
                url: "http://tempo:3200".to_string(),
                is_default: "false".to_string(),
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
                reference_count: 0,
                dashboard_count: 0,
            },
        ],
        orphaned_datasources: vec![test_support::DatasourceInventorySummary {
            uid: "unused-main".to_string(),
            name: "Unused Main".to_string(),
            datasource_type: "tempo".to_string(),
            access: "proxy".to_string(),
            url: "http://tempo:3200".to_string(),
            is_default: "false".to_string(),
            org: "Main Org.".to_string(),
            org_id: "1".to_string(),
            reference_count: 0,
            dashboard_count: 0,
        }],
        mixed_dashboards: Vec::new(),
    };
    let report = test_support::ExportInspectionQueryReport {
        import_dir: "/tmp/raw".to_string(),
        summary: test_support::QueryReportSummary {
            dashboard_count: 1,
            panel_count: 1,
            query_count: 1,
            report_row_count: 1,
        },
        queries: vec![test_support::ExportInspectionQueryRow {
            org: "Main Org.".to_string(),
            org_id: "1".to_string(),
            dashboard_uid: "logs-main".to_string(),
            dashboard_title: "Logs Main".to_string(),
            dashboard_tags: Vec::new(),
            folder_path: "Logs".to_string(),
            folder_full_path: "/Logs".to_string(),
            folder_level: "1".to_string(),
            folder_uid: "logs".to_string(),
            parent_folder_uid: String::new(),
            panel_id: "11".to_string(),
            panel_title: "Errors".to_string(),
            panel_type: "logs".to_string(),
            panel_target_count: 0,
            panel_query_count: 0,
            panel_datasource_count: 0,
            panel_variables: Vec::new(),
            ref_id: "A".to_string(),
            datasource: "logs-main".to_string(),
            datasource_name: "logs-main".to_string(),
            datasource_uid: "logs-main".to_string(),
            datasource_org: String::new(),
            datasource_org_id: String::new(),
            datasource_database: String::new(),
            datasource_bucket: String::new(),
            datasource_organization: String::new(),
            datasource_index_pattern: String::new(),
            datasource_type: "loki".to_string(),
            datasource_family: "loki".to_string(),
            query_field: "expr".to_string(),
            target_hidden: "false".to_string(),
            target_disabled: "false".to_string(),
            query_text: "{job=\"grafana\"}".to_string(),
            query_variables: Vec::new(),
            metrics: Vec::new(),
            functions: vec!["count_over_time".to_string()],
            measurements: vec!["job=\"grafana\"".to_string()],
            buckets: vec!["5m".to_string()],
            file_path: "/tmp/raw/logs-main.json".to_string(),
        }],
    };

    let document = test_support::build_export_inspection_governance_document(&summary, &report);
    let lines = test_support::render_governance_table_report("/tmp/raw", &document);
    let output = lines.join("\n");

    assert!(output.contains("Export inspection governance: /tmp/raw"));
    assert!(output.contains("# Summary"));
    assert!(output.contains("# Datasource Families"));
    assert!(output.contains("# Dashboard Dependencies"));
    assert!(output.contains("# Dashboard Governance"));
    assert!(output.contains("# Dashboard Datasource Edges"));
    assert!(output.contains("# Datasource Governance"));
    assert!(output.contains("# Datasources"));
    assert!(output.contains("# Risks"));
    assert!(output.contains("DASHBOARD_UID"));
    assert!(output.contains("QUERY_FIELDS"));
    assert!(output.contains("DATASOURCE_COUNT"));
    assert!(output.contains("DATASOURCE_FAMILY_COUNT"));
    assert!(output.contains("DASHBOARD_DATASOURCE_EDGES"));
    assert!(output.contains("DATASOURCES_WITH_RISKS"));
    assert!(output.contains("DASHBOARDS_WITH_RISKS"));
    assert!(output.contains("METRICS"));
    assert!(output.contains("FUNCTIONS"));
    assert!(output.contains("MEASUREMENTS"));
    assert!(output.contains("BUCKETS"));
    assert!(output.contains("MIXED_DATASOURCE"));
    assert!(output.contains("RISK_KINDS"));
    assert!(output.contains("/tmp/raw/logs-main.json"));
    assert!(output.contains("CATEGORY"));
    assert!(output.contains("RECOMMENDATION"));
    assert!(output.contains("logs-main"));
    assert!(output.contains("unused-main"));
    assert!(output.contains("MIXED_DASHBOARDS"));
    assert!(output.contains("ORPHANED_DATASOURCES"));
    assert!(output.contains("orphaned-datasource"));
    assert!(output.contains("Remove the unused datasource"));
}

#[test]
fn build_export_inspection_governance_document_adds_dashboard_datasource_edges() {
    let summary = test_support::ExportInspectionSummary {
        import_dir: "/tmp/raw".to_string(),
        export_org: Some("Main Org.".to_string()),
        export_org_id: Some("1".to_string()),
        dashboard_count: 1,
        folder_count: 1,
        panel_count: 3,
        query_count: 3,
        datasource_inventory_count: 2,
        orphaned_datasource_count: 0,
        mixed_dashboard_count: 1,
        folder_paths: Vec::new(),
        datasource_usage: Vec::new(),
        datasource_inventory: vec![
            test_support::DatasourceInventorySummary {
                uid: "prom-main".to_string(),
                name: "Prometheus Main".to_string(),
                datasource_type: "prometheus".to_string(),
                access: "proxy".to_string(),
                url: "http://prometheus:9090".to_string(),
                is_default: "true".to_string(),
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
                reference_count: 2,
                dashboard_count: 1,
            },
            test_support::DatasourceInventorySummary {
                uid: "logs-main".to_string(),
                name: "Logs Main".to_string(),
                datasource_type: "loki".to_string(),
                access: "proxy".to_string(),
                url: "http://loki:3100".to_string(),
                is_default: "false".to_string(),
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
                reference_count: 1,
                dashboard_count: 1,
            },
        ],
        orphaned_datasources: Vec::new(),
        mixed_dashboards: vec![test_support::MixedDashboardSummary {
            uid: "core-main".to_string(),
            title: "Core Main".to_string(),
            folder_path: "Platform".to_string(),
            datasource_count: 2,
            datasources: vec!["prom-main".to_string(), "logs-main".to_string()],
        }],
    };

    let mut prom_a = make_core_family_report_row(
        "core-main",
        "7",
        "A",
        "prom-main",
        "Prometheus Main",
        "prometheus",
        "prometheus",
        "sum(rate(http_requests_total[5m]))",
        &["job=\"grafana\""],
    );
    prom_a.query_field = "expr".to_string();
    prom_a.metrics = vec!["http_requests_total".to_string()];
    prom_a.functions = vec!["rate".to_string(), "sum".to_string()];
    prom_a.measurements = vec!["job=\"grafana\"".to_string()];
    prom_a.buckets = vec!["5m".to_string()];

    let mut prom_b = make_core_family_report_row(
        "core-main",
        "8",
        "B",
        "prom-main",
        "Prometheus Main",
        "prometheus",
        "prometheus",
        "sum(rate(process_cpu_seconds_total[1h]))",
        &["service.name"],
    );
    prom_b.query_field = "query".to_string();
    prom_b.metrics = vec!["process_cpu_seconds_total".to_string()];
    prom_b.functions = vec!["rate".to_string(), "sum".to_string()];
    prom_b.measurements = vec!["service.name".to_string()];
    prom_b.buckets = vec!["1h".to_string()];

    let mut loki = make_core_family_report_row(
        "core-main",
        "9",
        "C",
        "logs-main",
        "Logs Main",
        "loki",
        "loki",
        "{job=\"grafana\"} |= \"error\"",
        &["job=\"grafana\""],
    );
    loki.query_field = "expr".to_string();
    loki.functions = vec!["line_filter_contains".to_string()];
    loki.measurements = vec!["job=\"grafana\"".to_string()];

    let report = test_support::ExportInspectionQueryReport {
        import_dir: "/tmp/raw".to_string(),
        summary: test_support::QueryReportSummary {
            dashboard_count: 1,
            panel_count: 3,
            query_count: 3,
            report_row_count: 3,
        },
        queries: vec![prom_a, prom_b, loki],
    };

    let document = test_support::build_export_inspection_governance_document(&summary, &report);
    let document_json = serde_json::to_value(&document).unwrap();
    let edges = document_json["dashboardDatasourceEdges"]
        .as_array()
        .unwrap();
    let datasource_governance = document_json["datasourceGovernance"].as_array().unwrap();

    assert_eq!(document.summary.dashboard_datasource_edge_count, 2);
    assert_eq!(document.summary.datasource_risk_coverage_count, 2);
    assert_eq!(document.summary.dashboard_risk_coverage_count, 1);
    assert_eq!(edges.len(), 2);
    assert_eq!(datasource_governance.len(), 2);
    let dashboard_governance = document_json["dashboardGovernance"].as_array().unwrap();
    assert_eq!(dashboard_governance.len(), 1);

    let prom_edge = edges
        .iter()
        .find(|row| row["datasourceUid"] == json!("prom-main"))
        .unwrap();
    assert_eq!(prom_edge["dashboardUid"], json!("core-main"));
    assert_eq!(prom_edge["dashboardTitle"], json!("core-main Dashboard"));
    assert_eq!(prom_edge["panelCount"], json!(2));
    assert_eq!(prom_edge["queryCount"], json!(2));
    assert_eq!(prom_edge["queryFields"], json!(["expr", "query"]));
    assert_eq!(
        prom_edge["metrics"],
        json!(["http_requests_total", "process_cpu_seconds_total"])
    );
    assert_eq!(prom_edge["functions"], json!(["rate", "sum"]));
    assert_eq!(
        prom_edge["measurements"],
        json!(["job=\"grafana\"", "service.name"])
    );
    assert_eq!(prom_edge["buckets"], json!(["1h", "5m"]));

    let loki_edge = edges
        .iter()
        .find(|row| row["datasourceUid"] == json!("logs-main"))
        .unwrap();
    assert_eq!(loki_edge["family"], json!("loki"));
    assert_eq!(loki_edge["panelCount"], json!(1));
    assert_eq!(loki_edge["queryCount"], json!(1));
    assert_eq!(loki_edge["functions"], json!(["line_filter_contains"]));

    let prom_governance = datasource_governance
        .iter()
        .find(|row| row["datasourceUid"] == json!("prom-main"))
        .unwrap();
    assert_eq!(prom_governance["mixedDashboardCount"], json!(1));
    assert_eq!(
        prom_governance["riskKinds"],
        json!(["mixed-datasource-dashboard"])
    );

    let loki_governance = datasource_governance
        .iter()
        .find(|row| row["datasourceUid"] == json!("logs-main"))
        .unwrap();
    assert_eq!(
        loki_governance["riskKinds"],
        json!(["mixed-datasource-dashboard"])
    );

    let dashboard_governance_row = &dashboard_governance[0];
    assert_eq!(dashboard_governance_row["dashboardUid"], json!("core-main"));
    assert_eq!(dashboard_governance_row["mixedDatasource"], json!(true));
    assert_eq!(
        dashboard_governance_row["riskKinds"],
        json!(["large-prometheus-range", "mixed-datasource-dashboard"])
    );
}

#[test]
fn validate_inspect_export_report_args_rejects_panel_filter_without_report() {
    let args = InspectExportArgs {
        import_dir: PathBuf::from("./dashboards/raw"),
        json: false,
        table: false,
        report: None,
        output_format: None,
        report_columns: Vec::new(),
        report_filter_datasource: None,
        report_filter_panel_id: Some("7".to_string()),
        help_full: false,
        no_header: false,
        output_file: None,
    };

    let error = test_support::validate_inspect_export_report_args(&args).unwrap_err();
    assert!(error
        .to_string()
        .contains("--report-filter-panel-id is only supported together with --report or report-like --output-format"));
}

#[test]
fn inspect_live_dashboards_with_request_matches_export_output_files_for_core_family_rows() {
    let temp = tempdir().unwrap();
    let export_root = temp.path().join("export");
    fs::create_dir_all(export_root.join("General")).unwrap();

    let folder_inventory = json!([
        {
            "uid": "general",
            "title": "General",
            "path": "General",
            "org": "Main Org.",
            "orgId": "1"
        }
    ]);
    let datasource_inventory = json!([
        {
            "uid": "prom-main",
            "name": "Prometheus Main",
            "type": "prometheus",
            "access": "proxy",
            "url": "http://prometheus:9090",
            "isDefault": "false",
            "org": "Main Org.",
            "orgId": "1"
        },
        {
            "uid": "loki-main",
            "name": "Loki Main",
            "type": "loki",
            "access": "proxy",
            "url": "http://loki:3100",
            "isDefault": "false",
            "org": "Main Org.",
            "orgId": "1"
        },
        {
            "uid": "influx-main",
            "name": "Influx Main",
            "type": "influxdb",
            "access": "proxy",
            "url": "http://influxdb:8086",
            "database": "metrics_v1",
            "defaultBucket": "prod-default",
            "organization": "acme-observability",
            "isDefault": "false",
            "org": "Main Org.",
            "orgId": "1"
        },
        {
            "uid": "sql-main",
            "name": "SQL Main",
            "type": "postgres",
            "access": "proxy",
            "url": "postgresql://postgres:5432/metrics",
            "database": "analytics",
            "isDefault": "false",
            "org": "Main Org.",
            "orgId": "1"
        },
        {
            "uid": "search-main",
            "name": "OpenSearch Main",
            "type": "grafana-opensearch-datasource",
            "access": "proxy",
            "url": "http://opensearch:9200",
            "indexPattern": "logs-*",
            "isDefault": "false",
            "org": "Main Org.",
            "orgId": "1"
        },
        {
            "uid": "trace-main",
            "name": "Tempo Main",
            "type": "tempo",
            "access": "proxy",
            "url": "http://tempo:3200",
            "isDefault": "false",
            "org": "Main Org.",
            "orgId": "1"
        }
    ]);
    let dashboard_payload = json!({
        "dashboard": {
            "id": 11,
            "uid": "core-main",
            "title": "Core Main",
            "panels": [
                {
                    "id": 7,
                    "title": "CPU Quantiles",
                    "type": "timeseries",
                    "datasource": {"uid": "prom-main", "type": "prometheus"},
                    "targets": [
                        {
                            "refId": "A",
                            "datasource": {"uid": "prom-main", "type": "prometheus"},
                            "expr": "histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket{job=\"api\",handler=\"/orders\"}[5m])) by (le))"
                        }
                    ]
                },
                {
                    "id": 11,
                    "title": "Pipeline Errors",
                    "type": "logs",
                    "datasource": {"uid": "loki-main", "type": "loki"},
                    "targets": [
                        {
                            "refId": "B",
                            "datasource": {"uid": "loki-main", "type": "loki"},
                            "expr": "sum by (namespace) (count_over_time({job=\"grafana\",namespace!=\"kube-system\",cluster=~\"prod|stage\"} |= \"timeout\" | json | logfmt [10m]))"
                        }
                    ]
                },
                {
                    "id": 9,
                    "title": "Requests",
                    "type": "timeseries",
                    "datasource": {"uid": "influx-main", "type": "influxdb"},
                    "targets": [
                        {
                            "refId": "C",
                            "datasource": {"uid": "influx-main", "type": "influxdb"},
                            "query": "from(bucket: \"prod\") |> range(start: -1h) |> filter(fn: (r) => r._measurement == \"cpu\" and r.host == \"web-01\") |> aggregateWindow(every: 5m, fn: mean) |> yield(name: \"mean\")"
                        }
                    ]
                },
                {
                    "id": 13,
                    "title": "Host Ownership",
                    "type": "table",
                    "datasource": {"uid": "sql-main", "type": "postgres"},
                    "targets": [
                        {
                            "refId": "D",
                            "datasource": {"uid": "sql-main", "type": "postgres"},
                            "rawSql": "WITH recent_cpu AS (SELECT * FROM public.cpu_metrics) SELECT recent_cpu.host, hosts.owner FROM recent_cpu JOIN \"public\".\"hosts\" ON hosts.host = recent_cpu.host WHERE hosts.owner IS NOT NULL ORDER BY hosts.owner LIMIT 10"
                        }
                    ]
                },
                {
                    "id": 17,
                    "title": "OpenSearch Hits",
                    "type": "table",
                    "datasource": {"uid": "search-main", "type": "grafana-opensearch-datasource"},
                    "targets": [
                        {
                            "refId": "E",
                            "datasource": {"uid": "search-main", "type": "grafana-opensearch-datasource"},
                            "query": "_exists_:@timestamp AND resource.service.name:\"checkout\" AND status:[500 TO 599]"
                        }
                    ]
                },
                {
                    "id": 19,
                    "title": "Trace Search",
                    "type": "table",
                    "datasource": {"uid": "trace-main", "type": "tempo"},
                    "targets": [
                        {
                            "refId": "F",
                            "datasource": {"uid": "trace-main", "type": "tempo"},
                            "query": "resource.service.name:checkout AND trace.id:abc123 AND span.name:\"GET /orders\""
                        }
                    ]
                }
            ]
        },
        "meta": {
            "folderUid": "general",
            "folderTitle": "General"
        }
    });
    fs::write(
        export_root.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid",
            "foldersFile": FOLDER_INVENTORY_FILENAME,
            "datasourcesFile": DATASOURCE_INVENTORY_FILENAME
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        export_root.join(FOLDER_INVENTORY_FILENAME),
        serde_json::to_string_pretty(&folder_inventory).unwrap(),
    )
    .unwrap();
    fs::write(
        export_root.join(DATASOURCE_INVENTORY_FILENAME),
        serde_json::to_string_pretty(&datasource_inventory).unwrap(),
    )
    .unwrap();
    fs::write(
        export_root.join("General").join("core.json"),
        serde_json::to_string_pretty(&dashboard_payload).unwrap(),
    )
    .unwrap();

    let export_report_output = temp.path().join("export-report.json");
    let export_report_args = InspectExportArgs {
        import_dir: export_root.clone(),
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::Json),
        output_format: None,
        report_columns: Vec::new(),
        report_filter_datasource: None,
        report_filter_panel_id: None,
        help_full: false,
        no_header: false,
        output_file: Some(export_report_output.clone()),
    };
    let export_report_count = test_support::analyze_export_dir(&export_report_args).unwrap();

    let live_report_output = temp.path().join("live-report.json");

    let args = InspectLiveArgs {
        common: make_common_args("https://grafana.example.com".to_string()),
        page_size: 100,
        concurrency: 1,
        org_id: None,
        all_orgs: false,
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::Json),
        output_format: None,
        report_columns: Vec::new(),
        report_filter_datasource: None,
        report_filter_panel_id: None,
        progress: false,
        help_full: false,
        no_header: false,
        output_file: Some(live_report_output.clone()),
        interactive: false,
    };

    let live_report_count = test_support::inspect_live_dashboards_with_request(
        core_family_inspect_live_request_fixture(
            datasource_inventory.clone(),
            dashboard_payload.clone(),
        ),
        &args,
    )
    .unwrap();

    let export_report_document = read_json_output_file(&export_report_output);
    let live_report_document = read_json_output_file(&live_report_output);
    assert_eq!(export_report_count, 1);
    assert_eq!(live_report_count, 1);
    assert_eq!(
        normalize_queries_document_for_compare(&export_report_document),
        normalize_queries_document_for_compare(&live_report_document)
    );
    assert_eq!(
        export_report_document["summary"]["dashboardCount"],
        Value::from(1)
    );
    assert_eq!(
        export_report_document["summary"]["queryRecordCount"],
        Value::from(6)
    );

    for ref_id in ["A", "B", "C", "D", "E", "F"] {
        assert_json_query_report_row_parity(&export_report_document, &live_report_document, ref_id);
    }

    let export_governance_output = temp.path().join("export-governance.json");
    let export_governance_args = InspectExportArgs {
        import_dir: export_root.clone(),
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::GovernanceJson),
        output_format: None,
        report_columns: Vec::new(),
        report_filter_datasource: None,
        report_filter_panel_id: None,
        help_full: false,
        no_header: false,
        output_file: Some(export_governance_output.clone()),
    };
    let export_governance_count =
        test_support::analyze_export_dir(&export_governance_args).unwrap();
    let export_governance_document = read_json_output_file(&export_governance_output);

    let live_governance_output = temp.path().join("live-governance.json");
    let live_governance_args = InspectLiveArgs {
        common: make_common_args("https://grafana.example.com".to_string()),
        page_size: 100,
        concurrency: 1,
        org_id: None,
        all_orgs: false,
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::GovernanceJson),
        output_format: None,
        report_columns: Vec::new(),
        report_filter_datasource: None,
        report_filter_panel_id: None,
        progress: false,
        help_full: false,
        no_header: false,
        output_file: Some(live_governance_output.clone()),
        interactive: false,
    };
    let live_governance_count = test_support::inspect_live_dashboards_with_request(
        core_family_inspect_live_request_fixture(
            datasource_inventory.clone(),
            dashboard_payload.clone(),
        ),
        &live_governance_args,
    )
    .unwrap();
    let live_governance_document = read_json_output_file(&live_governance_output);

    let export_dependency_output = temp.path().join("export-dependency.json");
    let export_dependency_args = InspectExportArgs {
        import_dir: export_root.clone(),
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::DependencyJson),
        output_format: None,
        report_columns: Vec::new(),
        report_filter_datasource: None,
        report_filter_panel_id: None,
        help_full: false,
        no_header: false,
        output_file: Some(export_dependency_output.clone()),
    };
    let export_dependency_count =
        test_support::analyze_export_dir(&export_dependency_args).unwrap();
    let export_dependency_document = read_json_output_file(&export_dependency_output);

    let live_dependency_output = temp.path().join("live-dependency.json");
    let live_dependency_args = InspectLiveArgs {
        common: make_common_args("https://grafana.example.com".to_string()),
        page_size: 100,
        concurrency: 1,
        org_id: None,
        all_orgs: false,
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::DependencyJson),
        output_format: None,
        report_columns: Vec::new(),
        report_filter_datasource: None,
        report_filter_panel_id: None,
        progress: false,
        help_full: false,
        no_header: false,
        output_file: Some(live_dependency_output.clone()),
        interactive: false,
    };
    let live_dependency_count = test_support::inspect_live_dashboards_with_request(
        core_family_inspect_live_request_fixture(
            datasource_inventory.clone(),
            dashboard_payload.clone(),
        ),
        &live_dependency_args,
    )
    .unwrap();
    let live_dependency_document = read_json_output_file(&live_dependency_output);

    assert_eq!(export_governance_count, 1);
    assert_eq!(live_governance_count, 1);
    assert_governance_documents_match(&export_governance_document, &live_governance_document);
    assert_eq!(
        export_governance_document["summary"]["dashboardCount"],
        Value::from(1)
    );
    assert_eq!(
        export_governance_document["summary"]["queryRecordCount"],
        Value::from(6)
    );
    assert_eq!(
        export_governance_document["summary"]["datasourceFamilyCount"],
        Value::from(6)
    );
    assert_eq!(
        export_governance_document["summary"]["riskRecordCount"],
        Value::from(1)
    );
    assert_eq!(
        export_governance_document["dashboardDependencies"][0]["datasourceFamilies"],
        json!(["prometheus", "loki", "flux", "sql", "search", "tracing"])
    );
    assert_eq!(
        export_governance_document["dashboardDependencies"][0]["datasourceCount"],
        Value::from(6)
    );
    assert_eq!(
        export_governance_document["dashboardDependencies"][0]["datasourceFamilyCount"],
        Value::from(6)
    );

    assert_eq!(export_dependency_count, 1);
    assert_eq!(live_dependency_count, 1);
    assert_eq!(
        normalize_queries_document_for_compare(&export_dependency_document),
        normalize_queries_document_for_compare(&live_dependency_document)
    );
    assert_eq!(
        export_dependency_document["kind"],
        Value::String("grafana-utils-dashboard-dependency-contract".to_string())
    );
    assert_eq!(
        export_dependency_document["summary"]["queryCount"],
        Value::from(6)
    );
    assert_eq!(
        export_dependency_document["summary"]["dashboardCount"],
        Value::from(1)
    );
    assert_eq!(
        export_dependency_document["summary"]["panelCount"],
        Value::from(6)
    );
    assert_eq!(
        export_dependency_document["summary"]["datasourceCount"],
        Value::from(6)
    );
    assert_eq!(
        export_dependency_document["summary"]["orphanedDatasourceCount"],
        Value::from(0)
    );
    assert_eq!(
        export_dependency_document["queries"]
            .as_array()
            .unwrap()
            .len(),
        6
    );
    assert_eq!(
        export_dependency_document["datasourceUsage"]
            .as_array()
            .unwrap()
            .len(),
        6
    );
    assert_eq!(
        export_dependency_document["orphanedDatasources"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    let dependency_row_a = export_dependency_document["queries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["refId"] == Value::String("A".to_string()))
        .unwrap();
    assert_eq!(
        dependency_row_a["dashboardUid"],
        Value::String("core-main".to_string())
    );
    assert_eq!(
        dependency_row_a["panelTitle"],
        Value::String("CPU Quantiles".to_string())
    );
    assert_eq!(
        dependency_row_a["datasourceUid"],
        Value::String("prom-main".to_string())
    );
    assert_eq!(
        dependency_row_a["datasourceFamily"],
        Value::String("prometheus".to_string())
    );
    assert_eq!(
        dependency_row_a["queryField"],
        Value::String("expr".to_string())
    );
    let dependency_analysis = dependency_row_a["analysis"].as_object().unwrap();
    for key in ["metrics", "measurements", "buckets", "labels"] {
        assert!(dependency_analysis.contains_key(key), "missing key {key}");
    }
    assert!(dependency_row_a["analysis"]["metrics"].is_array());
    assert!(dependency_row_a["analysis"]["measurements"].is_array());
    assert!(dependency_row_a["analysis"]["buckets"].is_array());
}

#[test]
fn inspect_live_dashboards_with_request_all_orgs_aggregates_multiple_org_exports() {
    let temp = tempdir().unwrap();
    let export_root = temp.path().join("exports");
    let org_one_raw = export_root.join("org_1_Main_Org").join("raw");
    let org_two_raw = export_root.join("org_2_Ops_Org").join("raw");
    fs::create_dir_all(&org_one_raw).unwrap();
    fs::create_dir_all(&org_two_raw).unwrap();
    fs::write(
        export_root.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "root",
            "dashboardCount": 2,
            "indexFile": "index.json",
            "orgCount": 2,
            "orgs": [
                {"org": "Main Org.", "orgId": "1", "dashboardCount": 1},
                {"org": "Ops Org", "orgId": "2", "dashboardCount": 1}
            ]
        }))
        .unwrap(),
    )
    .unwrap();

    for (
        raw_dir,
        org_id,
        org_name,
        folder_uid,
        folder_title,
        dashboard_uid,
        dashboard_title,
        datasource_uid,
        datasource_name,
        file_name,
        query_text,
    ) in [
        (
            &org_one_raw,
            "1",
            "Main Org.",
            "general",
            "General",
            "cpu-main",
            "CPU Main",
            "prom-main",
            "Prometheus Main",
            "CPU_Main__cpu-main.json",
            "up",
        ),
        (
            &org_two_raw,
            "2",
            "Ops Org",
            "ops",
            "Ops",
            "latency-main",
            "Latency Main",
            "prom-two",
            "Prometheus Two",
            "Latency_Main__latency-main.json",
            "rate(http_requests_total[5m])",
        ),
    ] {
        fs::write(
            raw_dir.join(EXPORT_METADATA_FILENAME),
            serde_json::to_string_pretty(&json!({
                "kind": "grafana-utils-dashboard-export-index",
                "schemaVersion": TOOL_SCHEMA_VERSION,
                "variant": "raw",
                "dashboardCount": 1,
                "indexFile": "index.json",
                "format": "grafana-web-import-preserve-uid",
                "foldersFile": FOLDER_INVENTORY_FILENAME,
                "datasourcesFile": DATASOURCE_INVENTORY_FILENAME,
                "org": org_name,
                "orgId": org_id
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join(FOLDER_INVENTORY_FILENAME),
            serde_json::to_string_pretty(&json!([
                {
                    "uid": folder_uid,
                    "title": folder_title,
                    "path": folder_title,
                    "org": org_name,
                    "orgId": org_id
                }
            ]))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join(DATASOURCE_INVENTORY_FILENAME),
            serde_json::to_string_pretty(&json!([
                {
                    "uid": datasource_uid,
                    "name": datasource_name,
                    "type": "prometheus",
                    "access": "proxy",
                    "url": if org_id == "1" {
                        "http://prometheus:9090"
                    } else {
                        "http://prometheus-two:9090"
                    },
                    "isDefault": "true",
                    "org": org_name,
                    "orgId": org_id
                }
            ]))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join("index.json"),
            serde_json::to_string_pretty(&json!([
                {
                    "uid": dashboard_uid,
                    "title": dashboard_title,
                    "path": format!("{folder_title}/{file_name}"),
                    "format": "grafana-web-import-preserve-uid",
                    "org": org_name,
                    "orgId": org_id
                }
            ]))
            .unwrap(),
        )
        .unwrap();
        fs::create_dir_all(raw_dir.join(folder_title)).unwrap();
        fs::write(
            raw_dir.join(folder_title).join(file_name),
            serde_json::to_string_pretty(&json!({
                "dashboard": {
                    "id": if org_id == "1" { 11 } else { 12 },
                    "uid": dashboard_uid,
                    "title": dashboard_title,
                    "panels": [{
                        "id": if org_id == "1" { 7 } else { 8 },
                        "title": if org_id == "1" { "CPU Query" } else { "Latency Query" },
                        "type": "timeseries",
                        "datasource": {"uid": datasource_uid, "type": "prometheus"},
                        "targets": [{
                            "refId": "A",
                            "expr": query_text
                        }]
                    }]
                },
                "meta": {"folderUid": folder_uid, "folderTitle": folder_title}
            }))
            .unwrap(),
        )
        .unwrap();
    }

    let export_report_output = temp.path().join("export-report.json");
    let export_report_args = InspectExportArgs {
        import_dir: export_root.clone(),
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::Json),
        output_format: None,
        report_columns: Vec::new(),
        report_filter_datasource: None,
        report_filter_panel_id: None,
        help_full: false,
        no_header: false,
        output_file: Some(export_report_output.clone()),
    };
    let export_report_count = test_support::analyze_export_dir(&export_report_args).unwrap();
    let export_report_document = read_json_output_file(&export_report_output);

    let live_report_output = temp.path().join("live-report.json");
    let live_report_args = InspectLiveArgs {
        common: make_common_args("https://grafana.example.com".to_string()),
        page_size: 100,
        concurrency: 1,
        org_id: None,
        all_orgs: true,
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::Json),
        output_format: None,
        report_columns: Vec::new(),
        report_filter_datasource: None,
        report_filter_panel_id: None,
        progress: false,
        help_full: false,
        no_header: false,
        output_file: Some(live_report_output.clone()),
        interactive: false,
    };
    let live_report_count = test_support::inspect_live_dashboards_with_request(
        all_orgs_inspect_live_request_fixture(),
        &live_report_args,
    )
    .unwrap();
    let live_report_document = read_json_output_file(&live_report_output);

    let export_governance_output = temp.path().join("export-governance.json");
    let export_governance_args = InspectExportArgs {
        import_dir: export_root.clone(),
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::GovernanceJson),
        output_format: None,
        report_columns: Vec::new(),
        report_filter_datasource: None,
        report_filter_panel_id: None,
        help_full: false,
        no_header: false,
        output_file: Some(export_governance_output.clone()),
    };
    let export_governance_count =
        test_support::analyze_export_dir(&export_governance_args).unwrap();
    let export_governance_document = read_json_output_file(&export_governance_output);

    let live_governance_output = temp.path().join("live-governance.json");
    let live_governance_args = InspectLiveArgs {
        common: make_common_args("https://grafana.example.com".to_string()),
        page_size: 100,
        concurrency: 1,
        org_id: None,
        all_orgs: true,
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::GovernanceJson),
        output_format: None,
        report_columns: Vec::new(),
        report_filter_datasource: None,
        report_filter_panel_id: None,
        progress: false,
        help_full: false,
        no_header: false,
        output_file: Some(live_governance_output.clone()),
        interactive: false,
    };
    let live_governance_count = test_support::inspect_live_dashboards_with_request(
        all_orgs_inspect_live_request_fixture(),
        &live_governance_args,
    )
    .unwrap();
    let live_governance_document = read_json_output_file(&live_governance_output);

    let export_dependency_output = temp.path().join("export-dependency.json");
    let export_dependency_args = InspectExportArgs {
        import_dir: export_root.clone(),
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::DependencyJson),
        output_format: None,
        report_columns: Vec::new(),
        report_filter_datasource: None,
        report_filter_panel_id: None,
        help_full: false,
        no_header: false,
        output_file: Some(export_dependency_output.clone()),
    };
    let export_dependency_count =
        test_support::analyze_export_dir(&export_dependency_args).unwrap();
    let export_dependency_document = read_json_output_file(&export_dependency_output);

    let live_dependency_output = temp.path().join("live-dependency.json");
    let live_dependency_args = InspectLiveArgs {
        common: make_common_args("https://grafana.example.com".to_string()),
        page_size: 100,
        concurrency: 1,
        org_id: None,
        all_orgs: true,
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::DependencyJson),
        output_format: None,
        report_columns: Vec::new(),
        report_filter_datasource: None,
        report_filter_panel_id: None,
        progress: false,
        help_full: false,
        no_header: false,
        output_file: Some(live_dependency_output.clone()),
        interactive: false,
    };
    let live_dependency_count = test_support::inspect_live_dashboards_with_request(
        all_orgs_inspect_live_request_fixture(),
        &live_dependency_args,
    )
    .unwrap();
    let live_dependency_document = read_json_output_file(&live_dependency_output);

    assert_eq!(export_report_count, 2);
    assert_eq!(live_report_count, 2);
    assert_eq!(export_governance_count, 2);
    assert_eq!(live_governance_count, 2);
    assert_eq!(export_dependency_count, 2);
    assert_eq!(live_dependency_count, 2);

    assert_all_orgs_export_live_documents_match(
        &export_report_document,
        &live_report_document,
        &export_dependency_document,
        &live_dependency_document,
        &export_governance_document,
        &live_governance_document,
    );
    assert_eq!(
        export_report_document["summary"]["dashboardCount"],
        Value::from(2)
    );
    assert_eq!(
        export_report_document["summary"]["queryRecordCount"],
        Value::from(2)
    );
    assert_eq!(
        live_report_document["summary"]["dashboardCount"],
        Value::from(2)
    );
    assert_eq!(
        live_report_document["summary"]["queryRecordCount"],
        Value::from(2)
    );
    assert_eq!(
        export_dependency_document["summary"]["queryCount"],
        Value::from(2)
    );
    assert_eq!(
        export_dependency_document["summary"]["dashboardCount"],
        Value::from(2)
    );
    assert_eq!(
        export_dependency_document["summary"]["panelCount"],
        Value::from(2)
    );
    assert_eq!(
        export_dependency_document["summary"]["datasourceCount"],
        Value::from(2)
    );
    assert_eq!(
        export_dependency_document["summary"]["orphanedDatasourceCount"],
        Value::from(0)
    );
    assert_eq!(
        export_governance_document["summary"]["dashboardCount"],
        Value::from(2)
    );
    assert_eq!(
        export_governance_document["summary"]["queryRecordCount"],
        Value::from(2)
    );
    assert_eq!(
        live_governance_document["summary"]["dashboardCount"],
        Value::from(2)
    );
    assert_eq!(
        live_governance_document["summary"]["queryRecordCount"],
        Value::from(2)
    );
    assert_eq!(
        export_governance_document["summary"]["datasourceFamilyCount"],
        Value::from(1)
    );
    assert_eq!(
        export_governance_document["summary"]["riskRecordCount"],
        Value::from(1)
    );
    assert_eq!(
        live_governance_document["summary"]["datasourceFamilyCount"],
        Value::from(1)
    );
    assert_eq!(
        live_governance_document["summary"]["riskRecordCount"],
        Value::from(1)
    );
    assert_eq!(
        export_governance_document["dashboardDependencies"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        export_governance_document["dashboardDependencies"][0]["datasourceFamilies"],
        json!(["prometheus"])
    );
    assert_eq!(
        export_dependency_document["queries"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        export_dependency_document["datasourceUsage"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        export_dependency_document["kind"],
        Value::String("grafana-utils-dashboard-dependency-contract".to_string())
    );
    assert_eq!(
        export_dependency_document["summary"]["queryCount"],
        Value::from(2)
    );
    assert_eq!(
        export_dependency_document["summary"]["dashboardCount"],
        Value::from(2)
    );
    assert_eq!(
        export_dependency_document["summary"]["datasourceCount"],
        Value::from(2)
    );
    assert_eq!(
        export_dependency_document["summary"]["orphanedDatasourceCount"],
        Value::from(0)
    );
    assert_eq!(
        export_dependency_document["orphanedDatasources"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    let dependency_rows = export_dependency_document["queries"].as_array().unwrap();
    let cpu_row = dependency_rows
        .iter()
        .find(|row| row["dashboardUid"] == Value::String("cpu-main".to_string()))
        .unwrap();
    assert_eq!(
        cpu_row["panelTitle"],
        Value::String("CPU Query".to_string())
    );
    assert_eq!(
        cpu_row["datasourceUid"],
        Value::String("prom-main".to_string())
    );
    assert_eq!(
        cpu_row["datasourceFamily"],
        Value::String("prometheus".to_string())
    );
    assert_eq!(cpu_row["queryField"], Value::String("expr".to_string()));
    assert!(cpu_row["analysis"]["metrics"].is_array());
    let latency_row = dependency_rows
        .iter()
        .find(|row| row["dashboardUid"] == Value::String("latency-main".to_string()))
        .unwrap();
    assert_eq!(
        latency_row["panelTitle"],
        Value::String("Latency Query".to_string())
    );
    assert_eq!(
        latency_row["datasourceUid"],
        Value::String("prom-two".to_string())
    );
    assert_eq!(
        latency_row["datasourceFamily"],
        Value::String("prometheus".to_string())
    );
    assert_eq!(latency_row["queryField"], Value::String("expr".to_string()));
    assert!(latency_row["analysis"]["metrics"].is_array());
    assert_eq!(
        export_dependency_document["datasourceUsage"][0]["queryFields"],
        json!(["expr"])
    );
    assert_eq!(
        export_dependency_document["datasourceUsage"][1]["queryFields"],
        json!(["expr"])
    );
}

#[test]
fn inspect_live_dashboards_with_request_all_orgs_matches_export_root_governance_contract() {
    let temp = tempdir().unwrap();
    let export_dir = temp.path().join("dashboards");
    let export_args = ExportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        export_dir: export_dir.clone(),
        page_size: 500,
        org_id: None,
        all_orgs: true,
        flat: false,
        overwrite: true,
        without_dashboard_raw: false,
        without_dashboard_prompt: true,
        dry_run: false,
        progress: false,
        verbose: false,
    };
    let inspect_root_temp = tempdir().unwrap();

    let mut request_fixture = |method: reqwest::Method,
                               path: &str,
                               params: &[(String, String)],
                               _payload: Option<&Value>|
     -> crate::common::Result<Option<Value>> {
        let scoped_org = params
            .iter()
            .find(|(key, _)| key == "orgId")
            .map(|(_, value)| value.as_str());
        match (method, path, scoped_org) {
            (reqwest::Method::GET, "/api/orgs", _) => Ok(Some(json!([
                {"id": 1, "name": "Main Org."},
                {"id": 2, "name": "Ops Org"}
            ]))),
            (reqwest::Method::GET, "/api/org", Some("1")) => {
                Ok(Some(json!({"id": 1, "name": "Main Org."})))
            }
            (reqwest::Method::GET, "/api/org", Some("2")) => {
                Ok(Some(json!({"id": 2, "name": "Ops Org"})))
            }
            (reqwest::Method::GET, "/api/search", Some("1")) => Ok(Some(json!([
                {
                    "uid": "cpu-main",
                    "title": "CPU Main",
                    "type": "dash-db",
                    "folderUid": "general",
                    "folderTitle": "General"
                }
            ]))),
            (reqwest::Method::GET, "/api/search", Some("2")) => Ok(Some(json!([
                {
                    "uid": "logs-main",
                    "title": "Logs Main",
                    "type": "dash-db",
                    "folderUid": "ops",
                    "folderTitle": "Ops"
                }
            ]))),
            (reqwest::Method::GET, "/api/datasources", Some("1")) => Ok(Some(json!([
                {
                    "uid": "prom-main",
                    "name": "Prometheus Main",
                    "type": "prometheus",
                    "access": "proxy",
                    "url": "http://prometheus:9090",
                    "isDefault": true
                }
            ]))),
            (reqwest::Method::GET, "/api/datasources", Some("2")) => Ok(Some(json!([
                {
                    "uid": "logs-main",
                    "name": "Logs Main",
                    "type": "loki",
                    "access": "proxy",
                    "url": "http://loki:3100",
                    "isDefault": true
                }
            ]))),
            (reqwest::Method::GET, "/api/dashboards/uid/cpu-main", Some("1")) => Ok(Some(json!({
                "dashboard": {
                    "id": 11,
                    "uid": "cpu-main",
                    "title": "CPU Main",
                    "panels": [{
                        "id": 7,
                        "title": "CPU Query",
                        "type": "timeseries",
                        "datasource": {"uid": "prom-main", "type": "prometheus"},
                        "targets": [{"refId": "A", "expr": "up"}]
                    }]
                },
                "meta": {"folderUid": "general", "folderTitle": "General"}
            }))),
            (reqwest::Method::GET, "/api/dashboards/uid/logs-main", Some("2")) => Ok(Some(json!({
                "dashboard": {
                    "id": 12,
                    "uid": "logs-main",
                    "title": "Logs Main",
                    "panels": [{
                        "id": 8,
                        "title": "Logs Query",
                        "type": "logs",
                        "datasource": {"uid": "logs-main", "type": "loki"},
                        "targets": [{"refId": "A", "expr": "{job=\"grafana\"} |= \"error\" | json [5m]"}]
                    }]
                },
                "meta": {"folderUid": "ops", "folderTitle": "Ops"}
            }))),
            (reqwest::Method::GET, "/api/dashboards/uid/cpu-main/permissions", Some("1")) => {
                Ok(Some(json!([])))
            }
            (reqwest::Method::GET, "/api/dashboards/uid/logs-main/permissions", Some("2")) => {
                Ok(Some(json!([])))
            }
            (reqwest::Method::GET, "/api/folders/general", _) => {
                Ok(Some(json!({"uid": "general", "title": "General"})))
            }
            (reqwest::Method::GET, "/api/folders/general/permissions", _) => Ok(Some(json!([]))),
            (reqwest::Method::GET, "/api/folders/ops", _) => {
                Ok(Some(json!({"uid": "ops", "title": "Ops"})))
            }
            (reqwest::Method::GET, "/api/folders/ops/permissions", _) => Ok(Some(json!([]))),
            (method, path, _) => panic!("unexpected request {method} {path}"),
        }
    };

    let export_count =
        test_support::export_dashboards_with_request(&mut request_fixture, &export_args).unwrap();
    assert_eq!(export_count, 2);

    let export_import_dir =
        test_support::prepare_inspect_export_import_dir(inspect_root_temp.path(), &export_dir)
            .unwrap();
    let export_governance_output = temp.path().join("export-governance.json");
    let export_governance_args = InspectExportArgs {
        import_dir: export_import_dir.clone(),
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::GovernanceJson),
        output_format: None,
        report_columns: Vec::new(),
        report_filter_datasource: None,
        report_filter_panel_id: None,
        help_full: false,
        no_header: false,
        output_file: Some(export_governance_output.clone()),
    };
    let export_governance_count =
        test_support::analyze_export_dir(&export_governance_args).unwrap();
    let export_governance_document = read_json_output_file(&export_governance_output);

    let live_governance_output = temp.path().join("live-governance.json");
    let live_governance_args = InspectLiveArgs {
        common: make_common_args("https://grafana.example.com".to_string()),
        page_size: 100,
        concurrency: 1,
        org_id: None,
        all_orgs: true,
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::GovernanceJson),
        output_format: None,
        report_columns: Vec::new(),
        report_filter_datasource: None,
        report_filter_panel_id: None,
        progress: false,
        help_full: false,
        no_header: false,
        output_file: Some(live_governance_output.clone()),
        interactive: false,
    };
    let live_governance_count = test_support::inspect_live_dashboards_with_request(
        &mut request_fixture,
        &live_governance_args,
    )
    .unwrap();
    let live_governance_document = read_json_output_file(&live_governance_output);

    let export_report_output = temp.path().join("export-report.json");
    let export_report_args = InspectExportArgs {
        import_dir: export_import_dir.clone(),
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::Json),
        output_format: None,
        report_columns: Vec::new(),
        report_filter_datasource: None,
        report_filter_panel_id: None,
        help_full: false,
        no_header: false,
        output_file: Some(export_report_output.clone()),
    };
    let export_report_count = test_support::analyze_export_dir(&export_report_args).unwrap();
    let export_report_document = read_json_output_file(&export_report_output);

    let live_report_output = temp.path().join("live-report.json");
    let live_report_args = InspectLiveArgs {
        common: make_common_args("https://grafana.example.com".to_string()),
        page_size: 100,
        concurrency: 1,
        org_id: None,
        all_orgs: true,
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::Json),
        output_format: None,
        report_columns: Vec::new(),
        report_filter_datasource: None,
        report_filter_panel_id: None,
        progress: false,
        help_full: false,
        no_header: false,
        output_file: Some(live_report_output.clone()),
        interactive: false,
    };
    let live_report_count =
        test_support::inspect_live_dashboards_with_request(&mut request_fixture, &live_report_args)
            .unwrap();
    let live_report_document = read_json_output_file(&live_report_output);

    let export_dependency_output = temp.path().join("export-dependency.json");
    let export_dependency_args = InspectExportArgs {
        import_dir: export_import_dir.clone(),
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::DependencyJson),
        output_format: None,
        report_columns: Vec::new(),
        report_filter_datasource: None,
        report_filter_panel_id: None,
        help_full: false,
        no_header: false,
        output_file: Some(export_dependency_output.clone()),
    };
    let export_dependency_count =
        test_support::analyze_export_dir(&export_dependency_args).unwrap();
    let export_dependency_document = read_json_output_file(&export_dependency_output);

    let live_dependency_output = temp.path().join("live-dependency.json");
    let live_dependency_args = InspectLiveArgs {
        common: make_common_args("https://grafana.example.com".to_string()),
        page_size: 100,
        concurrency: 1,
        org_id: None,
        all_orgs: true,
        json: false,
        table: false,
        report: Some(InspectExportReportFormat::DependencyJson),
        output_format: None,
        report_columns: Vec::new(),
        report_filter_datasource: None,
        report_filter_panel_id: None,
        progress: false,
        help_full: false,
        no_header: false,
        output_file: Some(live_dependency_output.clone()),
        interactive: false,
    };
    let live_dependency_count = test_support::inspect_live_dashboards_with_request(
        &mut request_fixture,
        &live_dependency_args,
    )
    .unwrap();
    let live_dependency_document = read_json_output_file(&live_dependency_output);

    assert_eq!(export_governance_count, 2);
    assert_eq!(live_governance_count, 2);
    assert_eq!(export_report_count, 2);
    assert_eq!(live_report_count, 2);
    assert_eq!(export_dependency_count, 2);
    assert_eq!(live_dependency_count, 2);
    assert_governance_documents_match(&export_governance_document, &live_governance_document);
    assert_all_orgs_export_live_documents_match(
        &export_report_document,
        &live_report_document,
        &export_dependency_document,
        &live_dependency_document,
        &export_governance_document,
        &live_governance_document,
    );
    assert_eq!(
        export_governance_document["summary"]["dashboardCount"],
        Value::from(2)
    );
    assert_eq!(
        export_governance_document["summary"]["queryRecordCount"],
        Value::from(2)
    );
    assert_eq!(
        export_report_document["summary"]["dashboardCount"],
        Value::from(2)
    );
    assert_eq!(
        export_report_document["summary"]["queryRecordCount"],
        Value::from(2)
    );
    assert_eq!(
        live_report_document["summary"]["dashboardCount"],
        Value::from(2)
    );
    assert_eq!(
        live_report_document["summary"]["queryRecordCount"],
        Value::from(2)
    );
    assert_eq!(
        export_governance_document["summary"]["datasourceFamilyCount"],
        Value::from(2)
    );
    assert_eq!(
        export_governance_document["summary"]["datasourceCoverageCount"],
        Value::from(2)
    );
    assert_eq!(
        export_governance_document["summary"]["riskRecordCount"],
        Value::from(1)
    );
    assert_eq!(
        live_governance_document["summary"]["datasourceFamilyCount"],
        Value::from(2)
    );
    assert_eq!(
        live_governance_document["summary"]["riskRecordCount"],
        Value::from(1)
    );
    assert_eq!(
        export_dependency_document["summary"]["queryCount"],
        Value::from(2)
    );
    assert_eq!(
        export_dependency_document["summary"]["dashboardCount"],
        Value::from(2)
    );
    assert_eq!(
        export_dependency_document["summary"]["datasourceCount"],
        Value::from(2)
    );
    assert_eq!(
        export_dependency_document["summary"]["orphanedDatasourceCount"],
        Value::from(0)
    );

    let datasource_families = export_governance_document["datasourceFamilies"]
        .as_array()
        .unwrap();
    assert_eq!(datasource_families.len(), 2);
    let prometheus_family = datasource_families
        .iter()
        .find(|row| row["family"] == Value::String("prometheus".to_string()))
        .unwrap();
    assert_eq!(prometheus_family["datasourceTypes"], json!(["prometheus"]));
    assert_eq!(prometheus_family["dashboardCount"], Value::from(1));
    assert_eq!(prometheus_family["panelCount"], Value::from(1));
    assert_eq!(prometheus_family["queryCount"], Value::from(1));

    let loki_family = datasource_families
        .iter()
        .find(|row| row["family"] == Value::String("loki".to_string()))
        .unwrap();
    assert_eq!(loki_family["datasourceTypes"], json!(["loki"]));
    assert_eq!(loki_family["dashboardCount"], Value::from(1));
    assert_eq!(loki_family["panelCount"], Value::from(1));
    assert_eq!(loki_family["queryCount"], Value::from(1));

    let dashboard_dependencies = export_governance_document["dashboardDependencies"]
        .as_array()
        .unwrap();
    assert_eq!(dashboard_dependencies.len(), 2);
    let cpu_dependency = dashboard_dependencies
        .iter()
        .find(|row| row["dashboardUid"] == Value::String("cpu-main".to_string()))
        .unwrap();
    assert_eq!(cpu_dependency["datasourceFamilies"], json!(["prometheus"]));
    assert_eq!(cpu_dependency["panelCount"], Value::from(1));
    assert_eq!(cpu_dependency["queryCount"], Value::from(1));
    let logs_dependency = dashboard_dependencies
        .iter()
        .find(|row| row["dashboardUid"] == Value::String("logs-main".to_string()))
        .unwrap();
    assert_eq!(logs_dependency["datasourceFamilies"], json!(["loki"]));
    assert_eq!(logs_dependency["panelCount"], Value::from(1));
    assert_eq!(logs_dependency["queryCount"], Value::from(1));
}

#[test]
fn import_dashboards_with_client_imports_discovered_files() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("permissions.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-permission-export",
            "schemaVersion": 1,
            "items": []
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": 7, "uid": "abc", "title": "CPU", "schemaVersion": 38},
            "meta": {"folderUid": "old-folder"}
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: Some("new-folder".to_string()),
        ensure_folders: false,
        replace_existing: true,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: false,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };
    let mut posted_payloads = Vec::new();
    let count = import_dashboards_with_request(
        with_dashboard_import_live_preflight(
            json!([]),
            json!([]),
            |_method, path, _params, payload| {
                assert_eq!(path, "/api/dashboards/db");
                posted_payloads.push(payload.cloned().unwrap());
                Ok(Some(json!({"status": "success"})))
            },
        ),
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert_eq!(posted_payloads.len(), 1);
    assert_eq!(posted_payloads[0]["folderUid"], "new-folder");
    assert_eq!(posted_payloads[0]["dashboard"]["id"], Value::Null);
}

#[test]
fn import_dashboards_with_org_id_requires_basic_auth() {
    let temp = tempdir().unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: Some(7),
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: temp.path().join("raw"),
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: false,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: true,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };

    let error = import_dashboards_with_org_clients(&args).unwrap_err();

    assert!(error
        .to_string()
        .contains("Dashboard import with --org-id requires Basic auth"));
}

#[test]
fn import_dashboards_with_use_export_org_requires_basic_auth() {
    let temp = tempdir().unwrap();
    let mut args = make_import_args(temp.path().join("exports"));
    args.use_export_org = true;

    let error = import_dashboards_with_org_clients(&args).unwrap_err();

    assert!(error
        .to_string()
        .contains("Dashboard import with --use-export-org requires Basic auth"));
}

#[test]
fn import_dashboards_with_create_missing_orgs_during_dry_run_previews_org_creation() {
    let temp = tempdir().unwrap();
    let export_root = temp.path().join("exports");
    let org_nine_raw = export_root.join("org_9_Ops_Org").join("raw");
    fs::create_dir_all(&org_nine_raw).unwrap();
    fs::write(
        org_nine_raw.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        org_nine_raw.join("index.json"),
        serde_json::to_string_pretty(&json!([
            {
                "uid": "ops",
                "title": "Ops",
                "path": "ops.json",
                "format": "grafana-web-import-preserve-uid",
                "org": "Ops Org",
                "orgId": "9"
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        org_nine_raw.join("ops.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": null, "uid": "ops", "title": "Ops"}
        }))
        .unwrap(),
    )
    .unwrap();

    let mut args = make_import_args(export_root);
    args.common = make_basic_common_args("http://127.0.0.1:3000".to_string());
    args.use_export_org = true;
    args.create_missing_orgs = true;
    args.dry_run = true;

    let mut admin_calls = Vec::new();
    let mut import_calls = Vec::new();
    let count = test_support::import::import_dashboards_by_export_org_with_request(
        |method, path, _params, _payload| {
            admin_calls.push((method.to_string(), path.to_string()));
            match (method, path) {
                (reqwest::Method::GET, "/api/orgs") => Ok(Some(json!([]))),
                _ => Err(test_support::message(format!("unexpected request {path}"))),
            }
        },
        |target_org_id, scoped_args| {
            import_calls.push((target_org_id, scoped_args.import_dir.clone()));
            Ok(0)
        },
        |_target_org_id, scoped_args| {
            Ok(test_support::import::ImportDryRunReport {
                mode: "create-only".to_string(),
                import_dir: scoped_args.import_dir.clone(),
                folder_statuses: Vec::new(),
                dashboard_records: Vec::new(),
                skipped_missing_count: 0,
                skipped_folder_mismatch_count: 0,
            })
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 0);
    assert_eq!(
        admin_calls,
        vec![("GET".to_string(), "/api/orgs".to_string())]
    );
    assert!(import_calls.is_empty());
}

#[test]
fn routed_import_create_missing_orgs_dry_run_and_live_created_scope_stay_aligned() {
    let temp = tempdir().unwrap();
    let export_root = temp.path().join("exports");
    let org_nine_raw = export_root.join("org_9_Ops_Org").join("raw");
    write_combined_export_root_metadata(&export_root, &[("9", "Ops Org", "org_9_Ops_Org")]);
    write_basic_raw_export(
        &org_nine_raw,
        "9",
        "Ops Org",
        "ops-main",
        "Ops Main",
        "loki-nine",
        "loki",
        "logs",
        "ops",
        "Ops",
        "expr",
        "{job=\"grafana\"}",
    );

    let mut dry_run_args = make_import_args(export_root.clone());
    dry_run_args.common = make_basic_common_args("http://127.0.0.1:3000".to_string());
    dry_run_args.use_export_org = true;
    dry_run_args.create_missing_orgs = true;
    dry_run_args.dry_run = true;
    dry_run_args.json = true;

    let dry_run_payload: Value = serde_json::from_str(
        &test_support::import::build_routed_import_dry_run_json_with_request(
            |method, path, _params, _payload| match (method, path) {
                (reqwest::Method::GET, "/api/orgs") => Ok(Some(json!([]))),
                _ => Err(test_support::message(format!("unexpected request {path}"))),
            },
            |_target_org_id, scoped_args| {
                Ok(test_support::import::ImportDryRunReport {
                    mode: "create-only".to_string(),
                    import_dir: scoped_args.import_dir.clone(),
                    folder_statuses: Vec::new(),
                    dashboard_records: Vec::new(),
                    skipped_missing_count: 0,
                    skipped_folder_mismatch_count: 0,
                })
            },
            &dry_run_args,
        )
        .unwrap(),
    )
    .unwrap();

    let dry_run_org = dry_run_payload["orgs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(9))
        .unwrap()
        .clone();
    let dry_run_import = dry_run_payload["imports"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(9))
        .unwrap()
        .clone();

    let mut live_args = make_import_args(export_root);
    live_args.common = make_basic_common_args("http://127.0.0.1:3000".to_string());
    live_args.use_export_org = true;
    live_args.create_missing_orgs = true;
    live_args.dry_run = false;

    let mut admin_calls = Vec::new();
    let mut import_calls = Vec::new();
    let count = test_support::import::import_dashboards_by_export_org_with_request(
        |method, path, _params, payload| {
            admin_calls.push((method.to_string(), path.to_string()));
            match (method, path) {
                (reqwest::Method::GET, "/api/orgs") => Ok(Some(json!([]))),
                (reqwest::Method::POST, "/api/orgs") => {
                    assert_eq!(
                        payload
                            .and_then(|value| value.as_object())
                            .unwrap()
                            .get("name"),
                        Some(&json!("Ops Org"))
                    );
                    Ok(Some(json!({"orgId": "19"})))
                }
                _ => Err(test_support::message(format!("unexpected request {path}"))),
            }
        },
        |target_org_id, scoped_args| {
            import_calls.push((
                target_org_id,
                scoped_args.import_dir.clone(),
                scoped_args.org_id,
            ));
            Ok(1)
        },
        |_target_org_id, _scoped_args| unreachable!("dry-run collector should not be used"),
        &live_args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert_eq!(
        admin_calls,
        vec![
            ("GET".to_string(), "/api/orgs".to_string()),
            ("POST".to_string(), "/api/orgs".to_string()),
        ]
    );
    assert_eq!(import_calls, vec![(19, org_nine_raw.clone(), Some(19))]);

    assert_eq!(dry_run_org["orgAction"], json!("would-create"));
    assert_eq!(dry_run_org["targetOrgId"], Value::Null);
    assert_eq!(dry_run_import["orgAction"], json!("would-create"));
    assert_eq!(dry_run_import["targetOrgId"], Value::Null);
    assert_eq!(dry_run_import["dashboards"], json!([]));
    assert_eq!(dry_run_import["summary"]["dashboardCount"], json!(1));

    let would_create_summary = test_support::import::format_routed_import_scope_summary_fields(
        9,
        "Ops Org",
        "would-create",
        None,
        Path::new(dry_run_org["importDir"].as_str().unwrap()),
    );
    let created_summary = test_support::import::format_routed_import_scope_summary_fields(
        9,
        "Ops Org",
        "created",
        Some(19),
        &org_nine_raw,
    );
    assert!(would_create_summary.contains("export orgId=9"));
    assert!(would_create_summary.contains("orgAction=would-create"));
    assert!(would_create_summary.contains("targetOrgId=<new>"));
    assert!(would_create_summary.contains(&org_nine_raw.display().to_string()));
    assert!(created_summary.contains("export orgId=9"));
    assert!(created_summary.contains("orgAction=created"));
    assert!(created_summary.contains("targetOrgId=19"));
    assert!(created_summary.contains(&org_nine_raw.display().to_string()));
}

#[test]
fn routed_import_status_matrix_covers_exists_missing_would_create_and_created() {
    let temp = tempdir().unwrap();
    let export_root = temp.path().join("exports");
    let org_two_raw = export_root.join("org_2_Org_Two").join("raw");
    let org_nine_raw = export_root.join("org_9_Ops_Org").join("raw");
    write_combined_export_root_metadata(
        &export_root,
        &[
            ("2", "Org Two", "org_2_Org_Two"),
            ("9", "Ops Org", "org_9_Ops_Org"),
        ],
    );
    write_basic_raw_export(
        &org_two_raw,
        "2",
        "Org Two",
        "cpu-two",
        "CPU Two",
        "prom-two",
        "prometheus",
        "timeseries",
        "general",
        "General",
        "expr",
        "up",
    );
    write_basic_raw_export(
        &org_nine_raw,
        "9",
        "Ops Org",
        "ops-main",
        "Ops Main",
        "loki-nine",
        "loki",
        "logs",
        "ops",
        "Ops",
        "expr",
        "{job=\"grafana\"}",
    );

    let mut missing_args = make_import_args(export_root.clone());
    missing_args.common = make_basic_common_args("http://127.0.0.1:3000".to_string());
    missing_args.use_export_org = true;
    missing_args.dry_run = true;
    missing_args.json = true;
    missing_args.create_missing_orgs = false;

    let missing_payload: Value = serde_json::from_str(
        &test_support::import::build_routed_import_dry_run_json_with_request(
            |method, path, _params, _payload| match (method, path) {
                (reqwest::Method::GET, "/api/orgs") => Ok(Some(json!([
                    {"id": 2, "name": "Org Two"}
                ]))),
                _ => Err(test_support::message(format!("unexpected request {path}"))),
            },
            |_target_org_id, scoped_args| {
                Ok(test_support::import::ImportDryRunReport {
                    mode: "create-only".to_string(),
                    import_dir: scoped_args.import_dir.clone(),
                    folder_statuses: Vec::new(),
                    dashboard_records: Vec::new(),
                    skipped_missing_count: 0,
                    skipped_folder_mismatch_count: 0,
                })
            },
            &missing_args,
        )
        .unwrap(),
    )
    .unwrap();

    let mut would_create_args = make_import_args(export_root.clone());
    would_create_args.common = make_basic_common_args("http://127.0.0.1:3000".to_string());
    would_create_args.use_export_org = true;
    would_create_args.dry_run = true;
    would_create_args.json = true;
    would_create_args.create_missing_orgs = true;

    let would_create_payload: Value = serde_json::from_str(
        &test_support::import::build_routed_import_dry_run_json_with_request(
            |method, path, _params, _payload| match (method, path) {
                (reqwest::Method::GET, "/api/orgs") => Ok(Some(json!([
                    {"id": 2, "name": "Org Two"}
                ]))),
                _ => Err(test_support::message(format!("unexpected request {path}"))),
            },
            |_target_org_id, scoped_args| {
                Ok(test_support::import::ImportDryRunReport {
                    mode: "create-only".to_string(),
                    import_dir: scoped_args.import_dir.clone(),
                    folder_statuses: Vec::new(),
                    dashboard_records: Vec::new(),
                    skipped_missing_count: 0,
                    skipped_folder_mismatch_count: 0,
                })
            },
            &would_create_args,
        )
        .unwrap(),
    )
    .unwrap();

    let mut live_args = make_import_args(export_root);
    live_args.common = make_basic_common_args("http://127.0.0.1:3000".to_string());
    live_args.use_export_org = true;
    live_args.dry_run = false;
    live_args.create_missing_orgs = true;

    let mut created_rows = Vec::new();
    let count = test_support::import::import_dashboards_by_export_org_with_request(
        |method, path, _params, payload| match (method, path) {
            (reqwest::Method::GET, "/api/orgs") => Ok(Some(json!([
                {"id": 2, "name": "Org Two"}
            ]))),
            (reqwest::Method::POST, "/api/orgs") => {
                assert_eq!(
                    payload
                        .and_then(|value| value.as_object())
                        .unwrap()
                        .get("name"),
                    Some(&json!("Ops Org"))
                );
                Ok(Some(json!({"orgId": "19"})))
            }
            _ => Err(test_support::message(format!("unexpected request {path}"))),
        },
        |target_org_id, scoped_args| {
            created_rows.push((
                target_org_id,
                scoped_args.import_dir.clone(),
                scoped_args.org_id,
            ));
            Ok(1)
        },
        |_target_org_id, _scoped_args| unreachable!("dry-run collector should not be used"),
        &live_args,
    )
    .unwrap();

    assert_eq!(count, 2);
    assert_eq!(
        created_rows,
        vec![
            (2, org_two_raw.clone(), Some(2)),
            (19, org_nine_raw.clone(), Some(19))
        ]
    );

    let missing_orgs = missing_payload["orgs"].as_array().unwrap();
    let would_create_orgs = would_create_payload["orgs"].as_array().unwrap();
    let missing_existing = missing_orgs
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(2))
        .unwrap();
    let missing_missing = missing_orgs
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(9))
        .unwrap();
    let would_create_existing = would_create_orgs
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(2))
        .unwrap();
    let would_create_missing = would_create_orgs
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(9))
        .unwrap();

    assert_eq!(missing_payload["summary"]["existingOrgCount"], json!(1));
    assert_eq!(missing_payload["summary"]["missingOrgCount"], json!(1));
    assert_eq!(missing_payload["summary"]["wouldCreateOrgCount"], json!(0));
    assert_eq!(
        would_create_payload["summary"]["existingOrgCount"],
        json!(1)
    );
    assert_eq!(would_create_payload["summary"]["missingOrgCount"], json!(0));
    assert_eq!(
        would_create_payload["summary"]["wouldCreateOrgCount"],
        json!(1)
    );

    assert_eq!(missing_existing["orgAction"], json!("exists"));
    assert_eq!(missing_existing["targetOrgId"], json!(2));
    assert_eq!(missing_missing["orgAction"], json!("missing"));
    assert_eq!(missing_missing["targetOrgId"], Value::Null);
    assert_eq!(would_create_existing["orgAction"], json!("exists"));
    assert_eq!(would_create_existing["targetOrgId"], json!(2));
    assert_eq!(would_create_missing["orgAction"], json!("would-create"));
    assert_eq!(would_create_missing["targetOrgId"], Value::Null);

    let existing_summary = test_support::import::format_routed_import_scope_summary_fields(
        2,
        "Org Two",
        "exists",
        Some(2),
        &org_two_raw,
    );
    let missing_summary = test_support::import::format_routed_import_scope_summary_fields(
        9,
        "Ops Org",
        "missing",
        None,
        &org_nine_raw,
    );
    let would_create_summary = test_support::import::format_routed_import_scope_summary_fields(
        9,
        "Ops Org",
        "would-create",
        None,
        &org_nine_raw,
    );
    let created_summary = test_support::import::format_routed_import_scope_summary_fields(
        9,
        "Ops Org",
        "created",
        Some(19),
        &org_nine_raw,
    );
    assert!(existing_summary.contains("orgAction=exists"));
    assert!(existing_summary.contains("targetOrgId=2"));
    assert!(missing_summary.contains("orgAction=missing"));
    assert!(missing_summary.contains("targetOrgId=<new>"));
    assert!(would_create_summary.contains("orgAction=would-create"));
    assert!(would_create_summary.contains("targetOrgId=<new>"));
    assert!(created_summary.contains("orgAction=created"));
    assert!(created_summary.contains("targetOrgId=19"));
}

#[test]
fn import_dashboards_with_use_export_org_dry_run_filters_selected_orgs_without_creating_missing_targets(
) {
    let temp = tempdir().unwrap();
    let export_root = temp.path().join("exports");
    let org_two_raw = export_root.join("org_2_Org_Two").join("raw");
    let org_five_raw = export_root.join("org_5_Org_Five").join("raw");
    fs::create_dir_all(&org_two_raw).unwrap();
    fs::create_dir_all(&org_five_raw).unwrap();

    for (raw_dir, org_id, org_name, uid) in [
        (&org_two_raw, "2", "Org Two", "cpu-two"),
        (&org_five_raw, "5", "Org Five", "cpu-five"),
    ] {
        fs::write(
            raw_dir.join(EXPORT_METADATA_FILENAME),
            serde_json::to_string_pretty(&json!({
                "kind": "grafana-utils-dashboard-export-index",
                "schemaVersion": TOOL_SCHEMA_VERSION,
                "variant": "raw",
                "dashboardCount": 1,
                "indexFile": "index.json",
                "format": "grafana-web-import-preserve-uid"
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join("index.json"),
            serde_json::to_string_pretty(&json!([
                {
                    "uid": uid,
                    "title": "CPU",
                    "path": "dash.json",
                    "format": "grafana-web-import-preserve-uid",
                    "org": org_name,
                    "orgId": org_id
                }
            ]))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join("dash.json"),
            serde_json::to_string_pretty(&json!({
                "dashboard": {"id": null, "uid": uid, "title": "CPU"}
            }))
            .unwrap(),
        )
        .unwrap();
    }

    let mut args = make_import_args(export_root);
    args.common = make_basic_common_args("http://127.0.0.1:3000".to_string());
    args.use_export_org = true;
    args.only_org_id = vec![2, 5];
    args.dry_run = true;

    let mut admin_calls = Vec::new();
    let mut import_calls = Vec::new();
    test_support::import::import_dashboards_by_export_org_with_request(
        |method, path, _params, _payload| {
            admin_calls.push((method.to_string(), path.to_string()));
            match (method, path) {
                (reqwest::Method::GET, "/api/orgs") => Ok(Some(json!([
                    {"id": 2, "name": "Org Two"}
                ]))),
                _ => Err(test_support::message(format!("unexpected request {path}"))),
            }
        },
        |target_org_id, scoped_args| {
            import_calls.push((
                target_org_id,
                scoped_args.import_dir.clone(),
                scoped_args.org_id,
            ));
            Ok(0)
        },
        |_target_org_id, scoped_args| {
            Ok(test_support::import::ImportDryRunReport {
                mode: "create-only".to_string(),
                import_dir: scoped_args.import_dir.clone(),
                folder_statuses: Vec::new(),
                dashboard_records: Vec::new(),
                skipped_missing_count: 0,
                skipped_folder_mismatch_count: 0,
            })
        },
        &args,
    )
    .unwrap();

    assert_eq!(
        admin_calls,
        vec![("GET".to_string(), "/api/orgs".to_string())]
    );
    assert_eq!(import_calls, vec![(2, org_two_raw.clone(), Some(2))]);
}

#[test]
fn build_routed_import_dry_run_json_with_request_reuses_org_inventory_lookup() {
    let temp = tempdir().unwrap();
    let export_root = temp.path().join("exports");
    let org_two_raw = export_root.join("org_2_Org_Two").join("raw");
    let org_nine_raw = export_root.join("org_9_Ops_Org").join("raw");
    fs::create_dir_all(&org_two_raw).unwrap();
    fs::create_dir_all(&org_nine_raw).unwrap();

    for (raw_dir, org_id, org_name, uid) in [
        (&org_two_raw, "2", "Org Two", "cpu-two"),
        (&org_nine_raw, "9", "Ops Org", "ops-main"),
    ] {
        fs::write(
            raw_dir.join(EXPORT_METADATA_FILENAME),
            serde_json::to_string_pretty(&json!({
                "kind": "grafana-utils-dashboard-export-index",
                "schemaVersion": TOOL_SCHEMA_VERSION,
                "variant": "raw",
                "dashboardCount": 1,
                "indexFile": "index.json",
                "format": "grafana-web-import-preserve-uid"
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join("index.json"),
            serde_json::to_string_pretty(&json!([
                {
                    "uid": uid,
                    "title": "CPU",
                    "path": "dash.json",
                    "format": "grafana-web-import-preserve-uid",
                    "org": org_name,
                    "orgId": org_id
                }
            ]))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join("dash.json"),
            serde_json::to_string_pretty(&json!({
                "dashboard": {"id": null, "uid": uid, "title": "CPU"}
            }))
            .unwrap(),
        )
        .unwrap();
    }

    let mut args = make_import_args(export_root);
    args.common = make_basic_common_args("http://127.0.0.1:3000".to_string());
    args.use_export_org = true;
    args.create_missing_orgs = true;
    args.dry_run = true;
    args.json = true;

    let mut admin_calls = Vec::new();
    let payload: Value = serde_json::from_str(
        &test_support::import::build_routed_import_dry_run_json_with_request(
            |method, path, _params, _payload| {
                admin_calls.push((method.to_string(), path.to_string()));
                match (method, path) {
                    (reqwest::Method::GET, "/api/orgs") => Ok(Some(json!([
                        {"id": 2, "name": "Org Two"}
                    ]))),
                    _ => Err(test_support::message(format!("unexpected request {path}"))),
                }
            },
            |target_org_id, scoped_args| {
                Ok(test_support::import::ImportDryRunReport {
                    mode: "create-only".to_string(),
                    import_dir: scoped_args.import_dir.clone(),
                    folder_statuses: Vec::new(),
                    dashboard_records: vec![[
                        if target_org_id == 2 {
                            "cpu-two".to_string()
                        } else {
                            "ops-main".to_string()
                        },
                        "missing".to_string(),
                        "create".to_string(),
                        "General".to_string(),
                        "".to_string(),
                        "".to_string(),
                        "".to_string(),
                        scoped_args
                            .import_dir
                            .join("dash.json")
                            .display()
                            .to_string(),
                    ]],
                    skipped_missing_count: 0,
                    skipped_folder_mismatch_count: 0,
                })
            },
            &args,
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        admin_calls,
        vec![("GET".to_string(), "/api/orgs".to_string())]
    );
    assert_eq!(payload["orgs"].as_array().unwrap().len(), 2);
}

#[test]
fn build_routed_import_dry_run_json_with_request_reports_orgs_and_dashboards() {
    let temp = tempdir().unwrap();
    let export_root = temp.path().join("exports");
    let org_two_raw = export_root.join("org_2_Org_Two").join("raw");
    let org_nine_raw = export_root.join("org_9_Ops_Org").join("raw");
    fs::create_dir_all(export_root.join("raw")).unwrap();
    fs::create_dir_all(&org_two_raw).unwrap();
    fs::create_dir_all(&org_nine_raw).unwrap();
    fs::write(
        export_root.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-root",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "orgCount": 2
        }))
        .unwrap(),
    )
    .unwrap();

    for (raw_dir, org_id, org_name, uid) in [
        (&org_two_raw, "2", "Org Two", "cpu-two"),
        (&org_nine_raw, "9", "Ops Org", "ops-main"),
    ] {
        fs::write(
            raw_dir.join(EXPORT_METADATA_FILENAME),
            serde_json::to_string_pretty(&json!({
                "kind": "grafana-utils-dashboard-export-index",
                "schemaVersion": TOOL_SCHEMA_VERSION,
                "variant": "raw",
                "dashboardCount": 1,
                "indexFile": "index.json",
                "format": "grafana-web-import-preserve-uid"
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join("index.json"),
            serde_json::to_string_pretty(&json!([
                {
                    "uid": uid,
                    "title": "CPU",
                    "path": "dash.json",
                    "format": "grafana-web-import-preserve-uid",
                    "org": org_name,
                    "orgId": org_id
                }
            ]))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join("dash.json"),
            serde_json::to_string_pretty(&json!({
                "dashboard": {"id": null, "uid": uid, "title": "CPU"}
            }))
            .unwrap(),
        )
        .unwrap();
    }

    let mut args = make_import_args(export_root);
    args.common = make_basic_common_args("http://127.0.0.1:3000".to_string());
    args.use_export_org = true;
    args.create_missing_orgs = true;
    args.dry_run = true;
    args.json = true;

    let payload: Value = serde_json::from_str(
        &test_support::import::build_routed_import_dry_run_json_with_request(
            |method, path, _params, _payload| match (method, path) {
                (reqwest::Method::GET, "/api/orgs") => Ok(Some(json!([
                    {"id": 2, "name": "Org Two"}
                ]))),
                _ => Err(test_support::message(format!("unexpected request {path}"))),
            },
            |target_org_id, scoped_args| {
                Ok(test_support::import::ImportDryRunReport {
                    mode: "create-only".to_string(),
                    import_dir: scoped_args.import_dir.clone(),
                    folder_statuses: Vec::new(),
                    dashboard_records: vec![[
                        if target_org_id == 2 {
                            "cpu-two".to_string()
                        } else {
                            "ops-main".to_string()
                        },
                        "missing".to_string(),
                        "create".to_string(),
                        "General".to_string(),
                        "".to_string(),
                        "".to_string(),
                        "".to_string(),
                        scoped_args
                            .import_dir
                            .join("dash.json")
                            .display()
                            .to_string(),
                    ]],
                    skipped_missing_count: 0,
                    skipped_folder_mismatch_count: 0,
                })
            },
            &args,
        )
        .unwrap(),
    )
    .unwrap();

    let org_entries = payload["orgs"].as_array().unwrap();
    let import_entries = payload["imports"].as_array().unwrap();
    let existing_org = org_entries
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(2))
        .unwrap();
    let missing_org = org_entries
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(9))
        .unwrap();
    let existing_import = import_entries
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(2))
        .unwrap();
    let missing_import = import_entries
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(9))
        .unwrap();

    assert_eq!(payload["mode"], "routed-import-preview");
    assert_eq!(existing_org["orgAction"], "exists");
    assert_eq!(missing_org["orgAction"], "would-create");
    assert_eq!(existing_import["dashboards"][0]["uid"], "cpu-two");
    assert_eq!(missing_import["dashboards"], json!([]));
    assert_eq!(missing_import["summary"]["dashboardCount"], Value::from(1));
}

#[test]
fn import_dashboards_with_use_export_org_dry_run_table_returns_after_org_summary() {
    let temp = tempdir().unwrap();
    let export_root = temp.path().join("exports");
    let org_two_raw = export_root.join("org_2_Org_Two").join("raw");
    fs::create_dir_all(export_root.join("raw")).unwrap();
    fs::create_dir_all(&org_two_raw).unwrap();
    fs::write(
        export_root.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-root",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "orgCount": 1
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        org_two_raw.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        org_two_raw.join("index.json"),
        serde_json::to_string_pretty(&json!([
            {
                "uid": "cpu-two",
                "title": "CPU",
                "path": "dash.json",
                "format": "grafana-web-import-preserve-uid",
                "org": "Org Two",
                "orgId": "2"
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        org_two_raw.join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": null, "uid": "cpu-two", "title": "CPU"}
        }))
        .unwrap(),
    )
    .unwrap();

    let mut args = make_import_args(export_root);
    args.common = make_basic_common_args("http://127.0.0.1:3000".to_string());
    args.use_export_org = true;
    args.dry_run = true;
    args.table = true;

    let mut import_calls = Vec::new();
    let count = test_support::import::import_dashboards_by_export_org_with_request(
        |method, path, _params, _payload| match (method, path) {
            (reqwest::Method::GET, "/api/orgs") => Ok(Some(json!([
                {"id": 2, "name": "Org Two"}
            ]))),
            _ => Err(test_support::message(format!("unexpected request {path}"))),
        },
        |target_org_id, scoped_args| {
            import_calls.push((target_org_id, scoped_args.import_dir.clone()));
            Ok(0)
        },
        |_target_org_id, scoped_args| {
            Ok(test_support::import::ImportDryRunReport {
                mode: "create-only".to_string(),
                import_dir: scoped_args.import_dir.clone(),
                folder_statuses: Vec::new(),
                dashboard_records: Vec::new(),
                skipped_missing_count: 0,
                skipped_folder_mismatch_count: 0,
            })
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 0);
    assert!(import_calls.is_empty());
}

#[test]
fn build_import_auth_context_adds_org_header_for_basic_auth_imports() {
    let temp = tempdir().unwrap();
    let args = ImportArgs {
        common: make_basic_common_args("http://127.0.0.1:3000".to_string()),
        org_id: Some(7),
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: temp.path().join("raw"),
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: false,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: true,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };

    let context = build_import_auth_context(&args).unwrap();

    assert_eq!(context.auth_mode, "basic");
    assert!(context
        .headers
        .iter()
        .any(|(name, value)| { name == "X-Grafana-Org-Id" && value == "7" }));
}

#[test]
fn import_dashboards_with_use_export_org_filters_selected_orgs_and_creates_missing_targets() {
    let temp = tempdir().unwrap();
    let export_root = temp.path().join("exports");
    let org_two_raw = export_root.join("org_2_Org_Two").join("raw");
    let org_five_raw = export_root.join("org_5_Org_Five").join("raw");
    fs::create_dir_all(&org_two_raw).unwrap();
    fs::create_dir_all(&org_five_raw).unwrap();

    for (raw_dir, org_id, org_name, uid) in [
        (&org_two_raw, "2", "Org Two", "cpu-two"),
        (&org_five_raw, "5", "Org Five", "cpu-five"),
    ] {
        fs::write(
            raw_dir.join(EXPORT_METADATA_FILENAME),
            serde_json::to_string_pretty(&json!({
                "kind": "grafana-utils-dashboard-export-index",
                "schemaVersion": TOOL_SCHEMA_VERSION,
                "variant": "raw",
                "dashboardCount": 1,
                "indexFile": "index.json",
                "format": "grafana-web-import-preserve-uid"
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join("index.json"),
            serde_json::to_string_pretty(&json!([
                {
                    "uid": uid,
                    "title": "CPU",
                    "path": "dash.json",
                    "format": "grafana-web-import-preserve-uid",
                    "org": org_name,
                    "orgId": org_id
                }
            ]))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join("dash.json"),
            serde_json::to_string_pretty(&json!({
                "dashboard": {"id": null, "uid": uid, "title": "CPU"}
            }))
            .unwrap(),
        )
        .unwrap();
    }

    let mut args = make_import_args(export_root.clone());
    args.common = make_basic_common_args("http://127.0.0.1:3000".to_string());
    args.use_export_org = true;
    args.create_missing_orgs = true;
    args.only_org_id = vec![2];
    args.dry_run = false;

    let mut admin_calls = Vec::new();
    let mut import_calls = Vec::new();
    let count = test_support::import::import_dashboards_by_export_org_with_request(
        |method: reqwest::Method,
         path: &str,
         _params: &[(String, String)],
         payload: Option<&Value>| {
            admin_calls.push((method.to_string(), path.to_string()));
            match (method.clone(), path) {
                (reqwest::Method::GET, "/api/orgs") => Ok(Some(json!([]))),
                (reqwest::Method::POST, "/api/orgs") => {
                    assert_eq!(
                        payload
                            .and_then(|value| value.as_object())
                            .unwrap()
                            .get("name"),
                        Some(&json!("Org Two"))
                    );
                    Ok(Some(json!({"orgId": "9"})))
                }
                _ => Err(test_support::message(format!(
                    "unexpected request {method} {path}"
                ))),
            }
        },
        |target_org_id, scoped_args| {
            import_calls.push((
                target_org_id,
                scoped_args.import_dir.clone(),
                scoped_args.org_id,
            ));
            assert!(!scoped_args.use_export_org);
            assert!(scoped_args.only_org_id.is_empty());
            assert!(!scoped_args.create_missing_orgs);
            Ok(1)
        },
        |_target_org_id, scoped_args| {
            Ok(test_support::import::ImportDryRunReport {
                mode: "create-only".to_string(),
                import_dir: scoped_args.import_dir.clone(),
                folder_statuses: Vec::new(),
                dashboard_records: Vec::new(),
                skipped_missing_count: 0,
                skipped_folder_mismatch_count: 0,
            })
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert_eq!(
        admin_calls,
        vec![
            ("GET".to_string(), "/api/orgs".to_string()),
            ("POST".to_string(), "/api/orgs".to_string()),
        ]
    );
    assert_eq!(import_calls, vec![(9, org_two_raw.clone(), Some(9))]);
}

#[test]
fn import_dashboards_with_use_export_org_round_trips_combined_export_root_into_scoped_diff() {
    use std::cell::RefCell;
    use std::collections::BTreeMap;
    use std::rc::Rc;

    let temp = tempdir().unwrap();
    let export_root = temp.path().join("exports");
    let org_one_raw = export_root.join("org_1_Main_Org").join("raw");
    let org_two_raw = export_root.join("org_2_Ops_Org").join("raw");
    fs::create_dir_all(&org_one_raw).unwrap();
    fs::create_dir_all(&org_two_raw).unwrap();
    fs::write(
        export_root.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "root",
            "dashboardCount": 2,
            "indexFile": "index.json",
            "orgCount": 2,
            "orgs": [
                {
                    "org": "Main Org.",
                    "orgId": "1",
                    "dashboardCount": 1,
                    "exportDir": "org_1_Main_Org"
                },
                {
                    "org": "Ops Org",
                    "orgId": "2",
                    "dashboardCount": 1,
                    "exportDir": "org_2_Ops_Org"
                }
            ]
        }))
        .unwrap(),
    )
    .unwrap();

    for (
        raw_dir,
        org_id,
        org_name,
        dashboard_uid,
        dashboard_title,
        datasource_uid,
        datasource_type,
        panel_type,
        folder_uid,
        folder_title,
        query_field,
        query_text,
    ) in [
        (
            &org_one_raw,
            "1",
            "Main Org.",
            "cpu-main",
            "CPU Main",
            "prom-main",
            "prometheus",
            "timeseries",
            "general",
            "General",
            "expr",
            "up",
        ),
        (
            &org_two_raw,
            "2",
            "Ops Org",
            "logs-main",
            "Logs Main",
            "loki-main",
            "loki",
            "logs",
            "ops",
            "Ops",
            "expr",
            "{job=\"grafana\"} |= \"error\"",
        ),
    ] {
        fs::write(
            raw_dir.join(EXPORT_METADATA_FILENAME),
            serde_json::to_string_pretty(&json!({
                "kind": "grafana-utils-dashboard-export-index",
                "schemaVersion": TOOL_SCHEMA_VERSION,
                "variant": "raw",
                "dashboardCount": 1,
                "indexFile": "index.json",
                "format": "grafana-web-import-preserve-uid",
                "foldersFile": FOLDER_INVENTORY_FILENAME,
                "datasourcesFile": DATASOURCE_INVENTORY_FILENAME,
                "org": org_name,
                "orgId": org_id
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join(FOLDER_INVENTORY_FILENAME),
            serde_json::to_string_pretty(&json!([
                {
                    "uid": folder_uid,
                    "title": folder_title,
                    "path": folder_title,
                    "org": org_name,
                    "orgId": org_id
                }
            ]))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join(DATASOURCE_INVENTORY_FILENAME),
            serde_json::to_string_pretty(&json!([
                {
                    "uid": datasource_uid,
                    "name": datasource_uid,
                    "type": datasource_type,
                    "access": "proxy",
                    "url": "http://grafana.example.internal",
                    "isDefault": "true",
                    "org": org_name,
                    "orgId": org_id
                }
            ]))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join("index.json"),
            serde_json::to_string_pretty(&json!([
                {
                    "uid": dashboard_uid,
                    "title": dashboard_title,
                    "path": "dash.json",
                    "format": "grafana-web-import-preserve-uid",
                    "org": org_name,
                    "orgId": org_id
                }
            ]))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join("dash.json"),
            serde_json::to_string_pretty(&json!({
                "dashboard": {
                    "id": null,
                    "uid": dashboard_uid,
                    "title": dashboard_title,
                    "schemaVersion": 38,
                    "panels": [{
                        "id": 7,
                        "title": dashboard_title,
                        "type": panel_type,
                        "datasource": {"uid": datasource_uid, "type": datasource_type},
                        "targets": [{
                            "refId": "A",
                            query_field: query_text
                        }]
                    }]
                },
                "meta": {
                    "folderUid": folder_uid,
                    "folderTitle": folder_title
                }
            }))
            .unwrap(),
        )
        .unwrap();
    }

    let mut args = make_import_args(export_root.clone());
    args.common = make_basic_common_args("http://127.0.0.1:3000".to_string());
    args.use_export_org = true;
    args.dry_run = false;

    let imported_dashboards: Rc<RefCell<BTreeMap<i64, BTreeMap<String, Value>>>> =
        Rc::new(RefCell::new(BTreeMap::new()));
    let routed_scopes: Rc<RefCell<Vec<(i64, PathBuf)>>> = Rc::new(RefCell::new(Vec::new()));

    let imported_dashboards_for_import = Rc::clone(&imported_dashboards);
    let routed_scopes_for_import = Rc::clone(&routed_scopes);
    let count = test_support::import::import_dashboards_by_export_org_with_request(
        |method: reqwest::Method,
         path: &str,
         _params: &[(String, String)],
         _payload: Option<&Value>| {
            match (method.clone(), path) {
                (reqwest::Method::GET, "/api/orgs") => Ok(Some(json!([
                    {"id": 1, "name": "Main Org."},
                    {"id": 2, "name": "Ops Org"}
                ]))),
                _ => Err(test_support::message(format!(
                    "unexpected admin request {method} {path}"
                ))),
            }
        },
        move |target_org_id, scoped_args| {
            routed_scopes_for_import
                .borrow_mut()
                .push((target_org_id, scoped_args.import_dir.clone()));
            let imported_dashboards = Rc::clone(&imported_dashboards_for_import);
            import_dashboards_with_request(
                with_dashboard_import_live_preflight(
                    json!([
                        {"uid":"prom-main","name":"prom-main","type":"prometheus"},
                        {"uid":"loki-main","name":"loki-main","type":"loki"}
                    ]),
                    json!([
                        {"id":"timeseries"},
                        {"id":"logs"}
                    ]),
                    move |method, path, _params, payload| match (method.clone(), path) {
                        (reqwest::Method::POST, "/api/dashboards/db") => {
                            let payload = payload.cloned().unwrap();
                            let uid = payload["dashboard"]["uid"].as_str().unwrap().to_string();
                            imported_dashboards
                                .borrow_mut()
                                .entry(target_org_id)
                                .or_default()
                                .insert(uid, payload);
                            Ok(Some(json!({"status":"success"})))
                        }
                        (reqwest::Method::GET, _) if path.starts_with("/api/dashboards/uid/") => {
                            let uid = path.trim_start_matches("/api/dashboards/uid/");
                            Ok(imported_dashboards
                                .borrow()
                                .get(&target_org_id)
                                .and_then(|dashboards| dashboards.get(uid).cloned()))
                        }
                        _ => Err(test_support::message(format!(
                            "unexpected scoped import request {method} {path}"
                        ))),
                    },
                ),
                scoped_args,
            )
        },
        |_target_org_id, _scoped_args| unreachable!("dry-run collector should not be used"),
        &args,
    )
    .unwrap();

    assert_eq!(count, 2);
    assert_eq!(
        routed_scopes.borrow().as_slice(),
        &[(1, org_one_raw.clone()), (2, org_two_raw.clone())]
    );
    assert_eq!(imported_dashboards.borrow().len(), 2);

    for (target_org_id, import_dir) in routed_scopes.borrow().iter() {
        let stored = imported_dashboards
            .borrow()
            .get(target_org_id)
            .cloned()
            .unwrap();
        let folder_uid = if *target_org_id == 1 {
            "general"
        } else {
            "ops"
        };
        let diff_args = DiffArgs {
            common: make_common_args("http://127.0.0.1:3000".to_string()),
            import_dir: import_dir.clone(),
            import_folder_uid: Some(folder_uid.to_string()),
            context_lines: 3,
        };
        let differences = diff_dashboards_with_request(
            |_method, path, _params, _payload| {
                let uid = path.trim_start_matches("/api/dashboards/uid/");
                Ok(stored.get(uid).cloned())
            },
            &diff_args,
        )
        .unwrap();
        assert_eq!(
            differences, 0,
            "expected clean diff for org {target_org_id}"
        );
    }
}

#[test]
fn import_dashboards_with_use_export_org_stops_on_scoped_preflight_failure() {
    use std::cell::RefCell;
    use std::collections::BTreeMap;
    use std::rc::Rc;

    let temp = tempdir().unwrap();
    let export_root = temp.path().join("exports");
    let org_one_raw = export_root.join("org_1_Main_Org").join("raw");
    let org_two_raw = export_root.join("org_2_Ops_Org").join("raw");
    let org_three_raw = export_root.join("org_3_App_Org").join("raw");
    write_combined_export_root_metadata(
        &export_root,
        &[
            ("1", "Main Org.", "org_1_Main_Org"),
            ("2", "Ops Org", "org_2_Ops_Org"),
            ("3", "App Org", "org_3_App_Org"),
        ],
    );
    write_basic_raw_export(
        &org_one_raw,
        "1",
        "Main Org.",
        "cpu-main",
        "CPU Main",
        "prom-main",
        "prometheus",
        "timeseries",
        "general",
        "General",
        "expr",
        "up",
    );
    write_basic_raw_export(
        &org_two_raw,
        "2",
        "Ops Org",
        "logs-main",
        "Logs Main",
        "loki-main",
        "loki",
        "logs",
        "ops",
        "Ops",
        "expr",
        "{job=\"grafana\"} |= \"error\"",
    );
    write_basic_raw_export(
        &org_three_raw,
        "3",
        "App Org",
        "app-main",
        "App Main",
        "prom-app",
        "prometheus",
        "timeseries",
        "apps",
        "Apps",
        "expr",
        "rate(http_requests_total[5m])",
    );

    let mut args = make_import_args(export_root);
    args.common = make_basic_common_args("http://127.0.0.1:3000".to_string());
    args.use_export_org = true;
    args.dry_run = false;

    let attempted_orgs: Rc<RefCell<Vec<i64>>> = Rc::new(RefCell::new(Vec::new()));
    let imported_dashboards: Rc<RefCell<BTreeMap<i64, Vec<String>>>> =
        Rc::new(RefCell::new(BTreeMap::new()));

    let attempted_orgs_for_import = Rc::clone(&attempted_orgs);
    let imported_dashboards_for_import = Rc::clone(&imported_dashboards);
    let error = test_support::import::import_dashboards_by_export_org_with_request(
        |method: reqwest::Method,
         path: &str,
         _params: &[(String, String)],
         _payload: Option<&Value>| match (method.clone(), path) {
            (reqwest::Method::GET, "/api/orgs") => Ok(Some(json!([
                {"id": 1, "name": "Main Org."},
                {"id": 2, "name": "Ops Org"},
                {"id": 3, "name": "App Org"}
            ]))),
            _ => Err(test_support::message(format!(
                "unexpected admin request {method} {path}"
            ))),
        },
        move |target_org_id, scoped_args| {
            attempted_orgs_for_import.borrow_mut().push(target_org_id);
            let imported_dashboards = Rc::clone(&imported_dashboards_for_import);
            let preflight_datasources = if target_org_id == 1 {
                json!([
                    {"uid":"prom-main","name":"prom-main","type":"prometheus"}
                ])
            } else {
                json!([
                    {"uid":"other","name":"other","type":"prometheus"}
                ])
            };
            let preflight_plugins = if target_org_id == 1 {
                json!([
                    {"id":"timeseries"}
                ])
            } else {
                json!([
                    {"id":"logs"}
                ])
            };
            import_dashboards_with_request(
                with_dashboard_import_live_preflight(
                    preflight_datasources,
                    preflight_plugins,
                    move |method, path, _params, payload| match (method.clone(), path) {
                        (reqwest::Method::POST, "/api/dashboards/db") => {
                            let payload = payload.cloned().unwrap();
                            let uid = payload["dashboard"]["uid"].as_str().unwrap().to_string();
                            imported_dashboards
                                .borrow_mut()
                                .entry(target_org_id)
                                .or_default()
                                .push(uid);
                            Ok(Some(json!({"status":"success"})))
                        }
                        (reqwest::Method::GET, _) if path.starts_with("/api/dashboards/uid/") => {
                            Ok(None)
                        }
                        _ => Err(test_support::message(format!(
                            "unexpected scoped request {method} {path}"
                        ))),
                    },
                ),
                scoped_args,
            )
        },
        |_target_org_id, _scoped_args| unreachable!("dry-run collector should not be used"),
        &args,
    )
    .unwrap_err();

    let error_text = error.to_string();
    assert!(error_text.contains(
        "Dashboard routed import failed for export orgId=2 name=Ops Org orgAction=exists targetOrgId=2"
    ));
    assert!(error_text.contains("org_2_Ops_Org/raw"));
    assert!(error_text.contains("Refusing dashboard import because preflight reports"));
    assert_eq!(attempted_orgs.borrow().as_slice(), &[1, 2]);
    assert_eq!(
        imported_dashboards.borrow().get(&1),
        Some(&vec!["cpu-main".to_string()])
    );
    assert_eq!(imported_dashboards.borrow().get(&2), None);
    assert_eq!(imported_dashboards.borrow().get(&3), None);
}

#[test]
fn import_dashboards_with_use_export_org_only_org_id_skips_unselected_org_preflight() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let temp = tempdir().unwrap();
    let export_root = temp.path().join("exports");
    let org_one_raw = export_root.join("org_1_Main_Org").join("raw");
    let org_two_raw = export_root.join("org_2_Ops_Org").join("raw");
    write_combined_export_root_metadata(
        &export_root,
        &[
            ("1", "Main Org.", "org_1_Main_Org"),
            ("2", "Ops Org", "org_2_Ops_Org"),
        ],
    );
    write_basic_raw_export(
        &org_one_raw,
        "1",
        "Main Org.",
        "cpu-main",
        "CPU Main",
        "prom-main",
        "prometheus",
        "timeseries",
        "general",
        "General",
        "expr",
        "up",
    );
    write_basic_raw_export(
        &org_two_raw,
        "2",
        "Ops Org",
        "logs-main",
        "Logs Main",
        "loki-main",
        "loki",
        "logs",
        "ops",
        "Ops",
        "expr",
        "{job=\"grafana\"} |= \"error\"",
    );

    let mut args = make_import_args(export_root);
    args.common = make_basic_common_args("http://127.0.0.1:3000".to_string());
    args.use_export_org = true;
    args.only_org_id = vec![2];
    args.dry_run = false;

    let attempted_orgs: Rc<RefCell<Vec<i64>>> = Rc::new(RefCell::new(Vec::new()));
    let posts: Rc<RefCell<Vec<(i64, String)>>> = Rc::new(RefCell::new(Vec::new()));

    let attempted_orgs_for_import = Rc::clone(&attempted_orgs);
    let posts_for_import = Rc::clone(&posts);
    let count = test_support::import::import_dashboards_by_export_org_with_request(
        |method: reqwest::Method,
         path: &str,
         _params: &[(String, String)],
         _payload: Option<&Value>| match (method.clone(), path) {
            (reqwest::Method::GET, "/api/orgs") => Ok(Some(json!([
                {"id": 1, "name": "Main Org."},
                {"id": 2, "name": "Ops Org"}
            ]))),
            _ => Err(test_support::message(format!(
                "unexpected admin request {method} {path}"
            ))),
        },
        move |target_org_id, scoped_args| {
            attempted_orgs_for_import.borrow_mut().push(target_org_id);
            assert_eq!(target_org_id, 2, "unselected org should not be imported");
            let posts = Rc::clone(&posts_for_import);
            import_dashboards_with_request(
                with_dashboard_import_live_preflight(
                    json!([
                        {"uid":"loki-main","name":"loki-main","type":"loki"}
                    ]),
                    json!([
                        {"id":"logs"}
                    ]),
                    move |method, path, _params, payload| match (method.clone(), path) {
                        (reqwest::Method::POST, "/api/dashboards/db") => {
                            let payload = payload.cloned().unwrap();
                            posts.borrow_mut().push((
                                target_org_id,
                                payload["dashboard"]["uid"].as_str().unwrap().to_string(),
                            ));
                            Ok(Some(json!({"status":"success"})))
                        }
                        (reqwest::Method::GET, _) if path.starts_with("/api/dashboards/uid/") => {
                            Ok(None)
                        }
                        _ => Err(test_support::message(format!(
                            "unexpected scoped request {method} {path}"
                        ))),
                    },
                ),
                scoped_args,
            )
        },
        |_target_org_id, _scoped_args| unreachable!("dry-run collector should not be used"),
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert_eq!(attempted_orgs.borrow().as_slice(), &[2]);
    assert_eq!(posts.borrow().as_slice(), &[(2, "logs-main".to_string())]);
}

#[test]
fn build_routed_import_dry_run_json_and_live_failure_share_org_scope_identity() {
    let temp = tempdir().unwrap();
    let export_root = temp.path().join("exports");
    let org_one_raw = export_root.join("org_1_Main_Org").join("raw");
    let org_two_raw = export_root.join("org_2_Ops_Org").join("raw");
    write_combined_export_root_metadata(
        &export_root,
        &[
            ("1", "Main Org.", "org_1_Main_Org"),
            ("2", "Ops Org", "org_2_Ops_Org"),
        ],
    );
    write_basic_raw_export(
        &org_one_raw,
        "1",
        "Main Org.",
        "cpu-main",
        "CPU Main",
        "prom-main",
        "prometheus",
        "timeseries",
        "general",
        "General",
        "expr",
        "up",
    );
    write_basic_raw_export(
        &org_two_raw,
        "2",
        "Ops Org",
        "logs-main",
        "Logs Main",
        "loki-main",
        "loki",
        "logs",
        "ops",
        "Ops",
        "expr",
        "{job=\"grafana\"} |= \"error\"",
    );

    let mut dry_run_args = make_import_args(export_root.clone());
    dry_run_args.common = make_basic_common_args("http://127.0.0.1:3000".to_string());
    dry_run_args.use_export_org = true;
    dry_run_args.dry_run = true;
    dry_run_args.json = true;

    let dry_run_payload: Value = serde_json::from_str(
        &test_support::import::build_routed_import_dry_run_json_with_request(
            |method, path, _params, _payload| match (method, path) {
                (reqwest::Method::GET, "/api/orgs") => Ok(Some(json!([
                    {"id": 1, "name": "Main Org."},
                    {"id": 2, "name": "Ops Org"}
                ]))),
                _ => Err(test_support::message(format!("unexpected request {path}"))),
            },
            |_target_org_id, scoped_args| {
                Ok(test_support::import::ImportDryRunReport {
                    mode: "create-only".to_string(),
                    import_dir: scoped_args.import_dir.clone(),
                    folder_statuses: Vec::new(),
                    dashboard_records: Vec::new(),
                    skipped_missing_count: 0,
                    skipped_folder_mismatch_count: 0,
                })
            },
            &dry_run_args,
        )
        .unwrap(),
    )
    .unwrap();
    let dry_run_org_entry = dry_run_payload["orgs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(2))
        .unwrap()
        .clone();
    let dry_run_import_entry = dry_run_payload["imports"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(2))
        .unwrap()
        .clone();

    let mut live_args = make_import_args(export_root);
    live_args.common = make_basic_common_args("http://127.0.0.1:3000".to_string());
    live_args.use_export_org = true;
    live_args.dry_run = false;

    let error = test_support::import::import_dashboards_by_export_org_with_request(
        |method: reqwest::Method,
         path: &str,
         _params: &[(String, String)],
         _payload: Option<&Value>| match (method.clone(), path) {
            (reqwest::Method::GET, "/api/orgs") => Ok(Some(json!([
                {"id": 1, "name": "Main Org."},
                {"id": 2, "name": "Ops Org"}
            ]))),
            _ => Err(test_support::message(format!(
                "unexpected admin request {method} {path}"
            ))),
        },
        move |target_org_id, scoped_args| {
            let preflight_datasources = if target_org_id == 1 {
                json!([
                    {"uid":"prom-main","name":"prom-main","type":"prometheus"}
                ])
            } else {
                json!([
                    {"uid":"other","name":"other","type":"prometheus"}
                ])
            };
            let preflight_plugins = if target_org_id == 1 {
                json!([
                    {"id":"timeseries"}
                ])
            } else {
                json!([
                    {"id":"logs"}
                ])
            };
            import_dashboards_with_request(
                with_dashboard_import_live_preflight(
                    preflight_datasources,
                    preflight_plugins,
                    |_method, path, _params, _payload| match path {
                        "/api/dashboards/db" => Ok(Some(json!({"status":"success"}))),
                        _ if path.starts_with("/api/dashboards/uid/") => Ok(None),
                        _ => Err(test_support::message(format!(
                            "unexpected scoped path {path}"
                        ))),
                    },
                ),
                scoped_args,
            )
        },
        |_target_org_id, _scoped_args| unreachable!("dry-run collector should not be used"),
        &live_args,
    )
    .unwrap_err();

    let error_text = error.to_string();
    assert_eq!(dry_run_org_entry["sourceOrgId"], json!(2));
    assert_eq!(dry_run_org_entry["sourceOrgName"], json!("Ops Org"));
    assert_eq!(dry_run_org_entry["orgAction"], json!("exists"));
    assert_eq!(dry_run_org_entry["targetOrgId"], json!(2));
    assert_eq!(
        dry_run_org_entry["importDir"],
        json!(org_two_raw.display().to_string())
    );
    assert_eq!(dry_run_import_entry["sourceOrgId"], json!(2));
    assert_eq!(dry_run_import_entry["sourceOrgName"], json!("Ops Org"));
    assert_eq!(dry_run_import_entry["orgAction"], json!("exists"));
    assert_eq!(dry_run_import_entry["targetOrgId"], json!(2));
    assert!(error_text.contains("Dashboard routed import failed for export orgId=2"));
    assert!(error_text.contains("name=Ops Org"));
    assert!(error_text.contains("orgAction=exists"));
    assert!(error_text.contains("targetOrgId=2"));
    assert!(error_text.contains(&org_two_raw.display().to_string()));
}

#[test]
fn import_dashboards_rejects_mismatched_export_org_with_explicit_org_id() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("index.json"),
        serde_json::to_string_pretty(&json!([
            {
                "uid": "abc",
                "title": "CPU",
                "path": "dash.json",
                "format": "grafana-web-import-preserve-uid",
                "org": "Main Org.",
                "orgId": "1"
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": 7, "uid": "abc", "title": "CPU", "schemaVersion": 38}
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: Some(2),
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: false,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: true,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: true,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };

    let error = import_dashboards_with_request(|_method, _path, _params, _payload| Ok(None), &args)
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("raw export orgId 1 does not match target org 2"));
}

#[test]
fn import_dashboards_rejects_mismatched_export_org_with_current_token_org() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("index.json"),
        serde_json::to_string_pretty(&json!([
            {
                "uid": "abc",
                "title": "CPU",
                "path": "dash.json",
                "format": "grafana-web-import-preserve-uid",
                "org": "Main Org.",
                "orgId": "1"
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": 7, "uid": "abc", "title": "CPU", "schemaVersion": 38}
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: false,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: true,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: true,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };

    let error = import_dashboards_with_request(
        |_method, path, _params, _payload| match path {
            "/api/org" => Ok(Some(json!({"id": 2, "name": "Ops Org"}))),
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &args,
    )
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("raw export orgId 1 does not match target org 2"));
}

#[test]
fn import_dashboards_allows_matching_export_org_with_current_org_lookup() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("index.json"),
        serde_json::to_string_pretty(&json!([
            {
                "uid": "abc",
                "title": "CPU",
                "path": "dash.json",
                "format": "grafana-web-import-preserve-uid",
                "org": "Main Org.",
                "orgId": "2"
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": 7, "uid": "abc", "title": "CPU", "schemaVersion": 38}
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: false,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: true,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: true,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };
    let mut calls = Vec::new();

    let count = import_dashboards_with_request(
        |method, path, _params, _payload| {
            calls.push(format!("{} {}", method.as_str(), path));
            match (method, path) {
                (reqwest::Method::GET, "/api/search") => Ok(Some(json!([]))),
                (reqwest::Method::GET, "/api/org") => Ok(Some(json!({"id": 2, "name": "Ops Org"}))),
                (reqwest::Method::GET, "/api/dashboards/uid/abc") => Err(api_response(
                    404,
                    "http://127.0.0.1:3000/api/dashboards/uid/abc",
                    "{\"message\":\"not found\"}",
                )),
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            }
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert!(calls.contains(&"GET /api/org".to_string()));
    assert!(calls.contains(&"GET /api/search".to_string()));
}

#[test]
fn import_dashboards_with_dry_run_skips_post_requests() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": 7, "uid": "abc", "title": "CPU"}
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: true,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: true,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };

    let count = import_dashboards_with_request(
        with_dashboard_import_live_preflight(
            json!([]),
            json!([]),
            |_method, path, _params, _payload| match path {
                "/api/dashboards/uid/abc" => Ok(Some(json!({
                    "dashboard": {"id": 7, "uid": "abc", "title": "CPU"},
                    "meta": {"folderUid": "old-folder"}
                }))),
                "/api/folders/old-folder" => Ok(None),
                "/api/dashboards/db" => {
                    Err(test_support::message("dry-run must not post dashboards"))
                }
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            },
        ),
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
}

#[test]
fn import_dashboards_rejects_missing_dependencies_before_dashboard_lookup() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "id": 7,
                "uid": "abc",
                "title": "CPU",
                "schemaVersion": 38,
                "panels": [
                    {
                        "type": "row",
                        "panels": [
                            {
                                "type": "timeseries",
                                "datasource": {
                                    "uid": "prom-main",
                                    "name": "Prometheus Main"
                                }
                            }
                        ]
                    }
                ]
            }
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: false,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: false,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };
    let mut calls = Vec::new();

    let error = import_dashboards_with_request(
        |method, path, _params, _payload| {
            calls.push(format!("{} {}", method.as_str(), path));
            match path {
                "/api/org" => Ok(Some(json!({"id": 1, "name": "Main Org."}))),
                "/api/datasources" => Ok(Some(json!([
                    {"uid": "other", "name": "Other", "type": "loki"}
                ]))),
                "/api/plugins" => Ok(Some(json!([
                    {"id": "row"}
                ]))),
                "/api/dashboards/db" => Err(test_support::message("preflight should block POST")),
                "/api/dashboards/uid/abc" => {
                    Err(test_support::message("preflight should block lookup"))
                }
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            }
        },
        &args,
    )
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("Refusing dashboard import because preflight reports"));
    assert_eq!(
        calls,
        vec![
            "GET /api/datasources".to_string(),
            "GET /api/plugins".to_string()
        ]
    );
}

#[test]
fn import_dashboards_skips_dependency_preflight_for_dependency_free_dashboards() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "id": 7,
                "uid": "abc",
                "title": "CPU",
                "schemaVersion": 38
            }
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: false,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: false,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };
    let mut calls = Vec::new();

    let count = import_dashboards_with_request(
        |method, path, _params, payload| {
            calls.push(format!("{} {}", method.as_str(), path));
            match (method, path) {
                (reqwest::Method::POST, "/api/dashboards/db") => {
                    let payload = payload.cloned().unwrap();
                    assert_eq!(payload["dashboard"]["uid"], "abc");
                    Ok(Some(json!({"status": "success"})))
                }
                (reqwest::Method::GET, "/api/datasources")
                | (reqwest::Method::GET, "/api/plugins")
                | (reqwest::Method::GET, "/api/dashboards/uid/abc") => {
                    Err(test_support::message(format!("unexpected lookup {path}")))
                }
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            }
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert_eq!(calls, vec!["POST /api/dashboards/db".to_string()]);
}

#[test]
fn import_dashboards_with_shared_folder_lookup_reuses_folder_fetch_in_dry_run() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 2,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("index.json"),
        serde_json::to_string_pretty(&json!([
            {
                "uid": "abc",
                "title": "CPU",
                "path": "dash-a.json",
                "format": "grafana-web-import-preserve-uid"
            },
            {
                "uid": "def",
                "title": "Memory",
                "path": "dash-b.json",
                "format": "grafana-web-import-preserve-uid"
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash-a.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "id": 7,
                "uid": "abc",
                "title": "CPU"
            },
            "meta": {
                "folderUid": "old-folder"
            }
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash-b.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "id": 8,
                "uid": "def",
                "title": "Memory"
            },
            "meta": {
                "folderUid": "old-folder"
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let mut args = make_import_args(raw_dir);
    args.table = true;
    let calls: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let calls_for_request = Rc::clone(&calls);

    let count = import_dashboards_with_request(
        move |method, path, _params, _payload| {
            calls_for_request
                .borrow_mut()
                .push(format!("{} {}", method.as_str(), path));
            match (method, path) {
                (reqwest::Method::GET, "/api/search") => Ok(Some(json!([]))),
                (reqwest::Method::GET, "/api/datasources") => Ok(Some(json!([]))),
                (reqwest::Method::GET, "/api/plugins") => Ok(Some(json!([]))),
                (reqwest::Method::GET, "/api/dashboards/uid/abc") => Err(api_response(
                    404,
                    "http://127.0.0.1:3000/api/dashboards/uid/abc",
                    "{\"message\":\"not found\"}",
                )),
                (reqwest::Method::GET, "/api/dashboards/uid/def") => Err(api_response(
                    404,
                    "http://127.0.0.1:3000/api/dashboards/uid/def",
                    "{\"message\":\"not found\"}",
                )),
                (reqwest::Method::GET, "/api/folders/old-folder") => Ok(Some(json!({
                    "uid": "old-folder",
                    "title": "Old Folder",
                    "parents": []
                }))),
                (reqwest::Method::POST, "/api/dashboards/db") => {
                    Err(test_support::message("dry-run must not post dashboards"))
                }
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            }
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 2);
    assert_eq!(
        calls
            .borrow()
            .iter()
            .filter(|entry| entry.as_str() == "GET /api/folders/old-folder")
            .count(),
        1
    );
}

#[test]
fn import_dashboards_with_dry_run_summary_skips_unneeded_folder_lookup() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "id": 7,
                "uid": "abc",
                "title": "CPU"
            },
            "meta": {
                "folderUid": "old-folder"
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let mut args = make_import_args(raw_dir);
    args.replace_existing = true;
    args.dry_run = true;

    let calls: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let calls_for_request = Rc::clone(&calls);

    let count = import_dashboards_with_request(
        move |method, path, _params, _payload| {
            calls_for_request
                .borrow_mut()
                .push(format!("{} {}", method.as_str(), path));
            match (method, path) {
                (reqwest::Method::GET, "/api/search") => Ok(Some(json!([]))),
                (reqwest::Method::GET, "/api/dashboards/uid/abc") => Ok(Some(json!({
                    "dashboard": {"id": 7, "uid": "abc", "title": "CPU"},
                    "meta": {"folderUid": "new-folder"}
                }))),
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            }
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert_eq!(calls.borrow().as_slice(), ["GET /api/search"]);
}

#[test]
fn import_dashboards_with_dry_run_uses_search_summary_for_missing_action_lookup() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 2,
            "indexFile": "index.json"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash-a.json"),
        serde_json::to_string_pretty(&json!({"dashboard":{"uid":"abc","title":"CPU"}})).unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash-b.json"),
        serde_json::to_string_pretty(&json!({"dashboard":{"uid":"def","title":"Mem"}})).unwrap(),
    )
    .unwrap();

    let args = make_import_args(raw_dir);
    let calls: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let calls_for_request = Rc::clone(&calls);

    let count = import_dashboards_with_request(
        move |_method, path, _params, _payload| {
            calls_for_request.borrow_mut().push(format!("GET {path}"));
            match path {
                "/api/search" => Ok(Some(json!([]))),
                "/api/dashboards/db" => {
                    Err(test_support::message("dry-run must not post dashboards"))
                }
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            }
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 2);
    assert_eq!(
        calls
            .borrow()
            .iter()
            .filter(|entry| entry.starts_with("GET /api/dashboards/uid/"))
            .count(),
        0
    );
    assert_eq!(
        calls
            .borrow()
            .iter()
            .filter(|entry| entry == &"GET /api/search")
            .count(),
        1
    );
}

#[test]
fn import_dashboards_with_dry_run_replace_existing_reuses_summary_folder_uid() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 2,
            "indexFile": "index.json"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash-a.json"),
        serde_json::to_string_pretty(&json!({"dashboard":{"uid":"abc","title":"CPU"}})).unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash-b.json"),
        serde_json::to_string_pretty(&json!({"dashboard":{"uid":"def","title":"Mem"}})).unwrap(),
    )
    .unwrap();

    let mut args = make_import_args(raw_dir);
    args.replace_existing = true;
    let calls: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let calls_for_request = Rc::clone(&calls);

    let count = import_dashboards_with_request(
        move |_method, path, _params, _payload| {
            calls_for_request.borrow_mut().push(format!("GET {path}"));
            match path {
                "/api/search" => Ok(Some(json!([
                    {"uid":"abc","folderUid":"folder-a"},
                    {"uid":"def","folderUid":"folder-b"}
                ]))),
                "/api/dashboards/db" => {
                    Err(test_support::message("dry-run must not post dashboards"))
                }
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            }
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 2);
    assert_eq!(
        calls
            .borrow()
            .iter()
            .filter(|entry| entry.starts_with("GET /api/dashboards/uid/"))
            .count(),
        0
    );
}

#[test]
fn import_dashboards_with_matching_dependencies_posts_after_preflight() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "id": 7,
                "uid": "abc",
                "title": "CPU",
                "schemaVersion": 38,
                "panels": [
                    {
                        "type": "row",
                        "panels": [
                            {
                                "type": "timeseries",
                                "datasource": {
                                    "uid": "prom-main",
                                    "name": "Prometheus Main"
                                }
                            }
                        ]
                    }
                ]
            }
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: false,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: false,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };
    let mut calls = Vec::new();
    let mut posted_payloads = Vec::new();

    let count = import_dashboards_with_request(
        with_dashboard_import_live_preflight(
            json!([{"uid":"prom-main","name":"Prometheus Main","type":"prometheus"}]),
            json!([{ "id": "row" }, { "id": "timeseries" }]),
            |method, path, _params, payload| {
                calls.push(format!("{} {}", method.as_str(), path));
                match (method, path) {
                    (reqwest::Method::GET, "/api/search") => Ok(Some(json!([]))),
                    (reqwest::Method::GET, "/api/org") => {
                        Ok(Some(json!({"id": 1, "name": "Main Org."})))
                    }
                    (reqwest::Method::GET, "/api/dashboards/uid/abc") => Err(api_response(
                        404,
                        "http://127.0.0.1:3000/api/dashboards/uid/abc",
                        "{\"message\":\"not found\"}",
                    )),
                    (reqwest::Method::POST, "/api/dashboards/db") => {
                        posted_payloads.push(payload.cloned().unwrap());
                        Ok(Some(json!({"status": "success"})))
                    }
                    _ => Err(test_support::message(format!("unexpected path {path}"))),
                }
            },
        ),
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert_eq!(calls, vec!["POST /api/dashboards/db".to_string()]);
    assert_eq!(posted_payloads.len(), 1);
    assert_eq!(posted_payloads[0]["dashboard"]["uid"], "abc");
}

#[test]
fn import_dashboards_rejects_unsupported_export_schema_version() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION + 1,
            "variant": "raw",
            "dashboardCount": 0,
            "indexFile": "index.json"
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: false,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: false,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };

    let error = import_dashboards_with_request(|_method, _path, _params, _payload| Ok(None), &args)
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("Unsupported dashboard export schemaVersion"));
}

#[test]
fn import_dashboards_with_update_existing_only_skips_missing_dashboards() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 2,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("exists.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": 7, "uid": "abc", "title": "CPU", "schemaVersion": 38}
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("missing.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": 8, "uid": "xyz", "title": "Memory", "schemaVersion": 38}
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: false,
        update_existing_only: true,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: false,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };
    let mut posted_payloads = Vec::new();
    let mut calls = Vec::new();
    let count = import_dashboards_with_request(
        |method, path, _params, payload| {
            calls.push(format!("{} {}", method.as_str(), path));
            match (method, path) {
                (reqwest::Method::GET, "/api/datasources") => Ok(Some(json!([]))),
                (reqwest::Method::GET, "/api/plugins") => Ok(Some(json!([]))),
                (reqwest::Method::GET, "/api/search") => {
                    Ok(Some(json!([{"uid": "abc", "folderUid": "source-folder"}])))
                }
                (reqwest::Method::GET, "/api/dashboards/uid/abc") => Ok(Some(json!({
                    "dashboard": {"id": 7, "uid": "abc", "title": "CPU"}
                }))),
                (reqwest::Method::GET, "/api/dashboards/uid/xyz") => Err(api_response(
                    404,
                    "http://127.0.0.1:3000/api/dashboards/uid/xyz",
                    "{\"message\":\"not found\"}",
                )),
                (reqwest::Method::POST, "/api/dashboards/db") => {
                    posted_payloads.push(payload.cloned().unwrap());
                    Ok(Some(json!({"status": "success"})))
                }
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            }
        },
        &args,
    )
    .unwrap();

    assert_eq!(
        count, 1,
        "calls: {:?}, posted: {:?}",
        calls, posted_payloads
    );
    assert_eq!(posted_payloads.len(), 1);
    assert_eq!(posted_payloads[0]["dashboard"]["uid"], "abc");
    assert_eq!(posted_payloads[0]["overwrite"], true);
}

#[test]
fn import_dashboards_with_update_existing_only_table_marks_missing_dashboards_as_skipped() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("missing.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": 8, "uid": "xyz", "title": "Memory"}
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: false,
        update_existing_only: true,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: true,
        table: true,
        json: false,
        output_format: None,
        no_header: true,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };

    let count = import_dashboards_with_request(
        |method, path, _params, _payload| match (method, path) {
            (reqwest::Method::GET, "/api/search") => Ok(Some(json!([]))),
            (reqwest::Method::GET, "/api/dashboards/uid/xyz") => Err(api_response(
                404,
                "http://127.0.0.1:3000/api/dashboards/uid/xyz",
                "{\"message\":\"not found\"}",
            )),
            (reqwest::Method::POST, "/api/dashboards/db") => {
                Err(test_support::message("dry-run must not post dashboards"))
            }
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
}

#[test]
fn import_dashboards_replace_existing_preserves_destination_folder() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("exists.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": 7, "uid": "abc", "title": "CPU", "schemaVersion": 38},
            "meta": {"folderUid": "source-folder"}
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: true,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: false,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };
    let mut posted_payloads = Vec::new();
    let count = import_dashboards_with_request(
        |method, path, _params, payload| match (method, path) {
            (reqwest::Method::GET, "/api/datasources") => Ok(Some(json!([]))),
            (reqwest::Method::GET, "/api/plugins") => Ok(Some(json!([]))),
            (reqwest::Method::GET, "/api/search") => {
                Ok(Some(json!([{"uid":"abc","folderUid":"dest-folder"}])))
            }
            (reqwest::Method::GET, "/api/dashboards/uid/abc") => Ok(Some(json!({
                "dashboard": {"id": 7, "uid": "abc", "title": "CPU", "schemaVersion": 38},
                "meta": {"folderUid": "dest-folder"}
            }))),
            (reqwest::Method::POST, "/api/dashboards/db") => {
                posted_payloads.push(payload.cloned().unwrap());
                Ok(Some(json!({"status": "success"})))
            }
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert_eq!(posted_payloads.len(), 1);
    assert_eq!(posted_payloads[0]["folderUid"], "dest-folder");
    assert_eq!(posted_payloads[0]["overwrite"], true);
}

#[test]
fn import_dashboards_rejects_ensure_folders_with_import_folder_override() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": 7, "uid": "abc", "title": "CPU"},
            "meta": {"folderUid": "child"}
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: Some("override-folder".to_string()),
        ensure_folders: true,
        replace_existing: false,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: false,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };

    let error = import_dashboards_with_request(|_method, _path, _params, _payload| Ok(None), &args)
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("--ensure-folders cannot be combined with --import-folder-uid"));
}

#[test]
fn import_dashboards_rejects_matching_folder_path_with_import_folder_uid() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": 7, "uid": "abc", "title": "CPU"}
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: Some("override-folder".to_string()),
        ensure_folders: false,
        replace_existing: true,
        update_existing_only: false,
        require_matching_folder_path: true,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: false,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };

    let error = import_dashboards_with_request(|_method, _path, _params, _payload| Ok(None), &args)
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("--require-matching-folder-path cannot be combined with --import-folder-uid"));
}

#[test]
fn render_import_dry_run_table_includes_source_and_destination_folder_path_columns() {
    let records = vec![[
        "abc".to_string(),
        "exists".to_string(),
        "skip-folder-mismatch".to_string(),
        "Platform / Ops".to_string(),
        "Platform / Source".to_string(),
        "Platform / Ops".to_string(),
        "path".to_string(),
        "/tmp/raw/dash.json".to_string(),
    ]];

    let lines = render_import_dry_run_table(&records, true, None);

    assert!(lines[0].contains("SOURCE_FOLDER_PATH"));
    assert!(lines[0].contains("DESTINATION_FOLDER_PATH"));
    assert!(lines[0].contains("REASON"));
    assert!(lines[2].contains("Platform / Source"));
    assert!(lines[2].contains("Platform / Ops"));
    assert!(lines[2].contains("path"));
}

#[test]
fn render_import_dry_run_json_reports_skipped_folder_mismatch_dashboards() {
    let records = vec![[
        "abc".to_string(),
        "exists".to_string(),
        "skip-folder-mismatch".to_string(),
        "Platform / Ops".to_string(),
        "Platform / Source".to_string(),
        "Platform / Ops".to_string(),
        "path".to_string(),
        "/tmp/raw/dash.json".to_string(),
    ]];

    let payload = render_import_dry_run_json(
        "create-or-update",
        &[],
        &records,
        Path::new("/tmp/raw"),
        0,
        1,
    )
    .unwrap();
    let value: Value = serde_json::from_str(&payload).unwrap();

    assert_eq!(
        value["dashboards"][0]["sourceFolderPath"],
        Value::String("Platform / Source".to_string())
    );
    assert_eq!(
        value["dashboards"][0]["destinationFolderPath"],
        Value::String("Platform / Ops".to_string())
    );
    assert_eq!(
        value["dashboards"][0]["reason"],
        Value::String("path".to_string())
    );
    assert_eq!(
        value["summary"]["skippedFolderMismatchDashboards"],
        Value::from(1)
    );
}

#[test]
fn collect_import_dry_run_report_uses_export_folder_inventory_for_target_paths() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(raw_dir.join("Platform")).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid",
            "foldersFile": "folders.json"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join(FOLDER_INVENTORY_FILENAME),
        serde_json::to_string_pretty(&json!([
            {
                "uid": "child",
                "title": "Child",
                "path": "Platform / Child",
                "parentUid": "platform",
                "org": "Main Org.",
                "orgId": "1"
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": 7, "uid": "abc", "title": "CPU"},
            "meta": {"folderUid": "child"}
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: false,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: true,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };
    let mut calls = Vec::new();

    let report = test_support::import::collect_import_dry_run_report_with_request(
        |method: reqwest::Method,
         path: &str,
         _params: &[(String, String)],
         _payload: Option<&Value>| {
            calls.push(format!("{} {}", method.as_str(), path));
            match (method, path) {
                (reqwest::Method::GET, "/api/search") => Ok(Some(json!([]))),
                (reqwest::Method::GET, "/api/dashboards/uid/abc") => Err(api_response(
                    404,
                    "http://127.0.0.1:3000/api/dashboards/uid/abc",
                    "{\"message\":\"not found\"}",
                )),
                (reqwest::Method::GET, "/api/folders/child") => Err(test_support::message(
                    "dry-run should use exported folder inventory",
                )),
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            }
        },
        &args,
    )
    .unwrap();

    assert_eq!(calls, vec!["GET /api/search"]);
    assert_eq!(report.dashboard_records.len(), 1);
    assert_eq!(report.dashboard_records[0][3], "Platform / Child");
}

#[test]
fn collect_import_dry_run_report_prefers_live_folder_path_for_existing_dashboard_updates() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(raw_dir.join("Platform")).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid",
            "foldersFile": "folders.json"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join(FOLDER_INVENTORY_FILENAME),
        serde_json::to_string_pretty(&json!([
            {
                "uid": "dest-folder",
                "title": "Child",
                "path": "Platform / Child",
                "parentUid": "platform",
                "org": "Main Org.",
                "orgId": "1"
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": 7, "uid": "abc", "title": "CPU"},
            "meta": {"folderUid": "source-folder"}
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: true,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: true,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };
    let mut calls = Vec::new();

    let report = test_support::import::collect_import_dry_run_report_with_request(
        |method: reqwest::Method,
         path: &str,
         _params: &[(String, String)],
         _payload: Option<&Value>| {
            calls.push(format!("{} {}", method.as_str(), path));
            match (method, path) {
                (reqwest::Method::GET, "/api/search") => {
                    Ok(Some(json!([{"uid": "abc", "folderUid": "dest-folder"}])))
                }
                (reqwest::Method::GET, "/api/dashboards/uid/abc") => Ok(Some(json!({
                    "dashboard": {"id": 7, "uid": "abc", "title": "CPU"},
                    "meta": {"folderUid": "dest-folder"}
                }))),
                (reqwest::Method::GET, "/api/folders/dest-folder") => Ok(Some(json!({
                    "uid": "dest-folder",
                    "title": "Ops",
                    "parents": [{"uid": "platform", "title": "Platform"}]
                }))),
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            }
        },
        &args,
    )
    .unwrap();

    assert!(calls.contains(&"GET /api/search".to_string()));
    assert!(calls.contains(&"GET /api/folders/dest-folder".to_string()));
    assert_eq!(report.dashboard_records.len(), 1);
    assert_eq!(report.dashboard_records[0][3], "Platform / Ops");
}

#[test]
fn import_dashboards_with_matching_folder_path_skips_live_update_mismatch() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(raw_dir.join("Platform").join("Source")).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("Platform").join("Source").join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": 7, "uid": "abc", "title": "CPU", "schemaVersion": 38},
            "meta": {"folderUid": "source-folder"}
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: true,
        update_existing_only: false,
        require_matching_folder_path: true,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: false,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };
    let mut posted_payloads = Vec::new();

    let count = import_dashboards_with_request(
        |method, path, _params, payload| match (method, path) {
            (reqwest::Method::GET, "/api/datasources") => Ok(Some(json!([]))),
            (reqwest::Method::GET, "/api/plugins") => Ok(Some(json!([]))),
            (reqwest::Method::GET, "/api/search") => {
                Ok(Some(json!([{"uid": "abc", "folderUid": "dest-folder"}])))
            }
            (reqwest::Method::GET, "/api/dashboards/uid/abc") => Ok(Some(json!({
                "dashboard": {"id": 7, "uid": "abc", "title": "CPU", "schemaVersion": 38},
                "meta": {"folderUid": "dest-folder"}
            }))),
            (reqwest::Method::GET, "/api/folders/dest-folder") => Ok(Some(json!({
                "uid": "dest-folder",
                "title": "Ops",
                "parents": [{"uid": "platform", "title": "Platform"}]
            }))),
            (reqwest::Method::POST, "/api/dashboards/db") => {
                posted_payloads.push(payload.cloned().unwrap());
                Ok(Some(json!({"status": "success"})))
            }
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 0);
    assert!(posted_payloads.is_empty());
}

#[test]
fn import_dashboards_rejects_json_without_dry_run() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: false,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: false,
        table: false,
        json: true,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };

    let error = import_dashboards_with_request(|_method, _path, _params, _payload| Ok(None), &args)
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("--json is only supported with --dry-run"));
}

#[test]
fn import_dashboards_reject_output_columns_without_table_output() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 0,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: false,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: true,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: vec!["uid".to_string()],
        progress: false,
        verbose: false,
    };

    let error = import_dashboards_with_request(|_method, _path, _params, _payload| Ok(None), &args)
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("--output-columns is only supported with --dry-run --table"));
}

#[test]
fn import_dashboards_with_ensure_folders_creates_missing_folder_chain_from_raw_inventory() {
    let temp = tempdir().unwrap();
    let root_dir = temp.path();
    let raw_dir = root_dir.join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid",
            "foldersFile": "folders.json"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join(FOLDER_INVENTORY_FILENAME),
        serde_json::to_string_pretty(&json!([
            {
                "uid": "platform",
                "title": "Platform",
                "path": "Platform",
                "org": "Main Org.",
                "orgId": "1"
            },
            {
                "uid": "child",
                "title": "Child",
                "path": "Platform / Child",
                "parentUid": "platform",
                "org": "Main Org.",
                "orgId": "1"
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": 7, "uid": "abc", "title": "CPU", "schemaVersion": 38},
            "meta": {"folderUid": "child"}
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: None,
        ensure_folders: true,
        replace_existing: false,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: false,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };
    let mut calls = Vec::new();
    let mut posted_payloads = Vec::new();

    let count = import_dashboards_with_request(
        with_dashboard_import_live_preflight(
            json!([]),
            json!([]),
            |method, path, _params, payload| {
                calls.push(format!("{} {}", method.as_str(), path));
                match (method, path) {
                    (reqwest::Method::GET, "/api/search") => Ok(Some(json!([]))),
                    (reqwest::Method::GET, "/api/folders/child") => Ok(None),
                    (reqwest::Method::GET, "/api/folders/platform") => Ok(None),
                    (reqwest::Method::POST, "/api/folders") => {
                        posted_payloads.push(payload.cloned().unwrap());
                        Ok(Some(json!({"status": "success"})))
                    }
                    (reqwest::Method::POST, "/api/dashboards/db") => {
                        posted_payloads.push(payload.cloned().unwrap());
                        Ok(Some(json!({"status": "success"})))
                    }
                    _ => Err(test_support::message(format!("unexpected path {path}"))),
                }
            },
        ),
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert_eq!(
        posted_payloads,
        vec![
            json!({"uid": "platform", "title": "Platform"}),
            json!({"uid": "child", "title": "Child", "parentUid": "platform"}),
            json!({
                "dashboard": {"id": null, "uid": "abc", "title": "CPU", "schemaVersion": 38},
                "overwrite": false,
                "message": "sync dashboards",
                "folderUid": "child"
            })
        ]
    );
    assert_eq!(
        calls,
        vec![
            "GET /api/folders/child",
            "GET /api/folders/platform",
            "POST /api/folders",
            "POST /api/folders",
            "POST /api/dashboards/db"
        ]
    );
}

#[test]
fn import_dashboards_with_dry_run_and_ensure_folders_checks_folder_inventory() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid",
            "foldersFile": "folders.json"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join(FOLDER_INVENTORY_FILENAME),
        serde_json::to_string_pretty(&json!([
            {
                "uid": "platform",
                "title": "Platform",
                "path": "Platform",
                "org": "Main Org.",
                "orgId": "1"
            },
            {
                "uid": "child",
                "title": "Child",
                "path": "Platform / Child",
                "parentUid": "platform",
                "org": "Main Org.",
                "orgId": "1"
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": 7, "uid": "abc", "title": "CPU"},
            "meta": {"folderUid": "child"}
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: None,
        ensure_folders: true,
        replace_existing: false,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: true,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };
    let mut calls = Vec::new();

    let count = import_dashboards_with_request(
        |method, path, _params, _payload| {
            calls.push(format!("{} {}", method.as_str(), path));
            match (method, path) {
                (reqwest::Method::GET, "/api/search") => Ok(Some(json!([]))),
                (reqwest::Method::GET, "/api/folders/platform") => Ok(Some(json!({
                    "uid": "platform",
                    "title": "Platform",
                    "parents": []
                }))),
                (reqwest::Method::GET, "/api/folders/child") => Ok(None),
                (reqwest::Method::POST, "/api/folders") => {
                    Err(test_support::message("dry-run must not create folders"))
                }
                (reqwest::Method::POST, "/api/dashboards/db") => {
                    Err(test_support::message("dry-run must not post dashboards"))
                }
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            }
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert_eq!(
        calls,
        vec![
            "GET /api/folders/platform",
            "GET /api/folders/child",
            "GET /api/search"
        ]
    );
}

#[test]
fn import_dashboards_with_ensure_folders_requires_inventory_manifest() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid",
            "foldersFile": "folders.json"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": 7, "uid": "abc", "title": "CPU"},
            "meta": {"folderUid": "child"}
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: raw_dir,
        import_folder_uid: None,
        ensure_folders: true,
        replace_existing: false,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        dry_run: false,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };

    let error = import_dashboards_with_request(|_method, _path, _params, _payload| Ok(None), &args)
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("Folder inventory file not found for --ensure-folders"));
}

#[test]
fn collect_folder_inventory_statuses_with_request_reports_match_mismatch_and_missing() {
    let folders = vec![
        test_support::FolderInventoryItem {
            uid: "platform".to_string(),
            title: "Platform".to_string(),
            path: "Platform".to_string(),
            parent_uid: None,
            org: "Main Org.".to_string(),
            org_id: "1".to_string(),
        },
        test_support::FolderInventoryItem {
            uid: "child".to_string(),
            title: "Child".to_string(),
            path: "Platform / Child".to_string(),
            parent_uid: Some("platform".to_string()),
            org: "Main Org.".to_string(),
            org_id: "1".to_string(),
        },
        test_support::FolderInventoryItem {
            uid: "missing".to_string(),
            title: "Missing".to_string(),
            path: "Missing".to_string(),
            parent_uid: None,
            org: "Main Org.".to_string(),
            org_id: "1".to_string(),
        },
    ];

    let statuses = test_support::collect_folder_inventory_statuses_with_request(
        &mut |method, path, _params, _payload| match (method, path) {
            (reqwest::Method::GET, "/api/folders/platform") => Ok(Some(json!({
                "uid": "platform",
                "title": "Platform",
                "parents": []
            }))),
            (reqwest::Method::GET, "/api/folders/child") => Ok(Some(json!({
                "uid": "child",
                "title": "Legacy Child",
                "parents": [{"uid": "platform", "title": "Platform"}]
            }))),
            (reqwest::Method::GET, "/api/folders/missing") => Ok(None),
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &folders,
    )
    .unwrap();

    assert_eq!(statuses[0].kind, FolderInventoryStatusKind::Matches);
    assert_eq!(statuses[1].kind, FolderInventoryStatusKind::Mismatch);
    assert_eq!(statuses[2].kind, FolderInventoryStatusKind::Missing);
}

#[test]
fn diff_dashboards_with_client_returns_zero_for_matching_dashboard() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": 7, "uid": "abc", "title": "CPU"},
            "meta": {"folderUid": "old-folder"}
        }))
        .unwrap(),
    )
    .unwrap();
    let args = DiffArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        import_dir: raw_dir,
        import_folder_uid: Some("old-folder".to_string()),
        context_lines: 3,
    };

    let count = diff_dashboards_with_request(
        |_method, path, _params, _payload| match path {
            "/api/dashboards/uid/abc" => Ok(Some(json!({
                "dashboard": {"id": 7, "uid": "abc", "title": "CPU"},
                "meta": {"folderUid": "old-folder"}
            }))),
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 0);
}

#[test]
fn diff_dashboards_with_client_detects_dashboard_difference() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join(EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": 7, "uid": "abc", "title": "CPU"}
        }))
        .unwrap(),
    )
    .unwrap();
    let args = DiffArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        import_dir: raw_dir,
        import_folder_uid: None,
        context_lines: 3,
    };

    let count = diff_dashboards_with_request(
        |_method, path, _params, _payload| match path {
            "/api/dashboards/uid/abc" => Ok(Some(json!({
                "dashboard": {"id": 7, "uid": "abc", "title": "Memory"}
            }))),
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
}
