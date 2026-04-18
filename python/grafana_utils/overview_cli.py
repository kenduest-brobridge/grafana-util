"""Project-wide overview summaries for the Python CLI."""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, List, Optional, Dict

from .cli_shared import (
    OUTPUT_FORMAT_CHOICES,
    add_live_connection_args,
    build_live_clients,
    dump_document,
    summarize_path,
)
from .project_status import ProjectStatus, ProjectStatusFreshness
from .project_status_staged import build_staged_project_status
from .project_status_live import build_live_project_status

OVERVIEW_KIND = "grafana-utils-overview"
OVERVIEW_SCHEMA_VERSION = 1


@dataclass
class OverviewInputField:
    name: str
    value: str

    def to_dict(self) -> dict[str, Any]:
        return {"name": self.name, "value": self.value}


@dataclass
class OverviewArtifact:
    kind: str
    title: str
    inputs: List[OverviewInputField]
    document: Dict[str, Any]

    def to_dict(self) -> dict[str, Any]:
        return {
            "kind": self.kind,
            "title": self.title,
            "inputs": [i.to_dict() for i in self.inputs],
            "document": self.document,
        }


@dataclass
class OverviewSummary:
    artifact_count: int = 0
    dashboard_export_count: int = 0
    datasource_export_count: int = 0
    alert_export_count: int = 0
    access_user_export_count: int = 0
    access_team_export_count: int = 0
    access_org_export_count: int = 0
    access_service_account_export_count: int = 0
    sync_summary_count: int = 0
    bundle_preflight_count: int = 0
    promotion_preflight_count: int = 0

    def to_dict(self) -> dict[str, Any]:
        return {
            "artifactCount": self.artifact_count,
            "dashboardExportCount": self.dashboard_export_count,
            "datasourceExportCount": self.datasource_export_count,
            "alertExportCount": self.alert_export_count,
            "accessUserExportCount": self.access_user_export_count,
            "accessTeamExportCount": self.access_team_export_count,
            "accessOrgExportCount": self.access_org_export_count,
            "accessServiceAccountExportCount": self.access_service_account_export_count,
            "syncSummaryCount": self.sync_summary_count,
            "bundlePreflightCount": self.bundle_preflight_count,
            "promotionPreflightCount": self.promotion_preflight_count,
        }


@dataclass
class OverviewDocument:
    kind: str
    schema_version: int
    tool_version: str
    summary: OverviewSummary
    project_status: ProjectStatus
    artifacts: List[OverviewArtifact]
    discovery: Optional[Dict[str, Any]] = None

    def to_dict(self) -> dict[str, Any]:
        res = {
            "kind": self.kind,
            "schemaVersion": self.schema_version,
            "toolVersion": self.tool_version,
            "summary": self.summary.to_dict(),
            "projectStatus": self.project_status.to_dict(),
            "artifacts": [a.to_dict() for a in self.artifacts],
        }
        if self.discovery:
            res["discovery"] = self.discovery
        return res


def build_parser(prog: Optional[str] = None) -> argparse.ArgumentParser:
    """Build the overview parser."""

    parser = argparse.ArgumentParser(
        prog=prog or "grafana-util overview",
        description="Render project-wide staged or live overview summaries.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    subparsers = parser.add_subparsers(dest="command")
    subparsers.required = True

    live_parser = subparsers.add_parser(
        "live", help="Summarize live Grafana state from a reusable connection."
    )
    add_live_connection_args(live_parser)
    live_parser.add_argument("--sync-summary-file", default=None)
    live_parser.add_argument("--bundle-preflight-file", default=None)
    live_parser.add_argument("--promotion-summary-file", default=None)
    live_parser.add_argument("--promotion-mapping-file", default=None)
    live_parser.add_argument("--availability-file", default=None)
    live_parser.add_argument(
        "--output-format",
        choices=OUTPUT_FORMAT_CHOICES,
        default="text",
        help="Render the result as text, table, json, yaml, or interactive.",
    )

    staged_parser = subparsers.add_parser(
        "staged", help="Summarize staged artifacts from the local checkout."
    )
    for flag in (
        "--dashboard-export-dir",
        "--dashboard-provisioning-dir",
        "--datasource-export-dir",
        "--datasource-provisioning-file",
        "--access-user-export-dir",
        "--access-team-export-dir",
        "--access-org-export-dir",
        "--access-service-account-export-dir",
        "--desired-file",
        "--source-bundle",
        "--target-inventory",
        "--alert-export-dir",
        "--availability-file",
        "--mapping-file",
    ):
        staged_parser.add_argument(flag, default=None)
    staged_parser.add_argument(
        "--output-format",
        choices=OUTPUT_FORMAT_CHOICES,
        default="text",
        help="Render the result as text, table, json, yaml, or interactive.",
    )

    return parser


def build_overview_artifacts(args: argparse.Namespace) -> List[OverviewArtifact]:
    from .project_status_staged import load_json
    artifacts: List[OverviewArtifact] = []

    if args.dashboard_export_dir:
        path = Path(args.dashboard_export_dir)
        doc = load_json(str(path / "inspect-summary.json"))
        if doc:
            artifacts.append(OverviewArtifact("dashboard-export", "Dashboard export", [OverviewInputField("exportDir", str(path))], doc))

    if args.dashboard_provisioning_dir:
        path = Path(args.dashboard_provisioning_dir)
        doc = load_json(str(path / "inspect-summary.json"))
        if doc:
            artifacts.append(OverviewArtifact("dashboard-export", "Dashboard provisioning", [OverviewInputField("dashboardProvisioningDir", str(path))], doc))

    if args.datasource_export_dir:
        path = Path(args.datasource_export_dir)
        doc = load_json(str(path / "datasource-export-summary.json"))
        if doc:
            artifacts.append(OverviewArtifact("datasource-export", "Datasource export", [OverviewInputField("exportDir", str(path))], doc))

    if args.alert_export_dir:
        path = Path(args.alert_export_dir)
        doc = load_json(str(path / "alert-export-summary.json"))
        if doc:
            artifacts.append(OverviewArtifact("alert-export", "Alert export", [OverviewInputField("exportDir", str(path))], doc))

    if args.access_user_export_dir:
        path = Path(args.access_user_export_dir)
        doc = load_json(str(path / "users.json"))
        if doc:
            artifacts.append(OverviewArtifact("grafana-utils-access-user-export-index", "Access user export", [OverviewInputField("exportDir", str(path))], doc))

    if args.desired_file:
        path = Path(args.desired_file)
        doc = load_json(str(path)) # In Rust this builds a summary from raw specs
        if doc:
            artifacts.append(OverviewArtifact("sync-summary", "Sync summary", [OverviewInputField("desiredFile", str(path))], doc))

    return artifacts


def build_overview_summary_from_artifacts(artifacts: List[OverviewArtifact]) -> OverviewSummary:
    summary = OverviewSummary(artifact_count=len(artifacts))
    for a in artifacts:
        if a.kind == "dashboard-export":
            summary.dashboard_export_count += 1
        elif a.kind == "datasource-export":
            summary.datasource_export_count += 1
        elif a.kind == "alert-export":
            summary.alert_export_count += 1
        elif a.kind == "grafana-utils-access-user-export-index":
            summary.access_user_export_count += 1
        elif a.kind == "sync-summary":
            summary.sync_summary_count += 1
    return summary


def build_overview_document(args: argparse.Namespace) -> OverviewDocument:
    artifacts = build_overview_artifacts(args)
    summary = build_overview_summary_from_artifacts(artifacts)

    project_status = build_staged_project_status(
        dashboard_export_dir=args.dashboard_export_dir,
        dashboard_provisioning_dir=args.dashboard_provisioning_dir,
        datasource_export_dir=args.datasource_export_dir,
        datasource_provisioning_file=args.datasource_provisioning_file,
        access_user_export_dir=args.access_user_export_dir,
        access_team_export_dir=args.access_team_export_dir,
        access_org_export_dir=args.access_org_export_dir,
        access_service_account_export_dir=args.access_service_account_export_dir,
        alert_export_dir=args.alert_export_dir,
    )

    return OverviewDocument(
        kind=OVERVIEW_KIND,
        schema_version=OVERVIEW_SCHEMA_VERSION,
        tool_version="0.0.0",
        summary=summary,
        project_status=project_status,
        artifacts=artifacts,
    )


def render_project_status_decision_order(status: ProjectStatus) -> List[str]:
    lines = []
    for domain in status.domains:
        lines.append(f"  - {domain.id}: status={domain.status} reason={domain.reason_code}")
    return lines


def render_overview_text(doc: OverviewDocument) -> List[str]:
    status = doc.project_status
    lines = [
        "Project overview",
        f"Status: {status.overall.status} domains={status.overall.domain_count} present={status.overall.present_count} blocked={status.overall.blocked_count} blockers={status.overall.blocker_count} warnings={status.overall.warning_count} freshness={status.overall.freshness.status} oldestAge={status.overall.freshness.oldest_age_seconds or 0}s",
        f"Artifacts: {doc.summary.artifact_count} total, {doc.summary.dashboard_export_count} dashboard export, {doc.summary.datasource_export_count} datasource export, {doc.summary.alert_export_count} alert export, {doc.summary.access_user_export_count} access user export, {doc.summary.access_team_export_count} access team export, {doc.summary.access_org_export_count} access org export, {doc.summary.access_service_account_export_count} access service-account export, {doc.summary.sync_summary_count} sync summary, {doc.summary.bundle_preflight_count} bundle preflight, {doc.summary.promotion_preflight_count} promotion preflight",
    ]

    if doc.discovery:
        lines.append(f"Discovery: {json.dumps(doc.discovery)}")

    if status.domains:
        lines.append("Domain status:")
        for d in status.domains:
            line = f"- {d.id} status={d.status} reason={d.reason_code} primary={d.primary_count} blockers={d.blocker_count} warnings={d.warning_count} freshness={d.freshness.status}"
            if d.next_actions:
                line += f" next={d.next_actions[0]}"
            lines.append(line)

    if status.top_blockers:
        lines.append("Top blockers:")
        for b in status.top_blockers[:5]:
            lines.append(f"- {b.domain} {b.kind} count={b.count} source={b.source}")

    if status.next_actions:
        lines.append("Next actions:")
        for a in status.next_actions[:5]:
            lines.append(f"- {a.domain} reason={a.reason_code} action={a.action}")

    for a in doc.artifacts:
        lines.append("")
        lines.append(f"# {a.title}")
        lines.append(f"Summary: {a.kind} inputs={len(a.inputs)}")

    return lines


def live_command(args: argparse.Namespace) -> int:
    _, dashboard_client, datasource_client, alert_client, access_client = build_live_clients(args)
    status = build_live_project_status(
        dashboard_client=dashboard_client,
        datasource_client=datasource_client,
        alert_client=alert_client,
        access_client=access_client,
        sync_summary_file=args.sync_summary_file,
        bundle_preflight_file=args.bundle_preflight_file,
        promotion_summary_file=args.promotion_summary_file,
        promotion_mapping_file=args.promotion_mapping_file,
        availability_file=args.availability_file,
    )
    if args.output_format == "text":
        # Simplified: for live status, Rust often renders ProjectStatus text
        # Here we'll just dump JSON for consistency if not implementing full live overview doc
        dump_document(status.to_dict(), args.output_format)
    else:
        dump_document(status.to_dict(), args.output_format)
    return 0


def staged_command(args: argparse.Namespace) -> int:
    doc = build_overview_document(args)
    if args.output_format == "text":
        print("\n".join(render_overview_text(doc)))
    else:
        dump_document(doc.to_dict(), args.output_format)
    return 0


def parse_args(argv: Optional[list[str]] = None) -> argparse.Namespace:
    return build_parser().parse_args(argv)


def main(argv: Optional[list[str]] = None) -> int:
    try:
        args = parse_args(argv)
        if args.command == "live":
            return live_command(args)
        if args.command == "staged":
            return staged_command(args)
        raise RuntimeError("Unsupported overview command.")
    except Exception as exc:
        print(str(exc), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
