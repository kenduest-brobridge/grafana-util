"""Alert plan and apply orchestration."""

import json
from pathlib import Path
from typing import Any, Callable, Optional

from .common import (
    ALERT_DELETE_PREVIEW_KIND,
    ALERT_DELETE_PREVIEW_SCHEMA_VERSION,
    ALERT_PLAN_KIND,
    ALERT_PLAN_SCHEMA_VERSION,
    CONTACT_POINT_KIND,
    MUTE_TIMING_KIND,
    POLICIES_KIND,
    RULE_KIND,
    TEMPLATE_KIND,
    GrafanaError,
    RESOURCE_SUBDIR_BY_KIND,
)
from .provisioning import (
    build_compare_document,
    build_import_operation,
    build_resource_identity,
    detect_document_kind,
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
    from .provisioning import discover_alert_resource_files

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

        rows.append(
            build_plan_row(
                kind=kind,
                identity=identity,
                action=action,
                reason=reason,
                path=path.relative_to(desired_dir) if desired_dir in path.parents else path,
                desired=payload,
                live=remote_compare.get("spec") if remote_compare else None,
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


def execute_alert_plan(
    client: Any,
    plan_document: dict[str, Any],
    allow_policy_reset: bool,
) -> dict[str, Any]:
    """Execute the operations defined in an alert plan."""
    if plan_document.get("kind") != ALERT_PLAN_KIND:
        raise GrafanaError("Alert plan document kind is not supported.")

    rows = plan_document.get("rows", [])
    results = []
    applied_count = 0

    for row in rows:
        action = row.get("action")
        if action not in ("create", "update", "delete"):
            continue

        kind = row.get("kind")
        identity = row.get("identity")
        desired = row.get("desired")

        if action == "create":
            if kind == RULE_KIND:
                response = client.create_alert_rule(desired)
            elif kind == CONTACT_POINT_KIND:
                response = client.create_contact_point(desired)
            elif kind == MUTE_TIMING_KIND:
                response = client.create_mute_timing(desired)
            elif kind == TEMPLATE_KIND:
                # Templates use update for both create/update in some cases,
                # but let's follow standard client if available.
                response = client.update_template(identity, desired)
            else:
                response = client.update_notification_policies(desired)
        elif action == "update":
            if kind == RULE_KIND:
                response = client.update_alert_rule(identity, desired)
            elif kind == CONTACT_POINT_KIND:
                response = client.update_contact_point(identity, desired)
            elif kind == MUTE_TIMING_KIND:
                response = client.update_mute_timing(identity, desired)
            elif kind == TEMPLATE_KIND:
                response = client.update_template(identity, desired)
            else:
                response = client.update_notification_policies(desired)
        elif action == "delete":
            if kind == POLICIES_KIND and not allow_policy_reset:
                continue # Should have been blocked in plan
            
            if kind == RULE_KIND:
                response = client.request_json(f"/api/v1/provisioning/alert-rules/{identity}", method="DELETE")
            elif kind == CONTACT_POINT_KIND:
                response = client.request_json(f"/api/v1/provisioning/contact-points/{identity}", method="DELETE")
            elif kind == MUTE_TIMING_KIND:
                response = client.request_json(f"/api/v1/provisioning/mute-timings/{identity}", method="DELETE")
            elif kind == TEMPLATE_KIND:
                response = client.request_json(f"/api/v1/provisioning/templates/{identity}", method="DELETE")
            else:
                 # Policy reset usually means applying a default policy
                 continue 

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
