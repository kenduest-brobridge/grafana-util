//! Unified CLI dispatcher for Rust entrypoints.
//!
//! Purpose:
//! - Own only command topology, legacy alias normalization, and domain dispatch.
//! - Keep `grafana-util` and compatibility aliases in one place.
//! - Route to domain runners (`dashboard`, `alert`, `access`, `datasource`) without
//!   carrying transport/request behavior.
//!
//! Flow:
//! - Parse into `CliArgs` via Clap.
//! - Normalize legacy and namespaced command forms into one domain command enum.
//! - Delegate execution to the selected domain runner function.
//!
//! Caveats:
//! - Do not add domain logic or HTTP transport details here.
//! - Keep compatibility aliases minimal so deprecation windows are easy to track.
use clap::{Parser, Subcommand};

use crate::access::{run_access_cli, AccessCliArgs};
use crate::alert::{
    normalize_alert_group_command, normalize_alert_namespace_args, run_alert_cli, AlertCliArgs,
    AlertDiffArgs, AlertExportArgs, AlertImportArgs, AlertListArgs, AlertNamespaceArgs,
};
use crate::common::Result;
use crate::dashboard::{
    run_dashboard_cli, DashboardCliArgs, DashboardCommand, DiffArgs, ExportArgs, ImportArgs,
    InspectExportArgs, InspectLiveArgs, ListArgs, ListDataSourcesArgs,
};
use crate::datasource::{run_datasource_cli, DatasourceGroupCommand};
use crate::sync::{run_sync_cli, SyncGroupCommand};

const UNIFIED_HELP_TEXT: &str = "Examples:\n\n  Export dashboards across all visible orgs with Basic auth:\n    grafana-util dashboard export --url http://localhost:3000 --basic-user admin --basic-password admin --all-orgs --export-dir ./dashboards --overwrite\n\n  Preview a routed dashboard import before writing:\n    grafana-util dashboard import --url http://localhost:3000 --basic-user admin --basic-password admin --import-dir ./dashboards --use-export-org --create-missing-orgs --dry-run --output-format table\n\n  Inspect exported dashboards as a query tree:\n    grafana-util dashboard inspect-export --import-dir ./dashboards/raw --view query --layout tree --format table\n\n  Export alerting resources from the current org with an API token:\n    grafana-util alert export --url http://localhost:3000 --token \"$GRAFANA_API_TOKEN\" --output-dir ./alerts --overwrite\n\n  List Grafana organizations with memberships:\n    grafana-util access org list --url http://localhost:3000 --basic-user admin --basic-password admin --with-users --table\n\n  List current-org teams with member details:\n    grafana-util access team list --url http://localhost:3000 --basic-user admin --basic-password admin --with-members --table\n\n  Build a local staged sync preflight document:\n    grafana-util sync preflight --desired-file ./desired.json --availability-file ./availability.json";
const LIST_ALIAS_HELP: &str =
    "Examples:\n\n  Preferred namespaced form:\n    grafana-util dashboard list --url http://localhost:3000 --table\n\n  Compatibility alias form:\n    grafana-util list --url http://localhost:3000 --table";
const LIST_DATASOURCES_ALIAS_HELP: &str =
    "Examples:\n\n  Preferred namespaced form:\n    grafana-util datasource list --url http://localhost:3000 --table\n\n  Compatibility alias form:\n    grafana-util list-data-sources --url http://localhost:3000 --table";
const DASHBOARD_LIST_DATASOURCES_HELP: &str =
    "Examples:\n\n  Preferred datasource namespace:\n    grafana-util datasource list --url http://localhost:3000 --table\n\n  Compatibility dashboard form:\n    grafana-util dashboard list-data-sources --url http://localhost:3000 --table";
const DASHBOARD_LIST_HELP: &str =
    "Examples:\n\n  Table output:\n    grafana-util dashboard list --url http://localhost:3000 --table\n\n  JSON output for one folder:\n    grafana-util dashboard list --url http://localhost:3000 --token \"$GRAFANA_API_TOKEN\" --folder Infra --output-format json";
const DASHBOARD_EXPORT_HELP: &str =
    "Examples:\n\n  Export dashboards with Basic auth:\n    grafana-util dashboard export --url http://localhost:3000 --basic-user admin --basic-password admin --export-dir ./dashboards --overwrite\n\n  Export into a flat directory layout:\n    grafana-util dashboard export --url http://localhost:3000 --basic-user admin --basic-password admin --export-dir ./dashboards --flat";
const DASHBOARD_IMPORT_HELP: &str =
    "Examples:\n\n  Preview a dashboard import:\n    grafana-util dashboard import --url http://localhost:3000 --basic-user admin --basic-password admin --import-dir ./dashboards/raw --replace-existing --dry-run --output-format table\n\n  Replay a multi-org export into matching orgs:\n    grafana-util dashboard import --url http://localhost:3000 --basic-user admin --basic-password admin --import-dir ./dashboards --use-export-org --create-missing-orgs";
const DASHBOARD_DIFF_HELP: &str =
    "Examples:\n\n  Diff raw dashboard exports:\n    grafana-util dashboard diff --url http://localhost:3000 --token \"$GRAFANA_API_TOKEN\" --import-dir ./dashboards/raw";
const DASHBOARD_INSPECT_EXPORT_HELP: &str =
    "Examples:\n\n  Render a query tree report from exported dashboards:\n    grafana-util dashboard inspect-export --import-dir ./dashboards/raw --view query --layout tree --format table\n\n  Render a datasource report as JSON:\n    grafana-util dashboard inspect-export --import-dir ./dashboards/raw --view datasource --format json";
const DASHBOARD_INSPECT_LIVE_HELP: &str =
    "Examples:\n\n  Inspect live dashboards as a query tree:\n    grafana-util dashboard inspect-live --url http://localhost:3000 --token \"$GRAFANA_API_TOKEN\" --view query --layout tree --format text\n\n  Render a live governance report:\n    grafana-util dashboard inspect-live --url http://localhost:3000 --token \"$GRAFANA_API_TOKEN\" --view governance --format table";
const EXPORT_ALIAS_HELP: &str =
    "Examples:\n\n  Preferred namespaced form:\n    grafana-util dashboard export --url http://localhost:3000 --export-dir ./dashboards --overwrite\n\n  Compatibility alias form:\n    grafana-util export --url http://localhost:3000 --export-dir ./dashboards --overwrite";
const IMPORT_ALIAS_HELP: &str =
    "Examples:\n\n  Preferred namespaced form:\n    grafana-util dashboard import --url http://localhost:3000 --import-dir ./dashboards/raw --replace-existing --dry-run\n\n  Compatibility alias form:\n    grafana-util import --url http://localhost:3000 --import-dir ./dashboards/raw --replace-existing --dry-run";
const DIFF_ALIAS_HELP: &str =
    "Examples:\n\n  Preferred namespaced form:\n    grafana-util dashboard diff --url http://localhost:3000 --import-dir ./dashboards/raw\n\n  Compatibility alias form:\n    grafana-util diff --url http://localhost:3000 --import-dir ./dashboards/raw";
const INSPECT_EXPORT_ALIAS_HELP: &str =
    "Examples:\n\n  Preferred namespaced form:\n    grafana-util dashboard inspect-export --import-dir ./dashboards/raw --output-format report-table\n\n  Compatibility alias form:\n    grafana-util inspect-export --import-dir ./dashboards/raw --output-format report-table";
const INSPECT_LIVE_ALIAS_HELP: &str =
    "Examples:\n\n  Preferred namespaced form:\n    grafana-util dashboard inspect-live --url http://localhost:3000 --output-format governance-json\n\n  Compatibility alias form:\n    grafana-util inspect-live --url http://localhost:3000 --output-format governance-json";
const ALERT_EXPORT_ALIAS_HELP: &str =
    "Examples:\n\n  Preferred namespaced form:\n    grafana-util alert export --url http://localhost:3000 --output-dir ./alerts --overwrite\n\n  Compatibility alias form:\n    grafana-util export-alert --url http://localhost:3000 --output-dir ./alerts --overwrite";
const ALERT_IMPORT_ALIAS_HELP: &str =
    "Examples:\n\n  Preferred namespaced form:\n    grafana-util alert import --url http://localhost:3000 --import-dir ./alerts/raw --replace-existing --dry-run\n\n  Compatibility alias form:\n    grafana-util import-alert --url http://localhost:3000 --import-dir ./alerts/raw --replace-existing --dry-run";
const ALERT_DIFF_ALIAS_HELP: &str =
    "Examples:\n\n  Preferred namespaced form:\n    grafana-util alert diff --url http://localhost:3000 --diff-dir ./alerts/raw\n\n  Compatibility alias form:\n    grafana-util diff-alert --url http://localhost:3000 --diff-dir ./alerts/raw";
const ALERT_LIST_RULES_ALIAS_HELP: &str =
    "Examples:\n\n  Preferred namespaced form:\n    grafana-util alert list-rules --url http://localhost:3000 --table\n\n  Compatibility alias form:\n    grafana-util list-alert-rules --url http://localhost:3000 --table";
const ALERT_LIST_CONTACT_POINTS_ALIAS_HELP: &str =
    "Examples:\n\n  Preferred namespaced form:\n    grafana-util alert list-contact-points --url http://localhost:3000 --table\n\n  Compatibility alias form:\n    grafana-util list-alert-contact-points --url http://localhost:3000 --table";
const ALERT_LIST_MUTE_TIMINGS_ALIAS_HELP: &str =
    "Examples:\n\n  Preferred namespaced form:\n    grafana-util alert list-mute-timings --url http://localhost:3000 --table\n\n  Compatibility alias form:\n    grafana-util list-alert-mute-timings --url http://localhost:3000 --table";
const ALERT_LIST_TEMPLATES_ALIAS_HELP: &str =
    "Examples:\n\n  Preferred namespaced form:\n    grafana-util alert list-templates --url http://localhost:3000 --table\n\n  Compatibility alias form:\n    grafana-util list-alert-templates --url http://localhost:3000 --table";

#[derive(Debug, Clone, Subcommand)]
pub enum DashboardGroupCommand {
    #[command(
        visible_alias = "list-dashboard",
        about = "List dashboard summaries without writing export files.",
        after_help = DASHBOARD_LIST_HELP
    )]
    List(ListArgs),
    #[command(
        name = "list-data-sources",
        about = "Compatibility alias; prefer `grafana-util datasource list`.",
        after_help = DASHBOARD_LIST_DATASOURCES_HELP
    )]
    ListDataSources(ListDataSourcesArgs),
    #[command(
        visible_alias = "export-dashboard",
        about = "Export dashboards to raw/ and prompt/ JSON files.",
        after_help = DASHBOARD_EXPORT_HELP
    )]
    Export(ExportArgs),
    #[command(
        visible_alias = "import-dashboard",
        about = "Import dashboard JSON files through the Grafana API.",
        after_help = DASHBOARD_IMPORT_HELP
    )]
    Import(ImportArgs),
    #[command(
        about = "Compare local raw dashboard files against live Grafana dashboards.",
        after_help = DASHBOARD_DIFF_HELP
    )]
    Diff(DiffArgs),
    #[command(
        about = "Analyze a raw dashboard export directory and summarize its structure.",
        after_help = DASHBOARD_INSPECT_EXPORT_HELP
    )]
    InspectExport(InspectExportArgs),
    #[command(
        about = "Analyze live Grafana dashboards without writing a persistent export.",
        after_help = DASHBOARD_INSPECT_LIVE_HELP
    )]
    InspectLive(InspectLiveArgs),
}

#[derive(Debug, Clone, Subcommand)]
pub enum UnifiedCommand {
    #[command(about = "Run dashboard export, list, import, and diff workflows.")]
    Dashboard {
        #[command(subcommand)]
        command: DashboardGroupCommand,
    },
    #[command(about = "Run datasource list, export, import, and diff workflows.")]
    Datasource {
        #[command(subcommand)]
        command: DatasourceGroupCommand,
    },
    #[command(about = "Run local/document-only sync summary and preflight workflows.")]
    Sync {
        #[command(subcommand)]
        command: SyncGroupCommand,
    },
    #[command(about = "Compatibility alias; prefer `grafana-util dashboard list`.", after_help = LIST_ALIAS_HELP)]
    List(ListArgs),
    #[command(
        name = "list-data-sources",
        about = "Compatibility alias; prefer `grafana-util datasource list`.",
        after_help = LIST_DATASOURCES_ALIAS_HELP
    )]
    ListDataSources(ListDataSourcesArgs),
    #[command(about = "Compatibility alias; prefer `grafana-util dashboard export`.", after_help = EXPORT_ALIAS_HELP)]
    Export(ExportArgs),
    #[command(about = "Compatibility alias; prefer `grafana-util dashboard import`.", after_help = IMPORT_ALIAS_HELP)]
    Import(ImportArgs),
    #[command(about = "Compatibility alias; prefer `grafana-util dashboard diff`.", after_help = DIFF_ALIAS_HELP)]
    Diff(DiffArgs),
    #[command(about = "Compatibility alias; prefer `grafana-util dashboard inspect-export`.", after_help = INSPECT_EXPORT_ALIAS_HELP)]
    InspectExport(InspectExportArgs),
    #[command(about = "Compatibility alias; prefer `grafana-util dashboard inspect-live`.", after_help = INSPECT_LIVE_ALIAS_HELP)]
    InspectLive(InspectLiveArgs),
    #[command(about = "Export, import, or diff Grafana alerting resources.")]
    Alert(AlertNamespaceArgs),
    #[command(
        name = "export-alert",
        about = "Compatibility alias; prefer `grafana-util alert export`.",
        after_help = ALERT_EXPORT_ALIAS_HELP
    )]
    ExportAlert(AlertExportArgs),
    #[command(
        name = "import-alert",
        about = "Compatibility alias; prefer `grafana-util alert import`.",
        after_help = ALERT_IMPORT_ALIAS_HELP
    )]
    ImportAlert(AlertImportArgs),
    #[command(
        name = "diff-alert",
        about = "Compatibility alias; prefer `grafana-util alert diff`.",
        after_help = ALERT_DIFF_ALIAS_HELP
    )]
    DiffAlert(AlertDiffArgs),
    #[command(
        name = "list-alert-rules",
        about = "Compatibility alias; prefer `grafana-util alert list-rules`.",
        after_help = ALERT_LIST_RULES_ALIAS_HELP
    )]
    ListAlertRules(AlertListArgs),
    #[command(
        name = "list-alert-contact-points",
        about = "Compatibility alias; prefer `grafana-util alert list-contact-points`.",
        after_help = ALERT_LIST_CONTACT_POINTS_ALIAS_HELP
    )]
    ListAlertContactPoints(AlertListArgs),
    #[command(
        name = "list-alert-mute-timings",
        about = "Compatibility alias; prefer `grafana-util alert list-mute-timings`.",
        after_help = ALERT_LIST_MUTE_TIMINGS_ALIAS_HELP
    )]
    ListAlertMuteTimings(AlertListArgs),
    #[command(
        name = "list-alert-templates",
        about = "Compatibility alias; prefer `grafana-util alert list-templates`.",
        after_help = ALERT_LIST_TEMPLATES_ALIAS_HELP
    )]
    ListAlertTemplates(AlertListArgs),
    #[command(about = "List and manage Grafana users, teams, and service accounts.")]
    Access(AccessCliArgs),
}

#[derive(Debug, Clone, Parser)]
#[command(
    name = "grafana-util",
    about = "Unified Grafana dashboard, alerting, and access utility.",
    after_help = UNIFIED_HELP_TEXT
)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: UnifiedCommand,
}

/// Parse raw argv into the unified command tree.
///
/// This is intentionally side-effect-free and should only validate CLI shape.
pub fn parse_cli_from<I, T>(iter: I) -> CliArgs
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    CliArgs::parse_from(iter)
}

fn wrap_dashboard(command: DashboardCommand) -> DashboardCliArgs {
    DashboardCliArgs { command }
}

fn wrap_dashboard_group(command: DashboardGroupCommand) -> DashboardCliArgs {
    match command {
        DashboardGroupCommand::List(inner) => wrap_dashboard(DashboardCommand::List(inner)),
        DashboardGroupCommand::ListDataSources(inner) => {
            wrap_dashboard(DashboardCommand::ListDataSources(inner))
        }
        DashboardGroupCommand::Export(inner) => wrap_dashboard(DashboardCommand::Export(inner)),
        DashboardGroupCommand::Import(inner) => wrap_dashboard(DashboardCommand::Import(inner)),
        DashboardGroupCommand::Diff(inner) => wrap_dashboard(DashboardCommand::Diff(inner)),
        DashboardGroupCommand::InspectExport(inner) => {
            wrap_dashboard(DashboardCommand::InspectExport(inner))
        }
        DashboardGroupCommand::InspectLive(inner) => {
            wrap_dashboard(DashboardCommand::InspectLive(inner))
        }
    }
}

// Centralized command fan-out before invoking domain runners.
// Every unified CLI variant is normalized into one of dashboard/alert/datasource/access runners here.
fn dispatch_with_handlers<FD, FS, FY, FA, FX>(
    args: CliArgs,
    mut run_dashboard: FD,
    mut run_datasource: FS,
    mut run_sync: FY,
    mut run_alert: FA,
    mut run_access: FX,
) -> Result<()>
where
    FD: FnMut(DashboardCliArgs) -> Result<()>,
    FS: FnMut(DatasourceGroupCommand) -> Result<()>,
    FY: FnMut(SyncGroupCommand) -> Result<()>,
    FA: FnMut(AlertCliArgs) -> Result<()>,
    FX: FnMut(AccessCliArgs) -> Result<()>,
{
    match args.command {
        UnifiedCommand::Dashboard { command } => run_dashboard(wrap_dashboard_group(command)),
        UnifiedCommand::Datasource { command } => run_datasource(command),
        UnifiedCommand::Sync { command } => run_sync(command),
        UnifiedCommand::List(inner) => run_dashboard(wrap_dashboard(DashboardCommand::List(inner))),
        UnifiedCommand::ListDataSources(inner) => {
            run_dashboard(wrap_dashboard(DashboardCommand::ListDataSources(inner)))
        }
        UnifiedCommand::Export(inner) => {
            run_dashboard(wrap_dashboard(DashboardCommand::Export(inner)))
        }
        UnifiedCommand::Import(inner) => {
            run_dashboard(wrap_dashboard(DashboardCommand::Import(inner)))
        }
        UnifiedCommand::Diff(inner) => run_dashboard(wrap_dashboard(DashboardCommand::Diff(inner))),
        UnifiedCommand::InspectExport(inner) => {
            run_dashboard(wrap_dashboard(DashboardCommand::InspectExport(inner)))
        }
        UnifiedCommand::InspectLive(inner) => {
            run_dashboard(wrap_dashboard(DashboardCommand::InspectLive(inner)))
        }
        UnifiedCommand::Alert(inner) => run_alert(normalize_alert_namespace_args(inner)),
        UnifiedCommand::ExportAlert(inner) => run_alert(normalize_alert_group_command(
            crate::alert::AlertGroupCommand::Export(inner),
        )),
        UnifiedCommand::ImportAlert(inner) => run_alert(normalize_alert_group_command(
            crate::alert::AlertGroupCommand::Import(inner),
        )),
        UnifiedCommand::DiffAlert(inner) => run_alert(normalize_alert_group_command(
            crate::alert::AlertGroupCommand::Diff(inner),
        )),
        UnifiedCommand::ListAlertRules(inner) => run_alert(normalize_alert_group_command(
            crate::alert::AlertGroupCommand::ListRules(inner),
        )),
        UnifiedCommand::ListAlertContactPoints(inner) => run_alert(normalize_alert_group_command(
            crate::alert::AlertGroupCommand::ListContactPoints(inner),
        )),
        UnifiedCommand::ListAlertMuteTimings(inner) => run_alert(normalize_alert_group_command(
            crate::alert::AlertGroupCommand::ListMuteTimings(inner),
        )),
        UnifiedCommand::ListAlertTemplates(inner) => run_alert(normalize_alert_group_command(
            crate::alert::AlertGroupCommand::ListTemplates(inner),
        )),
        UnifiedCommand::Access(inner) => run_access(inner),
    }
}

/// Runtime entrypoint for unified execution.
///
/// Keeping handler execution injectable via `dispatch_with_handlers` allows tests to
/// validate dispatch logic without touching network transport.
pub fn run_cli(args: CliArgs) -> Result<()> {
    dispatch_with_handlers(
        args,
        run_dashboard_cli,
        run_datasource_cli,
        run_sync_cli,
        run_alert_cli,
        run_access_cli,
    )
}

#[cfg(test)]
#[path = "cli_rust_tests.rs"]
mod cli_rust_tests;
