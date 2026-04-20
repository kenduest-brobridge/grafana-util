use super::*;

#[test]
fn build_overview_artifacts_rejects_empty_inputs() {
    let args = OverviewArgs {
        dashboard_export_dir: None,
        dashboard_provisioning_dir: None,
        datasource_export_dir: None,
        datasource_provisioning_file: None,
        access_user_export_dir: None,
        access_team_export_dir: None,
        access_org_export_dir: None,
        access_service_account_export_dir: None,
        desired_file: None,
        source_bundle: None,
        target_inventory: None,
        alert_export_dir: None,
        availability_file: None,
        mapping_file: None,
        output_format: OverviewOutputFormat::Text,
    };

    let error = build_overview_artifacts(&args).unwrap_err().to_string();

    assert!(error.contains("at least one input artifact"));
}

#[test]
fn overview_args_parse_and_help_expose_output_mode() {
    let default_args = OverviewCliArgs::parse_from(["grafana-util"]);
    assert!(default_args.command.is_none());
    assert_eq!(
        default_args.staged.output_format,
        OverviewOutputFormat::Text
    );

    let table_args = OverviewCliArgs::parse_from(["grafana-util", "--output-format", "table"]);
    assert_eq!(table_args.staged.output_format, OverviewOutputFormat::Table);

    let csv_args = OverviewCliArgs::parse_from(["grafana-util", "--output-format", "csv"]);
    assert_eq!(csv_args.staged.output_format, OverviewOutputFormat::Csv);

    let json_args = OverviewCliArgs::parse_from(["grafana-util", "--output-format", "json"]);
    assert!(json_args.command.is_none());
    assert_eq!(json_args.staged.output_format, OverviewOutputFormat::Json);

    let yaml_args = OverviewCliArgs::parse_from(["grafana-util", "--output-format", "yaml"]);
    assert!(yaml_args.command.is_none());
    assert_eq!(yaml_args.staged.output_format, OverviewOutputFormat::Yaml);

    #[cfg(feature = "tui")]
    {
        let interactive_args =
            OverviewCliArgs::parse_from(["grafana-util", "--output-format", "interactive"]);
        assert!(interactive_args.command.is_none());
        assert_eq!(
            interactive_args.staged.output_format,
            OverviewOutputFormat::Interactive
        );
    }
    #[cfg(not(feature = "tui"))]
    {
        assert!(OverviewCliArgs::try_parse_from([
            "grafana-util",
            "--output-format",
            "interactive"
        ])
        .is_err());
    }

    assert!(crate::overview::OVERVIEW_HELP_TEXT
        .contains("grafana-util status overview --dashboard-export-dir ./dashboards/raw"));
    assert!(crate::overview::OVERVIEW_LIVE_HELP_TEXT
        .contains("grafana-util status overview live --url http://localhost:3000 --token"));
    let help = OverviewCliArgs::command().render_long_help().to_string();
    assert!(!help.contains("grafana-util overview"));
}

#[test]
fn overview_summary_rows_feed_the_shared_tabular_renderer() {
    let document = sample_overview_document();
    let rows = build_overview_summary_rows(&document);

    assert_eq!(rows[0], ("status", "partial".to_string()));
    assert_eq!(rows[1], ("scope", "staged-only".to_string()));
    assert_eq!(rows[11], ("artifactCount", "3".to_string()));

    let table = render_summary_table(&rows);
    assert_eq!(table.len(), rows.len() + 2);
    assert!(table[0].contains("field"));
    assert!(table[2].contains("status"));
    assert!(table[2].contains("partial"));

    let csv = render_summary_csv(&rows);
    assert_eq!(csv[0], "field,value");
    assert_eq!(csv[1], "status,partial");
}

#[test]
fn overview_text_renderer_includes_compact_discovery_summary() {
    let mut document = sample_overview_document();
    document.discovery = Some(json!({
        "workspaceRoot": "/tmp/grafana-oac-repo",
        "inputCount": 3,
        "inputs": {
            "dashboardExportDir": "/tmp/grafana-oac-repo/dashboards/git-sync/raw",
            "alertExportDir": "/tmp/grafana-oac-repo/alerts/raw",
            "datasourceProvisioningFile": "/tmp/grafana-oac-repo/datasources/provisioning/datasources.yaml"
        }
    }));

    let lines = render_overview_text(&document).unwrap();
    assert!(lines.iter().any(|line| line.contains(
        "Discovery: workspace-root=/tmp/grafana-oac-repo sources=dashboard-export, datasource-provisioning, alert-export"
    )));
}

#[test]
fn overview_args_support_datasource_provisioning_file() {
    let args = OverviewCliArgs::parse_from([
        "grafana-util",
        "--datasource-provisioning-file",
        "./datasources/provisioning/datasources.yaml",
        "--output-format",
        "json",
    ]);

    assert_eq!(
        args.staged.datasource_provisioning_file,
        Some(std::path::Path::new("./datasources/provisioning/datasources.yaml").to_path_buf())
    );
}

#[test]
fn overview_args_support_dashboard_provisioning_dir() {
    let args = OverviewCliArgs::parse_from([
        "grafana-util",
        "--dashboard-provisioning-dir",
        "./dashboards/provisioning",
        "--output-format",
        "json",
    ]);

    assert_eq!(
        args.staged.dashboard_provisioning_dir,
        Some(std::path::Path::new("./dashboards/provisioning").to_path_buf())
    );
}

#[test]
fn overview_args_reject_dashboard_export_and_provisioning_inputs_together() {
    let args = OverviewCliArgs::try_parse_from([
        "grafana-util",
        "--dashboard-export-dir",
        "./dashboards/raw",
        "--dashboard-provisioning-dir",
        "./dashboards/provisioning",
    ]);

    assert!(args.is_err());
}

#[test]
fn overview_cli_help_exposes_staged_and_live_shapes() {
    assert!(crate::overview::OVERVIEW_HELP_TEXT
        .contains("grafana-util status overview --dashboard-export-dir ./dashboards/raw"));
    assert!(crate::overview::OVERVIEW_LIVE_HELP_TEXT
        .contains("grafana-util status overview live --url http://localhost:3000 --token"));
    let overview_help = OverviewCliArgs::command().render_long_help().to_string();
    assert!(!overview_help.contains("grafana-util overview"));
}

#[test]
fn build_overview_artifacts_rejects_incomplete_bundle_context() {
    let args = OverviewArgs {
        dashboard_export_dir: None,
        dashboard_provisioning_dir: None,
        datasource_export_dir: None,
        datasource_provisioning_file: None,
        access_user_export_dir: None,
        access_team_export_dir: None,
        access_org_export_dir: None,
        access_service_account_export_dir: None,
        desired_file: None,
        source_bundle: Some("/tmp/source-bundle.json".into()),
        target_inventory: None,
        alert_export_dir: None,
        availability_file: None,
        mapping_file: None,
        output_format: OverviewOutputFormat::Text,
    };

    let error = build_overview_artifacts(&args).unwrap_err().to_string();

    assert!(error.contains("--source-bundle"));
    assert!(error.contains("--target-inventory"));
}
