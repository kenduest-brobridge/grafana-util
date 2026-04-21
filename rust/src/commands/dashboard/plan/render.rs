//! Dashboard plan text and table rendering.

use crate::common::Result;
use crate::review_contract::{
    REVIEW_ACTION_EXTRA_REMOTE, REVIEW_ACTION_SAME, REVIEW_ACTION_WOULD_CREATE,
    REVIEW_ACTION_WOULD_DELETE, REVIEW_ACTION_WOULD_UPDATE, REVIEW_STATUS_BLOCKED,
};

use super::super::DashboardPlanOutputFormat;
use super::build_dashboard_plan_json;
use super::types::DashboardPlanReport;

pub(crate) fn dashboard_plan_column_ids() -> &'static [&'static str] {
    &[
        "action_id",
        "action",
        "status",
        "dashboard_uid",
        "dashboard_title",
        "folder_uid",
        "folder_path",
        "source_org_id",
        "source_org_name",
        "target_org_id",
        "target_org_name",
        "match_basis",
        "changed_fields",
        "blocked_reason",
        "subject_type",
        "subject_name",
        "permission_name",
        "inherited",
        "source_file",
    ]
}

fn plan_output_columns(selected: &[String]) -> Vec<&'static str> {
    if selected.is_empty() || selected.iter().any(|value| value == "all") {
        return dashboard_plan_column_ids().to_vec();
    }
    selected
        .iter()
        .filter_map(|value| match value.as_str() {
            "action_id" => Some("action_id"),
            "action" => Some("action"),
            "status" => Some("status"),
            "dashboard_uid" => Some("dashboard_uid"),
            "dashboard_title" => Some("dashboard_title"),
            "folder_uid" => Some("folder_uid"),
            "folder_path" => Some("folder_path"),
            "source_org_id" => Some("source_org_id"),
            "source_org_name" => Some("source_org_name"),
            "target_org_id" => Some("target_org_id"),
            "target_org_name" => Some("target_org_name"),
            "match_basis" => Some("match_basis"),
            "changed_fields" => Some("changed_fields"),
            "blocked_reason" => Some("blocked_reason"),
            "subject_type" => Some("subject_type"),
            "subject_name" => Some("subject_name"),
            "permission_name" => Some("permission_name"),
            "inherited" => Some("inherited"),
            "source_file" => Some("source_file"),
            _ => None,
        })
        .collect()
}

pub(super) fn plan_summary_line(report: &DashboardPlanReport) -> String {
    format!(
        "Dashboard plan: checked={} same={} create={} update={} extra={} delete={} blocked={} warning={} orgs={} prune={}",
        report.summary.checked,
        report.summary.same,
        report.summary.create,
        report.summary.update,
        report.summary.extra,
        report.summary.delete,
        report.summary.blocked,
        report.summary.warning,
        report.summary.org_count,
        report.prune
    )
}

pub(super) fn render_plan_text(report: &DashboardPlanReport, show_same: bool) -> Vec<String> {
    let mut lines = Vec::new();
    for org in &report.orgs {
        lines.push(format!(
            "Org {} / {} -> {} / {}: checked={} same={} create={} update={} extra={} delete={} blocked={} warning={} action={}",
            org.source_org_id.as_deref().unwrap_or("-"),
            org.source_org_name,
            org.target_org_id.as_deref().unwrap_or("<current>"),
            org.target_org_name,
            org.checked,
            org.same,
            org.create,
            org.update,
            org.extra,
            org.delete,
            org.blocked,
            org.warning,
            org.org_action
        ));
    }
    if let Some(domains) = report
        .review
        .get("domains")
        .and_then(|value| value.as_array())
    {
        if !domains.is_empty() {
            let summary = domains
                .iter()
                .filter_map(|domain| {
                    let object = domain.as_object()?;
                    Some(format!(
                        "{}={}",
                        object
                            .get("id")
                            .and_then(|value| value.as_str())
                            .unwrap_or("unknown"),
                        object
                            .get("checked")
                            .and_then(|value| value.as_i64())
                            .unwrap_or(0)
                    ))
                })
                .collect::<Vec<String>>();
            if !summary.is_empty() {
                lines.push(format!("Domains: {}", summary.join("  ")));
            }
        }
    }
    for action in &report.actions {
        if !show_same && action.action == REVIEW_ACTION_SAME {
            continue;
        }
        lines.push(format!(
            "{} org={} uid={} title={} folder={} action={} status={} changed={}",
            if action.status == REVIEW_STATUS_BLOCKED {
                "BLOCK"
            } else if action.action == REVIEW_ACTION_WOULD_DELETE {
                "DELETE"
            } else if action.action == REVIEW_ACTION_WOULD_CREATE {
                "CREATE"
            } else if action.action == REVIEW_ACTION_WOULD_UPDATE {
                "UPDATE"
            } else if action.action == REVIEW_ACTION_EXTRA_REMOTE {
                "EXTRA"
            } else {
                "SAME"
            },
            action.target_org_name,
            action.dashboard_uid,
            action.title,
            action.folder_path,
            action.action,
            action.status,
            if action.changed_fields.is_empty() {
                "none".to_string()
            } else {
                action.changed_fields.join(",")
            }
        ));
        if let Some(permission) = &action.permission {
            lines.push(format!(
                "  permission subject={} name={} permission={} inherited={}",
                permission.subject_type,
                permission.subject_name,
                permission.permission_name,
                permission.inherited
            ));
        }
    }
    if let Some(reasons) = report
        .review
        .get("blockedReasons")
        .and_then(|value| value.as_array())
    {
        for reason in reasons.iter().filter_map(|value| value.as_str()) {
            lines.push(format!("Blocked reason: {reason}"));
        }
    }
    lines
}

pub(super) fn render_plan_table(
    report: &DashboardPlanReport,
    show_same: bool,
    include_header: bool,
    selected_columns: &[String],
) -> Vec<String> {
    let columns = plan_output_columns(selected_columns);
    let rows = report
        .actions
        .iter()
        .filter(|action| show_same || action.action != REVIEW_ACTION_SAME)
        .map(|action| {
            columns
                .iter()
                .map(|column| match *column {
                    "action_id" => action.action_id.clone(),
                    "action" => action.action.clone(),
                    "status" => action.status.clone(),
                    "dashboard_uid" => action.dashboard_uid.clone(),
                    "dashboard_title" => action.title.clone(),
                    "folder_uid" => action.folder_uid.clone(),
                    "folder_path" => action.folder_path.clone(),
                    "source_org_id" => action.source_org_id.clone().unwrap_or_default(),
                    "source_org_name" => action.source_org_name.clone(),
                    "target_org_id" => action.target_org_id.clone().unwrap_or_default(),
                    "target_org_name" => action.target_org_name.clone(),
                    "match_basis" => action.match_basis.clone(),
                    "changed_fields" => {
                        if action.changed_fields.is_empty() {
                            String::new()
                        } else {
                            action.changed_fields.join(",")
                        }
                    }
                    "blocked_reason" => action.blocked_reason.clone().unwrap_or_default(),
                    "subject_type" => action
                        .permission
                        .as_ref()
                        .map(|permission| permission.subject_type.clone())
                        .unwrap_or_default(),
                    "subject_name" => action
                        .permission
                        .as_ref()
                        .map(|permission| permission.subject_name.clone())
                        .unwrap_or_default(),
                    "permission_name" => action
                        .permission
                        .as_ref()
                        .map(|permission| permission.permission_name.clone())
                        .unwrap_or_default(),
                    "inherited" => action
                        .permission
                        .as_ref()
                        .map(|permission| permission.inherited.to_string())
                        .unwrap_or_default(),
                    "source_file" => action.source_file.clone().unwrap_or_default(),
                    _ => String::new(),
                })
                .collect::<Vec<String>>()
        })
        .collect::<Vec<Vec<String>>>();
    let headers = columns
        .iter()
        .map(|value| value.to_ascii_uppercase())
        .collect::<Vec<String>>();
    let widths = {
        let mut widths = headers
            .iter()
            .map(|header| header.len())
            .collect::<Vec<usize>>();
        for row in &rows {
            for (index, value) in row.iter().enumerate() {
                widths[index] = widths[index].max(value.len());
            }
        }
        widths
    };
    let format_row = |values: &[String]| -> String {
        values
            .iter()
            .enumerate()
            .map(|(index, value)| format!("{value:<width$}", width = widths[index]))
            .collect::<Vec<String>>()
            .join("  ")
    };
    let mut lines = Vec::new();
    if include_header {
        lines.push(format_row(&headers));
        lines.push(
            widths
                .iter()
                .map(|width| "-".repeat(*width))
                .collect::<Vec<String>>()
                .join("  "),
        );
    }
    for row in rows {
        lines.push(format_row(&row));
    }
    lines
}

pub(crate) fn print_dashboard_plan_report(
    report: &DashboardPlanReport,
    output_format: DashboardPlanOutputFormat,
    show_same: bool,
    no_header: bool,
    selected_columns: &[String],
) -> Result<()> {
    match output_format {
        DashboardPlanOutputFormat::Json => {
            print!(
                "{}",
                crate::common::render_json_value(&build_dashboard_plan_json(report)?)?
            );
        }
        DashboardPlanOutputFormat::Table => {
            for line in render_plan_table(report, show_same, !no_header, selected_columns) {
                println!("{line}");
            }
            println!("{}", plan_summary_line(report));
        }
        DashboardPlanOutputFormat::Text => {
            println!("{}", plan_summary_line(report));
            for line in render_plan_text(report, show_same) {
                println!("{line}");
            }
        }
    }
    Ok(())
}
