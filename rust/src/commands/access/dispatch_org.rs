use reqwest::Method;
use serde_json::Value;

use crate::artifact_workspace::ArtifactLane;
use crate::common::Result;

use super::super::cli_defs::build_http_client_no_org_id;
use super::super::org;
use super::super::{AccessCliArgs, AccessCommand, OrgCommand};
use super::dispatch_artifacts::{
    export_access_lane_path, local_access_lane_path, resolve_required_access_diff_dir,
    resolve_required_access_input_dir, run_access_cli_with_common,
};

pub(super) fn run_org_access_cli(command: &OrgCommand, args: &AccessCliArgs) -> Result<()> {
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
            let mut inner = inner.clone();
            resolve_required_access_input_dir(
                &mut inner.input_dir,
                inner.common.profile.as_deref(),
                inner.run,
                inner.run_id.as_deref(),
                inner.local,
                ArtifactLane::AccessOrgs,
                "access org import",
            )?;
            let resolved_args = AccessCliArgs {
                command: AccessCommand::Org {
                    command: OrgCommand::Import(inner),
                },
            };
            run_access_cli_with_common(
                match &resolved_args.command {
                    AccessCommand::Org {
                        command: OrgCommand::Import(inner),
                    } => &inner.common,
                    _ => unreachable!(),
                },
                &resolved_args,
                build_http_client_no_org_id,
            )
        }
        OrgCommand::Diff(inner) => {
            let mut inner = inner.clone();
            resolve_required_access_diff_dir(
                &mut inner.diff_dir,
                inner.common.profile.as_deref(),
                inner.run,
                inner.run_id.as_deref(),
                inner.local,
                ArtifactLane::AccessOrgs,
                "access org diff",
            )?;
            let resolved_args = AccessCliArgs {
                command: AccessCommand::Org {
                    command: OrgCommand::Diff(inner),
                },
            };
            run_access_cli_with_common(
                match &resolved_args.command {
                    AccessCommand::Org {
                        command: OrgCommand::Diff(inner),
                    } => &inner.common,
                    _ => unreachable!(),
                },
                &resolved_args,
                build_http_client_no_org_id,
            )
        }
        OrgCommand::Delete(inner) => {
            run_access_cli_with_common(&inner.common, args, build_http_client_no_org_id)
        }
    }
}

pub(super) fn run_org_access_cli_with_request<F>(
    command: &OrgCommand,
    request_json: &mut F,
    _args: &AccessCliArgs,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    match command {
        OrgCommand::List(args) => {
            if args.input_dir.is_some() {
                let _ = org::list_orgs_from_input_dir(args)?;
            } else {
                let _ = org::list_orgs_with_request(request_json, args)?;
            }
        }
        OrgCommand::Add(args) => {
            let _ = org::add_org_with_request(request_json, args)?;
        }
        OrgCommand::Modify(args) => {
            let _ = org::modify_org_with_request(request_json, args)?;
        }
        OrgCommand::Export(args) => {
            let _ = org::export_orgs_with_request(request_json, args)?;
        }
        OrgCommand::Import(args) => {
            let _ = org::import_orgs_with_request(request_json, args)?;
        }
        OrgCommand::Diff(args) => {
            let _ = org::diff_orgs_with_request(request_json, args)?;
        }
        OrgCommand::Delete(args) => {
            let _ = org::delete_org_with_request(request_json, args)?;
        }
    }
    Ok(())
}
