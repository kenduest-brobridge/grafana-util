use std::path::{Path, PathBuf};

use crate::common::Result;

#[derive(Debug, Clone)]
pub(super) struct SnapshotArtifactRunResolution {
    pub(super) scope_root: PathBuf,
    pub(super) profile: String,
    pub(super) run_id: String,
    pub(super) run_root: PathBuf,
}

fn snapshot_artifact_selector_from_flags(
    run: Option<&str>,
    run_id: Option<&str>,
    default_latest: bool,
) -> Result<Option<crate::artifact_workspace::RunSelector>> {
    if let Some(run_id) = run_id {
        return Ok(Some(crate::artifact_workspace::RunSelector::RunId(
            run_id.to_string(),
        )));
    }
    if let Some(run) = run {
        return match run {
            "latest" => Ok(Some(crate::artifact_workspace::RunSelector::Latest)),
            "timestamp" => Ok(Some(crate::artifact_workspace::RunSelector::Timestamp)),
            other => Err(crate::common::message(format!(
                "Unsupported artifact run selector '{other}'. Use latest or timestamp."
            ))),
        };
    }
    if default_latest {
        return Ok(Some(crate::artifact_workspace::RunSelector::Latest));
    }
    Ok(None)
}

fn snapshot_artifact_scope_root_for_profile(profile: Option<&str>) -> Result<(PathBuf, String)> {
    let config_path = crate::profile_config::resolve_profile_config_path();
    let config = if config_path.is_file() {
        Some(crate::profile_config::load_profile_config_file(
            &config_path,
        )?)
    } else {
        None
    };
    let artifact_root =
        crate::profile_config::resolve_artifact_root_path(config.as_ref(), &config_path);
    let profile_name = crate::artifact_workspace::profile_scope_name(profile).to_string();
    Ok((
        crate::artifact_workspace::profile_scope_path(&artifact_root, profile),
        profile_name,
    ))
}

fn resolve_snapshot_artifact_run(
    profile: Option<&str>,
    selector: &crate::artifact_workspace::RunSelector,
) -> Result<SnapshotArtifactRunResolution> {
    let (scope_root, profile_name) = snapshot_artifact_scope_root_for_profile(profile)?;
    let run_id = crate::artifact_workspace::resolve_run_id(&scope_root, selector)?;
    let run_root = crate::artifact_workspace::run_root_path(&scope_root, &run_id);
    Ok(SnapshotArtifactRunResolution {
        scope_root,
        profile: profile_name,
        run_id,
        run_root,
    })
}

pub(super) fn prepare_snapshot_export_artifact_run(
    args: &mut super::super::SnapshotExportArgs,
) -> Result<Option<SnapshotArtifactRunResolution>> {
    let Some(selector) =
        snapshot_artifact_selector_from_flags(args.run.as_deref(), args.run_id.as_deref(), false)?
    else {
        return Ok(None);
    };
    let resolved = resolve_snapshot_artifact_run(args.common.profile.as_deref(), &selector)?;
    if args.output_dir.as_path() == Path::new("snapshot") {
        args.output_dir = resolved.run_root.clone();
    }
    Ok(Some(resolved))
}

pub(super) fn record_snapshot_latest_run(resolved: &SnapshotArtifactRunResolution) -> Result<()> {
    crate::artifact_workspace::record_latest_run(
        &resolved.scope_root,
        &resolved.profile,
        &resolved.run_id,
    )?;
    Ok(())
}

pub(super) fn resolve_snapshot_review_input_dir(
    args: &super::super::SnapshotReviewArgs,
) -> Result<PathBuf> {
    let selector = snapshot_artifact_selector_from_flags(
        args.run.as_deref(),
        args.run_id.as_deref(),
        args.local,
    )?;
    if let Some(selector) = selector {
        let resolved = resolve_snapshot_artifact_run(None, &selector)?;
        if args.input_dir.as_path() == Path::new("snapshot") {
            return Ok(resolved.run_root);
        }
    }
    Ok(args.input_dir.clone())
}

pub fn build_snapshot_paths(output_dir: &Path) -> super::super::SnapshotPaths {
    let access = output_dir.join(super::super::SNAPSHOT_ACCESS_DIR);
    super::super::SnapshotPaths {
        dashboards: output_dir.join(super::super::SNAPSHOT_DASHBOARD_DIR),
        datasources: output_dir.join(super::super::SNAPSHOT_DATASOURCE_DIR),
        access_users: access.join(super::super::SNAPSHOT_ACCESS_USERS_DIR),
        access_teams: access.join(super::super::SNAPSHOT_ACCESS_TEAMS_DIR),
        access_orgs: access.join(super::super::SNAPSHOT_ACCESS_ORGS_DIR),
        access_service_accounts: access.join(super::super::SNAPSHOT_ACCESS_SERVICE_ACCOUNTS_DIR),
        access,
        metadata: output_dir.join(super::super::SNAPSHOT_METADATA_FILENAME),
    }
}
