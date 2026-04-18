"""Staged project-status coordinator."""

import json
from pathlib import Path
from typing import Any, Optional, Dict, List

from .project_status import ProjectDomainStatus, ProjectStatusFreshness, build_project_status
from .access.project_status import build_access_domain_status
from .alerts.project_status import build_alert_project_status_domain
from .dashboards.project_status import build_dashboard_domain_status
from .datasource.project_status import build_datasource_project_status


def load_json(path: Optional[str]) -> Optional[Dict[str, Any]]:
    if not path:
        return None
    p = Path(path)
    if not p.exists():
        return None
    try:
        with open(p, "r") as f:
            return json.load(f)
    except Exception:
        return None


def build_staged_project_status(
    dashboard_export_dir: Optional[str] = None,
    dashboard_provisioning_dir: Optional[str] = None,
    datasource_export_dir: Optional[str] = None,
    datasource_provisioning_file: Optional[str] = None,
    access_user_export_dir: Optional[str] = None,
    access_team_export_dir: Optional[str] = None,
    access_org_export_dir: Optional[str] = None,
    access_service_account_export_dir: Optional[str] = None,
    alert_export_dir: Optional[str] = None,
    tool_version: str = "0.0.0",
) -> Any:
    domains: List[ProjectDomainStatus] = []

    # Access
    access_inputs = {
        "user_export_document": load_json(str(Path(access_user_export_dir) / "users.json") if access_user_export_dir else None),
        "team_export_document": load_json(str(Path(access_team_export_dir) / "teams.json") if access_team_export_dir else None),
        "org_export_document": load_json(str(Path(access_org_export_dir) / "orgs.json") if access_org_export_dir else None),
        "service_account_export_document": load_json(str(Path(access_service_account_export_dir) / "service-accounts.json") if access_service_account_export_dir else None),
    }
    access_status = build_access_domain_status(**access_inputs)
    if access_status:
        domains.append(access_status)

    # Alerts
    alert_summary = load_json(str(Path(alert_export_dir) / "alert-export-summary.json") if alert_export_dir else None)
    alert_status = build_alert_project_status_domain(alert_summary)
    if alert_status:
        domains.append(alert_status)

    # Dashboards
    dashboard_dir = dashboard_export_dir or dashboard_provisioning_dir
    dashboard_summary = load_json(str(Path(dashboard_dir) / "inspect-summary.json") if dashboard_dir else None)
    dashboard_status = build_dashboard_domain_status(dashboard_summary)
    if dashboard_status:
        domains.append(dashboard_status)

    # Datasource
    datasource_dir = datasource_export_dir
    datasource_summary = load_json(str(Path(datasource_dir) / "datasource-export-summary.json") if datasource_dir else None)
    datasource_status = build_datasource_project_status(datasource_summary)
    if datasource_status:
        domains.append(datasource_status)

    return build_project_status(
        scope="staged",
        domain_count=6,
        freshness=ProjectStatusFreshness(status="ready", source_count=len(domains)),
        domains=domains,
        tool_version=tool_version,
    )
