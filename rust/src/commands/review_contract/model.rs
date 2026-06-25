use serde_json::Value;

use super::actions::{
    is_review_blocked_action, review_action_group, review_operation_kind_rank,
    REVIEW_STATUS_BLOCKED,
};

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
