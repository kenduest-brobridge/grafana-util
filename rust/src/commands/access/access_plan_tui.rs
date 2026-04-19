//! Read-only interactive review surface for access plan documents.

use crate::common::Result;
#[cfg(any(feature = "tui", test))]
use crate::review_contract::{
    REVIEW_ACTION_BLOCKED, REVIEW_ACTION_EXTRA_REMOTE, REVIEW_ACTION_SAME, REVIEW_ACTION_UNMANAGED,
    REVIEW_ACTION_WOULD_CREATE, REVIEW_ACTION_WOULD_DELETE, REVIEW_ACTION_WOULD_UPDATE,
    REVIEW_HINT_REMOTE_ONLY, REVIEW_STATUS_BLOCKED, REVIEW_STATUS_WARNING,
};

#[cfg(not(feature = "tui"))]
use crate::common::tui;
#[cfg(feature = "tui")]
use crate::interactive_browser::run_interactive_browser;
#[cfg(any(feature = "tui", test))]
use crate::interactive_browser::BrowserItem;
#[cfg(any(feature = "tui", test))]
use serde_json::{Map, Value};

use super::AccessPlanDocument;

#[cfg(any(feature = "tui", test))]
fn build_access_plan_summary_lines(document: &AccessPlanDocument) -> Vec<String> {
    let review = document.build_review_envelope();
    let mut lines = vec![
        "Access plan review".to_string(),
        format!(
            "Resources: {}  checked: {}  same: {}",
            document.summary.resource_count, document.summary.checked, document.summary.same
        ),
        format!(
            "Create: {}  update: {}  extra remote: {}  delete: {}",
            document.summary.create,
            document.summary.update,
            document.summary.extra_remote,
            document.summary.delete
        ),
        format!(
            "Blocked: {}  warning: {}  prune: {}  actions: {}",
            document.summary.blocked,
            document.summary.warning,
            document.summary.prune,
            document.actions.len()
        ),
    ];
    if !review.blocked_reasons.is_empty() {
        lines.push(format!(
            "Blocked reasons: {}",
            review.blocked_reasons.join(" | ")
        ));
    }
    lines
}

#[cfg(any(feature = "tui", test))]
fn compact_value(value: &Value) -> String {
    match value {
        Value::Null => "-".to_string(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                "-".to_string()
            } else {
                trimmed.to_string()
            }
        }
        Value::Array(items) => {
            let compact = items
                .iter()
                .map(compact_value)
                .filter(|value| value != "-")
                .collect::<Vec<_>>();
            if compact.is_empty() {
                "[]".to_string()
            } else {
                compact.join(", ")
            }
        }
        Value::Object(_) => serde_json::to_string(value).unwrap_or_else(|_| "<object>".to_string()),
    }
}

#[cfg(any(feature = "tui", test))]
fn raw_string_array(raw: &Value, key: &str) -> Vec<String> {
    raw.get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(any(feature = "tui", test))]
fn raw_target(raw: &Value) -> Option<&Map<String, Value>> {
    raw.get("target").and_then(Value::as_object)
}

#[cfg(any(feature = "tui", test))]
fn narrative_line(action: &super::access_plan_types::AccessPlanReviewActionProjection) -> String {
    let resource_kind = action.resource_kind.replace('-', " ");
    let narrative = match action.action.as_str() {
        REVIEW_ACTION_WOULD_CREATE => {
            format!("creates this {resource_kind} in Grafana from the reviewed bundle")
        }
        REVIEW_ACTION_WOULD_UPDATE => {
            format!("changes this live {resource_kind} so it matches the reviewed bundle")
        }
        REVIEW_ACTION_WOULD_DELETE => {
            format!("removes this live-only {resource_kind} because prune review marked it for deletion")
        }
        REVIEW_ACTION_SAME => {
            format!("found no drift for this {resource_kind}; live and bundle already agree")
        }
        REVIEW_ACTION_EXTRA_REMOTE => {
            format!("found a live-only {resource_kind} that is outside the reviewed bundle")
        }
        REVIEW_ACTION_BLOCKED | REVIEW_ACTION_UNMANAGED => {
            format!("found drift for this {resource_kind}, but Grafana should not apply it yet")
        }
        _ => format!("records this {resource_kind} review action for operator follow-up"),
    };
    format!("Narrative: {narrative}.")
}

#[cfg(any(feature = "tui", test))]
fn impact_line(
    action: &super::access_plan_types::AccessPlanReviewActionProjection,
) -> Option<String> {
    let changed_fields = raw_string_array(&action.raw, "changedFields");
    let fields = changed_fields
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let impact = if fields
        .iter()
        .any(|field| matches!(*field, "orgRole" | "grafanaAdmin" | "role"))
    {
        Some("permission or administrative reach would change".to_string())
    } else if fields
        .iter()
        .any(|field| matches!(*field, "users" | "members" | "admins" | "teams"))
    {
        Some("membership or group reach would change".to_string())
    } else if fields
        .iter()
        .any(|field| matches!(*field, "login" | "email" | "name" | "uid"))
    {
        Some("identity matching and ownership tracking would change".to_string())
    } else if fields
        .iter()
        .any(|field| matches!(*field, "disabled" | "tokens"))
    {
        Some("runtime access or automation credentials would change".to_string())
    } else if action.action == REVIEW_ACTION_WOULD_DELETE {
        Some("the live record would disappear after apply".to_string())
    } else if action.action == REVIEW_ACTION_WOULD_CREATE {
        Some("Grafana would gain a new managed access record".to_string())
    } else if action.status == REVIEW_STATUS_BLOCKED {
        Some("the requested drift stays unresolved until the blocker is cleared".to_string())
    } else if action.status == REVIEW_STATUS_WARNING {
        Some("the change needs operator confirmation before it is safe to approve".to_string())
    } else {
        None
    }?;
    Some(format!("Why this matters: {impact}."))
}

#[cfg(any(feature = "tui", test))]
fn blocked_or_warning_context(
    action: &super::access_plan_types::AccessPlanReviewActionProjection,
) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(reason) = &action.blocked_reason {
        lines.push(format!("Blocked context: {reason}."));
    }
    if action.status == REVIEW_STATUS_WARNING {
        let changed_fields = raw_string_array(&action.raw, "changedFields");
        if !changed_fields.is_empty() {
            lines.push(format!(
                "Warning context: verify bundle fields {} against the live target before approving.",
                changed_fields.join(", ")
            ));
        } else {
            lines.push(
                "Warning context: compare the reviewed bundle with the live target before approving."
                    .to_string(),
            );
        }
    }
    if let Some(target) = raw_target(&action.raw) {
        let flags = [
            "isExternal",
            "isProvisioned",
            "isExternallySynced",
            "isGrafanaAdminExternallySynced",
            "disabled",
        ]
        .into_iter()
        .filter_map(|key| {
            target
                .get(key)
                .map(|value| format!("{key}={}", compact_value(value)))
        })
        .collect::<Vec<_>>();
        if !flags.is_empty() && action.status == REVIEW_STATUS_BLOCKED {
            lines.push(format!(
                "Blocked evidence: live target flags {}.",
                flags.join(" ")
            ));
        }
    }
    lines
}

#[cfg(any(feature = "tui", test))]
fn next_check_lines(
    action: &super::access_plan_types::AccessPlanReviewActionProjection,
) -> Vec<String> {
    let mut lines = Vec::new();
    for hint in &action.review_hints {
        let hint_line = if hint.contains(REVIEW_HINT_REMOTE_ONLY) {
            "Check next: decide whether this live-only record should stay unmanaged or be deleted."
                .to_string()
        } else {
            format!("Check next: {}.", hint.trim_end_matches('.'))
        };
        if !lines.contains(&hint_line) {
            lines.push(hint_line);
        }
    }

    let default_line = if action.status == REVIEW_STATUS_BLOCKED {
        "Check next: confirm the blocker in Grafana and adjust the bundle or remote ownership before retrying."
    } else if action.action == REVIEW_ACTION_WOULD_DELETE {
        "Check next: confirm this live-only record is still safe to delete."
    } else if action.action == REVIEW_ACTION_WOULD_CREATE {
        "Check next: confirm identifiers, scope, and memberships before creating it."
    } else if action.action == REVIEW_ACTION_WOULD_UPDATE {
        "Check next: compare the listed bundle fields against the live target evidence."
    } else if action.status == REVIEW_STATUS_WARNING {
        "Check next: review the warning evidence and verify operator intent."
    } else {
        "Check next: no further action is needed unless the bundle changes."
    };
    let default_line = default_line.to_string();
    if !lines.contains(&default_line) {
        lines.push(default_line);
    }
    lines
}

#[cfg(any(feature = "tui", test))]
fn change_detail_lines(raw: &Value) -> Vec<String> {
    raw.get("changes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_object)
        .filter_map(|change| {
            let field = change
                .get("field")
                .and_then(Value::as_str)?
                .trim()
                .to_string();
            if field.is_empty() {
                return None;
            }
            let bundle = compact_value(change.get("before").unwrap_or(&Value::Null));
            let live = compact_value(change.get("after").unwrap_or(&Value::Null));
            Some(format!("Change: {field} bundle={bundle} live={live}"))
        })
        .collect()
}

#[cfg(any(feature = "tui", test))]
fn target_evidence_lines(raw: &Value) -> Vec<String> {
    let Some(target) = raw_target(raw) else {
        return Vec::new();
    };

    [
        "id",
        "uid",
        "login",
        "email",
        "name",
        "orgRole",
        "role",
        "grafanaAdmin",
        "orgId",
        "memberCount",
        "scope",
        "origin",
        "disabled",
    ]
    .into_iter()
    .filter_map(|key| {
        target
            .get(key)
            .map(|value| format!("Live target: {key}={}", compact_value(value)))
    })
    .collect()
}

#[cfg(any(feature = "tui", test))]
fn resource_focus_line(resource: &super::AccessPlanResourceReport) -> Option<String> {
    let detail = if resource.blocked > 0 {
        Some(format!(
            "{} blocked actions need operator resolution before this bundle can be applied cleanly",
            resource.blocked
        ))
    } else if resource.warning > 0 {
        Some(format!(
            "{} warning actions need operator review before approval",
            resource.warning
        ))
    } else if resource.create + resource.update + resource.delete > 0 {
        Some("this bundle contains live changes that should be reviewed before apply".to_string())
    } else if resource.same == resource.checked && resource.checked > 0 {
        Some("this bundle currently matches live state".to_string())
    } else {
        None
    }?;
    Some(format!("Review focus: {detail}."))
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn build_access_plan_browser_items(document: &AccessPlanDocument) -> Vec<BrowserItem> {
    let projection = document.build_review_projection();
    let mut items = Vec::with_capacity(document.resources.len() + projection.actions.len());

    for resource in &document.resources {
        items.push(BrowserItem {
            kind: "resource".to_string(),
            title: format!("{} bundle", resource.resource_kind),
            meta: format!(
                "{}  checked={} same={} create={} update={} extra={} delete={} blocked={} warning={}",
                if resource.bundle_present {
                    "present"
                } else {
                    "missing"
                },
                resource.checked,
                resource.same,
                resource.create,
                resource.update,
                resource.extra_remote,
                resource.delete,
                resource.blocked,
                resource.warning
            ),
            details: {
                let mut details = vec![
                    format!("Resource kind: {}", resource.resource_kind),
                    format!("Source path: {}", resource.source_path),
                    format!("Bundle present: {}", resource.bundle_present),
                    format!("Source count: {}", resource.source_count),
                    format!("Live count: {}", resource.live_count),
                ];
                if let Some(scope) = &resource.scope {
                    details.push(format!("Scope: {}", scope));
                }
                if let Some(focus) = resource_focus_line(resource) {
                    details.push(focus);
                }
                details.extend(resource.notes.iter().map(|note| format!("Note: {}", note)));
                details
            },
        });
    }

    for action in projection.actions {
        let source_path = action
            .raw
            .get("sourcePath")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let scope = action
            .raw
            .get("scope")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let mut details = vec![
            format!("Action id: {}", action.action_id),
            format!("Domain: {}", action.domain),
            format!("Resource kind: {}", action.resource_kind),
            format!("Identity: {}", action.identity),
            format!("Action: {}", action.action),
            format!("Status: {}", action.status),
            narrative_line(&action),
        ];
        if !scope.is_empty() {
            details.push(format!("Scope: {}", scope));
        }
        if let Some(impact) = impact_line(&action) {
            details.push(impact);
        }
        if let Some(detail) = &action.details {
            details.push(format!("Details: {}", detail));
        }
        if let Some(reason) = &action.blocked_reason {
            details.push(format!("Blocked reason: {}", reason));
        }
        if !source_path.is_empty() {
            details.push(format!("Source path: {}", source_path));
        }
        details.extend(change_detail_lines(&action.raw));
        details.extend(target_evidence_lines(&action.raw));
        details.extend(blocked_or_warning_context(&action));
        details.extend(
            action
                .review_hints
                .iter()
                .map(|hint| format!("Hint: {}", hint)),
        );
        details.extend(next_check_lines(&action));
        items.push(BrowserItem {
            kind: action.resource_kind.clone(),
            title: action.identity,
            meta: format!(
                "{}  {}  {}",
                action.status, action.action, action.order_group
            ),
            details,
        });
    }

    items
}

#[cfg(feature = "tui")]
pub(crate) fn run_access_plan_interactive(document: &AccessPlanDocument) -> Result<()> {
    let summary_lines = build_access_plan_summary_lines(document);
    let items = build_access_plan_browser_items(document);
    run_interactive_browser("Access plan review", &summary_lines, &items)
}

#[cfg(not(feature = "tui"))]
pub(crate) fn run_access_plan_interactive(_document: &AccessPlanDocument) -> Result<()> {
    Err(tui("Access plan --interactive requires the `tui` feature."))
}
