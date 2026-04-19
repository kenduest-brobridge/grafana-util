use std::fs;

use rpassword::prompt_password;

use crate::access::{self, AccessCliArgs};
use crate::common::{GrafanaCliError, Result};
use crate::dashboard::{
    self, CommonCliArgs, DashboardCliArgs, DashboardCommand, ExportArgs as DashboardExportArgs,
};
use crate::datasource::{DatasourceExportArgs, DatasourceGroupCommand};

use super::snapshot_access::run_snapshot_access_exports_with_handler;
use super::snapshot_artifacts::{
    build_snapshot_paths, prepare_snapshot_export_artifact_run, record_snapshot_latest_run,
};
use super::snapshot_metadata::{annotate_snapshot_root_scope_kinds, build_snapshot_root_metadata};

pub fn build_snapshot_dashboard_export_args(
    args: &super::super::SnapshotExportArgs,
) -> DashboardExportArgs {
    let paths = build_snapshot_paths(&args.output_dir);
    DashboardExportArgs {
        common: args.common.clone(),
        output_dir: paths.dashboards,
        page_size: dashboard::DEFAULT_PAGE_SIZE,
        org_id: None,
        all_orgs: true,
        flat: false,
        overwrite: args.overwrite,
        without_dashboard_raw: false,
        without_dashboard_prompt: false,
        without_dashboard_provisioning: false,
        include_history: false,
        provisioning_provider_name: "grafana-utils-dashboards".to_string(),
        provisioning_provider_org_id: None,
        provisioning_provider_path: None,
        provisioning_provider_disable_deletion: false,
        provisioning_provider_allow_ui_updates: false,
        provisioning_provider_update_interval_seconds: 30,
        dry_run: false,
        progress: false,
        verbose: false,
        run: None,
        run_id: None,
    }
}

pub fn build_snapshot_datasource_export_args(
    args: &super::super::SnapshotExportArgs,
) -> DatasourceExportArgs {
    let paths = build_snapshot_paths(&args.output_dir);
    DatasourceExportArgs {
        common: args.common.clone(),
        output_dir: Some(paths.datasources),
        run: None,
        run_id: None,
        org_id: None,
        all_orgs: true,
        overwrite: args.overwrite,
        without_datasource_provisioning: false,
        dry_run: false,
    }
}

pub(crate) fn materialize_snapshot_common_auth_with_prompt<F, G>(
    mut common: CommonCliArgs,
    mut prompt_password_reader: F,
    mut prompt_token_reader: G,
) -> Result<CommonCliArgs>
where
    F: FnMut() -> Result<String>,
    G: FnMut() -> Result<String>,
{
    if common.prompt_password && common.password.is_none() {
        common.password = Some(prompt_password_reader()?);
    }
    if common.prompt_token && common.api_token.is_none() {
        common.api_token = Some(prompt_token_reader()?);
    }
    common.prompt_password = false;
    common.prompt_token = false;
    Ok(common)
}

fn materialize_snapshot_common_auth(common: CommonCliArgs) -> Result<CommonCliArgs> {
    materialize_snapshot_common_auth_with_prompt(
        common,
        || prompt_password("Grafana Basic auth password: ").map_err(GrafanaCliError::from),
        || prompt_password("Grafana API token: ").map_err(GrafanaCliError::from),
    )
}

fn write_snapshot_root_metadata_file(args: &super::super::SnapshotExportArgs) -> Result<()> {
    let metadata_path = build_snapshot_paths(&args.output_dir).metadata;
    let metadata = build_snapshot_root_metadata(&args.output_dir, &args.common)?;
    fs::write(metadata_path, serde_json::to_string_pretty(&metadata)?)?;
    Ok(())
}

// Coordinate snapshot export by delegating each selected lane to its domain handler.
pub(crate) fn run_snapshot_export_selected_with_handlers<FD, FS, FA>(
    args: super::super::SnapshotExportArgs,
    selection: &super::super::SnapshotExportSelection,
    mut run_dashboard: FD,
    mut run_datasource: FS,
    mut run_access: FA,
) -> Result<()>
where
    FD: FnMut(DashboardCliArgs) -> Result<()>,
    FS: FnMut(DatasourceGroupCommand) -> Result<()>,
    FA: FnMut(AccessCliArgs) -> Result<()>,
{
    fs::create_dir_all(&args.output_dir)?;
    if selection.contains(super::super::SnapshotExportLane::Dashboards) {
        run_dashboard(DashboardCliArgs {
            color: args.common.color,
            command: DashboardCommand::Export(build_snapshot_dashboard_export_args(&args)),
        })?;
    }
    if selection.contains(super::super::SnapshotExportLane::Datasources) {
        run_datasource(DatasourceGroupCommand::Export(
            build_snapshot_datasource_export_args(&args),
        ))?;
    }
    run_snapshot_access_exports_with_handler(&args, selection, &mut run_access)?;
    annotate_snapshot_root_scope_kinds(&args.output_dir)?;
    write_snapshot_root_metadata_file(&args)?;
    Ok(())
}

#[cfg(test)]
pub(crate) fn run_snapshot_export_with_handlers<FD, FS, FA>(
    args: super::super::SnapshotExportArgs,
    mut run_dashboard: FD,
    mut run_datasource: FS,
    mut run_access: FA,
) -> Result<()>
where
    FD: FnMut(DashboardCliArgs) -> Result<()>,
    FS: FnMut(DatasourceGroupCommand) -> Result<()>,
    FA: FnMut(AccessCliArgs) -> Result<()>,
{
    run_snapshot_export_selected_with_handlers(
        args,
        &super::super::SnapshotExportSelection::all(),
        &mut run_dashboard,
        &mut run_datasource,
        &mut run_access,
    )
}

pub fn run_snapshot_export(args: super::super::SnapshotExportArgs) -> Result<()> {
    // Snapshot export is cross-domain orchestration:
    // each lane (dashboard/datasource/access) shares auth materialization and a single selection contract.
    let mut args = args;
    args.common = materialize_snapshot_common_auth(args.common)?;
    let artifact_run = prepare_snapshot_export_artifact_run(&mut args)?;
    let selection = if args.prompt {
        match super::super::prompt_snapshot_export_selection()? {
            Some(selection) => selection,
            None => return Ok(()),
        }
    } else {
        super::super::SnapshotExportSelection::all()
    };
    run_snapshot_export_selected_with_handlers(
        args,
        &selection,
        dashboard::run_dashboard_cli,
        crate::datasource::run_datasource_cli,
        access::run_access_cli,
    )?;
    if let Some(resolved) = artifact_run {
        record_snapshot_latest_run(&resolved)?;
    }
    Ok(())
}
