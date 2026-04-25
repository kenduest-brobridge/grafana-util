//! Query report column contract and rendering helpers.

use crate::common::{message, Result};
use crate::dashboard::cli_defs::InspectExportReportFormat;

use super::ExportInspectionQueryRow;

/// Constant for default report column ids.
pub(crate) const DEFAULT_REPORT_COLUMN_IDS: &[&str] = &[
    "org",
    "org_id",
    "dashboard_uid",
    "dashboard_title",
    "dashboard_tags",
    "folder_path",
    "folder_full_path",
    "folder_level",
    "folder_uid",
    "parent_folder_uid",
    "panel_id",
    "panel_title",
    "panel_type",
    "panel_query_count",
    "panel_datasource_count",
    "panel_variables",
    "ref_id",
    "datasource",
    "datasource_name",
    "datasource_org",
    "datasource_org_id",
    "datasource_database",
    "datasource_bucket",
    "datasource_organization",
    "datasource_index_pattern",
    "datasource_type",
    "datasource_family",
    "query_field",
    "query_variables",
    "metrics",
    "functions",
    "measurements",
    "buckets",
    "query",
    "file",
];

/// Constant for supported report column ids.
pub(crate) const SUPPORTED_REPORT_COLUMN_IDS: &[&str] = &[
    "org",
    "org_id",
    "dashboard_uid",
    "dashboard_title",
    "dashboard_tags",
    "folder_path",
    "folder_full_path",
    "folder_level",
    "folder_uid",
    "parent_folder_uid",
    "panel_id",
    "panel_title",
    "panel_type",
    "panel_target_count",
    "panel_query_count",
    "panel_datasource_count",
    "panel_variables",
    "ref_id",
    "datasource",
    "datasource_name",
    "datasource_uid",
    "datasource_org",
    "datasource_org_id",
    "datasource_database",
    "datasource_bucket",
    "datasource_organization",
    "datasource_index_pattern",
    "datasource_type",
    "datasource_family",
    "query_field",
    "target_hidden",
    "target_disabled",
    "query_variables",
    "metrics",
    "functions",
    "measurements",
    "buckets",
    "query",
    "file",
];

fn normalize_report_column_id(value: &str) -> &str {
    match value {
        "orgId" => "org_id",
        "dashboardUid" => "dashboard_uid",
        "dashboardTitle" => "dashboard_title",
        "dashboardTags" => "dashboard_tags",
        "folderPath" => "folder_path",
        "folderFullPath" => "folder_full_path",
        "folderLevel" => "folder_level",
        "folderUid" => "folder_uid",
        "parentFolderUid" => "parent_folder_uid",
        "panelId" => "panel_id",
        "panelTitle" => "panel_title",
        "panelType" => "panel_type",
        "panelTargetCount" => "panel_target_count",
        "panelQueryCount" => "panel_query_count",
        "panelDatasourceCount" => "panel_datasource_count",
        "panelVariables" => "panel_variables",
        "refId" => "ref_id",
        "datasourceName" => "datasource_name",
        "datasourceUid" => "datasource_uid",
        "datasourceOrg" => "datasource_org",
        "datasourceOrgId" => "datasource_org_id",
        "datasourceDatabase" => "datasource_database",
        "datasourceBucket" => "datasource_bucket",
        "datasourceOrganization" => "datasource_organization",
        "datasourceIndexPattern" => "datasource_index_pattern",
        "datasourceType" => "datasource_type",
        "datasourceFamily" => "datasource_family",
        "queryField" => "query_field",
        "targetHidden" => "target_hidden",
        "targetDisabled" => "target_disabled",
        "queryVariables" => "query_variables",
        "functions" => "functions",
        _ => value,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
/// Purpose: implementation note.
pub(crate) fn resolve_report_column_ids(selected: &[String]) -> Result<Vec<String>> {
    resolve_report_column_ids_for_format(None, selected)
}

/// Purpose: implementation note.
pub(crate) fn resolve_report_column_ids_for_format(
    report_format: Option<InspectExportReportFormat>,
    selected: &[String],
) -> Result<Vec<String>> {
    if selected.is_empty() {
        let defaults = if matches!(report_format, Some(InspectExportReportFormat::Csv)) {
            SUPPORTED_REPORT_COLUMN_IDS
        } else {
            DEFAULT_REPORT_COLUMN_IDS
        };
        return Ok(defaults.iter().map(|value| value.to_string()).collect());
    }
    let mut result = Vec::new();
    for value in selected {
        let normalized = normalize_report_column_id(value.trim());
        if normalized.is_empty() {
            continue;
        }
        if normalized == "all" {
            return Ok(SUPPORTED_REPORT_COLUMN_IDS
                .iter()
                .map(|value| value.to_string())
                .collect());
        }
        if !SUPPORTED_REPORT_COLUMN_IDS.contains(&normalized) {
            return Err(message(format!(
                "Unsupported --report-columns value {:?}. Supported columns: {}",
                normalized,
                std::iter::once("all")
                    .chain(SUPPORTED_REPORT_COLUMN_IDS.iter().copied())
                    .collect::<Vec<&str>>()
                    .join(",")
            )));
        }
        if !result.iter().any(|item| item == normalized) {
            result.push(normalized.to_string());
        }
    }
    if result.is_empty() {
        return Err(message(format!(
            "--report-columns did not include any supported columns. Supported columns: {}",
            std::iter::once("all")
                .chain(SUPPORTED_REPORT_COLUMN_IDS.iter().copied())
                .collect::<Vec<&str>>()
                .join(",")
        )));
    }
    Ok(result)
}

/// report column header.
pub(crate) fn report_column_header(column_id: &str) -> &'static str {
    match column_id {
        "org" => "ORG",
        "org_id" => "ORG_ID",
        "dashboard_uid" => "DASHBOARD_UID",
        "dashboard_title" => "DASHBOARD_TITLE",
        "dashboard_tags" => "DASHBOARD_TAGS",
        "folder_path" => "FOLDER_PATH",
        "folder_full_path" => "FOLDER_FULL_PATH",
        "folder_level" => "FOLDER_LEVEL",
        "folder_uid" => "FOLDER_UID",
        "parent_folder_uid" => "PARENT_FOLDER_UID",
        "panel_id" => "PANEL_ID",
        "panel_title" => "PANEL_TITLE",
        "panel_type" => "PANEL_TYPE",
        "panel_target_count" => "PANEL_TARGET_COUNT",
        "panel_query_count" => "PANEL_EFFECTIVE_QUERY_COUNT",
        "panel_datasource_count" => "PANEL_TOTAL_DATASOURCE_COUNT",
        "panel_variables" => "PANEL_VARIABLES",
        "ref_id" => "REF_ID",
        "datasource" => "DATASOURCE",
        "datasource_name" => "DATASOURCE_NAME",
        "datasource_uid" => "DATASOURCE_UID",
        "datasource_org" => "DATASOURCE_ORG",
        "datasource_org_id" => "DATASOURCE_ORG_ID",
        "datasource_database" => "DATASOURCE_DATABASE",
        "datasource_bucket" => "DATASOURCE_BUCKET",
        "datasource_organization" => "DATASOURCE_ORGANIZATION",
        "datasource_index_pattern" => "DATASOURCE_INDEX_PATTERN",
        "datasource_type" => "DATASOURCE_TYPE",
        "datasource_family" => "DATASOURCE_FAMILY",
        "query_field" => "QUERY_FIELD",
        "target_hidden" => "TARGET_HIDDEN",
        "target_disabled" => "TARGET_DISABLED",
        "query_variables" => "QUERY_VARIABLES",
        "metrics" => "METRICS",
        "functions" => "FUNCTIONS",
        "measurements" => "MEASUREMENTS",
        "buckets" => "BUCKETS",
        "query" => "QUERY",
        "file" => "FILE",
        _ => unreachable!("unsupported report column header"),
    }
}

/// Purpose: implementation note.
pub(crate) fn render_query_report_column(
    row: &ExportInspectionQueryRow,
    column_id: &str,
) -> String {
    match column_id {
        "org" => row.org.clone(),
        "org_id" => row.org_id.clone(),
        "dashboard_uid" => row.dashboard_uid.clone(),
        "dashboard_title" => row.dashboard_title.clone(),
        "dashboard_tags" => row.dashboard_tags.join(","),
        "folder_path" => row.folder_path.clone(),
        "folder_full_path" => row.folder_full_path.clone(),
        "folder_level" => row.folder_level.clone(),
        "folder_uid" => row.folder_uid.clone(),
        "parent_folder_uid" => row.parent_folder_uid.clone(),
        "panel_id" => row.panel_id.clone(),
        "panel_title" => row.panel_title.clone(),
        "panel_type" => row.panel_type.clone(),
        "panel_target_count" => row.panel_target_count.to_string(),
        "panel_query_count" => row.panel_query_count.to_string(),
        "panel_datasource_count" => row.panel_datasource_count.to_string(),
        "panel_variables" => row.panel_variables.join(","),
        "ref_id" => row.ref_id.clone(),
        "datasource" => row.datasource.clone(),
        "datasource_name" => row.datasource_name.clone(),
        "datasource_uid" => row.datasource_uid.clone(),
        "datasource_org" => row.datasource_org.clone(),
        "datasource_org_id" => row.datasource_org_id.clone(),
        "datasource_database" => row.datasource_database.clone(),
        "datasource_bucket" => row.datasource_bucket.clone(),
        "datasource_organization" => row.datasource_organization.clone(),
        "datasource_index_pattern" => row.datasource_index_pattern.clone(),
        "datasource_type" => row.datasource_type.clone(),
        "datasource_family" => row.datasource_family.clone(),
        "query_field" => row.query_field.clone(),
        "target_hidden" => row.target_hidden.clone(),
        "target_disabled" => row.target_disabled.clone(),
        "query_variables" => row.query_variables.join(","),
        "metrics" => row.metrics.join(","),
        "functions" => row.functions.join(","),
        "measurements" => row.measurements.join(","),
        "buckets" => row.buckets.join(","),
        "query" => row.query_text.clone(),
        "file" => row.file_path.clone(),
        _ => unreachable!("unsupported report column value"),
    }
}

/// report format supports columns.
pub(crate) fn report_format_supports_columns(format: InspectExportReportFormat) -> bool {
    matches!(
        format,
        InspectExportReportFormat::Table
            | InspectExportReportFormat::Csv
            | InspectExportReportFormat::TreeTable
    )
}
