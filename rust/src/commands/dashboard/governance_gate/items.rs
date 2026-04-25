use std::cmp::Reverse;

use super::DashboardGovernanceGateFinding;

fn field_or_dash(value: &str) -> &str {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "-"
    } else {
        trimmed
    }
}

fn shorten_text(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    let mut chars = trimmed.chars();
    let head = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{head}...")
    } else {
        head
    }
}

fn paired_label(value: &str, label_name: &str, label_value: &str) -> String {
    let value = value.trim();
    let label_value = label_value.trim();
    match (value.is_empty(), label_value.is_empty()) {
        (true, true) => "-".to_string(),
        (false, true) => value.to_string(),
        (true, false) => format!("{label_name}={label_value}"),
        (false, false) => format!("{value} ({label_name}={label_value})"),
    }
}

fn datasource_label(record: &DashboardGovernanceGateFinding) -> String {
    let mut parts = Vec::new();
    if !record.datasource.is_empty() {
        parts.push(format!("name={}", shorten_text(&record.datasource, 32)));
    }
    if !record.datasource_uid.is_empty() {
        parts.push(format!("uid={}", record.datasource_uid));
    }
    if !record.datasource_family.is_empty() {
        parts.push(format!("family={}", record.datasource_family));
    }
    if parts.is_empty() {
        "-".to_string()
    } else {
        parts.join(" ")
    }
}

fn finding_context_score(record: &DashboardGovernanceGateFinding) -> usize {
    [
        !record.dashboard_uid.trim().is_empty(),
        !record.dashboard_title.trim().is_empty(),
        !record.panel_id.trim().is_empty(),
        !record.panel_title.trim().is_empty(),
        !record.ref_id.trim().is_empty(),
        !record.datasource.trim().is_empty(),
        !record.datasource_uid.trim().is_empty(),
        !record.datasource_family.trim().is_empty(),
        !record.risk_kind.trim().is_empty(),
    ]
    .into_iter()
    .filter(|value| *value)
    .count()
}

fn finding_scope_title(record: &DashboardGovernanceGateFinding) -> String {
    let mut parts = Vec::new();
    if !record.dashboard_title.trim().is_empty() || !record.dashboard_uid.trim().is_empty() {
        parts.push(paired_label(
            &record.dashboard_title,
            "uid",
            &record.dashboard_uid,
        ));
    }
    if !record.panel_title.trim().is_empty() || !record.panel_id.trim().is_empty() {
        parts.push(paired_label(&record.panel_title, "id", &record.panel_id));
    }
    if parts.is_empty() {
        if !record.ref_id.trim().is_empty() {
            parts.push(format!("ref={}", record.ref_id.trim()));
        } else if !record.datasource.trim().is_empty() {
            parts.push(shorten_text(&record.datasource, 32));
        } else if !record.datasource_family.trim().is_empty() {
            parts.push(record.datasource_family.trim().to_string());
        } else {
            parts.push("unscoped".to_string());
        }
    }
    parts.join(" / ")
}

pub(crate) fn finding_sort_key(
    record: &DashboardGovernanceGateFinding,
) -> (
    u8,
    Reverse<usize>,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
) {
    let severity_rank = match record.severity.as_str() {
        "error" => 0,
        "warning" => 1,
        _ => 2,
    };
    (
        severity_rank,
        Reverse(finding_context_score(record)),
        record.dashboard_title.to_ascii_lowercase(),
        record.dashboard_uid.to_ascii_lowercase(),
        record.panel_title.to_ascii_lowercase(),
        record.panel_id.to_ascii_lowercase(),
        record.ref_id.to_ascii_lowercase(),
        record.code.to_ascii_lowercase(),
        record.message.to_ascii_lowercase(),
    )
}

fn finding_row_title(record: &DashboardGovernanceGateFinding) -> String {
    finding_scope_title(record)
}

fn finding_row_meta(record: &DashboardGovernanceGateFinding) -> String {
    let mut parts = Vec::new();
    if !record.severity.trim().is_empty() {
        parts.push(format!("sev={}", record.severity.trim()));
    }
    if !record.code.trim().is_empty() {
        parts.push(format!("code={}", record.code.trim()));
    }
    if !record.ref_id.trim().is_empty() {
        parts.push(format!("ref={}", record.ref_id.trim()));
    }
    if !record.dashboard_uid.trim().is_empty() {
        parts.push(format!("dashboardUid={}", record.dashboard_uid.trim()));
    } else if !record.dashboard_title.trim().is_empty() {
        parts.push(format!(
            "dashboard={}",
            shorten_text(&record.dashboard_title, 32)
        ));
    }
    if !record.panel_id.trim().is_empty() {
        parts.push(format!("panelId={}", record.panel_id.trim()));
    } else if !record.panel_title.trim().is_empty() {
        parts.push(format!("panel={}", shorten_text(&record.panel_title, 32)));
    }
    if !record.datasource.trim().is_empty() {
        parts.push(format!("ds={}", shorten_text(&record.datasource, 24)));
    }
    if !record.datasource_uid.trim().is_empty() {
        parts.push(format!("dsUid={}", record.datasource_uid.trim()));
    }
    if !record.datasource_family.trim().is_empty() {
        parts.push(format!("family={}", record.datasource_family.trim()));
    }
    if !record.risk_kind.trim().is_empty() {
        parts.push(format!("risk={}", record.risk_kind.trim()));
    }
    if parts.is_empty() {
        "-".to_string()
    } else {
        parts.join(" ")
    }
}

fn finding_row_details(record: &DashboardGovernanceGateFinding) -> Vec<String> {
    vec![
        format!("Scope: {}", finding_scope_title(record)),
        format!("Reason: {}", field_or_dash(&record.message)),
        format!("Severity: {}", field_or_dash(&record.severity)),
        format!("Code: {}", field_or_dash(&record.code)),
        format!("Risk kind: {}", field_or_dash(&record.risk_kind)),
        format!(
            "Dashboard: {}",
            paired_label(&record.dashboard_title, "uid", &record.dashboard_uid)
        ),
        format!(
            "Panel: {}",
            paired_label(&record.panel_title, "id", &record.panel_id)
        ),
        format!("Ref ID: {}", field_or_dash(&record.ref_id)),
        format!("Datasource: {}", datasource_label(record)),
    ]
}

pub(crate) fn build_browser_item(
    kind: &str,
    record: &DashboardGovernanceGateFinding,
) -> crate::interactive_browser::BrowserItem {
    crate::interactive_browser::BrowserItem {
        kind: kind.to_string(),
        title: finding_row_title(record),
        meta: finding_row_meta(record),
        details: finding_row_details(record),
    }
}
