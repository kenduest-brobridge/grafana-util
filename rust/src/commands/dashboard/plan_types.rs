//! Shared data types for dashboard reconcile plans.

use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::BTreeSet;
use std::path::PathBuf;

use super::FolderInventoryItem;
use crate::review_contract::{
    build_review_mutation_envelope, review_action_group, review_action_rank,
    review_operation_kind_rank, ReviewMutationAction,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DashboardPlanChange {
    pub(super) field: String,
    pub(super) before: Value,
    pub(super) after: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FolderPermissionPlanDetails {
    pub(super) subject_type: String,
    pub(super) subject_key: String,
    pub(super) subject_name: String,
    pub(super) permission: i64,
    pub(super) permission_name: String,
    pub(super) inherited: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DashboardPlanAction {
    pub(super) action_id: String,
    pub(super) domain: String,
    pub(super) resource_kind: String,
    pub(super) dashboard_uid: String,
    pub(super) title: String,
    pub(super) folder_uid: String,
    pub(super) folder_path: String,
    pub(super) source_org_id: Option<String>,
    pub(super) source_org_name: String,
    pub(super) target_org_id: Option<String>,
    pub(super) target_org_name: String,
    pub(super) match_basis: String,
    pub(super) action: String,
    pub(super) status: String,
    pub(super) changed_fields: Vec<String>,
    pub(super) changes: Vec<DashboardPlanChange>,
    pub(super) source_file: Option<String>,
    pub(super) target_uid: Option<String>,
    pub(super) target_version: Option<i64>,
    pub(super) target_evidence: Vec<String>,
    pub(super) dependency_hints: Vec<String>,
    pub(super) blocked_reason: Option<String>,
    pub(super) review_hints: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) permission: Option<FolderPermissionPlanDetails>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DashboardPlanOrgSummary {
    pub(super) source_org_id: Option<String>,
    pub(super) source_org_name: String,
    pub(super) target_org_id: Option<String>,
    pub(super) target_org_name: String,
    pub(super) org_action: String,
    pub(super) input_dir: String,
    pub(super) checked: usize,
    pub(super) same: usize,
    pub(super) create: usize,
    pub(super) update: usize,
    pub(super) extra: usize,
    pub(super) delete: usize,
    pub(super) blocked: usize,
    pub(super) warning: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DashboardPlanSummary {
    pub(super) checked: usize,
    pub(super) same: usize,
    pub(super) create: usize,
    pub(super) update: usize,
    pub(super) extra: usize,
    pub(super) delete: usize,
    pub(super) blocked: usize,
    pub(super) warning: usize,
    pub(super) org_count: usize,
    pub(super) would_create_org_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DashboardPlanReport {
    pub(super) kind: String,
    #[serde(rename = "schemaVersion")]
    pub(super) schema_version: i64,
    pub(super) tool_version: String,
    pub(super) mode: String,
    pub(super) scope: String,
    pub(super) input_type: String,
    pub(super) prune: bool,
    pub(super) summary: DashboardPlanSummary,
    pub(super) review: Value,
    pub(super) orgs: Vec<DashboardPlanOrgSummary>,
    pub(super) actions: Vec<DashboardPlanAction>,
}

#[derive(Debug, Clone)]
pub(super) struct DashboardPlanReviewActionProjection {
    pub(super) action_id: String,
    pub(super) action: String,
    pub(super) domain: String,
    pub(super) resource_kind: String,
    pub(super) identity: String,
    pub(super) status: String,
    pub(super) order_group: String,
    pub(super) kind_order: usize,
    pub(super) blocked_reason: Option<String>,
    pub(super) details: Option<String>,
    pub(super) review_hints: Vec<String>,
    pub(super) raw: Value,
}

#[derive(Debug, Clone)]
pub(super) struct DashboardPlanReviewProjection {
    pub(super) domains: Vec<String>,
    pub(super) actions: Vec<DashboardPlanReviewActionProjection>,
}

impl DashboardPlanAction {
    fn review_identity(&self) -> String {
        if !self.dashboard_uid.trim().is_empty() {
            self.dashboard_uid.clone()
        } else if !self.title.trim().is_empty() {
            self.title.clone()
        } else {
            self.source_file
                .clone()
                .unwrap_or_else(|| "dashboard".to_string())
        }
    }

    fn to_review_projection(&self) -> DashboardPlanReviewActionProjection {
        let raw = match serde_json::to_value(self) {
            Ok(Value::Object(object)) => Value::Object(object),
            Ok(other) => other,
            Err(_) => Value::Null,
        };
        DashboardPlanReviewActionProjection {
            action_id: self.action_id.clone(),
            action: self.action.clone(),
            domain: self.domain.clone(),
            resource_kind: self.resource_kind.clone(),
            identity: self.review_identity(),
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

impl DashboardPlanReport {
    pub(super) fn build_review_projection(&self) -> DashboardPlanReviewProjection {
        let mut actions = self
            .actions
            .iter()
            .map(DashboardPlanAction::to_review_projection)
            .collect::<Vec<_>>();
        let domains = actions
            .iter()
            .map(|action| action.domain.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        actions.sort_by(|left, right| {
            left.kind_order
                .cmp(&right.kind_order)
                .then_with(|| {
                    review_action_rank(&left.action).cmp(&review_action_rank(&right.action))
                })
                .then_with(|| left.domain.cmp(&right.domain))
                .then_with(|| left.identity.cmp(&right.identity))
                .then_with(|| left.action_id.cmp(&right.action_id))
        });
        DashboardPlanReviewProjection { domains, actions }
    }

    pub(super) fn build_review_envelope(&self) -> Value {
        let projection = self.build_review_projection();
        let domain_refs = projection
            .domains
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let review = build_review_mutation_envelope(
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
            &domain_refs,
        );
        Value::Object(Map::from_iter(vec![
            (
                "actions".to_string(),
                Value::Array(
                    review
                        .actions
                        .into_iter()
                        .map(|action| action.raw)
                        .collect(),
                ),
            ),
            (
                "domains".to_string(),
                Value::Array(
                    review
                        .domains
                        .into_iter()
                        .map(|domain| domain.raw)
                        .collect(),
                ),
            ),
            (
                "blockedReasons".to_string(),
                Value::Array(
                    review
                        .blocked_reasons
                        .into_iter()
                        .map(Value::String)
                        .collect(),
                ),
            ),
            (
                "summary".to_string(),
                Value::Object(Map::from_iter(vec![
                    (
                        "actionCount".to_string(),
                        Value::Number((review.summary.action_count as i64).into()),
                    ),
                    (
                        "domainCount".to_string(),
                        Value::Number((review.summary.domain_count as i64).into()),
                    ),
                    (
                        "sameCount".to_string(),
                        Value::Number((review.summary.same_count as i64).into()),
                    ),
                    (
                        "blockedCount".to_string(),
                        Value::Number((review.summary.blocked_count as i64).into()),
                    ),
                    (
                        "warningCount".to_string(),
                        Value::Number((review.summary.warning_count as i64).into()),
                    ),
                ])),
            ),
        ]))
    }
}

#[derive(Debug, Clone)]
pub(super) struct LocalDashboard {
    pub(super) file_path: String,
    pub(super) dashboard: Value,
    pub(super) dashboard_uid: String,
    pub(super) title: String,
    pub(super) folder_uid: String,
    pub(super) folder_path: String,
}

#[derive(Debug, Clone)]
pub(super) struct LiveDashboard {
    pub(super) uid: String,
    pub(super) title: String,
    pub(super) folder_uid: String,
    pub(super) folder_path: String,
    pub(super) version: Option<i64>,
    pub(super) evidence: Vec<String>,
    pub(super) payload: Value,
}

#[derive(Debug, Clone)]
pub(super) struct PlanLiveState {
    pub(super) live_datasources: Vec<Map<String, Value>>,
    pub(super) live_dashboards: Vec<LiveDashboard>,
    pub(super) live_folders: Vec<FolderInventoryItem>,
}

#[derive(Debug, Clone)]
pub(super) struct OrgPlanInput {
    pub(super) source_org_id: Option<String>,
    pub(super) source_org_name: String,
    pub(super) target_org_id: Option<String>,
    pub(super) target_org_name: String,
    pub(super) org_action: String,
    pub(super) input_dir: PathBuf,
    pub(super) local_dashboards: Vec<LocalDashboard>,
    pub(super) live_dashboards: Vec<LiveDashboard>,
    pub(super) live_datasources: Vec<Map<String, Value>>,
    pub(super) live_folders: Vec<FolderInventoryItem>,
    pub(super) folder_inventory: Vec<FolderInventoryItem>,
    pub(super) folder_permission_resources: Vec<FolderPermissionResource>,
    pub(super) live_folder_permission_resources: Vec<FolderPermissionResource>,
}

#[derive(Debug, Clone)]
pub(super) struct DashboardPlanInput {
    pub(super) scope: String,
    pub(super) input_type: String,
    pub(super) prune: bool,
    pub(super) include_folder_permissions: bool,
    pub(super) folder_permission_match: String,
    pub(super) orgs: Vec<OrgPlanInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FolderPermissionEntry {
    pub(super) subject_type: String,
    pub(super) subject_key: String,
    pub(super) subject_id: String,
    pub(super) subject_name: String,
    pub(super) permission: i64,
    pub(super) permission_name: String,
    pub(super) inherited: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FolderPermissionResource {
    pub(super) uid: String,
    pub(super) title: String,
    pub(super) path: String,
    pub(super) org: String,
    pub(super) org_id: String,
    pub(super) permissions: Vec<FolderPermissionEntry>,
}
