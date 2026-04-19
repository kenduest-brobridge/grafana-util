//! Datasource runtime artifact workspace resolution.

use std::path::PathBuf;

use crate::artifact_workspace::{
    lane_root_path, profile_scope_name, profile_scope_path, record_latest_run, resolve_run_id,
    run_root_path, ArtifactLane, RunSelector,
};
use crate::common::Result;
use crate::profile_config::{
    load_profile_config_file, resolve_artifact_root_path, resolve_profile_config_path,
};

use crate::datasource::datasource_cli_defs::ArtifactRunMode;

fn datasource_run_selector(
    run: Option<ArtifactRunMode>,
    run_id: Option<&str>,
    default_run: RunSelector,
) -> RunSelector {
    if let Some(run_id) = run_id {
        return RunSelector::RunId(run_id.to_string());
    }
    match run {
        Some(ArtifactRunMode::Timestamp) => RunSelector::Timestamp,
        Some(ArtifactRunMode::Latest) => RunSelector::Latest,
        None => default_run,
    }
}

fn datasource_artifact_lane_path(
    profile: Option<&str>,
    selector: RunSelector,
    lane: ArtifactLane,
    update_latest: bool,
) -> Result<PathBuf> {
    let config_path = resolve_profile_config_path();
    let config = if config_path.exists() {
        Some(load_profile_config_file(&config_path)?)
    } else {
        None
    };
    let artifact_root = resolve_artifact_root_path(config.as_ref(), &config_path);
    let scope_root = profile_scope_path(&artifact_root, profile);
    let run_id = resolve_run_id(&scope_root, &selector)?;
    if update_latest {
        record_latest_run(&scope_root, profile_scope_name(profile), &run_id)?;
    }
    Ok(lane_root_path(&run_root_path(&scope_root, &run_id), lane))
}

pub(super) fn resolve_datasource_artifact_input_dir(
    profile: Option<&str>,
    run: Option<ArtifactRunMode>,
    run_id: Option<&str>,
) -> Result<PathBuf> {
    datasource_artifact_lane_path(
        profile,
        datasource_run_selector(run, run_id, RunSelector::Latest),
        ArtifactLane::Datasources,
        false,
    )
}

pub(super) fn resolve_datasource_export_output_dir(
    args: &super::DatasourceExportArgs,
) -> Result<PathBuf> {
    if let Some(output_dir) = &args.output_dir {
        return Ok(output_dir.clone());
    }
    if args.run.is_none() && args.run_id.is_none() {
        return Ok(PathBuf::from("datasources"));
    }
    datasource_artifact_lane_path(
        args.common.profile.as_deref(),
        datasource_run_selector(args.run, args.run_id.as_deref(), RunSelector::Timestamp),
        ArtifactLane::Datasources,
        !args.dry_run,
    )
}
