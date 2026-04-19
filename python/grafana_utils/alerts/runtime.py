"""Alert plan and apply orchestration."""

import json
from pathlib import Path
from typing import Any, Optional

from grafana_utils import __version__ as TOOL_VERSION

from .common import (
    ALERT_DELETE_PREVIEW_KIND,
    ALERT_DELETE_PREVIEW_SCHEMA_VERSION,
    ALERT_PLAN_KIND,
    ALERT_PLAN_SCHEMA_VERSION,
    CONTACT_POINT_KIND,
    LINKED_DASHBOARD_ANNOTATION_KEY,
    LINKED_PANEL_ANNOTATION_KEY,
    MUTE_TIMING_KIND,
    POLICIES_KIND,
    RULE_KIND,
    TEMPLATE_KIND,
    value_to_string,
    GrafanaError,
    GrafanaApiError,
    RESOURCE_SUBDIR_BY_KIND,
)
from .provisioning import (
    build_compare_document,
    build_import_operation,
    build_contact_point_import_payload,
    build_mute_timing_import_payload,
    build_resource_identity,
    build_rule_import_payload,
    build_template_import_payload,
    build_policies_import_payload,
    fetch_live_compare_document,
    strip_server_managed_fields,
)


def build_plan_summary(rows: list[dict[str, Any]]) -> dict[str, Any]:
    """Build plan summary implementation."""

    def count(action: str) -> int:
        return len([r for r in rows if r.get("action") == action])

    warning = len([r for r in rows if r.get("status") == "warning"])
    return {
        "processed": len(rows),
        "create": count("create"),
        "update": count("update"),
        "noop": count("noop"),
        "delete": count("delete"),
        "blocked": count("blocked"),
        "warning": warning,
    }


def compare_field_changes(
    desired: Optional[dict[str, Any]],
    live: Optional[dict[str, Any]],
) -> tuple[list[str], list[dict[str, Any]]]:
    """Compare two objects and return changed fields and detailed changes."""
    fields = set()
    if desired:
        fields.update(desired.keys())
    if live:
        fields.update(live.keys())

    changed_fields = []
    changes = []
    for field in sorted(fields):
        desired_val = desired.get(field) if desired else None
        live_val = live.get(field) if live else None
        if desired_val != live_val:
            changed_fields.append(field)
            changes.append(
                {
                    "field": field,
                    "before": live_val,
                    "after": desired_val,
                }
            )
    return changed_fields, changes


def build_rule_review_hints(payload: dict[str, Any]) -> list[dict[str, Any]]:
    """Build rule review hints implementation."""
    annotations = payload.get("annotations")
    if not isinstance(annotations, dict):
        return []

    hints = []
    dashboard_uid = value_to_string(annotations.get(LINKED_DASHBOARD_ANNOTATION_KEY))
    if dashboard_uid:
        hints.append(
            {
                "code": "linked-dashboard-reference",
                "field": "annotations.__dashboardUid__",
                "before": dashboard_uid,
                "after": dashboard_uid,
            }
        )

    if LINKED_PANEL_ANNOTATION_KEY in annotations:
        panel_id = annotations.get(LINKED_PANEL_ANNOTATION_KEY)
        if panel_id is not None:
            panel_value = value_to_string(panel_id)
            hints.append(
                {
                    "code": "linked-panel-reference",
                    "field": "annotations.__panelId__",
                    "before": panel_value,
                    "after": panel_value,
                }
            )

    return hints


def build_alert_delete_preview_document(
    rows: list[dict[str, Any]],
    allow_policy_reset: bool,
) -> dict[str, Any]:
    """Build a delete-preview document implementation."""

    def count(action: str) -> int:
        return len([row for row in rows if str(row.get("action")) == action])

    return {
        "kind": ALERT_DELETE_PREVIEW_KIND,
        "schemaVersion": ALERT_DELETE_PREVIEW_SCHEMA_VERSION,
        "toolVersion": TOOL_VERSION,
        "reviewRequired": True,
        "reviewed": False,
        "allowPolicyReset": bool(allow_policy_reset),
        "summary": {
            "processed": len(rows),
            "delete": count("delete"),
            "blocked": count("blocked"),
        },
        "rows": rows,
    }


def init_alert_runtime_layout(root: Path) -> dict[str, Any]:
    """Create a scaffolded desired-state tree and return a bootstrap document."""

    created: list[str] = []
    root = Path(root)
    root.mkdir(parents=True, exist_ok=True)
    created.append(str(root))
    for _, subdir in RESOURCE_SUBDIR_BY_KIND.items():
        path = root / subdir
        path.mkdir(parents=True, exist_ok=True)
        created.append(str(path))

    return {
        "kind": "grafana-util-alert-init",
        "root": str(root),
        "created": created,
    }


def build_plan_row(
    kind: str,
    identity: str,
    action: str,
    reason: str,
    path: Optional[Path] = None,
    desired: Optional[dict[str, Any]] = None,
    live: Optional[dict[str, Any]] = None,
    review_hints: Optional[list[dict[str, Any]]] = None,
    blocked_reason: Optional[str] = None,
) -> dict[str, Any]:
    """Build a single row for the alert plan."""
    changed_fields, changes = (
        ([], []) if action == "noop" else compare_field_changes(desired, live)
    )

    status = "ready"
    if blocked_reason:
        status = "blocked"
    elif review_hints:
        status = "warning"
    elif action == "noop":
        status = "same"

    return {
        "domain": "alert",
        "resourceKind": kind,
        "kind": kind,
        "identity": identity,
        "actionId": f"{kind}::{identity}::{action}",
        "action": action,
        "status": status,
        "reason": reason,
        "blockedReason": blocked_reason,
        "reviewHints": review_hints or [],
        "changedFields": changed_fields,
        "changes": changes,
        "path": str(path) if path else None,
        "desired": desired,
        "live": live,
    }


def build_alert_plan_document(rows: list[dict[str, Any]], allow_prune: bool) -> dict[str, Any]:
    """Build the full alert plan document."""
    return {
        "kind": ALERT_PLAN_KIND,
        "schemaVersion": ALERT_PLAN_SCHEMA_VERSION,
        "toolVersion": TOOL_VERSION,
        "reviewRequired": True,
        "reviewed": False,
        "allowPrune": allow_prune,
        "summary": build_plan_summary(rows),
        "rows": rows,
    }


def build_alert_plan(
    client: Any,
    desired_dir: Path,
    allow_prune: bool,
) -> dict[str, Any]:
    """Build an alert plan by comparing local desired state with live Grafana."""
    from grafana_utils.alert_cli import discover_alert_resource_files

    resource_files = discover_alert_resource_files(desired_dir)
    rows = []
    desired_keys = set()

    for path in resource_files:
        try:
            with open(path, "r", encoding="utf-8") as f:
                document = json.load(f)
        except (OSError, json.JSONDecodeError) as exc:
            raise GrafanaError(f"Failed to load {path}: {exc}")

        kind, payload = build_import_operation(document)
        identity = build_resource_identity(kind, payload)
        key = (kind, identity)
        if key in desired_keys:
            raise GrafanaError(
                f"Duplicate alert desired identity detected for kind={kind} id={identity}."
            )
        desired_keys.add(key)

        desired_compare = build_compare_document(kind, payload)
        remote_compare = fetch_live_compare_document(client, kind, payload)

        if remote_compare is None:
            action = "create"
            reason = "missing-live"
        elif remote_compare == desired_compare:
            action = "noop"
            reason = "in-sync"
        else:
            action = "update"
            reason = "drift-detected"

        review_hints = build_rule_review_hints(payload) if kind == RULE_KIND else []
        rows.append(
            build_plan_row(
                kind=kind,
                identity=identity,
                action=action,
                reason=reason,
                path=path,
                desired=payload,
                live=remote_compare,
                review_hints=review_hints,
            )
        )

    # Check for live resources to prune
    for kind, subdir in RESOURCE_SUBDIR_BY_KIND.items():
        if kind == RULE_KIND:
            live_items = client.list_alert_rules()
        elif kind == CONTACT_POINT_KIND:
            live_items = client.list_contact_points()
        elif kind == MUTE_TIMING_KIND:
            live_items = client.list_mute_timings()
        elif kind == TEMPLATE_KIND:
            live_items = client.list_templates()
        elif kind == POLICIES_KIND:
            live_items = [client.get_notification_policies()]
        else:
            continue

        for item in live_items:
            payload = strip_server_managed_fields(kind, item)
            identity = build_resource_identity(kind, payload)
            if (kind, identity) in desired_keys:
                continue

            action = "delete" if allow_prune else "blocked"
            reason = "missing-from-desired-state" if allow_prune else "prune-required"
            blocked_reason = None if allow_prune else "prune-required"

            rows.append(
                build_plan_row(
                    kind=kind,
                    identity=identity,
                    action=action,
                    reason=reason,
                    blocked_reason=blocked_reason,
                    live=payload,
                )
            )

    return build_alert_plan_document(rows, allow_prune)


SUPPORTED_ALERT_PLAN_KINDS = (
    RULE_KIND,
    CONTACT_POINT_KIND,
    MUTE_TIMING_KIND,
    POLICIES_KIND,
    TEMPLATE_KIND,
)


def execute_alert_plan(
    client: Any,
    plan_document: dict[str, Any],
    allow_policy_reset: bool,
) -> dict[str, Any]:
    """Execute the operations defined in an alert plan."""
    if plan_document.get("kind") != ALERT_PLAN_KIND:
        raise GrafanaError("Alert plan document kind is not supported.")
    rows = plan_document.get("rows")
    if not isinstance(rows, list):
        raise GrafanaError("Alert plan document is missing rows.")

    results = []
    applied_count = 0

    for row in rows:
        if not isinstance(row, dict):
            raise GrafanaError("Alert plan row is not an object.")
        action = row.get("action")
        if action not in ("create", "update", "delete"):
            continue

        kind = row.get("kind")
        if not isinstance(kind, str):
            raise GrafanaError("Alert plan row is missing kind.")
        identity = row.get("identity")
        if kind not in SUPPORTED_ALERT_PLAN_KINDS:
            raise GrafanaError("Unsupported alert plan row kind: %r" % (kind,))
        response = None

        if action == "create":
            desired = row.get("desired")
            if not isinstance(desired, dict):
                raise GrafanaError("Alert plan row is missing desired.")
            if kind == RULE_KIND:
                response = client.create_alert_rule(
                    build_rule_import_payload(desired)
                )
            elif kind == CONTACT_POINT_KIND:
                response = client.create_contact_point(
                    build_contact_point_import_payload(desired)
                )
            elif kind == MUTE_TIMING_KIND:
                response = client.create_mute_timing(
                    build_mute_timing_import_payload(desired)
                )
            elif kind == TEMPLATE_KIND:
                payload = build_template_import_payload(desired)
                template_name = str(identity or payload.get("name") or "")
                payload.pop("name", None)
                payload["version"] = ""
                response = client.request_json(
                    f"/api/v1/provisioning/templates/{template_name}",
                    method="PUT",
                    payload=payload,
                )
            elif kind == POLICIES_KIND:
                response = client.update_notification_policies(
                    build_policies_import_payload(desired)
                )
        elif action == "update":
            desired = row.get("desired")
            if not isinstance(desired, dict):
                raise GrafanaError("Alert plan row is missing desired.")
            if kind == RULE_KIND:
                payload = build_rule_import_payload(desired)
                if not payload.get("uid") and identity:
                    payload["uid"] = str(identity)
                response = client.update_alert_rule(str(payload.get("uid", "") or ""), payload)
            elif kind == CONTACT_POINT_KIND:
                payload = build_contact_point_import_payload(desired)
                if not payload.get("uid") and identity:
                    payload["uid"] = str(identity)
                response = client.update_contact_point(
                    str(payload.get("uid", "") or ""),
                    payload,
                )
            elif kind == MUTE_TIMING_KIND:
                response = client.update_mute_timing(
                    identity, build_mute_timing_import_payload(desired)
                )
            elif kind == TEMPLATE_KIND:
                payload = build_template_import_payload(desired)
                payload.pop("name", None)
                template_name = str(identity or payload.get("name") or "")
                existing = None
                try:
                    existing = client.get_template(template_name)
                except GrafanaApiError as exc:
                    if getattr(exc, "status_code", None) != 404:
                        raise
                payload["version"] = str(
                    existing.get("version", "") if isinstance(existing, dict) else ""
                )
                response = client.request_json(
                    f"/api/v1/provisioning/templates/{template_name}",
                    method="PUT",
                    payload=payload,
                )
            elif kind == POLICIES_KIND:
                response = client.update_notification_policies(
                    build_policies_import_payload(desired)
                )
        elif action == "delete":
            if kind == RULE_KIND:
                response = client.request_json(f"/api/v1/provisioning/alert-rules/{identity}", method="DELETE")
            elif kind == CONTACT_POINT_KIND:
                response = client.request_json(f"/api/v1/provisioning/contact-points/{identity}", method="DELETE")
            elif kind == MUTE_TIMING_KIND:
                response = client.request_json(
                    f"/api/v1/provisioning/mute-timings/{identity}",
                    params={"version": ""},
                    method="DELETE",
                )
            elif kind == TEMPLATE_KIND:
                response = client.request_json(
                    f"/api/v1/provisioning/templates/{identity}",
                    params={"version": ""},
                    method="DELETE",
                )
            elif kind == POLICIES_KIND:
                if not allow_policy_reset:
                    raise GrafanaError(
                        "Refusing live notification policy reset without --allow-policy-reset."
                    )
                response = client.request_json(
                    "/api/v1/provisioning/policies",
                    method="DELETE",
                )
            else:
                raise GrafanaError("Unsupported alert delete kind: %r" % (kind,))

        if response is None:
            response = {}

        applied_count += 1
        results.append(
            {
                "kind": kind,
                "identity": identity,
                "action": action,
                "response": response,
            }
        )

    return {
        "kind": "grafana-util-alert-apply-result",
        "mode": "apply",
        "allowPolicyReset": allow_policy_reset,
        "appliedCount": applied_count,
        "results": results,
    }
