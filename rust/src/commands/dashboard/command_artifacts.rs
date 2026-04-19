use crate::common::{message, Result};
use std::path::{Path, PathBuf};

use super::{DashboardCliArgs, DashboardCommand};

#[derive(Debug, Clone)]
pub(super) struct ArtifactRunResolution {
    pub(super) scope_root: PathBuf,
    pub(super) profile: String,
    pub(super) run_id: String,
    pub(super) run_root: PathBuf,
}

fn artifact_selector_from_flags(
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
            other => Err(message(format!(
                "Unsupported artifact run selector '{other}'. Use latest or timestamp."
            ))),
        };
    }
    if default_latest {
        return Ok(Some(crate::artifact_workspace::RunSelector::Latest));
    }
    Ok(None)
}

fn artifact_scope_root_for_profile(profile: Option<&str>) -> Result<(PathBuf, String)> {
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

fn resolve_dashboard_artifact_run(
    profile: Option<&str>,
    selector: &crate::artifact_workspace::RunSelector,
) -> Result<ArtifactRunResolution> {
    let (scope_root, profile_name) = artifact_scope_root_for_profile(profile)?;
    let run_id = crate::artifact_workspace::resolve_run_id(&scope_root, selector)?;
    let run_root = crate::artifact_workspace::run_root_path(&scope_root, &run_id);
    Ok(ArtifactRunResolution {
        scope_root,
        profile: profile_name,
        run_id,
        run_root,
    })
}

fn dashboard_artifact_lane_path(run_root: &Path) -> PathBuf {
    crate::artifact_workspace::lane_root_path(
        run_root,
        crate::artifact_workspace::ArtifactLane::Dashboards,
    )
}

pub(super) fn prepare_dashboard_export_artifact_run(
    args: &mut super::ExportArgs,
) -> Result<Option<ArtifactRunResolution>> {
    let Some(selector) =
        artifact_selector_from_flags(args.run.as_deref(), args.run_id.as_deref(), false)?
    else {
        return Ok(None);
    };
    let resolved = resolve_dashboard_artifact_run(args.common.profile.as_deref(), &selector)?;
    if args.output_dir.as_path() == Path::new(super::DEFAULT_EXPORT_DIR) {
        args.output_dir = dashboard_artifact_lane_path(&resolved.run_root);
    }
    Ok(Some(resolved))
}

pub(super) fn record_dashboard_latest_run(resolved: &ArtifactRunResolution) -> Result<()> {
    crate::artifact_workspace::record_latest_run(
        &resolved.scope_root,
        &resolved.profile,
        &resolved.run_id,
    )?;
    Ok(())
}

fn resolve_dashboard_local_artifact_input(
    profile: Option<&str>,
    run: Option<&str>,
    run_id: Option<&str>,
    local: bool,
) -> Result<Option<PathBuf>> {
    let Some(selector) = artifact_selector_from_flags(run, run_id, local)? else {
        return Ok(None);
    };
    let resolved = resolve_dashboard_artifact_run(profile, &selector)?;
    Ok(Some(dashboard_artifact_lane_path(&resolved.run_root)))
}

fn is_empty_path(path: &Path) -> bool {
    path.as_os_str().is_empty()
}

fn materialize_dashboard_required_artifact_input(
    profile: Option<&str>,
    input_dir: &mut PathBuf,
    run: Option<&str>,
    run_id: Option<&str>,
    local: bool,
    command_name: &str,
) -> Result<()> {
    if is_empty_path(input_dir) {
        if let Some(resolved) = resolve_dashboard_local_artifact_input(profile, run, run_id, local)?
        {
            *input_dir = resolved;
            return Ok(());
        }
        return Err(message(format!(
            "{command_name} requires --input-dir or artifact workspace selection with --local, --run, or --run-id."
        )));
    }
    Ok(())
}

pub(super) fn materialize_dashboard_artifact_args(args: &mut DashboardCliArgs) -> Result<()> {
    match &mut args.command {
        DashboardCommand::Browse(inner)
            if inner.input_dir.is_none() && inner.workspace.is_none() =>
        {
            if let Some(input_dir) = resolve_dashboard_local_artifact_input(
                inner.common.profile.as_deref(),
                inner.run.as_deref(),
                inner.run_id.as_deref(),
                inner.local,
            )? {
                inner.input_dir = Some(input_dir);
            }
        }
        DashboardCommand::Summary(inner) if inner.input_dir.is_none() => {
            if let Some(input_dir) = resolve_dashboard_local_artifact_input(
                inner.common.profile.as_deref(),
                inner.run.as_deref(),
                inner.run_id.as_deref(),
                inner.local,
            )? {
                inner.input_dir = Some(input_dir);
            }
        }
        DashboardCommand::Import(inner) => {
            materialize_dashboard_required_artifact_input(
                inner.common.profile.as_deref(),
                &mut inner.input_dir,
                inner.run.as_deref(),
                inner.run_id.as_deref(),
                inner.local,
                "dashboard import",
            )?;
        }
        DashboardCommand::Diff(inner) => {
            materialize_dashboard_required_artifact_input(
                inner.common.profile.as_deref(),
                &mut inner.input_dir,
                inner.run.as_deref(),
                inner.run_id.as_deref(),
                inner.local,
                "dashboard diff",
            )?;
        }
        _ => {}
    }
    Ok(())
}
