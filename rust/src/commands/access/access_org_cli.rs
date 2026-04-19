use clap::{Args, Subcommand};
use std::path::PathBuf;

use super::access_cli_shared::{
    ACCESS_ORG_ADD_HELP_TEXT, ACCESS_ORG_DELETE_HELP_TEXT, ACCESS_ORG_DIFF_HELP_TEXT,
    ACCESS_ORG_EXPORT_HELP_TEXT, ACCESS_ORG_IMPORT_HELP_TEXT, ACCESS_ORG_LIST_HELP_TEXT,
    ACCESS_ORG_MODIFY_HELP_TEXT,
};
use super::{
    AccessArtifactRunMode, CommonCliArgsNoOrgId, ListOutputFormat, DEFAULT_ACCESS_ORG_EXPORT_DIR,
};

/// Struct definition for OrgListArgs.
#[derive(Debug, Clone, Args)]
pub struct OrgListArgs {
    #[command(flatten)]
    pub common: CommonCliArgsNoOrgId,
    #[arg(
        long,
        help = "List organizations from a local export bundle directory instead of live Grafana."
    )]
    pub input_dir: Option<PathBuf>,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "input_dir",
        help = "List organizations from the artifact workspace instead of live Grafana."
    )]
    pub local: bool,
    #[arg(
        long,
        value_enum,
        requires = "local",
        help = "With --local, select the artifact run to read from. Defaults to latest."
    )]
    pub run: Option<AccessArtifactRunMode>,
    #[arg(
        long = "run-id",
        requires = "local",
        help = "With --local, read from this explicit artifact run id."
    )]
    pub run_id: Option<String>,
    #[arg(long = "org-id", help = "Filter to one exact organization id.")]
    pub org_id: Option<i64>,
    #[arg(long, help = "Filter organizations by exact name.")]
    pub name: Option<String>,
    #[arg(long, help = "Filter organizations by a free-text search.")]
    pub query: Option<String>,
    #[arg(
        long,
        default_value_t = false,
        help = "Include org users and org roles in the rendered output."
    )]
    pub with_users: bool,
    #[arg(long, default_value_t = false, conflicts_with_all = ["csv", "json", "yaml"], help = "Render org summaries as a table.")]
    pub table: bool,
    #[arg(long, default_value_t = false, conflicts_with_all = ["table", "json", "yaml"], help = "Render org summaries as CSV.")]
    pub csv: bool,
    #[arg(long, default_value_t = false, conflicts_with_all = ["table", "csv", "yaml"], help = "Render org summaries as JSON.")]
    pub json: bool,
    #[arg(long, default_value_t = false, conflicts_with_all = ["table", "csv", "json"], help = "Render org summaries as YAML.")]
    pub yaml: bool,
    #[arg(
        long,
        value_enum,
        conflicts_with_all = ["table", "csv", "json", "yaml"],
        help = "Alternative single-flag output selector. Use text, table, csv, json, or yaml."
    )]
    pub output_format: Option<ListOutputFormat>,
}

/// Struct definition for OrgAddArgs.
#[derive(Debug, Clone, Args)]
pub struct OrgAddArgs {
    #[command(flatten)]
    pub common: CommonCliArgsNoOrgId,
    #[arg(long, help = "Name for the new Grafana organization.")]
    pub name: String,
    #[arg(
        long,
        default_value_t = false,
        help = "Render the create response as JSON."
    )]
    pub json: bool,
}

/// Struct definition for OrgModifyArgs.
#[derive(Debug, Clone, Args)]
pub struct OrgModifyArgs {
    #[command(flatten)]
    pub common: CommonCliArgsNoOrgId,
    #[arg(
        long = "org-id",
        conflicts_with = "name",
        help = "Target one organization by numeric Grafana org id."
    )]
    pub org_id: Option<i64>,
    #[arg(
        long,
        conflicts_with = "org_id",
        help = "Target one organization by exact name."
    )]
    pub name: Option<String>,
    #[arg(long, help = "Replace the organization name with this new value.")]
    pub set_name: String,
    #[arg(
        long,
        default_value_t = false,
        help = "Render the modify response as JSON."
    )]
    pub json: bool,
}

/// Struct definition for OrgDeleteArgs.
#[derive(Debug, Clone, Args)]
pub struct OrgDeleteArgs {
    #[command(flatten)]
    pub common: CommonCliArgsNoOrgId,
    #[arg(
        long = "org-id",
        conflicts_with = "name",
        help = "Delete one organization by numeric Grafana org id."
    )]
    pub org_id: Option<i64>,
    #[arg(
        long,
        conflicts_with = "org_id",
        help = "Delete one organization by exact name."
    )]
    pub name: Option<String>,
    #[arg(
        long,
        default_value_t = false,
        help = "Prompt for the target organization, show a terminal confirmation, and then delete."
    )]
    pub prompt: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Skip the terminal confirmation prompt in non-prompt mode."
    )]
    pub yes: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Render the delete response as JSON."
    )]
    pub json: bool,
}

/// Struct definition for OrgExportArgs.
#[derive(Debug, Clone, Args)]
pub struct OrgExportArgs {
    #[command(flatten)]
    pub common: CommonCliArgsNoOrgId,
    #[arg(long = "org-id", help = "Filter export to one exact organization id.")]
    pub org_id: Option<i64>,
    #[arg(
        long = "output-dir",
        default_value = DEFAULT_ACCESS_ORG_EXPORT_DIR,
        help = "Directory to write orgs.json and export-metadata.json."
    )]
    pub output_dir: PathBuf,
    #[arg(
        long,
        value_enum,
        help = "Write into an artifact workspace run instead of the default output directory."
    )]
    pub run: Option<AccessArtifactRunMode>,
    #[arg(
        long = "run-id",
        help = "Write into this explicit artifact run id instead of --run."
    )]
    pub run_id: Option<String>,
    #[arg(
        long,
        default_value_t = false,
        help = "Overwrite existing export files instead of failing."
    )]
    pub overwrite: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Preview export paths without writing files."
    )]
    pub dry_run: bool,
    #[arg(long, help = "Filter export to one exact organization name.")]
    pub name: Option<String>,
    #[arg(
        long,
        default_value_t = false,
        help = "Include org users and org roles in the export bundle."
    )]
    pub with_users: bool,
}

/// Struct definition for OrgImportArgs.
#[derive(Debug, Clone, Args)]
pub struct OrgImportArgs {
    #[command(flatten)]
    pub common: CommonCliArgsNoOrgId,
    #[arg(
        long = "input-dir",
        default_value = "",
        hide_default_value = true,
        help = "Import directory that contains orgs.json and export-metadata.json."
    )]
    pub input_dir: PathBuf,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "input_dir",
        help = "Import orgs from the artifact workspace instead of an explicit input directory."
    )]
    pub local: bool,
    #[arg(
        long,
        value_enum,
        conflicts_with_all = ["input_dir", "run_id"],
        help = "Select the artifact run to import from."
    )]
    pub run: Option<AccessArtifactRunMode>,
    #[arg(
        long = "run-id",
        conflicts_with_all = ["input_dir", "run"],
        help = "Import from this explicit artifact run id."
    )]
    pub run_id: Option<String>,
    #[arg(
        long,
        default_value_t = false,
        help = "Update matching existing orgs or create missing orgs instead of skipping them."
    )]
    pub replace_existing: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Preview import changes without writing to Grafana."
    )]
    pub dry_run: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Acknowledge destructive import operations when required."
    )]
    pub yes: bool,
}

/// Struct definition for OrgDiffArgs.
#[derive(Debug, Clone, Args)]
pub struct OrgDiffArgs {
    #[command(flatten)]
    pub common: CommonCliArgsNoOrgId,
    #[arg(
        long,
        default_value = DEFAULT_ACCESS_ORG_EXPORT_DIR,
        help = "Diff directory that contains orgs.json and export-metadata.json."
    )]
    pub diff_dir: PathBuf,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "diff_dir",
        help = "Diff orgs from the artifact workspace instead of an explicit diff directory."
    )]
    pub local: bool,
    #[arg(
        long,
        value_enum,
        conflicts_with_all = ["diff_dir", "run_id"],
        help = "Select the artifact run to diff from."
    )]
    pub run: Option<AccessArtifactRunMode>,
    #[arg(
        long = "run-id",
        conflicts_with_all = ["diff_dir", "run"],
        help = "Diff from this explicit artifact run id."
    )]
    pub run_id: Option<String>,
}

/// Organization inventory and lifecycle workflows.
#[derive(Debug, Clone, Subcommand)]
pub enum OrgCommand {
    #[command(
        about = "List live or local Grafana organizations.",
        after_help = ACCESS_ORG_LIST_HELP_TEXT
    )]
    List(OrgListArgs),
    #[command(
        about = "Create one Grafana organization.",
        after_help = ACCESS_ORG_ADD_HELP_TEXT
    )]
    Add(OrgAddArgs),
    #[command(
        about = "Rename one Grafana organization.",
        after_help = ACCESS_ORG_MODIFY_HELP_TEXT
    )]
    Modify(OrgModifyArgs),
    #[command(
        about = "Export Grafana organizations into a local reviewable bundle.",
        after_help = ACCESS_ORG_EXPORT_HELP_TEXT
    )]
    Export(OrgExportArgs),
    #[command(
        about = "Import Grafana organizations from a local bundle.",
        after_help = ACCESS_ORG_IMPORT_HELP_TEXT
    )]
    Import(OrgImportArgs),
    #[command(
        about = "Compare a local organization bundle against live Grafana organizations.",
        after_help = ACCESS_ORG_DIFF_HELP_TEXT
    )]
    Diff(OrgDiffArgs),
    #[command(
        about = "Delete one Grafana organization.",
        after_help = ACCESS_ORG_DELETE_HELP_TEXT
    )]
    Delete(OrgDeleteArgs),
}
