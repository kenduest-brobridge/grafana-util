//! Inspection report model and aggregation surface.
//! Defines summary/row schemas and grouped/report helpers used by both CLI renderers and tests.
use serde::Serialize;

#[path = "inspect_report_model/columns.rs"]
mod columns;

#[allow(unused_imports)]
pub(crate) use columns::{
    render_query_report_column, report_column_header, report_format_supports_columns,
    resolve_report_column_ids, resolve_report_column_ids_for_format, DEFAULT_REPORT_COLUMN_IDS,
    SUPPORTED_REPORT_COLUMN_IDS,
};

/// Struct definition for QueryReportSummary.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub(crate) struct QueryReportSummary {
    #[serde(rename = "dashboardCount")]
    pub(crate) dashboard_count: usize,
    #[serde(rename = "panelCount")]
    pub(crate) panel_count: usize,
    #[serde(rename = "queryCount")]
    pub(crate) query_count: usize,
    #[serde(rename = "queryRecordCount")]
    pub(crate) report_row_count: usize,
}

/// Struct definition for ExportInspectionQueryRow.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub(crate) struct ExportInspectionQueryRow {
    pub(crate) org: String,
    #[serde(rename = "orgId")]
    pub(crate) org_id: String,
    #[serde(rename = "dashboardUid")]
    pub(crate) dashboard_uid: String,
    #[serde(rename = "dashboardTitle")]
    pub(crate) dashboard_title: String,
    #[serde(rename = "dashboardTags")]
    pub(crate) dashboard_tags: Vec<String>,
    #[serde(rename = "folderPath")]
    pub(crate) folder_path: String,
    #[serde(rename = "folderFullPath")]
    pub(crate) folder_full_path: String,
    #[serde(rename = "folderLevel")]
    pub(crate) folder_level: String,
    #[serde(rename = "folderUid")]
    pub(crate) folder_uid: String,
    #[serde(rename = "parentFolderUid")]
    pub(crate) parent_folder_uid: String,
    #[serde(rename = "panelId")]
    pub(crate) panel_id: String,
    #[serde(rename = "panelTitle")]
    pub(crate) panel_title: String,
    #[serde(rename = "panelType")]
    pub(crate) panel_type: String,
    #[serde(rename = "panelTargetCount")]
    pub(crate) panel_target_count: usize,
    #[serde(rename = "panelQueryCount")]
    pub(crate) panel_query_count: usize,
    #[serde(rename = "panelDatasourceCount")]
    pub(crate) panel_datasource_count: usize,
    #[serde(rename = "panelVariables")]
    pub(crate) panel_variables: Vec<String>,
    #[serde(rename = "refId")]
    pub(crate) ref_id: String,
    pub(crate) datasource: String,
    #[serde(rename = "datasourceName")]
    pub(crate) datasource_name: String,
    #[serde(rename = "datasourceUid")]
    pub(crate) datasource_uid: String,
    #[serde(rename = "datasourceOrg")]
    pub(crate) datasource_org: String,
    #[serde(rename = "datasourceOrgId")]
    pub(crate) datasource_org_id: String,
    #[serde(rename = "datasourceDatabase")]
    pub(crate) datasource_database: String,
    #[serde(rename = "datasourceBucket")]
    pub(crate) datasource_bucket: String,
    #[serde(rename = "datasourceOrganization")]
    pub(crate) datasource_organization: String,
    #[serde(rename = "datasourceIndexPattern")]
    pub(crate) datasource_index_pattern: String,
    #[serde(rename = "datasourceType")]
    pub(crate) datasource_type: String,
    #[serde(rename = "datasourceFamily")]
    pub(crate) datasource_family: String,
    #[serde(rename = "queryField")]
    pub(crate) query_field: String,
    #[serde(rename = "targetHidden")]
    pub(crate) target_hidden: String,
    #[serde(rename = "targetDisabled")]
    pub(crate) target_disabled: String,
    #[serde(rename = "query")]
    pub(crate) query_text: String,
    #[serde(rename = "queryVariables")]
    pub(crate) query_variables: Vec<String>,
    pub(crate) metrics: Vec<String>,
    pub(crate) functions: Vec<String>,
    pub(crate) measurements: Vec<String>,
    pub(crate) buckets: Vec<String>,
    #[serde(rename = "file")]
    pub(crate) file_path: String,
}

/// Struct definition for ExportInspectionQueryReport.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct ExportInspectionQueryReport {
    pub(crate) input_dir: String,
    pub(crate) summary: QueryReportSummary,
    pub(crate) queries: Vec<ExportInspectionQueryRow>,
}

/// Struct definition for ExportInspectionQueryReportJsonSummary.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct ExportInspectionQueryReportJsonSummary {
    #[serde(rename = "dashboardCount")]
    pub(crate) dashboard_count: usize,
    #[serde(rename = "queryRecordCount")]
    pub(crate) query_record_count: usize,
}

/// Struct definition for ExportInspectionQueryReportDocument.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct ExportInspectionQueryReportDocument {
    pub(crate) summary: ExportInspectionQueryReportJsonSummary,
    pub(crate) queries: Vec<ExportInspectionQueryRow>,
}

/// Struct definition for GroupedQueryPanel.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GroupedQueryPanel {
    pub(crate) panel_id: String,
    pub(crate) panel_title: String,
    pub(crate) panel_type: String,
    pub(crate) panel_target_count: usize,
    pub(crate) panel_query_count: usize,
    pub(crate) datasources: Vec<String>,
    pub(crate) datasource_families: Vec<String>,
    pub(crate) query_fields: Vec<String>,
    pub(crate) queries: Vec<ExportInspectionQueryRow>,
}

/// Struct definition for GroupedQueryDashboard.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GroupedQueryDashboard {
    pub(crate) org: String,
    pub(crate) org_id: String,
    pub(crate) dashboard_uid: String,
    pub(crate) dashboard_title: String,
    pub(crate) folder_path: String,
    pub(crate) folder_uid: String,
    pub(crate) parent_folder_uid: String,
    pub(crate) file_path: String,
    pub(crate) datasources: Vec<String>,
    pub(crate) datasource_families: Vec<String>,
    pub(crate) panels: Vec<GroupedQueryPanel>,
}

/// Struct definition for NormalizedQueryReport.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NormalizedQueryReport {
    pub(crate) input_dir: String,
    pub(crate) summary: QueryReportSummary,
    pub(crate) dashboards: Vec<GroupedQueryDashboard>,
}

/// Purpose: implementation note.
pub(crate) fn build_query_report(
    input_dir: String,
    dashboard_count: usize,
    panel_count: usize,
    query_count: usize,
    queries: Vec<ExportInspectionQueryRow>,
) -> ExportInspectionQueryReport {
    ExportInspectionQueryReport {
        input_dir,
        summary: QueryReportSummary {
            dashboard_count,
            panel_count,
            query_count,
            report_row_count: queries.len(),
        },
        queries,
    }
}

/// Purpose: implementation note.
pub(crate) fn build_export_inspection_query_report_document(
    report: &ExportInspectionQueryReport,
) -> ExportInspectionQueryReportDocument {
    ExportInspectionQueryReportDocument {
        summary: ExportInspectionQueryReportJsonSummary {
            dashboard_count: report.summary.dashboard_count,
            query_record_count: report.queries.len(),
        },
        queries: report.queries.clone(),
    }
}

/// refresh filtered query report summary.
pub(crate) fn refresh_filtered_query_report_summary(report: &mut ExportInspectionQueryReport) {
    report.summary.dashboard_count = report
        .queries
        .iter()
        .map(|row| row.dashboard_uid.clone())
        .collect::<std::collections::BTreeSet<String>>()
        .len();
    report.summary.panel_count = report
        .queries
        .iter()
        .map(|row| {
            (
                row.dashboard_uid.clone(),
                row.panel_id.clone(),
                row.panel_title.clone(),
            )
        })
        .collect::<std::collections::BTreeSet<(String, String, String)>>()
        .len();
    report.summary.query_count = report.queries.len();
    report.summary.report_row_count = report.queries.len();
}

// Group query rows by dashboard/panel so report output is deterministic and renderable.
/// Purpose: implementation note.
pub(crate) fn normalize_query_report(
    report: &ExportInspectionQueryReport,
) -> NormalizedQueryReport {
    let mut dashboards = Vec::new();
    for row in &report.queries {
        let dashboard_index = dashboards
            .iter()
            .position(|item: &GroupedQueryDashboard| item.dashboard_uid == row.dashboard_uid)
            .unwrap_or_else(|| {
                dashboards.push(GroupedQueryDashboard {
                    org: row.org.clone(),
                    org_id: row.org_id.clone(),
                    dashboard_uid: row.dashboard_uid.clone(),
                    dashboard_title: row.dashboard_title.clone(),
                    folder_path: row.folder_path.clone(),
                    folder_uid: row.folder_uid.clone(),
                    parent_folder_uid: row.parent_folder_uid.clone(),
                    file_path: row.file_path.clone(),
                    datasources: Vec::new(),
                    datasource_families: Vec::new(),
                    panels: Vec::new(),
                });
                dashboards.len() - 1
            });
        if !row.file_path.is_empty() && dashboards[dashboard_index].file_path.is_empty() {
            dashboards[dashboard_index].file_path = row.file_path.clone();
        }
        if dashboards[dashboard_index].org.is_empty() {
            dashboards[dashboard_index].org = row.org.clone();
        }
        if dashboards[dashboard_index].org_id.is_empty() {
            dashboards[dashboard_index].org_id = row.org_id.clone();
        }
        if dashboards[dashboard_index].folder_uid.is_empty() {
            dashboards[dashboard_index].folder_uid = row.folder_uid.clone();
        }
        if dashboards[dashboard_index].parent_folder_uid.is_empty() {
            dashboards[dashboard_index].parent_folder_uid = row.parent_folder_uid.clone();
        }
        if !row.datasource.is_empty()
            && !dashboards[dashboard_index]
                .datasources
                .iter()
                .any(|value| value == &row.datasource)
        {
            dashboards[dashboard_index]
                .datasources
                .push(row.datasource.clone());
        }
        if !row.datasource_family.is_empty()
            && !dashboards[dashboard_index]
                .datasource_families
                .iter()
                .any(|value| value == &row.datasource_family)
        {
            dashboards[dashboard_index]
                .datasource_families
                .push(row.datasource_family.clone());
        }
        let panels = &mut dashboards[dashboard_index].panels;
        let panel_index = panels
            .iter()
            .position(|item| {
                item.panel_id == row.panel_id
                    && item.panel_title == row.panel_title
                    && item.panel_type == row.panel_type
            })
            .unwrap_or_else(|| {
                panels.push(GroupedQueryPanel {
                    panel_id: row.panel_id.clone(),
                    panel_title: row.panel_title.clone(),
                    panel_type: row.panel_type.clone(),
                    panel_target_count: row.panel_target_count,
                    panel_query_count: row.panel_query_count,
                    datasources: Vec::new(),
                    datasource_families: Vec::new(),
                    query_fields: Vec::new(),
                    queries: Vec::new(),
                });
                panels.len() - 1
            });
        panels[panel_index].panel_target_count = panels[panel_index]
            .panel_target_count
            .max(row.panel_target_count);
        panels[panel_index].panel_query_count = panels[panel_index]
            .panel_query_count
            .max(row.panel_query_count);
        if !row.datasource.is_empty()
            && !panels[panel_index]
                .datasources
                .iter()
                .any(|value| value == &row.datasource)
        {
            panels[panel_index].datasources.push(row.datasource.clone());
        }
        if !row.datasource_family.is_empty()
            && !panels[panel_index]
                .datasource_families
                .iter()
                .any(|value| value == &row.datasource_family)
        {
            panels[panel_index]
                .datasource_families
                .push(row.datasource_family.clone());
        }
        if !row.query_field.is_empty()
            && !panels[panel_index]
                .query_fields
                .iter()
                .any(|value| value == &row.query_field)
        {
            panels[panel_index]
                .query_fields
                .push(row.query_field.clone());
        }
        panels[panel_index].queries.push(row.clone());
    }
    NormalizedQueryReport {
        input_dir: report.input_dir.clone(),
        summary: report.summary.clone(),
        dashboards,
    }
}
