"""Live project-status coordinator."""

from typing import Any, List, Optional
from .project_status import ProjectDomainStatus, ProjectStatusFreshness, build_project_status
from .alerts.project_status import build_alert_live_project_status_domain
from .datasource.project_status import build_datasource_live_project_status
from .dashboards.project_status import build_live_dashboard_domain_status
from .clients.alert_client import GrafanaAlertClient
from .clients.datasource_client import GrafanaDatasourceClient
from .clients.dashboard_client import GrafanaClient
from .clients.access_client import GrafanaAccessClient


from pathlib import Path
from .project_status import (
    ProjectDomainStatus,
    ProjectStatusFreshness,
    ProjectStatusFreshnessSample,
    build_project_status,
    build_live_project_status_freshness,
    build_live_project_status_freshness_from_samples,
    build_live_project_status_freshness_from_source_count,
)
from .sync.project_status import build_live_sync_domain_status
from .promotion.project_status import build_live_promotion_project_status
from .project_status_staged import load_json


def stamp_live_domain_freshness(
    domain: ProjectDomainStatus,
    samples: List[ProjectStatusFreshnessSample],
) -> ProjectDomainStatus:
    if not samples:
        domain.freshness = build_live_project_status_freshness_from_source_count(len(domain.source_kinds))
    else:
        domain.freshness = build_live_project_status_freshness_from_samples(samples)
    return domain


def project_status_severity_rank(status: str) -> int:
    if status == "blocked":
        return 0
    if status == "partial":
        return 1
    if status == "ready":
        return 2
    return 3


def merge_project_status_findings(findings: List[Any]) -> List[Any]:
    from .project_status import ProjectStatusFinding
    merged = {}
    for f in findings:
        key = (f.kind, f.source)
        merged[key] = merged.get(key, 0) + f.count
    
    return [ProjectStatusFinding(kind=k, source=s, count=c) for (k, s), c in merged.items()]


def merge_live_domain_statuses(statuses: List[ProjectDomainStatus]) -> Optional[ProjectDomainStatus]:
    if not statuses:
        return None
    
    from .status_model import StatusReading, StatusRecordCount
    
    # Pick the "worst" status as the baseline for id/scope/etc
    aggregate = min(statuses, key=lambda s: (
        project_status_severity_rank(s.status),
        -s.blocker_count,
        -s.warning_count,
    ))

    blockers = merge_project_status_findings([f for s in statuses for f in s.blockers])
    warnings = merge_project_status_findings([f for s in statuses for f in s.warnings])
    
    # Calculate freshness across all
    ages = []
    total_source_count = 0
    for s in statuses:
        total_source_count += s.freshness.source_count
        if s.freshness.newest_age_seconds is not None:
            ages.append(s.freshness.newest_age_seconds)
        if s.freshness.oldest_age_seconds is not None:
            ages.append(s.freshness.oldest_age_seconds)
    freshness = build_live_project_status_freshness(total_source_count, ages)

    reason_code = aggregate.reason_code
    if not all(s.reason_code == aggregate.reason_code for s in statuses):
        reason_code = "multi-org-aggregate"
    
    mode = aggregate.mode
    if all(s.mode == aggregate.mode for s in statuses):
        mode = f"{aggregate.mode}-all-orgs"
    else:
        mode = "multi-org-aggregate"

    source_kinds = sorted(list(set(k for s in statuses for k in s.source_kinds)))
    signal_keys = sorted(list(set(k for s in statuses for k in s.signal_keys)))
    
    next_actions = []
    for s in statuses:
        for a in s.next_actions:
            if a not in next_actions:
                next_actions.append(a)

    return StatusReading(
        id=aggregate.id,
        scope=aggregate.scope,
        mode=mode,
        status=aggregate.status,
        reason_code=reason_code,
        primary_count=sum(s.primary_count for s in statuses),
        source_kinds=source_kinds,
        signal_keys=signal_keys,
        blockers=[StatusRecordCount(f.kind, f.count, f.source) for f in blockers],
        warnings=[StatusRecordCount(f.kind, f.count, f.source) for f in warnings],
        next_actions=next_actions,
        freshness=freshness,
    ).into_project_domain_status()


def _org_id_from_record(org: Any) -> Optional[str]:
    if not isinstance(org, dict):
        return None
    value = org.get("id") or org.get("orgId")
    text = str(value or "").strip()
    return text or None


def _read_failed_domain(
    domain_id: str,
    *,
    source: str,
    error: Exception,
    next_action: str,
    mode: str = "live-read-failed",
) -> ProjectDomainStatus:
    from .status_model import StatusReading, StatusRecordCount

    return StatusReading(
        id=domain_id,
        scope="live",
        mode=mode,
        status="blocked",
        reason_code="live-read-failed",
        primary_count=0,
        source_kinds=[source],
        signal_keys=[f"live.{domain_id}.read_failed"],
        blockers=[
            StatusRecordCount(
                kind="live-read-failed",
                count=1,
                source=f"{source}: {error}",
            )
        ],
        warnings=[],
        next_actions=[next_action],
        freshness=build_live_project_status_freshness_from_source_count(0),
    ).into_project_domain_status()


def _scope_clients(
    org_id: Optional[str],
    dashboard_client: GrafanaClient,
    datasource_client: GrafanaDatasourceClient,
    alert_client: GrafanaAlertClient,
    access_client: GrafanaAccessClient,
) -> tuple[GrafanaClient, GrafanaDatasourceClient, GrafanaAlertClient, GrafanaAccessClient]:
    if not org_id:
        return dashboard_client, datasource_client, alert_client, access_client
    return (
        dashboard_client.with_org_id(org_id),
        datasource_client.with_org_id(org_id),
        alert_client.with_org_id(org_id),
        access_client.with_org_id(org_id),
    )


def _build_live_grafana_domains_for_scope(
    dashboard_client: GrafanaClient,
    datasource_client: GrafanaDatasourceClient,
    alert_client: GrafanaAlertClient,
    access_client: GrafanaAccessClient,
    *,
    all_orgs: bool,
) -> list[ProjectDomainStatus]:
    domains: list[ProjectDomainStatus] = []
    next_action_suffix = " --all-orgs" if all_orgs else ""

    try:
        rules = alert_client.list_alert_rules()
        alert_status = build_alert_live_project_status_domain(
            rules_document=rules,
            contact_points_document=alert_client.list_contact_points(),
            mute_timings_document=alert_client.list_mute_timings(),
            policies_document=alert_client.get_notification_policies(),
            templates_document=alert_client.list_templates(),
        )
        if alert_status:
            domains.append(stamp_live_domain_freshness(alert_status, []))
    except Exception as exc:  # noqa: BLE001 - convert live read failures into status.
        domains.append(
            _read_failed_domain(
                "alert",
                source="live.alert",
                error=exc,
                next_action=f"restore alert/org read access, then re-run live status{next_action_suffix}",
            )
        )

    try:
        summaries = dashboard_client.iter_dashboard_summaries(500)
        dashboard_status = build_live_dashboard_domain_status(
            dashboard_summaries=summaries,
            datasources=datasource_client.list_datasources(),
        )
        if dashboard_status:
            domains.append(stamp_live_domain_freshness(dashboard_status, []))
    except Exception as exc:  # noqa: BLE001 - convert live read failures into status.
        domains.append(
            _read_failed_domain(
                "dashboard",
                source="live.dashboard",
                error=exc,
                next_action=f"restore dashboard/org read access, then re-run live status{next_action_suffix}",
            )
        )

    try:
        datasource_status = build_datasource_live_project_status(
            datasource_list=datasource_client.list_datasources(),
            current_org=access_client.get_current_org(),
        )
        if datasource_status:
            domains.append(stamp_live_domain_freshness(datasource_status, []))
    except Exception as exc:  # noqa: BLE001 - convert live read failures into status.
        domains.append(
            _read_failed_domain(
                "datasource",
                source="live.datasource",
                error=exc,
                next_action=f"restore datasource/org read access, then re-run live status{next_action_suffix}",
            )
        )

    try:
        from .status_model import StatusReading

        users = access_client.list_org_users()
        access_domain = StatusReading(
            id="access",
            scope="live",
            mode="live-list-surfaces",
            status="ready",
            reason_code="ready",
            primary_count=len(users),
            source_kinds=["live-org-users"],
            signal_keys=["live.users.count"],
            blockers=[],
            warnings=[],
            next_actions=["re-run access export after membership changes"],
        ).into_project_domain_status()
        domains.append(stamp_live_domain_freshness(access_domain, []))
    except Exception as exc:  # noqa: BLE001 - convert live read failures into status.
        domains.append(
            _read_failed_domain(
                "access",
                source="live.access",
                error=exc,
                next_action=f"restore access/org read access, then re-run live status{next_action_suffix}",
            )
        )

    return domains


def build_live_project_status(
    dashboard_client: GrafanaClient,
    datasource_client: GrafanaDatasourceClient,
    alert_client: GrafanaAlertClient,
    access_client: GrafanaAccessClient,
    sync_summary_file: Optional[str] = None,
    bundle_preflight_file: Optional[str] = None,
    promotion_summary_file: Optional[str] = None,
    promotion_mapping_file: Optional[str] = None,
    availability_file: Optional[str] = None,
    all_orgs: bool = False,
    tool_version: str = "0.0.0",
) -> Any:
    domains: List[ProjectDomainStatus] = []

    orgs: list[dict[str, Any]] = []
    if all_orgs:
        try:
            orgs = access_client.list_organizations()
        except Exception as exc:  # noqa: BLE001 - org discovery is a domain blocker.
            domains.append(
                _read_failed_domain(
                    "access",
                    source="live.orgs",
                    error=exc,
                    next_action="restore org list access, then re-run live status --all-orgs",
                )
            )

    if all_orgs and orgs:
        grouped: dict[str, list[ProjectDomainStatus]] = {
            "alert": [],
            "dashboard": [],
            "datasource": [],
            "access": [],
        }
        for org in orgs:
            org_id = _org_id_from_record(org)
            scoped_clients = _scope_clients(
                org_id,
                dashboard_client,
                datasource_client,
                alert_client,
                access_client,
            )
            for domain in _build_live_grafana_domains_for_scope(
                *scoped_clients,
                all_orgs=True,
            ):
                grouped.setdefault(domain.id, []).append(domain)
        for domain_id in ("alert", "dashboard", "datasource", "access"):
            merged = merge_live_domain_statuses(grouped.get(domain_id, []))
            if merged:
                domains.append(merged)
    elif not all_orgs:
        domains.extend(
            _build_live_grafana_domains_for_scope(
                dashboard_client,
                datasource_client,
                alert_client,
                access_client,
                all_orgs=False,
            )
        )

    # Sync
    try:
        sync_summary = load_json(sync_summary_file)
        bundle_preflight = load_json(bundle_preflight_file)
        sync_status = build_live_sync_domain_status(
            summary_document=sync_summary,
            bundle_preflight_document=bundle_preflight,
        )
        if sync_status:
            samples = []
            if sync_summary_file:
                samples.append(ProjectStatusFreshnessSample.from_path("sync-summary", Path(sync_summary_file)))
            if bundle_preflight_file:
                samples.append(ProjectStatusFreshnessSample.from_path("bundle-preflight", Path(bundle_preflight_file)))
            sync_status = stamp_live_domain_freshness(sync_status, samples)
            domains.append(sync_status)
    except Exception:
        pass

    # Promotion
    try:
        promotion_summary = load_json(promotion_summary_file)
        promotion_mapping = load_json(promotion_mapping_file)
        availability = load_json(availability_file)
        promotion_status = build_live_promotion_project_status(
            promotion_summary_document=promotion_summary,
            promotion_mapping_document=promotion_mapping,
            availability_document=availability,
        )
        if promotion_status:
            samples = []
            if promotion_summary_file:
                samples.append(ProjectStatusFreshnessSample.from_path("promotion-summary", Path(promotion_summary_file)))
            if promotion_mapping_file:
                samples.append(ProjectStatusFreshnessSample.from_path("promotion-mapping", Path(promotion_mapping_file)))
            if availability_file:
                samples.append(ProjectStatusFreshnessSample.from_path("availability", Path(availability_file)))
            promotion_status = stamp_live_domain_freshness(promotion_status, samples)
            domains.append(promotion_status)
    except Exception:
        pass

    ages = []
    total_source_count = 0
    for d in domains:
        total_source_count += d.freshness.source_count
        if d.freshness.newest_age_seconds is not None:
            ages.append(d.freshness.newest_age_seconds)
        if d.freshness.oldest_age_seconds is not None:
            ages.append(d.freshness.oldest_age_seconds)
    
    overall_freshness = build_live_project_status_freshness(total_source_count, ages)

    status = build_project_status(
        scope="live",
        domain_count=6,
        freshness=overall_freshness,
        domains=domains,
        tool_version=tool_version,
    )
    
    # Discovery summary
    try:
        health = dashboard_client.request_json("GET", "/api/health")
        status.discovery = {
            "instance": {
                "source": "api-health",
                "status": "available" if health else "unavailable",
                "health": health
            }
        }
    except Exception as e:
        status.discovery = {
            "instance": {
                "source": "api-health",
                "status": "unavailable",
                "error": str(e)
            }
        }
        
    return status
