//! Clap schema for access-management CLI commands.
//! Centralizes CLI argument enums and parser-normalization helpers for access handlers.
use clap::{Args, Parser, Subcommand, ValueEnum};

#[path = "access_cli_runtime.rs"]
mod access_cli_runtime;

#[path = "access_cli_shared.rs"]
mod access_cli_shared;

#[path = "access_service_account_cli.rs"]
mod access_service_account_cli;

use crate::common::CliColorChoice;

#[path = "access_org_cli.rs"]
mod access_org_cli;

#[path = "access_plan_cli.rs"]
mod access_plan_cli;

#[path = "access_team_cli.rs"]
mod access_team_cli;

pub use access_cli_runtime::{
    build_auth_context, build_auth_context_no_org_id, build_http_client,
    build_http_client_no_org_id, normalize_access_cli_args, parse_cli_from, root_command,
    AccessAuthContext,
};
pub(crate) use access_cli_runtime::{
    materialize_access_common_auth, materialize_access_common_auth_no_org_id,
};
pub use access_cli_shared::{
    CommonCliArgs, CommonCliArgsNoOrgId, DryRunOutputFormat, ListOutputFormat, PlanOutputFormat,
    Scope, ACCESS_EXPORT_KIND_ORGS, ACCESS_EXPORT_KIND_SERVICE_ACCOUNTS, ACCESS_EXPORT_KIND_TEAMS,
    ACCESS_EXPORT_KIND_USERS, ACCESS_EXPORT_METADATA_FILENAME, ACCESS_EXPORT_VERSION,
    ACCESS_ORG_EXPORT_FILENAME, ACCESS_SERVICE_ACCOUNT_EXPORT_FILENAME,
    ACCESS_TEAM_EXPORT_FILENAME, ACCESS_USER_EXPORT_FILENAME, DEFAULT_ACCESS_ORG_EXPORT_DIR,
    DEFAULT_ACCESS_SERVICE_ACCOUNT_EXPORT_DIR, DEFAULT_ACCESS_TEAM_EXPORT_DIR,
    DEFAULT_ACCESS_USER_EXPORT_DIR, DEFAULT_PAGE_SIZE, DEFAULT_TIMEOUT, DEFAULT_URL,
};
use access_cli_shared::{
    ACCESS_ORG_HELP_TEXT, ACCESS_PLAN_HELP_TEXT, ACCESS_ROOT_HELP_TEXT, ACCESS_TEAM_HELP_TEXT,
    ACCESS_USER_HELP_TEXT,
};
use access_service_account_cli::ACCESS_SERVICE_ACCOUNT_HELP_TEXT;
pub use access_service_account_cli::{
    ServiceAccountAddArgs, ServiceAccountCommand, ServiceAccountDiffArgs, ServiceAccountExportArgs,
    ServiceAccountImportArgs, ServiceAccountListArgs, ServiceAccountTokenAddArgs,
    ServiceAccountTokenCommand,
};
pub use access_org_cli::{
    OrgAddArgs, OrgCommand, OrgDeleteArgs, OrgDiffArgs, OrgExportArgs, OrgImportArgs, OrgListArgs,
    OrgModifyArgs,
};
pub use access_plan_cli::{AccessPlanArgs, AccessPlanResource};
pub use access_team_cli::{
    TeamAddArgs, TeamBrowseArgs, TeamCommand, TeamDiffArgs, TeamExportArgs, TeamImportArgs,
    TeamListArgs, TeamModifyArgs,
};

#[path = "access_user_cli.rs"]
mod access_user_cli;

pub use access_user_cli::{
    UserAddArgs, UserBrowseArgs, UserCommand, UserDeleteArgs, UserDiffArgs, UserExportArgs,
    UserImportArgs, UserListArgs, UserModifyArgs,
};

/// Artifact workspace run selector for access commands.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AccessArtifactRunMode {
    Latest,
    Timestamp,
}

/// User, organization, team, and service-account workflows.
#[derive(Debug, Clone, Subcommand)]
pub enum AccessCommand {
    #[command(after_help = ACCESS_USER_HELP_TEXT)]
    User {
        #[command(subcommand)]
        command: UserCommand,
    },
    #[command(after_help = ACCESS_ORG_HELP_TEXT)]
    Org {
        #[command(subcommand)]
        command: OrgCommand,
    },
    #[command(visible_alias = "group", after_help = ACCESS_TEAM_HELP_TEXT)]
    Team {
        #[command(subcommand)]
        command: TeamCommand,
    },
    #[command(name = "service-account", after_help = ACCESS_SERVICE_ACCOUNT_HELP_TEXT)]
    ServiceAccount {
        #[command(subcommand)]
        command: ServiceAccountCommand,
    },
    #[command(after_help = ACCESS_PLAN_HELP_TEXT)]
    Plan(AccessPlanArgs),
}

#[derive(Debug, Clone, Parser)]
#[command(
    name = "grafana-util access",
    about = "List and manage Grafana users, orgs, teams, and service accounts.",
    after_help = ACCESS_ROOT_HELP_TEXT,
    infer_long_args(true),
    styles = crate::help_styles::CLI_HELP_STYLES
)]
/// Struct definition for AccessCliRoot.
pub(crate) struct AccessCliRoot {
    #[arg(
        long,
        value_enum,
        default_value_t = CliColorChoice::Auto,
        help = "Colorize JSON output. Use auto, always, or never."
    )]
    color: CliColorChoice,
    #[command(flatten)]
    args: AccessCliArgs,
}

/// Struct definition for AccessCliArgs.
#[derive(Debug, Clone, Args)]
pub struct AccessCliArgs {
    #[command(subcommand)]
    pub command: AccessCommand,
}
