//! Inspection path for Dashboard resources: analysis, extraction, and report shaping.

#[path = "inspect_output/dependency.rs"]
mod dependency;
#[path = "inspect_output_report.rs"]
mod inspect_output_report;
#[path = "inspect_output/summary.rs"]
mod summary;

pub(crate) use dependency::render_export_inspection_dependency_output;
pub(crate) use inspect_output_report::render_export_inspection_report_output;
pub(crate) use summary::render_export_inspection_summary_output;

pub(crate) struct ExportInspectionRenderedOutput {
    pub(crate) output: String,
    pub(crate) dashboard_count: usize,
}

pub(crate) fn render_lines_to_string(lines: Vec<String>) -> String {
    let mut output = String::new();
    for line in lines {
        output.push_str(&line);
        output.push('\n');
    }
    output
}
