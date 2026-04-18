"""Live sync domain-status producer."""

from typing import Any, Optional, List, Dict
from ..project_status import (
    status_finding,
    ProjectDomainStatus,
    PROJECT_STATUS_BLOCKED,
    PROJECT_STATUS_PARTIAL,
    PROJECT_STATUS_READY,
)
from ..status_model import StatusReading, StatusRecordCount


SYNC_DOMAIN_ID = "sync"
SYNC_SCOPE = "live"
SYNC_MODE = "live-sync-surfaces"

SYNC_REASON_READY = PROJECT_STATUS_READY
SYNC_REASON_PARTIAL_NO_DATA = "partial-no-data"
SYNC_REASON_PARTIAL_MISSING_SURFACES = "partial-missing-surfaces"
SYNC_REASON_BLOCKED_BY_BLOCKERS = "blocked-by-blockers"

SYNC_SOURCE_KIND_SUMMARY = "sync-summary"
SYNC_SOURCE_KIND_BUNDLE_PREFLIGHT = "package-test"

SYNC_SIGNAL_KEYS_SUMMARY = ["summary.resourceCount"]
SYNC_SIGNAL_KEYS_BUNDLE_PREFLIGHT = [
    "summary.resourceCount",
    "summary.syncBlockingCount",
    "summary.providerBlockingCount",
    "summary.secretPlaceholderBlockingCount",
    "summary.alertArtifactCount",
    "summary.alertArtifactBlockedCount",
    "summary.alertArtifactPlanOnlyCount",
    "summary.blockedCount",
    "summary.planOnlyCount",
]

SYNC_BLOCKER_SYNC_BLOCKING = "sync-blocking"
SYNC_BLOCKER_PROVIDER_BLOCKING = "provider-blocking"
SYNC_BLOCKER_SECRET_PLACEHOLDER_BLOCKING = "secret-placeholder-blocking"
SYNC_BLOCKER_ALERT_ARTIFACT_BLOCKING = "alert-artifact-blocking"
SYNC_BLOCKER_BUNDLE_BLOCKING = "bundle-blocking"
SYNC_WARNING_ALERT_ARTIFACT_PLAN_ONLY = "alert-artifact-plan-only"
SYNC_WARNING_BUNDLE_PLAN_ONLY = "bundle-plan-only"

SYNC_RESOLVE_BLOCKERS_ACTIONS = [
    "resolve sync workflow blockers in the fixed order: sync, provider, secret-placeholder, alert-artifact",
]
SYNC_STAGE_AT_LEAST_ONE_ACTIONS = [
    "stage at least one dashboard, datasource, or alert resource"
]
SYNC_PROVIDE_PREFLIGHT_ACTIONS = [
    "provide a staged package-test document before interpreting live sync readiness"
]
SYNC_REVIEW_NON_BLOCKING_ACTIONS = [
    "review non-blocking sync findings before promotion or apply"
]
SYNC_REEXPORT_AFTER_CHANGES_ACTIONS = ["re-run sync summary after staged changes"]


def summary_number(document: Any, key: str) -> int:
    if not isinstance(document, dict):
        return 0
    summary = document.get("summary", {})
    val = summary.get(key, 0)
    return int(val) if isinstance(val, (int, float)) else 0


def next_actions_for_partial(resources: int, has_bundle_preflight: bool) -> List[str]:
    if resources == 0:
        return list(SYNC_STAGE_AT_LEAST_ONE_ACTIONS)
    elif has_bundle_preflight:
        return list(SYNC_REEXPORT_AFTER_CHANGES_ACTIONS)
    else:
        return list(SYNC_PROVIDE_PREFLIGHT_ACTIONS)


def next_actions_for_ready(warnings_present: bool) -> List[str]:
    if warnings_present:
        return list(SYNC_REVIEW_NON_BLOCKING_ACTIONS)
    else:
        return list(SYNC_REEXPORT_AFTER_CHANGES_ACTIONS)


def build_live_sync_domain_status(
    summary_document: Optional[Dict[str, Any]] = None,
    bundle_preflight_document: Optional[Dict[str, Any]] = None,
) -> Optional[ProjectDomainStatus]:
    if summary_document is None and bundle_preflight_document is None:
        return None

    source_kinds = []
    signal_keys = []
    if summary_document is not None:
        source_kinds.append(SYNC_SOURCE_KIND_SUMMARY)

    blockers = []
    warnings = []

    if bundle_preflight_document is not None:
        source_kinds.append(SYNC_SOURCE_KIND_BUNDLE_PREFLIGHT)
        signal_keys.extend(SYNC_SIGNAL_KEYS_BUNDLE_PREFLIGHT)

        resources = summary_number(bundle_preflight_document, "resourceCount")
        sync_blocking = summary_number(bundle_preflight_document, "syncBlockingCount")
        provider_blocking = summary_number(bundle_preflight_document, "providerBlockingCount")
        secret_blocking = summary_number(bundle_preflight_document, "secretPlaceholderBlockingCount")
        alert_blocking = summary_number(bundle_preflight_document, "alertArtifactBlockedCount")
        alert_plan_only = summary_number(bundle_preflight_document, "alertArtifactPlanOnlyCount")
        bundle_blocking = summary_number(bundle_preflight_document, "blockedCount")
        bundle_plan_only = summary_number(bundle_preflight_document, "planOnlyCount")

        if sync_blocking > 0:
            blockers.append(StatusRecordCount(SYNC_BLOCKER_SYNC_BLOCKING, sync_blocking, "summary.syncBlockingCount"))
        if provider_blocking > 0:
            blockers.append(StatusRecordCount(SYNC_BLOCKER_PROVIDER_BLOCKING, provider_blocking, "summary.providerBlockingCount"))
        if secret_blocking > 0:
            blockers.append(StatusRecordCount(SYNC_BLOCKER_SECRET_PLACEHOLDER_BLOCKING, secret_blocking, "summary.secretPlaceholderBlockingCount"))
        if alert_blocking > 0:
            blockers.append(StatusRecordCount(SYNC_BLOCKER_ALERT_ARTIFACT_BLOCKING, alert_blocking, "summary.alertArtifactBlockedCount"))
        if not blockers and bundle_blocking > 0:
            blockers.append(StatusRecordCount(SYNC_BLOCKER_BUNDLE_BLOCKING, bundle_blocking, "summary.blockedCount"))

        if alert_plan_only > 0:
            warnings.append(StatusRecordCount(SYNC_WARNING_ALERT_ARTIFACT_PLAN_ONLY, alert_plan_only, "summary.alertArtifactPlanOnlyCount"))
        elif bundle_plan_only > 0:
            warnings.append(StatusRecordCount(SYNC_WARNING_BUNDLE_PLAN_ONLY, bundle_plan_only, "summary.planOnlyCount"))

        has_blockers = len(blockers) > 0
        has_warnings = len(warnings) > 0
        if has_blockers:
            next_actions = list(SYNC_RESOLVE_BLOCKERS_ACTIONS)
            status = PROJECT_STATUS_BLOCKED
            reason_code = SYNC_REASON_BLOCKED_BY_BLOCKERS
        elif resources == 0:
            next_actions = next_actions_for_partial(resources, True)
            status = PROJECT_STATUS_PARTIAL
            reason_code = SYNC_REASON_PARTIAL_NO_DATA
        else:
            next_actions = next_actions_for_ready(has_warnings)
            status = PROJECT_STATUS_READY
            reason_code = SYNC_REASON_READY
    else:
        resources = summary_number(summary_document, "resourceCount") if summary_document else 0
        signal_keys.extend(SYNC_SIGNAL_KEYS_SUMMARY)
        next_actions = next_actions_for_partial(resources, False)
        status = PROJECT_STATUS_PARTIAL
        reason_code = SYNC_REASON_PARTIAL_NO_DATA if resources == 0 else SYASON_PARTIAL_MISSING_SURFACES
        if resources > 0:
            reason_code = SYNC_REASON_PARTIAL_MISSING_SURFACES

    return StatusReading(
        id=SYNC_DOMAIN_ID,
        scope=SYNC_SCOPE,
        mode=SYNC_MODE,
        status=status,
        reason_code=reason_code,
        primary_count=resources,
        source_kinds=source_kinds,
        signal_keys=signal_keys,
        blockers=blockers,
        warnings=warnings,
        next_actions=next_actions,
    ).into_project_domain_status()
