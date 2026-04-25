//! Shared status command surface.
//!
//! Maintainer note:
//! - This module owns the top-level `grafana-util status staged/live ...` help and schema surface.
//! - It should stay focused on command args, shared rendering, and high-level
//!   staged/live aggregation handoff.
//! - Domain-specific staged/live producer logic belongs in the owning domain
//!   modules, not here.

mod cli;

use crate::common::{render_json_value, set_json_color_choice, Result as CommonResult};
use crate::overview::{self, OverviewArgs, OverviewOutputFormat};
use crate::project_status::ProjectStatus;
use crate::project_status::{
    render_domain_finding_summary, render_project_status_decision_order,
    render_project_status_signal_summary,
};
use crate::project_status_live_runtime::build_live_project_status;
use crate::project_status_staged::build_staged_project_status;
use crate::sync::render_discovery_summary_from_value;
use crate::tabular_output::{print_lines, render_summary_csv, render_summary_table, render_yaml};
use serde_json::Value;

pub use self::cli::{
    ProjectStatusCliArgs, ProjectStatusLiveArgs, ProjectStatusOutputFormat,
    ProjectStatusStagedArgs, ProjectStatusSubcommand,
};
pub(crate) use self::cli::{PROJECT_STATUS_LIVE_HELP_TEXT, PROJECT_STATUS_STAGED_HELP_TEXT};

pub(crate) const PROJECT_STATUS_DOMAIN_COUNT: usize = 6;

fn staged_args_to_overview_args(args: &ProjectStatusStagedArgs) -> OverviewArgs {
    OverviewArgs {
        dashboard_export_dir: args.dashboard_export_dir.clone(),
        dashboard_provisioning_dir: args.dashboard_provisioning_dir.clone(),
        datasource_export_dir: args.datasource_export_dir.clone(),
        datasource_provisioning_file: args.datasource_provisioning_file.clone(),
        access_user_export_dir: args.access_user_export_dir.clone(),
        access_team_export_dir: args.access_team_export_dir.clone(),
        access_org_export_dir: args.access_org_export_dir.clone(),
        access_service_account_export_dir: args.access_service_account_export_dir.clone(),
        desired_file: args.desired_file.clone(),
        source_bundle: args.source_bundle.clone(),
        target_inventory: args.target_inventory.clone(),
        alert_export_dir: args.alert_export_dir.clone(),
        availability_file: args.availability_file.clone(),
        mapping_file: args.mapping_file.clone(),
        output_format: OverviewOutputFormat::Text,
    }
}

#[cfg(feature = "tui")]
// Interactive rendering path for status documents in TUI.
fn run_project_status_interactive(status: ProjectStatus) -> CommonResult<()> {
    crate::project_status_tui::run_project_status_interactive(status)
}

#[cfg(not(feature = "tui"))]
#[allow(dead_code)]
// Non-TUI fallback keeps all entrypoints compile-time complete.
fn run_project_status_interactive(_status: ProjectStatus) -> CommonResult<()> {
    Err(crate::common::tui(
        "Project-status interactive mode requires the `tui` feature.",
    ))
}

pub(crate) fn render_project_status_text(status: &ProjectStatus) -> Vec<String> {
    let mut lines = vec![
        "Project status".to_string(),
        format!(
            "Overall: status={} scope={} domains={} present={} blocked={} blockers={} warnings={} freshness={}",
            status.overall.status,
            status.scope,
            status.overall.domain_count,
            status.overall.present_count,
            status.overall.blocked_count,
            status.overall.blocker_count,
            status.overall.warning_count,
            status.overall.freshness.status,
        ),
    ];
    if let Some(discovery) = status.discovery.as_ref().and_then(Value::as_object) {
        if let Some(summary) = render_discovery_summary_from_value(discovery) {
            lines.push(summary);
        }
    }
    if let Some(summary) = render_project_status_signal_summary(status) {
        lines.push(summary);
    }
    if let Some(order) = render_project_status_decision_order(status) {
        lines.push("Decision order:".to_string());
        lines.extend(order);
    }
    if !status.domains.is_empty() {
        lines.push("Domains:".to_string());
        for domain in &status.domains {
            let mut line = format!(
                "- {} status={} mode={} primary={} blockers={} warnings={} freshness={}",
                domain.id,
                domain.status,
                domain.mode,
                domain.primary_count,
                domain.blocker_count,
                domain.warning_count,
                domain.freshness.status,
            );
            if let Some(action) = domain.next_actions.first() {
                line.push_str(&format!(" next={action}"));
            }
            if let Some(summary) = render_domain_finding_summary(&domain.blockers) {
                line.push_str(&format!(" blockerKinds={summary}"));
            }
            if let Some(summary) = render_domain_finding_summary(&domain.warnings) {
                line.push_str(&format!(" warningKinds={summary}"));
            }
            lines.push(line);
        }
    }
    if !status.top_blockers.is_empty() {
        lines.push("Top blockers:".to_string());
        for blocker in status.top_blockers.iter().take(5) {
            lines.push(format!(
                "- {} {} count={} source={}",
                blocker.domain, blocker.kind, blocker.count, blocker.source
            ));
        }
    }
    if !status.next_actions.is_empty() {
        lines.push("Next actions:".to_string());
        for action in status.next_actions.iter().take(5) {
            lines.push(format!(
                "- {} reason={} action={}",
                action.domain, action.reason_code, action.action
            ));
        }
    }
    lines
}

pub(crate) fn build_project_status_summary_rows(
    status: &ProjectStatus,
) -> Vec<(&'static str, String)> {
    vec![
        ("status", status.overall.status.clone()),
        ("scope", status.scope.clone()),
        ("domainCount", status.overall.domain_count.to_string()),
        ("presentCount", status.overall.present_count.to_string()),
        ("blockedCount", status.overall.blocked_count.to_string()),
        ("blockerCount", status.overall.blocker_count.to_string()),
        ("warningCount", status.overall.warning_count.to_string()),
        ("freshnessStatus", status.overall.freshness.status.clone()),
        (
            "freshnessSourceCount",
            status.overall.freshness.source_count.to_string(),
        ),
        (
            "freshnessNewestAgeSeconds",
            status
                .overall
                .freshness
                .newest_age_seconds
                .map(|value| value.to_string())
                .unwrap_or_default(),
        ),
        (
            "freshnessOldestAgeSeconds",
            status
                .overall
                .freshness
                .oldest_age_seconds
                .map(|value| value.to_string())
                .unwrap_or_default(),
        ),
        ("topBlockerCount", status.top_blockers.len().to_string()),
        ("nextActionCount", status.next_actions.len().to_string()),
    ]
}

/// Build the staged status document without rendering it.
pub fn execute_project_status_staged(
    args: &ProjectStatusStagedArgs,
) -> CommonResult<ProjectStatus> {
    let overview_args = staged_args_to_overview_args(args);
    let artifacts = overview::build_overview_artifacts(&overview_args)?;
    Ok(build_staged_project_status(&artifacts))
}

/// Build the live status document without rendering it.
pub fn execute_project_status_live(args: &ProjectStatusLiveArgs) -> CommonResult<ProjectStatus> {
    build_live_project_status(args)
}

pub fn run_project_status_staged(args: ProjectStatusStagedArgs) -> CommonResult<()> {
    // Staged status is deterministic and artifact-driven; it never touches live Grafana.
    let status = execute_project_status_staged(&args)?;
    match args.output_format {
        ProjectStatusOutputFormat::Table => {
            print_lines(&render_summary_table(&build_project_status_summary_rows(
                &status,
            )));
            Ok(())
        }
        ProjectStatusOutputFormat::Csv => {
            print_lines(&render_summary_csv(&build_project_status_summary_rows(
                &status,
            )));
            Ok(())
        }
        ProjectStatusOutputFormat::Text => {
            for line in render_project_status_text(&status) {
                println!("{line}");
            }
            Ok(())
        }
        ProjectStatusOutputFormat::Json => {
            println!("{}", render_json_value(&status)?);
            Ok(())
        }
        ProjectStatusOutputFormat::Yaml => {
            println!("{}", render_yaml(&status)?);
            Ok(())
        }
        #[cfg(feature = "tui")]
        ProjectStatusOutputFormat::Interactive => run_project_status_interactive(status),
    }
}

pub fn run_project_status_live(args: ProjectStatusLiveArgs) -> CommonResult<()> {
    // Live status is the operational contract that refreshes live domain state and folds
    // it into the same shared status output schema used by staged mode.
    let status = execute_project_status_live(&args)?;
    match args.output_format {
        ProjectStatusOutputFormat::Table => {
            print_lines(&render_summary_table(&build_project_status_summary_rows(
                &status,
            )));
            Ok(())
        }
        ProjectStatusOutputFormat::Csv => {
            print_lines(&render_summary_csv(&build_project_status_summary_rows(
                &status,
            )));
            Ok(())
        }
        ProjectStatusOutputFormat::Text => {
            for line in render_project_status_text(&status) {
                println!("{line}");
            }
            Ok(())
        }
        ProjectStatusOutputFormat::Json => {
            println!("{}", render_json_value(&status)?);
            Ok(())
        }
        ProjectStatusOutputFormat::Yaml => {
            println!("{}", render_yaml(&status)?);
            Ok(())
        }
        #[cfg(feature = "tui")]
        ProjectStatusOutputFormat::Interactive => run_project_status_interactive(status),
    }
}

pub fn run_project_status_cli(args: ProjectStatusCliArgs) -> CommonResult<()> {
    // CLI boundary: parse color choice, then route to either staged or live runner.
    set_json_color_choice(args.color);
    match args.command {
        ProjectStatusSubcommand::Staged(inner) => run_project_status_staged(inner),
        ProjectStatusSubcommand::Live(inner) => run_project_status_live(inner),
    }
}
