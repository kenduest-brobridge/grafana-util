use std::path::{Path, PathBuf};

use crate::artifact_workspace::{ArtifactLane, RunSelector};
use crate::common::{message, Result};
use crate::http::JsonHttpClient;

use super::super::artifact_workspace::{
    access_lane_path, access_run_root, access_run_selector, lane_for_plan_resource,
};
use super::super::cli_defs::AccessArtifactRunMode;
use super::super::{AccessCliArgs, AccessPlanArgs};

fn is_empty_path(path: &Path) -> bool {
    path.as_os_str().is_empty()
}

pub(super) fn local_access_lane_path(
    profile: Option<&str>,
    run: Option<AccessArtifactRunMode>,
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

pub(super) fn export_access_lane_path(
    profile: Option<&str>,
    run: Option<AccessArtifactRunMode>,
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

pub(super) fn resolve_required_access_input_dir(
    input_dir: &mut PathBuf,
    profile: Option<&str>,
    run: Option<AccessArtifactRunMode>,
    run_id: Option<&str>,
    local: bool,
    lane: ArtifactLane,
    command_name: &str,
) -> Result<()> {
    if is_empty_path(input_dir) {
        if local || run.is_some() || run_id.is_some() {
            *input_dir = local_access_lane_path(profile, run, run_id, lane)?;
            return Ok(());
        }
        return Err(message(format!(
            "{command_name} requires --input-dir or artifact workspace selection with --local, --run, or --run-id."
        )));
    }
    Ok(())
}

pub(super) fn resolve_required_access_diff_dir(
    diff_dir: &mut PathBuf,
    profile: Option<&str>,
    run: Option<AccessArtifactRunMode>,
    run_id: Option<&str>,
    local: bool,
    lane: ArtifactLane,
    command_name: &str,
) -> Result<()> {
    if local || run.is_some() || run_id.is_some() {
        *diff_dir = local_access_lane_path(profile, run, run_id, lane)?;
        return Ok(());
    }
    if is_empty_path(diff_dir) {
        return Err(message(format!(
            "{command_name} requires --diff-dir or artifact workspace selection with --local, --run, or --run-id."
        )));
    }
    Ok(())
}

pub(super) fn resolve_access_plan_args(args: &AccessPlanArgs) -> Result<AccessPlanArgs> {
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

pub(super) fn run_access_cli_with_common<C, F>(
    common: &C,
    args: &AccessCliArgs,
    build_client: F,
) -> Result<()>
where
    F: FnOnce(&C) -> Result<JsonHttpClient>,
{
    let client = build_client(common)?;
    super::super::run_access_cli_with_client(&client, args)
}
