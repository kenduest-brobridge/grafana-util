//! Datasource import dry-run output rendering helpers.

use serde_json::{Map, Value};

use crate::common::{render_json_value, tool_version, Result};

use super::datasource_import_dry_run_review::build_datasource_import_dry_run_review_envelope;
use super::datasource_import_export_support::DatasourceImportDryRunReport;
use super::datasource_import_secret_visibility::build_import_secret_visibility_entries;
use super::{render_import_table, DatasourceImportArgs};
use crate::datasource::DryRunOutputFormat;

pub(crate) fn format_datasource_import_dry_run_line(row: &[String]) -> String {
    format!(
        "Dry-run datasource uid={} name={} type={} match={} dest={} action={} targetUid={} targetVersion={} targetReadOnly={} blockedReason={} file={}",
        row[0], row[1], row[2], row[3], row[4], row[5], row[8], row[9], row[10], row[11], row[7]
    )
}

fn optional_string_value(value: &str) -> Value {
    if value.trim().is_empty() {
        Value::Null
    } else {
        Value::String(value.to_string())
    }
}

fn optional_i64_value(value: &str) -> Value {
    value.parse::<i64>().map(Value::from).unwrap_or(Value::Null)
}

fn optional_bool_value(value: &str) -> Value {
    value
        .parse::<bool>()
        .map(Value::from)
        .unwrap_or(Value::Null)
}

pub(crate) fn build_datasource_import_dry_run_json_value(
    report: &DatasourceImportDryRunReport,
) -> Value {
    let secret_visibility =
        build_import_secret_visibility_entries(&report.input_dir, report.input_format);
    Value::Object(Map::from_iter(vec![
        (
            "kind".to_string(),
            Value::String("grafana-util-datasource-import-dry-run".to_string()),
        ),
        ("schemaVersion".to_string(), Value::Number(1.into())),
        (
            "toolVersion".to_string(),
            Value::String(tool_version().to_string()),
        ),
        ("reviewRequired".to_string(), Value::Bool(true)),
        ("reviewed".to_string(), Value::Bool(false)),
        ("mode".to_string(), Value::String(report.mode.clone())),
        (
            "sourceOrgId".to_string(),
            Value::String(report.source_org_id.clone()),
        ),
        (
            "targetOrgId".to_string(),
            Value::String(report.target_org_id.clone()),
        ),
        (
            "datasources".to_string(),
            Value::Array(
                report
                    .rows
                    .iter()
                    .map(|row| {
                        Value::Object(Map::from_iter(vec![
                            ("uid".to_string(), Value::String(row[0].clone())),
                            ("name".to_string(), Value::String(row[1].clone())),
                            ("type".to_string(), Value::String(row[2].clone())),
                            ("matchBasis".to_string(), Value::String(row[3].clone())),
                            ("destination".to_string(), Value::String(row[4].clone())),
                            ("action".to_string(), Value::String(row[5].clone())),
                            ("orgId".to_string(), Value::String(row[6].clone())),
                            ("file".to_string(), Value::String(row[7].clone())),
                            ("targetUid".to_string(), optional_string_value(&row[8])),
                            ("targetVersion".to_string(), optional_i64_value(&row[9])),
                            ("targetReadOnly".to_string(), optional_bool_value(&row[10])),
                            ("blockedReason".to_string(), optional_string_value(&row[11])),
                        ]))
                    })
                    .collect(),
            ),
        ),
        (
            "summary".to_string(),
            Value::Object(Map::from_iter(vec![
                (
                    "datasourceCount".to_string(),
                    Value::Number((report.datasource_count as i64).into()),
                ),
                (
                    "wouldCreate".to_string(),
                    Value::Number((report.would_create as i64).into()),
                ),
                (
                    "wouldUpdate".to_string(),
                    Value::Number((report.would_update as i64).into()),
                ),
                (
                    "wouldSkip".to_string(),
                    Value::Number((report.would_skip as i64).into()),
                ),
                (
                    "wouldBlock".to_string(),
                    Value::Number((report.would_block as i64).into()),
                ),
                (
                    "secretVisibilityCount".to_string(),
                    Value::Number((secret_visibility.len() as i64).into()),
                ),
            ])),
        ),
        (
            "secretVisibility".to_string(),
            Value::Array(secret_visibility),
        ),
    ]))
}

pub(crate) fn print_datasource_import_dry_run_report(
    report: &DatasourceImportDryRunReport,
    args: &DatasourceImportArgs,
) -> Result<()> {
    if args.output_format == Some(DryRunOutputFormat::Interactive) {
        let envelope = build_datasource_import_dry_run_review_envelope(report);
        crate::review_browser::run_review_mutation_browser("Datasource import dry-run", &envelope)?;
    } else if args.json {
        print!(
            "{}",
            render_json_value(&build_datasource_import_dry_run_json_value(report))?
        );
    } else if args.table {
        for line in render_import_table(
            &report.rows,
            !args.no_header,
            if args.output_columns.is_empty() {
                None
            } else {
                Some(args.output_columns.as_slice())
            },
        ) {
            println!("{line}");
        }
        println!(
            "Dry-run checked {} datasource(s) from {}",
            report.datasource_count,
            report.input_dir.display()
        );
        let secret_visibility =
            build_import_secret_visibility_entries(&report.input_dir, report.input_format);
        if !secret_visibility.is_empty() {
            println!(
                "Secret placeholder visibility: {}",
                Value::Array(secret_visibility)
            );
        }
    } else {
        println!("Import mode: {}", report.mode);
        for row in &report.rows {
            println!("{}", format_datasource_import_dry_run_line(row));
        }
        println!(
            "Dry-run checked {} datasource(s) from {}",
            report.datasource_count,
            report.input_dir.display()
        );
        let secret_visibility =
            build_import_secret_visibility_entries(&report.input_dir, report.input_format);
        if !secret_visibility.is_empty() {
            println!(
                "Secret placeholder visibility: {}",
                Value::Array(secret_visibility)
            );
        }
    }
    Ok(())
}
