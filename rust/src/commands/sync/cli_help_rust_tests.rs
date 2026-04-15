//! Workspace CLI parser/help test suite.
//! Verifies task-first routing and CI workflow help contracts.
use super::{
    SyncAdvancedCommand, SyncCliArgs, SyncGroupCommand, SyncOutputFormat, DEFAULT_REVIEW_TOKEN,
};
use clap::{CommandFactory, Parser};
use std::path::Path;

fn render_workspace_subcommand_help(name: &str) -> String {
    let mut command = SyncCliArgs::command();
    let subcommand = command
        .find_subcommand_mut(name)
        .unwrap_or_else(|| panic!("missing workspace subcommand help for {name}"));
    let mut output = Vec::new();
    subcommand.write_long_help(&mut output).unwrap();
    String::from_utf8(output).unwrap()
}

fn render_workspace_ci_subcommand_help(name: &str) -> String {
    let mut command = SyncCliArgs::command();
    let ci_command = command
        .find_subcommand_mut("ci")
        .expect("missing workspace ci help");
    let subcommand = ci_command
        .find_subcommand_mut(name)
        .unwrap_or_else(|| panic!("missing workspace ci subcommand help for {name}"));
    let mut output = Vec::new();
    subcommand.write_long_help(&mut output).unwrap();
    String::from_utf8(output).unwrap()
}

fn assert_help_includes(help: &str, expected: &[&str]) {
    for text in expected {
        assert!(help.contains(text), "missing help text: {text}");
    }
}

#[test]
fn workspace_scan_help_includes_examples_and_output_heading() {
    let help = render_workspace_subcommand_help("scan");
    assert_help_includes(&help, &["Examples:", "Output Options"]);
}

#[test]
fn workspace_test_help_includes_examples_and_live_heading() {
    let help = render_workspace_subcommand_help("test");
    assert_help_includes(
        &help,
        &["Examples:", "Input Options", "Live Options", "--fetch-live"],
    );
}

#[test]
fn workspace_preview_help_includes_examples_and_live_heading() {
    let help = render_workspace_subcommand_help("preview");
    assert_help_includes(
        &help,
        &["Examples:", "Input Options", "Live Options", "--fetch-live"],
    );
}

#[test]
fn workspace_apply_help_includes_examples_and_approval_flags() {
    let help = render_workspace_subcommand_help("apply");
    assert_help_includes(
        &help,
        &[
            "Examples:",
            "Approval Options",
            "Live Options",
            "--approve",
            "--execute-live",
            "--allow-folder-delete",
            "--allow-policy-reset",
            "--preview-file",
        ],
    );
}

#[test]
fn workspace_ci_help_mentions_lower_level_workflows() {
    let help = render_workspace_subcommand_help("ci");
    assert_help_includes(
        &help,
        &[
            "summary",
            "mark-reviewed",
            "input-test",
            "package-test",
            "promote-test",
        ],
    );
}

#[test]
fn workspace_ci_audit_help_mentions_lock_and_drift_controls() {
    let help = render_workspace_ci_subcommand_help("audit");
    assert_help_includes(
        &help,
        &[
            "--managed-file",
            "--lock-file",
            "--write-lock",
            "--fail-on-drift",
            "--interactive",
        ],
    );
}

#[test]
fn workspace_ci_review_help_mentions_interactive_review() {
    let help = render_workspace_ci_subcommand_help("mark-reviewed");
    assert_help_includes(&help, &["--interactive"]);
}

#[test]
fn workspace_ci_package_test_help_includes_examples_and_grouped_headings() {
    let help = render_workspace_ci_subcommand_help("package-test");
    assert_help_includes(
        &help,
        &[
            "Examples:",
            "Input Options",
            "Live Options",
            "--availability-file",
            "secretPlaceholderNames",
            "\"providerNames\": [\"vault\"]",
        ],
    );
}

#[test]
fn workspace_ci_promote_test_help_includes_mapping_input() {
    let help = render_workspace_ci_subcommand_help("promote-test");
    assert_help_includes(
        &help,
        &[
            "Examples:",
            "Input Options",
            "staged review handoff",
            "--mapping-file",
            "--availability-file",
            "grafana-utils-sync-promotion-mapping",
        ],
    );
}

#[test]
fn workspace_package_help_includes_examples_and_output_heading() {
    let help = render_workspace_subcommand_help("package");
    assert_help_includes(
        &help,
        &[
            "Examples:",
            "grafana-util workspace package ./grafana-oac-repo",
            "--dashboard-export-dir",
            "--dashboard-provisioning-dir",
            "--output-file",
            "--also-stdout",
            "Mixed workspace package handoff",
            "workspace-package.json",
        ],
    );
}

#[test]
fn workspace_package_help_includes_workspace_example() {
    let help = render_workspace_subcommand_help("package");
    assert_help_includes(
        &help,
        &[
            "Examples:",
            "grafana-util workspace package ./grafana-oac-repo",
            "workspace-package.json",
        ],
    );
}

#[test]
fn workspace_root_help_includes_task_first_examples() {
    let mut command = SyncCliArgs::command();
    let mut output = Vec::new();
    command.write_long_help(&mut output).unwrap();
    let help = String::from_utf8(output).unwrap();

    assert_help_includes(
        &help,
        &[
            "Git Sync dashboards",
            "source provenance",
            "./grafana-oac-repo/",
            "dashboards/git-sync/provisioning",
            "grafana-util workspace scan ./grafana-oac-repo",
            "grafana-util workspace test ./grafana-oac-repo",
            "grafana-util workspace preview ./grafana-oac-repo",
            "alerts/raw",
            "datasources/provisioning",
            "grafana-util workspace scan",
            "grafana-util workspace test",
            "grafana-util workspace preview",
            "grafana-util workspace apply",
            "grafana-util workspace package",
            "grafana-util workspace ci package-test",
            "grafana-util workspace ci promote-test",
        ],
    );
}

#[test]
fn parse_workspace_cli_supports_package_workspace_command() {
    let args = SyncCliArgs::parse_from([
        "grafana-util",
        "package",
        "./grafana-oac-repo",
        "--output-file",
        "./sync-source-bundle.json",
    ]);

    match args.command {
        SyncGroupCommand::Bundle(inner) => {
            assert_eq!(
                inner.workspace,
                Some(Path::new("./grafana-oac-repo").to_path_buf())
            );
            assert_eq!(
                inner.output_file,
                Some(Path::new("./sync-source-bundle.json").to_path_buf())
            );
        }
        other => panic!("expected top-level bundle command, got {other:?}"),
    }
}

#[test]
fn parse_workspace_cli_supports_scan_command() {
    let args = SyncCliArgs::parse_from([
        "grafana-util",
        "scan",
        "./grafana-oac-repo",
        "--dashboard-export-dir",
        "./dashboards/raw",
        "--output-format",
        "json",
    ]);

    match args.command {
        SyncGroupCommand::Inspect(inner) => {
            assert_eq!(
                inner.inputs.dashboard_export_dir,
                Some(Path::new("./dashboards/raw").to_path_buf())
            );
            assert_eq!(inner.output.output_format, SyncOutputFormat::Json);
        }
        _ => panic!("expected scan"),
    }
}

#[test]
fn parse_workspace_cli_supports_test_command() {
    let args = SyncCliArgs::parse_from([
        "grafana-util",
        "test",
        "./grafana-oac-repo",
        "--source-bundle",
        "./bundle.json",
        "--target-inventory",
        "./target.json",
        "--mapping-file",
        "./mapping.json",
        "--fetch-live",
        "--output-format",
        "json",
    ]);

    match args.command {
        SyncGroupCommand::Check(inner) => {
            assert_eq!(
                inner.inputs.source_bundle,
                Some(Path::new("./bundle.json").to_path_buf())
            );
            assert_eq!(
                inner.target_inventory,
                Some(Path::new("./target.json").to_path_buf())
            );
            assert_eq!(
                inner.mapping_file,
                Some(Path::new("./mapping.json").to_path_buf())
            );
            assert!(inner.fetch_live);
            assert_eq!(inner.output.output_format, SyncOutputFormat::Json);
        }
        _ => panic!("expected test"),
    }
}

#[test]
fn parse_workspace_cli_supports_preview_command() {
    let args = SyncCliArgs::parse_from([
        "grafana-util",
        "preview",
        "./grafana-oac-repo",
        "--desired-file",
        "./desired.json",
        "--live-file",
        "./live.json",
        "--allow-prune",
        "--trace-id",
        "trace-explicit",
        "--output-format",
        "json",
    ]);

    match args.command {
        SyncGroupCommand::Preview(inner) => {
            assert_eq!(
                inner.inputs.desired_file,
                Some(Path::new("./desired.json").to_path_buf())
            );
            assert_eq!(
                inner.live_file,
                Some(Path::new("./live.json").to_path_buf())
            );
            assert!(inner.allow_prune);
            assert_eq!(inner.output.output_format, SyncOutputFormat::Json);
            assert_eq!(inner.trace_id, Some("trace-explicit".to_string()));
        }
        _ => panic!("expected preview"),
    }
}

#[test]
fn parse_workspace_cli_supports_preview_fetch_live_mode() {
    let args = SyncCliArgs::parse_from([
        "grafana-util",
        "preview",
        "./grafana-oac-repo",
        "--desired-file",
        "./desired.json",
        "--fetch-live",
        "--org-id",
        "7",
        "--page-size",
        "250",
        "--token",
        "test-token",
    ]);

    match args.command {
        SyncGroupCommand::Preview(inner) => {
            assert_eq!(
                inner.inputs.desired_file,
                Some(Path::new("./desired.json").to_path_buf())
            );
            assert_eq!(inner.live_file, None);
            assert!(inner.fetch_live);
            assert_eq!(inner.org_id, Some(7));
            assert_eq!(inner.page_size, 250);
        }
        _ => panic!("expected preview"),
    }
}

#[test]
fn parse_workspace_cli_supports_apply_command() {
    let args = SyncCliArgs::parse_from([
        "grafana-util",
        "apply",
        "--preview-file",
        "./plan.json",
        "--input-test-file",
        "./preflight.json",
        "--package-test-file",
        "./bundle-preflight.json",
        "--approve",
        "--output-format",
        "json",
    ]);

    match args.command {
        SyncGroupCommand::Apply(inner) => {
            assert_eq!(inner.plan_file.as_deref(), Some(Path::new("./plan.json")));
            assert_eq!(
                inner.preflight_file,
                Some(Path::new("./preflight.json").to_path_buf())
            );
            assert_eq!(
                inner.bundle_preflight_file,
                Some(Path::new("./bundle-preflight.json").to_path_buf())
            );
            assert!(inner.approve);
            assert_eq!(inner.output_format, SyncOutputFormat::Json);
        }
        _ => panic!("expected apply"),
    }
}

#[test]
fn parse_workspace_cli_supports_apply_execute_live_flags() {
    let args = SyncCliArgs::parse_from([
        "grafana-util",
        "apply",
        "--plan-file",
        "./plan.json",
        "--approve",
        "--execute-live",
        "--allow-folder-delete",
        "--allow-policy-reset",
        "--org-id",
        "9",
        "--token",
        "test-token",
    ]);

    match args.command {
        SyncGroupCommand::Apply(inner) => {
            assert_eq!(inner.plan_file.as_deref(), Some(Path::new("./plan.json")));
            assert!(inner.execute_live);
            assert!(inner.allow_folder_delete);
            assert!(inner.allow_policy_reset);
            assert_eq!(inner.org_id, Some(9));
        }
        _ => panic!("expected apply"),
    }
}

#[test]
fn parse_workspace_cli_supports_ci_review_command() {
    let args = SyncCliArgs::parse_from([
        "grafana-util",
        "ci",
        "mark-reviewed",
        "--plan-file",
        "./plan.json",
        "--output-format",
        "json",
    ]);

    match args.command {
        SyncGroupCommand::Advanced(inner) => match inner.command {
            SyncAdvancedCommand::Review(inner) => {
                assert_eq!(inner.plan_file, Path::new("./plan.json"));
                assert_eq!(inner.review_token, DEFAULT_REVIEW_TOKEN);
                assert_eq!(inner.output_format, SyncOutputFormat::Json);
            }
            _ => panic!("expected workspace ci mark-reviewed"),
        },
        _ => panic!("expected workspace ci"),
    }
}

#[test]
fn parse_workspace_cli_supports_ci_audit_command() {
    let args = SyncCliArgs::parse_from([
        "grafana-util",
        "ci",
        "audit",
        "--managed-file",
        "./desired.json",
        "--lock-file",
        "./sync-lock.json",
        "--live-file",
        "./live.json",
        "--write-lock",
        "./next-lock.json",
        "--fail-on-drift",
        "--interactive",
        "--output-format",
        "json",
    ]);

    match args.command {
        SyncGroupCommand::Advanced(inner) => match inner.command {
            SyncAdvancedCommand::Audit(inner) => {
                assert_eq!(inner.managed_file.unwrap(), Path::new("./desired.json"));
                assert_eq!(inner.lock_file.unwrap(), Path::new("./sync-lock.json"));
                assert_eq!(inner.live_file.unwrap(), Path::new("./live.json"));
                assert_eq!(inner.write_lock.unwrap(), Path::new("./next-lock.json"));
                assert!(inner.fail_on_drift);
                assert!(inner.interactive);
                assert_eq!(inner.output_format, SyncOutputFormat::Json);
            }
            _ => panic!("expected workspace ci audit"),
        },
        _ => panic!("expected workspace ci"),
    }
}

#[test]
fn parse_workspace_cli_supports_ci_promote_test_command() {
    let args = SyncCliArgs::parse_from([
        "grafana-util",
        "ci",
        "promote-test",
        "--source-bundle",
        "./bundle.json",
        "--target-inventory",
        "./target.json",
        "--mapping-file",
        "./promotion-map.json",
        "--output-format",
        "json",
    ]);

    match args.command {
        SyncGroupCommand::Advanced(inner) => match inner.command {
            SyncAdvancedCommand::PromotionPreflight(inner) => {
                assert_eq!(inner.source_bundle, Path::new("./bundle.json"));
                assert_eq!(inner.target_inventory, Path::new("./target.json"));
                assert_eq!(
                    inner.mapping_file,
                    Some(Path::new("./promotion-map.json").to_path_buf())
                );
                assert_eq!(inner.output_format, SyncOutputFormat::Json);
            }
            _ => panic!("expected workspace ci promote-test"),
        },
        _ => panic!("expected workspace ci"),
    }
}
