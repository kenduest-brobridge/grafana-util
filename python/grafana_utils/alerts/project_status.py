"""Alert live domain-status producer."""

from typing import Any, Optional, List, Tuple
from ..project_status import (
    status_finding,
    ProjectDomainStatus,
    PROJECT_STATUS_PARTIAL,
    PROJECT_STATUS_READY,
    PROJECT_STATUS_BLOCKED,
)
from ..status_model import StatusReading


ALERT_DOMAIN_ID = "alert"
ALERT_SCOPE = "live"
ALERT_MODE = "live-alert-surfaces"
ALERT_REASON_READY = PROJECT_STATUS_READY
ALERT_REASON_PARTIAL_NO_DATA = "partial-no-data"
ALERT_REASON_BLOCKED_BY_BLOCKERS = "blocked-by-blockers"

ALERT_SOURCE_KINDS = [
    "alert",
    "alert-contact-point",
    "alert-mute-timing",
    "alert-policy",
    "alert-template",
]
ALERT_SIGNAL_KEYS = [
    "live.alertRuleCount",
    "live.ruleLinkedCount",
    "live.ruleUnlinkedCount",
    "live.rulePanelMissingCount",
    "live.contactPointCount",
    "live.muteTimingCount",
    "live.policyCount",
    "live.templateCount",
]

ALERT_EXPORT_AT_LEAST_ONE_ACTIONS = ["capture at least one live alert resource"]
ALERT_LINK_AT_LEAST_ONE_RULE_ACTIONS = [
    "link at least one live alert rule to a dashboard before re-running the live alert snapshot",
]
ALERT_CAPTURE_POLICY_ACTIONS = [
    "capture at least one live alert policy before re-running the live alert snapshot"
]
ALERT_LINK_REMAINING_RULES_ACTIONS = [
    "link remaining live alert rules to dashboards before re-running the live alert snapshot"
]
ALERT_MISSING_CONTACT_POINTS_ACTIONS = [
    "capture at least one live contact point before re-running the live alert snapshot"
]
ALERT_MISSING_MUTE_TIMINGS_ACTIONS = [
    "capture at least one live mute timing before re-running the live alert snapshot"
]
ALERT_MISSING_TEMPLATES_ACTIONS = [
    "capture at least one live notification template before re-running the live alert snapshot"
]
ALERT_REFRESH_AFTER_CHANGES_ACTIONS = [
    "re-run the live alert snapshot after provisioning changes"
]
ALERT_WARNING_MISSING_CONTACT_POINTS = "missing-contact-points"
ALERT_WARNING_MISSING_MUTE_TIMINGS = "missing-mute-timings"
ALERT_WARNING_MISSING_TEMPLATES = "missing-templates"
ALERT_WARNING_MISSING_PANEL_LINKS = "missing-panel-links"
ALERT_BLOCKER_MISSING_LINKED_RULES = "missing-linked-alert-rules"
ALERT_BLOCKER_MISSING_POLICY = "missing-alert-policy"


def value_to_string(value: Any) -> str:
    if value is None:
        return ""
    if isinstance(value, str):
        return value
    import json
    return json.dumps(value)


def blocker_actions(kind: str) -> List[str]:
    if kind == ALERT_BLOCKER_MISSING_LINKED_RULES:
        return ALERT_LINK_AT_LEAST_ONE_RULE_ACTIONS
    if kind == ALERT_BLOCKER_MISSING_POLICY:
        return ALERT_CAPTURE_POLICY_ACTIONS
    return []


def array_count(document: Any) -> int:
    if isinstance(document, list):
        return len(document)
    return 0


def object_count(document: Any) -> int:
    if isinstance(document, dict) and document:
        return 1
    return 0


def rule_linkage_panel_counts(document: Any) -> Tuple[int, int, int]:
    if not isinstance(document, list):
        return (0, 0, 0)

    linked_rules = 0
    panel_linked_rules = 0

    for rule in document:
        if not isinstance(rule, dict):
            continue
        annotations = rule.get("annotations")
        if not isinstance(annotations, dict):
            continue

        has_dashboard_uid = bool(value_to_string(annotations.get("__dashboardUid__")).strip())
        if not has_dashboard_uid:
            continue

        linked_rules += 1
        has_panel_id = bool(value_to_string(annotations.get("__panelId__")).strip())
        if has_panel_id:
            panel_linked_rules += 1

    return (linked_rules, panel_linked_rules, len(document))


def push_next_actions(next_actions: List[str], actions: List[str]):
    for action in actions:
        if action not in next_actions:
            next_actions.append(action)


def add_missing_surface_signal(
    warnings: List[Any],
    next_actions: List[str],
    has_linked_rules: bool,
    count: int,
    warning_kind: str,
    source_key: str,
    action: List[str],
):
    if has_linked_rules and count == 0:
        warnings.append(status_finding(warning_kind, 1, source_key))
        push_next_actions(next_actions, action)


def add_missing_panel_link_signal(
    warnings: List[Any],
    next_actions: List[str],
    has_linked_rules: bool,
    linked_rules: int,
    panel_linked_rules: int,
):
    if has_linked_rules and panel_linked_rules < linked_rules:
        warnings.append(
            status_finding(
                ALERT_WARNING_MISSING_PANEL_LINKS,
                linked_rules - panel_linked_rules,
                "live.rulePanelMissingCount",
            )
        )
        push_next_actions(
            next_actions,
            ["capture panel IDs for linked live alert rules before promotion handoff"],
        )


ALERT_REEXPORT_AFTER_CHANGES_ACTIONS = [
    "re-run alert export after alerting changes"
]
ALERT_MISSING_POLICIES_ACTIONS = [
    "capture at least one notification policy before promotion handoff so routing drift stays reviewable"
]
ALERT_WARNING_MISSING_POLICIES = "missing-policies"


def summary_number(document: Any, key: str) -> int:
    if not isinstance(document, dict):
        return 0
    summary = document.get("summary")
    if not isinstance(summary, dict):
        return 0
    val = summary.get(key)
    if isinstance(val, (int, float)):
        return int(val)
    return 0


def add_missing_supporting_surface_signal(
    warnings: List[Any],
    next_actions: List[str],
    primary_count: int,
    count: int,
    warning_kind: str,
    source_key: str,
    action: List[str],
):
    if primary_count > 0 and count == 0:
        from ..status_model import StatusRecordCount
        warnings.append(StatusRecordCount(warning_kind, 1, source_key))
        push_next_actions(next_actions, action)


def build_alert_project_status_domain(
    summary_document: Optional[Any] = None,
) -> Optional[ProjectDomainStatus]:
    if summary_document is None:
        return None

    rules = summary_number(summary_document, "ruleCount")
    contact_points = summary_number(summary_document, "contactPointCount")
    policies = summary_number(summary_document, "policyCount")

    primary_count = max(rules, contact_points, policies)
    is_partial = primary_count == 0

    if is_partial:
        status = PROJECT_STATUS_PARTIAL
        reason_code = ALERT_REASON_PARTIAL_NO_DATA
        next_actions = list(ALERT_EXPORT_AT_LEAST_ONE_ACTIONS)
    else:
        status = PROJECT_STATUS_READY
        reason_code = ALERT_REASON_READY
        next_actions = list(ALERT_REEXPORT_AFTER_CHANGES_ACTIONS)

    warnings = []
    add_missing_supporting_surface_signal(
        warnings,
        next_actions,
        rules,
        contact_points,
        ALERT_WARNING_MISSING_CONTACT_POINTS,
        "summary.contactPointCount",
        ALERT_MISSING_CONTACT_POINTS_ACTIONS,
    )
    add_missing_supporting_surface_signal(
        warnings,
        next_actions,
        rules,
        policies,
        ALERT_WARNING_MISSING_POLICIES,
        "summary.policyCount",
        ALERT_MISSING_POLICIES_ACTIONS,
    )
    add_missing_supporting_surface_signal(
        warnings,
        next_actions,
        primary_count,
        summary_number(summary_document, "muteTimingCount"),
        ALERT_WARNING_MISSING_MUTE_TIMINGS,
        "summary.muteTimingCount",
        ALERT_MISSING_MUTE_TIMINGS_ACTIONS,
    )
    add_missing_supporting_surface_signal(
        warnings,
        next_actions,
        primary_count,
        summary_number(summary_document, "templateCount"),
        ALERT_WARNING_MISSING_TEMPLATES,
        "summary.templateCount",
        ALERT_MISSING_TEMPLATES_ACTIONS,
    )

    return StatusReading(
        id=ALERT_DOMAIN_ID,
        scope="staged",
        mode="artifact-summary",
        status=status,
        reason_code=reason_code,
        primary_count=primary_count,
        source_kinds=["alert-export"],
        signal_keys=[
            "summary.ruleCount",
            "summary.contactPointCount",
            "summary.policyCount",
            "summary.muteTimingCount",
            "summary.templateCount",
        ],
        blockers=[],
        warnings=warnings,
        next_actions=next_actions,
    ).into_project_domain_status()


def build_alert_live_project_status_domain(
    rules_document: Optional[Any] = None,
    contact_points_document: Optional[Any] = None,
    mute_timings_document: Optional[Any] = None,
    policies_document: Optional[Any] = None,
    templates_document: Optional[Any] = None,
) -> Optional[ProjectDomainStatus]:
    rules = array_count(rules_document)
    linked_rules, panel_linked_rules, rule_count = rule_linkage_panel_counts(rules_document)
    contact_points = array_count(contact_points_document)
    mute_timings = array_count(mute_timings_document)
    policies = object_count(policies_document)
    templates = array_count(templates_document)

    source_kinds = []
    if rules_document is not None:
        source_kinds.append(ALERT_SOURCE_KINDS[0])
    if contact_points_document is not None:
        source_kinds.append(ALERT_SOURCE_KINDS[1])
    if mute_timings_document is not None:
        source_kinds.append(ALERT_SOURCE_KINDS[2])
    if policies_document is not None:
        source_kinds.append(ALERT_SOURCE_KINDS[3])
    if templates_document is not None:
        source_kinds.append(ALERT_SOURCE_KINDS[4])

    if not source_kinds:
        return None

    primary_count = rules + contact_points + mute_timings + policies + templates
    unlinked_rules = max(0, rule_count - linked_rules)
    has_linked_rules = linked_rules > 0
    blockers = []
    warnings = []

    if rules > 0 and not has_linked_rules:
        blockers.append(
            status_finding(
                ALERT_BLOCKER_MISSING_LINKED_RULES,
                1,
                "live.ruleLinkedCount",
            )
        )
    elif unlinked_rules > 0:
        warnings.append(
            status_finding(
                "unlinked-alert-rules",
                unlinked_rules,
                "live.ruleUnlinkedCount",
            )
        )

    if has_linked_rules and policies == 0:
        blockers.append(
            status_finding(
                ALERT_BLOCKER_MISSING_POLICY,
                1,
                "live.policyCount",
            )
        )

    if rules == 0:
        status = PROJECT_STATUS_PARTIAL
        reason_code = ALERT_REASON_PARTIAL_NO_DATA
        next_actions = list(ALERT_EXPORT_AT_LEAST_ONE_ACTIONS)
    elif blockers:
        status = PROJECT_STATUS_BLOCKED
        reason_code = ALERT_REASON_BLOCKED_BY_BLOCKERS
        next_actions = []
        for b in blockers:
            next_actions.extend(blocker_actions(b.kind))
    elif unlinked_rules > 0:
        status = PROJECT_STATUS_READY
        reason_code = ALERT_REASON_READY
        next_actions = list(ALERT_LINK_REMAINING_RULES_ACTIONS)
    else:
        status = PROJECT_STATUS_READY
        reason_code = ALERT_REASON_READY
        next_actions = list(ALERT_REFRESH_AFTER_CHANGES_ACTIONS)

    add_missing_surface_signal(
        warnings,
        next_actions,
        has_linked_rules,
        contact_points,
        ALERT_WARNING_MISSING_CONTACT_POINTS,
        "live.contactPointCount",
        ALERT_MISSING_CONTACT_POINTS_ACTIONS,
    )
    add_missing_surface_signal(
        warnings,
        next_actions,
        has_linked_rules,
        mute_timings,
        ALERT_WARNING_MISSING_MUTE_TIMINGS,
        "live.muteTimingCount",
        ALERT_MISSING_MUTE_TIMINGS_ACTIONS,
    )
    add_missing_surface_signal(
        warnings,
        next_actions,
        has_linked_rules,
        templates,
        ALERT_WARNING_MISSING_TEMPLATES,
        "live.templateCount",
        ALERT_MISSING_TEMPLATES_ACTIONS,
    )
    add_missing_panel_link_signal(
        warnings,
        next_actions,
        has_linked_rules,
        linked_rules,
        panel_linked_rules,
    )

    from ..status_model import StatusRecordCount

    return StatusReading(
        id=ALERT_DOMAIN_ID,
        scope=ALERT_SCOPE,
        mode=ALERT_MODE,
        status=status,
        reason_code=reason_code,
        primary_count=primary_count,
        source_kinds=source_kinds,
        signal_keys=ALERT_SIGNAL_KEYS,
        blockers=[StatusRecordCount(b.kind, b.count, b.source) for b in blockers],
        warnings=[StatusRecordCount(w.kind, w.count, w.source) for w in warnings],
        next_actions=next_actions,
    ).into_project_domain_status()
