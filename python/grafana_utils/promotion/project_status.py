"""Live promotion domain-status producer."""

from typing import Any, Optional, List, Dict
from ..project_status import (
    status_finding,
    ProjectDomainStatus,
    PROJECT_STATUS_BLOCKED,
    PROJECT_STATUS_PARTIAL,
    PROJECT_STATUS_READY,
)
from ..status_model import StatusReading, StatusRecordCount


LIVE_PROMOTION_DOMAIN_ID = "promotion"
LIVE_PROMOTION_SCOPE = "live"
LIVE_PROMOTION_MODE = "live-promotion-surfaces"
LIVE_PROMOTION_REASON_READY = PROJECT_STATUS_READY
LIVE_PROMOTION_REASON_PARTIAL_NO_DATA = "partial-no-data"
LIVE_PROMOTION_REASON_BLOCKED_BY_BLOCKERS = "blocked-by-blockers"

LIVE_PROMOTION_SOURCE_KINDS = [
    "live-promotion-summary",
    "live-promotion-mapping",
    "live-promotion-availability",
]

LIVE_PROMOTION_SIGNAL_KEYS = [
    "summary.resourceCount",
    "summary.missingMappingCount",
    "summary.bundleBlockingCount",
    "summary.blockingCount",
    "mapping.entryCount",
    "availability.entryCount",
    "handoffSummary.reviewRequired",
    "handoffSummary.readyForReview",
    "handoffSummary.nextStage",
    "handoffSummary.blockingCount",
    "handoffSummary.reviewInstruction",
    "continuationSummary.stagedOnly",
    "continuationSummary.liveMutationAllowed",
    "continuationSummary.readyForContinuation",
    "continuationSummary.nextStage",
    "continuationSummary.blockingCount",
    "continuationSummary.continuationInstruction",
]

LIVE_PROMOTION_BLOCKER_MISSING_MAPPINGS = "missing-mappings"
LIVE_PROMOTION_BLOCKER_BUNDLE_BLOCKING = "bundle-blocking"
LIVE_PROMOTION_BLOCKER_BLOCKING = "blocking"
LIVE_PROMOTION_WARNING_REVIEW_HANDOFF = "review-handoff"
LIVE_PROMOTION_WARNING_APPLY_CONTINUATION = "apply-continuation"

LIVE_PROMOTION_RESOLVE_BLOCKERS_ACTIONS = [
    "resolve promotion blockers in the fixed order: missing-mapping, bundle-blocking, blocking"
]
LIVE_PROMOTION_STAGE_AT_LEAST_ONE_ACTIONS = [
    "stage at least one promotable resource before promotion"
]
LIVE_PROMOTION_PROVIDE_SUMMARY_ACTIONS = [
    "provide a staged promotion summary before interpreting live promotion readiness"
]
LIVE_PROMOTION_PROVIDE_MAPPING_ACTIONS = [
    "provide explicit promotion mappings before promotion"
]
LIVE_PROMOTION_PROVIDE_AVAILABILITY_ACTIONS = [
    "provide live availability hints before promotion"
]
LIVE_PROMOTION_REVIEW_HANOFF_ACTIONS = [
    "resolve the staged promotion handoff before review"
]
LIVE_PROMOTION_REVIEW_READY_ACTIONS = ["promotion handoff is review-ready"]
LIVE_PROMOTION_APPLY_CONTINUATION_ACTIONS = [
    "keep the promotion staged until the apply continuation is ready"
]
LIVE_PROMOTION_APPLY_READY_ACTIONS = [
    "promotion is apply-ready in the staged continuation"
]


def summary_number(document: Any, key: str) -> int:
    if not isinstance(document, dict):
        return 0
    summary = document.get("summary", {})
    val = summary.get(key, 0)
    return int(val) if isinstance(val, (int, float)) else 0


def array_count(document: Any) -> int:
    return len(document) if isinstance(document, list) else 0


def push_unique(next_actions: List[str], action: str):
    if action not in next_actions:
        next_actions.append(action)


def mapping_entry_count(document: Optional[Dict[str, Any]]) -> int:
    if not isinstance(document, dict):
        return 0

    folders = len(document.get("folders", {})) if isinstance(document.get("folders"), dict) else 0
    
    datasources = document.get("datasources", {})
    if not isinstance(datasources, dict):
        ds_uids = 0
        ds_names = 0
    else:
        ds_uids = len(datasources.get("uids", {})) if isinstance(datasources.get("uids"), dict) else 0
        ds_names = len(datasources.get("names", {})) if isinstance(datasources.get("names"), dict) else 0

    return folders + ds_uids + ds_names


def availability_entry_count(document: Optional[Dict[str, Any]]) -> int:
    if not isinstance(document, dict):
        return 0

    keys = [
        "pluginIds",
        "datasourceUids",
        "datasourceNames",
        "contactPoints",
        "providerNames",
        "secretPlaceholderNames",
    ]
    return sum(array_count(document.get(key)) for key in keys)


def summary_bool(document: Optional[Dict[str, Any]], section: str, key: str) -> bool:
    if not isinstance(document, dict):
        return False
    sec = document.get(section, {})
    if not isinstance(sec, dict):
        return False
    val = sec.get(key)
    return bool(val) if isinstance(val, bool) else False


def summary_text(document: Optional[Dict[str, Any]], section: str, key: str) -> Optional[str]:
    if not isinstance(document, dict):
        return None
    sec = document.get(section, {})
    if not isinstance(sec, dict):
        return None
    val = sec.get(key)
    if isinstance(val, str) and val.strip():
        return val.strip()
    return None


def add_handoff_evidence(
    document: Optional[Dict[str, Any]],
    warnings: List[StatusRecordCount],
    next_actions: List[str],
):
    if not isinstance(document, dict) or "handoffSummary" not in document:
        return

    warnings.append(StatusRecordCount(
        LIVE_PROMOTION_WARNING_REVIEW_HANDOFF,
        1,
        "handoffSummary.reviewRequired",
    ))

    if summary_bool(document, "handoffSummary", "readyForReview"):
        for action in LIVE_PROMOTION_REVIEW_READY_ACTIONS:
            push_unique(next_actions, action)
    else:
        action = summary_text(document, "handoffSummary", "reviewInstruction") or LIVE_PROMOTION_REVIEW_HANOFF_ACTIONS[0]
        push_unique(next_actions, action)


def add_continuation_evidence(
    document: Optional[Dict[str, Any]],
    warnings: List[StatusRecordCount],
    next_actions: List[str],
):
    if not isinstance(document, dict) or "continuationSummary" not in document:
        return

    warnings.append(StatusRecordCount(
        LIVE_PROMOTION_WARNING_APPLY_CONTINUATION,
        1,
        "continuationSummary.liveMutationAllowed",
    ))

    if summary_bool(document, "continuationSummary", "readyForContinuation"):
        for action in LIVE_PROMOTION_APPLY_READY_ACTIONS:
            push_unique(next_actions, action)
    else:
        action = summary_text(document, "continuationSummary", "continuationInstruction") or LIVE_PROMOTION_APPLY_CONTINUATION_ACTIONS[0]
        push_unique(next_actions, action)


def build_next_actions(
    summary_present: bool,
    mapping_present: bool,
    availability_present: bool,
    resource_count: int,
    mapping_count: int,
    availability_count: int,
) -> List[str]:
    next_actions = []

    if not summary_present:
        for action in LIVE_PROMOTION_PROVIDE_SUMMARY_ACTIONS:
            push_unique(next_actions, action)
    if resource_count == 0:
        for action in LIVE_PROMOTION_STAGE_AT_LEAST_ONE_ACTIONS:
            push_unique(next_actions, action)
    if not mapping_present or mapping_count == 0:
        for action in LIVE_PROMOTION_PROVIDE_MAPPING_ACTIONS:
            push_unique(next_actions, action)
    if not availability_present or availability_count == 0:
        for action in LIVE_PROMOTION_PROVIDE_AVAILABILITY_ACTIONS:
            push_unique(next_actions, action)

    return next_actions


def build_live_promotion_project_status(
    promotion_summary_document: Optional[Dict[str, Any]] = None,
    promotion_mapping_document: Optional[Dict[str, Any]] = None,
    availability_document: Optional[Dict[str, Any]] = None,
) -> Optional[ProjectDomainStatus]:
    if promotion_summary_document is None and promotion_mapping_document is None and availability_document is None:
        return None

    resource_count = summary_number(promotion_summary_document, "resourceCount") if promotion_summary_document else 0
    summary_present = promotion_summary_document is not None
    missing_mapping_count = summary_number(promotion_summary_document, "missingMappingCount") if promotion_summary_document else 0
    bundle_blocking_count = summary_number(promotion_summary_document, "bundleBlockingCount") if promotion_summary_document else 0
    blocking_count = summary_number(promotion_summary_document, "blockingCount") if promotion_summary_document else 0
    mapping_count = mapping_entry_count(promotion_mapping_document)
    availability_count = availability_entry_count(availability_document)

    source_kinds = []
    if promotion_summary_document is not None:
        source_kinds.append(LIVE_PROMOTION_SOURCE_KINDS[0])
    if promotion_mapping_document is not None:
        source_kinds.append(LIVE_PROMOTION_SOURCE_KINDS[1])
    if availability_document is not None:
        source_kinds.append(LIVE_PROMOTION_SOURCE_KINDS[2])

    blockers = []
    if missing_mapping_count > 0:
        blockers.append(StatusRecordCount(LIVE_PROMOTION_BLOCKER_MISSING_MAPPINGS, missing_mapping_count, "summary.missingMappingCount"))
    if bundle_blocking_count > 0:
        blockers.append(StatusRecordCount(LIVE_PROMOTION_BLOCKER_BUNDLE_BLOCKING, bundle_blocking_count, "summary.bundleBlockingCount"))
    if not blockers and blocking_count > 0:
        blockers.append(StatusRecordCount(LIVE_PROMOTION_BLOCKER_BLOCKING, blocking_count, "summary.blockingCount"))

    warnings = []
    evidence_actions = []
    add_handoff_evidence(promotion_summary_document, warnings, evidence_actions)
    add_continuation_evidence(promotion_summary_document, warnings, evidence_actions)

    if blockers:
        status = PROJECT_STATUS_BLOCKED
        reason_code = LIVE_PROMOTION_REASON_BLOCKED_BY_BLOCKERS
        next_actions = list(LIVE_PROMOTION_RESOLVE_BLOCKERS_ACTIONS)
    elif not summary_present or resource_count == 0 or mapping_count == 0 or availability_count == 0:
        next_actions = build_next_actions(
            summary_present,
            promotion_mapping_document is not None,
            availability_document is not None,
            resource_count,
            mapping_count,
            availability_count,
        )
        next_actions.extend(evidence_actions)
        status = PROJECT_STATUS_PARTIAL
        reason_code = LIVE_PROMOTION_REASON_PARTIAL_NO_DATA
    else:
        status = PROJECT_STATUS_READY
        reason_code = LIVE_PROMOTION_REASON_READY
        next_actions = evidence_actions

    return StatusReading(
        id=LIVE_PROMOTION_DOMAIN_ID,
        scope=LIVE_PROMOTION_SCOPE,
        mode=LIVE_PROMOTION_MODE,
        status=status,
        reason_code=reason_code,
        primary_count=resource_count,
        source_kinds=source_kinds,
        signal_keys=LIVE_PROMOTION_SIGNAL_KEYS,
        blockers=blockers,
        warnings=warnings,
        next_actions=next_actions,
    ).into_project_domain_status()
