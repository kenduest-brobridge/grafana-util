//! Dashboard CLI parser/help regressions kept separate from runtime-heavy tests.
use super::test_support;
use super::{
    parse_cli_from, DashboardCliArgs, DashboardCommand, InspectExportReportFormat,
    InspectOutputFormat, ValidationOutputFormat,
};
use clap::{CommandFactory, Parser};
use std::path::{Path, PathBuf};

fn render_dashboard_subcommand_help(name: &str) -> String {
    let mut command = DashboardCliArgs::command();
    command
        .find_subcommand_mut(name)
        .unwrap_or_else(|| panic!("missing {name} subcommand"))
        .render_help()
        .to_string()
}

fn render_dashboard_help() -> String {
    let mut command = DashboardCliArgs::command();
    command.render_help().to_string()
}

#[test]
fn parse_cli_supports_list_mode() {
    let args = parse_cli_from([
        "grafana-util",
        "list",
        "--url",
        "https://grafana.example.com",
        "--page-size",
        "25",
    ]);

    match args.command {
        DashboardCommand::List(list_args) => {
            assert_eq!(list_args.common.url, "https://grafana.example.com");
            assert_eq!(list_args.page_size, 25);
            assert_eq!(list_args.org_id, None);
            assert!(!list_args.all_orgs);
            assert!(!list_args.with_sources);
            assert!(list_args.output_columns.is_empty());
            assert!(!list_args.table);
            assert!(!list_args.csv);
            assert!(!list_args.json);
            assert!(!list_args.no_header);
        }
        _ => panic!("expected list command"),
    }
}

#[test]
fn parse_cli_supports_list_with_sources() {
    let args = parse_cli_from([
        "grafana-util",
        "list",
        "--url",
        "https://grafana.example.com",
        "--with-sources",
        "--json",
    ]);

    match args.command {
        DashboardCommand::List(list_args) => {
            assert_eq!(list_args.org_id, None);
            assert!(!list_args.all_orgs);
            assert!(list_args.with_sources);
            assert!(list_args.output_columns.is_empty());
            assert!(list_args.json);
            assert!(!list_args.table);
            assert!(!list_args.csv);
        }
        _ => panic!("expected list command"),
    }
}

#[test]
fn parse_cli_supports_list_output_columns_with_aliases() {
    let args = parse_cli_from([
        "grafana-util",
        "list",
        "--url",
        "https://grafana.example.com",
        "--output-columns",
        "uid,folderUid,orgId,sources,sourceUids",
    ]);

    match args.command {
        DashboardCommand::List(list_args) => {
            assert_eq!(
                list_args.output_columns,
                vec!["uid", "folder_uid", "org_id", "sources", "source_uids"]
            );
            assert!(!list_args.with_sources);
        }
        _ => panic!("expected list command"),
    }
}

#[test]
fn parse_cli_supports_list_output_format_csv() {
    let args = parse_cli_from([
        "grafana-util",
        "list",
        "--url",
        "https://grafana.example.com",
        "--output-format",
        "csv",
    ]);

    match args.command {
        DashboardCommand::List(list_args) => {
            assert_eq!(list_args.org_id, None);
            assert!(!list_args.all_orgs);
            assert!(!list_args.table);
            assert!(list_args.csv);
            assert!(!list_args.json);
        }
        _ => panic!("expected list command"),
    }
}

#[test]
fn parse_cli_supports_preferred_auth_aliases() {
    let args = parse_cli_from([
        "grafana-util",
        "export",
        "--token",
        "abc123",
        "--basic-user",
        "user",
        "--basic-password",
        "pass",
    ]);

    match args.command {
        DashboardCommand::Export(export_args) => {
            assert_eq!(export_args.common.api_token.as_deref(), Some("abc123"));
            assert_eq!(export_args.common.username.as_deref(), Some("user"));
            assert_eq!(export_args.common.password.as_deref(), Some("pass"));
            assert!(!export_args.common.prompt_password);
        }
        _ => panic!("expected export command"),
    }
}

#[test]
fn parse_cli_supports_prompt_password() {
    let args = parse_cli_from([
        "grafana-util",
        "export",
        "--basic-user",
        "user",
        "--prompt-password",
    ]);

    match args.command {
        DashboardCommand::Export(export_args) => {
            assert_eq!(export_args.common.username.as_deref(), Some("user"));
            assert_eq!(export_args.common.password.as_deref(), None);
            assert!(export_args.common.prompt_password);
        }
        _ => panic!("expected export command"),
    }
}

#[test]
fn parse_cli_supports_prompt_token() {
    let args = parse_cli_from(["grafana-util", "export", "--prompt-token"]);

    match args.command {
        DashboardCommand::Export(export_args) => {
            assert_eq!(export_args.common.api_token.as_deref(), None);
            assert!(export_args.common.prompt_token);
            assert!(!export_args.common.prompt_password);
        }
        _ => panic!("expected export command"),
    }
}

#[test]
fn parse_cli_supports_export_org_scope_flags() {
    let org_args = parse_cli_from(["grafana-util", "export", "--org-id", "7"]);
    let all_orgs_args = parse_cli_from(["grafana-util", "export", "--all-orgs"]);

    match org_args.command {
        DashboardCommand::Export(export_args) => {
            assert_eq!(export_args.org_id, Some(7));
            assert!(!export_args.all_orgs);
        }
        _ => panic!("expected export command"),
    }

    match all_orgs_args.command {
        DashboardCommand::Export(export_args) => {
            assert_eq!(export_args.org_id, None);
            assert!(export_args.all_orgs);
        }
        _ => panic!("expected export command"),
    }
}

#[test]
fn parse_cli_rejects_conflicting_export_org_scope_flags() {
    let error =
        DashboardCliArgs::try_parse_from(["grafana-util", "export", "--org-id", "7", "--all-orgs"])
            .unwrap_err();

    assert!(error.to_string().contains("--org-id"));
    assert!(error.to_string().contains("--all-orgs"));
}

#[test]
fn export_help_explains_flat_layout() {
    let help = render_dashboard_subcommand_help("export");
    assert!(help.contains("--all-orgs"));
    assert!(help.contains("Prefer Basic auth when you need cross-org export"));
    assert!(help.contains("Export dashboards across all visible orgs with Basic auth"));
    assert!(help.contains("Write dashboard files directly into each export variant directory"));
    assert!(help.contains("folder-based subdirectories on disk"));
}

#[test]
fn export_help_describes_progress_and_verbose_modes() {
    let help = render_dashboard_subcommand_help("export");
    assert!(help.contains("--progress"));
    assert!(help.contains("<current>/<total>"));
    assert!(help.contains("-v, --verbose"));
    assert!(help.contains("Overrides --progress output"));
    assert!(!help.contains("--username"));
    assert!(!help.contains("--password "));
}

#[test]
fn import_help_explains_common_operator_flags() {
    let help = render_dashboard_subcommand_help("import");
    assert!(help.contains("Use the raw/ export directory for single-org import"));
    assert!(help.contains("folder missing/match/mismatch state"));
    assert!(help.contains("skipped/blocked"));
    assert!(help.contains("folder check is also shown in table form"));
    assert!(help.contains("source raw folder path matches"));
    assert!(help.contains("--org-id"));
    assert!(help.contains("--use-export-org"));
    assert!(help.contains("--only-org-id"));
    assert!(help.contains("--create-missing-orgs"));
    assert!(help.contains("requires Basic auth"));
    assert!(help.contains("--require-matching-export-org"));
    assert!(help.contains("--output-columns"));
}

#[test]
fn top_level_help_includes_examples() {
    let help = render_dashboard_help();
    assert!(help.contains("Export dashboards from local Grafana with Basic auth"));
    assert!(help.contains("Export dashboards across all visible orgs with Basic auth"));
    assert!(help.contains("List dashboards across all visible orgs with Basic auth"));
    assert!(help.contains("Export dashboards with an API token from the current org"));
    assert!(help.contains("grafana-util export"));
    assert!(help.contains("--all-orgs"));
    assert!(help.contains("grafana-util diff"));
}

#[test]
fn list_help_mentions_cross_org_basic_auth_examples() {
    let help = render_dashboard_subcommand_help("list");
    assert!(help.contains("--all-orgs"));
    assert!(help.contains("Prefer Basic auth when you need cross-org listing"));
    assert!(help.contains("List dashboards across all visible orgs with Basic auth"));
    assert!(help.contains("List dashboards from one explicit org ID"));
    assert!(help.contains("List dashboards from the current org with an API token"));
}

#[test]
fn parse_cli_supports_list_csv_mode() {
    let args = parse_cli_from([
        "grafana-util",
        "list",
        "--url",
        "https://grafana.example.com",
        "--csv",
    ]);

    match args.command {
        DashboardCommand::List(list_args) => {
            assert_eq!(list_args.org_id, None);
            assert!(!list_args.all_orgs);
            assert!(!list_args.table);
            assert!(list_args.csv);
            assert!(!list_args.json);
        }
        _ => panic!("expected list command"),
    }
}

#[test]
fn parse_cli_supports_export_progress_and_verbose_flags() {
    let args = parse_cli_from(["grafana-util", "export", "--progress", "--verbose"]);

    match args.command {
        DashboardCommand::Export(export_args) => {
            assert!(export_args.progress);
            assert!(export_args.verbose);
        }
        _ => panic!("expected export command"),
    }
}

#[test]
fn parse_cli_supports_import_progress_and_verbose_flags() {
    let args = parse_cli_from([
        "grafana-util",
        "import",
        "--import-dir",
        "./dashboards/raw",
        "--progress",
        "-v",
    ]);

    match args.command {
        DashboardCommand::Import(import_args) => {
            assert!(import_args.progress);
            assert!(import_args.verbose);
        }
        _ => panic!("expected import command"),
    }
}

#[test]
fn parse_cli_supports_import_dry_run_json_flag() {
    let args = parse_cli_from([
        "grafana-util",
        "import",
        "--import-dir",
        "./dashboards/raw",
        "--dry-run",
        "--json",
    ]);

    match args.command {
        DashboardCommand::Import(import_args) => {
            assert!(import_args.dry_run);
            assert!(import_args.json);
        }
        _ => panic!("expected import command"),
    }
}

#[test]
fn parse_cli_supports_import_dry_run_output_format_table() {
    let args = parse_cli_from([
        "grafana-util",
        "import",
        "--import-dir",
        "./dashboards/raw",
        "--dry-run",
        "--output-format",
        "table",
    ]);

    match args.command {
        DashboardCommand::Import(import_args) => {
            assert!(import_args.dry_run);
            assert!(import_args.table);
            assert!(!import_args.json);
        }
        _ => panic!("expected import command"),
    }
}

#[test]
fn parse_cli_supports_import_dry_run_output_columns() {
    let args = parse_cli_from([
        "grafana-util",
        "import",
        "--import-dir",
        "./dashboards/raw",
        "--dry-run",
        "--output-format",
        "table",
        "--output-columns",
        "uid,action,source_folder_path,destinationFolderPath,reason,file",
    ]);

    match args.command {
        DashboardCommand::Import(import_args) => {
            assert!(import_args.table);
            assert_eq!(
                import_args.output_columns,
                vec![
                    "uid",
                    "action",
                    "source_folder_path",
                    "destination_folder_path",
                    "reason",
                    "file",
                ]
            );
        }
        _ => panic!("expected import command"),
    }
}

#[test]
fn parse_cli_supports_import_update_existing_only_flag() {
    let args = parse_cli_from([
        "grafana-util",
        "import",
        "--import-dir",
        "./dashboards/raw",
        "--update-existing-only",
    ]);

    match args.command {
        DashboardCommand::Import(import_args) => {
            assert!(import_args.update_existing_only);
            assert!(!import_args.replace_existing);
        }
        _ => panic!("expected import command"),
    }
}

#[test]
fn parse_cli_supports_import_require_matching_folder_path_flag() {
    let args = parse_cli_from([
        "grafana-util",
        "import",
        "--import-dir",
        "./dashboards/raw",
        "--require-matching-folder-path",
    ]);

    match args.command {
        DashboardCommand::Import(import_args) => {
            assert!(import_args.require_matching_folder_path);
        }
        _ => panic!("expected import command"),
    }
}

#[test]
fn parse_cli_supports_import_org_scope_flag() {
    let args = parse_cli_from([
        "grafana-util",
        "import",
        "--import-dir",
        "./dashboards/raw",
        "--org-id",
        "7",
    ]);

    match args.command {
        DashboardCommand::Import(import_args) => {
            assert_eq!(import_args.org_id, Some(7));
        }
        _ => panic!("expected import command"),
    }
}

#[test]
fn parse_cli_supports_import_by_export_org_flags() {
    let args = parse_cli_from([
        "grafana-util",
        "import",
        "--import-dir",
        "./dashboards",
        "--use-export-org",
        "--only-org-id",
        "2",
        "--only-org-id",
        "5",
        "--create-missing-orgs",
    ]);

    match args.command {
        DashboardCommand::Import(import_args) => {
            assert!(import_args.use_export_org);
            assert_eq!(import_args.only_org_id, vec![2, 5]);
            assert!(import_args.create_missing_orgs);
            assert_eq!(import_args.org_id, None);
        }
        _ => panic!("expected import command"),
    }
}

#[test]
fn parse_cli_supports_import_require_matching_export_org_flag() {
    let args = parse_cli_from([
        "grafana-util",
        "import",
        "--import-dir",
        "./dashboards/raw",
        "--require-matching-export-org",
    ]);

    match args.command {
        DashboardCommand::Import(import_args) => {
            assert!(import_args.require_matching_export_org);
        }
        _ => panic!("expected import command"),
    }
}

#[test]
fn parse_cli_rejects_import_org_id_with_use_export_org() {
    let error = DashboardCliArgs::try_parse_from([
        "grafana-util",
        "import",
        "--import-dir",
        "./dashboards",
        "--org-id",
        "7",
        "--use-export-org",
    ])
    .unwrap_err();

    assert!(error.to_string().contains("--org-id"));
    assert!(error.to_string().contains("--use-export-org"));
}

#[test]
fn parse_cli_supports_import_use_export_org_flags() {
    let args = parse_cli_from([
        "grafana-util",
        "import",
        "--import-dir",
        "./dashboards",
        "--use-export-org",
        "--only-org-id",
        "2",
        "--only-org-id",
        "5",
        "--create-missing-orgs",
    ]);

    match args.command {
        DashboardCommand::Import(import_args) => {
            assert!(import_args.use_export_org);
            assert_eq!(import_args.only_org_id, vec![2, 5]);
            assert!(import_args.create_missing_orgs);
        }
        _ => panic!("expected import command"),
    }
}

#[test]
fn parse_cli_supports_inspect_live_report_json_flag() {
    let args = parse_cli_from([
        "grafana-util",
        "inspect-live",
        "--url",
        "https://grafana.example.com",
        "--report",
        "json",
    ]);

    match args.command {
        DashboardCommand::InspectLive(inspect_args) => {
            assert_eq!(inspect_args.common.url, "https://grafana.example.com");
            assert_eq!(inspect_args.report, Some(InspectExportReportFormat::Json));
            assert!(!inspect_args.json);
            assert!(!inspect_args.table);
        }
        _ => panic!("expected inspect-live command"),
    }
}

#[test]
fn parse_cli_supports_inspect_live_output_format_flag() {
    let args = parse_cli_from([
        "grafana-util",
        "inspect-live",
        "--url",
        "https://grafana.example.com",
        "--output-format",
        "governance-json",
    ]);

    match args.command {
        DashboardCommand::InspectLive(inspect_args) => {
            assert_eq!(inspect_args.common.url, "https://grafana.example.com");
            assert_eq!(
                inspect_args.output_format,
                Some(InspectOutputFormat::GovernanceJson)
            );
            assert_eq!(inspect_args.report, None);
            assert!(!inspect_args.json);
            assert!(!inspect_args.table);
        }
        _ => panic!("expected inspect-live command"),
    }
}

#[test]
fn parse_cli_supports_inspect_live_output_format_dependency_json_flag() {
    let args = parse_cli_from([
        "grafana-util",
        "inspect-live",
        "--url",
        "https://grafana.example.com",
        "--output-format",
        "report-dependency-json",
    ]);

    match args.command {
        DashboardCommand::InspectLive(inspect_args) => {
            assert_eq!(inspect_args.common.url, "https://grafana.example.com");
            assert_eq!(
                inspect_args.output_format,
                Some(InspectOutputFormat::ReportDependencyJson)
            );
            assert_eq!(inspect_args.report, None);
            assert!(!inspect_args.json);
            assert!(!inspect_args.table);
        }
        _ => panic!("expected inspect-live command"),
    }
}

#[test]
fn parse_cli_supports_inspect_live_output_file() {
    let args = parse_cli_from([
        "grafana-util",
        "inspect-live",
        "--url",
        "https://grafana.example.com",
        "--output-format",
        "report-tree",
        "--output-file",
        "/tmp/inspect-live.txt",
    ]);

    match args.command {
        DashboardCommand::InspectLive(inspect_args) => {
            assert_eq!(
                inspect_args.output_file,
                Some(PathBuf::from("/tmp/inspect-live.txt"))
            );
            assert_eq!(
                inspect_args.output_format,
                Some(InspectOutputFormat::ReportTree)
            );
        }
        _ => panic!("expected inspect-live command"),
    }
}

#[test]
fn parse_cli_supports_inspect_live_report_tree_table_flag() {
    let args = parse_cli_from([
        "grafana-util",
        "inspect-live",
        "--url",
        "https://grafana.example.com",
        "--report",
        "tree-table",
    ]);

    match args.command {
        DashboardCommand::InspectLive(inspect_args) => {
            assert_eq!(inspect_args.common.url, "https://grafana.example.com");
            assert_eq!(
                inspect_args.report,
                Some(InspectExportReportFormat::TreeTable)
            );
            assert!(!inspect_args.json);
            assert!(!inspect_args.table);
        }
        _ => panic!("expected inspect-live command"),
    }
}

#[test]
fn parse_cli_supports_inspect_live_report_dependency_flag() {
    let args = parse_cli_from([
        "grafana-util",
        "inspect-live",
        "--url",
        "https://grafana.example.com",
        "--report",
        "dependency",
    ]);

    match args.command {
        DashboardCommand::InspectLive(inspect_args) => {
            assert_eq!(inspect_args.common.url, "https://grafana.example.com");
            assert_eq!(
                inspect_args.report,
                Some(InspectExportReportFormat::Dependency)
            );
            assert!(!inspect_args.json);
            assert!(!inspect_args.table);
        }
        _ => panic!("expected inspect-live command"),
    }
}

#[test]
fn parse_cli_supports_inspect_live_report_governance_json_flag() {
    let args = parse_cli_from([
        "grafana-util",
        "inspect-live",
        "--url",
        "https://grafana.example.com",
        "--report",
        "governance-json",
    ]);

    match args.command {
        DashboardCommand::InspectLive(inspect_args) => {
            assert_eq!(inspect_args.common.url, "https://grafana.example.com");
            assert_eq!(
                inspect_args.report,
                Some(InspectExportReportFormat::GovernanceJson)
            );
            assert!(!inspect_args.json);
            assert!(!inspect_args.table);
        }
        _ => panic!("expected inspect-live command"),
    }
}

#[test]
fn parse_cli_supports_inspect_live_help_full_flag() {
    let args = parse_cli_from([
        "grafana-util",
        "inspect-live",
        "--url",
        "https://grafana.example.com",
        "--help-full",
    ]);

    match args.command {
        DashboardCommand::InspectLive(inspect_args) => {
            assert!(inspect_args.help_full);
            assert_eq!(inspect_args.common.url, "https://grafana.example.com");
        }
        _ => panic!("expected inspect-live command"),
    }
}

#[test]
fn parse_cli_supports_inspect_live_all_orgs_flag() {
    let args = parse_cli_from(["grafana-util", "inspect-live", "--all-orgs", "--table"]);

    match args.command {
        DashboardCommand::InspectLive(inspect_args) => {
            assert!(inspect_args.all_orgs);
            assert!(inspect_args.table);
            assert!(inspect_args.org_id.is_none());
        }
        _ => panic!("expected inspect-live command"),
    }
}

#[test]
fn inspect_live_help_mentions_interactive_browser() {
    let help = render_dashboard_subcommand_help("inspect-live");
    assert!(help.contains("--interactive"));
}

#[test]
fn parse_cli_supports_dashboard_validate_export_command() {
    let args = parse_cli_from([
        "grafana-util",
        "validate-export",
        "--import-dir",
        "./dashboards/raw",
        "--reject-custom-plugins",
        "--reject-legacy-properties",
        "--target-schema-version",
        "39",
        "--output-format",
        "json",
        "--output-file",
        "./dashboard-validation.json",
    ]);

    match args.command {
        DashboardCommand::ValidateExport(validate_args) => {
            assert_eq!(validate_args.import_dir, Path::new("./dashboards/raw"));
            assert!(validate_args.reject_custom_plugins);
            assert!(validate_args.reject_legacy_properties);
            assert_eq!(validate_args.target_schema_version, Some(39));
            assert_eq!(validate_args.output_format, ValidationOutputFormat::Json);
            assert_eq!(
                validate_args.output_file,
                Some(PathBuf::from("./dashboard-validation.json"))
            );
        }
        _ => panic!("expected validate-export command"),
    }
}

#[test]
fn inspect_live_help_mentions_report_and_panel_filter_flags() {
    let help = render_dashboard_subcommand_help("inspect-live");

    assert!(help.contains("--report"));
    assert!(help.contains("--output-format"));
    assert!(help.contains("--report-filter-panel-id"));
    assert!(help.contains("--all-orgs"));
    assert!(help.contains("--concurrency"));
    assert!(help.contains("--progress"));
    assert!(help.contains("--help-full"));
    assert!(help.contains("tree"));
    assert!(help.contains("tree-table"));
    assert!(!help.contains("Extended Examples:"));
}

#[test]
fn inspect_export_help_lists_datasource_uid_report_column() {
    let mut command = DashboardCliArgs::command();
    let help = command
        .find_subcommand_mut("inspect-export")
        .expect("inspect-export subcommand")
        .render_help()
        .to_string();

    assert!(help.contains("datasource_uid"));
    assert!(help.contains("folder_level"));
    assert!(help.contains("folder_full_path"));
    assert!(help.contains("Use all to expand every supported column."));
    assert!(help.contains("datasource_type"));
    assert!(help.contains("datasource_family"));
    assert!(help.contains("dashboard_tags"));
    assert!(help.contains("panel_query_count"));
    assert!(help.contains("panel_datasource_count"));
    assert!(help.contains("panel_variables"));
    assert!(help.contains("query_variables"));
    assert!(help.contains("dashboardTags"));
    assert!(help.contains("panelQueryCount"));
    assert!(help.contains("panelDatasourceCount"));
    assert!(help.contains("panelVariables"));
    assert!(help.contains("queryVariables"));
    assert!(help.contains("file"));
    assert!(help.contains("dashboardUid"));
    assert!(help.contains("datasource label, uid, type, or family"));
    assert!(help.contains("--output-format"));
}

#[test]
fn inspect_export_help_full_includes_extended_examples() {
    let help = test_support::render_inspect_export_help_full();

    assert!(help.contains("--help-full"));
    assert!(help.contains("Extended Examples:"));
    assert!(help.contains("--report tree-table"));
    assert!(help.contains("--report-filter-datasource"));
    assert!(help.contains("--report-filter-panel-id 7"));
    assert!(help.contains("--report-columns"));
    assert!(help.contains(
        "--report-columns dashboard_tags,panel_id,panel_query_count,panel_datasource_count,query_variables,panel_variables"
    ));
    assert!(help.contains(
        "--report-columns dashboard_uid,folder_path,folder_full_path,folder_level,folder_uid,parent_folder_uid,file"
    ));
    assert!(help.contains(
        "--report-columns datasource_name,datasource_org,datasource_org_id,datasource_database,datasource_bucket,datasource_index_pattern,query"
    ));
}

#[test]
fn inspect_live_help_full_includes_extended_examples() {
    let help = test_support::render_inspect_live_help_full();

    assert!(help.contains("--help-full"));
    assert!(help.contains("Extended Examples:"));
    assert!(help.contains("--token \"$GRAFANA_API_TOKEN\""));
    assert!(help.contains("--report tree-table"));
    assert!(help.contains("--report-filter-panel-id"));
    assert!(help.contains("--report-columns"));
    assert!(help.contains(
        "--report-columns panel_id,ref_id,datasource_name,metrics,functions,buckets,query"
    ));
    assert!(help.contains(
        "--report-columns dashboard_tags,panel_id,panel_query_count,panel_datasource_count,query_variables,panel_variables"
    ));
    assert!(help.contains(
        "--report-columns dashboard_uid,folder_path,folder_full_path,folder_level,folder_uid,parent_folder_uid,file"
    ));
    assert!(help.contains(
        "--report-columns datasource_name,datasource_org,datasource_org_id,datasource_database,datasource_bucket,datasource_index_pattern,query"
    ));
}

#[test]
fn maybe_render_dashboard_help_full_from_os_args_handles_missing_required_args() {
    let help = test_support::maybe_render_dashboard_help_full_from_os_args([
        "grafana-util",
        "dashboard",
        "inspect-export",
        "--help-full",
    ])
    .expect("expected inspect-export full help");

    assert!(help.contains("inspect-export"));
    assert!(help.contains("Extended Examples:"));
    assert!(help.contains("--report tree-table"));
    assert!(help.contains("--report-filter-panel-id 7"));
}

#[test]
fn maybe_render_dashboard_help_full_from_os_args_ignores_other_commands() {
    let help = test_support::maybe_render_dashboard_help_full_from_os_args([
        "grafana-util",
        "export",
        "--help-full",
    ]);

    assert!(help.is_none());
}

#[test]
fn parse_cli_supports_list_json_mode() {
    let args = parse_cli_from([
        "grafana-util",
        "list",
        "--url",
        "https://grafana.example.com",
        "--json",
    ]);

    match args.command {
        DashboardCommand::List(list_args) => {
            assert_eq!(list_args.org_id, None);
            assert!(!list_args.all_orgs);
            assert!(!list_args.table);
            assert!(!list_args.csv);
            assert!(list_args.json);
        }
        _ => panic!("expected list command"),
    }
}

#[test]
fn parse_cli_rejects_conflicting_list_output_modes() {
    let error = DashboardCliArgs::try_parse_from([
        "grafana-util",
        "list",
        "--url",
        "https://grafana.example.com",
        "--table",
        "--json",
    ])
    .unwrap_err();

    assert!(error.to_string().contains("--table"));
    assert!(error.to_string().contains("--json"));
}

#[test]
fn parse_cli_supports_list_org_scope_flags() {
    let org_args = parse_cli_from(["grafana-util", "list", "--org-id", "7"]);
    let all_orgs_args = parse_cli_from(["grafana-util", "list", "--all-orgs"]);

    match org_args.command {
        DashboardCommand::List(list_args) => {
            assert_eq!(list_args.org_id, Some(7));
            assert!(!list_args.all_orgs);
        }
        _ => panic!("expected list command"),
    }

    match all_orgs_args.command {
        DashboardCommand::List(list_args) => {
            assert_eq!(list_args.org_id, None);
            assert!(list_args.all_orgs);
        }
        _ => panic!("expected list command"),
    }
}
