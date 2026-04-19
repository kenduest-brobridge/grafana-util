use reqwest::Method;
use serde_json::Value;

use crate::artifact_workspace::ArtifactLane;
use crate::common::{message, Result};

use super::super::browse_support::BrowseSwitch;
use super::super::browse_terminal::TerminalSession;
use super::super::cli_defs::build_http_client;
use super::super::{pending_delete, team, team_browse, user_browse};
use super::super::{AccessCliArgs, AccessCommand, TeamCommand};
use super::dispatch_artifacts::{
    export_access_lane_path, local_access_lane_path, resolve_required_access_diff_dir,
    resolve_required_access_input_dir, run_access_cli_with_common,
};

pub(super) fn run_team_access_cli(command: &TeamCommand, args: &AccessCliArgs) -> Result<()> {
    match command {
        TeamCommand::List(inner) => {
            let mut inner = inner.clone();
            if inner.local && inner.input_dir.is_none() {
                inner.input_dir = Some(local_access_lane_path(
                    inner.common.profile.as_deref(),
                    inner.run,
                    inner.run_id.as_deref(),
                    ArtifactLane::AccessTeams,
                )?);
            }
            if inner.input_dir.is_some() {
                team::list_teams_from_input_dir(&inner)?;
                Ok(())
            } else {
                run_access_cli_with_common(&inner.common, args, build_http_client)
            }
        }
        TeamCommand::Browse(inner) => {
            let mut inner = inner.clone();
            if inner.local && inner.input_dir.is_none() {
                inner.input_dir = Some(local_access_lane_path(
                    inner.common.profile.as_deref(),
                    inner.run,
                    inner.run_id.as_deref(),
                    ArtifactLane::AccessTeams,
                )?);
            }
            if inner.input_dir.is_some() {
                let local_args = AccessCliArgs {
                    command: AccessCommand::Team {
                        command: TeamCommand::Browse(inner),
                    },
                };
                super::run_access_cli_with_request(
                    |_method, _path, _params, _payload| {
                        Err(message(
                            "Local access team browse should not call the live request layer.",
                        ))
                    },
                    &local_args,
                )
            } else {
                run_access_cli_with_common(&inner.common, args, build_http_client)
            }
        }
        TeamCommand::Add(inner) => {
            run_access_cli_with_common(&inner.common, args, build_http_client)
        }
        TeamCommand::Modify(inner) => {
            run_access_cli_with_common(&inner.common, args, build_http_client)
        }
        TeamCommand::Export(inner) => {
            let mut inner = inner.clone();
            if let Some(output_dir) = export_access_lane_path(
                inner.common.profile.as_deref(),
                inner.run,
                inner.run_id.as_deref(),
                ArtifactLane::AccessTeams,
                inner.dry_run,
            )? {
                inner.output_dir = output_dir;
                let resolved_args = AccessCliArgs {
                    command: AccessCommand::Team {
                        command: TeamCommand::Export(inner),
                    },
                };
                run_access_cli_with_common(
                    match &resolved_args.command {
                        AccessCommand::Team {
                            command: TeamCommand::Export(inner),
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
        TeamCommand::Import(inner) => {
            let mut inner = inner.clone();
            resolve_required_access_input_dir(
                &mut inner.input_dir,
                inner.common.profile.as_deref(),
                inner.run,
                inner.run_id.as_deref(),
                inner.local,
                ArtifactLane::AccessTeams,
                "access team import",
            )?;
            let resolved_args = AccessCliArgs {
                command: AccessCommand::Team {
                    command: TeamCommand::Import(inner),
                },
            };
            run_access_cli_with_common(
                match &resolved_args.command {
                    AccessCommand::Team {
                        command: TeamCommand::Import(inner),
                    } => &inner.common,
                    _ => unreachable!(),
                },
                &resolved_args,
                build_http_client,
            )
        }
        TeamCommand::Diff(inner) => {
            let mut inner = inner.clone();
            resolve_required_access_diff_dir(
                &mut inner.diff_dir,
                inner.common.profile.as_deref(),
                inner.run,
                inner.run_id.as_deref(),
                inner.local,
                ArtifactLane::AccessTeams,
                "access team diff",
            )?;
            let resolved_args = AccessCliArgs {
                command: AccessCommand::Team {
                    command: TeamCommand::Diff(inner),
                },
            };
            run_access_cli_with_common(
                match &resolved_args.command {
                    AccessCommand::Team {
                        command: TeamCommand::Diff(inner),
                    } => &inner.common,
                    _ => unreachable!(),
                },
                &resolved_args,
                build_http_client,
            )
        }
        TeamCommand::Delete(inner) => {
            run_access_cli_with_common(&inner.common, args, build_http_client)
        }
    }
}

pub(super) fn run_team_access_cli_with_request<F>(
    command: &TeamCommand,
    request_json: &mut F,
    _args: &AccessCliArgs,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    match command {
        TeamCommand::List(args) => {
            if args.input_dir.is_some() {
                let _ = team::list_teams_from_input_dir(args)?;
            } else {
                let _ = team::list_teams_command_with_request(request_json, args)?;
            }
        }
        TeamCommand::Browse(args) => {
            #[cfg(feature = "tui")]
            {
                let mut session = TerminalSession::enter()?;
                let mut next = BrowseSwitch::ToTeam(args.clone());
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
                let _ = team_browse::browse_teams_with_request(request_json, args)?;
            }
        }
        TeamCommand::Add(args) => {
            let _ = team::add_team_with_request(request_json, args)?;
        }
        TeamCommand::Modify(args) => {
            let _ = team::modify_team_with_request(request_json, args)?;
        }
        TeamCommand::Export(args) => {
            let _ = team::export_teams_with_request(request_json, args)?;
        }
        TeamCommand::Import(args) => {
            let _ = team::import_teams_with_request(request_json, args)?;
        }
        TeamCommand::Diff(args) => {
            let _ = team::diff_teams_with_request(request_json, args)?;
        }
        TeamCommand::Delete(args) => {
            let _ = pending_delete::delete_team_with_request(request_json, args)?;
        }
    }
    Ok(())
}
