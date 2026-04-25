//! Inspection path for Dashboard resources: analysis, extraction, and report shaping.

use std::path::Path;

use crate::common::{render_json_value, Result};
use crate::dashboard::cli_defs::{InspectExportArgs, InspectExportReportFormat};
use crate::dashboard::inspect_governance::{
    build_export_inspection_governance_document, render_governance_table_report,
    ExportInspectionGovernanceDocument,
};
use crate::dashboard::inspect_report::ExportInspectionQueryReport;
use crate::dashboard::inspect_summary::ExportInspectionSummary;

use super::super::build_export_inspection_summary_for_variant;
use super::{
    render_export_inspection_dependency_output, render_lines_to_string,
    ExportInspectionRenderedOutput,
};

#[path = "inspect_output/query_report.rs"]
mod query_report;

use query_report::{
    render_export_inspection_column_report_output, render_export_inspection_queries_json_output,
    render_export_inspection_tree_output,
};

fn render_export_inspection_governance_output(
    summary: &ExportInspectionSummary,
    governance: &ExportInspectionGovernanceDocument,
    report_format: InspectExportReportFormat,
) -> Result<ExportInspectionRenderedOutput> {
    let output = if report_format == InspectExportReportFormat::GovernanceJson {
        format!("{}\n", render_json_value(governance)?)
    } else {
        render_lines_to_string(render_governance_table_report(
            &summary.input_dir,
            governance,
        ))
    };
    Ok(ExportInspectionRenderedOutput {
        output,
        dashboard_count: summary.dashboard_count,
    })
}

pub(crate) fn render_export_inspection_report_output(
    args: &InspectExportArgs,
    input_dir: &Path,
    expected_variant: &str,
    report_format: InspectExportReportFormat,
    report: &ExportInspectionQueryReport,
) -> Result<ExportInspectionRenderedOutput> {
    match report_format {
        InspectExportReportFormat::Governance | InspectExportReportFormat::GovernanceJson => {
            let summary = build_export_inspection_summary_for_variant(input_dir, expected_variant)?;
            let governance = build_export_inspection_governance_document(&summary, report);
            render_export_inspection_governance_output(&summary, &governance, report_format)
        }
        InspectExportReportFormat::QueriesJson => {
            render_export_inspection_queries_json_output(report)
        }
        InspectExportReportFormat::Dependency | InspectExportReportFormat::DependencyJson => {
            render_export_inspection_dependency_output(
                input_dir,
                expected_variant,
                report_format,
                report,
            )
        }
        InspectExportReportFormat::Tree => Ok(render_export_inspection_tree_output(report)),
        InspectExportReportFormat::TreeTable
        | InspectExportReportFormat::Csv
        | InspectExportReportFormat::Table => {
            render_export_inspection_column_report_output(args, report, report_format)
        }
    }
}
