use std::io::IsTerminal;
use std::path::Path;

use serde_json::Value;

use crate::common::{json_color_choice, json_color_enabled, string_field, value_as_object, Result};

use super::super::authoring::DashboardAuthoringReviewResult;
use super::super::extract_dashboard_object;

const ANSI_RESET: &str = "\x1b[0m";
const ANSI_HEADER: &str = "\x1b[1;36m";
const ANSI_SUCCESS: &str = "\x1b[1;32m";
const ANSI_WARNING: &str = "\x1b[1;33m";
const ANSI_ERROR: &str = "\x1b[1;31m";
pub(super) const ANSI_DIM: &str = "\x1b[2;90m";
const ANSI_LABEL: &str = "\x1b[1;37m";
const ANSI_VALUE: &str = "\x1b[0;37m";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EditStatusTone {
    Success,
    Warning,
    Error,
}

pub(super) fn style_enabled() -> bool {
    json_color_enabled(json_color_choice(), std::io::stdout().is_terminal())
}

pub(super) fn paint(text: &str, ansi: &str, enabled: bool) -> String {
    if enabled {
        format!("{ansi}{text}{ANSI_RESET}")
    } else {
        text.to_string()
    }
}

pub(super) fn render_section_heading(title: &str, enabled: bool) -> String {
    format!(
        "{} {}",
        paint("==", ANSI_HEADER, enabled),
        paint(title, ANSI_HEADER, enabled)
    )
}

pub(super) fn render_status_line(tone: EditStatusTone, text: &str, enabled: bool) -> String {
    let (label, ansi) = match tone {
        EditStatusTone::Success => ("OK", ANSI_SUCCESS),
        EditStatusTone::Warning => ("INFO", ANSI_WARNING),
        EditStatusTone::Error => ("ERROR", ANSI_ERROR),
    };
    format!("{} {}", paint(label, ansi, enabled), text)
}

fn render_key_value(label: &str, value: &str, enabled: bool) -> String {
    format!(
        "{} {}",
        paint(&format!("{label}:"), ANSI_LABEL, enabled),
        paint(value, ANSI_VALUE, enabled)
    )
}

fn join_tags(value: &Value) -> String {
    value
        .as_array()
        .map(|tags| {
            tags.iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "-".to_string())
}

pub(super) fn summarize_changes(original: &Value, edited: &Value) -> Result<Vec<(String, String)>> {
    let original_object = value_as_object(
        original,
        "Original dashboard edit payload must be an object.",
    )?;
    let edited_object = value_as_object(edited, "Edited dashboard payload must be an object.")?;
    let original_dashboard = extract_dashboard_object(original_object)?;
    let edited_dashboard = extract_dashboard_object(edited_object)?;
    let original_tags = join_tags(original_dashboard.get("tags").unwrap_or(&Value::Null));
    let edited_tags = join_tags(edited_dashboard.get("tags").unwrap_or(&Value::Null));
    Ok(vec![
        (
            "Dashboard UID".to_string(),
            format!(
                "{} -> {}",
                string_field(original_dashboard, "uid", ""),
                string_field(edited_dashboard, "uid", "")
            ),
        ),
        (
            "Title".to_string(),
            format!(
                "{} -> {}",
                string_field(original_dashboard, "title", ""),
                string_field(edited_dashboard, "title", "")
            ),
        ),
        (
            "Folder UID".to_string(),
            format!(
                "{} -> {}",
                string_field(original_object, "folderUid", "-"),
                string_field(edited_object, "folderUid", "-"),
            ),
        ),
        (
            "Tags".to_string(),
            format!("{original_tags} -> {edited_tags}"),
        ),
    ])
}

pub(super) fn render_change_summary(
    source_uid: &str,
    original: &Value,
    edited: &Value,
    enabled: bool,
) -> Result<Vec<String>> {
    let mut lines = vec![
        render_section_heading("Edit Summary", enabled),
        render_status_line(
            EditStatusTone::Success,
            &format!("Captured edited dashboard draft for {source_uid}."),
            enabled,
        ),
    ];
    for (label, value) in summarize_changes(original, edited)? {
        lines.push(render_key_value(&label, &value, enabled));
    }
    Ok(lines)
}

pub(super) fn render_review_summary(
    review: &DashboardAuthoringReviewResult,
    enabled: bool,
) -> Vec<String> {
    let tags = if review.tags.is_empty() {
        "-".to_string()
    } else {
        review.tags.join(", ")
    };
    let mut lines = vec![render_section_heading("Review", enabled)];
    lines.push(render_key_value("File", &review.input_file, enabled));
    lines.push(render_key_value("Kind", &review.document_kind, enabled));
    lines.push(render_key_value("Title", &review.title, enabled));
    lines.push(render_key_value("UID", &review.uid, enabled));
    lines.push(render_key_value(
        "Folder UID",
        review.folder_uid.as_deref().unwrap_or("-"),
        enabled,
    ));
    lines.push(render_key_value("Tags", &tags, enabled));
    lines.push(render_key_value(
        "dashboard.id",
        if review.dashboard_id_is_null {
            "null"
        } else {
            "non-null"
        },
        enabled,
    ));
    lines.push(render_key_value(
        "meta.message",
        if review.meta_message_present {
            "present"
        } else {
            "absent"
        },
        enabled,
    ));
    if review.blocking_issues.is_empty() {
        lines.push(render_status_line(
            EditStatusTone::Success,
            "Blocking issues: none",
            enabled,
        ));
    } else {
        lines.push(render_status_line(
            EditStatusTone::Error,
            &format!("Blocking issues: {}", review.blocking_issues.len()),
            enabled,
        ));
        for issue in &review.blocking_issues {
            lines.push(format!("  - {issue}"));
        }
    }
    lines.push(render_key_value(
        "Next action",
        &review.suggested_next_action,
        enabled,
    ));
    lines
}

pub(super) fn render_final_status(
    source_uid: &str,
    output_path: Option<&Path>,
    apply_live: bool,
    dry_run_only: bool,
    enabled: bool,
) -> Vec<String> {
    let mut lines = vec![render_section_heading("Result", enabled)];
    if apply_live {
        lines.push(render_status_line(
            EditStatusTone::Success,
            &format!("Applied edited dashboard {source_uid} back to Grafana."),
            enabled,
        ));
        lines.push(paint(
            "A new Grafana revision should now exist in live history.",
            ANSI_DIM,
            enabled,
        ));
        return lines;
    }

    if dry_run_only {
        lines.push(render_status_line(
            EditStatusTone::Success,
            &format!("Prepared edited dashboard preview for {source_uid}."),
            enabled,
        ));
        lines.push(paint(
            "No local draft file was written. The next output block is the live publish dry-run preview.",
            ANSI_DIM,
            enabled,
        ));
        return lines;
    }

    let path_text = output_path
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "-".to_string());
    lines.push(render_status_line(
        EditStatusTone::Success,
        &format!("Wrote edited dashboard draft for {source_uid}."),
        enabled,
    ));
    lines.push(render_key_value("Output", &path_text, enabled));
    lines.push(paint(
        "Nothing was written back to Grafana. Use publish or edit-live --apply-live when ready.",
        ANSI_DIM,
        enabled,
    ));
    lines
}
