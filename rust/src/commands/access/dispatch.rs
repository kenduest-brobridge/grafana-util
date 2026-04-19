use reqwest::Method;
use serde_json::Value;
use std::path::PathBuf;

use crate::artifact_workspace::{ArtifactLane, RunSelector};
use crate::common::{message, Result};
use crate::http::JsonHttpClient;

use super::artifact_workspace::{
    access_lane_path, access_run_root, access_run_selector, lane_for_plan_resource,
};
use super::cli_defs::{build_http_client, build_http_client_no_org_id};
use super::{
    access_plan, browse_support, browse_terminal, org, pending_delete, service_account, team,
    team_browse, user, user_browse, AccessCliArgs, AccessCommand, AccessPlanArgs, OrgCommand,
    ServiceAccountCommand, ServiceAccountTokenCommand, TeamCommand, UserCommand,
};

fn local_access_lane_path(
    profile: Option<&str>,
    run: Option<super::cli_defs::AccessArtifactRunMode>,
    run_id: Option<&str>,
    lane: ArtifactLane,
) -> Result<PathBuf> {
    access_lane_path(
        profile,
        access_run_selector(run, run_id, RunSelector::Latest),
        lane,
        false,
    )
}

fn export_access_lane_path(
    profile: Option<&str>,
    run: Option<super::cli_defs::AccessArtifactRunMode>,
    run_id: Option<&str>,
    lane: ArtifactLane,
    dry_run: bool,
) -> Result<Option<PathBuf>> {
    if run.is_none() && run_id.is_none() {
        return Ok(None);
    }
    access_lane_path(
        profile,
        access_run_selector(run, run_id, RunSelector::Timestamp),
        lane,
        !dry_run,
    )
    .map(Some)
}

fn resolve_access_plan_args(args: &AccessPlanArgs) -> Result<AccessPlanArgs> {
    let mut resolved = args.clone();
    if resolved.local && resolved.input_dir.is_none() {
        resolved.input_dir = if let Some(lane) = lane_for_plan_resource(&resolved.resource) {
            Some(local_access_lane_path(
                resolved.common.profile.as_deref(),
                resolved.run,
                resolved.run_id.as_deref(),
                lane,
            )?)
        } else {
            Some(access_run_root(
                resolved.common.profile.as_deref(),
                access_run_selector(
                    resolved.run,
                    resolved.run_id.as_deref(),
                    RunSelector::Latest,
                ),
                false,
            )?)
        };
    }
    Ok(resolved)
}

pub fn run_access_cli_with_client(client: &JsonHttpClient, args: &AccessCliArgs) -> Result<()> {
    run_access_cli_with_request(
        |method, path, params, payload| client.request_json(method, path, params, payload),
        args,
    )
}

fn run_access_cli_with_common<C, F>(common: &C, args: &AccessCliArgs, build_client: F) -> Result<()>
where
    F: FnOnce(&C) -> Result<JsonHttpClient>,
{
    let client = build_client(common)?;
    run_access_cli_with_client(&client, args)
}

fn run_user_access_cli(command: &UserCommand, args: &AccessCliArgs) -> Result<()> {
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
                run_access_cli_with_request(
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
            run_access_cli_with_common(&inner.common, args, build_http_client)
        }
        UserCommand::Diff(inner) => {
            run_access_cli_with_common(&inner.common, args, build_http_client)
        }
        UserCommand::Delete(inner) => {
            run_access_cli_with_common(&inner.common, args, build_http_client)
        }
    }
}

fn run_org_access_cli(command: &OrgCommand, args: &AccessCliArgs) -> Result<()> {
    match command {
        OrgCommand::List(inner) => {
            let mut inner = inner.clone();
            if inner.local && inner.input_dir.is_none() {
                inner.input_dir = Some(local_access_lane_path(
                    inner.common.profile.as_deref(),
                    inner.run,
                    inner.run_id.as_deref(),
                    ArtifactLane::AccessOrgs,
                )?);
            }
            if inner.input_dir.is_some() {
                org::list_orgs_from_input_dir(&inner)?;
                Ok(())
            } else {
                run_access_cli_with_common(&inner.common, args, build_http_client_no_org_id)
            }
        }
        OrgCommand::Add(inner) => {
            run_access_cli_with_common(&inner.common, args, build_http_client_no_org_id)
        }
        OrgCommand::Modify(inner) => {
            run_access_cli_with_common(&inner.common, args, build_http_client_no_org_id)
        }
        OrgCommand::Export(inner) => {
            let mut inner = inner.clone();
            if let Some(output_dir) = export_access_lane_path(
                inner.common.profile.as_deref(),
                inner.run,
                inner.run_id.as_deref(),
                ArtifactLane::AccessOrgs,
                inner.dry_run,
            )? {
                inner.output_dir = output_dir;
                let resolved_args = AccessCliArgs {
                    command: AccessCommand::Org {
                        command: OrgCommand::Export(inner),
                    },
                };
                run_access_cli_with_common(
                    match &resolved_args.command {
                        AccessCommand::Org {
                            command: OrgCommand::Export(inner),
                        } => &inner.common,
                        _ => unreachable!(),
                    },
                    &resolved_args,
                    build_http_client_no_org_id,
                )
            } else {
                run_access_cli_with_common(&inner.common, args, build_http_client_no_org_id)
            }
        }
        OrgCommand::Import(inner) => {
            run_access_cli_with_common(&inner.common, args, build_http_client_no_org_id)
        }
        OrgCommand::Diff(inner) => {
            run_access_cli_with_common(&inner.common, args, build_http_client_no_org_id)
        }
        OrgCommand::Delete(inner) => {
            run_access_cli_with_common(&inner.common, args, build_http_client_no_org_id)
        }
    }
}

fn run_team_access_cli(command: &TeamCommand, args: &AccessCliArgs) -> Result<()> {
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
                run_access_cli_with_request(
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
            run_access_cli_with_common(&inner.common, args, build_http_client)
        }
        TeamCommand::Diff(inner) => {
            run_access_cli_with_common(&inner.common, args, build_http_client)
        }
        TeamCommand::Delete(inner) => {
            run_access_cli_with_common(&inner.common, args, build_http_client)
        }
    }
}

fn run_service_account_access_cli(
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
            run_access_cli_with_common(&inner.common, args, build_http_client)
        }
        ServiceAccountCommand::Diff(inner) => {
            run_access_cli_with_common(&inner.common, args, build_http_client)
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

pub(crate) fn run_access_cli_with_materialized_args(args: &AccessCliArgs) -> Result<()> {
    match &args.command {
        AccessCommand::User { command } => run_user_access_cli(command, args),
        AccessCommand::Org { command } => run_org_access_cli(command, args),
        AccessCommand::Team { command } => run_team_access_cli(command, args),
        AccessCommand::ServiceAccount { command } => run_service_account_access_cli(command, args),
        AccessCommand::Plan(plan_args) => {
            let plan_args = resolve_access_plan_args(plan_args)?;
            let resolved_args = AccessCliArgs {
                command: AccessCommand::Plan(plan_args),
            };
            run_access_cli_with_common(
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
        AccessCommand::User { command } => match command {
            UserCommand::List(args) => {
                if args.input_dir.is_some() {
                    let _ = user::list_users_from_input_dir(args)?;
                } else {
                    let _ = user::list_users_with_request(&mut request_json, args)?;
                }
            }
            UserCommand::Browse(args) => {
                #[cfg(feature = "tui")]
                {
                    let mut session = browse_terminal::TerminalSession::enter()?;
                    let mut next = browse_support::BrowseSwitch::ToUser(args.clone());
                    loop {
                        next = match next {
                            browse_support::BrowseSwitch::Exit => break,
                            browse_support::BrowseSwitch::ToUser(inner) => {
                                user_browse::browse_users_in_session(
                                    &mut session,
                                    &mut request_json,
                                    &inner,
                                )?
                            }
                            browse_support::BrowseSwitch::ToTeam(inner) => {
                                team_browse::browse_teams_in_session(
                                    &mut session,
                                    &mut request_json,
                                    &inner,
                                )?
                            }
                        };
                    }
                }
                #[cfg(not(feature = "tui"))]
                {
                    let _ = user_browse::browse_users_with_request(&mut request_json, args)?;
                }
            }
            UserCommand::Add(args) => {
                let _ = user::add_user_with_request(&mut request_json, args)?;
            }
            UserCommand::Modify(args) => {
                let _ = user::modify_user_with_request(&mut request_json, args)?;
            }
            UserCommand::Export(args) => {
                let _ = user::export_users_with_request(&mut request_json, args)?;
            }
            UserCommand::Import(args) => {
                let _ = user::import_users_with_request(&mut request_json, args)?;
            }
            UserCommand::Diff(args) => {
                let _ = user::diff_users_with_request(&mut request_json, args)?;
            }
            UserCommand::Delete(args) => {
                let _ = user::delete_user_with_request(&mut request_json, args)?;
            }
        },
        AccessCommand::Org { command } => match command {
            OrgCommand::List(args) => {
                if args.input_dir.is_some() {
                    let _ = org::list_orgs_from_input_dir(args)?;
                } else {
                    let _ = org::list_orgs_with_request(&mut request_json, args)?;
                }
            }
            OrgCommand::Add(args) => {
                let _ = org::add_org_with_request(&mut request_json, args)?;
            }
            OrgCommand::Modify(args) => {
                let _ = org::modify_org_with_request(&mut request_json, args)?;
            }
            OrgCommand::Export(args) => {
                let _ = org::export_orgs_with_request(&mut request_json, args)?;
            }
            OrgCommand::Import(args) => {
                let _ = org::import_orgs_with_request(&mut request_json, args)?;
            }
            OrgCommand::Diff(args) => {
                let _ = org::diff_orgs_with_request(&mut request_json, args)?;
            }
            OrgCommand::Delete(args) => {
                let _ = org::delete_org_with_request(&mut request_json, args)?;
            }
        },
        AccessCommand::Team { command } => match command {
            TeamCommand::List(args) => {
                if args.input_dir.is_some() {
                    let _ = team::list_teams_from_input_dir(args)?;
                } else {
                    let _ = team::list_teams_command_with_request(&mut request_json, args)?;
                }
            }
            TeamCommand::Browse(args) => {
                #[cfg(feature = "tui")]
                {
                    let mut session = browse_terminal::TerminalSession::enter()?;
                    let mut next = browse_support::BrowseSwitch::ToTeam(args.clone());
                    loop {
                        next = match next {
                            browse_support::BrowseSwitch::Exit => break,
                            browse_support::BrowseSwitch::ToUser(inner) => {
                                user_browse::browse_users_in_session(
                                    &mut session,
                                    &mut request_json,
                                    &inner,
                                )?
                            }
                            browse_support::BrowseSwitch::ToTeam(inner) => {
                                team_browse::browse_teams_in_session(
                                    &mut session,
                                    &mut request_json,
                                    &inner,
                                )?
                            }
                        };
                    }
                }
                #[cfg(not(feature = "tui"))]
                {
                    let _ = team_browse::browse_teams_with_request(&mut request_json, args)?;
                }
            }
            TeamCommand::Add(args) => {
                let _ = team::add_team_with_request(&mut request_json, args)?;
            }
            TeamCommand::Modify(args) => {
                let _ = team::modify_team_with_request(&mut request_json, args)?;
            }
            TeamCommand::Export(args) => {
                let _ = team::export_teams_with_request(&mut request_json, args)?;
            }
            TeamCommand::Import(args) => {
                let _ = team::import_teams_with_request(&mut request_json, args)?;
            }
            TeamCommand::Diff(args) => {
                let _ = team::diff_teams_with_request(&mut request_json, args)?;
            }
            TeamCommand::Delete(args) => {
                let _ = pending_delete::delete_team_with_request(&mut request_json, args)?;
            }
        },
        AccessCommand::ServiceAccount { command } => match command {
            ServiceAccountCommand::List(args) => {
                if args.input_dir.is_some() {
                    let _ = service_account::list_service_accounts_from_input_dir(args)?;
                } else {
                    let _ = service_account::list_service_accounts_command_with_request(
                        &mut request_json,
                        args,
                    )?;
                }
            }
            ServiceAccountCommand::Add(args) => {
                let _ = service_account::add_service_account_with_request(&mut request_json, args)?;
            }
            ServiceAccountCommand::Export(args) => {
                let _ =
                    service_account::export_service_accounts_with_request(&mut request_json, args)?;
            }
            ServiceAccountCommand::Import(args) => {
                let _ =
                    service_account::import_service_accounts_with_request(&mut request_json, args)?;
            }
            ServiceAccountCommand::Diff(args) => {
                let _ =
                    service_account::diff_service_accounts_with_request(&mut request_json, args)?;
            }
            ServiceAccountCommand::Delete(args) => {
                let _ =
                    pending_delete::delete_service_account_with_request(&mut request_json, args)?;
            }
            ServiceAccountCommand::Token { command } => match command {
                ServiceAccountTokenCommand::Add(args) => {
                    let _ = service_account::add_service_account_token_with_request(
                        &mut request_json,
                        args,
                    )?;
                }
                ServiceAccountTokenCommand::Delete(args) => {
                    let _ = pending_delete::delete_service_account_token_with_request(
                        &mut request_json,
                        args,
                    )?;
                }
            },
        },
        AccessCommand::Plan(args) => {
            let args = resolve_access_plan_args(args)?;
            let _ = access_plan::access_plan_with_request(&mut request_json, &args)?;
        }
    }
    Ok(())
}
