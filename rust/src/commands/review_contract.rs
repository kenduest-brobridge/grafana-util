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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn review_apply_result_preserves_common_evidence_shape_with_domain_fields() {
        let mut result = ReviewApplyResult::new("apply");
        result.push_result(review_apply_result_entry(
            "grafana-alert-rule",
            "cpu-high",
            "create",
            json!({"uid": "cpu-high"}),
        ));

        let document = result.into_value_with_fields([
            ("kind", json!("grafana-util-alert-apply-result")),
            ("allowPolicyReset", json!(false)),
        ]);

        assert_eq!(
            document,
            json!({
                "kind": "grafana-util-alert-apply-result",
                "mode": "apply",
                "allowPolicyReset": false,
                "appliedCount": 1,
                "results": [{
                    "kind": "grafana-alert-rule",
                    "identity": "cpu-high",
                    "action": "create",
                    "response": {"uid": "cpu-high"}
                }]
            })
        );
    }
}
