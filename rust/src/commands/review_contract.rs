//! Shared review/action contract vocabulary.
//!
//! Keep machine-readable action and status strings centralized so plan, preview,
//! apply, and TUI layers do not drift when comparing the same review contract.

use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const REVIEW_ACTION_BLOCKED: &str = "blocked";
pub(crate) const REVIEW_ACTION_BLOCKED_AMBIGUOUS: &str = "blocked-ambiguous";
pub(crate) const REVIEW_ACTION_BLOCKED_MISSING_ORG: &str = "blocked-missing-org";
pub(crate) const REVIEW_ACTION_BLOCKED_READ_ONLY: &str = "blocked-read-only";
pub(crate) const REVIEW_ACTION_BLOCKED_TARGET: &str = "blocked-target";
pub(crate) const REVIEW_ACTION_BLOCKED_UID_MISMATCH: &str = "blocked-uid-mismatch";
pub(crate) const REVIEW_ACTION_EXTRA_REMOTE: &str = "extra-remote";
pub(crate) const REVIEW_ACTION_SAME: &str = "same";
pub(crate) const REVIEW_ACTION_UNMANAGED: &str = "unmanaged";
pub(crate) const REVIEW_ACTION_WOULD_CREATE: &str = "would-create";
pub(crate) const REVIEW_ACTION_WOULD_DELETE: &str = "would-delete";
pub(crate) const REVIEW_ACTION_WOULD_UPDATE: &str = "would-update";

pub(crate) const REVIEW_STATUS_BLOCKED: &str = "blocked";
pub(crate) const REVIEW_STATUS_READY: &str = "ready";
pub(crate) const REVIEW_STATUS_SAME: &str = "same";
pub(crate) const REVIEW_STATUS_WARNING: &str = "warning";

pub(crate) const REVIEW_REASON_AMBIGUOUS_LIVE_NAME_MATCH: &str = "ambiguous-live-name-match";
pub(crate) const REVIEW_REASON_TARGET_ORG_MISSING: &str = "target-org-missing";
pub(crate) const REVIEW_REASON_TARGET_PROVISIONED_OR_MANAGED: &str =
    "target-provisioned-or-managed";
pub(crate) const REVIEW_REASON_TARGET_READ_ONLY: &str = "target-read-only";
pub(crate) const REVIEW_REASON_UID_NAME_MISMATCH: &str = "uid-name-mismatch";

pub(crate) const REVIEW_HINT_MISSING_REMOTE: &str = "missing-remote";
pub(crate) const REVIEW_HINT_REMOTE_ONLY: &str = "remote-only";
pub(crate) const REVIEW_HINT_REQUIRES_SECRET_VALUES: &str = "requires-secret-values";

pub(crate) fn is_review_apply_action(action: &str) -> bool {
    matches!(
        action,
        REVIEW_ACTION_WOULD_CREATE | REVIEW_ACTION_WOULD_UPDATE | REVIEW_ACTION_WOULD_DELETE
    )
}

pub(crate) fn is_review_blocked_action(action: &str) -> bool {
    action.starts_with("blocked-")
        || action == REVIEW_ACTION_BLOCKED
        || action == REVIEW_ACTION_UNMANAGED
}

pub(crate) fn review_action_rank(action: &str) -> usize {
    match action {
        REVIEW_ACTION_WOULD_CREATE => 0,
        REVIEW_ACTION_WOULD_UPDATE => 1,
        REVIEW_ACTION_WOULD_DELETE => 2,
        REVIEW_ACTION_SAME => 3,
        REVIEW_ACTION_EXTRA_REMOTE => 4,
        REVIEW_ACTION_UNMANAGED => 5,
        _ => 6,
    }
}

fn create_update_domain_rank(domain: &str) -> usize {
    match domain {
        "folder" => 0,
        "datasource" => 1,
        "dashboard" => 2,
        "alert" => 3,
        "access" => 4,
        _ => 5,
    }
}

fn delete_domain_rank(domain: &str) -> usize {
    match domain {
        "alert" => 0,
        "dashboard" => 1,
        "datasource" => 2,
        "folder" | "access" => 3,
        _ => 4,
    }
}

pub(crate) fn review_operation_kind_rank(domain: &str, action: &str) -> usize {
    if action == REVIEW_ACTION_WOULD_DELETE {
        delete_domain_rank(domain)
    } else {
        create_update_domain_rank(domain)
    }
}

pub(crate) fn review_action_group(action: &str) -> &'static str {
    match action {
        REVIEW_ACTION_WOULD_DELETE => "delete",
        REVIEW_ACTION_WOULD_CREATE | REVIEW_ACTION_WOULD_UPDATE => "create-update",
        REVIEW_ACTION_SAME => "read-only",
        REVIEW_ACTION_EXTRA_REMOTE => REVIEW_STATUS_WARNING,
        REVIEW_ACTION_UNMANAGED => REVIEW_STATUS_BLOCKED,
        _ => "review",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewMutationAction {
    pub action_id: String,
    pub action: String,
    pub domain: String,
    pub resource_kind: String,
    pub identity: String,
    pub status: String,
    pub order_group: String,
    pub kind_order: usize,
    pub blocked_reason: Option<String>,
    pub details: Option<String>,
    pub review_hints: Vec<String>,
    pub raw: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ReviewBlockedReason(String);

impl ReviewBlockedReason {
    pub(crate) fn from_optional_text(reason: Option<&str>) -> Option<Self> {
        reason.and_then(Self::from_text)
    }

    pub(crate) fn from_text(reason: &str) -> Option<Self> {
        let normalized = reason.trim();
        if normalized.is_empty() {
            None
        } else {
            Some(Self(normalized.to_string()))
        }
    }

    pub(crate) fn from_action_fields(
        status: &str,
        action: &str,
        blocked_reason: Option<&str>,
        raw: &Value,
    ) -> Option<Self> {
        if status != REVIEW_STATUS_BLOCKED && !is_review_blocked_action(action) {
            return None;
        }
        Self::from_optional_text(blocked_reason).or_else(|| {
            raw.get("reason")
                .and_then(Value::as_str)
                .and_then(Self::from_text)
        })
    }

    pub(crate) fn into_string(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewMutationActionInput {
    pub action_id: String,
    pub action: String,
    pub domain: String,
    pub resource_kind: String,
    pub identity: String,
    pub status: String,
    pub blocked_reason: Option<String>,
    pub details: Option<String>,
    pub review_hints: Vec<String>,
    pub raw: Value,
}

impl From<ReviewMutationActionInput> for ReviewMutationAction {
    fn from(input: ReviewMutationActionInput) -> Self {
        let order_group = review_action_group(&input.action).to_string();
        let kind_order = review_operation_kind_rank(&input.domain, &input.action);
        ReviewMutationAction {
            action_id: input.action_id,
            action: input.action,
            domain: input.domain,
            resource_kind: input.resource_kind,
            identity: input.identity,
            status: input.status,
            order_group,
            kind_order,
            blocked_reason: input.blocked_reason,
            details: input.details,
            review_hints: input.review_hints,
            raw: input.raw,
        }
    }
}

#[cfg(any(feature = "tui", test))]
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

#[cfg(any(feature = "tui", test))]
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

#[cfg(any(feature = "tui", test))]
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

#[cfg(any(feature = "tui", test))]
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

#[cfg(any(feature = "tui", test))]
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

#[cfg(any(feature = "tui", test))]
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

#[cfg(any(feature = "tui", test))]
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

#[cfg(any(feature = "tui", test))]
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

#[cfg(any(feature = "tui", test))]
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

#[cfg(any(feature = "tui", test))]
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

#[cfg(any(feature = "tui", test))]
pub(crate) fn append_review_evidence_section(lines: &mut Vec<String>, review_lines: Vec<String>) {
    if review_lines.is_empty() {
        return;
    }
    lines.push("Review evidence:".to_string());
    lines.extend(review_lines);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewMutationDomain {
    pub id: String,
    pub checked: usize,
    pub same: usize,
    pub create: usize,
    pub update: usize,
    pub delete: usize,
    pub warning: usize,
    pub blocked: usize,
    pub action_count: usize,
    pub raw: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewMutationSummary {
    pub action_count: usize,
    pub domain_count: usize,
    pub same_count: usize,
    pub blocked_count: usize,
    pub warning_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewMutationEnvelope {
    pub actions: Vec<ReviewMutationAction>,
    pub domains: Vec<ReviewMutationDomain>,
    pub blocked_reasons: Vec<String>,
    pub summary: ReviewMutationSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewApplyResult {
    pub mode: String,
    pub results: Vec<Value>,
}

impl ReviewApplyResult {
    pub(crate) fn new(mode: impl Into<String>) -> Self {
        Self {
            mode: mode.into(),
            results: Vec::new(),
        }
    }

    pub(crate) fn from_results(mode: impl Into<String>, results: Vec<Value>) -> Self {
        Self {
            mode: mode.into(),
            results,
        }
    }

    pub(crate) fn push_result(&mut self, result: Value) {
        self.results.push(result);
    }

    pub(crate) fn into_value(self) -> Value {
        let extra_fields: [(String, Value); 0] = [];
        self.into_value_with_fields(extra_fields)
    }

    pub(crate) fn into_value_with_fields<K: Into<String>, const N: usize>(
        self,
        extra_fields: [(K, Value); N],
    ) -> Value {
        let mut object = Map::new();
        for (key, value) in extra_fields {
            object.insert(key.into(), value);
        }
        object.insert("mode".to_string(), Value::String(self.mode));
        object.insert(
            "appliedCount".to_string(),
            Value::Number((self.results.len() as i64).into()),
        );
        object.insert("results".to_string(), Value::Array(self.results));
        Value::Object(object)
    }
}

pub(crate) fn review_apply_result_entry(
    kind: impl Into<String>,
    identity: impl Into<String>,
    action: impl Into<String>,
    response: Value,
) -> Value {
    Value::Object(Map::from_iter(vec![
        ("kind".to_string(), Value::String(kind.into())),
        ("identity".to_string(), Value::String(identity.into())),
        ("action".to_string(), Value::String(action.into())),
        ("response".to_string(), response),
    ]))
}

fn collect_blocked_reasons(actions: &[ReviewMutationAction]) -> Vec<String> {
    let mut reasons = BTreeSet::new();
    for action in actions {
        if let Some(reason) = ReviewBlockedReason::from_action_fields(
            &action.status,
            &action.action,
            action.blocked_reason.as_deref(),
            &action.raw,
        ) {
            reasons.insert(reason.into_string());
        }
    }
    reasons.into_iter().take(5).collect()
}

fn summarize_review_domains(
    actions: &[ReviewMutationAction],
    expected_domains: &[&str],
) -> Vec<ReviewMutationDomain> {
    let mut grouped: BTreeMap<String, Vec<&ReviewMutationAction>> = BTreeMap::new();
    for action in actions {
        grouped
            .entry(action.domain.clone())
            .or_default()
            .push(action);
    }
    let mut domains = grouped
        .into_iter()
        .map(|(domain, items)| {
            let checked = items.len();
            let same = items
                .iter()
                .filter(|item| item.action == REVIEW_ACTION_SAME)
                .count();
            let create = items
                .iter()
                .filter(|item| item.action == REVIEW_ACTION_WOULD_CREATE)
                .count();
            let update = items
                .iter()
                .filter(|item| item.action == REVIEW_ACTION_WOULD_UPDATE)
                .count();
            let delete = items
                .iter()
                .filter(|item| item.action == REVIEW_ACTION_WOULD_DELETE)
                .count();
            let warning = items
                .iter()
                .filter(|item| item.status == REVIEW_STATUS_WARNING)
                .count();
            let blocked = items
                .iter()
                .filter(|item| item.status == REVIEW_STATUS_BLOCKED)
                .count();
            let raw = Value::Object(Map::from_iter(vec![
                ("id".to_string(), Value::String(domain.clone())),
                (
                    "checked".to_string(),
                    Value::Number((checked as i64).into()),
                ),
                (
                    REVIEW_ACTION_SAME.to_string(),
                    Value::Number((same as i64).into()),
                ),
                ("create".to_string(), Value::Number((create as i64).into())),
                ("update".to_string(), Value::Number((update as i64).into())),
                ("delete".to_string(), Value::Number((delete as i64).into())),
                (
                    REVIEW_STATUS_WARNING.to_string(),
                    Value::Number((warning as i64).into()),
                ),
                (
                    REVIEW_STATUS_BLOCKED.to_string(),
                    Value::Number((blocked as i64).into()),
                ),
                (
                    "actionCount".to_string(),
                    Value::Number((checked as i64).into()),
                ),
            ]));
            ReviewMutationDomain {
                id: domain,
                checked,
                same,
                create,
                update,
                delete,
                warning,
                blocked,
                action_count: checked,
                raw,
            }
        })
        .collect::<Vec<_>>();
    for domain in expected_domains {
        if domains.iter().any(|value| value.id == *domain) {
            continue;
        }
        domains.push(ReviewMutationDomain {
            id: (*domain).to_string(),
            checked: 0,
            same: 0,
            create: 0,
            update: 0,
            delete: 0,
            warning: 0,
            blocked: 0,
            action_count: 0,
            raw: Value::Object(Map::from_iter(vec![
                ("id".to_string(), Value::String((*domain).to_string())),
                ("checked".to_string(), Value::Number(0.into())),
                (REVIEW_ACTION_SAME.to_string(), Value::Number(0.into())),
                ("create".to_string(), Value::Number(0.into())),
                ("update".to_string(), Value::Number(0.into())),
                ("delete".to_string(), Value::Number(0.into())),
                (REVIEW_STATUS_WARNING.to_string(), Value::Number(0.into())),
                (REVIEW_STATUS_BLOCKED.to_string(), Value::Number(0.into())),
                ("actionCount".to_string(), Value::Number(0.into())),
            ])),
        });
    }
    domains.sort_by(|left, right| {
        create_update_domain_rank(left.id.as_str())
            .cmp(&create_update_domain_rank(right.id.as_str()))
    });
    domains
}

pub(crate) fn build_review_mutation_envelope(
    actions: Vec<ReviewMutationAction>,
    expected_domains: &[&str],
) -> ReviewMutationEnvelope {
    let domains = summarize_review_domains(&actions, expected_domains);
    let blocked_reasons = collect_blocked_reasons(&actions);
    let summary = ReviewMutationSummary {
        action_count: actions.len(),
        domain_count: domains.len(),
        same_count: actions
            .iter()
            .filter(|action| action.action == REVIEW_ACTION_SAME)
            .count(),
        blocked_count: actions
            .iter()
            .filter(|action| action.status == REVIEW_STATUS_BLOCKED)
            .count(),
        warning_count: actions
            .iter()
            .filter(|action| action.status == REVIEW_STATUS_WARNING)
            .count(),
    };
    ReviewMutationEnvelope {
        actions,
        domains,
        blocked_reasons,
        summary,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewMutationSummaryRow {
    pub domain: String,
    pub resource_kind: String,
    pub identity: String,
    pub action: String,
    pub status: String,
    pub details: Option<String>,
    pub action_count: usize,
    pub domain_count: usize,
    pub blocked_count: usize,
    pub warning_count: usize,
    pub blocked_reasons: Vec<String>,
}

#[allow(dead_code)]
pub(crate) fn build_review_mutation_summary_rows(
    envelope: &ReviewMutationEnvelope,
) -> Vec<ReviewMutationSummaryRow> {
    let mut rows = envelope
        .actions
        .iter()
        .map(|action| ReviewMutationSummaryRow {
            domain: action.domain.clone(),
            resource_kind: action.resource_kind.clone(),
            identity: action.identity.clone(),
            action: action.action.clone(),
            status: action.status.clone(),
            details: action.details.clone(),
            action_count: envelope.summary.action_count,
            domain_count: envelope.summary.domain_count,
            blocked_count: envelope.summary.blocked_count,
            warning_count: envelope.summary.warning_count,
            blocked_reasons: envelope.blocked_reasons.clone(),
        })
        .collect::<Vec<_>>();
    if rows.is_empty() {
        rows.push(ReviewMutationSummaryRow {
            domain: String::new(),
            resource_kind: String::new(),
            identity: String::new(),
            action: String::new(),
            status: String::new(),
            details: None,
            action_count: envelope.summary.action_count,
            domain_count: envelope.summary.domain_count,
            blocked_count: envelope.summary.blocked_count,
            warning_count: envelope.summary.warning_count,
            blocked_reasons: envelope.blocked_reasons.clone(),
        });
    }
    rows
}

#[cfg(test)]
#[path = "review_contract_tests.rs"]
mod tests;
