use clap::{Args, ValueEnum};
use std::path::PathBuf;

use super::{AccessArtifactRunMode, CommonCliArgs, PlanOutputFormat};

fn parse_access_plan_output_column(value: &str) -> std::result::Result<String, String> {
    match value {
        "all" => Ok("all".to_string()),
        "action_id" | "actionId" => Ok("action_id".to_string()),
        "resource_kind" | "resourceKind" => Ok("resource_kind".to_string()),
        "identity" => Ok("identity".to_string()),
        "action" => Ok("action".to_string()),
        "status" => Ok("status".to_string()),
        "changed_fields" | "changedFields" => Ok("changed_fields".to_string()),
        "changes" => Ok("changes".to_string()),
        "target" => Ok("target".to_string()),
        "blocked_reason" | "blockedReason" => Ok("blocked_reason".to_string()),
        "review_hints" | "reviewHints" => Ok("review_hints".to_string()),
        "source_path" | "sourcePath" => Ok("source_path".to_string()),
        _ => Err(format!(
            "Unsupported --output-columns value '{value}'. Supported values: all, action_id, resource_kind, identity, action, status, changed_fields, changes, target, blocked_reason, review_hints, source_path."
        )),
    }
}

/// Resource selector for access plan review flows.
#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
pub enum AccessPlanResource {
    User,
    Team,
    Org,
    #[value(name = "service-account")]
    ServiceAccount,
    All,
}

/// Access plan review arguments.
#[derive(Debug, Clone, Args)]
pub struct AccessPlanArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(
        long = "input-dir",
        required_unless_present = "local",
        help = "Directory that contains one or more access export bundles to review."
    )]
    pub input_dir: Option<PathBuf>,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "input_dir",
        help = "Read the selected access bundle from the artifact workspace instead of --input-dir."
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
    #[arg(
        long,
        value_enum,
        default_value_t = AccessPlanResource::User,
        help = "Select which access resource bundle to plan. Use all to aggregate user, org, team, and service-account bundles from one root directory."
    )]
    pub resource: AccessPlanResource,
    #[arg(
        long,
        default_value_t = false,
        help = "Include remote-only resources as delete candidates instead of just reporting them."
    )]
    pub prune: bool,
    #[arg(
        long,
        value_delimiter = ',',
        value_parser = parse_access_plan_output_column,
        help = "For text or table output, render only these comma-separated columns. Use all to expand every supported column. Supported values: all, action_id, resource_kind, identity, action, status, changed_fields, changes, target, blocked_reason, review_hints, source_path. JSON-style aliases like actionId and resourceKind are also accepted."
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
        default_value_t = false,
        help = "Hide the header row from table output."
    )]
    pub no_header: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Keep same-state rows visible in table output."
    )]
    pub show_same: bool,
    #[arg(
        long,
        value_enum,
        default_value_t = PlanOutputFormat::Text,
        help = "Output format for access plan review. Use text, table, or json."
    )]
    pub output_format: PlanOutputFormat,
}
