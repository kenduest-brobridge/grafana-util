use reqwest::Method;
use serde_json::Value;

use crate::artifact_workspace::ArtifactLane;
use crate::common::Result;

use super::super::cli_defs::build_http_client;
use super::super::{pending_delete, service_account};
use super::super::{
    AccessCliArgs, AccessCommand, ServiceAccountCommand, ServiceAccountTokenCommand,
};
use super::dispatch_artifacts::{
    export_access_lane_path, local_access_lane_path, resolve_required_access_diff_dir,
    resolve_required_access_input_dir, run_access_cli_with_common,
};

pub(super) fn run_service_account_access_cli(
    command: &ServiceAccountCommand,
    args: &AccessCliArgs,
) -> Result<()> {
    match command {
        ServiceAccountCommand::List(inner) => {
            let mut inner = inner.clone();
            if inner.local && inner.input_dir.is_none() {
                inner.input_dir = Some(local_access_lane_path(
                    inner.common.profile.as_deref(),
                    inner.run,
                    inner.run_id.as_deref(),
                    ArtifactLane::AccessServiceAccounts,
                )?);
            }
            if inner.input_dir.is_some() {
                service_account::list_service_accounts_from_input_dir(&inner)?;
                Ok(())
            } else {
                run_access_cli_with_common(&inner.common, args, build_http_client)
            }
        }
        ServiceAccountCommand::Add(inner) => {
            run_access_cli_with_common(&inner.common, args, build_http_client)
        }
        ServiceAccountCommand::Export(inner) => {
            let mut inner = inner.clone();
            if let Some(output_dir) = export_access_lane_path(
                inner.common.profile.as_deref(),
                inner.run,
                inner.run_id.as_deref(),
                ArtifactLane::AccessServiceAccounts,
                inner.dry_run,
            )? {
                inner.output_dir = output_dir;
                let resolved_args = AccessCliArgs {
                    command: AccessCommand::ServiceAccount {
                        command: ServiceAccountCommand::Export(inner),
                    },
                };
                run_access_cli_with_common(
                    match &resolved_args.command {
                        AccessCommand::ServiceAccount {
                            command: ServiceAccountCommand::Export(inner),
                        } => &inner.common,
                        _ => unreachable!(),
                    },
                    &resolved_args,
                    build_http_client,
                )
            } else {
                run_access_cli_with_common(&inner.common, args, build_http_client)
            }
        }
        ServiceAccountCommand::Import(inner) => {
            let mut inner = inner.clone();
            resolve_required_access_input_dir(
                &mut inner.input_dir,
                inner.common.profile.as_deref(),
                inner.run,
                inner.run_id.as_deref(),
                inner.local,
                ArtifactLane::AccessServiceAccounts,
                "access service-account import",
            )?;
            let resolved_args = AccessCliArgs {
                command: AccessCommand::ServiceAccount {
                    command: ServiceAccountCommand::Import(inner),
                },
            };
            run_access_cli_with_common(
                match &resolved_args.command {
                    AccessCommand::ServiceAccount {
                        command: ServiceAccountCommand::Import(inner),
                    } => &inner.common,
                    _ => unreachable!(),
                },
                &resolved_args,
                build_http_client,
            )
        }
        ServiceAccountCommand::Diff(inner) => {
            let mut inner = inner.clone();
            resolve_required_access_diff_dir(
                &mut inner.diff_dir,
                inner.common.profile.as_deref(),
                inner.run,
                inner.run_id.as_deref(),
                inner.local,
                ArtifactLane::AccessServiceAccounts,
                "access service-account diff",
            )?;
            let resolved_args = AccessCliArgs {
                command: AccessCommand::ServiceAccount {
                    command: ServiceAccountCommand::Diff(inner),
                },
            };
            run_access_cli_with_common(
                match &resolved_args.command {
                    AccessCommand::ServiceAccount {
                        command: ServiceAccountCommand::Diff(inner),
                    } => &inner.common,
                    _ => unreachable!(),
                },
                &resolved_args,
                build_http_client,
            )
        }
        ServiceAccountCommand::Delete(inner) => {
            run_access_cli_with_common(&inner.common, args, build_http_client)
        }
        ServiceAccountCommand::Token { command } => match command {
            ServiceAccountTokenCommand::Add(inner) => {
                run_access_cli_with_common(&inner.common, args, build_http_client)
            }
            ServiceAccountTokenCommand::Delete(inner) => {
                run_access_cli_with_common(&inner.common, args, build_http_client)
            }
        },
    }
}

pub(super) fn run_service_account_access_cli_with_request<F>(
    command: &ServiceAccountCommand,
    request_json: &mut F,
    _args: &AccessCliArgs,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    match command {
        ServiceAccountCommand::List(args) => {
            if args.input_dir.is_some() {
                let _ = service_account::list_service_accounts_from_input_dir(args)?;
            } else {
                let _ = service_account::list_service_accounts_command_with_request(
                    request_json,
                    args,
                )?;
            }
        }
        ServiceAccountCommand::Add(args) => {
            let _ = service_account::add_service_account_with_request(request_json, args)?;
        }
        ServiceAccountCommand::Export(args) => {
            let _ = service_account::export_service_accounts_with_request(request_json, args)?;
        }
        ServiceAccountCommand::Import(args) => {
            let _ = service_account::import_service_accounts_with_request(request_json, args)?;
        }
        ServiceAccountCommand::Diff(args) => {
            let _ = service_account::diff_service_accounts_with_request(request_json, args)?;
        }
        ServiceAccountCommand::Delete(args) => {
            let _ = pending_delete::delete_service_account_with_request(request_json, args)?;
        }
        ServiceAccountCommand::Token { command } => match command {
            ServiceAccountTokenCommand::Add(args) => {
                let _ =
                    service_account::add_service_account_token_with_request(request_json, args)?;
            }
            ServiceAccountTokenCommand::Delete(args) => {
                let _ =
                    pending_delete::delete_service_account_token_with_request(request_json, args)?;
            }
        },
    }
    Ok(())
}
