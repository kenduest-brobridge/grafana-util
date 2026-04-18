"""Access domain-status producer."""

from typing import Any, Optional, List, Dict
from ..project_status import (
    status_finding,
    ProjectDomainStatus,
    PROJECT_STATUS_PARTIAL,
    PROJECT_STATUS_READY,
)
from ..status_model import StatusReading, StatusRecordCount


ACCESS_DOMAIN_ID = "access"
ACCESS_SCOPE = "staged"
ACCESS_MODE = "staged-export-bundles"
ACCESS_REASON_READY = PROJECT_STATUS_READY
ACCESS_REASON_PARTIAL_NO_DATA = "partial-no-data"
ACCESS_REASON_PARTIAL_MISSING_BUNDLES = "partial-missing-bundles"

ACCESS_EXPORT_KIND_USERS = "grafana-utils-access-user-export-index"
ACCESS_EXPORT_KIND_TEAMS = "grafana-utils-access-team-export-index"
ACCESS_EXPORT_KIND_ORGS = "grafana-utils-access-org-export-index"
ACCESS_EXPORT_KIND_SERVICE_ACCOUNTS = "grafana-utils-access-service-account-export-index"

ACCESS_SIGNAL_KEYS = [
    "summary.users.recordCount",
    "summary.teams.recordCount",
    "summary.orgs.recordCount",
    "summary.serviceAccounts.recordCount",
]

ACCESS_EXPORT_USERS_LABEL = "users"
ACCESS_EXPORT_TEAMS_LABEL = "teams"
ACCESS_EXPORT_ORGS_LABEL = "orgs"
ACCESS_EXPORT_SERVICE_ACCOUNTS_LABEL = "service accounts"

ACCESS_MISSING_BUNDLE_KIND = "missing-bundle-kind"
ACCESS_READY_NEXT_ACTIONS = ["re-run access export after membership changes"]
ACCESS_NO_DATA_NEXT_ACTIONS = [
    "export at least one access user, team, org, or service-account record"
]
ACCESS_MIXED_WORKSPACE_REVIEW_ACTIONS = [
    "add the missing access export bundles before using this workspace as one mixed workspace handoff",
    "re-run workspace test after the missing access bundles are exported",
]


class AccessBundleSpec:
    def __init__(self, kind: str, signal_key: str, label: str):
        self.kind = kind
        self.signal_key = signal_key
        self.label = label


ACCESS_BUNDLE_SPECS = [
    AccessBundleSpec(ACCESS_EXPORT_KIND_USERS, "summary.users.recordCount", ACCESS_EXPORT_USERS_LABEL),
    AccessBundleSpec(ACCESS_EXPORT_KIND_TEAMS, "summary.teams.recordCount", ACCESS_EXPORT_TEAMS_LABEL),
    AccessBundleSpec(ACCESS_EXPORT_KIND_ORGS, "summary.orgs.recordCount", ACCESS_EXPORT_ORGS_LABEL),
    AccessBundleSpec(ACCESS_EXPORT_KIND_SERVICE_ACCOUNTS, "summary.serviceAccounts.recordCount", ACCESS_EXPORT_SERVICE_ACCOUNTS_LABEL),
]


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


def build_access_domain_status(
    user_export_document: Optional[Any] = None,
    team_export_document: Optional[Any] = None,
    org_export_document: Optional[Any] = None,
    service_account_export_document: Optional[Any] = None,
) -> Optional[ProjectDomainStatus]:
    source_kinds = []
    warnings = []
    total_records = 0
    missing_labels = []

    inputs = {
        ACCESS_EXPORT_KIND_USERS: user_export_document,
        ACCESS_EXPORT_KIND_TEAMS: team_export_document,
        ACCESS_EXPORT_KIND_ORGS: org_export_document,
        ACCESS_EXPORT_KIND_SERVICE_ACCOUNTS: service_account_export_document,
    }

    for spec in ACCESS_BUNDLE_SPECS:
        document = inputs.get(spec.kind)
        if document is not None:
            source_kinds.append(spec.kind)
            total_records += summary_number(document, "recordCount")
        else:
            missing_labels.append(spec.label)
            warnings.append(StatusRecordCount(ACCESS_MISSING_BUNDLE_KIND, 1, spec.signal_key))

    if not source_kinds:
        return None

    if missing_labels:
        next_actions = [f"export the missing access bundle kinds: {', '.join(missing_labels)}"]
        next_actions.extend(ACCESS_MIXED_WORKSPACE_REVIEW_ACTIONS)
        status = PROJECT_STATUS_PARTIAL
        reason_code = ACCESS_REASON_PARTIAL_MISSING_BUNDLES
    elif total_records == 0:
        status = PROJECT_STATUS_PARTIAL
        reason_code = ACCESS_REASON_PARTIAL_NO_DATA
        next_actions = list(ACCESS_NO_DATA_NEXT_ACTIONS)
    else:
        status = PROJECT_STATUS_READY
        reason_code = ACCESS_REASON_READY
        next_actions = list(ACCESS_READY_NEXT_ACTIONS)

    return StatusReading(
        id=ACCESS_DOMAIN_ID,
        scope=ACCESS_SCOPE,
        mode=ACCESS_MODE,
        status=status,
        reason_code=reason_code,
        primary_count=total_records,
        source_kinds=source_kinds,
        signal_keys=ACCESS_SIGNAL_KEYS,
        blockers=[],
        warnings=warnings,
        next_actions=next_actions,
    ).into_project_domain_status()
