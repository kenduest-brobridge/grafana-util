"""Dashboard live authoring and history helpers for the Python CLI."""

from __future__ import annotations

import copy
import json
import sys
from pathlib import Path
from typing import Any, Optional

from .clients.dashboard_client import GrafanaClient
from .dashboards.common import DEFAULT_DASHBOARD_TITLE, DEFAULT_FOLDER_UID, GrafanaError
from .dashboards.import_support import (
    build_import_payload,
    determine_dashboard_import_action,
    extract_dashboard_object,
    load_json_file,
)
from .dashboards.output_support import write_json_document
from .dashboards.transformer import build_preserved_web_import_document, collect_datasource_refs

HISTORY_EXPORT_KIND = "grafana-utils-dashboard-history-export"
HISTORY_LIST_KIND = "grafana-utils-dashboard-history-list"
HISTORY_RESTORE_KIND = "grafana-utils-dashboard-history-restore"
HISTORY_INVENTORY_KIND = "grafana-utils-dashboard-history-inventory"
DEFAULT_HISTORY_RESTORE_MESSAGE = "Restored by grafana-util dashboard history"


def _normalize_text(value: Any, default: str = "") -> str:
    text = str(value or "").strip()
    return text or default


def _load_dashboard_document_from_path(path: Path) -> dict[str, Any]:
    if str(path) == "-":
        try:
            raw = json.loads(sys.stdin.read())
        except json.JSONDecodeError as exc:
            raise GrafanaError(f"Invalid JSON in standard input: {exc}") from exc
        if not isinstance(raw, dict):
            raise GrafanaError("Dashboard file must contain a JSON object: -")
        return raw
    return load_json_file(path)


def load_dashboard_document(path: Path | str) -> dict[str, Any]:
    """Load one dashboard JSON document from disk or standard input."""

    return _load_dashboard_document_from_path(Path(path))


def _dashboard_object(document: dict[str, Any]) -> dict[str, Any]:
    return extract_dashboard_object(
        document,
        "Dashboard file must contain a JSON object.",
    )


def fetch_live_dashboard(client: GrafanaClient, dashboard_uid: str) -> dict[str, Any]:
    """Fetch one live dashboard and strip the Grafana wrapper."""

    payload = client.fetch_dashboard(dashboard_uid)
    return build_preserved_web_import_document(payload)


def clone_live_dashboard(
    client: GrafanaClient,
    source_uid: str,
    name: Optional[str] = None,
    uid: Optional[str] = None,
    folder_uid: Optional[str] = None,
) -> dict[str, Any]:
    """Fetch one live dashboard and apply simple local overrides."""

    document = fetch_live_dashboard(client, source_uid)
    if name is not None:
        document["title"] = name
    if uid is not None:
        document["uid"] = uid
    if folder_uid is not None:
        document["folderUid"] = folder_uid
    document["id"] = None
    return document


def patch_dashboard_document(
    document: dict[str, Any],
    name: Optional[str] = None,
    uid: Optional[str] = None,
    folder_uid: Optional[str] = None,
    message: Optional[str] = None,
    tags: Optional[list[str]] = None,
) -> dict[str, Any]:
    """Patch one dashboard JSON document in place and return it."""

    patched = copy.deepcopy(document)
    dashboard = _dashboard_object(patched)
    if name is not None:
        dashboard["title"] = name
    if uid is not None:
        dashboard["uid"] = uid
    if folder_uid is not None:
        dashboard["folderUid"] = folder_uid
    if tags is not None:
        dashboard["tags"] = list(tags)
    dashboard["id"] = None
    if message is not None:
        meta = patched.get("meta")
        if not isinstance(meta, dict):
            meta = {}
            patched["meta"] = meta
        meta["message"] = message
    if dashboard is not patched:
        patched["dashboard"] = dashboard
    return patched


def build_dashboard_review_document(document: dict[str, Any]) -> dict[str, Any]:
    """Build one lightweight review summary for a local dashboard draft."""

    dashboard = _dashboard_object(document)
    refs: list[Any] = []
    collect_datasource_refs(dashboard, refs)
    unique_refs = []
    seen = set()
    for ref in refs:
        key = json.dumps(ref, sort_keys=True, ensure_ascii=False, default=str)
        if key in seen:
            continue
        seen.add(key)
        unique_refs.append(ref)
    panels = dashboard.get("panels")
    panel_count = len(panels) if isinstance(panels, list) else 0
    folder_uid = _normalize_text(
        dashboard.get("folderUid")
        or (document.get("meta") or {}).get("folderUid")
        or DEFAULT_FOLDER_UID
    )
    return {
        "kind": "grafana-utils-dashboard-review",
        "dashboardUid": _normalize_text(dashboard.get("uid")),
        "title": _normalize_text(dashboard.get("title"), DEFAULT_DASHBOARD_TITLE),
        "folderUid": folder_uid,
        "tagCount": len(dashboard.get("tags") or []),
        "panelCount": panel_count,
        "datasourceRefCount": len(unique_refs),
        "hasInputs": "__inputs" in document or "__inputs" in dashboard,
        "hasRequires": "__requires" in document or "__requires" in dashboard,
    }


def build_dashboard_publish_payload(
    document: dict[str, Any],
    replace_existing: bool,
    message: str,
    folder_uid: Optional[str] = None,
) -> dict[str, Any]:
    """Normalize a local dashboard draft into an import payload."""

    payload = build_import_payload(
        document,
        folder_uid_override=folder_uid,
        replace_existing=replace_existing,
        message=message,
    )
    return payload


def preview_dashboard_publish(
    client: GrafanaClient,
    document: dict[str, Any],
    replace_existing: bool,
    message: str,
    folder_uid: Optional[str] = None,
) -> dict[str, Any]:
    """Predict the import action for one local dashboard draft."""

    payload = build_dashboard_publish_payload(
        document,
        replace_existing=replace_existing,
        message=message,
        folder_uid=folder_uid,
    )
    action = determine_dashboard_import_action(
        client,
        payload,
        replace_existing=replace_existing,
    )
    dashboard = payload["dashboard"]
    return {
        "kind": "grafana-utils-dashboard-publish-preview",
        "dashboardUid": _normalize_text(dashboard.get("uid")),
        "title": _normalize_text(dashboard.get("title"), DEFAULT_DASHBOARD_TITLE),
        "folderUid": _normalize_text(payload.get("folderUid") or folder_uid),
        "action": action,
        "message": message,
    }


def publish_dashboard_document(
    client: GrafanaClient,
    document: dict[str, Any],
    replace_existing: bool,
    message: str,
    folder_uid: Optional[str] = None,
) -> dict[str, Any]:
    """Send one reviewed local dashboard draft back to Grafana."""

    payload = build_dashboard_publish_payload(
        document,
        replace_existing=replace_existing,
        message=message,
        folder_uid=folder_uid,
    )
    response = client.import_dashboard(payload)
    if not isinstance(response, dict):
        raise GrafanaError("Unexpected dashboard publish response from Grafana.")
    return response


def list_dashboard_history_versions(
    client: GrafanaClient,
    dashboard_uid: str,
    limit: int,
) -> list[dict[str, Any]]:
    """Fetch the live version history list for one dashboard."""

    response = client.request_json(
        f"/api/dashboards/uid/{dashboard_uid}/versions",
        params={"limit": limit},
    )
    if isinstance(response, list):
        items = response
    elif isinstance(response, dict):
        items = response.get("versions") or []
    else:
        raise GrafanaError("Unexpected dashboard history versions payload from Grafana.")
    versions = []
    for item in items:
        if not isinstance(item, dict):
            continue
        versions.append(
            {
                "version": int(item.get("version") or 0),
                "created": _normalize_text(item.get("created"), "-"),
                "createdBy": _normalize_text(item.get("createdBy"), "-"),
                "message": _normalize_text(item.get("message")),
            }
        )
    return versions


def build_dashboard_history_list_document(
    client: GrafanaClient,
    dashboard_uid: str,
    limit: int,
) -> dict[str, Any]:
    """Build one dashboard history list document from live Grafana."""

    versions = list_dashboard_history_versions(client, dashboard_uid, limit)
    return {
        "kind": HISTORY_LIST_KIND,
        "dashboardUid": dashboard_uid,
        "versionCount": len(versions),
        "versions": versions,
    }


def build_dashboard_history_list_document_from_export(
    document: dict[str, Any],
) -> dict[str, Any]:
    """Build one dashboard history list document from a local export artifact."""

    versions = []
    for item in document.get("versions") or []:
        if not isinstance(item, dict):
            continue
        versions.append(
            {
                "version": int(item.get("version") or 0),
                "created": _normalize_text(item.get("created"), "-"),
                "createdBy": _normalize_text(item.get("createdBy"), "-"),
                "message": _normalize_text(item.get("message")),
            }
        )
    return {
        "kind": HISTORY_LIST_KIND,
        "dashboardUid": _normalize_text(document.get("dashboardUid")),
        "versionCount": len(versions),
        "versions": versions,
    }


def _fetch_dashboard_history_version_dashboard(
    client: GrafanaClient,
    dashboard_uid: str,
    version: int,
) -> dict[str, Any]:
    response = client.request_json(f"/api/dashboards/uid/{dashboard_uid}/versions/{version}")
    if not isinstance(response, dict):
        raise GrafanaError("Unexpected dashboard history version payload from Grafana.")
    data = response.get("data")
    if not isinstance(data, dict):
        raise GrafanaError("Dashboard history version payload did not include dashboard data.")
    return data


def build_dashboard_history_export_document(
    client: GrafanaClient,
    dashboard_uid: str,
    limit: int,
) -> dict[str, Any]:
    """Export one dashboard's revision history into a reusable JSON bundle."""

    current = client.fetch_dashboard(dashboard_uid)
    dashboard = _dashboard_object(current)
    versions = []
    for item in list_dashboard_history_versions(client, dashboard_uid, limit):
        version_number = int(item.get("version") or 0)
        versions.append(
            {
                "version": version_number,
                "created": item.get("created") or "-",
                "createdBy": item.get("createdBy") or "-",
                "message": item.get("message") or "",
                "dashboard": _fetch_dashboard_history_version_dashboard(
                    client, dashboard_uid, version_number
                ),
            }
        )
    return {
        "kind": HISTORY_EXPORT_KIND,
        "dashboardUid": dashboard_uid,
        "currentVersion": int(dashboard.get("version") or 0),
        "currentTitle": _normalize_text(
            dashboard.get("title"), DEFAULT_DASHBOARD_TITLE
        ),
        "versionCount": len(versions),
        "versions": versions,
    }


def load_history_export_document(path: Path | str) -> dict[str, Any]:
    """Load one local dashboard history export artifact from disk."""

    document = load_json_file(Path(path))
    if _normalize_text(document.get("kind")) != HISTORY_EXPORT_KIND:
        raise GrafanaError(
            f"Expected {HISTORY_EXPORT_KIND} at {path}, found {document.get('kind')}."
        )
    return document


def build_history_inventory_document(
    input_dir: Path,
    artifacts: list[tuple[Path, dict[str, Any]]],
) -> dict[str, Any]:
    """Build one inventory document for a local history export tree."""

    items = []
    for path, document in artifacts:
        items.append(
            {
                "dashboardUid": _normalize_text(document.get("dashboardUid")),
                "currentTitle": _normalize_text(document.get("currentTitle")),
                "currentVersion": int(document.get("currentVersion") or 0),
                "versionCount": int(document.get("versionCount") or 0),
                "scope": _derive_history_scope(input_dir, path),
                "path": str(path),
            }
        )
    return {
        "kind": HISTORY_INVENTORY_KIND,
        "artifactCount": len(items),
        "artifacts": items,
    }


def _derive_history_scope(input_dir: Path, artifact_path: Path) -> Optional[str]:
    try:
        relative = artifact_path.relative_to(input_dir)
    except ValueError:
        return None
    parts = list(relative.parts)
    if "history" in parts:
        index = parts.index("history")
        prefix = parts[:index]
        if prefix:
            return "/".join(prefix)
    if len(parts) > 1:
        return parts[0]
    return None


def load_history_artifacts(input_dir: Path) -> list[tuple[Path, dict[str, Any]]]:
    """Load every local dashboard history artifact beneath one input directory."""

    if not input_dir.exists():
        raise GrafanaError(f"Input directory does not exist: {input_dir}")
    if not input_dir.is_dir():
        raise GrafanaError(f"Input path is not a directory: {input_dir}")
    artifacts: list[tuple[Path, dict[str, Any]]] = []
    for path in sorted(input_dir.rglob("*.history.json")):
        artifacts.append((path, load_history_export_document(path)))
    return artifacts


def restore_dashboard_history_version(
    client: GrafanaClient,
    dashboard_uid: str,
    version: int,
    message: Optional[str] = None,
) -> dict[str, Any]:
    """Restore one historical version as a new latest dashboard revision."""

    current = client.fetch_dashboard(dashboard_uid)
    current_dashboard = _dashboard_object(current)
    restored_dashboard = _fetch_dashboard_history_version_dashboard(
        client, dashboard_uid, version
    )
    restored_dashboard["id"] = current_dashboard.get("id")
    restored_dashboard["uid"] = dashboard_uid
    restored_dashboard["version"] = current_dashboard.get("version", 0)
    payload: dict[str, Any] = {
        "dashboard": restored_dashboard,
        "overwrite": True,
        "message": message
        or f"{DEFAULT_HISTORY_RESTORE_MESSAGE} to version {version}",
    }
    current_meta = current.get("meta") if isinstance(current, dict) else None
    if isinstance(current_meta, dict):
        folder_uid = _normalize_text(current_meta.get("folderUid"))
        if folder_uid:
            payload["folderUid"] = folder_uid
    response = client.import_dashboard(payload)
    if not isinstance(response, dict):
        raise GrafanaError("Unexpected dashboard history restore response from Grafana.")
    return {
        "kind": HISTORY_RESTORE_KIND,
        "dashboardUid": dashboard_uid,
        "currentVersion": int(current_dashboard.get("version") or 0),
        "restoreVersion": version,
        "message": payload["message"],
        "response": response,
    }


def preview_dashboard_history_restore(
    client: GrafanaClient,
    dashboard_uid: str,
    version: int,
    message: Optional[str] = None,
) -> dict[str, Any]:
    """Preview one dashboard history restore without mutating Grafana."""

    current = client.fetch_dashboard(dashboard_uid)
    current_dashboard = _dashboard_object(current)
    restored_dashboard = _fetch_dashboard_history_version_dashboard(
        client, dashboard_uid, version
    )
    current_meta = current.get("meta") if isinstance(current, dict) else None
    target_folder_uid = None
    if isinstance(current_meta, dict):
        target_folder_uid = _normalize_text(current_meta.get("folderUid")) or None
    return {
        "kind": HISTORY_RESTORE_KIND,
        "dashboardUid": dashboard_uid,
        "currentVersion": int(current_dashboard.get("version") or 0),
        "restoreVersion": version,
        "currentTitle": _normalize_text(
            current_dashboard.get("title"), DEFAULT_DASHBOARD_TITLE
        ),
        "restoredTitle": _normalize_text(
            restored_dashboard.get("title"), DEFAULT_DASHBOARD_TITLE
        ),
        "targetFolderUid": target_folder_uid,
        "createsNewRevision": True,
        "message": message or f"{DEFAULT_HISTORY_RESTORE_MESSAGE} to version {version}",
        "mode": "dry-run",
    }


def validate_dashboard_export_tree(import_dir: Path) -> dict[str, Any]:
    """Validate one dashboard export tree without mutating Grafana."""
    if not import_dir.exists():
        raise GrafanaError(f"Input directory does not exist: {import_dir}")
    if not import_dir.is_dir():
        raise GrafanaError(f"Input path is not a directory: {import_dir}")
    excluded_names = {
        "index.json",
        "export-metadata.json",
        "folders.json",
        "datasources.json",
        "permissions.json",
    }
    files = [
        path
        for path in sorted(import_dir.rglob("*.json"))
        if path.name not in excluded_names and path.name != ".inspect-source-root"
    ]
    parsed = []
    for path in files:
        parsed.append(load_json_file(path))
        build_import_payload(parsed[-1], folder_uid_override=None, replace_existing=False, message="")
    return {
        "kind": "grafana-utils-dashboard-export-validation",
        "inputDir": str(import_dir),
        "fileCount": len(files),
        "dashboardCount": len(parsed),
    }
