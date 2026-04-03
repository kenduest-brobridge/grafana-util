"""Dashboard delete planning helpers."""

import copy
from typing import Any, Optional

from .common import DEFAULT_DASHBOARD_TITLE, DEFAULT_FOLDER_TITLE, GrafanaError
from .folder_support import collect_folder_inventory
from .listing import attach_dashboard_folder_paths


DELETE_OUTPUT_FORMAT_CHOICES = ("text", "table", "json")


def clone_args(args: Any, **overrides: Any) -> Any:
    """Clone an argparse namespace-like object with overrides."""
    values = dict(vars(args))
    values.update(overrides)
    return type(args)(**values)


def normalize_folder_path(path: Optional[str]) -> str:
    """Normalize folder path input for exact and subtree matches."""
    normalized = str(path or "").strip()
    parts = [part.strip() for part in normalized.split("/") if part.strip()]
    if parts:
        normalized = " / ".join(parts)
    return normalized


def validate_delete_args(args: Any) -> None:
    """Validate dashboard delete arguments after parse normalization."""
    uid = str(getattr(args, "uid", "") or "").strip()
    path = normalize_folder_path(getattr(args, "path", None))
    interactive = bool(getattr(args, "interactive", False))
    dry_run = bool(getattr(args, "dry_run", False))
    delete_folders = bool(getattr(args, "delete_folders", False))
    table = bool(getattr(args, "table", False))
    json_output = bool(getattr(args, "json", False))

    if uid and path:
        raise GrafanaError("Choose either --uid or --path for dashboard delete.")
    if delete_folders and not path:
        raise GrafanaError("--delete-folders requires --path for dashboard delete.")
    if table and not dry_run:
        raise GrafanaError("--table is only supported with --dry-run for dashboard delete.")
    if json_output and not dry_run:
        raise GrafanaError("--json is only supported with --dry-run for dashboard delete.")
    if getattr(args, "no_header", False) and not table:
        raise GrafanaError(
            "--no-header is only supported with --dry-run --table for dashboard delete."
        )
    if interactive:
        if table or json_output or getattr(args, "output_format", None):
            raise GrafanaError(
                "--interactive cannot be combined with machine-readable dashboard delete output."
            )
        return
    if not uid and not path:
        raise GrafanaError("Dashboard delete requires --uid or --path unless --interactive is used.")
    if not dry_run and not bool(getattr(args, "yes", False)):
        raise GrafanaError("Dashboard delete requires --yes unless --dry-run is set.")


def build_delete_record(
    kind: str,
    uid: str,
    name: str,
    folder_path: str,
    action: str,
) -> dict[str, str]:
    """Build one stable delete dry-run record."""
    return {
        "kind": kind,
        "uid": uid,
        "name": name,
        "folderPath": folder_path,
        "action": action,
    }


def build_delete_plan(client: Any, args: Any) -> dict[str, Any]:
    """Resolve delete targets from live Grafana inventory."""
    page_size = int(getattr(args, "page_size", 500))
    summaries = attach_dashboard_folder_paths(
        client,
        client.iter_dashboard_summaries(page_size),
    )
    uid = str(getattr(args, "uid", "") or "").strip()
    root_path = normalize_folder_path(getattr(args, "path", None))
    matched_dashboards: list[dict[str, Any]] = []

    if uid:
        for summary in summaries:
            if str(summary.get("uid") or "").strip() == uid:
                matched_dashboards = [dict(summary)]
                break
        if not matched_dashboards:
            raise GrafanaError("Dashboard not found by uid: %s" % uid)
    elif root_path:
        matched_dashboards = [
            dict(summary)
            for summary in summaries
            if _folder_path_matches(summary.get("folderPath"), root_path)
        ]
        if not matched_dashboards:
            raise GrafanaError("Dashboard folder path did not match any dashboards: %s" % root_path)
    else:
        raise GrafanaError("Dashboard delete plan requires a selector.")

    matched_dashboards.sort(
        key=lambda item: (
            str(item.get("folderPath") or DEFAULT_FOLDER_TITLE),
            str(item.get("title") or DEFAULT_DASHBOARD_TITLE),
            str(item.get("uid") or ""),
        )
    )

    folders: list[dict[str, str]] = []
    if root_path and bool(getattr(args, "delete_folders", False)):
        org = client.fetch_current_org()
        folder_inventory = collect_folder_inventory(client, org, matched_dashboards)
        folders = [
            dict(item)
            for item in folder_inventory
            if _folder_path_matches(item.get("path"), root_path)
        ]
        folders.sort(
            key=lambda item: (
                -str(item.get("path") or "").count(" / "),
                str(item.get("path") or ""),
                str(item.get("uid") or ""),
            )
        )

    dashboard_records = [
        build_delete_record(
            "dashboard",
            str(item.get("uid") or ""),
            str(item.get("title") or DEFAULT_DASHBOARD_TITLE),
            str(item.get("folderPath") or item.get("folderTitle") or DEFAULT_FOLDER_TITLE),
            "delete",
        )
        for item in matched_dashboards
    ]
    folder_records = [
        build_delete_record(
            "folder",
            str(item.get("uid") or ""),
            str(item.get("title") or DEFAULT_FOLDER_TITLE),
            str(item.get("path") or DEFAULT_FOLDER_TITLE),
            "delete-folder",
        )
        for item in folders
    ]
    return {
        "selector": {
            "uid": uid,
            "path": root_path,
            "interactive": bool(getattr(args, "interactive", False)),
        },
        "deleteFolders": bool(getattr(args, "delete_folders", False)),
        "dashboards": matched_dashboards,
        "folders": folders,
        "dashboardRecords": dashboard_records,
        "folderRecords": folder_records,
        "records": dashboard_records + folder_records,
        "summary": {
            "dashboardCount": len(dashboard_records),
            "folderCount": len(folder_records),
        },
    }


def execute_delete_plan(client: Any, plan: dict[str, Any]) -> dict[str, int]:
    """Execute one delete plan against Grafana."""
    for item in plan.get("dashboards", []):
        client.delete_dashboard(str(item.get("uid") or ""))
    for item in plan.get("folders", []):
        client.delete_folder(str(item.get("uid") or ""))
    summary = plan.get("summary") or {}
    return {
        "dashboardCount": int(summary.get("dashboardCount") or 0),
        "folderCount": int(summary.get("folderCount") or 0),
    }


def _folder_path_matches(candidate: Any, root_path: str) -> bool:
    normalized = normalize_folder_path(candidate)
    return normalized == root_path or normalized.startswith(root_path + " / ")
