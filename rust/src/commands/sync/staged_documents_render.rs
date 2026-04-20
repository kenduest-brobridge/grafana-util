//! Sync staged document rendering and drift display helpers.
#![cfg_attr(not(any(feature = "tui", test)), allow(dead_code))]

use crate::alert_sync::ALERT_SYNC_KIND;
use crate::common::{message, Result};
use serde_json::Value;

use super::super::live::load_apply_intent_operations;

#[cfg(feature = "tui")]
use std::cmp::Ordering;

const SYNC_SUMMARY_KIND: &str = "grafana-utils-sync-summary";
const SYNC_PLAN_KIND: &str = "grafana-utils-sync-plan";
const SYNC_APPLY_INTENT_KIND: &str = "grafana-utils-sync-apply-intent";

mod doc_key {
    pub(super) const APPLIED_AT: &str = "appliedAt";
    pub(super) const APPLIED_BY: &str = "appliedBy";
    pub(super) const APPROVAL_REASON: &str = "approvalReason";
    pub(super) const APPROVED: &str = "approved";
    pub(super) const APPLY_NOTE: &str = "applyNote";
    pub(super) const BLOCKED_REASONS: &str = "blockedReasons";
    pub(super) const BUNDLE_PREFLIGHT_SUMMARY: &str = "bundlePreflightSummary";
    pub(super) const DOMAINS: &str = "domains";
    pub(super) const KIND: &str = "kind";
    pub(super) const ORDERING: &str = "ordering";
    pub(super) const PARENT_TRACE_ID: &str = "parentTraceId";
    pub(super) const PREFLIGHT_SUMMARY: &str = "preflightSummary";
    pub(super) const REVIEW_NOTE: &str = "reviewNote";
    pub(super) const REVIEW_REQUIRED: &str = "reviewRequired";
    pub(super) const REVIEWED: &str = "reviewed";
    pub(super) const REVIEWED_AT: &str = "reviewedAt";
    pub(super) const REVIEWED_BY: &str = "reviewedBy";
    pub(super) const STAGE: &str = "stage";
    pub(super) const STEP_INDEX: &str = "stepIndex";
    pub(super) const SUMMARY: &str = "summary";
    pub(super) const TRACE_ID: &str = "traceId";
}

mod summary_key {
    pub(super) const ALERT_ARTIFACT_BLOCKING_COUNT: &str = "alertArtifactBlockingCount";
    pub(super) const ALERT_ARTIFACT_COUNT: &str = "alertArtifactCount";
    pub(super) const ALERT_ARTIFACT_PLAN_ONLY_COUNT: &str = "alertArtifactPlanOnlyCount";
    pub(super) const ALERT_BLOCKED: &str = "alert_blocked";
    pub(super) const ALERT_COUNT: &str = "alertCount";
    pub(super) const ALERT_CANDIDATE: &str = "alert_candidate";
    pub(super) const ALERT_PLAN_ONLY: &str = "alert_plan_only";
    pub(super) const BLOCKED_COUNT: &str = "blockedCount";
    pub(super) const BLOCKED_REASONS: &str = "blocked_reasons";
    pub(super) const BLOCKING_COUNT: &str = "blockingCount";
    pub(super) const CANDIDATE_COUNT: &str = "candidateCount";
    pub(super) const CHECK_COUNT: &str = "checkCount";
    pub(super) const DASHBOARD_COUNT: &str = "dashboardCount";
    pub(super) const DATASOURCE_COUNT: &str = "datasourceCount";
    pub(super) const FOLDER_COUNT: &str = "folderCount";
    pub(super) const MODE: &str = "mode";
    pub(super) const NOOP: &str = "noop";
    pub(super) const OK_COUNT: &str = "okCount";
    pub(super) const PLAN_ONLY_COUNT: &str = "planOnlyCount";
    pub(super) const PROVIDER_BLOCKING_COUNT: &str = "providerBlockingCount";
    pub(super) const RESOURCE_COUNT: &str = "resourceCount";
    pub(super) const SECRET_PLACEHOLDER_BLOCKING_COUNT: &str = "secretPlaceholderBlockingCount";
    pub(super) const SYNC_BLOCKING_COUNT: &str = "syncBlockingCount";
    pub(super) const UNMANAGED: &str = "unmanaged";
    pub(super) const WOULD_CREATE: &str = "would_create";
    pub(super) const WOULD_DELETE: &str = "would_delete";
    pub(super) const WOULD_UPDATE: &str = "would_update";
}

fn is_document_kind(document: &Value, expected_kind: &str) -> bool {
    document.get(doc_key::KIND).and_then(Value::as_str) == Some(expected_kind)
}

fn document_summary<'a>(
    document: &'a Value,
    missing_message: &str,
) -> Result<&'a serde_json::Map<String, Value>> {
    document
        .get(doc_key::SUMMARY)
        .and_then(Value::as_object)
        .ok_or_else(|| message(missing_message))
}

fn number_field(object: &serde_json::Map<String, Value>, key: &str) -> i64 {
    object.get(key).and_then(Value::as_i64).unwrap_or(0)
}

fn bool_field(document: &Value, key: &str) -> bool {
    document.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn string_field<'a>(document: &'a Value, key: &str, fallback: &'a str) -> &'a str {
    document
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or(fallback)
}

pub fn render_sync_summary_text(document: &Value) -> Result<Vec<String>> {
    if !is_document_kind(document, SYNC_SUMMARY_KIND) {
        return Err(message("Sync summary document kind is not supported."));
    }
    let summary = document_summary(document, "Sync summary document is missing summary.")?;
    Ok(vec![
        "Sync summary".to_string(),
        format!(
            "Resources: {} total, {} dashboards, {} datasources, {} folders, {} alerts",
            number_field(summary, summary_key::RESOURCE_COUNT),
            number_field(summary, summary_key::DASHBOARD_COUNT),
            number_field(summary, summary_key::DATASOURCE_COUNT),
            number_field(summary, summary_key::FOLDER_COUNT),
            number_field(summary, summary_key::ALERT_COUNT),
        ),
    ])
}

pub fn render_alert_sync_assessment_text(document: &Value) -> Result<Vec<String>> {
    if !is_document_kind(document, ALERT_SYNC_KIND) {
        return Err(message(
            "Alert sync assessment document kind is not supported.",
        ));
    }
    let summary = document_summary(
        document,
        "Alert sync assessment document is missing summary.",
    )?;
    let mut lines = vec![
        "Alert sync assessment".to_string(),
        format!(
            "Alerts: {} total, {} candidate, {} plan-only, {} blocked",
            number_field(summary, summary_key::ALERT_COUNT),
            number_field(summary, summary_key::CANDIDATE_COUNT),
            number_field(summary, summary_key::PLAN_ONLY_COUNT),
            number_field(summary, summary_key::BLOCKED_COUNT),
        ),
        String::new(),
        "# Alerts".to_string(),
    ];
    if let Some(items) = document.get("alerts").and_then(Value::as_array) {
        for item in items {
            if let Some(object) = item.as_object() {
                lines.push(format!(
                    "- {} status={} liveApplyAllowed={} detail={}",
                    object
                        .get("identity")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown"),
                    object
                        .get("status")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown"),
                    if object
                        .get("liveApplyAllowed")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                    {
                        "true"
                    } else {
                        "false"
                    },
                    object.get("detail").and_then(Value::as_str).unwrap_or(""),
                ));
            }
        }
    }
    Ok(lines)
}

pub fn render_sync_plan_text(document: &Value) -> Result<Vec<String>> {
    if !is_document_kind(document, SYNC_PLAN_KIND) {
        return Err(message("Sync plan document kind is not supported."));
    }
    let summary = document_summary(document, "Sync plan document is missing summary.")?;
    let mut lines = vec![
        "Sync plan".to_string(),
        format!(
            "Trace: {}",
            string_field(document, doc_key::TRACE_ID, "missing")
        ),
        format!(
            "Lineage: stage={} step={} parent={}",
            string_field(document, doc_key::STAGE, "missing"),
            document
                .get(doc_key::STEP_INDEX)
                .and_then(Value::as_i64)
                .unwrap_or(0),
            string_field(document, doc_key::PARENT_TRACE_ID, "none")
        ),
        format!(
            "Summary: create={} update={} delete={} noop={} unmanaged={}",
            number_field(summary, summary_key::WOULD_CREATE),
            number_field(summary, summary_key::WOULD_UPDATE),
            number_field(summary, summary_key::WOULD_DELETE),
            number_field(summary, summary_key::NOOP),
            number_field(summary, summary_key::UNMANAGED),
        ),
        format!(
            "Alerts: candidate={} plan-only={} blocked={}",
            number_field(summary, summary_key::ALERT_CANDIDATE),
            number_field(summary, summary_key::ALERT_PLAN_ONLY),
            number_field(summary, summary_key::ALERT_BLOCKED),
        ),
        "Plan-only alert items stay review-only until this plan is approved and applied."
            .to_string(),
        format!(
            "Review: required={} reviewed={}",
            bool_field(document, doc_key::REVIEW_REQUIRED),
            bool_field(document, doc_key::REVIEWED),
        ),
    ];
    if let Some(domains) = document.get(doc_key::DOMAINS).and_then(Value::as_array) {
        if !domains.is_empty() {
            let domain_text = domains
                .iter()
                .filter_map(Value::as_object)
                .map(|domain| {
                    format!(
                        "{}={}",
                        domain
                            .get("id")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown"),
                        domain.get("checked").and_then(Value::as_i64).unwrap_or(0)
                    )
                })
                .collect::<Vec<String>>()
                .join("  ");
            lines.insert(5, format!("Domains: {domain_text}"));
        }
    }
    if let Some(ordering_mode) = document
        .get(doc_key::ORDERING)
        .and_then(Value::as_object)
        .and_then(|ordering| ordering.get(summary_key::MODE))
        .and_then(Value::as_str)
    {
        lines.insert(5, format!("Ordering: {ordering_mode}"));
    }
    if let Some(reviewed_by) = document.get(doc_key::REVIEWED_BY).and_then(Value::as_str) {
        lines.push(format!("Reviewed by: {reviewed_by}"));
    }
    if let Some(reviewed_at) = document.get(doc_key::REVIEWED_AT).and_then(Value::as_str) {
        lines.push(format!("Reviewed at: {reviewed_at}"));
    }
    if let Some(review_note) = document.get(doc_key::REVIEW_NOTE).and_then(Value::as_str) {
        lines.push(format!("Review note: {review_note}"));
    }
    if let Some(reasons) = document
        .get(doc_key::BLOCKED_REASONS)
        .and_then(Value::as_array)
        .or_else(|| {
            summary
                .get(summary_key::BLOCKED_REASONS)
                .and_then(Value::as_array)
        })
    {
        for reason in reasons.iter().filter_map(Value::as_str) {
            lines.push(format!("Blocked reason: {reason}"));
        }
    }
    Ok(lines)
}

pub fn render_sync_apply_intent_text(document: &Value) -> Result<Vec<String>> {
    if !is_document_kind(document, SYNC_APPLY_INTENT_KIND) {
        return Err(message("Sync apply intent document kind is not supported."));
    }
    let summary = document_summary(document, "Sync apply intent document is missing summary.")?;
    let operations = load_apply_intent_operations(document)?;
    let mut lines = vec![
        "Sync apply intent".to_string(),
        format!(
            "Trace: {}",
            string_field(document, doc_key::TRACE_ID, "missing")
        ),
        format!(
            "Lineage: stage={} step={} parent={}",
            string_field(document, doc_key::STAGE, "missing"),
            document
                .get(doc_key::STEP_INDEX)
                .and_then(Value::as_i64)
                .unwrap_or(0),
            string_field(document, doc_key::PARENT_TRACE_ID, "none")
        ),
        format!(
            "Summary: create={} update={} delete={} executable={}",
            number_field(summary, summary_key::WOULD_CREATE),
            number_field(summary, summary_key::WOULD_UPDATE),
            number_field(summary, summary_key::WOULD_DELETE),
            operations.len(),
        ),
        format!(
            "Review: required={} reviewed={} approved={}",
            bool_field(document, doc_key::REVIEW_REQUIRED),
            bool_field(document, doc_key::REVIEWED),
            bool_field(document, doc_key::APPROVED),
        ),
    ];
    if let Some(preflight_summary) = document
        .get(doc_key::PREFLIGHT_SUMMARY)
        .and_then(Value::as_object)
    {
        lines.push(format!(
            "Input-test: kind={} checks={} ok={} blocking={}",
            preflight_summary
                .get(doc_key::KIND)
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            number_field(preflight_summary, summary_key::CHECK_COUNT),
            number_field(preflight_summary, summary_key::OK_COUNT),
            number_field(preflight_summary, summary_key::BLOCKING_COUNT),
        ));
    }
    if let Some(bundle_summary) = document
        .get(doc_key::BUNDLE_PREFLIGHT_SUMMARY)
        .and_then(Value::as_object)
    {
        lines.push(format!(
            "Package-test: resources={} sync-blocking={} provider-blocking={} secret-placeholder-blocking={} alert-artifacts={} plan-only={} blocking={}",
            number_field(bundle_summary, summary_key::RESOURCE_COUNT),
            number_field(bundle_summary, summary_key::SYNC_BLOCKING_COUNT),
            number_field(bundle_summary, summary_key::PROVIDER_BLOCKING_COUNT),
            number_field(bundle_summary, summary_key::SECRET_PLACEHOLDER_BLOCKING_COUNT),
            number_field(bundle_summary, summary_key::ALERT_ARTIFACT_COUNT),
            number_field(bundle_summary, summary_key::ALERT_ARTIFACT_PLAN_ONLY_COUNT),
            number_field(bundle_summary, summary_key::ALERT_ARTIFACT_BLOCKING_COUNT),
        ));
        lines.push(
            "Reason: input-test and package-test blocking must be 0 before apply; missing provider or secret placeholder availability blocks apply; plan-only alert artifacts stay staged."
                .to_string(),
        );
    }
    if let Some(applied_by) = document.get(doc_key::APPLIED_BY).and_then(Value::as_str) {
        lines.push(format!("Applied by: {applied_by}"));
    }
    if let Some(applied_at) = document.get(doc_key::APPLIED_AT).and_then(Value::as_str) {
        lines.push(format!("Applied at: {applied_at}"));
    }
    if let Some(approval_reason) = document
        .get(doc_key::APPROVAL_REASON)
        .and_then(Value::as_str)
    {
        lines.push(format!("Approval reason: {approval_reason}"));
    }
    if let Some(apply_note) = document.get(doc_key::APPLY_NOTE).and_then(Value::as_str) {
        lines.push(format!("Apply note: {apply_note}"));
    }
    Ok(lines)
}

#[cfg(feature = "tui")]
fn sync_audit_field<'a>(row: &'a Value, key: &str) -> &'a str {
    row.get(key).and_then(Value::as_str).unwrap_or("")
}

#[cfg(feature = "tui")]
fn sync_audit_display<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.is_empty() {
        fallback
    } else {
        value
    }
}

#[cfg(feature = "tui")]
fn sync_audit_status_rank(status: &str) -> u8 {
    match status {
        "missing-live" => 0,
        "missing-lock" => 1,
        "drift-detected" => 2,
        _ => 3,
    }
}

#[cfg(feature = "tui")]
pub(crate) fn sync_audit_drift_cmp(left: &Value, right: &Value) -> Ordering {
    sync_audit_status_rank(sync_audit_field(left, "status"))
        .cmp(&sync_audit_status_rank(sync_audit_field(right, "status")))
        .then_with(|| sync_audit_field(left, "kind").cmp(sync_audit_field(right, "kind")))
        .then_with(|| sync_audit_field(left, "identity").cmp(sync_audit_field(right, "identity")))
        .then_with(|| sync_audit_field(left, "title").cmp(sync_audit_field(right, "title")))
        .then_with(|| {
            sync_audit_field(left, "sourcePath").cmp(sync_audit_field(right, "sourcePath"))
        })
}

#[cfg(feature = "tui")]
pub(crate) fn sync_audit_drift_title(drift: &Value) -> String {
    format!(
        "{} {}",
        sync_audit_display(sync_audit_field(drift, "kind"), "unknown"),
        sync_audit_display(sync_audit_field(drift, "identity"), "unknown"),
    )
}

#[cfg(feature = "tui")]
pub(crate) fn sync_audit_drift_meta(drift: &Value) -> String {
    let baseline_status = sync_audit_display(sync_audit_field(drift, "baselineStatus"), "unknown");
    let current_status = sync_audit_display(sync_audit_field(drift, "currentStatus"), "unknown");
    format!(
        "{} | base={} cur={}",
        sync_audit_display(sync_audit_field(drift, "status"), "unknown"),
        baseline_status,
        current_status
    )
}

#[cfg(feature = "tui")]
pub(crate) fn sync_audit_drift_details(drift: &Value) -> Vec<String> {
    let mut details = vec![
        format!(
            "Triage: {}",
            sync_audit_display(sync_audit_field(drift, "status"), "(unknown)")
        ),
        format!(
            "Baseline/current: {} -> {}",
            sync_audit_display(sync_audit_field(drift, "baselineStatus"), "(unknown)"),
            sync_audit_display(sync_audit_field(drift, "currentStatus"), "(unknown)")
        ),
        format!(
            "Source: {}",
            sync_audit_display(sync_audit_field(drift, "sourcePath"), "(not set)")
        ),
    ];

    let drifted_fields = drift
        .get("driftedFields")
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        .unwrap_or_default();
    details.push(format!(
        "Fields: {}",
        if drifted_fields.is_empty() {
            "none".to_string()
        } else {
            drifted_fields.join(", ")
        }
    ));
    let baseline_checksum = drift
        .get("baselineChecksum")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or("(none)");
    let current_checksum = drift
        .get("currentChecksum")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or("(none)");
    if baseline_checksum != "(none)" || current_checksum != "(none)" {
        details.push(format!(
            "Checksums: baseline={} current={}",
            baseline_checksum, current_checksum
        ));
    }
    details
}
