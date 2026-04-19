//! Internal workspace review view model for preview/review helpers.
//!
//! This adapter normalizes the staged sync preview into a reusable
//! action/domain view without changing the public JSON contract. Callers can
//! reuse the typed fields for future TUI surfaces while still emitting the
//! legacy plan payload shape.

use crate::common::{message, Result};
use crate::review_contract::{
    build_review_mutation_envelope, review_action_group, review_action_rank,
    review_operation_kind_rank, ReviewMutationAction, ReviewMutationEnvelope,
    REVIEW_ACTION_BLOCKED_AMBIGUOUS, REVIEW_ACTION_BLOCKED_MISSING_ORG,
    REVIEW_ACTION_BLOCKED_READ_ONLY, REVIEW_ACTION_BLOCKED_UID_MISMATCH,
    REVIEW_ACTION_EXTRA_REMOTE, REVIEW_ACTION_SAME, REVIEW_ACTION_UNMANAGED,
    REVIEW_ACTION_WOULD_CREATE, REVIEW_ACTION_WOULD_DELETE, REVIEW_ACTION_WOULD_UPDATE,
    REVIEW_STATUS_BLOCKED, REVIEW_STATUS_READY, REVIEW_STATUS_SAME, REVIEW_STATUS_WARNING,
};
use serde_json::{Map, Value};

fn derive_domain(resource_kind: &str) -> String {
    match resource_kind {
        "dashboard" | "datasource" | "folder" => resource_kind.to_string(),
        "alert"
        | "alert-policy"
        | "alert-contact-point"
        | "alert-mute-timing"
        | "alert-template" => "alert".to_string(),
        "user" | "team" | "org" | "service-account" => "access".to_string(),
        other if other.contains("access") => "access".to_string(),
        _ => "workspace".to_string(),
    }
}

fn derive_resource_kind(action: &Map<String, Value>) -> String {
    action
        .get("resourceKind")
        .and_then(Value::as_str)
        .or_else(|| action.get("kind").and_then(Value::as_str))
        .unwrap_or("workspace")
        .to_string()
}

fn derive_identity(action: &Map<String, Value>) -> String {
    action
        .get("identity")
        .and_then(Value::as_str)
        .or_else(|| action.get("uid").and_then(Value::as_str))
        .or_else(|| action.get("name").and_then(Value::as_str))
        .or_else(|| action.get("sourcePath").and_then(Value::as_str))
        .unwrap_or("unknown")
        .to_string()
}

fn derive_action_id(
    action: &Map<String, Value>,
    domain: &str,
    resource_kind: &str,
    identity: &str,
) -> String {
    action
        .get("actionId")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| {
            let kind = if resource_kind.is_empty() {
                "unknown"
            } else {
                resource_kind
            };
            let identity_kind = if action
                .get("uid")
                .and_then(Value::as_str)
                .map(|text| !text.trim().is_empty())
                .unwrap_or(false)
            {
                "uid"
            } else {
                "identity"
            };
            format!("{domain}:{kind}:{identity_kind}:{identity}")
        })
}

fn derive_status(action: &str, existing: Option<&str>) -> String {
    existing
        .map(str::to_string)
        .unwrap_or_else(|| match action {
            REVIEW_ACTION_SAME => REVIEW_STATUS_SAME.to_string(),
            REVIEW_ACTION_WOULD_CREATE
            | REVIEW_ACTION_WOULD_UPDATE
            | REVIEW_ACTION_WOULD_DELETE => REVIEW_STATUS_READY.to_string(),
            REVIEW_ACTION_EXTRA_REMOTE => REVIEW_STATUS_WARNING.to_string(),
            REVIEW_ACTION_BLOCKED_READ_ONLY
            | REVIEW_ACTION_BLOCKED_AMBIGUOUS
            | REVIEW_ACTION_BLOCKED_UID_MISMATCH
            | REVIEW_ACTION_BLOCKED_MISSING_ORG
            | REVIEW_ACTION_UNMANAGED => REVIEW_STATUS_BLOCKED.to_string(),
            _ => REVIEW_STATUS_WARNING.to_string(),
        })
}

pub(crate) type WorkspaceReviewAction = ReviewMutationAction;
pub(crate) type WorkspaceReviewView = ReviewMutationEnvelope;

fn normalize_action(action: &Value) -> Result<WorkspaceReviewAction> {
    let Some(object) = action.as_object() else {
        return Err(message("Workspace preview action is not a JSON object."));
    };
    let mut normalized = object.clone();
    let resource_kind = derive_resource_kind(&normalized);
    let domain = normalized
        .get("domain")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| derive_domain(&resource_kind));
    let identity = derive_identity(&normalized);
    let action_name = normalized
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    if !normalized.contains_key("resourceKind") {
        normalized.insert(
            "resourceKind".to_string(),
            Value::String(resource_kind.clone()),
        );
    }
    if !normalized.contains_key("domain") {
        normalized.insert("domain".to_string(), Value::String(domain.clone()));
    }
    if !normalized.contains_key("kind") {
        normalized.insert("kind".to_string(), Value::String(resource_kind.clone()));
    }
    if !normalized.contains_key("identity") {
        normalized.insert("identity".to_string(), Value::String(identity.clone()));
    }
    let status = derive_status(
        &action_name,
        normalized.get("status").and_then(Value::as_str),
    );
    normalized.insert("status".to_string(), Value::String(status.clone()));
    let action_id = derive_action_id(&normalized, &domain, &resource_kind, &identity);
    if !normalized.contains_key("actionId") {
        normalized.insert("actionId".to_string(), Value::String(action_id.clone()));
    }
    if !normalized.contains_key("orderGroup") {
        normalized.insert(
            "orderGroup".to_string(),
            Value::String(review_action_group(&action_name).to_string()),
        );
    }
    if !normalized.contains_key("kindOrder") {
        normalized.insert(
            "kindOrder".to_string(),
            Value::Number(review_operation_kind_rank(&domain, &action_name).into()),
        );
    }
    if !normalized.contains_key("reviewHints") {
        normalized.insert("reviewHints".to_string(), Value::Array(Vec::new()));
    }
    if !normalized.contains_key("blockedReason") {
        if let Some(reason) = normalized.get("reason").and_then(Value::as_str) {
            normalized.insert(
                "blockedReason".to_string(),
                Value::String(reason.to_string()),
            );
        }
    }
    if !normalized.contains_key("details") {
        if let Some(detail) = normalized.get("detail").and_then(Value::as_str) {
            normalized.insert("details".to_string(), Value::String(detail.to_string()));
        }
    }
    Ok(WorkspaceReviewAction {
        action_id,
        action: action_name,
        domain,
        resource_kind,
        identity,
        status,
        order_group: normalized
            .get("orderGroup")
            .and_then(Value::as_str)
            .unwrap_or("review")
            .to_string(),
        kind_order: normalized
            .get("kindOrder")
            .and_then(Value::as_i64)
            .unwrap_or(0) as usize,
        blocked_reason: normalized
            .get("blockedReason")
            .and_then(Value::as_str)
            .map(str::to_string),
        details: normalized
            .get("details")
            .and_then(Value::as_str)
            .map(str::to_string),
        review_hints: normalized
            .get("reviewHints")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        raw: Value::Object(normalized),
    })
}

fn collect_actions(document: &Map<String, Value>) -> Result<Vec<WorkspaceReviewAction>> {
    let source = document
        .get("actions")
        .or_else(|| document.get("operations"))
        .and_then(Value::as_array)
        .ok_or_else(|| message("Sync plan document is missing actions or operations."))?;
    let mut actions = source
        .iter()
        .map(normalize_action)
        .collect::<Result<Vec<WorkspaceReviewAction>>>()?;
    actions.sort_by(|left, right| {
        left.kind_order
            .cmp(&right.kind_order)
            .then_with(|| review_action_rank(&left.action).cmp(&review_action_rank(&right.action)))
            .then_with(|| left.domain.cmp(&right.domain))
            .then_with(|| left.identity.cmp(&right.identity))
            .then_with(|| left.action_id.cmp(&right.action_id))
    });
    Ok(actions)
}

pub(crate) fn build_workspace_review_view(document: &Value) -> Result<WorkspaceReviewView> {
    let object = document
        .as_object()
        .ok_or_else(|| message("Sync plan document is not a JSON object."))?;
    if object.get("kind").and_then(Value::as_str) != Some("grafana-utils-sync-plan") {
        return Err(message("Sync plan document kind is not supported."));
    }
    let actions = collect_actions(object)?;
    Ok(build_review_mutation_envelope(
        actions,
        &["dashboard", "datasource", "alert", "access"],
    ))
}
