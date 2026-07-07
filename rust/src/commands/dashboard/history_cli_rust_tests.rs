//! Dashboard history CLI contracts and export artifact tests.
#![allow(unused_imports)]

use super::history::{
    build_dashboard_history_diff_document_with_request,
    build_dashboard_history_export_document_from_current_with_version_fetcher,
    build_dashboard_history_export_document_with_request,
    build_dashboard_history_list_document_with_request, export_dashboard_history_with_request,
    run_dashboard_history_restore,
};
use super::test_support;
use super::{
    discover_dashboard_files, CommonCliArgs, HistoryDiffArgs, HistoryExportArgs, HistoryListArgs,
    HistoryOutputFormat, HistoryRestoreArgs,
};
use crate::common::DiffOutputFormat;
use reqwest::Method;
use serde_json::{json, Value};
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tempfile::tempdir;

fn make_history_common_args() -> CommonCliArgs {
    CommonCliArgs {
        color: crate::common::CliColorChoice::Auto,
        profile: None,
        url: "http://127.0.0.1:3000".to_string(),
        api_token: Some("token".to_string()),
        username: None,
        password: None,
        prompt_password: false,
        prompt_token: false,
        timeout: 30,
        verify_ssl: false,
    }
}

fn write_history_export_artifact(path: &std::path::Path, uid: &str, version: i64, title: &str) {
    fs::write(
        path,
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-util-dashboard-history-export",
            "schemaVersion": 1,
            "toolVersion": "0.8.0-dev",
            "dashboardUid": uid,
            "currentVersion": version,
            "currentTitle": title,
            "versionCount": 1,
            "versions": [
                {
                    "version": version,
                    "created": "2026-04-07T12:00:00Z",
                    "createdBy": "ops",
                    "message": format!("Revision {version}"),
                    "dashboard": {
                        "uid": uid,
                        "title": title,
                        "version": version,
                        "panels": [
                            {
                                "id": 1,
                                "type": "graph",
                                "title": title
                            }
                        ]
                    }
                }
            ]
        }))
        .unwrap(),
    )
    .unwrap();
}

fn write_single_version_history_export_artifact(
    path: &std::path::Path,
    uid: &str,
    version: i64,
    title: &str,
) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    write_history_export_artifact(path, uid, version, title);
}

#[test]
fn dashboard_history_list_document_collects_recent_versions() {
    let document = build_dashboard_history_list_document_with_request(
        |method, path, params, _payload| match (method.clone(), path) {
            (Method::GET, "/api/dashboards/uid/cpu-main/versions") => {
                assert_eq!(
                    params,
                    vec![("limit".to_string(), "5".to_string())].as_slice()
                );
                Ok(Some(json!([
                    {
                        "version": 19,
                        "created": "2026-04-01T10:00:00Z",
                        "createdBy": "ops",
                        "message": "Tune CPU panel"
                    },
                    {
                        "version": 18,
                        "created": "2026-03-30T09:00:00Z",
                        "createdBy": "sre",
                        "message": "Add datasource override"
                    }
                ])))
            }
            _ => Err(test_support::message(format!(
                "unexpected request {method} {path}"
            ))),
        },
        "cpu-main",
        5,
    )
    .unwrap();

    assert_eq!(
        document.kind,
        crate::dashboard::history::DASHBOARD_HISTORY_LIST_KIND
    );
    assert_eq!(document.dashboard_uid, "cpu-main");
    assert_eq!(document.version_count, 2);
    assert_eq!(document.versions[0].version, 19);
    assert_eq!(document.versions[1].created_by, "sre");
}

#[test]
fn dashboard_history_restore_requires_yes_without_dry_run() {
    let args = HistoryRestoreArgs {
        common: make_history_common_args(),
        dashboard_uid: "cpu-main".to_string(),
        version: Some(17),
        prompt: false,
        dry_run: false,
        output_format: HistoryOutputFormat::Text,
        message: None,
        yes: false,
    };

    let error = run_dashboard_history_restore(
        |method, path, _params, _payload| match (method.clone(), path) {
            (Method::GET, "/api/dashboards/uid/cpu-main") => Ok(Some(json!({
                "dashboard": {"uid": "cpu-main", "title": "CPU Main", "version": 21},
                "meta": {"folderUid": "infra"}
            }))),
            (Method::GET, "/api/dashboards/uid/cpu-main/versions/17") => Ok(Some(json!({
                "data": {"uid": "cpu-main", "title": "CPU Main"}
            }))),
            _ => Err(test_support::message(format!(
                "unexpected request {method} {path}"
            ))),
        },
        &args,
    )
    .unwrap_err();

    assert!(error.to_string().contains("requires --yes"));
}

#[test]
fn dashboard_history_export_writes_json_artifact_with_dashboard_payloads() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("cpu-main.history.json");
    let args = HistoryExportArgs {
        common: make_history_common_args(),
        dashboard_uid: "cpu-main".to_string(),
        output: output.clone(),
        limit: 2,
        overwrite: false,
    };

    export_dashboard_history_with_request(
        |method, path, params, _payload| match (method.clone(), path) {
            (Method::GET, "/api/dashboards/uid/cpu-main") => Ok(Some(json!({
                "dashboard": {"uid": "cpu-main", "title": "CPU Main", "version": 21}
            }))),
            (Method::GET, "/api/dashboards/uid/cpu-main/versions") => {
                assert_eq!(
                    params,
                    vec![("limit".to_string(), "2".to_string())].as_slice()
                );
                Ok(Some(json!([
                    {
                        "version": 21,
                        "created": "2026-04-02T12:00:00Z",
                        "createdBy": "ops",
                        "message": "Tune thresholds"
                    },
                    {
                        "version": 20,
                        "created": "2026-04-01T12:00:00Z",
                        "createdBy": "sre",
                        "message": "Add region variable"
                    }
                ])))
            }
            (Method::GET, "/api/dashboards/uid/cpu-main/versions/21") => Ok(Some(json!({
                "data": {"uid": "cpu-main", "title": "CPU Main", "version": 21}
            }))),
            (Method::GET, "/api/dashboards/uid/cpu-main/versions/20") => Ok(Some(json!({
                "data": {"uid": "cpu-main", "title": "CPU Main", "version": 20}
            }))),
            _ => Err(test_support::message(format!(
                "unexpected request {method} {path}"
            ))),
        },
        &args,
    )
    .unwrap();

    let artifact: Value = serde_json::from_str(&fs::read_to_string(&output).unwrap()).unwrap();
    assert_eq!(
        artifact["kind"],
        crate::dashboard::history::DASHBOARD_HISTORY_EXPORT_KIND
    );
    assert_eq!(artifact["dashboardUid"], "cpu-main");
    assert_eq!(artifact["versionCount"], 2);
    assert_eq!(artifact["versions"][0]["dashboard"]["title"], "CPU Main");
}

#[test]
fn dashboard_history_export_fetches_version_payloads_in_parallel_and_preserves_order() {
    let current_payload = json!({
        "dashboard": {"uid": "cpu-main", "title": "CPU Main", "version": 22}
    });
    let active_fetches = AtomicUsize::new(0);
    let max_active_fetches = AtomicUsize::new(0);
    let document = build_dashboard_history_export_document_from_current_with_version_fetcher(
        |method, path, params, _payload| match (method.clone(), path) {
            (Method::GET, "/api/dashboards/uid/cpu-main/versions") => {
                assert_eq!(
                    params,
                    vec![("limit".to_string(), "3".to_string())].as_slice()
                );
                Ok(Some(json!([
                    {"version": 22, "created": "2026-04-03T12:00:00Z", "createdBy": "ops", "message": "Tune CPU"},
                    {"version": 21, "created": "2026-04-02T12:00:00Z", "createdBy": "sre", "message": "Add memory panel"},
                    {"version": 20, "created": "2026-04-01T12:00:00Z", "createdBy": "ops", "message": "Initial dashboard"}
                ])))
            }
            _ => Err(test_support::message(format!(
                "unexpected request {method} {path}"
            ))),
        },
        "cpu-main",
        3,
        &current_payload,
        |version| {
            let active = active_fetches.fetch_add(1, Ordering::SeqCst) + 1;
            max_active_fetches.fetch_max(active, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(50));
            active_fetches.fetch_sub(1, Ordering::SeqCst);
            Ok(serde_json::Map::from_iter([
                ("uid".to_string(), json!("cpu-main")),
                ("title".to_string(), json!(format!("CPU Main v{version}"))),
                ("version".to_string(), json!(version)),
            ]))
        },
        3,
    )
    .unwrap();

    assert!(max_active_fetches.load(Ordering::SeqCst) > 1);
    assert_eq!(document.current_version, 22);
    assert_eq!(document.current_title, "CPU Main");
    assert_eq!(
        document
            .versions
            .iter()
            .map(|version| version.version)
            .collect::<Vec<_>>(),
        vec![22, 21, 20]
    );
    assert_eq!(document.versions[2].dashboard["title"], "CPU Main v20");
}

#[test]
fn dashboard_history_diff_builds_json_contract_from_local_export_roots() {
    let base_dir = tempdir().unwrap();
    let new_dir = tempdir().unwrap();
    let base_root = base_dir.path().join("exports-2026-04-01/history");
    let new_root = new_dir.path().join("exports-2026-04-07/history");
    fs::create_dir_all(&base_root).unwrap();
    fs::create_dir_all(&new_root).unwrap();

    let base_path = base_root.join("cpu-main.history.json");
    let new_path = new_root.join("cpu-main.history.json");
    write_history_export_artifact(&base_path, "cpu-main", 17, "CPU Main");
    write_history_export_artifact(&new_path, "cpu-main", 21, "CPU Main");

    let args = HistoryDiffArgs {
        common: make_history_common_args(),
        base_dashboard_uid: Some("cpu-main".to_string()),
        base_input: None,
        base_input_dir: Some(base_dir.path().join("exports-2026-04-01")),
        base_version: 17,
        new_dashboard_uid: Some("cpu-main".to_string()),
        new_input: None,
        new_input_dir: Some(new_dir.path().join("exports-2026-04-07")),
        new_version: 21,
        output_format: DiffOutputFormat::Json,
        context_lines: 3,
    };

    let document = build_dashboard_history_diff_document_with_request(
        |_method, _path, _params, _payload| {
            Err(test_support::message(
                "local history diff should not call Grafana",
            ))
        },
        &args,
    )
    .unwrap();

    assert_eq!(
        document["kind"],
        json!("grafana-util-dashboard-history-diff")
    );
    assert_eq!(document["schemaVersion"], json!(1));
    assert_eq!(document["summary"]["checked"], json!(1));
    assert_eq!(document["summary"]["same"], json!(0));
    assert_eq!(document["summary"]["different"], json!(1));
    let row = document["rows"][0].as_object().unwrap();
    assert_eq!(row["domain"], json!("dashboard"));
    assert_eq!(row["resourceKind"], json!("dashboard-history"));
    assert_eq!(row["identity"], json!("cpu-main"));
    assert_eq!(row["status"], json!("different"));
    assert_eq!(row["baseVersion"], json!(17));
    assert_eq!(row["newVersion"], json!(21));
    assert_eq!(
        row["baseSource"],
        json!(base_path.display().to_string() + "@17")
    );
    assert_eq!(
        row["newSource"],
        json!(new_path.display().to_string() + "@21")
    );
    assert_eq!(row["changedFields"], json!(["dashboard"]));
    assert_eq!(row["contextLines"], json!(3));
    assert!(row["diffText"].as_str().unwrap().contains("--- "));
    assert!(row["diffText"].as_str().unwrap().contains("+++ "));
}

#[test]
fn dashboard_history_diff_builds_json_contract_from_live_versions() {
    let args = HistoryDiffArgs {
        common: make_history_common_args(),
        base_dashboard_uid: Some("cpu-main".to_string()),
        base_input: None,
        base_input_dir: None,
        base_version: 17,
        new_dashboard_uid: Some("cpu-main".to_string()),
        new_input: None,
        new_input_dir: None,
        new_version: 21,
        output_format: DiffOutputFormat::Json,
        context_lines: 2,
    };

    let document = build_dashboard_history_diff_document_with_request(
        |method, path, _params, _payload| match (method.clone(), path) {
            (Method::GET, "/api/dashboards/uid/cpu-main/versions/17") => Ok(Some(json!({
                "data": {
                    "uid": "cpu-main",
                    "title": "CPU Main",
                    "version": 17,
                    "panels": [{"id": 1, "title": "Old"}]
                }
            }))),
            (Method::GET, "/api/dashboards/uid/cpu-main/versions/21") => Ok(Some(json!({
                "data": {
                    "uid": "cpu-main",
                    "title": "CPU Main",
                    "version": 21,
                    "panels": [{"id": 1, "title": "New"}]
                }
            }))),
            _ => Err(test_support::message(format!(
                "unexpected request {method} {path}"
            ))),
        },
        &args,
    )
    .unwrap();

    assert_eq!(
        document["kind"],
        json!("grafana-util-dashboard-history-diff")
    );
    assert_eq!(document["summary"]["checked"], json!(1));
    assert_eq!(document["summary"]["different"], json!(1));
    let row = document["rows"][0].as_object().unwrap();
    assert_eq!(row["baseSource"], json!("grafana:cpu-main@17"));
    assert_eq!(row["newSource"], json!("grafana:cpu-main@21"));
    assert_eq!(row["identity"], json!("cpu-main"));
    assert_eq!(row["status"], json!("different"));
    assert_eq!(
        row["path"],
        json!("grafana:cpu-main@17 -> grafana:cpu-main@21")
    );
    assert_eq!(row["contextLines"], json!(2));
    assert!(row["diffText"]
        .as_str()
        .unwrap()
        .contains("grafana:cpu-main@17"));
    assert!(row["diffText"]
        .as_str()
        .unwrap()
        .contains("grafana:cpu-main@21"));
}

#[test]
fn dashboard_history_list_reads_single_local_artifact() {
    let temp = tempdir().unwrap();
    let artifact_path = temp.path().join("cpu-main.history.json");
    fs::write(
        &artifact_path,
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-util-dashboard-history-export",
            "schemaVersion": 1,
            "toolVersion": "0.8.0-dev",
            "dashboardUid": "cpu-main",
            "currentVersion": 21,
            "currentTitle": "CPU Main",
            "versionCount": 2,
            "versions": [
                {
                    "version": 21,
                    "created": "2026-04-02T12:00:00Z",
                    "createdBy": "ops",
                    "message": "Tune thresholds",
                    "dashboard": {"uid": "cpu-main", "title": "CPU Main", "version": 21}
                },
                {
                    "version": 20,
                    "created": "2026-04-01T12:00:00Z",
                    "createdBy": "sre",
                    "message": "Add region variable",
                    "dashboard": {"uid": "cpu-main", "title": "CPU Main", "version": 20}
                }
            ]
        }))
        .unwrap(),
    )
    .unwrap();

    let args = HistoryListArgs {
        common: make_history_common_args(),
        dashboard_uid: Some("cpu-main".to_string()),
        input: Some(artifact_path),
        input_dir: None,
        limit: 20,
        output_format: HistoryOutputFormat::Json,
    };

    super::history::run_dashboard_history_list(
        |_method, _path, _params, _payload| Err(test_support::message("should not call Grafana")),
        &args,
    )
    .unwrap();
}

#[test]
fn dashboard_history_list_rejects_mismatched_local_artifact_uid() {
    let temp = tempdir().unwrap();
    let artifact_path = temp.path().join("cpu-main.history.json");
    fs::write(
        &artifact_path,
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-util-dashboard-history-export",
            "schemaVersion": 1,
            "toolVersion": "0.8.0-dev",
            "dashboardUid": "cpu-main",
            "currentVersion": 21,
            "currentTitle": "CPU Main",
            "versionCount": 0,
            "versions": []
        }))
        .unwrap(),
    )
    .unwrap();

    let args = HistoryListArgs {
        common: make_history_common_args(),
        dashboard_uid: Some("memory-main".to_string()),
        input: Some(artifact_path),
        input_dir: None,
        limit: 20,
        output_format: HistoryOutputFormat::Json,
    };

    let error = super::history::run_dashboard_history_list(
        |_method, _path, _params, _payload| Err(test_support::message("should not call Grafana")),
        &args,
    )
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("contains dashboard UID cpu-main instead of memory-main"));
}

#[test]
fn dashboard_history_list_reads_export_tree_inventory_without_uid_filter() {
    let temp = tempdir().unwrap();
    let input_dir = temp.path().join("dashboards");
    fs::create_dir_all(input_dir.join("all-orgs/org_1_Main_Org/history")).unwrap();
    fs::create_dir_all(input_dir.join("all-orgs/org_2_Ops_Org/history")).unwrap();
    fs::write(
        input_dir.join("all-orgs/org_1_Main_Org/history/cpu-main.history.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-util-dashboard-history-export",
            "schemaVersion": 1,
            "toolVersion": "0.8.0-dev",
            "dashboardUid": "cpu-main",
            "currentVersion": 21,
            "currentTitle": "CPU Main",
            "versionCount": 2,
            "versions": []
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        input_dir.join("all-orgs/org_2_Ops_Org/history/ops-main.history.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-util-dashboard-history-export",
            "schemaVersion": 1,
            "toolVersion": "0.8.0-dev",
            "dashboardUid": "ops-main",
            "currentVersion": 12,
            "currentTitle": "Ops Main",
            "versionCount": 3,
            "versions": []
        }))
        .unwrap(),
    )
    .unwrap();

    let args = HistoryListArgs {
        common: make_history_common_args(),
        dashboard_uid: None,
        input: None,
        input_dir: Some(input_dir),
        limit: 20,
        output_format: HistoryOutputFormat::Json,
    };

    super::history::run_dashboard_history_list(
        |_method, _path, _params, _payload| Err(test_support::message("should not call Grafana")),
        &args,
    )
    .unwrap();
}

#[test]
fn dashboard_history_list_rejects_ambiguous_uid_in_export_tree() {
    let temp = tempdir().unwrap();
    let input_dir = temp.path().join("dashboards");
    fs::create_dir_all(input_dir.join("all-orgs/org_1_Main_Org/history")).unwrap();
    fs::create_dir_all(input_dir.join("all-orgs/org_2_Ops_Org/history")).unwrap();
    for path in [
        input_dir.join("all-orgs/org_1_Main_Org/history/cpu-main.history.json"),
        input_dir.join("all-orgs/org_2_Ops_Org/history/cpu-main.history.json"),
    ] {
        fs::write(
            path,
            serde_json::to_string_pretty(&json!({
                "kind": "grafana-util-dashboard-history-export",
                "schemaVersion": 1,
                "toolVersion": "0.8.0-dev",
                "dashboardUid": "cpu-main",
                "currentVersion": 21,
                "currentTitle": "CPU Main",
                "versionCount": 2,
                "versions": []
            }))
            .unwrap(),
        )
        .unwrap();
    }

    let args = HistoryListArgs {
        common: make_history_common_args(),
        dashboard_uid: Some("cpu-main".to_string()),
        input: None,
        input_dir: Some(input_dir),
        limit: 20,
        output_format: HistoryOutputFormat::Table,
    };

    let error = super::history::run_dashboard_history_list(
        |_method, _path, _params, _payload| Err(test_support::message("should not call Grafana")),
        &args,
    )
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("Multiple dashboard history artifacts for UID cpu-main"));
}

#[test]
fn dashboard_history_list_merges_per_version_files_for_same_uid_scope() {
    let temp = tempdir().unwrap();
    let input_dir = temp.path().join("dashboards");
    write_single_version_history_export_artifact(
        &input_dir.join("history/cpu-main/v21.history.json"),
        "cpu-main",
        21,
        "CPU Main",
    );
    write_single_version_history_export_artifact(
        &input_dir.join("history/cpu-main/v20.history.json"),
        "cpu-main",
        20,
        "CPU Main",
    );

    let artifacts =
        super::history::load_logical_history_artifacts_from_import_dir(&input_dir).unwrap();
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].document.version_count, 2);
    assert!(artifacts[0]
        .path
        .ends_with("history/cpu-main/v21.history.json"));
    assert_eq!(artifacts[0].document.versions[0].version, 21);
    assert_eq!(artifacts[0].document.versions[1].version, 20);

    let args = HistoryListArgs {
        common: make_history_common_args(),
        dashboard_uid: Some("cpu-main".to_string()),
        input: None,
        input_dir: Some(input_dir),
        limit: 20,
        output_format: HistoryOutputFormat::Json,
    };

    super::history::run_dashboard_history_list(
        |_method, _path, _params, _payload| Err(test_support::message("should not call Grafana")),
        &args,
    )
    .unwrap();
}

#[test]
fn dashboard_history_list_keeps_same_uid_per_version_files_separate_by_scope() {
    let temp = tempdir().unwrap();
    let input_dir = temp.path().join("dashboards");
    write_single_version_history_export_artifact(
        &input_dir.join("all-orgs/org_1_Main_Org/history/cpu-main/v21.history.json"),
        "cpu-main",
        21,
        "CPU Main",
    );
    write_single_version_history_export_artifact(
        &input_dir.join("all-orgs/org_2_Ops_Org/history/cpu-main/v4.history.json"),
        "cpu-main",
        4,
        "CPU Main",
    );

    let artifacts =
        super::history::load_logical_history_artifacts_from_import_dir(&input_dir).unwrap();
    assert_eq!(artifacts.len(), 2);
    assert_eq!(
        artifacts
            .iter()
            .map(|artifact| artifact.scope.as_deref().unwrap_or(""))
            .collect::<Vec<_>>(),
        vec!["all-orgs/org_1_Main_Org", "all-orgs/org_2_Ops_Org"]
    );

    let args = HistoryListArgs {
        common: make_history_common_args(),
        dashboard_uid: Some("cpu-main".to_string()),
        input: None,
        input_dir: Some(input_dir),
        limit: 20,
        output_format: HistoryOutputFormat::Table,
    };

    let error = super::history::run_dashboard_history_list(
        |_method, _path, _params, _payload| Err(test_support::message("should not call Grafana")),
        &args,
    )
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("Multiple dashboard history artifacts for UID cpu-main"));
}

#[test]
fn dashboard_history_diff_resolves_per_version_files_from_import_dir() {
    let temp = tempdir().unwrap();
    let input_dir = temp.path().join("dashboards");
    write_single_version_history_export_artifact(
        &input_dir.join("history/cpu-main/v21.history.json"),
        "cpu-main",
        21,
        "CPU Main New",
    );
    write_single_version_history_export_artifact(
        &input_dir.join("history/cpu-main/v20.history.json"),
        "cpu-main",
        20,
        "CPU Main Old",
    );

    let args = HistoryDiffArgs {
        common: make_history_common_args(),
        base_dashboard_uid: Some("cpu-main".to_string()),
        base_input: None,
        base_input_dir: Some(input_dir.clone()),
        base_version: 20,
        new_dashboard_uid: Some("cpu-main".to_string()),
        new_input: None,
        new_input_dir: Some(input_dir),
        new_version: 21,
        context_lines: 3,
        output_format: DiffOutputFormat::Json,
    };

    let changed = super::history::run_dashboard_history_diff(
        |_method, _path, _params, _payload| Err(test_support::message("should not call Grafana")),
        &args,
    )
    .unwrap();

    assert_eq!(changed, 1);
}

#[test]
fn discover_dashboard_files_ignores_history_export_artifacts() {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join("raw/history")).unwrap();
    fs::create_dir_all(temp.path().join("raw/general")).unwrap();
    fs::write(
        temp.path().join("raw/general/cpu-main.json"),
        serde_json::to_string_pretty(&json!({"uid": "cpu-main", "title": "CPU Main"})).unwrap(),
    )
    .unwrap();
    fs::write(
        temp.path().join("raw/history/cpu-main.history.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-util-dashboard-history-export",
            "schemaVersion": 1,
            "dashboardUid": "cpu-main",
            "versions": []
        }))
        .unwrap(),
    )
    .unwrap();

    let files = discover_dashboard_files(&temp.path().join("raw")).unwrap();
    assert_eq!(files, vec![temp.path().join("raw/general/cpu-main.json")]);
}
