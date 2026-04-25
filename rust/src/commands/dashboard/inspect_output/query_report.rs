use crate::common::{render_json_value, Result};
use crate::dashboard::cli_defs::{InspectExportArgs, InspectExportReportFormat};
use crate::dashboard::inspect_render::{
    render_csv, render_grouped_query_report, render_grouped_query_table_report, render_simple_table,
};
use crate::dashboard::inspect_report::{
    build_export_inspection_query_report_document, render_query_report_column,
    report_column_header, resolve_report_column_ids_for_format, ExportInspectionQueryReport,
};

use super::{render_lines_to_string, ExportInspectionRenderedOutput};

pub(crate) fn render_export_inspection_column_report_output(
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
            report.input_dir
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

pub(crate) fn render_export_inspection_tree_output(
    report: &ExportInspectionQueryReport,
) -> ExportInspectionRenderedOutput {
    ExportInspectionRenderedOutput {
        output: render_lines_to_string(render_grouped_query_report(report)),
        dashboard_count: report.summary.dashboard_count,
    }
}

pub(crate) fn render_export_inspection_queries_json_output(
    report: &ExportInspectionQueryReport,
) -> Result<ExportInspectionRenderedOutput> {
    let document = build_export_inspection_query_report_document(report);
    Ok(ExportInspectionRenderedOutput {
        output: format!("{}\n", render_json_value(&document)?),
        dashboard_count: report.summary.dashboard_count,
    })
}
