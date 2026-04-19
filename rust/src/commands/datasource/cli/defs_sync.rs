use clap::Args;
use std::path::PathBuf;

use crate::common::DiffOutputFormat;
use crate::dashboard::CommonCliArgs;

use super::{
    parse_datasource_import_output_column, parse_datasource_plan_output_column, ArtifactRunMode,
    DatasourceImportInputFormat, DatasourcePlanOutputFormat, DryRunOutputFormat,
};

#[derive(Debug, Clone, Args)]
pub struct DatasourceExportArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(
        long = "output-dir",
        help = "Directory to write exported datasource inventory into. Export writes datasources.json plus index/manifest files at that root, and writes provisioning/datasources.yaml by default."
    )]
    pub output_dir: Option<PathBuf>,
    #[arg(
        long,
        value_enum,
        conflicts_with = "output_dir",
        help = "When writing to the artifact workspace (no --output-dir), select which run id to write into: timestamp (new run id) or latest (existing recorded run). Defaults to timestamp.",
        help_heading = "Artifact Options"
    )]
    pub run: Option<ArtifactRunMode>,
    #[arg(
        long = "run-id",
        conflicts_with = "output_dir",
        help = "When writing to the artifact workspace (no --output-dir), write into this explicit run id instead of --run.",
        help_heading = "Artifact Options"
    )]
    pub run_id: Option<String>,
    #[arg(
        long,
        conflicts_with = "all_orgs",
        help = "Export datasource inventory from this explicit Grafana org ID instead of the current org. Requires Basic auth."
    )]
    pub org_id: Option<i64>,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "org_id",
        help = "Enumerate all visible Grafana orgs and export one datasource inventory bundle per org under the export root. Requires Basic auth."
    )]
    pub all_orgs: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Replace existing export files in the target directory instead of failing."
    )]
    pub overwrite: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Skip the file-provisioning provisioning/ export variant. Use this only when Grafana datasource provisioning files are not needed."
    )]
    pub without_datasource_provisioning: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Preview the datasource export files that would be written without changing disk."
    )]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Args)]
pub struct DatasourceImportArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(
        long = "input-dir",
        required_unless_present = "local",
        conflicts_with = "local",
        help = "Import datasource inventory from this path. For inventory input, point this at the datasource export root that contains datasources.json and export-metadata.json. For provisioning input, point this at the export root, provisioning directory, or concrete datasources.yaml file.",
        help_heading = "Input Options"
    )]
    pub input_dir: Option<PathBuf>,
    #[arg(
        long,
        default_value_t = false,
        help = "Import datasource inventory from the artifact workspace datasource lane instead of --input-dir.",
        help_heading = "Input Options"
    )]
    pub local: bool,
    #[arg(
        long,
        value_enum,
        requires = "local",
        help = "With --local and no --input-dir, select which artifact run to read from. Defaults to latest.",
        help_heading = "Input Options"
    )]
    pub run: Option<ArtifactRunMode>,
    #[arg(
        long = "run-id",
        requires = "local",
        help = "With --local and no --input-dir, import from this explicit artifact run id (overrides --run).",
        help_heading = "Input Options"
    )]
    pub run_id: Option<String>,
    #[arg(
        long,
        value_enum,
        default_value_t = DatasourceImportInputFormat::Inventory,
        help = "Select the datasource import input format. Use inventory for the existing datasources.json export contract, or provisioning to load Grafana datasource provisioning YAML from an export root, provisioning directory, or concrete datasources.yaml file.",
        help_heading = "Input Options"
    )]
    pub input_format: DatasourceImportInputFormat,
    #[arg(
        long,
        conflicts_with = "use_export_org",
        help = "Import datasources into this Grafana org ID instead of the current org context. Requires Basic auth."
    )]
    pub org_id: Option<i64>,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "require_matching_export_org",
        help = "Import a combined multi-org datasource export root by routing each org-scoped datasource bundle back into the matching Grafana org. Requires Basic auth."
    )]
    pub use_export_org: bool,
    #[arg(
        long = "only-org-id",
        requires = "use_export_org",
        conflicts_with = "org_id",
        help = "With --use-export-org, import only these exported source org IDs. Repeat the flag to select multiple orgs."
    )]
    pub only_org_id: Vec<i64>,
    #[arg(
        long,
        default_value_t = false,
        requires = "use_export_org",
        help = "With --use-export-org, create a missing destination org when an exported source org ID does not exist in Grafana. The new org is created from the exported org name and then used as the import target."
    )]
    pub create_missing_orgs: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Require the datasource export orgId to match the target Grafana org before dry-run or live import."
    )]
    pub require_matching_export_org: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Update an existing destination datasource when the imported datasource identity already exists. Without this flag, existing matches are blocked."
    )]
    pub replace_existing: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Reconcile only datasources that already exist in Grafana. Missing destination identities are skipped instead of created."
    )]
    pub update_existing_only: bool,
    #[arg(
        long,
        help = "Inline JSON object string mapping datasource secret placeholder names to resolved secret values for live import when the export records include secureJsonDataPlaceholders.",
        help_heading = "Secrets"
    )]
    pub secret_values: Option<String>,
    #[arg(
        long = "secret-values-file",
        help = "Path to a JSON file containing the datasource secret placeholder map for import.",
        help_heading = "Secrets"
    )]
    pub secret_values_file: Option<PathBuf>,
    #[arg(
        long,
        default_value_t = false,
        help = "Preview what datasource import would do without changing Grafana."
    )]
    pub dry_run: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "For --dry-run only, render a compact table instead of per-datasource log lines."
    )]
    pub table: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "For --dry-run only, render one JSON document with mode, datasource actions, and summary counts."
    )]
    pub json: bool,
    #[arg(
        long,
        value_enum,
        conflicts_with_all = ["table", "json"],
        help = "Alternative single-flag output selector for --dry-run output. Use text, table, or json."
    )]
    pub output_format: Option<DryRunOutputFormat>,
    #[arg(
        long,
        default_value_t = false,
        help = "For --dry-run --table only, omit the table header row."
    )]
    pub no_header: bool,
    #[arg(
        long,
        value_delimiter = ',',
        requires = "dry_run",
        value_parser = parse_datasource_import_output_column,
        help = "For --dry-run --table only, render only these comma-separated columns. Supported values: uid, name, type, destination, action, org_id, file, target_uid, target_version, target_read_only, blocked_reason."
    )]
    pub output_columns: Vec<String>,
    #[arg(
        long,
        default_value_t = false,
        requires = "dry_run",
        help = "Print the supported --output-columns values and exit."
    )]
    pub list_columns: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Show concise per-datasource progress in <current>/<total> form while processing files."
    )]
    pub progress: bool,
    #[arg(
        short = 'v',
        long,
        default_value_t = false,
        help = "Show detailed per-item import output. Overrides --progress output."
    )]
    pub verbose: bool,
}

#[derive(Debug, Clone, Args)]
pub struct DatasourceDiffArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(
        long,
        required_unless_present = "local",
        conflicts_with = "local",
        help = "Compare datasource inventory from this directory against live Grafana and print an operator-summary diff report. For provisioning input, point this at the export root, provisioning directory, or concrete datasources.yaml file.",
        help_heading = "Input Options"
    )]
    pub diff_dir: Option<PathBuf>,
    #[arg(
        long,
        default_value_t = false,
        help = "Diff datasource inventory from the artifact workspace datasource lane instead of --diff-dir.",
        help_heading = "Input Options"
    )]
    pub local: bool,
    #[arg(
        long,
        value_enum,
        requires = "local",
        help = "With --local and no --diff-dir, select which artifact run to read from. Defaults to latest.",
        help_heading = "Input Options"
    )]
    pub run: Option<ArtifactRunMode>,
    #[arg(
        long = "run-id",
        requires = "local",
        help = "With --local and no --diff-dir, diff from this explicit artifact run id (overrides --run).",
        help_heading = "Input Options"
    )]
    pub run_id: Option<String>,
    #[arg(
        long,
        value_enum,
        default_value_t = DatasourceImportInputFormat::Inventory,
        help = "Select the datasource diff input format. Use inventory for the existing datasources.json export contract, or provisioning to load Grafana datasource provisioning YAML from an export root, provisioning directory, or concrete datasources.yaml file.",
        help_heading = "Input Options"
    )]
    pub input_format: DatasourceImportInputFormat,
    #[arg(
        long = "output-format",
        value_enum,
        default_value_t = DiffOutputFormat::Text,
        help = "Render diff output as text or json."
    )]
    pub output_format: DiffOutputFormat,
}

#[derive(Debug, Clone, Args)]
pub struct DatasourcePlanArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(
        long = "input-dir",
        required_unless_present = "local",
        conflicts_with = "local",
        help = "Local datasource bundle to plan against live Grafana. For provisioning input, point this at the export root, provisioning directory, or concrete datasources.yaml file.",
        help_heading = "Input Options"
    )]
    pub input_dir: Option<PathBuf>,
    #[arg(
        long,
        default_value_t = false,
        help = "Plan against datasource inventory from the artifact workspace datasource lane instead of --input-dir.",
        help_heading = "Input Options"
    )]
    pub local: bool,
    #[arg(
        long,
        value_enum,
        requires = "local",
        help = "With --local and no --input-dir, select which artifact run to read from. Defaults to latest.",
        help_heading = "Input Options"
    )]
    pub run: Option<ArtifactRunMode>,
    #[arg(
        long = "run-id",
        requires = "local",
        help = "With --local and no --input-dir, plan from this explicit artifact run id (overrides --run).",
        help_heading = "Input Options"
    )]
    pub run_id: Option<String>,
    #[arg(
        long,
        value_enum,
        default_value_t = DatasourceImportInputFormat::Inventory,
        help = "Select the datasource plan input format. Use inventory for datasources.json exports, or provisioning for Grafana datasource provisioning YAML.",
        help_heading = "Input Options"
    )]
    pub input_format: DatasourceImportInputFormat,
    #[arg(
        long,
        conflicts_with = "use_export_org",
        help = "Plan against this Grafana org ID instead of the current org context. Requires Basic auth.",
        help_heading = "Target Options"
    )]
    pub org_id: Option<i64>,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "org_id",
        help = "Plan a combined multi-org datasource export root by routing each org-scoped bundle back into the matching Grafana org. Requires Basic auth.",
        help_heading = "Target Options"
    )]
    pub use_export_org: bool,
    #[arg(
        long = "only-org-id",
        requires = "use_export_org",
        conflicts_with = "org_id",
        help = "With --use-export-org, plan only these exported source org IDs. Repeat the flag to select multiple orgs.",
        help_heading = "Target Options"
    )]
    pub only_org_id: Vec<i64>,
    #[arg(
        long,
        default_value_t = false,
        requires = "use_export_org",
        help = "With --use-export-org, mark missing destination orgs as would-create instead of blocked.",
        help_heading = "Target Options"
    )]
    pub create_missing_orgs: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Include remote-only datasources as would-delete candidates. Plan remains review-only and does not mutate Grafana.",
        help_heading = "Safety Options"
    )]
    pub prune: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Include same/unchanged rows in text and table output.",
        help_heading = "Output Options"
    )]
    pub show_same: bool,
    #[arg(
        long,
        value_enum,
        default_value_t = DatasourcePlanOutputFormat::Text,
        help = "Render datasource plan output as text, table, or json.",
        help_heading = "Output Options"
    )]
    pub output_format: DatasourcePlanOutputFormat,
    #[arg(
        long,
        default_value_t = false,
        help = "For table output only, omit the table header row.",
        help_heading = "Output Options"
    )]
    pub no_header: bool,
    #[arg(
        long,
        value_delimiter = ',',
        value_parser = parse_datasource_plan_output_column,
        help = "For table output only, render only these comma-separated columns. Supported values: action_id, action, status, uid, name, type, match_basis, source_org_id, target_org_id, target_uid, target_version, target_read_only, changed_fields, blocked_reason, source_file.",
        help_heading = "Output Options"
    )]
    pub output_columns: Vec<String>,
    #[arg(
        long,
        default_value_t = false,
        help = "Print the supported --output-columns values and exit.",
        help_heading = "Output Options"
    )]
    pub list_columns: bool,
}
