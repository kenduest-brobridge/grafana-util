use clap::{Args, Subcommand};
use std::path::PathBuf;

use super::access_cli_shared::{
    ACCESS_TEAM_ADD_HELP_TEXT, ACCESS_TEAM_BROWSE_HELP_TEXT, ACCESS_TEAM_DELETE_HELP_TEXT,
    ACCESS_TEAM_DIFF_HELP_TEXT, ACCESS_TEAM_EXPORT_HELP_TEXT, ACCESS_TEAM_IMPORT_HELP_TEXT,
    ACCESS_TEAM_LIST_HELP_TEXT, ACCESS_TEAM_MODIFY_HELP_TEXT,
};
use super::{
    AccessArtifactRunMode, CommonCliArgs, DryRunOutputFormat, ListOutputFormat,
    DEFAULT_ACCESS_TEAM_EXPORT_DIR, DEFAULT_PAGE_SIZE,
};
use super::super::pending_delete::TeamDeleteArgs;

fn parse_team_list_output_column(value: &str) -> std::result::Result<String, String> {
    match value {
        "all" => Ok("all".to_string()),
        "id" => Ok("id".to_string()),
        "name" => Ok("name".to_string()),
        "email" => Ok("email".to_string()),
        "member_count" | "memberCount" => Ok("member_count".to_string()),
        "members" => Ok("members".to_string()),
        _ => Err(format!(
            "Unsupported --output-columns value '{value}'. Supported values: all, id, name, email, member_count, members."
        )),
    }
}

/// Struct definition for TeamListArgs.
#[derive(Debug, Clone, Args)]
pub struct TeamListArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(
        long,
        help = "List teams from a local export bundle directory instead of live Grafana."
    )]
    pub input_dir: Option<PathBuf>,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "input_dir",
        help = "List teams from the artifact workspace instead of live Grafana."
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
    #[arg(long, help = "Filter teams by a free-text search.")]
    pub query: Option<String>,
    #[arg(long, help = "Filter teams by exact team name.")]
    pub name: Option<String>,
    #[arg(
        long,
        default_value_t = false,
        help = "Include team members and admins in the rendered output."
    )]
    pub with_members: bool,
    #[arg(
        long,
        value_delimiter = ',',
        value_parser = parse_team_list_output_column,
        help = "For text, table, or csv output, render only these comma-separated columns. Use all to expand every supported column. Supported values: all, id, name, email, member_count, members. JSON-style aliases like memberCount are also accepted."
    )]
    pub output_columns: Vec<String>,
    #[arg(
        long,
        default_value_t = false,
        help = "Print the supported --output-columns values and exit."
    )]
    pub list_columns: bool,
    #[arg(
        long,
        default_value_t = 1,
        help = "Result page number for paginated Grafana list APIs."
    )]
    pub page: usize,
    #[arg(long, default_value_t = DEFAULT_PAGE_SIZE, help = "Number of teams to request per page.")]
    pub per_page: usize,
    #[arg(long, default_value_t = false, conflicts_with_all = ["csv", "json", "yaml"], help = "Render team summaries as a table.")]
    pub table: bool,
    #[arg(long, default_value_t = false, conflicts_with_all = ["table", "json", "yaml"], help = "Render team summaries as CSV.")]
    pub csv: bool,
    #[arg(long, default_value_t = false, conflicts_with_all = ["table", "csv", "yaml"], help = "Render team summaries as JSON.")]
    pub json: bool,
    #[arg(long, default_value_t = false, conflicts_with_all = ["table", "csv", "json"], help = "Render team summaries as YAML.")]
    pub yaml: bool,
    #[arg(
        long,
        value_enum,
        conflicts_with_all = ["table", "csv", "json", "yaml"],
        help = "Alternative single-flag output selector. Use text, table, csv, json, or yaml."
    )]
    pub output_format: Option<ListOutputFormat>,
}

/// Arguments for interactive team browsing.
#[derive(Debug, Clone, Args)]
pub struct TeamBrowseArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(
        long,
        help = "Browse teams from a local export bundle directory instead of live Grafana."
    )]
    pub input_dir: Option<PathBuf>,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "input_dir",
        help = "Browse teams from the artifact workspace instead of live Grafana."
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
    #[arg(long, help = "Filter teams by a free-text search.")]
    pub query: Option<String>,
    #[arg(long, help = "Filter teams by exact team name.")]
    pub name: Option<String>,
    #[arg(
        long,
        default_value_t = false,
        help = "Include team members and admins in the browse detail view."
    )]
    pub with_members: bool,
    #[arg(
        long,
        default_value_t = 1,
        help = "Result page number for paginated Grafana list APIs."
    )]
    pub page: usize,
    #[arg(long, default_value_t = DEFAULT_PAGE_SIZE, help = "Number of teams to request per page.")]
    pub per_page: usize,
}

/// Create one Grafana team with optional contact and membership data.
#[derive(Debug, Clone, Args)]
pub struct TeamAddArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(long, help = "Name for the new Grafana team.")]
    pub name: String,
    #[arg(long, help = "Optional contact email for the new Grafana team.")]
    pub email: Option<String>,
    #[arg(
        long = "member",
        help = "Add one or more members by user id, exact login, or exact email as part of team creation."
    )]
    pub members: Vec<String>,
    #[arg(
        long = "admin",
        help = "Add one or more team admins by user id, exact login, or exact email as part of team creation."
    )]
    pub admins: Vec<String>,
    #[arg(
        long,
        default_value_t = false,
        help = "Render the create response as JSON."
    )]
    pub json: bool,
}

/// Struct definition for TeamExportArgs.
#[derive(Debug, Clone, Args)]
pub struct TeamExportArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(
        long = "output-dir",
        default_value = DEFAULT_ACCESS_TEAM_EXPORT_DIR,
        help = "Directory to write teams.json and export-metadata.json."
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
        help = "Replace existing export files in the target directory instead of failing."
    )]
    pub overwrite: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Preview export paths without writing files."
    )]
    pub dry_run: bool,
    #[arg(
        long,
        default_value_t = true,
        help = "Include team members and admins in exported team records."
    )]
    pub with_members: bool,
}

/// Struct definition for TeamImportArgs.
#[derive(Debug, Clone, Args)]
pub struct TeamImportArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(
        long = "input-dir",
        default_value = "",
        hide_default_value = true,
        help = "Import directory that contains teams.json and export-metadata.json."
    )]
    pub input_dir: PathBuf,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "input_dir",
        help = "Import teams from the artifact workspace instead of an explicit input directory."
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
        help = "Update matching existing teams instead of failing on duplicates."
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
        requires = "dry_run",
        help = "For --dry-run only, render a compact table instead of per-record log lines."
    )]
    pub table: bool,
    #[arg(
        long,
        default_value_t = false,
        requires = "dry_run",
        help = "For --dry-run only, render one JSON document with action rows and summary counts."
    )]
    pub json: bool,
    #[arg(
        long,
        value_enum,
        default_value_t = DryRunOutputFormat::Text,
        conflicts_with_all = ["table", "json"],
        help = "Alternative single-flag output selector for --dry-run output. Use text, table, or json."
    )]
    pub output_format: DryRunOutputFormat,
    #[arg(
        long,
        default_value_t = false,
        help = "Acknowledge destructive team-member synchronization operations."
    )]
    pub yes: bool,
}

/// Struct definition for TeamDiffArgs.
#[derive(Debug, Clone, Args)]
pub struct TeamDiffArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(
        long,
        default_value = "access-teams",
        help = "Diff directory that contains teams.json and export-metadata.json."
    )]
    pub diff_dir: PathBuf,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "diff_dir",
        help = "Diff teams from the artifact workspace instead of an explicit diff directory."
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

/// Struct definition for TeamModifyArgs.
#[derive(Debug, Clone, Args)]
pub struct TeamModifyArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(
        long,
        conflicts_with = "name",
        help = "Target one team by numeric Grafana team id."
    )]
    pub team_id: Option<String>,
    #[arg(
        long,
        conflicts_with = "team_id",
        help = "Target one team by exact team name."
    )]
    pub name: Option<String>,
    #[arg(
        long = "add-member",
        help = "Add one or more members by user id, exact login, or exact email."
    )]
    pub add_member: Vec<String>,
    #[arg(
        long = "remove-member",
        help = "Remove one or more members by user id, exact login, or exact email."
    )]
    pub remove_member: Vec<String>,
    #[arg(
        long = "add-admin",
        help = "Promote one or more members to team admin by user id, exact login, or exact email."
    )]
    pub add_admin: Vec<String>,
    #[arg(
        long = "remove-admin",
        help = "Remove team-admin status from one or more members by user id, exact login, or exact email."
    )]
    pub remove_admin: Vec<String>,
    #[arg(
        long,
        default_value_t = false,
        help = "Render the modify response as JSON."
    )]
    pub json: bool,
}

/// Team inventory and membership workflows.
#[derive(Debug, Clone, Subcommand)]
pub enum TeamCommand {
    #[command(
        about = "List live or local Grafana teams.",
        after_help = ACCESS_TEAM_LIST_HELP_TEXT
    )]
    List(TeamListArgs),
    #[command(
        about = "Browse live or local Grafana teams interactively.",
        after_help = ACCESS_TEAM_BROWSE_HELP_TEXT
    )]
    Browse(TeamBrowseArgs),
    #[command(
        about = "Create one Grafana team with optional contact and membership data.",
        after_help = ACCESS_TEAM_ADD_HELP_TEXT
    )]
    Add(TeamAddArgs),
    #[command(
        about = "Modify one Grafana team and its membership.",
        after_help = ACCESS_TEAM_MODIFY_HELP_TEXT
    )]
    Modify(TeamModifyArgs),
    #[command(
        about = "Export Grafana teams into a local reviewable bundle.",
        after_help = ACCESS_TEAM_EXPORT_HELP_TEXT
    )]
    Export(TeamExportArgs),
    #[command(
        about = "Import Grafana teams from a local bundle.",
        after_help = ACCESS_TEAM_IMPORT_HELP_TEXT
    )]
    Import(TeamImportArgs),
    #[command(
        about = "Compare a local team bundle against live Grafana teams.",
        after_help = ACCESS_TEAM_DIFF_HELP_TEXT
    )]
    Diff(TeamDiffArgs),
    #[command(
        about = "Delete one Grafana team.",
        after_help = ACCESS_TEAM_DELETE_HELP_TEXT
    )]
    Delete(TeamDeleteArgs),
}
