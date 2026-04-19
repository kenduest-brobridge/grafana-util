use crate::artifact_workspace::{
    lane_root_path, profile_scope_path, record_latest_run, run_root_path, ArtifactLane,
};
use crate::common::{CliColorChoice, DiffOutputFormat};
use crate::dashboard::command_artifacts;
use crate::dashboard::{
    parse_cli_from, DashboardCliArgs, DashboardCommand, DashboardImportInputFormat,
};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tempfile::tempdir;

static DASHBOARD_ARTIFACT_WORKSPACE_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn with_dashboard_artifact_workspace<R>(profile: Option<&str>, body: impl FnOnce(&Path) -> R) -> R {
    let _guard = DASHBOARD_ARTIFACT_WORKSPACE_TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap();
    let temp = tempdir().unwrap();
    let config_path = temp.path().join("grafana-util.yaml");
    crate::profile_config::set_profile_config_path_override(Some(config_path));

    struct ResetProfileConfig;

    impl Drop for ResetProfileConfig {
        fn drop(&mut self) {
            crate::profile_config::set_profile_config_path_override(None);
        }
    }

    let _reset = ResetProfileConfig;
    let artifact_root = temp.path().join(".grafana-util").join("artifacts");
    let scope_root = profile_scope_path(&artifact_root, profile);
    body(&scope_root)
}

fn dashboard_artifact_lane_path(scope_root: &Path, run_id: &str) -> PathBuf {
    lane_root_path(&run_root_path(scope_root, run_id), ArtifactLane::Dashboards)
}

fn artifact_test_common_args() -> crate::dashboard::CommonCliArgs {
    crate::dashboard::CommonCliArgs {
        color: CliColorChoice::Never,
        profile: None,
        url: "http://127.0.0.1:3000".to_string(),
        api_token: None,
        username: None,
        password: None,
        prompt_password: false,
        prompt_token: false,
        timeout: 30,
        verify_ssl: false,
    }
}

fn artifact_test_import_args() -> crate::dashboard::ImportArgs {
    crate::dashboard::ImportArgs {
        common: artifact_test_common_args(),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        input_dir: PathBuf::new(),
        local: true,
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
        dry_run: false,
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

fn artifact_test_diff_args(run_id: &str) -> crate::dashboard::DiffArgs {
    crate::dashboard::DiffArgs {
        common: artifact_test_common_args(),
        input_dir: PathBuf::new(),
        local: false,
        run: None,
        run_id: Some(run_id.to_string()),
        input_format: DashboardImportInputFormat::Raw,
        import_folder_uid: None,
        context_lines: 3,
        output_format: DiffOutputFormat::Text,
    }
}

#[test]
fn dashboard_export_run_timestamp_uses_artifact_dashboard_lane_and_records_latest_run() {
    with_dashboard_artifact_workspace(None, |scope_root| {
        let mut args = parse_cli_from(["grafana-util", "export", "--run", "timestamp"]);

        let resolved = match &mut args.command {
            DashboardCommand::Export(export_args) => {
                let resolved =
                    command_artifacts::prepare_dashboard_export_artifact_run(export_args)
                        .unwrap()
                        .expect("expected export artifact run");
                assert_eq!(
                    export_args.output_dir,
                    lane_root_path(&resolved.run_root, ArtifactLane::Dashboards)
                );
                resolved
            }
            _ => panic!("expected export command"),
        };

        command_artifacts::record_dashboard_latest_run(&resolved).unwrap();

        let latest = crate::artifact_workspace::read_latest_run_pointer(scope_root)
            .unwrap()
            .expect("expected latest run pointer");
        assert_eq!(latest.profile, "default");
        assert_eq!(latest.run_id, resolved.run_id);
        assert_eq!(latest.run_path, format!("runs/{}", resolved.run_id));
    });
}

#[test]
fn dashboard_browse_local_materializes_artifact_dashboard_lane_input_dir() {
    with_dashboard_artifact_workspace(None, |scope_root| {
        let run_id = "run-20260420";
        record_latest_run(scope_root, "default", run_id).unwrap();
        let mut args = parse_cli_from(["grafana-util", "browse", "--local"]);

        command_artifacts::materialize_dashboard_artifact_args(&mut args).unwrap();

        match args.command {
            DashboardCommand::Browse(browse_args) => {
                assert_eq!(
                    browse_args.input_dir,
                    Some(dashboard_artifact_lane_path(scope_root, run_id))
                );
            }
            _ => panic!("expected browse command"),
        }
    });
}

#[test]
fn dashboard_summary_run_id_materializes_artifact_dashboard_lane_input_dir() {
    with_dashboard_artifact_workspace(None, |scope_root| {
        let run_id = "run-20260420";
        let mut args = parse_cli_from(["grafana-util", "summary", "--run-id", run_id]);

        command_artifacts::materialize_dashboard_artifact_args(&mut args).unwrap();

        match args.command {
            DashboardCommand::Summary(summary_args) => {
                assert_eq!(
                    summary_args.input_dir,
                    Some(dashboard_artifact_lane_path(scope_root, run_id))
                );
                assert_eq!(summary_args.run_id.as_deref(), Some(run_id));
            }
            _ => panic!("expected summary command"),
        }
    });
}

#[test]
fn dashboard_import_local_materializes_artifact_dashboard_lane_input_dir() {
    with_dashboard_artifact_workspace(None, |scope_root| {
        let run_id = "run-20260420";
        record_latest_run(scope_root, "default", run_id).unwrap();
        let mut args = DashboardCliArgs {
            color: CliColorChoice::Never,
            command: DashboardCommand::Import(artifact_test_import_args()),
        };

        command_artifacts::materialize_dashboard_artifact_args(&mut args).unwrap();

        match args.command {
            DashboardCommand::Import(import_args) => {
                assert_eq!(
                    import_args.input_dir,
                    dashboard_artifact_lane_path(scope_root, run_id)
                );
            }
            _ => panic!("expected import command"),
        }
    });
}

#[test]
fn dashboard_diff_run_id_materializes_artifact_dashboard_lane_input_dir() {
    with_dashboard_artifact_workspace(None, |scope_root| {
        let run_id = "run-20260420";
        let mut args = DashboardCliArgs {
            color: CliColorChoice::Never,
            command: DashboardCommand::Diff(artifact_test_diff_args(run_id)),
        };

        command_artifacts::materialize_dashboard_artifact_args(&mut args).unwrap();

        match args.command {
            DashboardCommand::Diff(diff_args) => {
                assert_eq!(
                    diff_args.input_dir,
                    dashboard_artifact_lane_path(scope_root, run_id)
                );
            }
            _ => panic!("expected diff command"),
        }
    });
}
