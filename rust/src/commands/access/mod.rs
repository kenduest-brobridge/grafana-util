//! Access-management domain orchestrator.
//!
//! Owns access command dispatch, argument normalization, and the shared parser/model re-exports
//! used by the CLI and tests.
use crate::common::Result;

// Internal modules stay split by resource kind so user/org/team/service-account
// workflows can evolve independently while this file keeps only domain routing.
#[path = "access_plan.rs"]
mod access_plan;
#[path = "access_plan_org.rs"]
mod access_plan_org;
#[path = "access_plan_team.rs"]
mod access_plan_team;
#[path = "artifact_workspace.rs"]
mod artifact_workspace;
#[path = "auth_materialize.rs"]
mod auth_materialize;
#[path = "browse_support.rs"]
mod browse_support;
#[path = "browse_terminal.rs"]
mod browse_terminal;
#[path = "cli_defs.rs"]
mod cli_defs;
#[path = "dispatch.rs"]
mod dispatch;
#[path = "facade_support.rs"]
mod facade_support;
#[path = "import_dry_run.rs"]
mod import_dry_run;
#[path = "live_project_status.rs"]
mod live_project_status;
#[path = "org.rs"]
mod org;
#[path = "pending_delete.rs"]
mod pending_delete;
#[path = "project_status.rs"]
mod project_status;
#[path = "render.rs"]
mod render;
#[path = "request_helpers.rs"]
mod request_helpers;
#[path = "service_account.rs"]
mod service_account;
#[path = "team.rs"]
mod team;
#[path = "team_browse.rs"]
mod team_browse;
#[path = "team_import_export_diff.rs"]
mod team_import_export_diff;
#[path = "team_runtime.rs"]
mod team_runtime;
#[path = "user.rs"]
mod user;
#[path = "user_browse.rs"]
mod user_browse;

pub use cli_defs::{
    build_auth_context, build_http_client, build_http_client_no_org_id, normalize_access_cli_args,
    parse_cli_from, root_command, AccessAuthContext, AccessCliArgs, AccessCommand, AccessPlanArgs,
    CommonCliArgs, CommonCliArgsNoOrgId, DryRunOutputFormat, OrgAddArgs, OrgCommand, OrgDeleteArgs,
    OrgDiffArgs, OrgExportArgs, OrgImportArgs, OrgListArgs, OrgModifyArgs, Scope,
    ServiceAccountAddArgs, ServiceAccountCommand, ServiceAccountDiffArgs, ServiceAccountExportArgs,
    ServiceAccountImportArgs, ServiceAccountListArgs, ServiceAccountTokenAddArgs,
    ServiceAccountTokenCommand, TeamAddArgs, TeamBrowseArgs, TeamCommand, TeamDiffArgs,
    TeamExportArgs, TeamImportArgs, TeamListArgs, TeamModifyArgs, UserAddArgs, UserBrowseArgs,
    UserCommand, UserDeleteArgs, UserDiffArgs, UserExportArgs, UserImportArgs, UserListArgs,
    UserModifyArgs, ACCESS_EXPORT_KIND_ORGS, ACCESS_EXPORT_KIND_SERVICE_ACCOUNTS,
    ACCESS_EXPORT_KIND_TEAMS, ACCESS_EXPORT_KIND_USERS, ACCESS_EXPORT_METADATA_FILENAME,
    ACCESS_EXPORT_VERSION, ACCESS_ORG_EXPORT_FILENAME, ACCESS_SERVICE_ACCOUNT_EXPORT_FILENAME,
    ACCESS_TEAM_EXPORT_FILENAME, ACCESS_USER_EXPORT_FILENAME, DEFAULT_ACCESS_ORG_EXPORT_DIR,
    DEFAULT_ACCESS_SERVICE_ACCOUNT_EXPORT_DIR, DEFAULT_ACCESS_TEAM_EXPORT_DIR,
    DEFAULT_ACCESS_USER_EXPORT_DIR, DEFAULT_PAGE_SIZE, DEFAULT_TIMEOUT, DEFAULT_URL,
};
#[allow(unused_imports)]
pub(crate) use facade_support::{
    build_access_live_domain_status, build_access_live_domain_status_with_request,
    build_access_live_read_failed_domain_status,
};
pub(crate) use import_dry_run::build_access_import_dry_run_document;
pub use pending_delete::{
    GroupCommandStage, ServiceAccountDeleteArgs, ServiceAccountTokenDeleteArgs, TeamDeleteArgs,
};
pub(crate) use project_status::{build_access_domain_status, AccessDomainStatusInputs};
use request_helpers::{request_array, request_object, request_object_list_field};

pub fn run_access_cli(args: AccessCliArgs) -> Result<()> {
    // Access CLI boundary:
    // normalize and materialize auth/headers once, then dispatch to user/org/team/service-account handlers.
    let args = normalize_access_cli_args(args);
    if let AccessCommand::Plan(plan_args) = &args.command {
        if plan_args.list_columns {
            access_plan::print_access_plan_columns();
            return Ok(());
        }
    }
    match &args.command {
        AccessCommand::User {
            command: UserCommand::List(inner),
        } if inner.list_columns => {
            let _ = user::list_users_from_input_dir(inner)?;
            return Ok(());
        }
        AccessCommand::Team {
            command: TeamCommand::List(inner),
        } if inner.list_columns => {
            let _ = team::list_teams_from_input_dir(inner)?;
            return Ok(());
        }
        AccessCommand::ServiceAccount {
            command: ServiceAccountCommand::List(inner),
        } if inner.list_columns => {
            let _ = service_account::list_service_accounts_from_input_dir(inner)?;
            return Ok(());
        }
        _ => {}
    }
    let args = auth_materialize::materialize_access_command_auth(args)?;
    dispatch::run_access_cli_with_materialized_args(&args)
}

pub use dispatch::{run_access_cli_with_client, run_access_cli_with_request};

#[cfg(test)]
#[path = "rust_tests.rs"]
mod access_rust_tests;
