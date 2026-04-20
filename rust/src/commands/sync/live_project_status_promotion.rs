//! Richer live promotion domain-status producer.
//!
//! Maintainer note:
//! - This module is intentionally more informative than the old transport-only
//!   stub, but it still stays conservative about readiness.
//! - It only promotes to `ready` when the staged promotion summary, mapping,
//!   and availability inputs all supply explicit evidence.
//! - When the staged promotion summary already carries handoff and
//!   apply-continuation summaries, those are surfaced as warnings and next
//!   actions instead of being inferred from transport state.

#![allow(dead_code)]

use serde_json::Value;

use crate::project_status::{
    status_finding, ProjectDomainStatus, ProjectStatusFinding, PROJECT_STATUS_BLOCKED,
    PROJECT_STATUS_PARTIAL, PROJECT_STATUS_READY,
};
use crate::project_status_model::StatusReading;

use super::project_status_json::{
    push_unique, section_bool, section_object, section_text, summary_number, value_array_count,
};

const LIVE_PROMOTION_DOMAIN_ID: &str = "promotion";
const LIVE_PROMOTION_SCOPE: &str = "live";
const LIVE_PROMOTION_MODE: &str = "live-promotion-surfaces";
const LIVE_PROMOTION_REASON_READY: &str = PROJECT_STATUS_READY;
const LIVE_PROMOTION_REASON_PARTIAL_NO_DATA: &str = "partial-no-data";
const LIVE_PROMOTION_REASON_BLOCKED_BY_BLOCKERS: &str = "blocked-by-blockers";

const LIVE_PROMOTION_SOURCE_KINDS: &[&str] = &[
    "live-promotion-summary",
    "live-promotion-mapping",
    "live-promotion-availability",
];

mod summary_key {
    pub(super) const RESOURCE_COUNT: &str = "resourceCount";
    pub(super) const MISSING_MAPPING_COUNT: &str = "missingMappingCount";
    pub(super) const BUNDLE_BLOCKING_COUNT: &str = "bundleBlockingCount";
    pub(super) const BLOCKING_COUNT: &str = "blockingCount";
}

mod mapping_key {
    pub(super) const FOLDERS: &str = "folders";
    pub(super) const DATASOURCES: &str = "datasources";
    pub(super) const DATASOURCE_UIDS: &str = "uids";
    pub(super) const DATASOURCE_NAMES: &str = "names";
}

mod availability_key {
    pub(super) const PLUGIN_IDS: &str = "pluginIds";
    pub(super) const DATASOURCE_UIDS: &str = "datasourceUids";
    pub(super) const DATASOURCE_NAMES: &str = "datasourceNames";
    pub(super) const CONTACT_POINTS: &str = "contactPoints";
    pub(super) const PROVIDER_NAMES: &str = "providerNames";
    pub(super) const SECRET_PLACEHOLDER_NAMES: &str = "secretPlaceholderNames";
}

mod section {
    pub(super) const HANDOFF: &str = "handoffSummary";
    pub(super) const CONTINUATION: &str = "continuationSummary";
}

mod handoff_key {
    pub(super) const READY_FOR_REVIEW: &str = "readyForReview";
    pub(super) const REVIEW_INSTRUCTION: &str = "reviewInstruction";
}

mod continuation_key {
    pub(super) const READY_FOR_CONTINUATION: &str = "readyForContinuation";
    pub(super) const INSTRUCTION: &str = "continuationInstruction";
}

mod signal {
    pub(super) const SUMMARY_RESOURCE_COUNT: &str = "summary.resourceCount";
    pub(super) const SUMMARY_MISSING_MAPPING_COUNT: &str = "summary.missingMappingCount";
    pub(super) const SUMMARY_BUNDLE_BLOCKING_COUNT: &str = "summary.bundleBlockingCount";
    pub(super) const SUMMARY_BLOCKING_COUNT: &str = "summary.blockingCount";
    pub(super) const MAPPING_ENTRY_COUNT: &str = "mapping.entryCount";
    pub(super) const AVAILABILITY_ENTRY_COUNT: &str = "availability.entryCount";
    pub(super) const HANDOFF_REVIEW_REQUIRED: &str = "handoffSummary.reviewRequired";
    pub(super) const HANDOFF_READY_FOR_REVIEW: &str = "handoffSummary.readyForReview";
    pub(super) const HANDOFF_NEXT_STAGE: &str = "handoffSummary.nextStage";
    pub(super) const HANDOFF_BLOCKING_COUNT: &str = "handoffSummary.blockingCount";
    pub(super) const HANDOFF_REVIEW_INSTRUCTION: &str = "handoffSummary.reviewInstruction";
    pub(super) const CONTINUATION_STAGED_ONLY: &str = "continuationSummary.stagedOnly";
    pub(super) const CONTINUATION_LIVE_MUTATION_ALLOWED: &str =
        "continuationSummary.liveMutationAllowed";
    pub(super) const CONTINUATION_READY_FOR_CONTINUATION: &str =
        "continuationSummary.readyForContinuation";
    pub(super) const CONTINUATION_NEXT_STAGE: &str = "continuationSummary.nextStage";
    pub(super) const CONTINUATION_BLOCKING_COUNT: &str = "continuationSummary.blockingCount";
    pub(super) const CONTINUATION_INSTRUCTION: &str = "continuationSummary.continuationInstruction";
}

const LIVE_PROMOTION_SIGNAL_KEYS: &[&str] = &[
    signal::SUMMARY_RESOURCE_COUNT,
    signal::SUMMARY_MISSING_MAPPING_COUNT,
    signal::SUMMARY_BUNDLE_BLOCKING_COUNT,
    signal::SUMMARY_BLOCKING_COUNT,
    signal::MAPPING_ENTRY_COUNT,
    signal::AVAILABILITY_ENTRY_COUNT,
    signal::HANDOFF_REVIEW_REQUIRED,
    signal::HANDOFF_READY_FOR_REVIEW,
    signal::HANDOFF_NEXT_STAGE,
    signal::HANDOFF_BLOCKING_COUNT,
    signal::HANDOFF_REVIEW_INSTRUCTION,
    signal::CONTINUATION_STAGED_ONLY,
    signal::CONTINUATION_LIVE_MUTATION_ALLOWED,
    signal::CONTINUATION_READY_FOR_CONTINUATION,
    signal::CONTINUATION_NEXT_STAGE,
    signal::CONTINUATION_BLOCKING_COUNT,
    signal::CONTINUATION_INSTRUCTION,
];

const LIVE_PROMOTION_AVAILABILITY_COLLECTION_KEYS: &[&str] = &[
    availability_key::PLUGIN_IDS,
    availability_key::DATASOURCE_UIDS,
    availability_key::DATASOURCE_NAMES,
    availability_key::CONTACT_POINTS,
    availability_key::PROVIDER_NAMES,
    availability_key::SECRET_PLACEHOLDER_NAMES,
];

const LIVE_PROMOTION_BLOCKER_MISSING_MAPPINGS: &str = "missing-mappings";
const LIVE_PROMOTION_BLOCKER_BUNDLE_BLOCKING: &str = "bundle-blocking";
const LIVE_PROMOTION_BLOCKER_BLOCKING: &str = "blocking";
const LIVE_PROMOTION_WARNING_REVIEW_HANDOFF: &str = "review-handoff";
const LIVE_PROMOTION_WARNING_APPLY_CONTINUATION: &str = "apply-continuation";

const LIVE_PROMOTION_RESOLVE_BLOCKERS_ACTIONS: &[&str] =
    &["resolve promotion blockers in the fixed order: missing-mapping, bundle-blocking, blocking"];
const LIVE_PROMOTION_STAGE_AT_LEAST_ONE_ACTIONS: &[&str] =
    &["stage at least one promotable resource before promotion"];
const LIVE_PROMOTION_PROVIDE_SUMMARY_ACTIONS: &[&str] =
    &["provide a staged promotion summary before interpreting live promotion readiness"];
const LIVE_PROMOTION_PROVIDE_MAPPING_ACTIONS: &[&str] =
    &["provide explicit promotion mappings before promotion"];
const LIVE_PROMOTION_PROVIDE_AVAILABILITY_ACTIONS: &[&str] =
    &["provide live availability hints before promotion"];
const LIVE_PROMOTION_REVIEW_HANOFF_ACTIONS: &[&str] =
    &["resolve the staged promotion handoff before review"];
const LIVE_PROMOTION_REVIEW_READY_ACTIONS: &[&str] = &["promotion handoff is review-ready"];
const LIVE_PROMOTION_APPLY_CONTINUATION_ACTIONS: &[&str] =
    &["keep the promotion staged until the apply continuation is ready"];
const LIVE_PROMOTION_APPLY_READY_ACTIONS: &[&str] =
    &["promotion is apply-ready in the staged continuation"];

#[derive(Debug, Clone, Default)]
pub(crate) struct LivePromotionProjectStatusInputs<'a> {
    pub promotion_summary_document: Option<&'a Value>,
    pub promotion_mapping_document: Option<&'a Value>,
    pub availability_document: Option<&'a Value>,
}

fn mapping_entry_count(document: Option<&Value>) -> usize {
    let Some(object) = document.and_then(Value::as_object) else {
        return 0;
    };

    let folders = object
        .get(mapping_key::FOLDERS)
        .and_then(Value::as_object)
        .map(|value| value.len())
        .unwrap_or(0);
    let datasource_uid_mappings = object
        .get(mapping_key::DATASOURCES)
        .and_then(Value::as_object)
        .and_then(|value| value.get(mapping_key::DATASOURCE_UIDS))
        .and_then(Value::as_object)
        .map(|value| value.len())
        .unwrap_or(0);
    let datasource_name_mappings = object
        .get(mapping_key::DATASOURCES)
        .and_then(Value::as_object)
        .and_then(|value| value.get(mapping_key::DATASOURCE_NAMES))
        .and_then(Value::as_object)
        .map(|value| value.len())
        .unwrap_or(0);

    folders + datasource_uid_mappings + datasource_name_mappings
}

fn availability_entry_count(document: Option<&Value>) -> usize {
    let Some(object) = document.and_then(Value::as_object) else {
        return 0;
    };

    LIVE_PROMOTION_AVAILABILITY_COLLECTION_KEYS
        .iter()
        .copied()
        .map(|key| value_array_count(object.get(key)))
        .sum()
}

fn add_handoff_evidence(
    document: Option<&Value>,
    warnings: &mut Vec<ProjectStatusFinding>,
    next_actions: &mut Vec<String>,
) {
    if section_object(document, section::HANDOFF).is_none() {
        return;
    }

    warnings.push(status_finding(
        LIVE_PROMOTION_WARNING_REVIEW_HANDOFF,
        1,
        signal::HANDOFF_REVIEW_REQUIRED,
    ));

    if section_bool(document, section::HANDOFF, handoff_key::READY_FOR_REVIEW) {
        for action in LIVE_PROMOTION_REVIEW_READY_ACTIONS {
            push_unique(next_actions, action);
        }
    } else {
        let action = section_text(document, section::HANDOFF, handoff_key::REVIEW_INSTRUCTION)
            .unwrap_or_else(|| LIVE_PROMOTION_REVIEW_HANOFF_ACTIONS[0].to_string());
        push_unique(next_actions, &action);
    }
}

fn add_continuation_evidence(
    document: Option<&Value>,
    warnings: &mut Vec<ProjectStatusFinding>,
    next_actions: &mut Vec<String>,
) {
    if section_object(document, section::CONTINUATION).is_none() {
        return;
    }

    warnings.push(status_finding(
        LIVE_PROMOTION_WARNING_APPLY_CONTINUATION,
        1,
        signal::CONTINUATION_LIVE_MUTATION_ALLOWED,
    ));

    if section_bool(
        document,
        section::CONTINUATION,
        continuation_key::READY_FOR_CONTINUATION,
    ) {
        for action in LIVE_PROMOTION_APPLY_READY_ACTIONS {
            push_unique(next_actions, action);
        }
    } else {
        let action = section_text(
            document,
            section::CONTINUATION,
            continuation_key::INSTRUCTION,
        )
        .unwrap_or_else(|| LIVE_PROMOTION_APPLY_CONTINUATION_ACTIONS[0].to_string());
        push_unique(next_actions, &action);
    }
}

fn build_next_actions(
    summary_present: bool,
    mapping_present: bool,
    availability_present: bool,
    resource_count: usize,
    mapping_count: usize,
    availability_count: usize,
) -> Vec<String> {
    let mut next_actions = Vec::new();

    if !summary_present {
        for action in LIVE_PROMOTION_PROVIDE_SUMMARY_ACTIONS {
            push_unique(&mut next_actions, action);
        }
    }
    if resource_count == 0 {
        for action in LIVE_PROMOTION_STAGE_AT_LEAST_ONE_ACTIONS {
            push_unique(&mut next_actions, action);
        }
    }
    if !mapping_present || mapping_count == 0 {
        for action in LIVE_PROMOTION_PROVIDE_MAPPING_ACTIONS {
            push_unique(&mut next_actions, action);
        }
    }
    if !availability_present || availability_count == 0 {
        for action in LIVE_PROMOTION_PROVIDE_AVAILABILITY_ACTIONS {
            push_unique(&mut next_actions, action);
        }
    }

    next_actions
}

pub(crate) fn build_live_promotion_project_status(
    inputs: LivePromotionProjectStatusInputs<'_>,
) -> Option<ProjectDomainStatus> {
    if inputs.promotion_summary_document.is_none()
        && inputs.promotion_mapping_document.is_none()
        && inputs.availability_document.is_none()
    {
        return None;
    }

    let resource_count = inputs
        .promotion_summary_document
        .map(|document| summary_number(document, summary_key::RESOURCE_COUNT))
        .unwrap_or(0);
    let summary_present = inputs.promotion_summary_document.is_some();
    let missing_mapping_count = inputs
        .promotion_summary_document
        .map(|document| summary_number(document, summary_key::MISSING_MAPPING_COUNT))
        .unwrap_or(0);
    let bundle_blocking_count = inputs
        .promotion_summary_document
        .map(|document| summary_number(document, summary_key::BUNDLE_BLOCKING_COUNT))
        .unwrap_or(0);
    let blocking_count = inputs
        .promotion_summary_document
        .map(|document| summary_number(document, summary_key::BLOCKING_COUNT))
        .unwrap_or(0);
    let mapping_count = mapping_entry_count(inputs.promotion_mapping_document);
    let availability_count = availability_entry_count(inputs.availability_document);

    let mut source_kinds = Vec::new();
    if inputs.promotion_summary_document.is_some() {
        source_kinds.push(LIVE_PROMOTION_SOURCE_KINDS[0].to_string());
    }
    if inputs.promotion_mapping_document.is_some() {
        source_kinds.push(LIVE_PROMOTION_SOURCE_KINDS[1].to_string());
    }
    if inputs.availability_document.is_some() {
        source_kinds.push(LIVE_PROMOTION_SOURCE_KINDS[2].to_string());
    }

    let mut blockers = Vec::new();
    if missing_mapping_count > 0 {
        blockers.push(status_finding(
            LIVE_PROMOTION_BLOCKER_MISSING_MAPPINGS,
            missing_mapping_count,
            signal::SUMMARY_MISSING_MAPPING_COUNT,
        ));
    }
    if bundle_blocking_count > 0 {
        blockers.push(status_finding(
            LIVE_PROMOTION_BLOCKER_BUNDLE_BLOCKING,
            bundle_blocking_count,
            signal::SUMMARY_BUNDLE_BLOCKING_COUNT,
        ));
    }
    if blockers.is_empty() && blocking_count > 0 {
        blockers.push(status_finding(
            LIVE_PROMOTION_BLOCKER_BLOCKING,
            blocking_count,
            signal::SUMMARY_BLOCKING_COUNT,
        ));
    }

    let mut warnings = Vec::new();
    let mut evidence_actions = Vec::new();
    add_handoff_evidence(
        inputs.promotion_summary_document,
        &mut warnings,
        &mut evidence_actions,
    );
    add_continuation_evidence(
        inputs.promotion_summary_document,
        &mut warnings,
        &mut evidence_actions,
    );

    let (status, reason_code, next_actions) = if !blockers.is_empty() {
        (
            PROJECT_STATUS_BLOCKED,
            LIVE_PROMOTION_REASON_BLOCKED_BY_BLOCKERS,
            LIVE_PROMOTION_RESOLVE_BLOCKERS_ACTIONS
                .iter()
                .map(|item| (*item).to_string())
                .collect::<Vec<String>>(),
        )
    } else if !summary_present
        || resource_count == 0
        || mapping_count == 0
        || availability_count == 0
    {
        let mut next_actions = build_next_actions(
            summary_present,
            inputs.promotion_mapping_document.is_some(),
            inputs.availability_document.is_some(),
            resource_count,
            mapping_count,
            availability_count,
        );
        next_actions.extend(evidence_actions);
        (
            PROJECT_STATUS_PARTIAL,
            LIVE_PROMOTION_REASON_PARTIAL_NO_DATA,
            next_actions,
        )
    } else {
        (
            PROJECT_STATUS_READY,
            LIVE_PROMOTION_REASON_READY,
            evidence_actions,
        )
    };

    Some(
        StatusReading {
            id: LIVE_PROMOTION_DOMAIN_ID.to_string(),
            scope: LIVE_PROMOTION_SCOPE.to_string(),
            mode: LIVE_PROMOTION_MODE.to_string(),
            status: status.to_string(),
            reason_code: reason_code.to_string(),
            primary_count: resource_count,
            source_kinds,
            signal_keys: LIVE_PROMOTION_SIGNAL_KEYS
                .iter()
                .map(|item| (*item).to_string())
                .collect(),
            blockers: blockers.into_iter().map(Into::into).collect(),
            warnings: warnings.into_iter().map(Into::into).collect(),
            next_actions,
            freshness: Default::default(),
        }
        .into_project_domain_status(),
    )
}
