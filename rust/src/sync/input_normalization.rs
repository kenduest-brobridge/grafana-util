//! Normalization helpers for task-first `grafana-util change` workflows.
//!
//! This module is intentionally small and boring: it owns the input assembly
//! and normalization steps that convert repo-local staged inputs into the
//! canonical shapes consumed by inspect/check/preview/bundle workflows.

use std::path::PathBuf;

use super::{
    build_alert_sync_specs, load_alerting_bundle_section, load_dashboard_bundle_sections,
    load_dashboard_provisioning_bundle_sections, load_datasource_provisioning_records,
    load_json_array_file, load_json_value, normalize_datasource_bundle_item, ChangeCheckArgs,
    ChangeInspectArgs, ChangeStagedInputsArgs, Result,
};
use crate::common::message;
use crate::dashboard::{
    resolve_inspect_export_import_dir, DashboardImportInputFormat, InspectExportInputType,
    TempInspectDir,
};
use crate::overview::{OverviewArgs, OverviewOutputFormat};
use crate::project_status_command::{ProjectStatusOutputFormat, ProjectStatusStagedArgs};
use serde_json::{Map, Value};

#[derive(Default)]
pub(crate) struct NormalizedChangeDashboardInputs {
    pub(crate) _temp_dir: Option<TempInspectDir>,
    pub(crate) dashboard_export_dir: Option<PathBuf>,
    pub(crate) dashboard_provisioning_dir: Option<PathBuf>,
}

pub(crate) fn build_overview_args(
    args: &ChangeInspectArgs,
    discovered_dashboard_export_dir: Option<&PathBuf>,
    discovered_dashboard_provisioning_dir: Option<&PathBuf>,
    discovered_datasource_provisioning_file: Option<&PathBuf>,
    discovered_desired_file: Option<&PathBuf>,
    discovered_source_bundle: Option<&PathBuf>,
    discovered_target_inventory: Option<&PathBuf>,
    discovered_alert_export_dir: Option<&PathBuf>,
    discovered_availability_file: Option<&PathBuf>,
    discovered_mapping_file: Option<&PathBuf>,
) -> OverviewArgs {
    OverviewArgs {
        dashboard_export_dir: args
            .inputs
            .dashboard_export_dir
            .clone()
            .or(discovered_dashboard_export_dir.cloned()),
        dashboard_provisioning_dir: args
            .inputs
            .dashboard_provisioning_dir
            .clone()
            .or(discovered_dashboard_provisioning_dir.cloned()),
        datasource_export_dir: None,
        datasource_provisioning_file: args
            .inputs
            .datasource_provisioning_file
            .clone()
            .or(discovered_datasource_provisioning_file.cloned()),
        access_user_export_dir: None,
        access_team_export_dir: None,
        access_org_export_dir: None,
        access_service_account_export_dir: None,
        desired_file: args
            .inputs
            .desired_file
            .clone()
            .or(discovered_desired_file.cloned()),
        source_bundle: args
            .inputs
            .source_bundle
            .clone()
            .or(discovered_source_bundle.cloned()),
        target_inventory: discovered_target_inventory.cloned(),
        alert_export_dir: args
            .inputs
            .alert_export_dir
            .clone()
            .or(discovered_alert_export_dir.cloned()),
        availability_file: discovered_availability_file.cloned(),
        mapping_file: discovered_mapping_file.cloned(),
        output_format: match args.output.output_format {
            super::SyncOutputFormat::Text => OverviewOutputFormat::Text,
            super::SyncOutputFormat::Json => OverviewOutputFormat::Json,
        },
    }
}

pub(crate) fn normalize_change_dashboard_inputs(
    dashboard_export_dir: Option<&PathBuf>,
    dashboard_provisioning_dir: Option<&PathBuf>,
) -> Result<NormalizedChangeDashboardInputs> {
    let mut temp_dir = None;
    let mut normalized_export_dir = None;
    let mut normalized_provisioning_dir = None;

    if let Some(path) = dashboard_export_dir {
        let holder = TempInspectDir::new("change-dashboard-export-input")?;
        let resolved = resolve_inspect_export_import_dir(
            &holder.path,
            path,
            DashboardImportInputFormat::Raw,
            Some(InspectExportInputType::Raw),
            false,
        )?;
        normalized_export_dir = Some(resolved.input_dir);
        temp_dir = Some(holder);
    } else if let Some(path) = dashboard_provisioning_dir {
        let holder = TempInspectDir::new("change-dashboard-provisioning-input")?;
        let resolved = resolve_inspect_export_import_dir(
            &holder.path,
            path,
            DashboardImportInputFormat::Provisioning,
            None,
            false,
        )?;
        normalized_provisioning_dir = Some(resolved.input_dir);
        temp_dir = Some(holder);
    }

    Ok(NormalizedChangeDashboardInputs {
        _temp_dir: temp_dir,
        dashboard_export_dir: normalized_export_dir,
        dashboard_provisioning_dir: normalized_provisioning_dir,
    })
}

pub(crate) fn build_status_args(
    args: &ChangeCheckArgs,
    discovered_dashboard_export_dir: Option<&PathBuf>,
    discovered_dashboard_provisioning_dir: Option<&PathBuf>,
    discovered_datasource_provisioning_file: Option<&PathBuf>,
    discovered_desired_file: Option<&PathBuf>,
    discovered_source_bundle: Option<&PathBuf>,
    discovered_target_inventory: Option<&PathBuf>,
    discovered_alert_export_dir: Option<&PathBuf>,
    discovered_availability_file: Option<&PathBuf>,
    discovered_mapping_file: Option<&PathBuf>,
) -> ProjectStatusStagedArgs {
    ProjectStatusStagedArgs {
        dashboard_export_dir: args
            .inputs
            .dashboard_export_dir
            .clone()
            .or(discovered_dashboard_export_dir.cloned()),
        dashboard_provisioning_dir: args
            .inputs
            .dashboard_provisioning_dir
            .clone()
            .or(discovered_dashboard_provisioning_dir.cloned()),
        datasource_export_dir: None,
        datasource_provisioning_file: args
            .inputs
            .datasource_provisioning_file
            .clone()
            .or(discovered_datasource_provisioning_file.cloned()),
        access_user_export_dir: None,
        access_team_export_dir: None,
        access_org_export_dir: None,
        access_service_account_export_dir: None,
        desired_file: args
            .inputs
            .desired_file
            .clone()
            .or(discovered_desired_file.cloned()),
        source_bundle: args
            .inputs
            .source_bundle
            .clone()
            .or(discovered_source_bundle.cloned()),
        target_inventory: args
            .target_inventory
            .clone()
            .or(discovered_target_inventory.cloned()),
        alert_export_dir: args
            .inputs
            .alert_export_dir
            .clone()
            .or(discovered_alert_export_dir.cloned()),
        availability_file: args
            .availability_file
            .clone()
            .or(discovered_availability_file.cloned()),
        mapping_file: args
            .mapping_file
            .clone()
            .or(discovered_mapping_file.cloned()),
        output_format: match args.output.output_format {
            super::SyncOutputFormat::Text => ProjectStatusOutputFormat::Text,
            super::SyncOutputFormat::Json => ProjectStatusOutputFormat::Json,
        },
    }
}

pub(crate) fn select_preview_dashboard_sources<'a>(
    inputs: &'a ChangeStagedInputsArgs,
    dashboard_export_dir: Option<&'a PathBuf>,
    dashboard_provisioning_dir: Option<&'a PathBuf>,
) -> Result<(Option<&'a PathBuf>, Option<&'a PathBuf>)> {
    if inputs.dashboard_export_dir.is_some() && inputs.dashboard_provisioning_dir.is_some() {
        return Err(message(
            "Change preview accepts only one dashboard source: --dashboard-export-dir or --dashboard-provisioning-dir.",
        ));
    }
    if let Some(path) = inputs.dashboard_export_dir.as_ref() {
        return Ok((Some(path), None));
    }
    if let Some(path) = inputs.dashboard_provisioning_dir.as_ref() {
        return Ok((None, Some(path)));
    }
    if let Some(path) = dashboard_export_dir {
        return Ok((Some(path), None));
    }
    Ok((None, dashboard_provisioning_dir))
}

pub(crate) fn build_change_bundle_specs(
    inputs: &ChangeStagedInputsArgs,
    dashboard_export_dir: Option<&PathBuf>,
    dashboard_provisioning_dir: Option<&PathBuf>,
    discovered_alert_export_dir: Option<&PathBuf>,
    discovered_datasource_provisioning_file: Option<&PathBuf>,
) -> Result<Option<Vec<Value>>> {
    let (dashboard_export_dir, dashboard_provisioning_dir) =
        select_preview_dashboard_sources(inputs, dashboard_export_dir, dashboard_provisioning_dir)?;
    let alert_export_dir = inputs
        .alert_export_dir
        .as_ref()
        .or(discovered_alert_export_dir);
    let datasource_provisioning_file = inputs
        .datasource_provisioning_file
        .as_ref()
        .or(discovered_datasource_provisioning_file);

    if dashboard_export_dir.is_none()
        && dashboard_provisioning_dir.is_none()
        && alert_export_dir.is_none()
        && datasource_provisioning_file.is_none()
    {
        return Ok(None);
    }

    let mut dashboards = Vec::new();
    let mut datasources = Vec::new();
    let mut folders = Vec::new();
    if let Some(output_dir) = dashboard_export_dir {
        let (dashboard_items, dashboard_datasources, folder_items, _dashboard_metadata) =
            load_dashboard_bundle_sections(
                output_dir,
                output_dir,
                datasource_provisioning_file.map(PathBuf::as_path),
            )?;
        dashboards = dashboard_items;
        datasources.extend(dashboard_datasources);
        folders = folder_items;
    } else if let Some(provisioning_dir) = dashboard_provisioning_dir {
        let (dashboard_items, dashboard_datasources, folder_items, _dashboard_metadata) =
            load_dashboard_provisioning_bundle_sections(
                provisioning_dir,
                datasource_provisioning_file.map(PathBuf::as_path),
            )?;
        dashboards = dashboard_items;
        datasources.extend(dashboard_datasources);
        folders = folder_items;
    }
    if let Some(path) = datasource_provisioning_file {
        datasources = load_datasource_provisioning_records(path)?
            .into_iter()
            .map(|item| normalize_datasource_bundle_item(&item))
            .collect::<Result<Vec<_>>>()?;
    }
    let alerting = match alert_export_dir {
        Some(path) => load_alerting_bundle_section(path)?,
        None => Value::Object(Map::new()),
    };
    let alerts = build_alert_sync_specs(&alerting)?;

    let mut desired_specs = Vec::new();
    desired_specs.extend(dashboards);
    desired_specs.extend(datasources);
    desired_specs.extend(folders);
    desired_specs.extend(alerts);
    Ok(Some(desired_specs))
}

pub(crate) fn load_preview_desired_specs(
    inputs: &ChangeStagedInputsArgs,
    discovered_desired_file: Option<&PathBuf>,
    discovered_source_bundle: Option<&PathBuf>,
    dashboard_export_dir: Option<&PathBuf>,
    dashboard_provisioning_dir: Option<&PathBuf>,
    discovered_alert_export_dir: Option<&PathBuf>,
    discovered_datasource_provisioning_file: Option<&PathBuf>,
) -> Result<Vec<Value>> {
    if let Some(path) = inputs.desired_file.as_ref().or(discovered_desired_file) {
        return load_json_array_file(path, "Change desired input");
    }
    if let Some(path) = inputs.source_bundle.as_ref().or(discovered_source_bundle) {
        let source_bundle = load_json_value(path, "Change source bundle input")?;
        let bundle = source_bundle
            .as_object()
            .ok_or_else(|| message("Change source bundle input must be a JSON object."))?;
        let mut desired_specs = Vec::new();
        for key in ["dashboards", "datasources", "folders", "alerts"] {
            if let Some(items) = bundle.get(key).and_then(Value::as_array) {
                desired_specs.extend(items.iter().cloned());
            }
        }
        return Ok(desired_specs);
    }
    if let Some(specs) = build_change_bundle_specs(
        inputs,
        dashboard_export_dir,
        dashboard_provisioning_dir,
        discovered_alert_export_dir,
        discovered_datasource_provisioning_file,
    )? {
        return Ok(specs);
    }
    Err(message(
        "Change preview could not find a staged desired change file, source bundle, or staged export/provisioning inputs.",
    ))
}

#[cfg(test)]
mod input_normalization_tests {
    use super::*;

    fn staged_inputs(
        dashboard_export_dir: Option<&str>,
        dashboard_provisioning_dir: Option<&str>,
    ) -> ChangeStagedInputsArgs {
        ChangeStagedInputsArgs {
            workspace: PathBuf::from("."),
            desired_file: None,
            source_bundle: None,
            dashboard_export_dir: dashboard_export_dir.map(PathBuf::from),
            dashboard_provisioning_dir: dashboard_provisioning_dir.map(PathBuf::from),
            alert_export_dir: None,
            datasource_export_file: None,
            datasource_provisioning_file: None,
        }
    }

    #[test]
    fn select_preview_dashboard_sources_prefers_explicit_export_input() {
        let inputs = staged_inputs(Some("./dashboards/raw"), None);
        let discovered_provisioning = PathBuf::from("./dashboards/provisioning");
        let (output_dir, provisioning_dir) =
            select_preview_dashboard_sources(&inputs, None, Some(&discovered_provisioning))
                .unwrap();

        let expected_output_dir = PathBuf::from("./dashboards/raw");
        assert_eq!(output_dir, Some(&expected_output_dir));
        assert!(provisioning_dir.is_none());
    }

    #[test]
    fn select_preview_dashboard_sources_prefers_explicit_provisioning_input() {
        let inputs = staged_inputs(None, Some("./dashboards/provisioning"));
        let discovered_export = PathBuf::from("./dashboards/raw");
        let (output_dir, provisioning_dir) =
            select_preview_dashboard_sources(&inputs, Some(&discovered_export), None).unwrap();

        assert!(output_dir.is_none());
        let expected_provisioning_dir = PathBuf::from("./dashboards/provisioning");
        assert_eq!(provisioning_dir, Some(&expected_provisioning_dir));
    }

    #[test]
    fn select_preview_dashboard_sources_rejects_two_explicit_dashboard_inputs() {
        let inputs = staged_inputs(Some("./dashboards/raw"), Some("./dashboards/provisioning"));
        let error = select_preview_dashboard_sources(&inputs, None, None).unwrap_err();

        assert_eq!(
            error.to_string(),
            "Change preview accepts only one dashboard source: --dashboard-export-dir or --dashboard-provisioning-dir."
        );
    }
}
