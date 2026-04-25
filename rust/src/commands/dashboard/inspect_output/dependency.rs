use std::path::Path;

use crate::common::{render_json_value, Result};
use crate::dashboard::cli_defs::InspectExportReportFormat;
use crate::dashboard::files::{load_datasource_inventory, load_export_metadata};
use crate::dashboard::inspect_dependency_render::render_export_inspection_dependency_table_report;
use crate::dashboard::inspect_report::ExportInspectionQueryReport;
use crate::dashboard_inspection_dependency_contract::{
    build_offline_dependency_contract_document_from_report_rows,
    build_offline_dependency_contract_from_report_rows,
};

use super::{render_lines_to_string, ExportInspectionRenderedOutput};

pub(crate) fn render_export_inspection_dependency_output(
    input_dir: &Path,
    expected_variant: &str,
    report_format: InspectExportReportFormat,
    report: &ExportInspectionQueryReport,
) -> Result<ExportInspectionRenderedOutput> {
    let metadata = load_export_metadata(input_dir, Some(expected_variant))?;
    let datasource_inventory = load_datasource_inventory(input_dir, metadata.as_ref())?;
    let output = if report_format == InspectExportReportFormat::DependencyJson {
        format!(
            "{}\n",
            render_json_value(&build_offline_dependency_contract_from_report_rows(
                &report.queries,
                &datasource_inventory,
            ))?
        )
    } else {
        let document = build_offline_dependency_contract_document_from_report_rows(
            &report.queries,
            &datasource_inventory,
        );
        render_lines_to_string(render_export_inspection_dependency_table_report(
            &report.input_dir,
            &document,
        ))
    };
    Ok(ExportInspectionRenderedOutput {
        output,
        dashboard_count: report.summary.dashboard_count,
    })
}

#[cfg(test)]
mod tests {
    use super::render_export_inspection_dependency_output;
    use crate::dashboard::cli_defs::InspectExportReportFormat;
    use crate::dashboard::files::build_export_metadata;
    use crate::dashboard::inspect_report::{ExportInspectionQueryReport, QueryReportSummary};
    use crate::dashboard::test_support::make_core_family_report_row;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    fn make_report(input_dir: &str) -> ExportInspectionQueryReport {
        ExportInspectionQueryReport {
            input_dir: input_dir.to_string(),
            summary: QueryReportSummary {
                dashboard_count: 1,
                panel_count: 1,
                query_count: 1,
                report_row_count: 1,
            },
            queries: vec![make_core_family_report_row(
                "cpu-main",
                "7",
                "A",
                "prom-main",
                "Prometheus Main",
                "prometheus",
                "prometheus",
                "sum(rate(up[5m]))",
                &["job=\"api\""],
            )],
        }
    }

    #[test]
    fn render_export_inspection_dependency_output_renders_dependency_text_and_json_distinctly() {
        let temp = tempdir().unwrap();
        let input_dir = temp.path();
        let metadata = build_export_metadata(
            "raw",
            1,
            None,
            None,
            Some("datasources.json"),
            None,
            None,
            None,
            None,
            "local",
            None,
            Some(input_dir),
            None,
            input_dir,
            &input_dir.join("export-metadata.json"),
        );
        fs::write(
            input_dir.join("export-metadata.json"),
            serde_json::to_string_pretty(&metadata).unwrap() + "\n",
        )
        .unwrap();
        fs::write(
            input_dir.join("datasources.json"),
            serde_json::to_string_pretty(&json!([
                {
                    "uid": "prom-main",
                    "name": "Prometheus Main",
                    "type": "prometheus",
                    "access": "proxy",
                    "url": "http://prometheus:9090",
                    "database": "",
                    "defaultBucket": "",
                    "organization": "",
                    "indexPattern": "",
                    "isDefault": "true",
                    "org": "Main Org.",
                    "orgId": "1"
                },
                {
                    "uid": "unused-main",
                    "name": "Unused Main",
                    "type": "postgres",
                    "access": "proxy",
                    "url": "postgresql://postgres:5432/unused",
                    "database": "metrics",
                    "defaultBucket": "",
                    "organization": "",
                    "indexPattern": "",
                    "isDefault": "false",
                    "org": "Main Org.",
                    "orgId": "1"
                }
            ]))
            .unwrap()
                + "\n",
        )
        .unwrap();

        let report = make_report(&input_dir.display().to_string());

        let dependency_output = render_export_inspection_dependency_output(
            input_dir,
            "raw",
            InspectExportReportFormat::Dependency,
            &report,
        )
        .unwrap();
        assert!(dependency_output
            .output
            .starts_with("Export inspection dependency: "));
        assert!(dependency_output.output.contains("# Datasource usage"));
        assert!(dependency_output
            .output
            .contains("# Dashboard dependencies"));
        assert!(dependency_output.output.contains("# Orphaned datasources"));
        assert!(dependency_output.output.contains("cpu-main"));
        assert!(dependency_output.output.contains("Prometheus Main"));
        assert!(dependency_output.output.contains("Unused Main"));
        assert!(!dependency_output.output.trim_start().starts_with('{'));

        let dependency_json_output = render_export_inspection_dependency_output(
            input_dir,
            "raw",
            InspectExportReportFormat::DependencyJson,
            &report,
        )
        .unwrap();
        assert!(dependency_json_output.output.trim_start().starts_with('{'));
        assert!(dependency_json_output
            .output
            .contains("\"datasourceUid\": \"prom-main\""));
        assert!(dependency_json_output
            .output
            .contains("\"dashboardDependencies\""));
        assert!(dependency_json_output
            .output
            .contains("\"orphanedDatasources\""));
    }
}
