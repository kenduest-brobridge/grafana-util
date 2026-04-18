use crate::common::{render_json_value, Result};
use crate::tabular_output::render_yaml;

use super::super::inspect_render::render_simple_table;
use super::{ExportLayoutOutputFormat, ExportLayoutPlan};

pub(super) fn print_export_layout_plan(
    plan: &ExportLayoutPlan,
    output_format: ExportLayoutOutputFormat,
    no_header: bool,
    show_operations: bool,
) -> Result<()> {
    match output_format {
        ExportLayoutOutputFormat::Json => print!("{}", render_json_value(plan)?),
        ExportLayoutOutputFormat::Yaml => print!("{}", render_yaml(plan)?),
        ExportLayoutOutputFormat::Csv => {
            print!("{}", render_export_layout_plan_csv(plan, show_operations))
        }
        ExportLayoutOutputFormat::Table => {
            for line in render_export_layout_plan_table(plan, show_operations, !no_header) {
                println!("{line}");
            }
        }
        ExportLayoutOutputFormat::Text => {
            print!("{}", render_export_layout_plan_text(plan, show_operations));
        }
    }
    Ok(())
}

pub(super) fn render_export_layout_plan_table(
    plan: &ExportLayoutPlan,
    show_operations: bool,
    include_header: bool,
) -> Vec<String> {
    if show_operations {
        let mut rows = plan
            .operations
            .iter()
            .map(|operation| {
                vec![
                    operation.action.to_uppercase(),
                    operation.variant.clone(),
                    operation.uid.clone(),
                    operation.from.clone(),
                    operation.to.clone(),
                    operation.reason.clone().unwrap_or_else(|| "-".to_string()),
                ]
            })
            .collect::<Vec<_>>();
        rows.extend(plan.extra_files.iter().map(|extra_file| {
            vec![
                "EXTRA".to_string(),
                extra_file.variant.clone(),
                "-".to_string(),
                extra_file.path.clone(),
                extra_file.path.clone(),
                extra_file.reason.clone(),
            ]
        }));
        return render_simple_table(
            &["ACTION", "VARIANT", "UID", "FROM", "TO", "REASON"],
            &rows,
            include_header,
        );
    }

    render_simple_table(
        &["FIELD", "VALUE"],
        &export_layout_summary_rows(plan),
        include_header,
    )
}

fn export_layout_summary_rows(plan: &ExportLayoutPlan) -> Vec<Vec<String>> {
    let mut rows = vec![
        vec![
            "dashboards".to_string(),
            (plan.summary.move_count + plan.summary.unchanged_count + plan.summary.blocked_count)
                .to_string(),
        ],
        vec!["variants".to_string(), plan.variants.join(",")],
        vec!["move".to_string(), plan.summary.move_count.to_string()],
        vec!["same".to_string(), plan.summary.unchanged_count.to_string()],
        vec![
            "blocked".to_string(),
            plan.summary.blocked_count.to_string(),
        ],
        vec![
            "extra".to_string(),
            plan.summary.extra_file_count.to_string(),
        ],
    ];
    if let Some(output_dir) = &plan.output_dir {
        rows.push(vec!["output".to_string(), output_dir.clone()]);
    }
    rows
}

pub(super) fn render_export_layout_plan_csv(
    plan: &ExportLayoutPlan,
    show_operations: bool,
) -> String {
    let mut rows = Vec::new();
    if show_operations {
        rows.push(vec![
            "action".to_string(),
            "variant".to_string(),
            "uid".to_string(),
            "from".to_string(),
            "to".to_string(),
            "reason".to_string(),
        ]);
        rows.extend(plan.operations.iter().map(|operation| {
            vec![
                operation.action.clone(),
                operation.variant.clone(),
                operation.uid.clone(),
                operation.from.clone(),
                operation.to.clone(),
                operation.reason.clone().unwrap_or_default(),
            ]
        }));
        rows.extend(plan.extra_files.iter().map(|extra_file| {
            vec![
                "extra".to_string(),
                extra_file.variant.clone(),
                String::new(),
                extra_file.path.clone(),
                extra_file.path.clone(),
                extra_file.reason.clone(),
            ]
        }));
    } else {
        rows.push(vec![
            "dashboards".to_string(),
            "variants".to_string(),
            "move".to_string(),
            "same".to_string(),
            "blocked".to_string(),
            "extra".to_string(),
            "output".to_string(),
        ]);
        rows.push(vec![
            (plan.summary.move_count + plan.summary.unchanged_count + plan.summary.blocked_count)
                .to_string(),
            plan.variants.join("|"),
            plan.summary.move_count.to_string(),
            plan.summary.unchanged_count.to_string(),
            plan.summary.blocked_count.to_string(),
            plan.summary.extra_file_count.to_string(),
            plan.output_dir.clone().unwrap_or_default(),
        ]);
    }
    rows.into_iter()
        .map(|row| {
            row.into_iter()
                .map(|field| csv_escape(&field))
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

pub(super) fn render_export_layout_plan_text(
    plan: &ExportLayoutPlan,
    show_operations: bool,
) -> String {
    let mut lines = Vec::new();
    lines.push("Export layout repair plan".to_string());
    lines.push(format!(
        "  Dashboards: {}",
        plan.summary.move_count + plan.summary.unchanged_count + plan.summary.blocked_count
    ));
    lines.push(format!("  Variants: {}", plan.variants.join(", ")));
    lines.push(format!(
        "  Operations: {} move, {} same, {} blocked, {} extra",
        plan.summary.move_count,
        plan.summary.unchanged_count,
        plan.summary.blocked_count,
        plan.summary.extra_file_count
    ));
    if let Some(output_dir) = &plan.output_dir {
        lines.push(format!("  Output: {output_dir}"));
    }
    if show_operations {
        lines.push(String::new());
        lines.push("Operations".to_string());
        lines.extend(plan.operations.iter().map(|operation| {
            let reason = operation
                .reason
                .as_ref()
                .map(|value| format!(" reason={value}"))
                .unwrap_or_default();
            format!(
                "  {} {} uid={} {} -> {}{}",
                operation.action.to_uppercase(),
                operation.variant,
                operation.uid,
                operation.from,
                operation.to,
                reason
            )
        }));
        lines.extend(plan.extra_files.iter().map(|extra_file| {
            format!(
                "  EXTRA {} path={} handling={} reason={}",
                extra_file.variant, extra_file.path, extra_file.handling, extra_file.reason
            )
        }));
    }
    lines.join("\n") + "\n"
}
