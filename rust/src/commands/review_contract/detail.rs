use serde_json::{Map, Value};

use super::actions::{
    REVIEW_ACTION_BLOCKED, REVIEW_ACTION_EXTRA_REMOTE, REVIEW_ACTION_SAME, REVIEW_ACTION_UNMANAGED,
    REVIEW_ACTION_WOULD_CREATE, REVIEW_ACTION_WOULD_DELETE, REVIEW_ACTION_WOULD_UPDATE,
    REVIEW_HINT_REMOTE_ONLY, REVIEW_STATUS_BLOCKED, REVIEW_STATUS_WARNING,
};
use super::model::ReviewMutationAction;

pub(crate) fn build_review_mutation_action_detail_lines(
    action: &ReviewMutationAction,
) -> Vec<String> {
    let mut lines = vec![
        format!("Review action id: {}", action.action_id),
        format!("Review domain: {}", action.domain),
        format!("Review resource kind: {}", action.resource_kind),
        format!(
            "Review identity: {} {}",
            action.resource_kind, action.identity
        ),
        format!(
            "Review action: {} (status={})",
            action.action, action.status
        ),
    ];
    if let Some(details) = &action.details {
        lines.push(format!("Review details: {}", details));
    }
    if let Some(reason) = &action.blocked_reason {
        lines.push(format!(
            "Review blocker status: {} by {}",
            action.status, reason
        ));
    } else if action.status == REVIEW_STATUS_BLOCKED {
        lines.push(format!("Review blocker status: {}", action.status));
    }
    lines
}

fn compact_review_value(value: &Value) -> String {
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
                .map(compact_review_value)
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

fn review_raw_string_array(raw: &Value, key: &str) -> Vec<String> {
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

pub(crate) fn build_review_mutation_action_narrative_line(action: &ReviewMutationAction) -> String {
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

pub(crate) fn build_review_mutation_action_impact_line(
    action: &ReviewMutationAction,
) -> Option<String> {
    let changed_fields = review_raw_string_array(&action.raw, "changedFields");
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

pub(crate) fn build_review_mutation_action_change_detail_lines(
    action: &ReviewMutationAction,
) -> Vec<String> {
    action
        .raw
        .get("changes")
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
            if !crate::review_diff::is_safe_review_changed_field(&field) {
                return None;
            }
            let bundle = compact_review_value(change.get("before").unwrap_or(&Value::Null));
            let live = compact_review_value(change.get("after").unwrap_or(&Value::Null));
            Some(format!("Change: {field} bundle={bundle} live={live}"))
        })
        .collect()
}

pub(crate) fn build_review_mutation_action_target_evidence_lines(
    action: &ReviewMutationAction,
) -> Vec<String> {
    let Some(target) = action.raw.get("target").and_then(Value::as_object) else {
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
            .map(|value| format!("Live target: {key}={}", compact_review_value(value)))
    })
    .collect()
}

pub(crate) fn build_review_mutation_action_context_lines(
    action: &ReviewMutationAction,
) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(reason) = &action.blocked_reason {
        lines.push(format!("Blocked context: {reason}."));
    }
    if action.status == REVIEW_STATUS_WARNING {
        let changed_fields = review_raw_string_array(&action.raw, "changedFields");
        let changed_fields = changed_fields
            .into_iter()
            .filter(|field| crate::review_diff::is_safe_review_changed_field(field))
            .collect::<Vec<_>>();
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
    if let Some(target) = action.raw.get("target").and_then(Value::as_object) {
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
                .map(|value| format!("{key}={}", compact_review_value(value)))
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

pub(crate) fn build_review_mutation_action_next_check_lines(
    action: &ReviewMutationAction,
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

pub(crate) fn build_review_mutation_action_diff_preview_lines(
    action: &ReviewMutationAction,
) -> Vec<String> {
    let mut live = Map::new();
    let mut desired = Map::new();
    let mut changed_fields = Vec::new();
    for change in action
        .raw
        .get("changes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_object)
    {
        let Some(field) = change
            .get("field")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|field| !field.is_empty())
        else {
            continue;
        };
        if !crate::review_diff::is_safe_review_changed_field(field) {
            continue;
        }
        changed_fields.push(field.to_string());
        live.insert(
            field.to_string(),
            change.get("after").cloned().unwrap_or(Value::Null),
        );
        desired.insert(
            field.to_string(),
            change.get("before").cloned().unwrap_or(Value::Null),
        );
    }
    if changed_fields.is_empty() {
        return Vec::new();
    }
    let Ok(model) =
        crate::review_diff::build_review_diff_model(crate::review_diff::ReviewDiffInput {
            title: format!("{} {}", action.resource_kind, action.identity),
            action: action.action.clone(),
            live: Some(&live),
            desired: Some(&desired),
            changed_fields,
        })
    else {
        return Vec::new();
    };
    crate::review_diff::review_diff_model_preview_lines(&model, 4)
}

pub(crate) fn append_review_evidence_section(lines: &mut Vec<String>, review_lines: Vec<String>) {
    if review_lines.is_empty() {
        return;
    }
    lines.push("Review evidence:".to_string());
    lines.extend(review_lines);
}
