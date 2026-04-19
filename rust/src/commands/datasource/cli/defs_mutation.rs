use clap::{ArgAction, Args};

use crate::dashboard::CommonCliArgs;
use crate::datasource_catalog::DatasourcePresetProfile;

use super::{parse_bool_choice, DryRunOutputFormat};

#[derive(Debug, Clone, Args)]
pub struct DatasourceAddArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(
        long,
        help = "Datasource UID to create. Optional but recommended for stable identity."
    )]
    pub uid: Option<String>,
    #[arg(long, help = "Datasource name to create.")]
    pub name: String,
    #[arg(
        long = "type",
        help = "Grafana datasource plugin type id to create. Supported aliases from `datasource types` are normalized to canonical type ids."
    )]
    pub datasource_type: String,
    #[arg(
        long,
        default_value_t = false,
        help = "Legacy shortcut for starter preset defaults on supported datasource types."
    )]
    pub apply_supported_defaults: bool,
    #[arg(
        long,
        value_enum,
        help = "Apply a preset profile for supported datasource types. Use starter to match --apply-supported-defaults or full for a richer scaffold."
    )]
    pub preset_profile: Option<DatasourcePresetProfile>,
    #[arg(long, help = "Datasource access mode such as proxy or direct.")]
    pub access: Option<String>,
    #[arg(long, help = "Datasource target URL to store in Grafana.")]
    pub datasource_url: Option<String>,
    #[arg(
        long = "default",
        default_value_t = false,
        help = "Mark the new datasource as the default datasource."
    )]
    pub is_default: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Enable basic auth for the datasource."
    )]
    pub basic_auth: bool,
    #[arg(long, help = "Username for datasource basic auth.")]
    pub basic_auth_user: Option<String>,
    #[arg(
        long,
        help = "Password for datasource basic auth. Stored in secureJsonData."
    )]
    pub basic_auth_password: Option<String>,
    #[arg(
        long,
        help = "Datasource user/login field where the plugin supports it."
    )]
    pub user: Option<String>,
    #[arg(
        long = "password",
        help = "Datasource password field where the plugin supports it. Stored in secureJsonData."
    )]
    pub datasource_password: Option<String>,
    #[arg(
        long,
        default_value_t = false,
        help = "Send browser credentials such as cookies for supported datasource types."
    )]
    pub with_credentials: bool,
    #[arg(long, action = ArgAction::Append, help = "Add one custom HTTP header for supported datasource types. May be specified multiple times.", value_name = "NAME=VALUE")]
    pub http_header: Vec<String>,
    #[arg(
        long,
        default_value_t = false,
        help = "Set jsonData.tlsSkipVerify=true for supported datasource types."
    )]
    pub tls_skip_verify: bool,
    #[arg(
        long,
        help = "Set jsonData.serverName for supported datasource TLS validation."
    )]
    pub server_name: Option<String>,
    #[arg(long, help = "Inline JSON object string for datasource jsonData.")]
    pub json_data: Option<String>,
    #[arg(
        long,
        help = "Inline JSON object string for datasource secureJsonData."
    )]
    pub secure_json_data: Option<String>,
    #[arg(
        long,
        help = "Inline JSON object string mapping secureJsonData field names to ${secret:...} placeholders."
    )]
    pub secure_json_data_placeholders: Option<String>,
    #[arg(
        long,
        help = "Inline JSON object string mapping secret placeholder names to resolved secret values for add."
    )]
    pub secret_values: Option<String>,
    #[arg(
        long,
        default_value_t = false,
        help = "Preview what datasource add would do without changing Grafana."
    )]
    pub dry_run: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "For --dry-run only, render a compact table instead of plain text."
    )]
    pub table: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "For --dry-run only, render one JSON document."
    )]
    pub json: bool,
    #[arg(long, value_enum, conflicts_with_all = ["table", "json"], help = "Alternative single-flag output selector for datasource add dry-run output. Use text, table, or json.")]
    pub output_format: Option<DryRunOutputFormat>,
    #[arg(
        long,
        default_value_t = false,
        help = "For --dry-run --table only, omit the table header row."
    )]
    pub no_header: bool,
}

#[derive(Debug, Clone, Args)]
pub struct DatasourceDeleteArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(
        long,
        required_unless_present = "name",
        conflicts_with = "name",
        help = "Datasource UID to delete.",
        help_heading = "Target Options"
    )]
    pub uid: Option<String>,
    #[arg(
        long,
        required_unless_present = "uid",
        conflicts_with = "uid",
        help = "Datasource name to delete when UID is not available.",
        help_heading = "Target Options"
    )]
    pub name: Option<String>,
    #[arg(
        long,
        default_value_t = false,
        help = "Acknowledge the live datasource delete. Required unless --dry-run is set.",
        help_heading = "Safety Options"
    )]
    pub yes: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Preview what datasource delete would do without changing Grafana.",
        help_heading = "Output Options"
    )]
    pub dry_run: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "For --dry-run only, render a compact table instead of plain text.",
        help_heading = "Output Options"
    )]
    pub table: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "For --dry-run only, render one JSON document.",
        help_heading = "Output Options"
    )]
    pub json: bool,
    #[arg(long, value_enum, conflicts_with_all = ["table", "json"], help = "Alternative single-flag output selector for datasource delete dry-run output. Use text, table, or json.", help_heading = "Output Options")]
    pub output_format: Option<DryRunOutputFormat>,
    #[arg(
        long,
        default_value_t = false,
        help = "For --dry-run --table only, omit the table header row.",
        help_heading = "Output Options"
    )]
    pub no_header: bool,
}

#[derive(Debug, Clone, Args)]
pub struct DatasourceModifyArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(long, help = "Datasource UID to modify.")]
    pub uid: String,
    #[arg(long, help = "Replace the datasource URL stored in Grafana.")]
    pub set_url: Option<String>,
    #[arg(
        long,
        help = "Replace the datasource access mode such as proxy or direct."
    )]
    pub set_access: Option<String>,
    #[arg(
        long,
        value_parser = parse_bool_choice,
        help = "Set whether Grafana treats this datasource as default. Use true or false."
    )]
    pub set_default: Option<bool>,
    #[arg(
        long,
        default_value_t = false,
        help = "Enable basic auth for the datasource."
    )]
    pub basic_auth: bool,
    #[arg(long, help = "Replace datasource basic auth username.")]
    pub basic_auth_user: Option<String>,
    #[arg(
        long,
        help = "Replace datasource basic auth password. Stored in secureJsonData."
    )]
    pub basic_auth_password: Option<String>,
    #[arg(
        long,
        help = "Replace datasource user/login field where the plugin supports it."
    )]
    pub user: Option<String>,
    #[arg(
        long = "password",
        help = "Replace datasource password field where the plugin supports it. Stored in secureJsonData."
    )]
    pub datasource_password: Option<String>,
    #[arg(
        long,
        default_value_t = false,
        help = "Set withCredentials=true for supported datasource types."
    )]
    pub with_credentials: bool,
    #[arg(long, action = ArgAction::Append, help = "Replace or add one custom HTTP header for supported datasource types. May be specified multiple times.", value_name = "NAME=VALUE")]
    pub http_header: Vec<String>,
    #[arg(
        long,
        default_value_t = false,
        help = "Set jsonData.tlsSkipVerify=true for supported datasource types."
    )]
    pub tls_skip_verify: bool,
    #[arg(
        long,
        help = "Set jsonData.serverName for supported datasource TLS validation."
    )]
    pub server_name: Option<String>,
    #[arg(
        long,
        help = "Inline JSON object string to merge into datasource jsonData."
    )]
    pub json_data: Option<String>,
    #[arg(
        long,
        help = "Inline JSON object string to send in datasource secureJsonData."
    )]
    pub secure_json_data: Option<String>,
    #[arg(
        long,
        help = "Inline JSON object string mapping secureJsonData field names to ${secret:...} placeholders."
    )]
    pub secure_json_data_placeholders: Option<String>,
    #[arg(
        long,
        help = "Inline JSON object string mapping secret placeholder names to resolved secret values for modify."
    )]
    pub secret_values: Option<String>,
    #[arg(
        long,
        default_value_t = false,
        help = "Preview what datasource modify would do without changing Grafana."
    )]
    pub dry_run: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "For --dry-run only, render a compact table instead of plain text."
    )]
    pub table: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "For --dry-run only, render one JSON document."
    )]
    pub json: bool,
    #[arg(long, value_enum, conflicts_with_all = ["table", "json"], help = "Alternative single-flag output selector for datasource modify dry-run output. Use text, table, or json.")]
    pub output_format: Option<DryRunOutputFormat>,
    #[arg(
        long,
        default_value_t = false,
        help = "For --dry-run --table only, omit the table header row."
    )]
    pub no_header: bool,
}
