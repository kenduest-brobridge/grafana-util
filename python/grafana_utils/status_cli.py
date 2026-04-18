#!/usr/bin/env python3
"""Shared readiness/status summaries for the Python CLI."""

from __future__ import annotations

import argparse
import sys
from typing import Optional

from . import overview_cli
from .cli_shared import dump_document


from .project_status_staged import build_staged_project_status
from .project_status_live import build_live_project_status
from .cli_shared import build_live_clients


def build_parser(prog: Optional[str] = None) -> argparse.ArgumentParser:
    """Build the status parser."""

    parser = overview_cli.build_parser(prog=prog or "grafana-util status")
    # overview_cli.build_parser already adds the subcommands 'live' and 'staged'
    # we just need to make sure the flags are there.
    return parser


def parse_args(argv: Optional[list[str]] = None) -> argparse.Namespace:
    return build_parser().parse_args(argv)


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
    dump_document(status.to_dict(), args.output_format)
    return 0


def staged_command(args: argparse.Namespace) -> int:
    status = build_staged_project_status(
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
    dump_document(status.to_dict(), args.output_format)
    return 0


def main(argv: Optional[list[str]] = None) -> int:
    try:
        args = parse_args(argv)
        if args.command == "live":
            return live_command(args)
        if args.command == "staged":
            return staged_command(args)
        raise RuntimeError("Unsupported status command.")
    except ValueError as exc:
        print(str(exc), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
