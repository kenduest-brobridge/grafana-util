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

    @classmethod
    def all(cls) -> List[str]:
        return [cls.DASHBOARDS, cls.FOLDERS, cls.DATASOURCES, cls.ALERT_RULES, cls.ORGS]

    @classmethod
    def singular_label(cls, kind: str) -> str:
        return {
            cls.DASHBOARDS: "dashboard",
            cls.FOLDERS: "folder",
            cls.DATASOURCES: "datasource",
            cls.ALERT_RULES: "alert-rule",
            cls.ORGS: "org",
        }.get(kind, kind)

    @classmethod
    def description(cls, kind: str) -> str:
        return {
            cls.DASHBOARDS: "Grafana dashboards from /api/search and /api/dashboards/uid/{uid}.",
            cls.FOLDERS: "Grafana folders from /api/folders and /api/folders/{uid}.",
            cls.DATASOURCES: "Grafana datasources from /api/datasources and /api/datasources/uid/{uid}.",
            cls.ALERT_RULES: "Grafana alert rules from /api/v1/provisioning/alert-rules and /api/v1/provisioning/alert-rules/{uid}.",
            cls.ORGS: "Grafana org inventory from /api/orgs and /api/orgs/{id}.",
        }.get(kind, "")

    @classmethod
    def selector_pattern(cls, kind: str) -> str:
        return {
            cls.DASHBOARDS: "dashboards/<uid>",
            cls.FOLDERS: "folders/<uid>",
            cls.DATASOURCES: "datasources/<uid>",
            cls.ALERT_RULES: "alert-rules/<uid>",
            cls.ORGS: "orgs/<id>",
        }.get(kind, "")

    @classmethod
    def list_endpoint(cls, kind: str) -> str:
        return {
            cls.DASHBOARDS: "GET /api/search",
            cls.FOLDERS: "GET /api/folders",
            cls.DATASOURCES: "GET /api/datasources",
            cls.ALERT_RULES: "GET /api/v1/provisioning/alert-rules",
            cls.ORGS: "GET /api/orgs",
        }.get(kind, "")

    @classmethod
    def get_endpoint(cls, kind: str) -> str:
        return {
            cls.DASHBOARDS: "GET /api/dashboards/uid/{uid}",
            cls.FOLDERS: "GET /api/folders/{uid}",
            cls.DATASOURCES: "GET /api/datasources/uid/{uid}",
            cls.ALERT_RULES: "GET /api/v1/provisioning/alert-rules/{uid}",
            cls.ORGS: "GET /api/orgs/{id}",
        }.get(kind, "")


SUPPORTED_KINDS = ResourceKind.all()


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
    
    document = {
        "kind": args.kind,
        "count": len(items),
        "items": items
    }
    
    if args.output_format == "table":
        headers = []
        rows = []
        if args.kind == ResourceKind.DASHBOARDS:
            headers = ["uid", "title", "folder"]
            rows = [[item.get("uid", ""), item.get("title", item.get("name", "")), item.get("folderTitle", "")] for item in items]
        elif args.kind == ResourceKind.FOLDERS:
            headers = ["uid", "title", "parent_uid"]
            rows = [[item.get("uid", ""), item.get("title", ""), item.get("parentUid", "")] for item in items]
        elif args.kind == ResourceKind.DATASOURCES:
            headers = ["uid", "name", "type"]
            rows = [[item.get("uid", ""), item.get("name", ""), item.get("type", "")] for item in items]
        elif args.kind == ResourceKind.ALERT_RULES:
            headers = ["uid", "title", "folder_uid"]
            rows = [[item.get("uid", ""), item.get("title", ""), item.get("folderUID", "")] for item in items]
        elif args.kind == ResourceKind.ORGS:
            headers = ["id", "name", "address"]
            rows = [[str(item.get("id", "")), item.get("name", ""), item.get("address", "")] for item in items]
        
        from .tabular_output import render_table
        for line in render_table(headers, rows):
            print(line)
        return 0

    dump_document(document, args.output_format)
    return 0


def describe_command(args: argparse.Namespace) -> int:
    kinds = [args.kind] if args.kind else SUPPORTED_KINDS
    records = []
    for kind in kinds:
        records.append({
            "kind": kind,
            "singular": ResourceKind.singular_label(kind),
            "selector": ResourceKind.selector_pattern(kind),
            "list_endpoint": ResourceKind.list_endpoint(kind),
            "get_endpoint": ResourceKind.get_endpoint(kind),
            "description": ResourceKind.description(kind),
        })
    
    document = {
        "kind": args.kind,
        "count": len(records),
        "items": records
    }

    if args.output_format == "table":
        headers = ["kind", "singular", "selector", "list_endpoint", "get_endpoint", "description"]
        rows = [[r["kind"], r["singular"], r["selector"], r["list_endpoint"], r["get_endpoint"], r["description"]] for r in records]
        from .tabular_output import render_table
        for line in render_table(headers, rows):
            print(line)
        return 0

    dump_document(document, args.output_format)
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
            records = []
            for kind in SUPPORTED_KINDS:
                records.append({
                    "kind": kind,
                    "singular": ResourceKind.singular_label(kind),
                    "description": ResourceKind.description(kind),
                })
            if args.output_format == "table":
                headers = ["kind", "singular", "description"]
                rows = [[r["kind"], r["singular"], r["description"]] for r in records]
                from .tabular_output import render_table
                for line in render_table(headers, rows):
                    print(line)
                return 0
            dump_document(records, args.output_format)
            return 0
        if args.command == "describe":
            return describe_command(args)
        if args.command == "list":
            return list_command(args)
        if args.command == "get":
            return get_command(args)
    except Exception as e:
        print(str(e), file=sys.stderr)
        return 1
    return 0
