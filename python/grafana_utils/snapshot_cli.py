#!/usr/bin/env python3
"""Local snapshot bundle export and review CLI."""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Optional

from .cli_shared import (
    OUTPUT_FORMAT_CHOICES,
    add_live_connection_args,
    build_live_clients,
    dump_document,
)
from .profile_config import (
    LATEST_RUN_POINTER_FILENAME,
    load_profile_document,
    profile_scope_path,
    read_latest_run_id,
    resolve_artifact_root_path,
    resolve_input_lane_path,
)

SNAPSHOT_BUNDLE_FILENAME = "snapshot.json"
SNAPSHOT_RUN_SELECTOR_CHOICES = ("latest", "timestamp")
SNAPSHOT_LANE_FILE_COUNT_PATHS = {
    "dashboards": "dashboards",
    "datasources": "datasources",
    "users": "access/users",
    "teams": "access/teams",
    "organizations": "access/orgs",
    "serviceAccounts": "access/service-accounts",
}


def build_parser(prog: Optional[str] = None) -> argparse.ArgumentParser:
    """Build the snapshot parser."""

    parser = argparse.ArgumentParser(
        prog=prog or "grafana-util snapshot",
        description="Export and review Grafana snapshot inventory bundles.",
        epilog=(
            "Examples:\n\n"
            "  grafana-util snapshot export --profile prod --output-dir ./snapshot\n"
            "  grafana-util snapshot review --input-dir ./snapshot --output-format json\n"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    subparsers = parser.add_subparsers(dest="command")
    subparsers.required = True

    export_parser = subparsers.add_parser(
        "export", help="Export a local snapshot bundle from live Grafana."
    )
    add_live_connection_args(export_parser)
    export_parser.add_argument(
        "--page-size",
        type=int,
        default=500,
        help="Dashboard search page size when collecting live inventory.",
    )
    export_parser.add_argument(
        "--run",
        choices=SNAPSHOT_RUN_SELECTOR_CHOICES,
        default=None,
        help="Select the run to export into when using the artifact workspace.",
    )
    export_parser.add_argument(
        "--run-id",
        default=None,
        help="Target one explicit artifact run id when using the artifact workspace.",
    )
    export_parser.add_argument(
        "--output-dir",
        default="snapshot",
        help="Directory to write the snapshot bundle into.",
    )
    export_parser.add_argument(
        "--overwrite",
        action="store_true",
        help="Overwrite an existing snapshot bundle file.",
    )
    export_parser.add_argument(
        "--output-format",
        choices=OUTPUT_FORMAT_CHOICES,
        default="text",
        help="Render the summary as text, table, json, yaml, or interactive.",
    )

    review_parser = subparsers.add_parser(
        "review", help="Review a local snapshot bundle without touching Grafana."
    )
    review_parser.add_argument(
        "--local",
        action="store_true",
        help="Review the selected artifact workspace snapshot root.",
    )
    review_parser.add_argument(
        "--run",
        choices=SNAPSHOT_RUN_SELECTOR_CHOICES,
        default=None,
        help="Select the run to review when using the artifact workspace.",
    )
    review_parser.add_argument(
        "--run-id",
        default=None,
        help="Target one explicit artifact run id when using the artifact workspace.",
    )
    review_parser.add_argument(
        "--input-dir",
        default=None,
        help="Directory that contains snapshot.json.",
    )
    review_parser.add_argument(
        "--input-file",
        default=None,
        help="Optional explicit snapshot JSON file path.",
    )
    review_parser.add_argument(
        "--output-format",
        choices=OUTPUT_FORMAT_CHOICES,
        default="text",
        help="Render the review as text, table, json, yaml, or interactive.",
    )
    review_parser.add_argument(
        "--interactive",
        action="store_true",
        help="Accept the interactive output contract used by the handbook.",
    )

    return parser


def _resolve_snapshot_artifact_root(args: argparse.Namespace) -> Path:
    """Resolve one artifact workspace snapshot root from the selected run."""
    return resolve_input_lane_path(
        ".",
        run=getattr(args, "run", None),
        run_id=getattr(args, "run_id", None),
        profile_name=getattr(args, "profile", None),
        config_path=getattr(args, "config", None),
    )


def _resolve_snapshot_output_dir(args: argparse.Namespace) -> Path:
    """Resolve the export output directory, defaulting into the artifact workspace."""
    output_dir = Path(args.output_dir)
    if (
        str(args.output_dir or "").strip() == "snapshot"
        and (getattr(args, "run", None) or getattr(args, "run_id", None))
    ):
        document = load_profile_document(getattr(args, "config", None))
        scope_root = profile_scope_path(
            resolve_artifact_root_path(document, getattr(args, "config", None)),
            getattr(args, "profile", None),
        )
        if getattr(args, "run_id", None):
            run_id = str(args.run_id).strip()
        elif getattr(args, "run", None) == "latest":
            run_id = read_latest_run_id(scope_root)
        else:
            run_id = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
        if not run_id or run_id in {".", ".."} or "/" in run_id or "\\" in run_id:
            raise ValueError("Invalid artifact run id: %s" % run_id)
        scope_root.mkdir(parents=True, exist_ok=True)
        (scope_root / LATEST_RUN_POINTER_FILENAME).write_text(
            json.dumps({"runId": run_id}, indent=2) + "\n",
            encoding="utf-8",
        )
        return scope_root / "runs" / run_id
    return output_dir


def _snapshot_bundle(args: argparse.Namespace) -> dict[str, Any]:
    details, dashboard_client, datasource_client, alert_client, access_client = build_live_clients(
        args, config_path=args.config
    )
    dashboards = dashboard_client.iter_dashboard_summaries(int(args.page_size))
    datasources = datasource_client.list_datasources()
    users = access_client.list_org_users()
    teams = access_client.list_teams(None, 1, 500)
    orgs = access_client.list_organizations()
    service_accounts = access_client.list_service_accounts(None, 1, 500)
    alert_rules = alert_client.list_alert_rules()
    return {
        "kind": "snapshot-bundle",
        "profile": details.profile_name,
        "url": details.url,
        "summary": {
            "dashboardCount": len(dashboards),
            "datasourceCount": len(datasources),
            "userCount": len(users),
            "teamCount": len(teams),
            "organizationCount": len(orgs),
            "serviceAccountCount": len(service_accounts),
            "alertRuleCount": len(alert_rules),
        },
        "dashboards": dashboards,
        "datasources": datasources,
        "access": {
            "users": users,
            "teams": teams,
            "organizations": orgs,
            "serviceAccounts": service_accounts,
        },
        "alerts": alert_rules,
    }


def export_command(args: argparse.Namespace) -> int:
    document = _snapshot_bundle(args)
    output_dir = _resolve_snapshot_output_dir(args)
    output_dir.mkdir(parents=True, exist_ok=True)
    output_file = output_dir / SNAPSHOT_BUNDLE_FILENAME
    if output_file.exists() and not bool(args.overwrite):
        raise ValueError("Snapshot bundle already exists: %s" % output_file)
    output_file.write_text(
        json.dumps(document, indent=2, sort_keys=False),
        encoding="utf-8",
    )
    dump_document(
        {
            "kind": document["kind"],
            "outputFile": str(output_file),
            "summary": document["summary"],
        },
        args.output_format,
    )
    return 0


def _count_json_files(path: Path) -> int:
    if not path.exists():
        return 0
    if path.is_file():
        return 1 if path.suffix == ".json" else 0
    return len([item for item in path.rglob("*.json") if item.is_file()])


def _build_snapshot_lane_document(root: Path) -> dict[str, Any]:
    lane_counts = {
        key: _count_json_files(root / relative_path)
        for key, relative_path in SNAPSHOT_LANE_FILE_COUNT_PATHS.items()
    }
    return {
        "kind": "snapshot-root",
        "summary": {
            "dashboardCount": lane_counts["dashboards"],
            "datasourceCount": lane_counts["datasources"],
            "userCount": lane_counts["users"],
            "teamCount": lane_counts["teams"],
            "organizationCount": lane_counts["organizations"],
            "serviceAccountCount": lane_counts["serviceAccounts"],
        },
        "dashboards": [{} for _ in range(lane_counts["dashboards"])],
        "datasources": [{} for _ in range(lane_counts["datasources"])],
        "access": {
            "users": [{} for _ in range(lane_counts["users"])],
            "teams": [{} for _ in range(lane_counts["teams"])],
            "organizations": [{} for _ in range(lane_counts["organizations"])],
            "serviceAccounts": [{} for _ in range(lane_counts["serviceAccounts"])],
        },
        "alerts": [],
    }


def _resolve_snapshot_input_path(args: argparse.Namespace) -> Path:
    if getattr(args, "local", False) and (
        getattr(args, "input_dir", None) or getattr(args, "input_file", None)
    ):
        raise ValueError("--input-dir, --input-file, and --local cannot be combined.")
    if getattr(args, "input_dir", None) and getattr(args, "input_file", None):
        raise ValueError("Provide only one of --input-dir or --input-file.")
    if getattr(args, "local", False):
        return _resolve_snapshot_artifact_root(args)
    if getattr(args, "input_file", None):
        return Path(args.input_file)
    if getattr(args, "input_dir", None):
        return Path(args.input_dir)
    raise ValueError("Provide either --input-dir, --input-file, or --local.")


def _load_snapshot_document(args: argparse.Namespace) -> dict[str, Any]:
    path = _resolve_snapshot_input_path(args)
    if path.is_file():
        data = json.loads(path.read_text(encoding="utf-8"))
        if not isinstance(data, dict):
            raise ValueError("Snapshot bundle must be a JSON object: %s" % path)
        return data
    if not path.exists():
        raise ValueError("Snapshot bundle not found: %s" % path)
    if not path.is_dir():
        raise ValueError("Snapshot bundle not found: %s" % path)
    snapshot_file = path / SNAPSHOT_BUNDLE_FILENAME
    if snapshot_file.is_file():
        data = json.loads(snapshot_file.read_text(encoding="utf-8"))
        if not isinstance(data, dict):
            raise ValueError("Snapshot bundle must be a JSON object: %s" % snapshot_file)
        return data
    return _build_snapshot_lane_document(path)


def review_command(args: argparse.Namespace) -> int:
    document = _load_snapshot_document(args)
    summary = dict(document.get("summary") or {})
    review_document = {
        "kind": document.get("kind", "snapshot-bundle"),
        "summary": summary,
        "items": {
            "dashboards": len(document.get("dashboards") or []),
            "datasources": len(document.get("datasources") or []),
            "users": len((document.get("access") or {}).get("users") or []),
            "teams": len((document.get("access") or {}).get("teams") or []),
            "organizations": len((document.get("access") or {}).get("organizations") or []),
            "serviceAccounts": len((document.get("access") or {}).get("serviceAccounts") or []),
            "alerts": len(document.get("alerts") or []),
        },
    }
    if args.interactive and args.output_format == "text":
        review_document["mode"] = "interactive"
    dump_document(review_document, args.output_format)
    return 0


def parse_args(argv: Optional[list[str]] = None) -> argparse.Namespace:
    return build_parser().parse_args(argv)


def main(argv: Optional[list[str]] = None) -> int:
    try:
        args = parse_args(argv)
        if args.command == "export":
            return export_command(args)
        if args.command == "review":
            return review_command(args)
        raise RuntimeError("Unsupported snapshot command.")
    except ValueError as exc:
        print(str(exc), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
