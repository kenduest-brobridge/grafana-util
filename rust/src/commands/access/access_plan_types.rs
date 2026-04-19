//! Access plan output contract types shared by CLI renderers and future TUI callers.

use crate::review_contract::{
    build_review_mutation_envelope, review_action_group, review_operation_kind_rank,
    ReviewMutationAction, ReviewMutationEnvelope,
};
use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessPlanChange {
    pub field: String,
    pub before: Value,
    pub after: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessPlanAction {
    pub action_id: String,
    pub domain: String,
    pub resource_kind: String,
    pub identity: String,
    pub scope: Option<String>,
    pub action: String,
    pub status: String,
    pub changed_fields: Vec<String>,
    pub changes: Vec<AccessPlanChange>,
    pub target: Option<Map<String, Value>>,
    pub blocked_reason: Option<String>,
    pub review_hints: Vec<String>,
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessPlanResourceReport {
    pub resource_kind: String,
    pub source_path: String,
    pub bundle_present: bool,
    pub source_count: usize,
    pub live_count: usize,
    pub checked: usize,
    pub same: usize,
    pub create: usize,
    pub update: usize,
    pub extra_remote: usize,
    pub delete: usize,
    pub blocked: usize,
    pub warning: usize,
    pub scope: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessPlanSummary {
    pub resource_count: usize,
    pub checked: usize,
    pub same: usize,
    pub create: usize,
    pub update: usize,
    pub extra_remote: usize,
    pub delete: usize,
    pub blocked: usize,
    pub warning: usize,
    pub prune: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessPlanDocument {
    pub kind: String,
    pub schema_version: i64,
    pub tool_version: String,
    pub summary: AccessPlanSummary,
    pub resources: Vec<AccessPlanResourceReport>,
    pub actions: Vec<AccessPlanAction>,
}

#[derive(Debug, Clone)]
pub(crate) struct AccessPlanReviewActionProjection {
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

#[derive(Debug, Clone)]
pub(crate) struct AccessPlanReviewProjection {
    pub domains: Vec<&'static str>,
    pub actions: Vec<AccessPlanReviewActionProjection>,
}

impl AccessPlanAction {
    #[cfg_attr(not(test), allow(dead_code))]
    fn to_review_projection(&self) -> AccessPlanReviewActionProjection {
        let raw = match serde_json::to_value(self) {
            Ok(Value::Object(object)) => Value::Object(object),
            Ok(other) => other,
            Err(_) => Value::Null,
        };
        AccessPlanReviewActionProjection {
            action_id: self.action_id.clone(),
            action: self.action.clone(),
            domain: self.domain.clone(),
            resource_kind: self.resource_kind.clone(),
            identity: self.identity.clone(),
            status: self.status.clone(),
            order_group: review_action_group(&self.action).to_string(),
            kind_order: review_operation_kind_rank(&self.domain, &self.action),
            blocked_reason: self.blocked_reason.clone(),
            details: (!self.changed_fields.is_empty())
                .then(|| format!("fields={}", self.changed_fields.join(","))),
            review_hints: self.review_hints.clone(),
            raw,
        }
    }
}

impl AccessPlanDocument {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn build_review_projection(&self) -> AccessPlanReviewProjection {
        AccessPlanReviewProjection {
            domains: vec!["access"],
            actions: self
                .actions
                .iter()
                .map(AccessPlanAction::to_review_projection)
                .collect(),
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn build_review_envelope(&self) -> ReviewMutationEnvelope {
        let projection = self.build_review_projection();
        build_review_mutation_envelope(
            projection
                .actions
                .into_iter()
                .map(|action| ReviewMutationAction {
                    action_id: action.action_id,
                    action: action.action,
                    domain: action.domain,
                    resource_kind: action.resource_kind,
                    identity: action.identity,
                    status: action.status,
                    order_group: action.order_group,
                    kind_order: action.kind_order,
                    blocked_reason: action.blocked_reason,
                    details: action.details,
                    review_hints: action.review_hints,
                    raw: action.raw,
                })
                .collect(),
            &projection.domains,
        )
    }
}
