use crate::project_status_command::{
    ProjectStatusCliArgs, ProjectStatusOutputFormat, ProjectStatusSubcommand,
    PROJECT_STATUS_LIVE_HELP_TEXT, PROJECT_STATUS_STAGED_HELP_TEXT,
};
use clap::{CommandFactory, Parser};
use std::path::Path;

#[test]
fn project_status_cli_help_and_parse_support_datasource_provisioning_file() {
    let mut command = ProjectStatusCliArgs::command();
    let subcommand = command
        .find_subcommand_mut("staged")
        .expect("missing staged help");
    let help = subcommand.render_long_help().to_string();
    assert!(help.contains("--dashboard-provisioning-dir"));
    assert!(help.contains("--datasource-provisioning-file"));
    assert!(help.contains("Render staged or live status as table, csv, text, json, yaml"));
    assert!(PROJECT_STATUS_STAGED_HELP_TEXT
        .contains("grafana-util status staged --dashboard-export-dir ./dashboards/raw"));
    assert!(PROJECT_STATUS_LIVE_HELP_TEXT
        .contains("grafana-util status live --url http://localhost:3000 --token"));
    assert!(!help.contains("grafana-util observe"));

    let args = ProjectStatusCliArgs::parse_from([
        "grafana-util",
        "staged",
        "--datasource-provisioning-file",
        "./datasources/provisioning/datasources.yaml",
        "--output-format",
        "json",
    ]);

    match args.command {
        ProjectStatusSubcommand::Staged(inner) => {
            assert_eq!(
                inner.datasource_provisioning_file,
                Some(Path::new("./datasources/provisioning/datasources.yaml").to_path_buf())
            );
        }
        _ => panic!("expected staged"),
    }
}

#[test]
fn project_status_cli_supports_all_output_modes_for_staged_and_live_commands() {
    for (output, expected) in [
        ("table", ProjectStatusOutputFormat::Table),
        ("csv", ProjectStatusOutputFormat::Csv),
        ("text", ProjectStatusOutputFormat::Text),
        ("json", ProjectStatusOutputFormat::Json),
        ("yaml", ProjectStatusOutputFormat::Yaml),
    ] {
        let staged_args = ProjectStatusCliArgs::parse_from([
            "grafana-util",
            "staged",
            "--desired-file",
            "./desired.json",
            "--output-format",
            output,
        ]);
        let live_args = ProjectStatusCliArgs::parse_from([
            "grafana-util",
            "live",
            "--url",
            "http://127.0.0.1:3000",
            "--output-format",
            output,
        ]);

        match staged_args.command {
            ProjectStatusSubcommand::Staged(inner) => {
                assert_eq!(inner.output_format, expected);
            }
            _ => panic!("expected staged"),
        }

        match live_args.command {
            ProjectStatusSubcommand::Live(inner) => {
                assert_eq!(inner.output_format, expected);
            }
            _ => panic!("expected live"),
        }
    }

    #[cfg(feature = "tui")]
    {
        let staged_args = ProjectStatusCliArgs::parse_from([
            "grafana-util",
            "staged",
            "--desired-file",
            "./desired.json",
            "--output-format",
            "interactive",
        ]);
        let live_args = ProjectStatusCliArgs::parse_from([
            "grafana-util",
            "live",
            "--url",
            "http://127.0.0.1:3000",
            "--output-format",
            "interactive",
        ]);

        match staged_args.command {
            ProjectStatusSubcommand::Staged(inner) => {
                assert_eq!(inner.output_format, ProjectStatusOutputFormat::Interactive);
            }
            _ => panic!("expected staged"),
        }

        match live_args.command {
            ProjectStatusSubcommand::Live(inner) => {
                assert_eq!(inner.output_format, ProjectStatusOutputFormat::Interactive);
            }
            _ => panic!("expected live"),
        }
    }
}

#[test]
fn project_status_cli_supports_dashboard_provisioning_dir() {
    let args = ProjectStatusCliArgs::parse_from([
        "grafana-util",
        "staged",
        "--dashboard-provisioning-dir",
        "./dashboards/provisioning",
        "--output-format",
        "json",
    ]);

    match args.command {
        ProjectStatusSubcommand::Staged(inner) => {
            assert_eq!(
                inner.dashboard_provisioning_dir,
                Some(Path::new("./dashboards/provisioning").to_path_buf())
            );
        }
        _ => panic!("expected staged"),
    }
}

#[test]
fn project_status_cli_rejects_dashboard_export_and_provisioning_inputs_together() {
    let args = ProjectStatusCliArgs::try_parse_from([
        "grafana-util",
        "staged",
        "--dashboard-export-dir",
        "./dashboards/raw",
        "--dashboard-provisioning-dir",
        "./dashboards/provisioning",
    ]);

    assert!(args.is_err());
}
