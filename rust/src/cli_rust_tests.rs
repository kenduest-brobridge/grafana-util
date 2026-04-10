use super::{
    dispatch_with_handlers, maybe_render_unified_help_from_os_args, parse_cli_from,
    render_unified_help_text, CliArgs, UnifiedCommand,
};
use crate::cli_help_examples::{paint_section, paint_support};
use crate::dashboard::SimpleOutputFormat;
use crate::help_styles::CLI_HELP_STYLES;
use crate::profile_cli::ProfileCommand;
use crate::resource::{ResourceCommand, ResourceKind, ResourceOutputFormat};
use clap::builder::styling::AnsiColor;
use clap::CommandFactory;
use std::cell::RefCell;
use std::path::Path;

fn render_cli_help_path(path: &[&str]) -> String {
    let mut command = CliArgs::command();
    let mut current = &mut command;
    for segment in path {
        current = current
            .find_subcommand_mut(segment)
            .unwrap_or_else(|| panic!("missing cli subcommand {segment}"));
    }
    current.render_help().to_string()
}

#[test]
fn unified_help_mentions_common_and_advanced_surfaces() {
    let help = render_unified_help_text(false);
    assert!(help.contains("config profile add"));
    assert!(help.contains("observe overview"));
    assert!(help.contains("export dashboard"));
    assert!(help.contains("export alert"));
    assert!(help.contains("change preview"));
    assert!(help.contains("advanced dashboard sync import"));
    assert!(!help.contains("dashboard capture screenshot"));
    assert!(!help.contains("alert migrate export"));
}

#[test]
fn advanced_help_groups_domains_and_examples() {
    let help = render_cli_help_path(&["advanced"]);
    assert!(help.contains("dashboard"));
    assert!(help.contains("alert"));
    assert!(help.contains("Examples:"));
    assert!(help.contains("advanced dashboard sync import"));
}

#[test]
fn export_dashboard_help_shows_examples_and_grouped_headings() {
    let help = render_cli_help_path(&["export", "dashboard"]);
    assert!(help.contains("Connection Options"));
    assert!(help.contains("Selection Options"));
    assert!(help.contains("Export Variant Options"));
    assert!(help.contains("Notes:"));
    assert!(help.contains("Examples:"));
    assert!(help.contains("Preview the files and indexes without writing to disk."));
    assert!(help.contains("grafana-util export dashboard"));
    assert!(help.contains("--without-raw"));
    assert!(help.contains("--provider-name <NAME>"));
    assert!(help.contains("--provider-update-interval-seconds <SECONDS>"));
    assert!(!help.contains("--without-dashboard-raw"));
    assert!(!help.contains("--provisioning-provider-name"));
}

#[test]
fn export_dashboard_help_colorizes_notes_and_example_commands() {
    let help = maybe_render_unified_help_from_os_args(
        ["grafana-util", "export", "dashboard", "--help"],
        true,
    )
    .expect("expected export dashboard help");
    assert!(help.contains(&paint_section("Notes:")));
    assert!(help.contains(&paint_section("Examples:")));
    assert!(help.contains(&paint_support("Export dashboards from the current org:")));
    assert!(help.contains("grafana-util export dashboard"));
}

#[test]
fn export_dashboard_help_ends_with_blank_line() {
    let help = maybe_render_unified_help_from_os_args(
        ["grafana-util", "export", "dashboard", "--help"],
        false,
    )
    .expect("expected export dashboard help");
    assert!(help.ends_with("\n\n"));
}

#[test]
fn export_dashboard_help_colorizes_default_context_bright_green() {
    let help = maybe_render_unified_help_from_os_args(
        ["grafana-util", "export", "dashboard", "--help"],
        true,
    )
    .expect("expected export dashboard help");
    assert!(help.contains("default:"));
    assert!(help.contains("500"));
    assert!(help.contains("grafana-utils-dashboards"));
    assert!(help.contains("possible values:"));
}

#[test]
fn cli_help_styles_use_bright_green_bold_context() {
    let rendered = format!("{}", CLI_HELP_STYLES.get_context());
    let expected = format!("{}", AnsiColor::BrightGreen.on_default().bold());
    assert_eq!(rendered, expected);
}

#[test]
fn export_dashboard_help_colorizes_option_descriptions_as_secondary_text() {
    let help = maybe_render_unified_help_from_os_args(
        ["grafana-util", "export", "dashboard", "--help"],
        true,
    )
    .expect("expected export dashboard help");
    assert!(help.contains(&format!(
        "          {}",
        paint_support("Set the generated provisioning provider name.")
    )));
}

#[test]
fn advanced_dashboard_sync_import_help_shows_grouped_headings_and_examples() {
    let help = render_cli_help_path(&["advanced", "dashboard", "sync", "import"]);
    assert!(help.contains("Connection Options"));
    assert!(help.contains("Routing Options"));
    assert!(help.contains("Review Output Options"));
    assert!(help.contains("Examples:"));
    assert!(help.contains("grafana-util advanced dashboard sync import"));
}

#[test]
fn advanced_dashboard_sync_export_help_uses_short_export_flags() {
    let help = render_cli_help_path(&["advanced", "dashboard", "sync", "export"]);
    assert!(help.contains("--without-raw"));
    assert!(help.contains("--provider-name <NAME>"));
    assert!(help.contains("--provider-update-interval-seconds <SECONDS>"));
    assert!(!help.contains("--without-dashboard-raw"));
    assert!(!help.contains("--provisioning-provider-name"));
}

#[test]
fn dashboard_short_help_uses_grouped_lanes_only() {
    let help =
        maybe_render_unified_help_from_os_args(["grafana-util", "dashboard", "-h"], false).unwrap();
    assert!(help.contains("live         browse, list, vars, fetch, clone, edit, delete, history"));
    assert!(help.contains("draft        review, patch, serve, publish"));
    assert!(help.contains("sync         export, import, diff, convert"));
    assert!(help.contains("analyze      summary, topology, impact, governance"));
    assert!(help.contains("capture      screenshot"));
    assert!(!help.contains("fetch-live"));
    assert!(!help.contains("clone-live"));
    assert!(!help.contains("list-vars"));
}

#[test]
fn alert_short_help_uses_grouped_lanes_only() {
    let help =
        maybe_render_unified_help_from_os_args(["grafana-util", "alert", "-h"], false).unwrap();
    assert!(help.contains("list-rules"));
    assert!(help.contains("list-contact-points"));
}

#[test]
fn parse_cli_supports_observe_surface() {
    let live_args: CliArgs = parse_cli_from(["grafana-util", "observe", "live", "--all-orgs"]);
    let overview_args: CliArgs = parse_cli_from([
        "grafana-util",
        "observe",
        "overview",
        "--dashboard-export-dir",
        "./dashboards/raw",
    ]);
    let resource_args: CliArgs = parse_cli_from([
        "grafana-util",
        "observe",
        "resource",
        "describe",
        "dashboards",
        "--output-format",
        "json",
    ]);

    match live_args.command {
        UnifiedCommand::Observe { command } => match command {
            super::ObserveCommand::Live(inner) => assert!(inner.all_orgs),
            _ => panic!("expected observe live"),
        },
        _ => panic!("expected observe command"),
    }

    match overview_args.command {
        UnifiedCommand::Observe { command } => match command {
            super::ObserveCommand::Overview { staged, command } => {
                assert!(command.is_none());
                assert_eq!(
                    staged.dashboard_export_dir.as_deref(),
                    Some(Path::new("./dashboards/raw"))
                );
            }
            _ => panic!("expected observe overview"),
        },
        _ => panic!("expected observe command"),
    }

    match resource_args.command {
        UnifiedCommand::Observe { command } => match command {
            super::ObserveCommand::Resource { command } => match command {
                ResourceCommand::Describe(inner) => {
                    assert_eq!(inner.kind, Some(ResourceKind::Dashboards));
                    assert_eq!(inner.output_format, ResourceOutputFormat::Json);
                }
                _ => panic!("expected observe resource describe"),
            },
            _ => panic!("expected observe resource"),
        },
        _ => panic!("expected observe command"),
    }
}

#[test]
fn parse_cli_supports_config_profile_surface() {
    let args: CliArgs = parse_cli_from([
        "grafana-util",
        "config",
        "profile",
        "show",
        "--profile",
        "prod",
        "--output-format",
        "yaml",
    ]);

    match args.command {
        UnifiedCommand::Config { command } => match command {
            super::ConfigCommand::Profile(profile_args) => match profile_args.command {
                ProfileCommand::Show(show_args) => {
                    assert_eq!(show_args.profile.as_deref(), Some("prod"));
                    assert_eq!(show_args.output_format, SimpleOutputFormat::Yaml);
                }
                _ => panic!("expected config profile show"),
            },
        },
        _ => panic!("expected config command"),
    }
}

#[test]
fn parse_cli_supports_task_first_export_surface() {
    let dashboard_args: CliArgs = parse_cli_from([
        "grafana-util",
        "export",
        "dashboard",
        "--output-dir",
        "./dashboards",
        "--overwrite",
    ]);
    let access_args: CliArgs = parse_cli_from([
        "grafana-util",
        "export",
        "access",
        "service-account",
        "--output-dir",
        "./access-service-accounts",
        "--overwrite",
    ]);

    match dashboard_args.command {
        UnifiedCommand::Export { command } => match command {
            super::ExportCommand::Dashboard(inner) => {
                assert_eq!(inner.output_dir, Path::new("./dashboards"));
                assert!(inner.overwrite);
            }
            _ => panic!("expected export dashboard"),
        },
        _ => panic!("expected export command"),
    }

    match access_args.command {
        UnifiedCommand::Export { command } => match command {
            super::ExportCommand::Access { command } => match command {
                super::ExportAccessCommand::ServiceAccount(inner) => {
                    assert_eq!(inner.output_dir, Path::new("./access-service-accounts"));
                    assert!(inner.overwrite);
                }
                _ => panic!("expected export access service-account"),
            },
            _ => panic!("expected export access"),
        },
        _ => panic!("expected export command"),
    }
}

#[test]
fn parse_cli_supports_advanced_dashboard_surface() {
    let args: CliArgs = parse_cli_from([
        "grafana-util",
        "advanced",
        "dashboard",
        "sync",
        "import",
        "--input-dir",
        "./dashboards/raw",
        "--dry-run",
        "--table",
    ]);

    match args.command {
        UnifiedCommand::Advanced { command } => match command {
            super::AdvancedCommand::Dashboard { command } => match command {
                super::DashboardGroupCommand::Sync { command } => match command {
                    super::DashboardSyncCommand::Import(inner) => {
                        assert_eq!(inner.input_dir, Path::new("./dashboards/raw"));
                        assert!(inner.dry_run);
                        assert!(inner.table);
                    }
                    _ => panic!("expected advanced dashboard sync import"),
                },
                _ => panic!("expected advanced dashboard sync"),
            },
            _ => panic!("expected advanced dashboard"),
        },
        _ => panic!("expected advanced command"),
    }
}

#[test]
fn parse_cli_supports_grouped_dashboard_surfaces() {
    let live_args: CliArgs = parse_cli_from([
        "grafana-util",
        "dashboard",
        "live",
        "fetch",
        "--dashboard-uid",
        "cpu-main",
        "--output",
        "./cpu-main.json",
    ]);
    let draft_args: CliArgs = parse_cli_from([
        "grafana-util",
        "dashboard",
        "draft",
        "review",
        "--input",
        "./drafts/cpu-main.json",
    ]);
    let sync_args: CliArgs = parse_cli_from([
        "grafana-util",
        "dashboard",
        "sync",
        "export",
        "--output-dir",
        "./dashboards",
        "--overwrite",
    ]);
    let analyze_args: CliArgs = parse_cli_from([
        "grafana-util",
        "dashboard",
        "analyze",
        "summary",
        "--input-dir",
        "./dashboards/raw",
    ]);
    let capture_args: CliArgs = parse_cli_from([
        "grafana-util",
        "dashboard",
        "capture",
        "screenshot",
        "--dashboard-uid",
        "cpu-main",
        "--output",
        "./cpu-main.png",
    ]);

    match live_args.command {
        UnifiedCommand::Dashboard { command } => match command {
            super::DashboardGroupCommand::Live { command } => match command {
                super::DashboardLiveCommand::Fetch(inner) => {
                    assert_eq!(inner.dashboard_uid, "cpu-main");
                    assert_eq!(inner.output, Path::new("./cpu-main.json"));
                }
                _ => panic!("expected dashboard live fetch"),
            },
            _ => panic!("expected dashboard live"),
        },
        _ => panic!("expected dashboard command"),
    }

    match draft_args.command {
        UnifiedCommand::Dashboard { command } => match command {
            super::DashboardGroupCommand::Draft { command } => match command {
                super::DashboardDraftCommand::Review(inner) => {
                    assert_eq!(inner.input, Path::new("./drafts/cpu-main.json"));
                }
                _ => panic!("expected dashboard draft review"),
            },
            _ => panic!("expected dashboard draft"),
        },
        _ => panic!("expected dashboard command"),
    }

    match sync_args.command {
        UnifiedCommand::Dashboard { command } => match command {
            super::DashboardGroupCommand::Sync { command } => match command {
                super::DashboardSyncCommand::Export(inner) => {
                    assert_eq!(inner.output_dir, Path::new("./dashboards"));
                    assert!(inner.overwrite);
                }
                _ => panic!("expected dashboard sync export"),
            },
            _ => panic!("expected dashboard sync"),
        },
        _ => panic!("expected dashboard command"),
    }

    match analyze_args.command {
        UnifiedCommand::Dashboard { command } => match command {
            super::DashboardGroupCommand::Analyze { command } => match command {
                super::DashboardAnalyzeCommand::Summary(inner) => {
                    assert_eq!(
                        inner.input_dir.as_deref(),
                        Some(Path::new("./dashboards/raw"))
                    );
                }
                _ => panic!("expected dashboard analyze summary"),
            },
            _ => panic!("expected dashboard analyze"),
        },
        _ => panic!("expected dashboard command"),
    }

    match capture_args.command {
        UnifiedCommand::Dashboard { command } => match command {
            super::DashboardGroupCommand::Capture { command } => match command {
                super::DashboardCaptureCommand::Screenshot(inner) => {
                    assert_eq!(inner.dashboard_uid.as_deref(), Some("cpu-main"));
                    assert_eq!(inner.output, Path::new("./cpu-main.png"));
                }
            },
            _ => panic!("expected dashboard capture"),
        },
        _ => panic!("expected dashboard command"),
    }
}

#[test]
fn parse_cli_supports_grouped_dashboard_convert_surface() {
    let args: CliArgs = parse_cli_from([
        "grafana-util",
        "dashboard",
        "sync",
        "convert",
        "raw-to-prompt",
        "--input-file",
        "./dashboards/raw/cpu-main.json",
    ]);

    match args.command {
        UnifiedCommand::Dashboard { command } => match command {
            super::DashboardGroupCommand::Sync { command } => match command {
                super::DashboardSyncCommand::Convert { command } => match command {
                    super::DashboardSyncConvertCommand::RawToPrompt(inner) => {
                        assert_eq!(
                            inner.input_file,
                            vec![Path::new("./dashboards/raw/cpu-main.json").to_path_buf()]
                        );
                    }
                },
                _ => panic!("expected dashboard sync convert"),
            },
            _ => panic!("expected dashboard sync"),
        },
        _ => panic!("expected dashboard command"),
    }
}

#[test]
fn parse_cli_supports_grouped_alert_surfaces() {
    let migrate_args: CliArgs = parse_cli_from([
        "grafana-util",
        "alert",
        "migrate",
        "export",
        "--output-dir",
        "./alerts",
        "--overwrite",
    ]);
    let author_rule_args: CliArgs = parse_cli_from([
        "grafana-util",
        "alert",
        "author",
        "rule",
        "add",
        "--desired-dir",
        "./alerts/desired",
        "--name",
        "cpu-high",
        "--folder",
        "platform-alerts",
        "--rule-group",
        "cpu",
        "--receiver",
        "pagerduty-primary",
        "--expr",
        "A",
        "--threshold",
        "80",
        "--above",
    ]);
    let scaffold_args: CliArgs = parse_cli_from([
        "grafana-util",
        "alert",
        "scaffold",
        "rule",
        "--desired-dir",
        "./alerts/desired",
        "--name",
        "cpu-main",
    ]);
    let change_args: CliArgs = parse_cli_from([
        "grafana-util",
        "alert",
        "change",
        "plan",
        "--desired-dir",
        "./alerts/desired",
        "--output-format",
        "json",
    ]);

    match migrate_args.command {
        UnifiedCommand::Alert(inner) => match inner.command {
            super::AlertCommandSurface::Migrate { command } => match command {
                super::AlertMigrateCommand::Export(inner) => {
                    assert_eq!(inner.output_dir, Path::new("./alerts"));
                    assert!(inner.overwrite);
                }
                _ => panic!("expected alert migrate export"),
            },
            _ => panic!("expected alert migrate"),
        },
        _ => panic!("expected alert command"),
    }

    match author_rule_args.command {
        UnifiedCommand::Alert(inner) => match inner.command {
            super::AlertCommandSurface::Author { command } => match command {
                super::AlertAuthorCommand::Rule { command } => match command {
                    super::AlertAuthorRuleCommand::Add(inner) => {
                        assert_eq!(inner.base.desired_dir, Path::new("./alerts/desired"));
                        assert_eq!(inner.name, "cpu-high");
                        assert_eq!(inner.folder, "platform-alerts");
                    }
                    _ => panic!("expected alert author rule add"),
                },
                _ => panic!("expected alert author rule"),
            },
            _ => panic!("expected alert author"),
        },
        _ => panic!("expected alert command"),
    }

    match scaffold_args.command {
        UnifiedCommand::Alert(inner) => match inner.command {
            super::AlertCommandSurface::Scaffold { command } => match command {
                super::AlertScaffoldCommand::Rule(inner) => {
                    assert_eq!(inner.desired_dir, Path::new("./alerts/desired"));
                    assert_eq!(inner.name, "cpu-main");
                }
                _ => panic!("expected alert scaffold rule"),
            },
            _ => panic!("expected alert scaffold"),
        },
        _ => panic!("expected alert command"),
    }

    match change_args.command {
        UnifiedCommand::Alert(inner) => match inner.command {
            super::AlertCommandSurface::Change { command } => match command {
                super::AlertChangeCommand::Plan(inner) => {
                    assert_eq!(inner.desired_dir, Path::new("./alerts/desired"));
                    assert_eq!(format!("{:?}", inner.output_format), "Json");
                }
                _ => panic!("expected alert change plan"),
            },
            _ => panic!("expected alert change"),
        },
        _ => panic!("expected alert command"),
    }
}

#[test]
fn docs_describe_grouped_surfaces_without_compatibility_language() {
    let en_index = include_str!("../../docs/commands/en/index.md");
    assert!(en_index.contains("Start Here"));
    assert!(en_index.contains("Common Tasks"));
    assert!(en_index.contains("Advanced Tasks"));
    assert!(en_index.contains("observe"));
    assert!(en_index.contains("export"));
    assert!(en_index.contains("config profile"));
    assert!(!en_index.contains("Compatibility Pages"));
    assert!(!en_index.contains("compatibility aliases remain"));

    let zh_index = include_str!("../../docs/commands/zh-TW/index.md");
    assert!(zh_index.contains("先從這裡開始"));
    assert!(zh_index.contains("常用工作"));
    assert!(zh_index.contains("進階工作"));
    assert!(zh_index.contains("observe"));
    assert!(zh_index.contains("export"));
    assert!(zh_index.contains("config profile"));
    assert!(!zh_index.contains("相容頁面"));
    assert!(!zh_index.contains("相容別名"));
}

#[test]
fn dispatch_routes_observe_live_to_project_status_handler() {
    let routed = RefCell::new(Vec::<String>::new());
    let args: CliArgs = parse_cli_from(["grafana-util", "observe", "live", "--all-orgs"]);

    let result = dispatch_with_handlers(
        args,
        |_dashboard_args| Ok(()),
        |_datasource_args| Ok(()),
        |_sync_args| Ok(()),
        |_alert_args| Ok(()),
        |_access_args| Ok(()),
        |_profile_args| Ok(()),
        |_snapshot_args| Ok(()),
        |_overview_args| Ok(()),
        |_status_args| {
            routed.borrow_mut().push("status".to_string());
            Ok(())
        },
    );

    assert!(result.is_ok());
    assert_eq!(*routed.borrow(), vec!["status".to_string()]);
}

#[test]
fn dispatch_routes_export_dashboard_to_dashboard_handler() {
    let routed = RefCell::new(Vec::<String>::new());
    let args: CliArgs = parse_cli_from([
        "grafana-util",
        "export",
        "dashboard",
        "--output-dir",
        "./dashboards",
    ]);

    let result = dispatch_with_handlers(
        args,
        |_dashboard_args| {
            routed.borrow_mut().push("dashboard".to_string());
            Ok(())
        },
        |_datasource_args| Ok(()),
        |_sync_args| Ok(()),
        |_alert_args| Ok(()),
        |_access_args| Ok(()),
        |_profile_args| Ok(()),
        |_snapshot_args| Ok(()),
        |_overview_args| Ok(()),
        |_status_args| Ok(()),
    );

    assert!(result.is_ok());
    assert_eq!(*routed.borrow(), vec!["dashboard".to_string()]);
}
