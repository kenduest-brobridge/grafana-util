"""Dashboard delete rendering helpers."""

import json
from typing import Any


DELETE_TABLE_HEADERS = (
    ("kind", "KIND"),
    ("uid", "UID"),
    ("name", "NAME"),
    ("folderPath", "FOLDER_PATH"),
    ("action", "ACTION"),
)


def render_dashboard_delete_table(
    records: list[dict[str, str]],
    include_header: bool = True,
) -> list[str]:
    """Render dashboard delete records as a compact table."""
    widths = [
        max(
            len(header),
            *(len(str(item.get(key) or "")) for item in records),
        )
        for key, header in DELETE_TABLE_HEADERS
    ]

    def render_row(values: list[str]) -> str:
        return "  ".join(
            value.ljust(widths[index]) for index, value in enumerate(values)
        ).rstrip()

    lines: list[str] = []
    if include_header:
        lines.append(render_row([header for _key, header in DELETE_TABLE_HEADERS]))
        lines.append(render_row(["-" * width for width in widths]))
    for item in records:
        lines.append(
            render_row([str(item.get(key) or "") for key, _header in DELETE_TABLE_HEADERS])
        )
    return lines


def render_dashboard_delete_json(plan: dict[str, Any]) -> str:
    """Render a delete plan as structured JSON."""
    payload = {
        "selector": dict(plan.get("selector") or {}),
        "deleteFolders": bool(plan.get("deleteFolders", False)),
        "items": [dict(item) for item in (plan.get("records") or [])],
        "summary": dict(plan.get("summary") or {}),
    }
    return json.dumps(payload, indent=2, sort_keys=False)


def render_dashboard_delete_text(plan: dict[str, Any]) -> list[str]:
    """Render dashboard delete dry-run lines in plain text."""
    lines = []
    for item in plan.get("dashboardRecords", []):
        lines.append(
            "Dry-run dashboard delete uid=%s name=%s folderPath=%s action=%s"
            % (
                item.get("uid") or "-",
                item.get("name") or "-",
                item.get("folderPath") or "-",
                item.get("action") or "-",
            )
        )
    for item in plan.get("folderRecords", []):
        lines.append(
            "Dry-run folder delete uid=%s title=%s path=%s action=%s"
            % (
                item.get("uid") or "-",
                item.get("name") or "-",
                item.get("folderPath") or "-",
                item.get("action") or "-",
            )
        )
    summary = plan.get("summary") or {}
    lines.append(
        "Dry-run matched %s dashboard(s)%s"
        % (
            int(summary.get("dashboardCount") or 0),
            (
                " and %s folder(s)" % int(summary.get("folderCount") or 0)
                if int(summary.get("folderCount") or 0) > 0
                else ""
            ),
        )
    )
    return lines


def format_live_dashboard_delete_line(item: dict[str, Any]) -> str:
    """Render one live dashboard delete status line."""
    return "Deleted dashboard uid=%s name=%s folderPath=%s" % (
        item.get("uid") or "-",
        item.get("title") or item.get("name") or "-",
        item.get("folderPath") or item.get("folderTitle") or "-",
    )


def format_live_folder_delete_line(item: dict[str, Any]) -> str:
    """Render one live folder delete status line."""
    return "Deleted folder uid=%s title=%s path=%s" % (
        item.get("uid") or "-",
        item.get("title") or item.get("name") or "-",
        item.get("path") or item.get("folderPath") or "-",
    )
