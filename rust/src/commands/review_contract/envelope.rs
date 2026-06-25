use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

use super::actions::{
    create_update_domain_rank, REVIEW_ACTION_SAME, REVIEW_ACTION_WOULD_CREATE,
    REVIEW_ACTION_WOULD_DELETE, REVIEW_ACTION_WOULD_UPDATE, REVIEW_STATUS_BLOCKED,
    REVIEW_STATUS_WARNING,
};
use super::model::{ReviewBlockedReason, ReviewMutationAction};

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
