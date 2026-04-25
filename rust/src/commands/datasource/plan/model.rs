use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;

use super::super::DatasourceImportRecord;
use crate::review_contract::{
    build_review_mutation_envelope, review_action_rank, ReviewBlockedReason, ReviewMutationAction,
    ReviewMutationActionInput, ReviewMutationEnvelope,
};
pub(crate) use crate::review_contract::{
    REVIEW_ACTION_BLOCKED_AMBIGUOUS as PLAN_ACTION_BLOCKED_AMBIGUOUS,
    REVIEW_ACTION_BLOCKED_MISSING_ORG as PLAN_ACTION_BLOCKED_MISSING_ORG,
    REVIEW_ACTION_BLOCKED_READ_ONLY as PLAN_ACTION_BLOCKED_READ_ONLY,
    REVIEW_ACTION_BLOCKED_UID_MISMATCH as PLAN_ACTION_BLOCKED_UID_MISMATCH,
    REVIEW_ACTION_EXTRA_REMOTE as PLAN_ACTION_EXTRA_REMOTE, REVIEW_ACTION_SAME as PLAN_ACTION_SAME,
    REVIEW_ACTION_WOULD_CREATE as PLAN_ACTION_WOULD_CREATE,
    REVIEW_ACTION_WOULD_DELETE as PLAN_ACTION_WOULD_DELETE,
    REVIEW_ACTION_WOULD_UPDATE as PLAN_ACTION_WOULD_UPDATE,
    REVIEW_HINT_MISSING_REMOTE as PLAN_HINT_MISSING_REMOTE,
    REVIEW_HINT_REMOTE_ONLY as PLAN_HINT_REMOTE_ONLY,
    REVIEW_HINT_REQUIRES_SECRET_VALUES as PLAN_HINT_REQUIRES_SECRET_VALUES,
    REVIEW_REASON_AMBIGUOUS_LIVE_NAME_MATCH as PLAN_REASON_AMBIGUOUS_LIVE_NAME_MATCH,
    REVIEW_REASON_TARGET_ORG_MISSING as PLAN_REASON_TARGET_ORG_MISSING,
    REVIEW_REASON_TARGET_READ_ONLY as PLAN_REASON_TARGET_READ_ONLY,
    REVIEW_REASON_UID_NAME_MISMATCH as PLAN_REASON_UID_NAME_MISMATCH,
    REVIEW_STATUS_BLOCKED as PLAN_STATUS_BLOCKED, REVIEW_STATUS_READY as PLAN_STATUS_READY,
    REVIEW_STATUS_SAME as PLAN_STATUS_SAME, REVIEW_STATUS_WARNING as PLAN_STATUS_WARNING,
};

#[derive(Debug, Clone)]
pub(crate) struct DatasourcePlanOrgInput {
    pub(crate) source_org_id: String,
    pub(crate) source_org_name: String,
    pub(crate) target_org_id: Option<String>,
    pub(crate) target_org_name: String,
    pub(crate) org_action: String,
    pub(crate) input_dir: PathBuf,
    pub(crate) records: Vec<DatasourceImportRecord>,
    pub(crate) live: Vec<serde_json::Map<String, Value>>,
}

#[derive(Debug, Clone)]
pub(crate) struct DatasourcePlanInput {
    pub(crate) scope: String,
    pub(crate) input_format: String,
    pub(crate) prune: bool,
    pub(crate) orgs: Vec<DatasourcePlanOrgInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasourcePlanChange {
    pub(crate) field: String,
    pub(crate) before: Value,
    pub(crate) after: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasourcePlanAction {
    pub(crate) action_id: String,
    pub(crate) domain: String,
    pub(crate) resource_kind: String,
    pub(crate) uid: String,
    pub(crate) name: String,
    #[serde(rename = "type")]
    pub(crate) datasource_type: String,
    pub(crate) source_org_id: Option<String>,
    pub(crate) target_org_id: Option<String>,
    pub(crate) match_basis: String,
    pub(crate) action: String,
    pub(crate) status: String,
    pub(crate) changed_fields: Vec<String>,
    pub(crate) changes: Vec<DatasourcePlanChange>,
    pub(crate) source_file: Option<String>,
    pub(crate) target_uid: Option<String>,
    pub(crate) target_version: Option<i64>,
    pub(crate) target_read_only: Option<bool>,
    pub(crate) blocked_reason: Option<String>,
    pub(crate) review_hints: Vec<String>,
    pub(crate) requires_secret_values: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasourcePlanOrgSummary {
    pub(crate) source_org_id: Option<String>,
    pub(crate) source_org_name: String,
    pub(crate) target_org_id: Option<String>,
    pub(crate) target_org_name: String,
    pub(crate) org_action: String,
    pub(crate) input_dir: String,
    pub(crate) checked: usize,
    pub(crate) same: usize,
    pub(crate) create: usize,
    pub(crate) update: usize,
    pub(crate) extra: usize,
    pub(crate) delete: usize,
    pub(crate) blocked: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasourcePlanSummary {
    pub(crate) checked: usize,
    pub(crate) same: usize,
    pub(crate) create: usize,
    pub(crate) update: usize,
    pub(crate) extra: usize,
    pub(crate) delete: usize,
    pub(crate) blocked: usize,
    pub(crate) warning: usize,
    pub(crate) org_count: usize,
    pub(crate) would_create_org_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasourcePlanReport {
    pub(crate) kind: String,
    pub(crate) schema_version: i64,
    pub(crate) tool_version: String,
    pub(crate) mode: String,
    pub(crate) scope: String,
    pub(crate) input_format: String,
    pub(crate) prune: bool,
    pub(crate) summary: DatasourcePlanSummary,
    pub(crate) orgs: Vec<DatasourcePlanOrgSummary>,
    pub(crate) actions: Vec<DatasourcePlanAction>,
}

#[derive(Debug, Clone)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DatasourcePlanReviewProjection {
    pub(crate) domains: Vec<&'static str>,
    pub(crate) actions: Vec<ReviewMutationAction>,
}

impl DatasourcePlanAction {
    #[cfg_attr(not(test), allow(dead_code))]
    fn review_identity(&self) -> String {
        if !self.uid.trim().is_empty() {
            self.uid.clone()
        } else if !self.name.trim().is_empty() {
            self.name.clone()
        } else {
            self.action_id.clone()
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn to_review_projection(&self) -> ReviewMutationAction {
        let raw = match serde_json::to_value(self) {
            Ok(Value::Object(object)) => Value::Object(object),
            Ok(other) => other,
            Err(_) => Value::Null,
        };
        ReviewMutationActionInput {
            action_id: self.action_id.clone(),
            action: self.action.clone(),
            domain: self.domain.clone(),
            resource_kind: self.resource_kind.clone(),
            identity: self.review_identity(),
            status: self.status.clone(),
            blocked_reason: ReviewBlockedReason::from_optional_text(self.blocked_reason.as_deref())
                .map(ReviewBlockedReason::into_string),
            details: (!self.changed_fields.is_empty())
                .then(|| format!("fields={}", self.changed_fields.join(","))),
            review_hints: self.review_hints.clone(),
            raw,
        }
        .into()
    }
}

impl DatasourcePlanReport {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn build_review_projection(&self) -> DatasourcePlanReviewProjection {
        let mut actions = self
            .actions
            .iter()
            .map(DatasourcePlanAction::to_review_projection)
            .collect::<Vec<_>>();
        actions.sort_by(|left, right| {
            left.kind_order
                .cmp(&right.kind_order)
                .then_with(|| {
                    review_action_rank(&left.action).cmp(&review_action_rank(&right.action))
                })
                .then_with(|| left.identity.cmp(&right.identity))
                .then_with(|| left.action_id.cmp(&right.action_id))
        });
        DatasourcePlanReviewProjection {
            domains: vec!["datasource"],
            actions,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn build_review_envelope(&self) -> ReviewMutationEnvelope {
        let projection = self.build_review_projection();
        build_review_mutation_envelope(projection.actions, &projection.domains)
    }
}
