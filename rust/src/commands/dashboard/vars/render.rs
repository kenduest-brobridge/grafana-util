use std::path::PathBuf;

use crate::common::{emit_plain_output, render_json_value, Result};
use crate::tabular_output::render_yaml;

use super::super::inspect_render::{render_csv, render_simple_table};
use super::super::{InspectVarsArgs, SimpleOutputFormat};
use super::{DashboardVariableDocument, DashboardVariableRow};

pub(super) fn write_inspect_vars_output(
    output: &str,
    output_file: Option<&PathBuf>,
    also_stdout: bool,
) -> Result<()> {
    emit_plain_output(output, output_file.map(PathBuf::as_path), also_stdout)
}

pub(crate) fn render_dashboard_variable_output(
    args: &InspectVarsArgs,
    document: &DashboardVariableDocument,
) -> Result<String> {
    match args.output_format.unwrap_or(SimpleOutputFormat::Table) {
        SimpleOutputFormat::Text => Ok(format!("{}\n", render_dashboard_variable_text(document))),
        SimpleOutputFormat::Json => Ok(format!("{}\n", render_json_value(document)?)),
        SimpleOutputFormat::Yaml => Ok(format!("{}\n", render_yaml(document)?)),
        SimpleOutputFormat::Csv => {
            let mut rendered = String::new();
            for line in render_csv(
                &[
                    "name",
                    "type",
                    "label",
                    "current",
                    "datasource",
                    "multi",
                    "include_all",
                    "option_count",
                    "options",
                ],
                &build_variable_table_rows(&document.variables),
            ) {
                rendered.push_str(&line);
                rendered.push('\n');
            }
            Ok(rendered)
        }
        SimpleOutputFormat::Table => {
            let mut rendered = String::new();
            for line in render_simple_table(
                &["NAME", "TYPE", "LABEL", "CURRENT", "DATASOURCE", "OPTIONS"],
                &document
                    .variables
                    .iter()
                    .map(|row| {
                        vec![
                            row.name.clone(),
                            row.variable_type.clone(),
                            row.label.clone(),
                            row.current.clone(),
                            row.datasource.clone(),
                            summarize_options(row),
                        ]
                    })
                    .collect::<Vec<Vec<String>>>(),
                !args.no_header,
            ) {
                rendered.push_str(&line);
                rendered.push('\n');
            }
            Ok(rendered)
        }
    }
}

fn render_dashboard_variable_text(document: &DashboardVariableDocument) -> String {
    let mut rendered = String::new();
    rendered.push_str(&format!(
        "Dashboard variables: {} ({})\n",
        document.dashboard_title, document.dashboard_uid
    ));
    rendered.push_str(&format!("Variable count: {}\n", document.variable_count));
    if document.variables.is_empty() {
        return rendered;
    }
    rendered.push('\n');
    rendered.push_str("# Variables\n");
    for row in &document.variables {
        rendered.push_str(&format!(
            "- name={} type={} label={} current={} datasource={} query={} multi={} include_all={} options={}\n",
            row.name,
            row.variable_type,
            row.label,
            row.current,
            row.datasource,
            row.query,
            row.multi,
            row.include_all,
            summarize_options(row)
        ));
    }
    rendered
}

fn build_variable_table_rows(rows: &[DashboardVariableRow]) -> Vec<Vec<String>> {
    rows.iter()
        .map(|row| {
            vec![
                row.name.clone(),
                row.variable_type.clone(),
                row.label.clone(),
                row.current.clone(),
                row.datasource.clone(),
                row.multi.to_string(),
                row.include_all.to_string(),
                row.option_count.to_string(),
                summarize_options(row),
            ]
        })
        .collect()
}

fn summarize_options(row: &DashboardVariableRow) -> String {
    const LIMIT: usize = 6;
    if row.options.is_empty() {
        return String::new();
    }
    let mut preview = row
        .options
        .iter()
        .take(LIMIT)
        .cloned()
        .collect::<Vec<String>>();
    if row.options.len() > LIMIT {
        preview.push(format!("(+{} more)", row.options.len() - LIMIT));
    }
    preview.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn write_inspect_vars_output_strips_ansi_and_trailing_newlines() {
        let temp = tempdir().unwrap();
        let output_file = temp.path().join("variables.txt");

        write_inspect_vars_output(
            "Dashboard variables: CPU Main (cpu-main)\n\u{1b}[1;36mVariable count: 1\u{1b}[0m\n\n",
            Some(&output_file),
            false,
        )
        .unwrap();

        let raw = fs::read_to_string(output_file).unwrap();
        assert_eq!(
            raw,
            "Dashboard variables: CPU Main (cpu-main)\nVariable count: 1\n"
        );
    }
}
