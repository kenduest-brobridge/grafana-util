use super::make_common_args;
use crate::dashboard::{run_clone_folder_with_request, CloneFolderArgs, CloneFolderOutputFormat};
use reqwest::Method;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

fn not_found(path: &str) -> crate::common::GrafanaCliError {
    crate::common::api_response(404, format!("http://grafana.local{path}"), "not found")
}

fn clone_folder_args() -> CloneFolderArgs {
    CloneFolderArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        page_size: 500,
        source_folder_uid: Some("infra".to_string()),
        source_path: None,
        target_folder_uid: "staging-infra".to_string(),
        target_folder_title: Some("Staging Infra".to_string()),
        target_parent_folder_uid: None,
        create_target_folder: true,
        recursive: false,
        uid_prefix: None,
        uid_suffix: "-copy".to_string(),
        title_prefix: None,
        title_suffix: None,
        replace_existing: false,
        message: "Cloned by grafana-utils dashboard clone-folder".to_string(),
        yes: false,
        dry_run: true,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
    }
}

#[test]
fn clone_folder_dry_run_plans_direct_dashboard_copies_and_target_folder_create() {
    let args = clone_folder_args();

    let report = run_clone_folder_with_request(
        |method, path, params, _payload| match (method.clone(), path) {
            (Method::GET, "/api/search") => {
                assert!(params.iter().any(|(key, value)| key == "type" && value == "dash-db"));
                Ok(Some(json!([
                    {"uid":"cpu-main","title":"CPU Main","type":"dash-db","folderUid":"infra","folderTitle":"Infra"},
                    {"uid":"db-main","title":"DB Main","type":"dash-db","folderUid":"db","folderTitle":"DB"}
                ])))
            }
            (Method::GET, "/api/folders/infra") => Ok(Some(json!({
                "uid": "infra",
                "title": "Infra",
                "parents": []
            }))),
            (Method::GET, "/api/folders/db") => Ok(Some(json!({
                "uid": "db",
                "title": "DB",
                "parents": []
            }))),
            (Method::GET, "/api/folders/staging-infra") => Ok(None),
            (Method::GET, "/api/dashboards/uid/cpu-main-copy") => {
                Err(not_found("/api/dashboards/uid/cpu-main-copy"))
            }
            _ => Err(crate::common::message(format!(
                "unexpected request {method} {path}"
            ))),
        },
        &args,
    )
    .unwrap();

    assert_eq!(report.folder_actions.len(), 1);
    assert_eq!(report.folder_actions[0].action, "create");
    assert_eq!(report.folder_actions[0].target_uid, "staging-infra");
    assert_eq!(report.dashboard_actions.len(), 1);
    assert_eq!(report.dashboard_actions[0].source_uid, "cpu-main");
    assert_eq!(report.dashboard_actions[0].target_uid, "cpu-main-copy");
    assert_eq!(
        report.dashboard_actions[0].target_folder_uid,
        "staging-infra"
    );
    assert_eq!(report.dashboard_actions[0].action, "create");
}

#[test]
fn clone_folder_create_target_folder_requires_explicit_target_title() {
    let mut args = clone_folder_args();
    args.target_folder_title = None;

    let error = run_clone_folder_with_request(
        |_method, _path, _params, _payload| {
            Err(crate::common::message(
                "request should not run before validation",
            ))
        },
        &args,
    )
    .unwrap_err();

    assert!(error.to_string().contains("--target-folder-title"));
}

#[test]
fn clone_folder_treats_target_folder_and_dashboard_404_as_missing() {
    let args = clone_folder_args();

    let report = run_clone_folder_with_request(
        |method, path, _params, _payload| match (method.clone(), path) {
            (Method::GET, "/api/search") => Ok(Some(json!([
                {"uid":"cpu-main","title":"CPU Main","type":"dash-db","folderUid":"infra","folderTitle":"Infra"}
            ]))),
            (Method::GET, "/api/folders/infra") => Ok(Some(json!({
                "uid": "infra",
                "title": "Infra",
                "parents": []
            }))),
            (Method::GET, "/api/folders/staging-infra") => {
                Err(not_found("/api/folders/staging-infra"))
            }
            (Method::GET, "/api/dashboards/uid/cpu-main-copy") => {
                Err(not_found("/api/dashboards/uid/cpu-main-copy"))
            }
            _ => Err(crate::common::message(format!(
                "unexpected request {method} {path}"
            ))),
        },
        &args,
    )
    .unwrap();

    assert_eq!(report.folder_actions[0].action, "create");
    assert_eq!(report.dashboard_actions[0].action, "create");
}

#[test]
fn clone_folder_existing_target_folder_does_not_require_title_to_match_source() {
    let mut args = clone_folder_args();
    args.target_folder_title = None;
    args.create_target_folder = false;

    let report = run_clone_folder_with_request(
        |method, path, _params, _payload| match (method.clone(), path) {
            (Method::GET, "/api/search") => Ok(Some(json!([
                {"uid":"cpu-main","title":"CPU Main","type":"dash-db","folderUid":"infra","folderTitle":"Infra"}
            ]))),
            (Method::GET, "/api/folders/infra") => Ok(Some(json!({
                "uid": "infra",
                "title": "Infra",
                "parents": []
            }))),
            (Method::GET, "/api/folders/staging-infra") => Ok(Some(json!({
                "uid": "staging-infra",
                "title": "Staging Infra",
                "parents": []
            }))),
            (Method::GET, "/api/dashboards/uid/cpu-main-copy") => {
                Err(not_found("/api/dashboards/uid/cpu-main-copy"))
            }
            _ => Err(crate::common::message(format!(
                "unexpected request {method} {path}"
            ))),
        },
        &args,
    )
    .unwrap();

    assert!(report.folder_actions.is_empty());
    assert_eq!(
        report.dashboard_actions[0].target_folder_uid,
        "staging-infra"
    );
    assert_eq!(report.dashboard_actions[0].action, "create");
}

#[test]
fn clone_folder_blocks_target_folder_parent_mismatch() {
    let mut args = clone_folder_args();
    args.target_parent_folder_uid = Some("expected-parent".to_string());

    let report = run_clone_folder_with_request(
        |method, path, _params, _payload| match (method.clone(), path) {
            (Method::GET, "/api/search") => Ok(Some(json!([
                {"uid":"cpu-main","title":"CPU Main","type":"dash-db","folderUid":"infra","folderTitle":"Infra"}
            ]))),
            (Method::GET, "/api/folders/infra") => Ok(Some(json!({
                "uid": "infra",
                "title": "Infra",
                "parents": []
            }))),
            (Method::GET, "/api/folders/staging-infra") => Ok(Some(json!({
                "uid": "staging-infra",
                "title": "Staging Infra",
                "parents": [{"uid":"other-parent","title":"Other Parent"}]
            }))),
            (Method::GET, "/api/dashboards/uid/cpu-main-copy") => {
                Err(not_found("/api/dashboards/uid/cpu-main-copy"))
            }
            _ => Err(crate::common::message(format!(
                "unexpected request {method} {path}"
            ))),
        },
        &args,
    )
    .unwrap();

    assert_eq!(report.folder_actions.len(), 1);
    assert_eq!(report.folder_actions[0].action, "blocked");
    assert_eq!(
        report.folder_actions[0].reason.as_deref(),
        Some("target-folder-mismatch")
    );
}

#[test]
fn clone_folder_apply_creates_folder_then_imports_cloned_dashboard() {
    let mut args = clone_folder_args();
    args.dry_run = false;
    args.yes = true;

    let requests = Arc::new(Mutex::new(Vec::<String>::new()));
    let payloads = Arc::new(Mutex::new(Vec::<Value>::new()));
    let recorded_requests = requests.clone();
    let recorded_payloads = payloads.clone();

    run_clone_folder_with_request(
        move |method, path, _params, payload| {
            recorded_requests
                .lock()
                .unwrap()
                .push(format!("{method} {path}"));
            match (method.clone(), path) {
                (Method::GET, "/api/search") => Ok(Some(json!([
                    {"uid":"cpu-main","title":"CPU Main","type":"dash-db","folderUid":"infra","folderTitle":"Infra"}
                ]))),
                (Method::GET, "/api/folders/infra") => Ok(Some(json!({
                    "uid": "infra",
                    "title": "Infra",
                    "parents": []
                }))),
                (Method::GET, "/api/folders/staging-infra") => Ok(None),
                (Method::GET, "/api/dashboards/uid/cpu-main-copy") => {
                    Err(not_found("/api/dashboards/uid/cpu-main-copy"))
                }
                (Method::GET, "/api/dashboards/uid/cpu-main") => Ok(Some(json!({
                    "dashboard": {
                        "id": 7,
                        "uid": "cpu-main",
                        "title": "CPU Main",
                        "schemaVersion": 38,
                        "panels": []
                    },
                    "meta": {"folderUid": "infra"}
                }))),
                (Method::POST, "/api/folders") | (Method::POST, "/api/dashboards/db") => {
                    recorded_payloads
                        .lock()
                        .unwrap()
                        .push(payload.cloned().unwrap_or(Value::Null));
                    Ok(Some(json!({"status": "success"})))
                }
                _ => Err(crate::common::message(format!(
                    "unexpected request {method} {path}"
                ))),
            }
        },
        &args,
    )
    .unwrap();

    let requests = requests.lock().unwrap();
    let folder_post = requests
        .iter()
        .position(|item| item == "POST /api/folders")
        .unwrap();
    let dashboard_post = requests
        .iter()
        .position(|item| item == "POST /api/dashboards/db")
        .unwrap();
    assert!(folder_post < dashboard_post);

    let payloads = payloads.lock().unwrap();
    assert_eq!(payloads[0]["uid"], "staging-infra");
    assert_eq!(payloads[0]["title"], "Staging Infra");
    assert_eq!(payloads[1]["dashboard"]["id"], Value::Null);
    assert_eq!(payloads[1]["dashboard"]["uid"], "cpu-main-copy");
    assert_eq!(payloads[1]["dashboard"]["title"], "CPU Main");
    assert_eq!(payloads[1]["folderUid"], "staging-infra");
    assert_eq!(payloads[1]["overwrite"], false);
}

#[test]
fn clone_folder_recursive_maps_child_folder_and_blocks_existing_dashboard_by_default() {
    let mut args = clone_folder_args();
    args.recursive = true;

    let report = run_clone_folder_with_request(
        |method, path, _params, _payload| match (method.clone(), path) {
            (Method::GET, "/api/search") => Ok(Some(json!([
                {"uid":"cpu-main","title":"CPU Main","type":"dash-db","folderUid":"infra","folderTitle":"Infra"},
                {"uid":"latency","title":"Latency","type":"dash-db","folderUid":"infra-child","folderTitle":"Child"}
            ]))),
            (Method::GET, "/api/folders/infra") => Ok(Some(json!({
                "uid": "infra",
                "title": "Infra",
                "parents": []
            }))),
            (Method::GET, "/api/folders/infra-child") => Ok(Some(json!({
                "uid": "infra-child",
                "title": "Child",
                "parents": [{"uid":"infra","title":"Infra"}]
            }))),
            (Method::GET, "/api/folders/staging-infra") => Ok(Some(json!({
                "uid": "staging-infra",
                "title": "Staging Infra",
                "parents": []
            }))),
            (Method::GET, "/api/folders/staging-infra-infra-child") => Ok(None),
            (Method::GET, "/api/dashboards/uid/cpu-main-copy") => {
                Err(not_found("/api/dashboards/uid/cpu-main-copy"))
            }
            (Method::GET, "/api/dashboards/uid/latency-copy") => Ok(Some(json!({
                "dashboard": {"uid":"latency-copy","title":"Latency"}
            }))),
            _ => Err(crate::common::message(format!(
                "unexpected request {method} {path}"
            ))),
        },
        &args,
    )
    .unwrap();

    assert_eq!(report.folder_actions.len(), 1);
    assert_eq!(
        report.folder_actions[0].target_uid,
        "staging-infra-infra-child"
    );
    assert_eq!(
        report.folder_actions[0].target_parent_uid.as_deref(),
        Some("staging-infra")
    );
    assert_eq!(report.dashboard_actions.len(), 2);
    let child = report
        .dashboard_actions
        .iter()
        .find(|action| action.source_uid == "latency")
        .unwrap();
    assert_eq!(child.target_folder_uid, "staging-infra-infra-child");
    assert_eq!(child.action, "blocked");
    assert_eq!(child.reason.as_deref(), Some("target-dashboard-exists"));
}
