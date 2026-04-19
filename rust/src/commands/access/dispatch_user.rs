use reqwest::Method;
use serde_json::Value;

use crate::artifact_workspace::ArtifactLane;
use crate::common::{message, Result};

use super::super::browse_support::BrowseSwitch;
use super::super::browse_terminal::TerminalSession;
use super::super::cli_defs::build_http_client;
use super::super::{team_browse, user, user_browse};
use super::super::{AccessCliArgs, AccessCommand, UserCommand};
use super::dispatch_artifacts::{
    export_access_lane_path, local_access_lane_path, resolve_required_access_diff_dir,
    resolve_required_access_input_dir, run_access_cli_with_common,
};

pub(super) fn run_user_access_cli(command: &UserCommand, args: &AccessCliArgs) -> Result<()> {
    match command {
        UserCommand::List(inner) => {
            let mut inner = inner.clone();
            if inner.local && inner.input_dir.is_none() {
                inner.input_dir = Some(local_access_lane_path(
                    inner.common.profile.as_deref(),
                    inner.run,
                    inner.run_id.as_deref(),
                    ArtifactLane::AccessUsers,
                )?);
            }
            if inner.input_dir.is_some() {
                user::list_users_from_input_dir(&inner)?;
                Ok(())
            } else {
                run_access_cli_with_common(&inner.common, args, build_http_client)
            }
        }
        UserCommand::Browse(inner) => {
            let mut inner = inner.clone();
            if inner.local && inner.input_dir.is_none() {
                inner.input_dir = Some(local_access_lane_path(
                    inner.common.profile.as_deref(),
                    inner.run,
                    inner.run_id.as_deref(),
                    ArtifactLane::AccessUsers,
                )?);
            }
            if inner.input_dir.is_some() {
                let local_args = AccessCliArgs {
                    command: AccessCommand::User {
                        command: UserCommand::Browse(inner),
                    },
                };
                super::run_access_cli_with_request(
                    |_method, _path, _params, _payload| {
                        Err(message(
                            "Local access user browse should not call the live request layer.",
                        ))
                    },
                    &local_args,
                )
            } else {
                run_access_cli_with_common(&inner.common, args, build_http_client)
            }
        }
        UserCommand::Add(inner) => {
            run_access_cli_with_common(&inner.common, args, build_http_client)
        }
        UserCommand::Modify(inner) => {
            run_access_cli_with_common(&inner.common, args, build_http_client)
        }
        UserCommand::Export(inner) => {
            let mut inner = inner.clone();
            if let Some(output_dir) = export_access_lane_path(
                inner.common.profile.as_deref(),
                inner.run,
                inner.run_id.as_deref(),
                ArtifactLane::AccessUsers,
                inner.dry_run,
            )? {
                inner.output_dir = output_dir;
                let resolved_args = AccessCliArgs {
                    command: AccessCommand::User {
                        command: UserCommand::Export(inner),
                    },
                };
                run_access_cli_with_common(
                    match &resolved_args.command {
                        AccessCommand::User {
                            command: UserCommand::Export(inner),
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
        UserCommand::Import(inner) => {
            let mut inner = inner.clone();
            resolve_required_access_input_dir(
                &mut inner.input_dir,
                inner.common.profile.as_deref(),
                inner.run,
                inner.run_id.as_deref(),
                inner.local,
                ArtifactLane::AccessUsers,
                "access user import",
            )?;
            let resolved_args = AccessCliArgs {
                command: AccessCommand::User {
                    command: UserCommand::Import(inner),
                },
            };
            run_access_cli_with_common(
                match &resolved_args.command {
                    AccessCommand::User {
                        command: UserCommand::Import(inner),
                    } => &inner.common,
                    _ => unreachable!(),
                },
                &resolved_args,
                build_http_client,
            )
        }
        UserCommand::Diff(inner) => {
            let mut inner = inner.clone();
            resolve_required_access_diff_dir(
                &mut inner.diff_dir,
                inner.common.profile.as_deref(),
                inner.run,
                inner.run_id.as_deref(),
                inner.local,
                ArtifactLane::AccessUsers,
                "access user diff",
            )?;
            let resolved_args = AccessCliArgs {
                command: AccessCommand::User {
                    command: UserCommand::Diff(inner),
                },
            };
            run_access_cli_with_common(
                match &resolved_args.command {
                    AccessCommand::User {
                        command: UserCommand::Diff(inner),
                    } => &inner.common,
                    _ => unreachable!(),
                },
                &resolved_args,
                build_http_client,
            )
        }
        UserCommand::Delete(inner) => {
            run_access_cli_with_common(&inner.common, args, build_http_client)
        }
    }
}

pub(super) fn run_user_access_cli_with_request<F>(
    command: &UserCommand,
    request_json: &mut F,
    _args: &AccessCliArgs,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    match command {
        UserCommand::List(args) => {
            if args.input_dir.is_some() {
                let _ = user::list_users_from_input_dir(args)?;
            } else {
                let _ = user::list_users_with_request(request_json, args)?;
            }
        }
        UserCommand::Browse(args) => {
            #[cfg(feature = "tui")]
            {
                let mut session = TerminalSession::enter()?;
                let mut next = BrowseSwitch::ToUser(args.clone());
                loop {
                    next = match next {
                        BrowseSwitch::Exit => break,
                        BrowseSwitch::ToUser(inner) => user_browse::browse_users_in_session(
                            &mut session,
                            &mut *request_json,
                            &inner,
                        )?,
                        BrowseSwitch::ToTeam(inner) => team_browse::browse_teams_in_session(
                            &mut session,
                            &mut *request_json,
                            &inner,
                        )?,
                    };
                }
            }
            #[cfg(not(feature = "tui"))]
            {
                let _ = user_browse::browse_users_with_request(request_json, args)?;
            }
        }
        UserCommand::Add(args) => {
            let _ = user::add_user_with_request(request_json, args)?;
        }
        UserCommand::Modify(args) => {
            let _ = user::modify_user_with_request(request_json, args)?;
        }
        UserCommand::Export(args) => {
            let _ = user::export_users_with_request(request_json, args)?;
        }
        UserCommand::Import(args) => {
            let _ = user::import_users_with_request(request_json, args)?;
        }
        UserCommand::Diff(args) => {
            let _ = user::diff_users_with_request(request_json, args)?;
        }
        UserCommand::Delete(args) => {
            let _ = user::delete_user_with_request(request_json, args)?;
        }
    }
    Ok(())
}
