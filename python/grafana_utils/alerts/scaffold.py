"""Alert authoring and scaffolding helpers."""

import json
from pathlib import Path
from typing import Any, Optional

from .common import (
    CONTACT_POINT_KIND,
    CONTACT_POINTS_SUBDIR,
    MANAGED_ROUTE_LABEL_KEY,
    POLICIES_KIND,
    RULE_KIND,
    RULES_SUBDIR,
    TEMPLATE_KIND,
    TEMPLATES_SUBDIR,
    sanitize_path_component,
    value_to_string,
)
from .provisioning import (
    build_contact_point_export_document,
    build_rule_export_document,
    build_template_export_document,
    RESOURCE_SUBDIR_BY_KIND,
)


def scaffold_identity(name: str, fallback: str) -> str:
    """Scaffold identity implementation."""
    identity = sanitize_path_component(name)
    return identity if identity and identity != "untitled" else fallback


def build_folder_resolution_contract(
    folder_uid: str, folder_title: Optional[str] = None
) -> dict[str, str]:
    """Build folder resolution contract implementation."""
    title = (folder_title or "").strip()
    return {
        "folderUid": folder_uid,
        "folderTitle": title,
        "resolution": "uid-only" if not title else "uid-or-title",
    }


def build_stable_route_label_value(name: str) -> str:
    """Build stable route label value implementation."""
    value = sanitize_path_component(name)
    return value if value and value != "untitled" else "managed-route"


def build_simple_rule_body(
    title: str,
    folder_uid: str,
    rule_group: str,
    route_name: str,
) -> dict[str, Any]:
    """Build simple rule body implementation."""
    labels = {
        MANAGED_ROUTE_LABEL_KEY: build_stable_route_label_value(route_name),
    }
    return {
        "uid": scaffold_identity(title, "rule"),
        "title": title,
        "folderUID": folder_uid.strip() if folder_uid.strip() else "general",
        "ruleGroup": rule_group.strip() if rule_group.strip() else "default",
        "condition": "A",
        "data": [],
        "for": "5m",
        "noDataState": "NoData",
        "execErrState": "Alerting",
        "annotations": {},
        "labels": labels,
    }


def build_new_rule_scaffold_document_with_route(
    name: str,
    folder_uid: str,
    rule_group: str,
    route_name: str,
) -> dict[str, Any]:
    """Build new rule scaffold document with route implementation."""
    body = build_simple_rule_body(name, folder_uid, rule_group, route_name)
    document = build_rule_export_document(body)
    metadata = document.setdefault("metadata", {})
    metadata["folder"] = build_folder_resolution_contract(folder_uid)
    metadata["route"] = {
        "labelKey": MANAGED_ROUTE_LABEL_KEY,
        "labelValue": build_stable_route_label_value(route_name),
    }
    return document


def build_new_rule_scaffold_document(name: str) -> dict[str, Any]:
    """Build new rule scaffold document implementation."""
    return build_new_rule_scaffold_document_with_route(name, "general", "default", name)


def build_new_template_scaffold_document(name: str) -> dict[str, Any]:
    """Build new template scaffold document implementation."""
    return build_template_export_document(
        {
            "name": name,
            "template": '{{ define "%s" }}replace me{{ end }}' % name,
        }
    )


def build_contact_point_scaffold_document(
    name: str, channel_type: str = "webhook"
) -> dict[str, Any]:
    """Build contact point scaffold document implementation."""
    identity = scaffold_identity(name, "contact-point")
    if channel_type == "email":
        normalized_type = "email"
        settings = {
            "addresses": ["alerts@example.com"],
            "singleEmail": False,
        }
    elif channel_type == "slack":
        normalized_type = "slack"
        settings = {
            "recipient": "#alerts",
            "token": "${SLACK_BOT_TOKEN}",
            "text": '{{ template "slack.default" . }}',
        }
    else:
        normalized_type = "webhook"
        settings = {"url": "http://127.0.0.1:9000/notify"}

    document = build_contact_point_export_document(
        {
            "uid": identity,
            "name": name,
            "type": normalized_type,
            "settings": settings,
        }
    )

    spec = document.get("spec", {})
    settings_obj = spec.get("settings", {})
    keys = sorted(settings_obj.keys())

    metadata = document.setdefault("metadata", {})
    metadata["authoring"] = {
        "channelType": normalized_type,
        "settingsKeys": keys,
    }
    return document


def build_new_contact_point_scaffold_document(name: str) -> dict[str, Any]:
    """Build new contact point scaffold document implementation."""
    return build_contact_point_scaffold_document(name, "webhook")


def route_matcher_entries(route: dict[str, Any]) -> list[list[str]]:
    """Extract object matchers as string lists."""
    matchers = route.get("object_matchers", [])
    if not isinstance(matchers, list):
        return []
    entries = []
    for matcher in matchers:
        if isinstance(matcher, list) and len(matcher) == 3:
            entries.append([value_to_string(item) for item in matcher])
    return entries


def route_matches_stable_label(route: dict[str, Any], route_name: str) -> bool:
    """Check if a route matches the managed stable label."""
    expected_value = build_stable_route_label_value(route_name)
    for matcher in route_matcher_entries(route):
        if (
            len(matcher) == 3
            and matcher[0] == MANAGED_ROUTE_LABEL_KEY
            and matcher[1] == "="
            and matcher[2] == expected_value
        ):
            return True
    return False


def build_route_preview(route: dict[str, Any]) -> dict[str, Any]:
    """Build a simplified preview of a notification route."""

    def unique_sorted_strings(values: Any) -> list[str]:
        if not isinstance(values, list):
            return []
        items = sorted(set(value_to_string(v) for v in values))
        return items

    group_by = unique_sorted_strings(route.get("group_by", []))
    matchers = route.get("object_matchers", [])
    if not isinstance(matchers, list):
        matchers = []

    return {
        "receiver": str(route.get("receiver", "")),
        "continue": bool(route.get("continue", False)),
        "groupBy": group_by,
        "matchers": matchers,
        "childRouteCount": len(route.get("routes", [])),
    }


def normalize_route_matchers(route: dict[str, Any], route_name: str) -> None:
    """Ensure the managed stable label is present in route matchers."""
    managed_value = build_stable_route_label_value(route_name)
    matchers = route_matcher_entries(route)
    # Remove existing managed label if present
    matchers = [
        m
        for m in matchers
        if not (m[0] == MANAGED_ROUTE_LABEL_KEY and m[1] == "=")
    ]
    matchers.append([MANAGED_ROUTE_LABEL_KEY, "=", managed_value])
    # Sort and dedup
    matchers.sort(key=lambda x: json.dumps(x))
    seen = set()
    deduped = []
    for m in matchers:
        s = json.dumps(m)
        if s not in seen:
            deduped.append(m)
            seen.add(s)
    route["object_matchers"] = deduped


def upsert_managed_policy_subtree(
    policy: dict[str, Any],
    route_name: str,
    route: dict[str, Any],
) -> tuple[dict[str, Any], str]:
    """Insert or update a managed route in the policy tree."""
    normalized_route = dict(route)
    normalize_route_matchers(normalized_route, route_name)

    next_policy = dict(policy)
    routes = next_policy.get("routes", [])
    if not isinstance(routes, list):
        routes = []

    matching_indexes = [
        i for i, r in enumerate(routes) if route_matches_stable_label(r, route_name)
    ]

    if len(matching_indexes) > 1:
        from .common import GrafanaError

        raise GrafanaError(
            "Managed route label %r is not unique in notification policies."
            % build_stable_route_label_value(route_name)
        )

    if matching_indexes:
        index = matching_indexes[0]
        if routes[index] == normalized_route:
            action = "noop"
        else:
            routes[index] = normalized_route
            action = "updated"
    else:
        routes.append(normalized_route)
        action = "created"

    next_policy["routes"] = routes
    return next_policy, action


def remove_managed_policy_subtree(
    policy: dict[str, Any],
    route_name: str,
) -> tuple[dict[str, Any], str]:
    """Remove a managed route from the policy tree."""
    next_policy = dict(policy)
    routes = next_policy.get("routes", [])
    if not isinstance(routes, list):
        return next_policy, "noop"

    matching_indexes = [
        i for i, r in enumerate(routes) if route_matches_stable_label(r, route_name)
    ]

    if len(matching_indexes) > 1:
        from .common import GrafanaError

        raise GrafanaError(
            "Managed route label %r is not unique in notification policies."
            % build_stable_route_label_value(route_name)
        )

    if not matching_indexes:
        return next_policy, "noop"

    index = matching_indexes[0]
    routes.pop(index)
    next_policy["routes"] = routes
    return next_policy, "deleted"


def build_managed_policy_route_preview(
    current_policy: dict[str, Any],
    route_name: str,
    desired_route: Optional[dict[str, Any]],
) -> dict[str, Any]:
    """Build a preview document for managed policy edits."""
    current_route = next(
        (
            r
            for r in current_policy.get("routes", [])
            if route_matches_stable_label(r, route_name)
        ),
        None,
    )

    if desired_route is not None:
        next_policy, action = upsert_managed_policy_subtree(
            current_policy, route_name, desired_route
        )
    else:
        next_policy, action = remove_managed_policy_subtree(current_policy, route_name)

    next_route = next(
        (
            r
            for r in next_policy.get("routes", [])
            if route_matches_stable_label(r, route_name)
        ),
        None,
    )

    return {
        "action": action,
        "managedRouteKey": MANAGED_ROUTE_LABEL_KEY,
        "managedRouteValue": build_stable_route_label_value(route_name),
        "currentRoute": build_route_preview(current_route) if current_route else None,
        "nextRoute": build_route_preview(next_route) if next_route else None,
        "nextPolicyRouteCount": len(next_policy.get("routes", [])),
    }
