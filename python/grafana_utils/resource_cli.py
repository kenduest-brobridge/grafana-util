"""Generic read-only resource queries for Grafana objects."""

import argparse
import sys
import json
from typing import Any, List, Optional, Dict
from .cli_shared import build_live_clients, dump_document, OUTPUT_FORMAT_CHOICES


class ResourceKind:
    DASHBOARDS = "dashboards"
    FOLDERS = "folders"
    DATASOURCES = "datasources"
    ALERT_RULES = "alert-rules"
    ORGS = "orgs"


SUPPORTED_KINDS = [
    ResourceKind.DASHBOARDS,
    ResourceKind.FOLDERS,
    ResourceKind.DATASOURCES,
    ResourceKind.ALERT_RULES,
    ResourceKind.ORGS,
]


def build_parser(prog: Optional[str] = None) -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog=prog or "grafana-util resource",
        description="Generic read-only resource queries for Grafana objects.",
    )
    subparsers = parser.add_subparsers(dest="command")
    subparsers.required = True

    # kinds
    kinds_parser = subparsers.add_parser("kinds", help="List supported resource kinds.")
    kinds_parser.add_argument("--output-format", choices=OUTPUT_FORMAT_CHOICES, default="table")

    # describe
    describe_parser = subparsers.add_parser("describe", help="Describe supported resource kinds.")
    describe_parser.add_argument("kind", nargs="?", choices=SUPPORTED_KINDS, default=None)
    describe_parser.add_argument("--output-format", choices=OUTPUT_FORMAT_CHOICES, default="table")

    # list
    list_parser = subparsers.add_parser("list", help="List resources of a specific kind.")
    list_parser.add_argument("kind", choices=SUPPORTED_KINDS)
    from .cli_shared import add_live_connection_args
    add_live_connection_args(list_parser)
    list_parser.add_argument("--output-format", choices=OUTPUT_FORMAT_CHOICES, default="table")

    # get
    get_parser = subparsers.add_parser("get", help="Fetch a single resource by selector.")
    get_parser.add_argument("selector", help="<kind>/<identity>, e.g. dashboards/cpu-main")
    add_live_connection_args(get_parser)
    get_parser.add_argument("--output-format", choices=OUTPUT_FORMAT_CHOICES, default="json")

    return parser


def parse_selector(selector: str):
    if "/" not in selector:
        raise ValueError("Resource selector must use <kind>/<identity>.")
    kind, identity = selector.split("/", 1)
    if kind not in SUPPORTED_KINDS:
        raise ValueError(f"Unsupported kind '{kind}'.")
    return kind, identity


def list_command(args: argparse.Namespace) -> int:
    _, dashboard_client, datasource_client, alert_client, access_client = build_live_clients(args)
    
    items = []
    if args.kind == ResourceKind.DASHBOARDS:
        items = alert_client.search_dashboards("")
    elif args.kind == ResourceKind.FOLDERS:
        items = dashboard_client.request_json("GET", "/api/folders")
    elif args.kind == ResourceKind.DATASOURCES:
        items = datasource_client.list_datasources()
    elif args.kind == ResourceKind.ALERT_RULES:
        items = alert_client.list_alert_rules()
    elif args.kind == ResourceKind.ORGS:
        items = access_client.request_json("GET", "/api/orgs")
    
    dump_document({"kind": args.kind, "count": len(items), "items": items}, args.output_format)
    return 0


def get_command(args: argparse.Namespace) -> int:
    kind, identity = parse_selector(args.selector)
    _, dashboard_client, datasource_client, alert_client, access_client = build_live_clients(args)
    
    item = None
    if kind == ResourceKind.DASHBOARDS:
        item = dashboard_client.get_dashboard(identity)
    elif kind == ResourceKind.FOLDERS:
        item = dashboard_client.request_json("GET", f"/api/folders/{identity}")
    elif kind == ResourceKind.DATASOURCES:
        item = datasource_client.request_json("GET", f"/api/datasources/uid/{identity}")
    elif kind == ResourceKind.ALERT_RULES:
        item = alert_client.get_alert_rule(identity)
    elif kind == ResourceKind.ORGS:
        item = access_client.request_json("GET", f"/api/orgs/{identity}")
    
    dump_document(item, args.output_format)
    return 0


def main(argv: Optional[list[str]] = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        if args.command == "kinds":
            dump_document(SUPPORTED_KINDS, args.output_format)
            return 0
        if args.command == "describe":
            # Simplified describe for parity
            dump_document({"supported": SUPPORTED_KINDS}, args.output_format)
            return 0
        if args.command == "list":
            return list_command(args)
        if args.command == "get":
            return get_command(args)
    except Exception as e:
        print(str(e), file=sys.stderr)
        return 1
    return 0
