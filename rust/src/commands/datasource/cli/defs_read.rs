use clap::{Args, ValueEnum};
use std::path::PathBuf;

use crate::dashboard::{CommonCliArgs, SimpleOutputFormat};

use super::{parse_datasource_list_output_column, DatasourceImportInputFormat, ListOutputFormat};

/// Artifact workspace run selector for commands that integrate with artifact workspace lanes.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ArtifactRunMode {
    Latest,
    Timestamp,
}

#[derive(Debug, Clone, Args)]
pub struct DatasourceTypesArgs {
    #[arg(
        long,
        value_enum,
        default_value_t = SimpleOutputFormat::Text,
        help = "Render the supported datasource catalog as text, table, csv, json, or yaml."
    )]
    pub output_format: SimpleOutputFormat,
}

#[derive(Debug, Clone, Args)]
pub struct DatasourceListArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(
        long,
        help = "Read datasource inventory from this local export root, provisioning directory, or concrete datasources.yaml file instead of Grafana.",
        help_heading = "Input Options"
    )]
    pub input_dir: Option<PathBuf>,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "input_dir",
        help = "Read datasource inventory from the artifact workspace instead of live Grafana. When set (and --input-dir is omitted), the datasource lane is resolved from the selected profile and --run/--run-id.",
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
        help = "With --local and no --input-dir, read from this explicit artifact run id (overrides --run).",
        help_heading = "Input Options"
    )]
    pub run_id: Option<String>,
    #[arg(
        long,
        value_enum,
        help = "For --input-dir only, select inventory or provisioning when the local path could be interpreted either way.",
        help_heading = "Input Options"
    )]
    pub input_format: Option<DatasourceImportInputFormat>,
    #[arg(
        long,
        conflicts_with = "all_orgs",
        help = "List datasources from one explicit Grafana org ID instead of the current org. Requires Basic auth."
    )]
    pub org_id: Option<i64>,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "org_id",
        help = "Enumerate all visible Grafana orgs and aggregate datasource inventory across them. Requires Basic auth."
    )]
    pub all_orgs: bool,
    #[arg(long, default_value_t = false, conflicts_with_all = ["table", "csv", "json", "yaml", "output_format"], help = "Render datasource summaries as plain text.", help_heading = "Output Options")]
    pub text: bool,
    #[arg(long, default_value_t = false, conflicts_with_all = ["text", "csv", "json", "yaml", "output_format"], help = "Render datasource summaries as a table.", help_heading = "Output Options")]
    pub table: bool,
    #[arg(long, default_value_t = false, conflicts_with_all = ["text", "table", "json", "yaml", "output_format"], help = "Render datasource summaries as CSV.", help_heading = "Output Options")]
    pub csv: bool,
    #[arg(long, default_value_t = false, conflicts_with_all = ["text", "table", "csv", "yaml", "output_format"], help = "Render datasource summaries as JSON.", help_heading = "Output Options")]
    pub json: bool,
    #[arg(long, default_value_t = false, conflicts_with_all = ["text", "table", "csv", "json", "output_format"], help = "Render datasource summaries as YAML.", help_heading = "Output Options")]
    pub yaml: bool,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with_all = ["text", "table", "csv", "json", "yaml", "output_format"],
        help = "For --input-dir only, open the local datasource inventory workbench.",
        help_heading = "Output Options"
    )]
    pub interactive: bool,
    #[arg(
        long,
        value_enum,
        conflicts_with_all = ["text", "table", "csv", "json", "yaml", "interactive"],
        help = "Alternative single-flag output selector. Use text, table, csv, json, or yaml.",
        help_heading = "Output Options"
    )]
    pub output_format: Option<ListOutputFormat>,
    #[arg(
        long,
        default_value_t = false,
        help = "Do not print table headers when rendering the default table output.",
        help_heading = "Output Options"
    )]
    pub no_header: bool,
    #[arg(
        long,
        value_delimiter = ',',
        value_parser = parse_datasource_list_output_column,
        help = "Render only these comma-separated datasource fields. Use all to expand every discovered field in human-readable output. Common ids include uid, name, type, access, url, is_default, database, org, org_id, plus nested paths such as jsonData.organization or secureJsonFields.basicAuthPassword. JSON-style aliases like isDefault and orgId are also accepted.",
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

#[derive(Debug, Clone, Args)]
pub struct DatasourceBrowseArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(
        long,
        conflicts_with = "all_orgs",
        help = "Browse datasources from one explicit Grafana org ID instead of the current org. Requires Basic auth."
    )]
    pub org_id: Option<i64>,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "org_id",
        help = "Enumerate all visible Grafana orgs and browse datasource inventory across them. Requires Basic auth."
    )]
    pub all_orgs: bool,
}
