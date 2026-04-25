//! Inspection path for Dashboard resources: analysis, extraction, and report shaping.

#[path = "inspect_orchestration_input.rs"]
mod inspect_orchestration_input;
#[cfg(feature = "tui")]
#[path = "inspect_orchestration/tui.rs"]
mod tui;

#[cfg(not(feature = "tui"))]
use std::io::{self, IsTerminal, Write};
use std::path::Path;

#[cfg(feature = "tui")]
use self::tui::{prompt_interactive_input_type, run_interactive_export_workbench};
use super::super::cli_defs::{
    DashboardImportInputFormat, InspectExportArgs, InspectExportInputType,
    InspectExportReportFormat, InspectOutputFormat,
};
use super::super::files::DashboardSourceKind;
use super::super::inspect_live::TempInspectDir;
use super::super::inspect_report::{
    refresh_filtered_query_report_summary, report_format_supports_columns,
    resolve_report_column_ids_for_format, ExportInspectionQueryReport,
};
use super::inspect_output::{
    render_export_inspection_report_output, render_export_inspection_summary_output,
};
use super::inspect_query_report::build_export_inspection_query_report_for_variant;
use super::write_inspect_output;
use crate::common::{message, Result};
pub(crate) use inspect_orchestration_input::ResolvedInspectExportInput;

fn map_output_format_to_report(
    output_format: InspectOutputFormat,
) -> Option<InspectExportReportFormat> {
    match output_format {
        InspectOutputFormat::Text
        | InspectOutputFormat::Table
        | InspectOutputFormat::Csv
        | InspectOutputFormat::Json
        | InspectOutputFormat::Yaml => None,
        InspectOutputFormat::Tree => Some(InspectExportReportFormat::Tree),
        InspectOutputFormat::TreeTable => Some(InspectExportReportFormat::TreeTable),
        InspectOutputFormat::Dependency => Some(InspectExportReportFormat::Dependency),
        InspectOutputFormat::DependencyJson => Some(InspectExportReportFormat::DependencyJson),
        InspectOutputFormat::Governance => Some(InspectExportReportFormat::Governance),
        InspectOutputFormat::GovernanceJson => Some(InspectExportReportFormat::GovernanceJson),
        InspectOutputFormat::QueriesJson => Some(InspectExportReportFormat::QueriesJson),
    }
}

pub(crate) fn effective_inspect_report_format(
    args: &InspectExportArgs,
) -> Option<InspectExportReportFormat> {
    args.output_format.and_then(map_output_format_to_report)
}

pub(crate) fn effective_inspect_output_format(args: &InspectExportArgs) -> InspectOutputFormat {
    args.output_format.unwrap_or({
        if args.text {
            InspectOutputFormat::Text
        } else if args.table {
            InspectOutputFormat::Table
        } else if args.csv {
            InspectOutputFormat::Csv
        } else if args.json {
            InspectOutputFormat::Json
        } else if args.yaml {
            InspectOutputFormat::Yaml
        } else {
            InspectOutputFormat::Text
        }
    })
}

pub(crate) fn resolve_inspect_export_import_dir(
    temp_root: &Path,
    input_dir: &Path,
    input_format: DashboardImportInputFormat,
    input_type: Option<InspectExportInputType>,
    interactive: bool,
) -> Result<ResolvedInspectExportInput> {
    inspect_orchestration_input::resolve_inspect_export_import_dir_with_prompt(
        temp_root,
        input_dir,
        input_format,
        input_type,
        interactive,
        prompt_interactive_input_type,
    )
}

#[cfg(any(test, not(feature = "tui")))]
fn parse_interactive_input_type_answer(answer: &str) -> Option<InspectExportInputType> {
    match answer.trim().to_ascii_lowercase().as_str() {
        "1" | "raw" | "r" => Some(InspectExportInputType::Raw),
        "2" | "source" | "s" | "prompt" | "p" => Some(InspectExportInputType::Source),
        _ => None,
    }
}

#[cfg(not(feature = "tui"))]
fn prompt_interactive_input_type(input_dir: &Path) -> Result<InspectExportInputType> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(message(format!(
            "Import path {} contains both raw/ and prompt/ dashboard variants. Re-run with --input-type raw or --input-type source.",
            input_dir.display()
        )));
    }
    loop {
        println!("Title: Choose dashboard export variant");
        println!("Import: {}", input_dir.display());
        println!();
        println!("1. raw (Inspect API-safe raw export artifacts)");
        println!("2. source (Inspect prompt/source export artifacts)");
        print!("Choice [1-2/raw/source]: ");
        io::stdout().flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        if let Some(input_type) = parse_interactive_input_type_answer(&line) {
            return Ok(input_type);
        }
        eprintln!("Enter 1, 2, raw, or source.");
    }
}

pub(crate) fn apply_query_report_filters(
    mut report: ExportInspectionQueryReport,
    datasource_filter: Option<&str>,
    panel_id_filter: Option<&str>,
) -> ExportInspectionQueryReport {
    let datasource_filter = datasource_filter
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let panel_id_filter = panel_id_filter
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if datasource_filter.is_none() && panel_id_filter.is_none() {
        return report;
    }
    report.queries.retain(|row| {
        let datasource_match = datasource_filter
            .map(|value| {
                row.datasource == value
                    || row.datasource_uid == value
                    || row.datasource_type == value
                    || row.datasource_family == value
            })
            .unwrap_or(true);
        let panel_match = panel_id_filter
            .map(|value| row.panel_id == value)
            .unwrap_or(true);
        datasource_match && panel_match
    });
    refresh_filtered_query_report_summary(&mut report);
    report
}

pub(crate) fn validate_inspect_export_report_args(args: &InspectExportArgs) -> Result<()> {
    let report_format = effective_inspect_report_format(args);
    if report_format.is_none() {
        if !args.report_columns.is_empty() {
            return Err(message(
                "--report-columns is only supported together with table, csv, tree-table, or queries-json output.",
            ));
        }
        if args.report_filter_datasource.is_some() {
            return Err(message(
                "--report-filter-datasource is only supported together with table, csv, tree-table, dependency, dependency-json, governance, governance-json, or queries-json output.",
            ));
        }
        if args.report_filter_panel_id.is_some() {
            return Err(message(
                "--report-filter-panel-id is only supported together with table, csv, tree-table, dependency, dependency-json, governance, governance-json, or queries-json output.",
            ));
        }
        return Ok(());
    }
    if report_format
        .map(|format| !report_format_supports_columns(format))
        .unwrap_or(false)
        && !args.report_columns.is_empty()
    {
        return Err(message(
            "--report-columns is only supported with table, csv, or tree-table output.",
        ));
    }
    let _ = resolve_report_column_ids_for_format(report_format, &args.report_columns)?;
    Ok(())
}

fn analyze_export_dir_at_path(
    args: &InspectExportArgs,
    input_dir: &Path,
    expected_variant: &str,
    source_kind: Option<DashboardSourceKind>,
) -> Result<usize> {
    if args.interactive {
        return run_interactive_export_workbench(input_dir, expected_variant, source_kind);
    }
    let write_output = |output: &str| -> Result<()> {
        write_inspect_output(output, args.output_file.as_ref(), args.also_stdout)
    };

    if let Some(report_format) = effective_inspect_report_format(args) {
        let report = apply_query_report_filters(
            build_export_inspection_query_report_for_variant(input_dir, expected_variant)?,
            args.report_filter_datasource.as_deref(),
            args.report_filter_panel_id.as_deref(),
        );
        let rendered = render_export_inspection_report_output(
            args,
            input_dir,
            expected_variant,
            report_format,
            &report,
        )?;
        write_output(&rendered.output)?;
        return Ok(rendered.dashboard_count);
    }

    let summary =
        super::super::build_export_inspection_summary_for_variant(input_dir, expected_variant)?;
    let output = render_export_inspection_summary_output(args, &summary)?;
    write_output(&output)?;
    Ok(summary.dashboard_count)
}

#[cfg(not(feature = "tui"))]
// Non-TUI path preserves signature by returning a feature-missing error.
fn run_interactive_export_workbench(
    _import_dir: &Path,
    _expected_variant: &str,
    _source_kind: Option<DashboardSourceKind>,
) -> Result<usize> {
    super::super::tui_not_built("summary-export --interactive")
}

pub(crate) fn analyze_export_dir(args: &InspectExportArgs) -> Result<usize> {
    validate_inspect_export_report_args(args)?;
    let temp_dir = TempInspectDir::new("summary-export")?;
    let resolved = resolve_inspect_export_import_dir(
        &temp_dir.path,
        &args.input_dir,
        args.input_format,
        args.input_type,
        args.interactive,
    )?;
    analyze_export_dir_at_path(
        args,
        &resolved.input_dir,
        resolved.expected_variant,
        resolved.source_kind,
    )
}

#[cfg(test)]
mod tests {
    use super::{parse_interactive_input_type_answer, InspectExportInputType};

    #[test]
    fn parse_interactive_input_type_answer_accepts_expected_aliases() {
        assert_eq!(
            parse_interactive_input_type_answer("raw"),
            Some(InspectExportInputType::Raw)
        );
        assert_eq!(
            parse_interactive_input_type_answer("r"),
            Some(InspectExportInputType::Raw)
        );
        assert_eq!(
            parse_interactive_input_type_answer("source"),
            Some(InspectExportInputType::Source)
        );
        assert_eq!(
            parse_interactive_input_type_answer("prompt"),
            Some(InspectExportInputType::Source)
        );
        assert_eq!(
            parse_interactive_input_type_answer("s"),
            Some(InspectExportInputType::Source)
        );
        assert_eq!(parse_interactive_input_type_answer(""), None);
    }
}
