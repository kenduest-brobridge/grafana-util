use reqwest::Method;
use serde_json::Value;

use crate::common::Result;
use crate::http::JsonHttpClient;

#[path = "dispatch_artifacts.rs"]
mod dispatch_artifacts;
#[path = "dispatch_org.rs"]
mod dispatch_org;
#[path = "dispatch_service_account.rs"]
mod dispatch_service_account;
#[path = "dispatch_team.rs"]
mod dispatch_team;
#[path = "dispatch_user.rs"]
mod dispatch_user;

use super::cli_defs::build_http_client;
use super::{access_plan, AccessCliArgs, AccessCommand};

pub fn run_access_cli_with_client(client: &JsonHttpClient, args: &AccessCliArgs) -> Result<()> {
    run_access_cli_with_request(
        |method, path, params, payload| client.request_json(method, path, params, payload),
        args,
    )
}

pub(crate) fn run_access_cli_with_materialized_args(args: &AccessCliArgs) -> Result<()> {
    match &args.command {
        AccessCommand::User { command } => dispatch_user::run_user_access_cli(command, args),
        AccessCommand::Org { command } => dispatch_org::run_org_access_cli(command, args),
        AccessCommand::Team { command } => dispatch_team::run_team_access_cli(command, args),
        AccessCommand::ServiceAccount { command } => {
            dispatch_service_account::run_service_account_access_cli(command, args)
        }
        AccessCommand::Plan(plan_args) => {
            let plan_args = dispatch_artifacts::resolve_access_plan_args(plan_args)?;
            let resolved_args = AccessCliArgs {
                command: AccessCommand::Plan(plan_args),
            };
            dispatch_artifacts::run_access_cli_with_common(
                match &resolved_args.command {
                    AccessCommand::Plan(plan_args) => &plan_args.common,
                    _ => unreachable!(),
                },
                &resolved_args,
                build_http_client,
            )
        }
    }
}

/// Access execution path with request-function injection.
///
/// Receives fully parsed CLI args and routes each command branch to matching handler
/// functions that perform request execution.
pub fn run_access_cli_with_request<F>(mut request_json: F, args: &AccessCliArgs) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    match &args.command {
        AccessCommand::User { command } => {
            dispatch_user::run_user_access_cli_with_request(command, &mut request_json, args)?
        }
        AccessCommand::Org { command } => {
            dispatch_org::run_org_access_cli_with_request(command, &mut request_json, args)?
        }
        AccessCommand::Team { command } => {
            dispatch_team::run_team_access_cli_with_request(command, &mut request_json, args)?
        }
        AccessCommand::ServiceAccount { command } => {
            dispatch_service_account::run_service_account_access_cli_with_request(
                command,
                &mut request_json,
                args,
            )?
        }
        AccessCommand::Plan(args) => {
            let args = dispatch_artifacts::resolve_access_plan_args(args)?;
            let _ = access_plan::access_plan_with_request(&mut request_json, &args)?;
        }
    }
    Ok(())
}
