use clap::{ArgAction, Args};
use std::path::PathBuf;

use crate::common::DiffOutputFormat;

use super::super::alert_cli_commands::{
    AlertCommandOutputFormat, AlertListOutputFormat, AlertResourceKind,
};
use super::AlertCommonArgs;

use super::super::super::DEFAULT_OUTPUT_DIR;

/// Arguments for exporting alerting resources from Grafana.
#[derive(Debug, Clone, Args)]
pub struct AlertExportArgs {
    #[command(flatten)]
    pub common: AlertCommonArgs,
    #[arg(
        long,
        default_value = DEFAULT_OUTPUT_DIR,
        help = "Directory to write exported alerting resources into. Export writes files under raw/."
    )]
    pub output_dir: PathBuf,
    #[arg(
        long,
        default_value_t = false,
        help = "Write rule, contact-point, mute-timing, and template files directly into their resource directories instead of nested subdirectories."
    )]
    pub flat: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Overwrite existing exported files."
    )]
    pub overwrite: bool,
}

/// Arguments for importing alerting resources from a local export directory.
#[derive(Debug, Clone, Args)]
pub struct AlertImportArgs {
    #[command(flatten)]
    pub common: AlertCommonArgs,
    #[arg(
        long = "input-dir",
        help = "Import alerting resource JSON from this directory instead of exporting. Point this to the raw/ export directory explicitly."
    )]
    pub input_dir: PathBuf,
    #[arg(
        long,
        default_value_t = false,
        help = "Update existing resources with the same identity instead of failing on import."
    )]
    pub replace_existing: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Show whether each import file would create or update resources without changing Grafana."
    )]
    pub dry_run: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Render dry-run import output as structured JSON. Only supported with --dry-run."
    )]
    pub json: bool,
    #[arg(
        long,
        help = "JSON file that maps source dashboard UIDs to target dashboard UIDs for linked alert-rule repair during import."
    )]
    pub dashboard_uid_map: Option<PathBuf>,
    #[arg(
        long,
        help = "JSON file that maps source dashboard UID and source panel ID to a target panel ID for linked alert-rule repair during import."
    )]
    pub panel_id_map: Option<PathBuf>,
}

/// Arguments for diffing local alert exports against live Grafana state.
#[derive(Debug, Clone, Args)]
pub struct AlertDiffArgs {
    #[command(flatten)]
    pub common: AlertCommonArgs,
    #[arg(
        long,
        help = "Compare alerting resource JSON from this directory against Grafana. Point this to the raw/ export directory explicitly."
    )]
    pub diff_dir: PathBuf,
    #[arg(
        long,
        default_value_t = false,
        help = "Deprecated compatibility flag. Equivalent to --output-format json."
    )]
    pub json: bool,
    #[arg(
        long = "output-format",
        value_enum,
        default_value_t = DiffOutputFormat::Text,
        help = "Render diff output as text or json."
    )]
    pub output_format: DiffOutputFormat,
    #[arg(
        long,
        help = "JSON file that maps source dashboard UIDs to target dashboard UIDs for linked alert-rule repair during import."
    )]
    pub dashboard_uid_map: Option<PathBuf>,
    #[arg(
        long,
        help = "JSON file that maps source dashboard UID and source panel ID to a target panel ID for linked alert-rule repair during import."
    )]
    pub panel_id_map: Option<PathBuf>,
}

/// Struct definition for AlertListArgs.
#[derive(Debug, Clone, Args)]
pub struct AlertListArgs {
    #[command(flatten)]
    pub common: AlertCommonArgs,
    #[arg(
        long,
        conflicts_with = "all_orgs",
        help = "List alerting resources from this Grafana org ID. This requires Basic auth."
    )]
    pub org_id: Option<i64>,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "org_id",
        help = "Enumerate all visible Grafana orgs and aggregate alerting inventory across them. This requires Basic auth."
    )]
    pub all_orgs: bool,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with_all = ["table", "csv", "json", "yaml", "output_format"],
        help = "Render list output as plain text.",
        help_heading = "Output Options"
    )]
    pub text: bool,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with_all = ["text", "csv", "json", "yaml", "output_format"],
        help = "Render list output as a table. This is the default.",
        help_heading = "Output Options"
    )]
    pub table: bool,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with_all = ["text", "table", "json", "yaml", "output_format"],
        help = "Render list output as CSV.",
        help_heading = "Output Options"
    )]
    pub csv: bool,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with_all = ["text", "table", "csv", "yaml", "output_format"],
        help = "Render list output as JSON.",
        help_heading = "Output Options"
    )]
    pub json: bool,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with_all = ["text", "table", "csv", "json", "output_format"],
        help = "Render list output as YAML.",
        help_heading = "Output Options"
    )]
    pub yaml: bool,
    #[arg(
        long,
        value_enum,
        conflicts_with_all = ["text", "table", "csv", "json", "yaml"],
        help = "Alternative single-flag output selector. Use text, table, csv, json, or yaml.",
        help_heading = "Output Options"
    )]
    pub output_format: Option<AlertListOutputFormat>,
    #[arg(
        long,
        default_value_t = false,
        help = "Omit the table header row.",
        help_heading = "Output Options"
    )]
    pub no_header: bool,
}

/// Arguments for building a staged alert apply plan.
#[derive(Debug, Clone, Args)]
pub struct AlertPlanArgs {
    #[command(flatten)]
    pub common: AlertCommonArgs,
    #[arg(
        long,
        help = "Directory containing the desired alert resource definitions to plan from."
    )]
    pub desired_dir: PathBuf,
    #[arg(
        long,
        default_value_t = false,
        help = "Mark live-only alert resources as delete candidates in the staged plan."
    )]
    pub prune: bool,
    #[arg(
        long,
        help = "JSON file that maps source dashboard UIDs to target dashboard UIDs for linked alert-rule repair during planning."
    )]
    pub dashboard_uid_map: Option<PathBuf>,
    #[arg(
        long,
        help = "JSON file that maps source dashboard UID and source panel ID to a target panel ID for linked alert-rule repair during planning."
    )]
    pub panel_id_map: Option<PathBuf>,
    #[arg(
        long = "output-format",
        value_enum,
        default_value_t = AlertCommandOutputFormat::Text,
        help = "Render plan output as text or json."
    )]
    pub output_format: AlertCommandOutputFormat,
}

/// Arguments for applying a reviewed alert plan.
#[derive(Debug, Clone, Args)]
pub struct AlertApplyArgs {
    #[command(flatten)]
    pub common: AlertCommonArgs,
    #[arg(long, help = "JSON file containing the reviewed alert plan document.")]
    pub plan_file: PathBuf,
    #[arg(
        long,
        action = ArgAction::SetTrue,
        required = true,
        help = "Explicit acknowledgement required before alert apply execution is allowed."
    )]
    pub approve: bool,
    #[arg(
        long = "output-format",
        value_enum,
        default_value_t = AlertCommandOutputFormat::Text,
        help = "Render apply output as text or json."
    )]
    pub output_format: AlertCommandOutputFormat,
}

/// Arguments for deleting one managed alert resource.
#[derive(Debug, Clone, Args)]
pub struct AlertDeleteArgs {
    #[command(flatten)]
    pub common: AlertCommonArgs,
    #[arg(long, value_enum, help = "Alert resource kind to delete.")]
    pub kind: AlertResourceKind,
    #[arg(
        long,
        help = "Explicit resource identity for the selected delete kind."
    )]
    pub identity: String,
    #[arg(
        long,
        default_value_t = false,
        help = "Allow notification policy tree reset when deleting the policy-tree resource kind."
    )]
    pub allow_policy_reset: bool,
    #[arg(
        long = "output-format",
        value_enum,
        default_value_t = AlertCommandOutputFormat::Text,
        help = "Render delete preview or execution output as text or json."
    )]
    pub output_format: AlertCommandOutputFormat,
}
