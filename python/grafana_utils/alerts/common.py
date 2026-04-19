"""Shared alerting constants and exceptions."""

import re

RAW_EXPORT_SUBDIR = "raw"
RULES_SUBDIR = "rules"
CONTACT_POINTS_SUBDIR = "contact-points"
MUTE_TIMINGS_SUBDIR = "mute-timings"
POLICIES_SUBDIR = "policies"
TEMPLATES_SUBDIR = "templates"
LINKED_DASHBOARD_ANNOTATION_KEY = "__dashboardUid__"
LINKED_PANEL_ANNOTATION_KEY = "__panelId__"

RULE_KIND = "grafana-alert-rule"
CONTACT_POINT_KIND = "grafana-contact-point"
MUTE_TIMING_KIND = "grafana-mute-timing"
POLICIES_KIND = "grafana-notification-policies"
TEMPLATE_KIND = "grafana-notification-template"
TOOL_API_VERSION = 1
TOOL_SCHEMA_VERSION = 1
ROOT_INDEX_KIND = "grafana-utils-alert-export-index"

ALERT_PLAN_KIND = "grafana-util-alert-plan"
ALERT_PLAN_SCHEMA_VERSION = 1
ALERT_DELETE_PREVIEW_KIND = "grafana-util-alert-delete-preview"
ALERT_DELETE_PREVIEW_SCHEMA_VERSION = 1
ALERT_IMPORT_DRY_RUN_KIND = "grafana-util-alert-import-dry-run"
ALERT_IMPORT_DRY_RUN_SCHEMA_VERSION = 1
ALERT_MANAGED_POLICY_PREVIEW_SCHEMA_VERSION = 1

MANAGED_ROUTE_LABEL_KEY = "grafana_utils_route"


def sanitize_path_component(value: str) -> str:
    """Sanitize path component implementation."""
    normalized = re.sub(r"[^\w.\- ]+", "_", value.strip(), flags=re.UNICODE)
    normalized = re.sub(r"\s+", "_", normalized)
    normalized = re.sub(r"_+", "_", normalized)
    normalized = normalized.strip("._")
    return normalized or "untitled"


def value_to_string(value: any) -> str:
    """Convert any value to a stable string representation."""
    if value is None:
        return ""
    if isinstance(value, bool):
        return str(value).lower()
    return str(value)

RESOURCE_SUBDIR_BY_KIND = {
    RULE_KIND: RULES_SUBDIR,
    CONTACT_POINT_KIND: CONTACT_POINTS_SUBDIR,
    MUTE_TIMING_KIND: MUTE_TIMINGS_SUBDIR,
    POLICIES_KIND: POLICIES_SUBDIR,
    TEMPLATE_KIND: TEMPLATES_SUBDIR,
}
SERVER_MANAGED_FIELDS_BY_KIND = {
    RULE_KIND: {"id", "updated", "provenance"},
    CONTACT_POINT_KIND: {"provenance"},
    MUTE_TIMING_KIND: {"version", "provenance"},
    POLICIES_KIND: {"provenance"},
    TEMPLATE_KIND: {"provenance"},
}


class GrafanaError(RuntimeError):
    """Raised when Grafana returns an unexpected response."""


class GrafanaApiError(GrafanaError):
    """Raised when Grafana returns an HTTP error response."""

    def __init__(self, status_code: int, url: str, body: str) -> None:
        # Purpose: implementation note.
        # Args: see function signature.
        # Returns: see implementation.

        # Call graph: see callers/callees.
        #   Upstream callers: 無
        #   Downstream callees: 無

        self.status_code = status_code
        self.url = url
        self.body = body
        super().__init__("Grafana API error %s for %s: %s" % (status_code, url, body))
