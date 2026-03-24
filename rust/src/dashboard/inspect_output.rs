use std::path::Path;

use crate::common::Result;
use crate::dashboard_inspection_dependency_contract::build_offline_dependency_contract_from_report_rows;

use crate::dashboard::cli_defs::{InspectExportArgs, InspectExportReportFormat};
use crate::dashboard::files::{load_datasource_inventory, load_export_metadata};
use crate::dashboard::inspect_governance::{
    build_export_inspection_governance_document, render_governance_table_report,
    ExportInspectionGovernanceDocument,
};
use crate::dashboard::inspect_render::{
    render_csv, render_grouped_query_report, render_grouped_query_table_report, render_simple_table,
};
use crate::dashboard::inspect_report::{
    build_export_inspection_query_report_document, render_query_report_column, report_column_header,
    resolve_report_column_ids_for_format, ExportInspectionQueryReport,
};
use crate::dashboard::inspect_summary::ExportInspectionSummary;
use crate::dashboard::models::DatasourceInventoryItem;
use super::{build_export_inspection_summary, RAW_EXPORT_SUBDIR};

pub(super) struct ExportInspectionRenderedOutput {
    pub(super) output: String,
    pub(super) dashboard_count: usize,
}

fn render_lines_to_string(lines: Vec<String>) -> String {
    let mut output = String::new();
    for line in lines {
        output.push_str(&line);
        output.push('\n');
    }
    output
}

fn render_export_inspection_governance_output(
    summary: &ExportInspectionSummary,
    governance: &ExportInspectionGovernanceDocument,
    report_format: InspectExportReportFormat,
) -> Result<ExportInspectionRenderedOutput> {
    let output = if report_format == InspectExportReportFormat::GovernanceJson {
        format!("{}\n", serde_json::to_string_pretty(governance)?)
    } else {
        render_lines_to_string(render_governance_table_report(
            &summary.import_dir,
            governance,
        ))
    };
    Ok(ExportInspectionRenderedOutput {
        output,
        dashboard_count: summary.dashboard_count,
    })
}

fn render_export_inspection_dependency_output(
    report: &ExportInspectionQueryReport,
    datasource_inventory: &[DatasourceInventoryItem],
) -> Result<ExportInspectionRenderedOutput> {
    let payload =
        build_offline_dependency_contract_from_report_rows(&report.queries, datasource_inventory);
    Ok(ExportInspectionRenderedOutput {
        output: format!("{}\n", serde_json::to_string_pretty(&payload)?),
        dashboard_count: report.summary.dashboard_count,
    })
}

fn render_export_inspection_column_report_output(
    args: &InspectExportArgs,
    report: &ExportInspectionQueryReport,
    report_format: InspectExportReportFormat,
) -> Result<ExportInspectionRenderedOutput> {
    let column_ids =
        resolve_report_column_ids_for_format(Some(report_format), &args.report_columns)?;
    let output = if report_format == InspectExportReportFormat::TreeTable {
        render_lines_to_string(render_grouped_query_table_report(
            report,
            &column_ids,
            !args.no_header,
        ))
    } else if report_format == InspectExportReportFormat::Csv {
        let headers = column_ids
            .iter()
            .map(|column_id| report_column_header(column_id))
            .collect::<Vec<&str>>();
        let rows = report
            .queries
            .iter()
            .map(|item| {
                column_ids
                    .iter()
                    .map(|column_id| render_query_report_column(item, column_id))
                    .collect::<Vec<String>>()
            })
            .collect::<Vec<Vec<String>>>();
        render_lines_to_string(render_csv(&headers, &rows))
    } else {
        let rows = report
            .queries
            .iter()
            .map(|item| {
                column_ids
                    .iter()
                    .map(|column_id| render_query_report_column(item, column_id))
                    .collect::<Vec<String>>()
            })
            .collect::<Vec<Vec<String>>>();
        let headers = column_ids
            .iter()
            .map(|column_id| report_column_header(column_id))
            .collect::<Vec<&str>>();
        let mut output = String::new();
        output.push_str(&format!(
            "Export inspection report: {}\n\n",
            report.import_dir
        ));
        output.push_str("# Query report\n");
        for line in render_simple_table(&headers, &rows, !args.no_header) {
            output.push_str(&line);
            output.push('\n');
        }
        output
    };
    Ok(ExportInspectionRenderedOutput {
        output,
        dashboard_count: report.summary.dashboard_count,
    })
}

fn render_export_inspection_tree_output(
    report: &ExportInspectionQueryReport,
) -> ExportInspectionRenderedOutput {
    ExportInspectionRenderedOutput {
        output: render_lines_to_string(render_grouped_query_report(report)),
        dashboard_count: report.summary.dashboard_count,
    }
}

fn render_export_inspection_json_output(
    report: &ExportInspectionQueryReport,
) -> Result<ExportInspectionRenderedOutput> {
    let document = build_export_inspection_query_report_document(report);
    Ok(ExportInspectionRenderedOutput {
        output: format!("{}\n", serde_json::to_string_pretty(&document)?),
        dashboard_count: report.summary.dashboard_count,
    })
}

pub(super) fn render_export_inspection_report_output(
    args: &InspectExportArgs,
    import_dir: &Path,
    report_format: InspectExportReportFormat,
    report: &ExportInspectionQueryReport,
) -> Result<ExportInspectionRenderedOutput> {
    match report_format {
        InspectExportReportFormat::Governance | InspectExportReportFormat::GovernanceJson => {
            let summary = build_export_inspection_summary(import_dir)?;
            let governance = build_export_inspection_governance_document(&summary, report);
            render_export_inspection_governance_output(&summary, &governance, report_format)
        }
        InspectExportReportFormat::Json => render_export_inspection_json_output(report),
        InspectExportReportFormat::Dependency | InspectExportReportFormat::DependencyJson => {
            let metadata = load_export_metadata(import_dir, Some(RAW_EXPORT_SUBDIR))?;
            let datasource_inventory = load_datasource_inventory(import_dir, metadata.as_ref())?;
            render_export_inspection_dependency_output(report, &datasource_inventory)
        }
        InspectExportReportFormat::Tree => Ok(render_export_inspection_tree_output(report)),
        InspectExportReportFormat::TreeTable
        | InspectExportReportFormat::Csv
        | InspectExportReportFormat::Table => {
            render_export_inspection_column_report_output(args, report, report_format)
        }
    }
}
