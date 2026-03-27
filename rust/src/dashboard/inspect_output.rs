use std::path::Path;

use crate::common::Result;
use crate::dashboard_inspection_dependency_contract::{
    build_offline_dependency_contract_document_from_report_rows,
    build_offline_dependency_contract_from_report_rows, OfflineDependencyReportDocument,
};

use super::{build_export_inspection_summary, RAW_EXPORT_SUBDIR};
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
    build_export_inspection_query_report_document, render_query_report_column,
    report_column_header, resolve_report_column_ids_for_format, ExportInspectionQueryReport,
};
use crate::dashboard::inspect_summary::ExportInspectionSummary;

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

fn render_dependency_section(
    lines: &mut Vec<String>,
    title: &str,
    headers: &[&str],
    rows: &[Vec<String>],
) {
    lines.push(String::new());
    lines.push(title.to_string());
    if rows.is_empty() {
        lines.push("(none)".to_string());
        return;
    }
    lines.extend(render_simple_table(headers, rows, true));
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "(none)".to_string()
    } else {
        values.join(",")
    }
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

fn render_export_inspection_dependency_table_report(
    import_dir: &str,
    document: &OfflineDependencyReportDocument,
) -> Vec<String> {
    let mut lines = vec![
        format!("Export inspection dependency: {}", import_dir),
        String::new(),
    ];

    lines.push("# Summary".to_string());
    let summary_rows = vec![vec![
        document
            .summary
            .get("queryCount")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
            .to_string(),
        document
            .summary
            .get("dashboardCount")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
            .to_string(),
        document
            .summary
            .get("panelCount")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
            .to_string(),
        document
            .summary
            .get("datasourceCount")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
            .to_string(),
        document
            .summary
            .get("orphanedDatasourceCount")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
            .to_string(),
    ]];
    lines.extend(render_simple_table(
        &[
            "QUERY_COUNT",
            "DASHBOARD_COUNT",
            "PANEL_COUNT",
            "DATASOURCE_COUNT",
            "ORPHANED_DATASOURCE_COUNT",
        ],
        &summary_rows,
        true,
    ));

    let usage_rows = document
        .usage
        .iter()
        .map(|item| {
            vec![
                item.datasource_identity.clone(),
                item.datasource_uid.clone(),
                item.datasource_type.clone(),
                item.family.clone(),
                item.query_count.to_string(),
                item.dashboard_count.to_string(),
                item.panel_count.to_string(),
                item.reference_count.to_string(),
                join_or_none(&item.query_fields),
            ]
        })
        .collect::<Vec<Vec<String>>>();
    render_dependency_section(
        &mut lines,
        "# Datasource usage",
        &[
            "DATASOURCE",
            "UID",
            "TYPE",
            "FAMILY",
            "QUERIES",
            "DASHBOARDS",
            "PANELS",
            "REFS",
            "QUERY_FIELDS",
        ],
        &usage_rows,
    );

    let dashboard_rows = document
        .dashboard_dependencies
        .iter()
        .map(|item| {
            vec![
                item.dashboard_uid.clone(),
                item.dashboard_title.clone(),
                item.query_count.to_string(),
                item.panel_count.to_string(),
                item.datasource_count.to_string(),
                item.datasource_family_count.to_string(),
                join_or_none(&item.query_fields),
                join_or_none(&item.metrics),
                join_or_none(&item.functions),
                join_or_none(&item.measurements),
                join_or_none(&item.buckets),
            ]
        })
        .collect::<Vec<Vec<String>>>();
    render_dependency_section(
        &mut lines,
        "# Dashboard dependencies",
        &[
            "DASHBOARD_UID",
            "TITLE",
            "QUERIES",
            "PANELS",
            "DATASOURCES",
            "FAMILIES",
            "QUERY_FIELDS",
            "METRICS",
            "FUNCTIONS",
            "MEASUREMENTS",
            "BUCKETS",
        ],
        &dashboard_rows,
    );

    let orphan_rows = document
        .orphaned
        .iter()
        .map(|item| {
            vec![
                item.org.clone(),
                item.org_id.clone(),
                item.uid.clone(),
                item.name.clone(),
                item.datasource_type.clone(),
                item.family.clone(),
                item.access.clone(),
                item.is_default.clone(),
                item.url.clone(),
                item.database.clone(),
                item.default_bucket.clone(),
                item.organization.clone(),
                item.index_pattern.clone(),
            ]
        })
        .collect::<Vec<Vec<String>>>();
    render_dependency_section(
        &mut lines,
        "# Orphaned datasources",
        &[
            "ORG",
            "ORG_ID",
            "UID",
            "NAME",
            "TYPE",
            "FAMILY",
            "ACCESS",
            "IS_DEFAULT",
            "URL",
            "DATABASE",
            "DEFAULT_BUCKET",
            "ORGANIZATION",
            "INDEX_PATTERN",
        ],
        &orphan_rows,
    );

    lines
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
            if report_format == InspectExportReportFormat::DependencyJson {
                Ok(ExportInspectionRenderedOutput {
                    output: format!(
                        "{}\n",
                        serde_json::to_string_pretty(
                            &build_offline_dependency_contract_from_report_rows(
                                &report.queries,
                                &datasource_inventory,
                            )
                        )?
                    ),
                    dashboard_count: report.summary.dashboard_count,
                })
            } else {
                let document = build_offline_dependency_contract_document_from_report_rows(
                    &report.queries,
                    &datasource_inventory,
                );
                Ok(ExportInspectionRenderedOutput {
                    output: render_lines_to_string(
                        render_export_inspection_dependency_table_report(
                            &report.import_dir,
                            &document,
                        ),
                    ),
                    dashboard_count: report.summary.dashboard_count,
                })
            }
        }
        InspectExportReportFormat::Tree => Ok(render_export_inspection_tree_output(report)),
        InspectExportReportFormat::TreeTable
        | InspectExportReportFormat::Csv
        | InspectExportReportFormat::Table => {
            render_export_inspection_column_report_output(args, report, report_format)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dashboard::files::build_export_metadata;
    use crate::dashboard::inspect_report::{ExportInspectionQueryReport, QueryReportSummary};
    use crate::dashboard::test_support::make_core_family_report_row;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    fn make_report(import_dir: &str) -> ExportInspectionQueryReport {
        ExportInspectionQueryReport {
            import_dir: import_dir.to_string(),
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
    fn render_export_inspection_report_output_renders_dependency_text_and_json_distinctly() {
        let temp = tempdir().unwrap();
        let import_dir = temp.path();
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
        );
        fs::write(
            import_dir.join("export-metadata.json"),
            serde_json::to_string_pretty(&metadata).unwrap() + "\n",
        )
        .unwrap();
        fs::write(
            import_dir.join("datasources.json"),
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

        let args = crate::dashboard::cli_defs::InspectExportArgs {
            import_dir: import_dir.to_path_buf(),
            json: false,
            table: false,
            report: Some(InspectExportReportFormat::Dependency),
            output_format: None,
            report_columns: Vec::new(),
            report_filter_datasource: None,
            report_filter_panel_id: None,
            help_full: false,
            no_header: false,
            output_file: None,
            interactive: false,
        };
        let report = make_report(&import_dir.display().to_string());

        let dependency_output = render_export_inspection_report_output(
            &args,
            import_dir,
            InspectExportReportFormat::Dependency,
            &report,
        )
        .unwrap();
        assert!(dependency_output
            .output
            .starts_with("Export inspection dependency: "));
        assert!(dependency_output.output.contains("# Datasource usage"));
        assert!(dependency_output.output.contains("# Orphaned datasources"));
        assert!(dependency_output.output.contains("Prometheus Main"));
        assert!(dependency_output.output.contains("Unused Main"));
        assert!(!dependency_output.output.trim_start().starts_with('{'));

        let dependency_json_output = render_export_inspection_report_output(
            &args,
            import_dir,
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
            .contains("\"orphanedDatasources\""));
    }
}
