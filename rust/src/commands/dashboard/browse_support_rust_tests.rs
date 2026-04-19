use super::*;
use crate::common::CliColorChoice;
use crate::dashboard::{BrowseArgs, CommonCliArgs, DashboardImportInputFormat};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

fn make_browse_args(input_dir: std::path::PathBuf) -> BrowseArgs {
    BrowseArgs {
        common: CommonCliArgs {
            color: CliColorChoice::Auto,
            profile: None,
            url: "https://grafana.example.com".to_string(),
            api_token: Some("secret".to_string()),
            username: None,
            password: None,
            prompt_password: false,
            prompt_token: false,
            timeout: 30,
            verify_ssl: false,
        },
        workspace: None,
        input_dir: Some(input_dir),
        local: false,
        run: None,
        run_id: None,
        input_format: DashboardImportInputFormat::Raw,
        page_size: 500,
        org_id: None,
        all_orgs: false,
        path: None,
    }
}

#[test]
fn local_import_dir_browse_ignores_history_artifacts() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    let dashboard_dir = raw_dir.join("Platform/Infra");
    let history_dir = raw_dir.join("history");
    fs::create_dir_all(&dashboard_dir).unwrap();
    fs::create_dir_all(&history_dir).unwrap();
    fs::write(
        dashboard_dir.join("cpu-main.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "uid": "cpu-main",
                "title": "CPU Main"
            }
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        history_dir.join("cpu-main.history.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-util-dashboard-history-export",
            "schemaVersion": 1,
            "dashboardUid": "cpu-main"
        }))
        .unwrap(),
    )
    .unwrap();

    let args = make_browse_args(raw_dir.clone());
    let document = load_dashboard_browse_document_for_args(
        &mut |_method, _path, _params, _payload| {
            Err(message("local browse should not call Grafana"))
        },
        &args,
    )
    .unwrap();

    assert_eq!(document.summary.dashboard_count, 1);
    assert_eq!(document.summary.folder_count, 2);
    assert_eq!(
        document.summary.scope_label,
        format!("Local export tree ({})", raw_dir.display())
    );
    assert_eq!(document.nodes.len(), 3);
    assert_eq!(document.nodes[0].kind, DashboardBrowseNodeKind::Folder);
    assert_eq!(document.nodes[0].title, "Platform");
    assert_eq!(document.nodes[1].kind, DashboardBrowseNodeKind::Folder);
    assert_eq!(document.nodes[1].title, "Infra");
    assert_eq!(document.nodes[2].kind, DashboardBrowseNodeKind::Dashboard);
    assert_eq!(document.nodes[2].title, "CPU Main");
    assert_eq!(document.nodes[2].details[4], "UID: cpu-main");
    assert!(document.nodes[2]
        .details
        .iter()
        .any(|line| line.contains("Local browse: live details are unavailable.")));
}

#[test]
fn workspace_root_browse_resolves_git_sync_raw_tree() {
    let temp = tempdir().unwrap();
    let workspace = temp.path();
    let raw_dir = workspace.join("dashboards/git-sync/raw");
    let dashboard_dir = raw_dir.join("Platform/Infra");
    fs::create_dir_all(&dashboard_dir).unwrap();
    fs::write(
        dashboard_dir.join("cpu-main.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "uid": "cpu-main",
                "title": "CPU Main"
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let mut args = make_browse_args(PathBuf::from("."));
    args.input_dir = None;
    args.workspace = Some(workspace.to_path_buf());
    let document = load_dashboard_browse_document_for_args(
        &mut |_method, _path, _params, _payload| {
            Err(message("local browse should not call Grafana"))
        },
        &args,
    )
    .unwrap();

    assert_eq!(document.summary.dashboard_count, 1);
    assert_eq!(
        document.summary.scope_label,
        format!("Local export tree ({})", raw_dir.display())
    );
    assert_eq!(
        document.nodes.last().map(|node| node.title.as_str()),
        Some("CPU Main")
    );
}

#[test]
fn workspace_root_browse_rejects_non_dashboard_repo_root() {
    let temp = tempdir().unwrap();
    fs::write(
        temp.path().join("dashboard_like.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "uid": "cpu-main",
                "title": "CPU Main"
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let mut args = make_browse_args(PathBuf::from("."));
    args.input_dir = None;
    args.workspace = Some(temp.path().to_path_buf());
    let error = load_dashboard_browse_document_for_args(
        &mut |_method, _path, _params, _payload| {
            Err(message("local browse should not call Grafana"))
        },
        &args,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("does not contain a browsable raw dashboard tree"));
}
