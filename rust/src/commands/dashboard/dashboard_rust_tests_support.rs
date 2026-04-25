use super::test_support::{build_topology_document, CommonCliArgs, ImportArgs, TopologyDocument};
use crate::dashboard::DashboardImportInputFormat;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub(in crate::dashboard) type TestRequestResult = crate::common::Result<Option<Value>>;

pub(in crate::dashboard) fn make_common_args(base_url: String) -> CommonCliArgs {
    CommonCliArgs {
        color: crate::common::CliColorChoice::Auto,
        profile: None,
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

pub(in crate::dashboard) fn make_basic_common_args(base_url: String) -> CommonCliArgs {
    CommonCliArgs {
        color: crate::common::CliColorChoice::Auto,
        profile: None,
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

#[allow(dead_code)]
pub(in crate::dashboard) fn load_prompt_export_cases() -> Vec<Value> {
    serde_json::from_str(include_str!(
        "../../../../tests/fixtures/dashboard_prompt_export_cases.json"
    ))
    .unwrap()
}

pub(in crate::dashboard) fn load_inspection_analyzer_cases() -> Vec<Value> {
    serde_json::from_str(include_str!(
        "../../../../tests/fixtures/dashboard_inspection_analyzer_cases.json"
    ))
    .unwrap()
}

pub(in crate::dashboard) fn sample_topology_tui_document() -> TopologyDocument {
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
pub(in crate::dashboard) fn with_dashboard_import_live_preflight<F>(
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

pub(in crate::dashboard) fn make_import_args(input_dir: PathBuf) -> ImportArgs {
    ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        input_dir,
        local: false,
        run: None,
        run_id: None,
        input_format: DashboardImportInputFormat::Raw,
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: false,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "sync dashboards".to_string(),
        interactive: false,
        dry_run: true,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        list_columns: false,
        progress: false,
        verbose: false,
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::dashboard) fn write_basic_raw_export(
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
        raw_dir.join(super::test_support::EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": super::test_support::TOOL_SCHEMA_VERSION,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid",
            "foldersFile": super::test_support::FOLDER_INVENTORY_FILENAME,
            "datasourcesFile": super::test_support::DATASOURCE_INVENTORY_FILENAME,
            "org": org_name,
            "orgId": org_id
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        raw_dir.join(super::test_support::FOLDER_INVENTORY_FILENAME),
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
        raw_dir.join(super::test_support::DATASOURCE_INVENTORY_FILENAME),
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

#[allow(clippy::too_many_arguments)]
pub(in crate::dashboard) fn write_basic_provisioning_export(
    provisioning_dir: &Path,
    org_id: &str,
    org_name: &str,
    dashboard_uid: &str,
    dashboard_title: &str,
    datasource_uid: &str,
    datasource_type: &str,
    panel_type: &str,
    dashboard_rel_path: &str,
    query_field: &str,
    query_text: &str,
) {
    let dashboards_dir = provisioning_dir.join("dashboards");
    let dashboard_path = dashboards_dir.join(dashboard_rel_path);
    fs::create_dir_all(dashboard_path.parent().unwrap()).unwrap();
    fs::write(
        provisioning_dir.join(super::test_support::EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": super::test_support::TOOL_SCHEMA_VERSION,
            "variant": "provisioning",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-file-provisioning-dashboard",
            "org": org_name,
            "orgId": org_id
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        provisioning_dir.join("index.json"),
        serde_json::to_string_pretty(&json!([
            {
                "uid": dashboard_uid,
                "title": dashboard_title,
                "path": format!("dashboards/{dashboard_rel_path}"),
                "format": "grafana-file-provisioning-dashboard",
                "org": org_name,
                "orgId": org_id
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        dashboard_path,
        serde_json::to_string_pretty(&json!({
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
        }))
        .unwrap(),
    )
    .unwrap();
}

pub(in crate::dashboard) fn write_combined_export_root_metadata(
    export_root: &Path,
    orgs: &[(&str, &str, &str)],
) {
    fs::create_dir_all(export_root).unwrap();
    let org_entries: Vec<Value> = orgs
        .iter()
        .map(|(org_id, org_name, output_dir)| {
            json!({
                "org": org_name,
                "orgId": org_id,
                "dashboardCount": 1,
                "exportDir": output_dir
            })
        })
        .collect();
    fs::write(
        export_root.join(super::test_support::EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": super::test_support::TOOL_SCHEMA_VERSION,
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
