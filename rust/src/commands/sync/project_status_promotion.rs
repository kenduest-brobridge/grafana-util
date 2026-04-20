//! Shared promotion domain-status producer.
//!
//! Maintainer note:
//! - This module derives one promotion-owned domain-status row from the staged
//!   promotion preflight document.
//! - Keep the producer document-driven and conservative; missing mappings,
//!   bundle blocking, and staged blocking should remain explicit signals.

use serde_json::Value;

use crate::project_status::{
    status_finding, ProjectDomainStatus, PROJECT_STATUS_BLOCKED, PROJECT_STATUS_PARTIAL,
    PROJECT_STATUS_READY,
};
use crate::project_status_model::StatusReading;

use super::project_status_json::{
    push_unique, section_bool, section_number, section_object, section_text, summary_number,
};

const PROMOTION_DOMAIN_ID: &str = "promotion";
const PROMOTION_SCOPE: &str = "staged";
const PROMOTION_MODE: &str = "artifact-summary";
const PROMOTION_REASON_READY: &str = PROJECT_STATUS_READY;
const PROMOTION_REASON_PARTIAL_NO_DATA: &str = "partial-no-data";
const PROMOTION_REASON_BLOCKED_BY_BLOCKERS: &str = "blocked-by-blockers";

const PROMOTION_SOURCE_KINDS: &[&str] = &["promotion-preflight"];

mod summary_key {
    pub(super) const RESOURCE_COUNT: &str = "resourceCount";
    pub(super) const MISSING_MAPPING_COUNT: &str = "missingMappingCount";
    pub(super) const BUNDLE_BLOCKING_COUNT: &str = "bundleBlockingCount";
    pub(super) const BLOCKING_COUNT: &str = "blockingCount";
}

mod section {
    pub(super) const HANDOFF: &str = "handoffSummary";
    pub(super) const CONTINUATION: &str = "continuationSummary";
    pub(super) const CHECK_SUMMARY: &str = "checkSummary";
}

mod handoff_key {
    pub(super) const READY_FOR_REVIEW: &str = "readyForReview";
    pub(super) const BLOCKING_COUNT: &str = "blockingCount";
    pub(super) const REVIEW_INSTRUCTION: &str = "reviewInstruction";
}

mod continuation_key {
    pub(super) const READY_FOR_CONTINUATION: &str = "readyForContinuation";
    pub(super) const RESOLVED_COUNT: &str = "resolvedCount";
    pub(super) const BLOCKING_COUNT: &str = "blockingCount";
    pub(super) const INSTRUCTION: &str = "continuationInstruction";
}

mod check_summary_key {
    pub(super) const FOLDER_REMAP_COUNT: &str = "folderRemapCount";
    pub(super) const DATASOURCE_UID_REMAP_COUNT: &str = "datasourceUidRemapCount";
    pub(super) const DATASOURCE_NAME_REMAP_COUNT: &str = "datasourceNameRemapCount";
}

mod signal {
    pub(super) const SUMMARY_RESOURCE_COUNT: &str = "summary.resourceCount";
    pub(super) const SUMMARY_MISSING_MAPPING_COUNT: &str = "summary.missingMappingCount";
    pub(super) const SUMMARY_BUNDLE_BLOCKING_COUNT: &str = "summary.bundleBlockingCount";
    pub(super) const SUMMARY_BLOCKING_COUNT: &str = "summary.blockingCount";
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
    pub(super) const CONTINUATION_RESOLVED_COUNT: &str = "continuationSummary.resolvedCount";
    pub(super) const CONTINUATION_INSTRUCTION: &str = "continuationSummary.continuationInstruction";
    pub(super) const CHECK_FOLDER_REMAP_COUNT: &str = "checkSummary.folderRemapCount";
    pub(super) const CHECK_DATASOURCE_UID_REMAP_COUNT: &str =
        "checkSummary.datasourceUidRemapCount";
    pub(super) const CHECK_DATASOURCE_NAME_REMAP_COUNT: &str =
        "checkSummary.datasourceNameRemapCount";
    pub(super) const CHECK_RESOLVED_COUNT: &str = "checkSummary.resolvedCount";
    pub(super) const CHECK_DIRECT_COUNT: &str = "checkSummary.directCount";
    pub(super) const CHECK_MAPPED_COUNT: &str = "checkSummary.mappedCount";
    pub(super) const CHECK_MISSING_TARGET_COUNT: &str = "checkSummary.missingTargetCount";
}

const PROMOTION_SIGNAL_KEYS: &[&str] = &[
    signal::SUMMARY_RESOURCE_COUNT,
    signal::SUMMARY_MISSING_MAPPING_COUNT,
    signal::SUMMARY_BUNDLE_BLOCKING_COUNT,
    signal::SUMMARY_BLOCKING_COUNT,
];
const PROMOTION_HANDOFF_SIGNAL_KEYS: &[&str] = &[
    signal::HANDOFF_REVIEW_REQUIRED,
    signal::HANDOFF_READY_FOR_REVIEW,
    signal::HANDOFF_NEXT_STAGE,
    signal::HANDOFF_BLOCKING_COUNT,
    signal::HANDOFF_REVIEW_INSTRUCTION,
];
const PROMOTION_CONTINUATION_SIGNAL_KEYS: &[&str] = &[
    signal::CONTINUATION_STAGED_ONLY,
    signal::CONTINUATION_LIVE_MUTATION_ALLOWED,
    signal::CONTINUATION_READY_FOR_CONTINUATION,
    signal::CONTINUATION_NEXT_STAGE,
    signal::CONTINUATION_BLOCKING_COUNT,
    signal::CONTINUATION_RESOLVED_COUNT,
    signal::CONTINUATION_INSTRUCTION,
];
const PROMOTION_CHECK_SUMMARY_SIGNAL_KEYS: &[&str] = &[
    signal::CHECK_FOLDER_REMAP_COUNT,
    signal::CHECK_DATASOURCE_UID_REMAP_COUNT,
    signal::CHECK_DATASOURCE_NAME_REMAP_COUNT,
    signal::CHECK_RESOLVED_COUNT,
    signal::CHECK_DIRECT_COUNT,
    signal::CHECK_MAPPED_COUNT,
    signal::CHECK_MISSING_TARGET_COUNT,
];

const PROMOTION_BLOCKER_MISSING_MAPPINGS: &str = "missing-mappings";
const PROMOTION_BLOCKER_BUNDLE_BLOCKING: &str = "bundle-blocking";
const PROMOTION_BLOCKER_BLOCKING: &str = "blocking";
const PROMOTION_WARNING_REVIEW_HANDOFF: &str = "review-handoff";
const PROMOTION_WARNING_APPLY_CONTINUATION: &str = "apply-continuation";
const PROMOTION_WARNING_FOLDER_REMAPS: &str = "folder-remaps";
const PROMOTION_WARNING_DATASOURCE_UID_REMAPS: &str = "datasource-uid-remaps";
const PROMOTION_WARNING_DATASOURCE_NAME_REMAPS: &str = "datasource-name-remaps";

const PROMOTION_RESOLVE_BLOCKERS_ACTIONS: &[&str] =
    &["resolve promotion blockers in the fixed order: missing-mapping, bundle-blocking, blocking"];
const PROMOTION_STAGE_AT_LEAST_ONE_ACTIONS: &[&str] =
    &["stage at least one promotable resource before promotion"];
const PROMOTION_REVIEW_READY_ACTIONS: &[&str] = &["promotion handoff is review-ready"];
const PROMOTION_REVIEW_HANDOFF_ACTIONS: &[&str] =
    &["resolve the staged promotion handoff before review"];
const PROMOTION_APPLY_READY_ACTIONS: &[&str] =
    &["promotion is apply-ready in the staged continuation"];
const PROMOTION_APPLY_CONTINUATION_ACTIONS: &[&str] =
    &["keep the promotion staged until the apply continuation is ready"];
const PROMOTION_REVIEW_FOLDER_REMAPS_ACTIONS: &[&str] =
    &["review folder remaps before promotion review"];
const PROMOTION_REVIEW_DATASOURCE_REMAPS_ACTIONS: &[&str] =
    &["review datasource remaps before promotion review"];

fn handoff_warning_source(document: Option<&Value>) -> &'static str {
    if section_bool(document, section::HANDOFF, handoff_key::READY_FOR_REVIEW)
        && section_text(document, section::HANDOFF, handoff_key::REVIEW_INSTRUCTION).is_none()
    {
        signal::HANDOFF_NEXT_STAGE
    } else if section_bool(document, section::HANDOFF, handoff_key::READY_FOR_REVIEW) {
        signal::HANDOFF_READY_FOR_REVIEW
    } else if section_text(document, section::HANDOFF, handoff_key::REVIEW_INSTRUCTION).is_some() {
        signal::HANDOFF_REVIEW_INSTRUCTION
    } else {
        signal::HANDOFF_REVIEW_REQUIRED
    }
}

fn continuation_warning_source(document: Option<&Value>) -> &'static str {
    if section_bool(
        document,
        section::CONTINUATION,
        continuation_key::READY_FOR_CONTINUATION,
    ) && section_number(
        document,
        section::CONTINUATION,
        continuation_key::RESOLVED_COUNT,
    ) > 0
    {
        signal::CONTINUATION_RESOLVED_COUNT
    } else if section_bool(
        document,
        section::CONTINUATION,
        continuation_key::READY_FOR_CONTINUATION,
    ) && section_text(
        document,
        section::CONTINUATION,
        continuation_key::INSTRUCTION,
    )
    .is_none()
    {
        signal::CONTINUATION_NEXT_STAGE
    } else if section_bool(
        document,
        section::CONTINUATION,
        continuation_key::READY_FOR_CONTINUATION,
    ) {
        signal::CONTINUATION_READY_FOR_CONTINUATION
    } else if section_text(
        document,
        section::CONTINUATION,
        continuation_key::INSTRUCTION,
    )
    .is_some()
    {
        signal::CONTINUATION_INSTRUCTION
    } else {
        signal::CONTINUATION_LIVE_MUTATION_ALLOWED
    }
}

fn continuation_warning_count(document: Option<&Value>) -> usize {
    let resolved = section_number(
        document,
        section::CONTINUATION,
        continuation_key::RESOLVED_COUNT,
    );
    if continuation_warning_source(document) == signal::CONTINUATION_RESOLVED_COUNT {
        resolved.max(1)
    } else {
        1
    }
}

pub(crate) fn build_promotion_domain_status(
    promotion_preflight_document: Option<&Value>,
) -> Option<ProjectDomainStatus> {
    let document = promotion_preflight_document?;
    let resources = summary_number(document, summary_key::RESOURCE_COUNT);
    let missing_mappings = summary_number(document, summary_key::MISSING_MAPPING_COUNT);
    let bundle_blocking = summary_number(document, summary_key::BUNDLE_BLOCKING_COUNT);
    let summary_blocking = summary_number(document, summary_key::BLOCKING_COUNT);
    let handoff_blocking = section_number(
        Some(document),
        section::HANDOFF,
        handoff_key::BLOCKING_COUNT,
    );
    let continuation_blocking = section_number(
        Some(document),
        section::CONTINUATION,
        continuation_key::BLOCKING_COUNT,
    );
    let blocking = summary_blocking
        .max(handoff_blocking)
        .max(continuation_blocking);
    let blocking_source = if summary_blocking > 0 {
        signal::SUMMARY_BLOCKING_COUNT
    } else if handoff_blocking > 0 {
        signal::HANDOFF_BLOCKING_COUNT
    } else {
        signal::CONTINUATION_BLOCKING_COUNT
    };

    let mut blockers = Vec::new();
    if missing_mappings > 0 {
        blockers.push(status_finding(
            PROMOTION_BLOCKER_MISSING_MAPPINGS,
            missing_mappings,
            signal::SUMMARY_MISSING_MAPPING_COUNT,
        ));
    }
    if bundle_blocking > 0 {
        blockers.push(status_finding(
            PROMOTION_BLOCKER_BUNDLE_BLOCKING,
            bundle_blocking,
            signal::SUMMARY_BUNDLE_BLOCKING_COUNT,
        ));
    }
    if blockers.is_empty() && blocking > 0 {
        blockers.push(status_finding(
            PROMOTION_BLOCKER_BLOCKING,
            blocking,
            blocking_source,
        ));
    }

    let mut signal_keys = PROMOTION_SIGNAL_KEYS
        .iter()
        .map(|item| (*item).to_string())
        .collect::<Vec<_>>();
    if section_object(Some(document), section::HANDOFF).is_some() {
        signal_keys.extend(
            PROMOTION_HANDOFF_SIGNAL_KEYS
                .iter()
                .map(|item| (*item).to_string()),
        );
    }
    if section_object(Some(document), section::CONTINUATION).is_some() {
        signal_keys.extend(
            PROMOTION_CONTINUATION_SIGNAL_KEYS
                .iter()
                .map(|item| (*item).to_string()),
        );
    }
    if section_object(Some(document), section::CHECK_SUMMARY).is_some() {
        signal_keys.extend(
            PROMOTION_CHECK_SUMMARY_SIGNAL_KEYS
                .iter()
                .map(|item| (*item).to_string()),
        );
    }

    let has_blockers = !blockers.is_empty();
    let is_partial = resources == 0;
    let (status, reason_code) = if has_blockers {
        (PROJECT_STATUS_BLOCKED, PROMOTION_REASON_BLOCKED_BY_BLOCKERS)
    } else if is_partial {
        (PROJECT_STATUS_PARTIAL, PROMOTION_REASON_PARTIAL_NO_DATA)
    } else {
        (PROJECT_STATUS_READY, PROMOTION_REASON_READY)
    };
    let mut warnings = Vec::new();
    let folder_remaps = section_number(
        Some(document),
        section::CHECK_SUMMARY,
        check_summary_key::FOLDER_REMAP_COUNT,
    );
    let datasource_uid_remaps = section_number(
        Some(document),
        section::CHECK_SUMMARY,
        check_summary_key::DATASOURCE_UID_REMAP_COUNT,
    );
    let datasource_name_remaps = section_number(
        Some(document),
        section::CHECK_SUMMARY,
        check_summary_key::DATASOURCE_NAME_REMAP_COUNT,
    );
    if folder_remaps > 0 {
        warnings.push(status_finding(
            PROMOTION_WARNING_FOLDER_REMAPS,
            folder_remaps,
            signal::CHECK_FOLDER_REMAP_COUNT,
        ));
    }
    if datasource_uid_remaps > 0 {
        warnings.push(status_finding(
            PROMOTION_WARNING_DATASOURCE_UID_REMAPS,
            datasource_uid_remaps,
            signal::CHECK_DATASOURCE_UID_REMAP_COUNT,
        ));
    }
    if datasource_name_remaps > 0 {
        warnings.push(status_finding(
            PROMOTION_WARNING_DATASOURCE_NAME_REMAPS,
            datasource_name_remaps,
            signal::CHECK_DATASOURCE_NAME_REMAP_COUNT,
        ));
    }
    let next_actions = if has_blockers {
        PROMOTION_RESOLVE_BLOCKERS_ACTIONS
            .iter()
            .map(|item| (*item).to_string())
            .collect::<Vec<_>>()
    } else if is_partial {
        PROMOTION_STAGE_AT_LEAST_ONE_ACTIONS
            .iter()
            .map(|item| (*item).to_string())
            .collect::<Vec<_>>()
    } else {
        let mut actions = Vec::new();
        if section_object(Some(document), section::HANDOFF).is_some() {
            warnings.push(status_finding(
                PROMOTION_WARNING_REVIEW_HANDOFF,
                1,
                handoff_warning_source(Some(document)),
            ));
            let action = if section_bool(
                Some(document),
                section::HANDOFF,
                handoff_key::READY_FOR_REVIEW,
            ) {
                PROMOTION_REVIEW_READY_ACTIONS[0].to_string()
            } else {
                section_text(
                    Some(document),
                    section::HANDOFF,
                    handoff_key::REVIEW_INSTRUCTION,
                )
                .unwrap_or_else(|| PROMOTION_REVIEW_HANDOFF_ACTIONS[0].to_string())
            };
            push_unique(&mut actions, &action);
        }
        if folder_remaps > 0 {
            push_unique(&mut actions, PROMOTION_REVIEW_FOLDER_REMAPS_ACTIONS[0]);
        }
        if datasource_uid_remaps > 0 || datasource_name_remaps > 0 {
            push_unique(&mut actions, PROMOTION_REVIEW_DATASOURCE_REMAPS_ACTIONS[0]);
        }
        if section_object(Some(document), section::CONTINUATION).is_some() {
            warnings.push(status_finding(
                PROMOTION_WARNING_APPLY_CONTINUATION,
                continuation_warning_count(Some(document)),
                continuation_warning_source(Some(document)),
            ));
            let action = if section_bool(
                Some(document),
                section::CONTINUATION,
                continuation_key::READY_FOR_CONTINUATION,
            ) {
                PROMOTION_APPLY_READY_ACTIONS[0].to_string()
            } else {
                section_text(
                    Some(document),
                    section::CONTINUATION,
                    continuation_key::INSTRUCTION,
                )
                .unwrap_or_else(|| PROMOTION_APPLY_CONTINUATION_ACTIONS[0].to_string())
            };
            push_unique(&mut actions, &action);
        }
        actions
    };

    Some(
        StatusReading {
            id: PROMOTION_DOMAIN_ID.to_string(),
            scope: PROMOTION_SCOPE.to_string(),
            mode: PROMOTION_MODE.to_string(),
            status: status.to_string(),
            reason_code: reason_code.to_string(),
            primary_count: resources,
            source_kinds: PROMOTION_SOURCE_KINDS
                .iter()
                .map(|item| (*item).to_string())
                .collect(),
            signal_keys,
            blockers: blockers.into_iter().map(Into::into).collect(),
            warnings: warnings.into_iter().map(Into::into).collect(),
            next_actions,
            freshness: Default::default(),
        }
        .into_project_domain_status(),
    )
}
