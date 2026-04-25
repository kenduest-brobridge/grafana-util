//! CLI argument and help surface for the status command.

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

use crate::common::CliColorChoice;

pub(crate) const PROJECT_STATUS_HELP_TEXT: &str = "Examples:\n\n  Render staged project status as a summary table from raw dashboard artifacts:\n    grafana-util status staged --dashboard-export-dir ./dashboards/raw --desired-file ./desired.json --output-format table\n\n  Render staged project status from dashboard provisioning artifacts:\n    grafana-util status staged --dashboard-provisioning-dir ./dashboards/provisioning --output-format csv\n\n  Render staged project status from datasource provisioning YAML:\n    grafana-util status staged --datasource-provisioning-file ./datasources/provisioning/datasources.yaml --output-format yaml\n\n  Render live project status with staged workspace context:\n    grafana-util status live --url http://localhost:3000 --token \"$GRAFANA_API_TOKEN\" --sync-summary-file ./sync-summary.json --package-test-file ./workspace-package-test.json --output-format json\n\nSchema guide:\n  grafana-util status --help-schema\n  grafana-util status staged --help-schema\n  grafana-util status live --help-schema";
pub(crate) const PROJECT_STATUS_STAGED_HELP_TEXT: &str = "Examples:\n\n  Render staged project status as a summary table from raw dashboard artifacts:\n    grafana-util status staged --dashboard-export-dir ./dashboards/raw --desired-file ./desired.json --output-format table\n\n  Render staged project status from dashboard provisioning artifacts in the interactive workbench:\n    grafana-util status staged --dashboard-provisioning-dir ./dashboards/provisioning --alert-export-dir ./alerts --output-format interactive\n\n  Render staged project status from datasource provisioning YAML:\n    grafana-util status staged --datasource-provisioning-file ./datasources/provisioning/datasources.yaml --output-format yaml\n\nSchema guide:\n  grafana-util status staged --help-schema";
pub(crate) const PROJECT_STATUS_LIVE_HELP_TEXT: &str = "Examples:\n\n  Render live project status as YAML:\n    grafana-util status live --url http://localhost:3000 --token \"$GRAFANA_API_TOKEN\" --output-format yaml\n\n  Render live status across visible orgs while layering staged sync context:\n    grafana-util status live --url http://localhost:3000 --basic-user admin --basic-password admin --all-orgs --sync-summary-file ./sync-summary.json --output-format interactive\n\nSchema guide:\n  grafana-util status live --help-schema";

#[cfg(feature = "tui")]
const PROJECT_STATUS_OUTPUT_HELP: &str =
    "Render project status as table, csv, text, json, yaml, or interactive output.";

#[cfg(not(feature = "tui"))]
const PROJECT_STATUS_OUTPUT_HELP: &str =
    "Render project status as table, csv, text, json, or yaml output.";

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum ProjectStatusOutputFormat {
    Table,
    Csv,
    Text,
    Json,
    Yaml,
    #[cfg(feature = "tui")]
    Interactive,
}

#[derive(Debug, Clone, Args)]
pub struct ProjectStatusStagedArgs {
    #[arg(
        long,
        conflicts_with = "dashboard_provisioning_dir",
        help = "Dashboard export directory to summarize from staged artifacts."
    )]
    pub dashboard_export_dir: Option<PathBuf>,
    #[arg(
        long,
        conflicts_with = "dashboard_export_dir",
        help = "Dashboard provisioning directory to summarize from staged artifacts."
    )]
    pub dashboard_provisioning_dir: Option<PathBuf>,
    #[arg(
        long,
        conflicts_with = "datasource_provisioning_file",
        help = "Datasource export directory to summarize from staged artifacts."
    )]
    pub datasource_export_dir: Option<PathBuf>,
    #[arg(
        long,
        conflicts_with = "datasource_export_dir",
        help = "Datasource provisioning YAML file to summarize instead of the stable inventory contract.",
        help_heading = "Input Options"
    )]
    pub datasource_provisioning_file: Option<PathBuf>,
    #[arg(
        long,
        help = "Access user export directory to summarize from staged artifacts."
    )]
    pub access_user_export_dir: Option<PathBuf>,
    #[arg(
        long,
        help = "Access team export directory to summarize from staged artifacts."
    )]
    pub access_team_export_dir: Option<PathBuf>,
    #[arg(
        long,
        help = "Access org export directory to summarize from staged artifacts."
    )]
    pub access_org_export_dir: Option<PathBuf>,
    #[arg(
        long,
        help = "Access service-account export directory to summarize from staged artifacts."
    )]
    pub access_service_account_export_dir: Option<PathBuf>,
    #[arg(long, help = "Desired staged file to summarize from staged artifacts.")]
    pub desired_file: Option<PathBuf>,
    #[arg(
        long,
        help = "Source bundle JSON file used by staged bundle/promotion checks."
    )]
    pub source_bundle: Option<PathBuf>,
    #[arg(
        long,
        help = "Target inventory JSON file used by staged bundle/promotion checks."
    )]
    pub target_inventory: Option<PathBuf>,
    #[arg(
        long,
        help = "Alert export directory to summarize from staged artifacts."
    )]
    pub alert_export_dir: Option<PathBuf>,
    #[arg(
        long,
        help = "Optional availability JSON reused by staged preflight builders."
    )]
    pub availability_file: Option<PathBuf>,
    #[arg(
        long,
        help = "Optional mapping JSON reused by staged promotion builders."
    )]
    pub mapping_file: Option<PathBuf>,
    #[arg(
        long = "output-format",
        value_enum,
        default_value_t = ProjectStatusOutputFormat::Text,
        help = PROJECT_STATUS_OUTPUT_HELP
    )]
    pub output_format: ProjectStatusOutputFormat,
}

#[derive(Debug, Clone, Args)]
pub struct ProjectStatusLiveArgs {
    #[arg(
        long,
        help = "Load connection defaults from the selected repo-local profile in grafana-util.yaml."
    )]
    pub profile: Option<String>,
    #[arg(
        long,
        default_value = "",
        hide_default_value = true,
        help = "Grafana base URL. Required unless supplied by --profile or GRAFANA_URL."
    )]
    pub url: String,
    #[arg(
        long = "token",
        visible_alias = "api-token",
        help = "Grafana API token. Preferred flag: --token. Falls back to GRAFANA_API_TOKEN."
    )]
    pub api_token: Option<String>,
    #[arg(
        long = "basic-user",
        help = "Grafana Basic auth username. Preferred flag: --basic-user. Falls back to GRAFANA_USERNAME."
    )]
    pub username: Option<String>,
    #[arg(
        long = "basic-password",
        help = "Grafana Basic auth password. Preferred flag: --basic-password. Falls back to GRAFANA_PASSWORD."
    )]
    pub password: Option<String>,
    #[arg(
        long,
        default_value_t = false,
        help = "Prompt for the Grafana Basic auth password."
    )]
    pub prompt_password: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Prompt for the Grafana API token."
    )]
    pub prompt_token: bool,
    #[arg(long, default_value_t = 30, help = "HTTP timeout in seconds.")]
    pub timeout: u64,
    #[arg(
        long,
        default_value_t = false,
        help = "Enable TLS certificate verification. Verification is disabled by default."
    )]
    pub verify_ssl: bool,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with_all = ["verify_ssl", "ca_cert"],
        help = "Disable TLS certificate verification explicitly."
    )]
    pub insecure: bool,
    #[arg(
        long = "ca-cert",
        value_name = "PATH",
        help = "PEM bundle file to trust for Grafana TLS verification."
    )]
    pub ca_cert: Option<PathBuf>,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "org_id",
        help = "Query live status across all Grafana organizations where the domain supports it."
    )]
    pub all_orgs: bool,
    #[arg(
        long,
        help = "Grafana organization id to scope live reads where supported."
    )]
    pub org_id: Option<i64>,
    #[arg(
        long,
        help = "Optional staged sync-summary JSON used to deepen live status."
    )]
    pub sync_summary_file: Option<PathBuf>,
    #[arg(
        long = "package-test-file",
        alias = "bundle-preflight-file",
        help = "Optional workspace package-test JSON used to deepen live status."
    )]
    pub bundle_preflight_file: Option<PathBuf>,
    #[arg(
        long,
        help = "Optional staged promotion summary JSON used to deepen live promotion status."
    )]
    pub promotion_summary_file: Option<PathBuf>,
    #[arg(
        long,
        help = "Optional staged promotion mapping JSON used to deepen live promotion status."
    )]
    pub mapping_file: Option<PathBuf>,
    #[arg(
        long,
        help = "Optional staged availability JSON used to deepen live promotion status."
    )]
    pub availability_file: Option<PathBuf>,
    #[arg(
        long = "output-format",
        value_enum,
        default_value_t = ProjectStatusOutputFormat::Text,
        help = PROJECT_STATUS_OUTPUT_HELP
    )]
    pub output_format: ProjectStatusOutputFormat,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ProjectStatusSubcommand {
    #[command(
        about = "Render project status from staged artifacts. Use exported project inputs.",
        after_help = PROJECT_STATUS_STAGED_HELP_TEXT
    )]
    Staged(ProjectStatusStagedArgs),
    #[command(
        about = "Render project status from live Grafana read surfaces. Use current Grafana state plus optional staged context files.",
        after_help = PROJECT_STATUS_LIVE_HELP_TEXT
    )]
    Live(ProjectStatusLiveArgs),
}

#[derive(Debug, Clone, Parser)]
#[command(
    name = "grafana-util status",
    about = "Render project-wide staged or live status through a shared status contract. Staged subcommands read exports; live subcommands query Grafana.",
    after_help = PROJECT_STATUS_HELP_TEXT
)]
pub struct ProjectStatusCliArgs {
    #[arg(
        long,
        value_enum,
        default_value_t = CliColorChoice::Auto,
        help = "Colorize JSON output. Use auto, always, or never."
    )]
    pub color: CliColorChoice,
    #[command(subcommand)]
    pub command: ProjectStatusSubcommand,
}
