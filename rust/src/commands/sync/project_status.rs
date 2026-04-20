//! Shared sync domain-status producer.
//!
//! Maintainer note:
//! - This module derives one sync-owned domain-status row from existing staged
//!   sync documents.
//! - Keep this document-driven and reusable by multiple consumers.

use serde_json::Value;

use crate::project_status::{
    status_finding, ProjectDomainStatus, PROJECT_STATUS_BLOCKED, PROJECT_STATUS_PARTIAL,
    PROJECT_STATUS_READY,
};
use crate::project_status_model::StatusReading;

use super::project_status_json::{
    section_array_count, section_name, section_object, section_summary_number, summary_number,
};

const SYNC_DOMAIN_ID: &str = "sync";
const SYNC_SCOPE: &str = "staged";
const SYNC_MODE: &str = "staged-documents";
const SYNC_REASON_READY: &str = PROJECT_STATUS_READY;
const SYNC_REASON_PARTIAL_NO_DATA: &str = "partial-no-data";
const SYNC_REASON_BLOCKED_BY_BLOCKERS: &str = "blocked-by-blockers";

const SYNC_BLOCKER_SYNC_BLOCKING: &str = "sync-blocking";
const SYNC_BLOCKER_PROVIDER_BLOCKING: &str = "provider-blocking";
const SYNC_BLOCKER_SECRET_PLACEHOLDER_BLOCKING: &str = "secret-placeholder-blocking";
const SYNC_BLOCKER_ALERT_ARTIFACT_BLOCKING: &str = "alert-artifact-blocking";
const SYNC_WARNING_PROVIDER_REVIEW: &str = "provider-review";
const SYNC_WARNING_SECRET_PLACEHOLDER_REVIEW: &str = "secret-placeholder-review";
const SYNC_WARNING_ALERT_ARTIFACT_PLAN_ONLY: &str = "alert-artifact-plan-only";
const SYNC_WARNING_ALERT_ARTIFACT_REVIEW: &str = "alert-artifact-review";
const SYNC_PROVIDER_ASSESSMENT_SIGNAL_KEY: &str = "providerAssessment.summary.blockingCount";
const SYNC_SECRET_PLACEHOLDER_ASSESSMENT_SIGNAL_KEY: &str =
    "secretPlaceholderAssessment.summary.blockingCount";
const SYNC_PROVIDER_ASSESSMENT_PLAN_SIGNAL_KEY: &str = "providerAssessment.plans";
const SYNC_SECRET_PLACEHOLDER_ASSESSMENT_PLAN_SIGNAL_KEY: &str =
    "secretPlaceholderAssessment.plans";
const SYNC_BUNDLE_PREFLIGHT_SIGNAL_KEYS: &[&str] = &[
    "summary.syncBlockingCount",
    "summary.providerBlockingCount",
    "summary.secretPlaceholderBlockingCount",
    "summary.alertArtifactBlockedCount",
    "summary.alertArtifactPlanOnlyCount",
    "summary.alertArtifactCount",
];

const SYNC_RESOLVE_BLOCKERS_ACTIONS: &[&str] = &[
    "resolve sync workflow blockers in the fixed order: sync, provider, secret-placeholder, alert-artifact",
];
const SYNC_STAGE_AT_LEAST_ONE_ACTIONS: &[&str] =
    &["stage at least one dashboard, datasource, or alert resource"];
const SYNC_REVIEW_NON_BLOCKING_ACTIONS: &[&str] =
    &["review non-blocking sync findings before promotion or apply"];
const SYNC_REEXPORT_AFTER_CHANGES_ACTIONS: &[&str] = &["re-run sync summary after staged changes"];

#[derive(Debug, Clone, Default)]
pub struct SyncDomainStatusInputs<'a> {
    pub summary_document: Option<&'a Value>,
    pub bundle_preflight_document: Option<&'a Value>,
}

pub(crate) fn build_sync_domain_status(
    inputs: SyncDomainStatusInputs<'_>,
) -> Option<ProjectDomainStatus> {
    let summary_document = inputs.summary_document;
    let bundle_preflight_document = inputs.bundle_preflight_document;
    if summary_document.is_none() && bundle_preflight_document.is_none() {
        return None;
    }

    let resources = summary_document
        .map(|document| summary_number(document, "resourceCount"))
        .or_else(|| {
            bundle_preflight_document.map(|document| summary_number(document, "resourceCount"))
        })
        .unwrap_or(0);

    let mut source_kinds = Vec::new();
    let mut signal_keys = Vec::new();
    if summary_document.is_some() {
        source_kinds.push("sync-summary".to_string());
        signal_keys.push("summary.resourceCount".to_string());
    } else if bundle_preflight_document.is_some() {
        signal_keys.push("summary.resourceCount".to_string());
    }

    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    if let Some(document) = bundle_preflight_document {
        source_kinds.push("package-test".to_string());
        signal_keys.extend(
            SYNC_BUNDLE_PREFLIGHT_SIGNAL_KEYS
                .iter()
                .map(|item| (*item).to_string()),
        );
        let sync_blocking = summary_number(document, "syncBlockingCount");
        let provider_blocking = summary_number(document, "providerBlockingCount").max(
            section_summary_number(document, section_name::PROVIDER_ASSESSMENT, "blockingCount"),
        );
        let provider_plan_count =
            section_array_count(document, section_name::PROVIDER_ASSESSMENT, "plans");
        let secret_blocking =
            summary_number(document, "secretPlaceholderBlockingCount").max(section_summary_number(
                document,
                section_name::SECRET_PLACEHOLDER_ASSESSMENT,
                "blockingCount",
            ));
        let secret_placeholder_plan_count = section_array_count(
            document,
            section_name::SECRET_PLACEHOLDER_ASSESSMENT,
            "plans",
        );
        let alert_blocking =
            summary_number(document, "alertArtifactBlockedCount").max(section_summary_number(
                document,
                section_name::ALERT_ARTIFACT_ASSESSMENT,
                "blockedCount",
            ));
        let alert_plan_only =
            summary_number(document, "alertArtifactPlanOnlyCount").max(section_summary_number(
                document,
                section_name::ALERT_ARTIFACT_ASSESSMENT,
                "planOnlyCount",
            ));
        let alert_artifact_count =
            summary_number(document, "alertArtifactCount").max(section_summary_number(
                document,
                section_name::ALERT_ARTIFACT_ASSESSMENT,
                "resourceCount",
            ));
        let provider_assessment_present =
            section_object(Some(document), section_name::PROVIDER_ASSESSMENT).is_some();
        let secret_placeholder_assessment_present =
            section_object(Some(document), section_name::SECRET_PLACEHOLDER_ASSESSMENT).is_some();

        if provider_assessment_present {
            signal_keys.push(SYNC_PROVIDER_ASSESSMENT_SIGNAL_KEY.to_string());
            signal_keys.push(SYNC_PROVIDER_ASSESSMENT_PLAN_SIGNAL_KEY.to_string());
        }
        if secret_placeholder_assessment_present {
            signal_keys.push(SYNC_SECRET_PLACEHOLDER_ASSESSMENT_SIGNAL_KEY.to_string());
            signal_keys.push(SYNC_SECRET_PLACEHOLDER_ASSESSMENT_PLAN_SIGNAL_KEY.to_string());
        }

        if sync_blocking > 0 {
            blockers.push(status_finding(
                SYNC_BLOCKER_SYNC_BLOCKING,
                sync_blocking,
                "summary.syncBlockingCount",
            ));
        }
        if provider_blocking > 0 {
            blockers.push(status_finding(
                SYNC_BLOCKER_PROVIDER_BLOCKING,
                provider_blocking,
                if summary_number(document, "providerBlockingCount") > 0 {
                    "summary.providerBlockingCount"
                } else {
                    SYNC_PROVIDER_ASSESSMENT_SIGNAL_KEY
                },
            ));
        }
        if secret_blocking > 0 {
            blockers.push(status_finding(
                SYNC_BLOCKER_SECRET_PLACEHOLDER_BLOCKING,
                secret_blocking,
                if summary_number(document, "secretPlaceholderBlockingCount") > 0 {
                    "summary.secretPlaceholderBlockingCount"
                } else {
                    SYNC_SECRET_PLACEHOLDER_ASSESSMENT_SIGNAL_KEY
                },
            ));
        }
        if provider_plan_count > 0 && provider_blocking == 0 {
            warnings.push(status_finding(
                SYNC_WARNING_PROVIDER_REVIEW,
                provider_plan_count,
                SYNC_PROVIDER_ASSESSMENT_PLAN_SIGNAL_KEY,
            ));
        }
        if secret_placeholder_plan_count > 0 && secret_blocking == 0 {
            warnings.push(status_finding(
                SYNC_WARNING_SECRET_PLACEHOLDER_REVIEW,
                secret_placeholder_plan_count,
                SYNC_SECRET_PLACEHOLDER_ASSESSMENT_PLAN_SIGNAL_KEY,
            ));
        }
        if alert_blocking > 0 {
            blockers.push(status_finding(
                SYNC_BLOCKER_ALERT_ARTIFACT_BLOCKING,
                alert_blocking,
                if summary_number(document, "alertArtifactBlockedCount") > 0 {
                    "summary.alertArtifactBlockedCount"
                } else {
                    "alertArtifactAssessment.summary.blockedCount"
                },
            ));
        }
        if alert_plan_only > 0 {
            warnings.push(status_finding(
                SYNC_WARNING_ALERT_ARTIFACT_PLAN_ONLY,
                alert_plan_only,
                if summary_number(document, "alertArtifactPlanOnlyCount") > 0 {
                    "summary.alertArtifactPlanOnlyCount"
                } else {
                    "alertArtifactAssessment.summary.planOnlyCount"
                },
            ));
        }
        if alert_artifact_count > 0 && alert_blocking == 0 && alert_plan_only == 0 {
            warnings.push(status_finding(
                SYNC_WARNING_ALERT_ARTIFACT_REVIEW,
                alert_artifact_count,
                if summary_number(document, "alertArtifactCount") > 0 {
                    "summary.alertArtifactCount"
                } else {
                    "alertArtifactAssessment.summary.resourceCount"
                },
            ));
        }
    }

    let has_blockers = !blockers.is_empty();
    let is_partial = resources == 0;
    let (status, reason_code, next_actions) = if has_blockers {
        (
            PROJECT_STATUS_BLOCKED,
            SYNC_REASON_BLOCKED_BY_BLOCKERS,
            SYNC_RESOLVE_BLOCKERS_ACTIONS,
        )
    } else if is_partial {
        (
            PROJECT_STATUS_PARTIAL,
            SYNC_REASON_PARTIAL_NO_DATA,
            SYNC_STAGE_AT_LEAST_ONE_ACTIONS,
        )
    } else if !warnings.is_empty() {
        (
            PROJECT_STATUS_READY,
            SYNC_REASON_READY,
            SYNC_REVIEW_NON_BLOCKING_ACTIONS,
        )
    } else {
        (
            PROJECT_STATUS_READY,
            SYNC_REASON_READY,
            SYNC_REEXPORT_AFTER_CHANGES_ACTIONS,
        )
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
            next_actions: next_actions
                .iter()
                .map(|item| (*item).to_string())
                .collect(),
            freshness: Default::default(),
        }
        .into_project_domain_status(),
    )
}
