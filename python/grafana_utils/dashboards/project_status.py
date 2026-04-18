"""Dashboard domain-status producer."""

from typing import Any, Optional, List, Dict, Set
from ..project_status import (
    status_finding,
    ProjectDomainStatus,
    PROJECT_STATUS_BLOCKED,
    PROJECT_STATUS_PARTIAL,
    PROJECT_STATUS_READY,
)
from ..status_model import StatusReading, StatusRecordCount


DASHBOARD_DOMAIN_ID = "dashboard"
DASHBOARD_SCOPE_STAGED = "staged"
DASHBOARD_MODE_STAGED = "inspect-summary"
DASHBOARD_SCOPE_LIVE = "live"
DASHBOARD_MODE_LIVE = "live-dashboard-read"

DASHBOARD_REASON_READY = PROJECT_STATUS_READY
DASHBOARD_REASON_PARTIAL_NO_DATA = "partial-no-data"
DASHBOARD_REASON_PARTIAL_NO_DATASOURCES = "partial-no-datasources"
DASHBOARD_REASON_BLOCKED_BY_BLOCKERS = "blocked-by-blockers"

DASHBOARD_SOURCE_KINDS_STAGED = ["dashboard-export"]
DASHBOARD_SIGNAL_KEYS_STAGED = [
    "summary.dashboardCount",
    "summary.queryCount",
    "summary.orphanedDatasourceCount",
    "summary.mixedDatasourceDashboardCount",
]

DASHBOARD_SOURCE_KINDS_LIVE = ["live-dashboard-search", "live-datasource-list"]
DASHBOARD_SIGNAL_KEYS_LIVE = [
    "live.dashboardCount",
    "live.datasourceCount",
    "live.folderCount",
    "live.dashboardTitleGapCount",
    "live.folderTitleGapCount",
    "live.importReadyDashboardCount",
]

DASHBOARD_ORPHANED_DATASOURCES_KEY = "orphanedDatasources"
DASHBOARD_MIXED_DASHBOARDS_KEY = "mixedDatasourceDashboards"
DASHBOARD_DASHBOARD_GOVERNANCE_KEY = "dashboardGovernance"
DASHBOARD_DATASOURCE_GOVERNANCE_KEY = "datasourceGovernance"
DASHBOARD_DASHBOARD_DEPENDENCIES_KEY = "dashboardDependencies"

DASHBOARD_BLOCKER_ORPHANED_DATASOURCES = "orphaned-datasources"
DASHBOARD_BLOCKER_MIXED_DASHBOARDS = "mixed-dashboards"
DASHBOARD_WARNING_RISK_RECORDS = "risk-records"
DASHBOARD_WARNING_HIGH_BLAST_RADIUS_DATASOURCES = "high-blast-radius-datasources"
DASHBOARD_WARNING_QUERY_AUDITS = "query-audits"
DASHBOARD_WARNING_DASHBOARD_AUDITS = "dashboard-audits"
DASHBOARD_WARNING_IMPORT_READINESS_GAPS = "import-readiness-gaps"
DASHBOARD_WARNING_IMPORT_READINESS_DETAIL_GAPS = "import-readiness-detail-gaps"

DASHBOARD_EXPORT_AT_LEAST_ONE_ACTIONS = ["export at least one dashboard"]
DASHBOARD_REVIEW_GOVERNANCE_WARNINGS_ACTIONS = [
    "review dashboard governance warnings before promotion or apply"
]
DASHBOARD_RESOLVE_BLOCKERS_ACTIONS = [
    "resolve orphaned datasources, then mixed dashboards"
]

DASHBOARD_READY_NEXT_ACTIONS_LIVE = [
    "re-run live dashboard read after dashboard, folder, or datasource changes"
]
DASHBOARD_NO_DATA_NEXT_ACTIONS_LIVE = [
    "create or import at least one dashboard, then re-run live dashboard read"
]
DASHBOARD_NO_DATASOURCES_NEXT_ACTIONS_LIVE = [
    "create or import at least one datasource, then re-run live dashboard read"
]
DASHBOARD_REVIEW_WARNING_NEXT_ACTIONS_LIVE = [
    "review live dashboard governance and import-readiness warnings before re-running live dashboard read"
]

DASHBOARD_DATASOURCE_EMPTY_SIGNAL_KEY = "live.datasourceInventoryEmpty"
DASHBOARD_DATASOURCE_DRIFT_SIGNAL_KEY = "live.datasourceDriftCount"
DASHBOARD_FOLDER_SPREAD_SIGNAL_KEY = "live.folderSpreadCount"
DASHBOARD_ROOT_SCOPE_SIGNAL_KEY = "live.rootDashboardCount"
DASHBOARD_TITLE_GAP_SIGNAL_KEY = "live.dashboardTitleGapCount"
DASHBOARD_FOLDER_TITLE_GAP_SIGNAL_KEY = "live.folderTitleGapCount"
DASHBOARD_IMPORT_READY_SIGNAL_KEY = "live.importReadyDashboardCount"

DASHBOARD_WARNING_KIND_EMPTY_DATASOURCE_INVENTORY = "live-datasource-inventory-empty"
DASHBOARD_WARNING_KIND_DATASOURCE_DRIFT = "live-datasource-inventory-drift"
DASHBOARD_WARNING_KIND_FOLDER_SPREAD = "live-dashboard-folder-spread"
DASHBOARD_WARNING_KIND_ROOT_SCOPE = "live-dashboard-root-scope"
DASHBOARD_WARNING_KIND_TITLE_GAP = "live-dashboard-title-gap"
DASHBOARD_WARNING_KIND_FOLDER_TITLE_GAP = "live-dashboard-folder-title-gap"


def summary_number(document: Dict[str, Any], key: str) -> int:
    summary = document.get("summary", {})
    val = summary.get(key, 0)
    return int(val) if isinstance(val, (int, float)) else 0


def detail_source_key(document_key: str, item: Dict[str, Any], index: int, fields: List[str]) -> str:
    for field in fields:
        val = item.get(field)
        if isinstance(val, str) and val.strip():
            return f"{document_key}.{val.strip()}"
    return f"{document_key}[{index}]"


def push_detail_findings(
    findings: List[StatusRecordCount],
    signal_keys: Set[str],
    kind: str,
    document_key: str,
    items: Optional[List[Dict[str, Any]]],
    fields: List[str],
    fallback_count: int,
    fallback_source: str,
):
    if items:
        signal_keys.add(document_key)
        for index, item in enumerate(items):
            source = detail_source_key(document_key, item, index, fields)
            findings.append(StatusRecordCount(kind, 1, source))
            signal_keys.add(source)
    elif fallback_count > 0:
        findings.append(StatusRecordCount(kind, fallback_count, fallback_source))
        signal_keys.add(fallback_source)


def push_detail_findings_with_count(
    findings: List[StatusRecordCount],
    signal_keys: Set[str],
    kind: str,
    document_key: str,
    items: Optional[List[Dict[str, Any]]],
    fields: List[str],
    fallback_count: int,
    fallback_source: str,
    count_for_item_key: str,
):
    if items:
        signal_keys.add(document_key)
        used_detail = False
        for index, item in enumerate(items):
            count = item.get(count_for_item_key, 0)
            if not isinstance(count, (int, float)) or count <= 0:
                continue
            source = detail_source_key(document_key, item, index, fields)
            findings.append(StatusRecordCount(kind, int(count), source))
            signal_keys.add(source)
            used_detail = True
        if used_detail:
            return

    if fallback_count > 0:
        findings.append(StatusRecordCount(kind, fallback_count, fallback_source))
        signal_keys.add(fallback_source)


def build_dashboard_domain_status(
    summary_document: Optional[Dict[str, Any]] = None,
) -> Optional[ProjectDomainStatus]:
    if summary_document is None:
        return None

    dashboards = summary_number(summary_document, "dashboardCount")
    queries = summary_number(summary_document, "queryCount")
    orphaned = summary_number(summary_document, "orphanedDatasourceCount")
    mixed = summary_number(summary_document, "mixedDatasourceDashboardCount")
    risk_records = summary_number(summary_document, "riskRecordCount")
    high_blast_radius_datasources = summary_number(summary_document, "highBlastRadiusDatasourceCount")
    query_audits = summary_number(summary_document, "queryAuditCount")
    dashboard_audits = summary_number(summary_document, "dashboardAuditCount")

    orphaned_items = summary_document.get(DASHBOARD_ORPHANED_DATASOURCES_KEY)
    mixed_items = summary_document.get(DASHBOARD_MIXED_DASHBOARDS_KEY)
    dashboard_gov_items = summary_document.get(DASHBOARD_DASHBOARD_GOVERNANCE_KEY)
    datasource_gov_items = summary_document.get(DASHBOARD_DATASOURCE_GOVERNANCE_KEY)
    dependency_items = summary_document.get(DASHBOARD_DASHBOARD_DEPENDENCIES_KEY)

    blockers = []
    warnings = []
    signal_keys = set(DASHBOARD_SIGNAL_KEYS_STAGED)

    push_detail_findings(
        blockers, signal_keys, DASHBOARD_BLOCKER_ORPHANED_DATASOURCES,
        DASHBOARD_ORPHANED_DATASOURCES_KEY, orphaned_items, ["uid", "name"],
        orphaned, "summary.orphanedDatasourceCount"
    )
    push_detail_findings(
        blockers, signal_keys, DASHBOARD_BLOCKER_MIXED_DASHBOARDS,
        DASHBOARD_MIXED_DASHBOARDS_KEY, mixed_items, ["uid", "title"],
        mixed, "summary.mixedDatasourceDashboardCount"
    )
    push_detail_findings_with_count(
        warnings, signal_keys, DASHBOARD_WARNING_RISK_RECORDS,
        DASHBOARD_DASHBOARD_GOVERNANCE_KEY, dashboard_gov_items, ["dashboardUid", "dashboardTitle"],
        risk_records, "summary.riskRecordCount", "riskCount"
    )

    if high_blast_radius_datasources > 0:
        warnings.append(StatusRecordCount(DASHBOARD_WARNING_HIGH_BLAST_RADIUS_DATASOURCES, high_blast_radius_datasources, "summary.highBlastRadiusDatasourceCount"))
        signal_keys.add("summary.highBlastRadiusDatasourceCount")

    if query_audits > 0:
        warnings.append(StatusRecordCount(DASHBOARD_WARNING_QUERY_AUDITS, query_audits, "summary.queryAuditCount"))
        signal_keys.add("summary.queryAuditCount")

    if dashboard_audits > 0:
        warnings.append(StatusRecordCount(DASHBOARD_WARNING_DASHBOARD_AUDITS, dashboard_audits, "summary.dashboardAuditCount"))
        signal_keys.add("summary.dashboardAuditCount")

    if dependency_items:
        import_ready_count = sum(1 for d in dependency_items if d.get("file", "").strip())
        gap = max(0, dashboards - import_ready_count)
        if gap > 0:
            warnings.append(StatusRecordCount(DASHBOARD_WARNING_IMPORT_READINESS_GAPS, gap, DASHBOARD_DASHBOARD_DEPENDENCIES_KEY))

    if blockers:
        status = PROJECT_STATUS_BLOCKED
        reason_code = DASHBOARD_REASON_BLOCKED_BY_BLOCKERS
        next_actions = list(DASHBOARD_RESOLVE_BLOCKERS_ACTIONS)
    elif dashboards == 0:
        status = PROJECT_STATUS_PARTIAL
        reason_code = DASHBOARD_REASON_PARTIAL_NO_DATA
        next_actions = list(DASHBOARD_EXPORT_AT_LEAST_ONE_ACTIONS)
    else:
        status = PROJECT_STATUS_READY
        reason_code = DASHBOARD_REASON_READY
        next_actions = []

    if warnings and not blockers:
        next_actions.extend(DASHBOARD_REVIEW_GOVERNANCE_WARNINGS_ACTIONS)

    return StatusReading(
        id=DASHBOARD_DOMAIN_ID,
        scope=DASHBOARD_SCOPE_STAGED,
        mode=DASHBOARD_MODE_STAGED,
        status=status,
        reason_code=reason_code,
        primary_count=max(dashboards, queries),
        source_kinds=DASHBOARD_SOURCE_KINDS_STAGED,
        signal_keys=sorted(list(signal_keys)),
        blockers=blockers,
        warnings=warnings,
        next_actions=next_actions,
    ).into_project_domain_status()


def build_live_dashboard_domain_status(
    dashboard_summaries: List[Dict[str, Any]],
    datasources: List[Dict[str, Any]],
) -> ProjectDomainStatus:
    def get_str(d, k):
        val = d.get(k, "")
        return str(val) if val is not None else ""

    dashboard_uids = {get_str(s, "uid") for s in dashboard_summaries if get_str(s, "uid")}
    dashboard_count = len(dashboard_uids)
    root_count = sum(1 for s in dashboard_summaries if not get_str(s, "folderUid"))
    folder_uids = {get_str(s, "folderUid") for s in dashboard_summaries if get_str(s, "folderUid")}
    folder_count = len(folder_uids)
    title_gap = sum(1 for s in dashboard_summaries if not get_str(s, "title").strip())
    folder_title_gap = sum(1 for s in dashboard_summaries if get_str(s, "folderUid").strip() and not get_str(s, "folderTitle").strip())
    
    import_ready_uids = set()
    for s in dashboard_summaries:
        uid = get_str(s, "uid")
        if not uid: continue
        title = get_str(s, "title")
        folder_uid = get_str(s, "folderUid")
        folder_title = get_str(s, "folderTitle")
        if title.strip() and (not folder_uid.strip() or folder_title.strip()):
            import_ready_uids.add(uid)
    import_ready_count = len(import_ready_uids)

    datasource_uids = {get_str(d, "uid") for d in datasources if get_str(d, "uid")}
    datasource_count = len(datasource_uids)
    raw_datasource_count = len(datasources)

    signal_keys = set(DASHBOARD_SIGNAL_KEYS_LIVE)
    warnings = []

    if dashboard_count > 0 and datasource_count == 0:
        warnings.append(StatusRecordCount(DASHBOARD_WARNING_KIND_EMPTY_DATASOURCE_INVENTORY, 1, DASHBOARD_DATASOURCE_EMPTY_SIGNAL_KEY))
        signal_keys.add(DASHBOARD_DATASOURCE_EMPTY_SIGNAL_KEY)
    else:
        drift = max(0, raw_datasource_count - datasource_count)
        if drift > 0:
            warnings.append(StatusRecordCount(DASHBOARD_WARNING_KIND_DATASOURCE_DRIFT, drift, DASHBOARD_DATASOURCE_DRIFT_SIGNAL_KEY))
            signal_keys.add(DASHBOARD_DATASOURCE_DRIFT_SIGNAL_KEY)
        
        if folder_count > 1:
            warnings.append(StatusRecordCount(DASHBOARD_WARNING_KIND_FOLDER_SPREAD, folder_count - 1, DASHBOARD_FOLDER_SPREAD_SIGNAL_KEY))
            signal_keys.add(DASHBOARD_FOLDER_SPREAD_SIGNAL_KEY)
        
        if folder_count > 0 and root_count > 0:
            warnings.append(StatusRecordCount(DASHBOARD_WARNING_KIND_ROOT_SCOPE, root_count, DASHBOARD_ROOT_SCOPE_SIGNAL_KEY))
            signal_keys.add(DASHBOARD_ROOT_SCOPE_SIGNAL_KEY)
            
        if title_gap > 0:
            warnings.append(StatusRecordCount(DASHBOARD_WARNING_KIND_TITLE_GAP, title_gap, DASHBOARD_TITLE_GAP_SIGNAL_KEY))
            signal_keys.add(DASHBOARD_TITLE_GAP_SIGNAL_KEY)
            
        if folder_title_gap > 0:
            warnings.append(StatusRecordCount(DASHBOARD_WARNING_KIND_FOLDER_TITLE_GAP, folder_title_gap, DASHBOARD_FOLDER_TITLE_GAP_SIGNAL_KEY))
            signal_keys.add(DASHBOARD_FOLDER_TITLE_GAP_SIGNAL_KEY)
            
        ready_gap = max(0, dashboard_count - import_ready_count)
        if ready_gap > 0:
            warnings.append(StatusRecordCount("live-dashboard-import-ready-gap", ready_gap, DASHBOARD_IMPORT_READY_SIGNAL_KEY))
            signal_keys.add(DASHBOARD_IMPORT_READY_SIGNAL_KEY)

    if dashboard_count == 0:
        status = PROJECT_STATUS_PARTIAL
        reason_code = DASHBOARD_REASON_PARTIAL_NO_DATA
        next_actions = list(DASHBOARD_NO_DATA_NEXT_ACTIONS_LIVE)
    elif datasource_count == 0:
        status = PROJECT_STATUS_PARTIAL
        reason_code = DASHBOARD_REASON_PARTIAL_NO_DATASOURCES
        next_actions = list(DASHBOARD_NO_DATASOURCES_NEXT_ACTIONS_LIVE)
    else:
        status = PROJECT_STATUS_READY
        reason_code = DASHBOARD_REASON_READY
        next_actions = list(DASHBOARD_READY_NEXT_ACTIONS_LIVE)

    if warnings and status == PROJECT_STATUS_READY:
        next_actions.extend(DASHBOARD_REVIEW_WARNING_NEXT_ACTIONS_LIVE)

    return StatusReading(
        id=DASHBOARD_DOMAIN_ID,
        scope=DASHBOARD_SCOPE_LIVE,
        mode=DASHBOARD_MODE_LIVE,
        status=status,
        reason_code=reason_code,
        primary_count=dashboard_count,
        source_kinds=DASHBOARD_SOURCE_KINDS_LIVE,
        signal_keys=sorted(list(signal_keys)),
        blockers=[],
        warnings=warnings,
        next_actions=next_actions,
    ).into_project_domain_status()
