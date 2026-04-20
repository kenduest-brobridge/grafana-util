//! Live sync domain-status producer.
//!
//! Maintainer note:
//! - This module derives one sync-owned domain-status row from staged sync
//!   summary and package-test surfaces.
//! - Keep the producer conservative: package-test data is preferred when it
//!   exists, staged summary data is only a fallback, and missing surfaces stay
//!   explicit.

#![allow(dead_code)]

use serde_json::Value;

use crate::project_status::{
    status_finding, ProjectDomainStatus, PROJECT_STATUS_BLOCKED, PROJECT_STATUS_PARTIAL,
    PROJECT_STATUS_READY,
};
use crate::project_status_model::StatusReading;

use super::project_status_json::summary_number;

const SYNC_DOMAIN_ID: &str = "sync";
const SYNC_SCOPE: &str = "live";
const SYNC_MODE: &str = "live-sync-surfaces";

const SYNC_REASON_READY: &str = PROJECT_STATUS_READY;
const SYNC_REASON_PARTIAL_NO_DATA: &str = "partial-no-data";
const SYNC_REASON_PARTIAL_MISSING_SURFACES: &str = "partial-missing-surfaces";
const SYNC_REASON_BLOCKED_BY_BLOCKERS: &str = "blocked-by-blockers";

const SYNC_SOURCE_KIND_SUMMARY: &str = "sync-summary";
const SYNC_SOURCE_KIND_BUNDLE_PREFLIGHT: &str = "package-test";

mod summary_key {
    pub(super) const RESOURCE_COUNT: &str = "resourceCount";
    pub(super) const SYNC_BLOCKING_COUNT: &str = "syncBlockingCount";
    pub(super) const PROVIDER_BLOCKING_COUNT: &str = "providerBlockingCount";
    pub(super) const SECRET_PLACEHOLDER_BLOCKING_COUNT: &str = "secretPlaceholderBlockingCount";
    pub(super) const ALERT_ARTIFACT_BLOCKED_COUNT: &str = "alertArtifactBlockedCount";
    pub(super) const ALERT_ARTIFACT_PLAN_ONLY_COUNT: &str = "alertArtifactPlanOnlyCount";
    pub(super) const BLOCKED_COUNT: &str = "blockedCount";
    pub(super) const PLAN_ONLY_COUNT: &str = "planOnlyCount";
}

mod signal {
    pub(super) const SUMMARY_RESOURCE_COUNT: &str = "summary.resourceCount";
    pub(super) const SUMMARY_SYNC_BLOCKING_COUNT: &str = "summary.syncBlockingCount";
    pub(super) const SUMMARY_PROVIDER_BLOCKING_COUNT: &str = "summary.providerBlockingCount";
    pub(super) const SUMMARY_SECRET_PLACEHOLDER_BLOCKING_COUNT: &str =
        "summary.secretPlaceholderBlockingCount";
    pub(super) const SUMMARY_ALERT_ARTIFACT_COUNT: &str = "summary.alertArtifactCount";
    pub(super) const SUMMARY_ALERT_ARTIFACT_BLOCKED_COUNT: &str =
        "summary.alertArtifactBlockedCount";
    pub(super) const SUMMARY_ALERT_ARTIFACT_PLAN_ONLY_COUNT: &str =
        "summary.alertArtifactPlanOnlyCount";
    pub(super) const SUMMARY_BLOCKED_COUNT: &str = "summary.blockedCount";
    pub(super) const SUMMARY_PLAN_ONLY_COUNT: &str = "summary.planOnlyCount";
}

const SYNC_SIGNAL_KEYS_SUMMARY: &[&str] = &[signal::SUMMARY_RESOURCE_COUNT];
const SYNC_SIGNAL_KEYS_BUNDLE_PREFLIGHT: &[&str] = &[
    signal::SUMMARY_RESOURCE_COUNT,
    signal::SUMMARY_SYNC_BLOCKING_COUNT,
    signal::SUMMARY_PROVIDER_BLOCKING_COUNT,
    signal::SUMMARY_SECRET_PLACEHOLDER_BLOCKING_COUNT,
    signal::SUMMARY_ALERT_ARTIFACT_COUNT,
    signal::SUMMARY_ALERT_ARTIFACT_BLOCKED_COUNT,
    signal::SUMMARY_ALERT_ARTIFACT_PLAN_ONLY_COUNT,
    signal::SUMMARY_BLOCKED_COUNT,
    signal::SUMMARY_PLAN_ONLY_COUNT,
];

const SYNC_BLOCKER_SYNC_BLOCKING: &str = "sync-blocking";
const SYNC_BLOCKER_PROVIDER_BLOCKING: &str = "provider-blocking";
const SYNC_BLOCKER_SECRET_PLACEHOLDER_BLOCKING: &str = "secret-placeholder-blocking";
const SYNC_BLOCKER_ALERT_ARTIFACT_BLOCKING: &str = "alert-artifact-blocking";
const SYNC_BLOCKER_BUNDLE_BLOCKING: &str = "bundle-blocking";
const SYNC_WARNING_ALERT_ARTIFACT_PLAN_ONLY: &str = "alert-artifact-plan-only";
const SYNC_WARNING_BUNDLE_PLAN_ONLY: &str = "bundle-plan-only";

const SYNC_RESOLVE_BLOCKERS_ACTIONS: &[&str] = &[
    "resolve sync workflow blockers in the fixed order: sync, provider, secret-placeholder, alert-artifact",
];
const SYNC_STAGE_AT_LEAST_ONE_ACTIONS: &[&str] =
    &["stage at least one dashboard, datasource, or alert resource"];
const SYNC_PROVIDE_PREFLIGHT_ACTIONS: &[&str] =
    &["provide a staged package-test document before interpreting live sync readiness"];
const SYNC_REVIEW_NON_BLOCKING_ACTIONS: &[&str] =
    &["review non-blocking sync findings before promotion or apply"];
const SYNC_REEXPORT_AFTER_CHANGES_ACTIONS: &[&str] = &["re-run sync summary after staged changes"];

#[derive(Debug, Clone, Default)]
pub struct SyncLiveProjectStatusInputs<'a> {
    pub summary_document: Option<&'a Value>,
    pub bundle_preflight_document: Option<&'a Value>,
}

fn next_actions_for_partial(resources: usize, has_bundle_preflight: bool) -> Vec<String> {
    if resources == 0 {
        SYNC_STAGE_AT_LEAST_ONE_ACTIONS
            .iter()
            .map(|item| (*item).to_string())
            .collect()
    } else if has_bundle_preflight {
        SYNC_REEXPORT_AFTER_CHANGES_ACTIONS
            .iter()
            .map(|item| (*item).to_string())
            .collect()
    } else {
        SYNC_PROVIDE_PREFLIGHT_ACTIONS
            .iter()
            .map(|item| (*item).to_string())
            .collect()
    }
}

fn next_actions_for_ready(warnings_present: bool) -> Vec<String> {
    if warnings_present {
        SYNC_REVIEW_NON_BLOCKING_ACTIONS
            .iter()
            .map(|item| (*item).to_string())
            .collect()
    } else {
        SYNC_REEXPORT_AFTER_CHANGES_ACTIONS
            .iter()
            .map(|item| (*item).to_string())
            .collect()
    }
}

pub(crate) fn build_live_sync_domain_status(
    inputs: SyncLiveProjectStatusInputs<'_>,
) -> Option<ProjectDomainStatus> {
    if inputs.summary_document.is_none() && inputs.bundle_preflight_document.is_none() {
        return None;
    }

    let mut source_kinds = Vec::new();
    let mut signal_keys = Vec::new();
    if inputs.summary_document.is_some() {
        source_kinds.push(SYNC_SOURCE_KIND_SUMMARY.to_string());
    }

    let mut blockers = Vec::new();
    let mut warnings = Vec::new();

    let (resources, status, reason_code, next_actions) = if let Some(document) =
        inputs.bundle_preflight_document
    {
        source_kinds.push(SYNC_SOURCE_KIND_BUNDLE_PREFLIGHT.to_string());
        signal_keys.extend(
            SYNC_SIGNAL_KEYS_BUNDLE_PREFLIGHT
                .iter()
                .map(|item| (*item).to_string()),
        );

        let resources = summary_number(document, summary_key::RESOURCE_COUNT);
        let sync_blocking = summary_number(document, summary_key::SYNC_BLOCKING_COUNT);
        let provider_blocking = summary_number(document, summary_key::PROVIDER_BLOCKING_COUNT);
        let secret_blocking =
            summary_number(document, summary_key::SECRET_PLACEHOLDER_BLOCKING_COUNT);
        let alert_blocking = summary_number(document, summary_key::ALERT_ARTIFACT_BLOCKED_COUNT);
        let alert_plan_only = summary_number(document, summary_key::ALERT_ARTIFACT_PLAN_ONLY_COUNT);
        let bundle_blocking = summary_number(document, summary_key::BLOCKED_COUNT);
        let bundle_plan_only = summary_number(document, summary_key::PLAN_ONLY_COUNT);

        // Keep the bundle-preflight handoff signals visible even when a
        // subset of them does not change the final status.
        if sync_blocking > 0 {
            blockers.push(status_finding(
                SYNC_BLOCKER_SYNC_BLOCKING,
                sync_blocking,
                signal::SUMMARY_SYNC_BLOCKING_COUNT,
            ));
        }
        if provider_blocking > 0 {
            blockers.push(status_finding(
                SYNC_BLOCKER_PROVIDER_BLOCKING,
                provider_blocking,
                signal::SUMMARY_PROVIDER_BLOCKING_COUNT,
            ));
        }
        if secret_blocking > 0 {
            blockers.push(status_finding(
                SYNC_BLOCKER_SECRET_PLACEHOLDER_BLOCKING,
                secret_blocking,
                signal::SUMMARY_SECRET_PLACEHOLDER_BLOCKING_COUNT,
            ));
        }
        if alert_blocking > 0 {
            blockers.push(status_finding(
                SYNC_BLOCKER_ALERT_ARTIFACT_BLOCKING,
                alert_blocking,
                signal::SUMMARY_ALERT_ARTIFACT_BLOCKED_COUNT,
            ));
        }
        if blockers.is_empty() && bundle_blocking > 0 {
            blockers.push(status_finding(
                SYNC_BLOCKER_BUNDLE_BLOCKING,
                bundle_blocking,
                signal::SUMMARY_BLOCKED_COUNT,
            ));
        }
        if alert_plan_only > 0 {
            warnings.push(status_finding(
                SYNC_WARNING_ALERT_ARTIFACT_PLAN_ONLY,
                alert_plan_only,
                signal::SUMMARY_ALERT_ARTIFACT_PLAN_ONLY_COUNT,
            ));
        } else if bundle_plan_only > 0 {
            warnings.push(status_finding(
                SYNC_WARNING_BUNDLE_PLAN_ONLY,
                bundle_plan_only,
                signal::SUMMARY_PLAN_ONLY_COUNT,
            ));
        }

        let has_blockers = !blockers.is_empty();
        let has_warnings = !warnings.is_empty();
        let next_actions = if has_blockers {
            SYNC_RESOLVE_BLOCKERS_ACTIONS
                .iter()
                .map(|item| (*item).to_string())
                .collect()
        } else if resources == 0 {
            next_actions_for_partial(resources, true)
        } else {
            next_actions_for_ready(has_warnings)
        };
        let status = if has_blockers {
            PROJECT_STATUS_BLOCKED
        } else if resources == 0 {
            PROJECT_STATUS_PARTIAL
        } else {
            PROJECT_STATUS_READY
        };
        let reason_code = if has_blockers {
            SYNC_REASON_BLOCKED_BY_BLOCKERS
        } else if resources == 0 {
            SYNC_REASON_PARTIAL_NO_DATA
        } else {
            SYNC_REASON_READY
        };
        (resources, status, reason_code, next_actions)
    } else {
        let resources = inputs
            .summary_document
            .map(|document| summary_number(document, summary_key::RESOURCE_COUNT))
            .unwrap_or(0);
        signal_keys.extend(
            SYNC_SIGNAL_KEYS_SUMMARY
                .iter()
                .map(|item| (*item).to_string()),
        );

        let next_actions = next_actions_for_partial(resources, false);
        let reason_code = if resources == 0 {
            SYNC_REASON_PARTIAL_NO_DATA
        } else {
            SYNC_REASON_PARTIAL_MISSING_SURFACES
        };
        (resources, PROJECT_STATUS_PARTIAL, reason_code, next_actions)
    };

    Some(
        StatusReading {
            id: SYNC_DOMAIN_ID.to_string(),
            scope: SYNC_SCOPE.to_string(),
            mode: SYNC_MODE.to_string(),
            status: status.to_string(),
            reason_code: reason_code.to_string(),
            primary_count: resources,
            source_kinds,
            signal_keys,
            blockers: blockers.into_iter().map(Into::into).collect(),
            warnings: warnings.into_iter().map(Into::into).collect(),
            next_actions,
            freshness: Default::default(),
        }
        .into_project_domain_status(),
    )
}
