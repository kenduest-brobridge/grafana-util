//! Summary aggregates for dashboard inspection reports.
//! Provides compact DTOs for folder/datasource/dashboard-level coverage metrics.
use serde::Serialize;

#[path = "inspect_summary/projection.rs"]
mod projection;

#[allow(unused_imports)]
pub(crate) use projection::{
    build_export_inspection_summary_document, build_export_inspection_summary_rows,
    ExportInspectionSummaryDocument, ExportInspectionSummaryJsonSummary,
};

/// Struct definition for ExportFolderUsage.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct ExportFolderUsage {
    pub(crate) path: String,
    pub(crate) dashboards: usize,
}

/// Struct definition for ExportDatasourceUsage.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct ExportDatasourceUsage {
    pub(crate) datasource: String,
    pub(crate) reference_count: usize,
    pub(crate) dashboard_count: usize,
}

/// Struct definition for DatasourceInventorySummary.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct DatasourceInventorySummary {
    pub(crate) uid: String,
    pub(crate) name: String,
    #[serde(rename = "type")]
    pub(crate) datasource_type: String,
    pub(crate) access: String,
    pub(crate) url: String,
    #[serde(rename = "isDefault")]
    pub(crate) is_default: String,
    pub(crate) org: String,
    #[serde(rename = "orgId")]
    pub(crate) org_id: String,
    #[serde(rename = "referenceCount")]
    pub(crate) reference_count: usize,
    #[serde(rename = "dashboardCount")]
    pub(crate) dashboard_count: usize,
}

/// Struct definition for MixedDashboardSummary.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct MixedDashboardSummary {
    pub(crate) uid: String,
    pub(crate) title: String,
    pub(crate) folder_path: String,
    pub(crate) datasource_count: usize,
    pub(crate) datasources: Vec<String>,
}

/// Struct definition for ExportInspectionSummary.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct ExportInspectionSummary {
    pub(crate) input_dir: String,
    pub(crate) export_org: Option<String>,
    pub(crate) export_org_id: Option<String>,
    pub(crate) dashboard_count: usize,
    pub(crate) folder_count: usize,
    pub(crate) panel_count: usize,
    pub(crate) query_count: usize,
    pub(crate) datasource_inventory_count: usize,
    pub(crate) orphaned_datasource_count: usize,
    pub(crate) mixed_dashboard_count: usize,
    pub(crate) folder_paths: Vec<ExportFolderUsage>,
    pub(crate) datasource_usage: Vec<ExportDatasourceUsage>,
    pub(crate) datasource_inventory: Vec<DatasourceInventorySummary>,
    pub(crate) orphaned_datasources: Vec<DatasourceInventorySummary>,
    pub(crate) mixed_dashboards: Vec<MixedDashboardSummary>,
}
