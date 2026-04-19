//! Access artifact workspace path resolution.

use std::path::PathBuf;

use crate::artifact_workspace::{
    lane_root_path, profile_scope_name, profile_scope_path, record_latest_run, resolve_run_id,
    run_root_path, ArtifactLane, RunSelector,
};
use crate::common::Result;
use crate::profile_config::{
    load_profile_config_file, resolve_artifact_root_path, resolve_profile_config_path,
};

use super::cli_defs::{AccessArtifactRunMode, AccessPlanResource};

pub(crate) fn access_run_selector(
    run: Option<AccessArtifactRunMode>,
    run_id: Option<&str>,
    default_run: RunSelector,
) -> RunSelector {
    if let Some(run_id) = run_id {
        return RunSelector::RunId(run_id.to_string());
    }
    match run {
        Some(AccessArtifactRunMode::Timestamp) => RunSelector::Timestamp,
        Some(AccessArtifactRunMode::Latest) => RunSelector::Latest,
        None => default_run,
    }
}

fn access_artifact_root(profile: Option<&str>) -> Result<(PathBuf, String)> {
    let config_path = resolve_profile_config_path();
    let config = if config_path.exists() {
        Some(load_profile_config_file(&config_path)?)
    } else {
        None
    };
    let artifact_root = resolve_artifact_root_path(config.as_ref(), &config_path);
    let profile_name = profile_scope_name(profile).to_string();
    Ok((profile_scope_path(&artifact_root, profile), profile_name))
}

pub(crate) fn access_lane_path(
    profile: Option<&str>,
    selector: RunSelector,
    lane: ArtifactLane,
    update_latest: bool,
) -> Result<PathBuf> {
    let (scope_root, profile_name) = access_artifact_root(profile)?;
    let run_id = resolve_run_id(&scope_root, &selector)?;
    if update_latest {
        record_latest_run(&scope_root, &profile_name, &run_id)?;
    }
    Ok(lane_root_path(&run_root_path(&scope_root, &run_id), lane))
}

pub(crate) fn access_run_root(
    profile: Option<&str>,
    selector: RunSelector,
    update_latest: bool,
) -> Result<PathBuf> {
    let (scope_root, profile_name) = access_artifact_root(profile)?;
    let run_id = resolve_run_id(&scope_root, &selector)?;
    if update_latest {
        record_latest_run(&scope_root, &profile_name, &run_id)?;
    }
    Ok(run_root_path(&scope_root, &run_id).join("access"))
}

pub(crate) fn lane_for_plan_resource(resource: &AccessPlanResource) -> Option<ArtifactLane> {
    match resource {
        AccessPlanResource::User => Some(ArtifactLane::AccessUsers),
        AccessPlanResource::Team => Some(ArtifactLane::AccessTeams),
        AccessPlanResource::Org => Some(ArtifactLane::AccessOrgs),
        AccessPlanResource::ServiceAccount => Some(ArtifactLane::AccessServiceAccounts),
        AccessPlanResource::All => None,
    }
}
