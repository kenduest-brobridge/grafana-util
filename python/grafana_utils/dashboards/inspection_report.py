"""Dashboard inspection report model and document helpers."""

from collections import OrderedDict
import re
from pathlib import Path
from typing import Any, Optional

from .common import (
    DEFAULT_DASHBOARD_TITLE,
    DEFAULT_FOLDER_TITLE,
    DEFAULT_UNKNOWN_UID,
    GrafanaError,
)
from .inspection_analyzers import build_query_field_and_text, dispatch_query_analysis
from .inspection_analyzers.contract import unique_strings
from .transformer import is_builtin_datasource_ref, is_placeholder_string

INSPECT_SOURCE_ROOT_FILENAME = ".inspect-source-root"

REPORT_COLUMN_HEADERS = OrderedDict(
    [
        ("dashboardUid", "DASHBOARD_UID"),
        ("dashboardTitle", "DASHBOARD_TITLE"),
        ("dashboardTags", "DASHBOARD_TAGS"),
        ("folderPath", "FOLDER_PATH"),
        ("folderFullPath", "FOLDER_FULL_PATH"),
        ("folderLevel", "FOLDER_LEVEL"),
        ("folderUid", "FOLDER_UID"),
        ("parentFolderUid", "PARENT_FOLDER_UID"),
        ("panelId", "PANEL_ID"),
        ("panelTitle", "PANEL_TITLE"),
        ("panelType", "PANEL_TYPE"),
        ("panelTargetCount", "PANEL_TARGET_COUNT"),
        ("panelQueryCount", "PANEL_EFFECTIVE_QUERY_COUNT"),
        ("panelDatasourceCount", "PANEL_TOTAL_DATASOURCE_COUNT"),
        ("refId", "REF_ID"),
        ("datasource", "DATASOURCE"),
        ("datasourceName", "DATASOURCE_NAME"),
        ("datasourceOrg", "DATASOURCE_ORG"),
        ("datasourceOrgId", "DATASOURCE_ORG_ID"),
        ("datasourceDatabase", "DATASOURCE_DATABASE"),
        ("datasourceBucket", "DATASOURCE_BUCKET"),
        ("datasourceOrganization", "DATASOURCE_ORGANIZATION"),
        ("datasourceIndexPattern", "DATASOURCE_INDEX_PATTERN"),
        ("datasourceType", "DATASOURCE_TYPE"),
        ("datasourceFamily", "DATASOURCE_FAMILY"),
        ("queryField", "QUERY_FIELD"),
        ("targetHidden", "TARGET_HIDDEN"),
        ("targetDisabled", "TARGET_DISABLED"),
        ("queryVariables", "QUERY_VARIABLES"),
        ("panelVariables", "PANEL_VARIABLES"),
        ("metrics", "METRICS"),
        ("functions", "FUNCTIONS"),
        ("measurements", "MEASUREMENTS"),
        ("buckets", "BUCKETS"),
        ("query", "QUERY"),
        ("file", "FILE"),
    ]
)
OPTIONAL_REPORT_COLUMN_HEADERS = OrderedDict([("datasourceUid", "DATASOURCE_UID")])
DEFAULT_REPORT_COLUMN_IDS = (
    "dashboardUid",
    "dashboardTitle",
    "folderPath",
    "folderUid",
    "parentFolderUid",
    "panelId",
    "panelTitle",
    "panelType",
    "refId",
    "datasource",
    "datasourceName",
    "datasourceOrg",
    "datasourceOrgId",
    "datasourceDatabase",
    "datasourceBucket",
    "datasourceOrganization",
    "datasourceIndexPattern",
    "datasourceType",
    "datasourceFamily",
    "queryField",
    "metrics",
    "functions",
    "measurements",
    "buckets",
    "query",
    "file",
)
REPORT_COLUMN_ALIASES = {
    "dashboard_uid": "dashboardUid",
    "dashboard_title": "dashboardTitle",
    "dashboard_tags": "dashboardTags",
    "folder_path": "folderPath",
    "folder_full_path": "folderFullPath",
    "folder_level": "folderLevel",
    "folder_uid": "folderUid",
    "parent_folder_uid": "parentFolderUid",
    "panel_id": "panelId",
    "panel_title": "panelTitle",
    "panel_type": "panelType",
    "panel_target_count": "panelTargetCount",
    "panel_query_count": "panelQueryCount",
    "panel_datasource_count": "panelDatasourceCount",
    "ref_id": "refId",
    "datasource_name": "datasourceName",
    "datasource_org": "datasourceOrg",
    "datasource_org_id": "datasourceOrgId",
    "datasource_database": "datasourceDatabase",
    "datasource_bucket": "datasourceBucket",
    "datasource_organization": "datasourceOrganization",
    "datasource_index_pattern": "datasourceIndexPattern",
    "query_field": "queryField",
    "target_hidden": "targetHidden",
    "target_disabled": "targetDisabled",
    "query_variables": "queryVariables",
    "panel_variables": "panelVariables",
    "datasource_uid": "datasourceUid",
    "datasource_type": "datasourceType",
    "datasource_family": "datasourceFamily",
}
SUPPORTED_REPORT_COLUMN_HEADERS = OrderedDict(
    list(REPORT_COLUMN_HEADERS.items()) + list(OPTIONAL_REPORT_COLUMN_HEADERS.items())
)
SUPPORTED_REPORT_COLUMN_VALUES = tuple(
    list(REPORT_COLUMN_ALIASES.keys())
    + list(SUPPORTED_REPORT_COLUMN_HEADERS.keys())
)
INSPECT_REPORT_FORMAT_CHOICES = (
    "table",
    "json",
    "csv",
    "tree",
    "tree-table",
    "dependency",
    "dependency-json",
    "governance",
    "governance-json",
)
NORMALIZED_QUERY_REPORT_FIELDS = (
    "dashboardUid",
    "dashboardTitle",
    "dashboardTags",
    "folderPath",
    "folderFullPath",
    "folderLevel",
    "folderUid",
    "parentFolderUid",
    "panelId",
    "panelTitle",
    "panelType",
    "panelTargetCount",
    "panelQueryCount",
    "panelDatasourceCount",
    "refId",
    "datasource",
    "datasourceName",
    "datasourceUid",
    "datasourceOrg",
    "datasourceOrgId",
    "datasourceDatabase",
    "datasourceBucket",
    "datasourceOrganization",
    "datasourceIndexPattern",
    "datasourceType",
    "datasourceFamily",
    "queryField",
    "targetHidden",
    "targetDisabled",
    "queryVariables",
    "panelVariables",
    "query",
    "metrics",
    "functions",
    "measurements",
    "buckets",
    "file",
)
def format_supported_report_column_values() -> str:
    """Render supported report column ids for CLI help and parser errors."""
    # Call graph: see callers/callees.
    #   Upstream callers: 無
    #   Downstream callees: 無

    return ", ".join(["all"] + list(SUPPORTED_REPORT_COLUMN_VALUES))


VARIABLE_TOKEN_PATTERN = re.compile(
    r"""
    \$\{([A-Za-z_][A-Za-z0-9_]*)[^}]*\}
    |\$([A-Za-z_][A-Za-z0-9_]*)
    |\[\[([A-Za-z_][A-Za-z0-9_]*)\]\]
    """,
    re.VERBOSE,
)
QUERY_TEXT_FIELDS = {
    "expr",
    "expression",
    "query",
    "rawSql",
    "sql",
    "rawQuery",
    "jql",
    "logql",
    "search",
    "definition",
    "command",
}


def extract_dashboard_tags(dashboard: dict[str, Any]) -> list[str]:
    """Collect stable dashboard tags from the top-level dashboard object."""
    tags = dashboard.get("tags")
    if not isinstance(tags, list):
        return []
    return unique_strings(
        [str(tag).strip() for tag in tags if str(tag).strip()]
    )


def extract_query_variable_names(query_text: str) -> list[str]:
    """Collect Grafana template variables referenced by one query string."""
    if not query_text:
        return []
    names = []
    for match in VARIABLE_TOKEN_PATTERN.finditer(query_text):
        for group in match.groups():
            if group:
                names.append(group)
                break
    return unique_strings(names)


def _collect_panel_variable_names(value: Any, names: list[str]) -> None:
    """Recursively collect non-query panel variable references."""
    if isinstance(value, str):
        names.extend(extract_query_variable_names(value))
        return
    if isinstance(value, list):
        for item in value:
            _collect_panel_variable_names(item, names)
        return
    if not isinstance(value, dict):
        return
    for key, item in value.items():
        if key in QUERY_TEXT_FIELDS:
            continue
        _collect_panel_variable_names(item, names)


def extract_panel_variables(panel: dict[str, Any]) -> list[str]:
    """Collect template variable references from one panel object."""
    names: list[str] = []
    _collect_panel_variable_names(panel, names)
    return unique_strings(names)


def build_panel_report_context(
    panel: dict[str, Any],
    targets: list[dict[str, Any]],
    datasources_by_uid: dict[str, dict[str, str]],
    datasources_by_name: dict[str, dict[str, str]],
) -> dict[str, Any]:
    """Build panel-level counts and variable references for query rows."""
    datasource_labels = []
    active_query_count = 0
    for target in targets:
        if not _is_target_disabled(target):
            query_field, query_text = build_query_field_and_text(target)
            if query_field and query_text.strip():
                active_query_count += 1
        label = describe_panel_datasource(
            panel,
            target,
            datasources_by_uid,
            datasources_by_name,
        )
        if label and not _is_target_disabled(target) and label not in datasource_labels:
            datasource_labels.append(label)
    return {
        "panelVariables": extract_panel_variables(panel),
        "panelTargetCount": str(len(targets)),
        "panelQueryCount": str(active_query_count),
        "panelDatasourceCount": str(len(datasource_labels)),
    }


def _is_truthy_flag(value: Any) -> bool:
    """Normalize Grafana boolean-like target flags."""
    if isinstance(value, bool):
        return value
    if isinstance(value, (int, float)):
        return value != 0
    text = str(value or "").strip().lower()
    return text in {"true", "1", "yes"}


def _is_target_hidden(target: dict[str, Any]) -> bool:
    """Check whether one target is hidden."""
    return _is_truthy_flag(target.get("hide"))


def _is_target_disabled(target: dict[str, Any]) -> bool:
    """Check whether one target is disabled."""
    return _is_truthy_flag(target.get("disabled"))


def resolve_inspection_source_file_path(import_dir: Path, dashboard_file: Path) -> str:
    """Render the original source file path when inspect uses a merged temp root."""
    source_root_path = import_dir / INSPECT_SOURCE_ROOT_FILENAME
    if not source_root_path.is_file():
        return str(dashboard_file)
    try:
        source_root_text = source_root_path.read_text(encoding="utf-8").strip()
    except OSError:
        return str(dashboard_file)
    if not source_root_text:
        return str(dashboard_file)
    try:
        relative_path = dashboard_file.relative_to(import_dir)
    except ValueError:
        return str(dashboard_file)
    parts = relative_path.parts
    source_root = Path(source_root_text)
    if parts and str(parts[0]).startswith("org_"):
        return str(source_root / parts[0] / "raw" / Path(*parts[1:]))
    return str(source_root / relative_path)


def resolve_inspection_folder_path(
    import_dir: Path,
    dashboard_file: Path,
    folder_record: Optional[dict[str, Any]] = None,
) -> str:
    """Render an operator-friendly folder path for merged inspection roots."""
    if folder_record:
        folder_path = str(
            folder_record.get("path")
            or folder_record.get("title")
            or DEFAULT_FOLDER_TITLE
        ).strip()
        if folder_path:
            return folder_path

    try:
        relative_path = dashboard_file.relative_to(import_dir)
    except ValueError:
        return str(DEFAULT_FOLDER_TITLE)

    parts = list(relative_path.parts[:-1])
    if parts and parts[0].startswith("org_") and len(parts) >= 3 and parts[1] == "raw":
        parts = parts[2:]
    elif parts and parts[0].startswith("org_") and len(parts) >= 2:
        parts = parts[1:]
    folder_path = " / ".join(parts).strip()
    return folder_path or DEFAULT_FOLDER_TITLE


def calculate_folder_level(folder_path: str) -> str:
    """Count the visible Grafana folder depth from one logical folder path."""
    segments = [segment.strip() for segment in str(folder_path or "").split(" / ")]
    level = len([segment for segment in segments if segment])
    return str(level) if level else ""


def calculate_folder_full_path(folder_path: str) -> str:
    """Render one stable slash-prefixed folder path for CSV and JSON output."""
    segments = [segment.strip() for segment in str(folder_path or "").split(" / ")]
    normalized = [segment for segment in segments if segment]
    if not normalized or normalized == [DEFAULT_FOLDER_TITLE]:
        return "/"
    return "/" + "/".join(normalized)


def build_export_inspection_report_document(
    import_dir: Path,
    deps: dict[str, Any],
) -> dict[str, Any]:
    """Analyze one raw export directory and emit one per-query inspection record."""
    # Call graph: see callers/callees.
    #   Upstream callers: 無
    #   Downstream callees: 342

    metadata = deps["load_export_metadata"](
        import_dir, expected_variant=deps["RAW_EXPORT_SUBDIR"]
    )
    dashboard_files = deps["discover_dashboard_files"](import_dir)
    folder_inventory = deps["load_folder_inventory"](import_dir, metadata)
    datasource_inventory = deps["load_datasource_inventory"](import_dir, metadata)
    folder_lookup = deps["build_folder_inventory_lookup"](folder_inventory)
    datasources_by_uid = {}
    datasources_by_name = {}
    for datasource in datasource_inventory:
        uid = str(datasource.get("uid") or "").strip()
        name = str(datasource.get("name") or "").strip()
        if uid:
            datasources_by_uid[uid] = dict(datasource)
        if name:
            datasources_by_name[name] = dict(datasource)
    records = []

    for dashboard_file in dashboard_files:
        document = deps["load_json_file"](dashboard_file)
        dashboard = deps["extract_dashboard_object"](
            document, "Dashboard payload must be a JSON object."
        )
        dashboard_tags = extract_dashboard_tags(dashboard)
        folder_record = deps["resolve_folder_inventory_record_for_dashboard"](
            document,
            dashboard_file,
            import_dir,
            folder_lookup,
        )
        folder_path = resolve_inspection_folder_path(
            import_dir,
            dashboard_file,
            folder_record,
        )
        for panel in deps["iter_dashboard_panels"](dashboard.get("panels")):
            targets = panel.get("targets")
            if not isinstance(targets, list):
                continue
            target_records = [target for target in targets if isinstance(target, dict)]
            panel_context = build_panel_report_context(
                panel,
                target_records,
                datasources_by_uid,
                datasources_by_name,
            )
            for target in target_records:
                records.append(
                    build_query_report_record(
                        import_dir,
                        dashboard,
                        folder_path,
                        folder_record,
                        panel,
                        target,
                        dashboard_file,
                        datasources_by_uid,
                        datasources_by_name,
                        dashboard_tags=dashboard_tags,
                        panel_context=panel_context,
                    )
                )

    records.sort(
        key=lambda item: (
            item["folderPath"],
            item["dashboardTitle"],
            item["dashboardUid"],
            item["panelId"],
            item["refId"],
        )
    )
    return {
        "summary": {
            "dashboardCount": len(
                set(record["dashboardUid"] for record in records)
            ),
            "queryRecordCount": len(records),
        },
        "queries": records,
    }


def describe_export_datasource_ref(
    ref: Any,
    datasources_by_uid: dict[str, dict[str, str]],
    datasources_by_name: dict[str, dict[str, str]],
) -> str:
    """Render one exported datasource reference into a stable label."""
    if ref is None:
        return ""
    if isinstance(ref, str):
        label = ref.strip()
        if not label:
            return ""
        if is_builtin_datasource_ref(label):
            return ""
        datasource = datasources_by_name.get(label)
        if datasource is not None:
            return str(datasource.get("uid") or label)
        return label
    if not isinstance(ref, dict):
        return str(ref).strip()
    uid = str(ref.get("uid") or "").strip()
    name = str(ref.get("name") or "").strip()
    ref_type = str(ref.get("type") or "").strip()
    if uid:
        if is_builtin_datasource_ref(uid):
            return ""
        datasource = datasources_by_uid.get(uid)
        if datasource is not None:
            return str(datasource.get("uid") or uid)
        return uid
    if name:
        datasource = datasources_by_name.get(name)
        if datasource is not None:
            return str(datasource.get("uid") or name)
        return name
    return ref_type


def describe_panel_datasource(
    panel: dict[str, Any],
    target: dict[str, Any],
    datasources_by_uid: dict[str, dict[str, str]],
    datasources_by_name: dict[str, dict[str, str]],
) -> str:
    """Resolve one panel/query datasource label from target or panel scope."""
    target_ref = target.get("datasource")
    panel_ref = panel.get("datasource")
    label = describe_export_datasource_ref(
        target_ref,
        datasources_by_uid,
        datasources_by_name,
    )
    if label:
        return label
    return describe_export_datasource_ref(
        panel_ref,
        datasources_by_uid,
        datasources_by_name,
    )


def describe_panel_datasource_uid(
    panel: dict[str, Any],
    target: dict[str, Any],
    datasources_by_name: dict[str, dict[str, str]],
) -> str:
    """Resolve one best-effort datasource uid for a panel/query target."""
    for ref in (target.get("datasource"), panel.get("datasource")):
        if isinstance(ref, dict):
            uid = str(ref.get("uid") or "").strip()
            if uid:
                return uid
            name = str(ref.get("name") or "").strip()
            if name and datasources_by_name.get(name):
                return str(datasources_by_name[name].get("uid") or "")
        elif isinstance(ref, str):
            name = ref.strip()
            if name and datasources_by_name.get(name):
                return str(datasources_by_name[name].get("uid") or "")
    return ""


def describe_panel_datasource_name(
    panel: dict[str, Any],
    target: dict[str, Any],
    datasources_by_uid: dict[str, dict[str, str]],
    datasources_by_name: dict[str, dict[str, str]],
) -> str:
    """Resolve one best-effort datasource display name for a panel/query target."""
    for ref in (target.get("datasource"), panel.get("datasource")):
        if isinstance(ref, dict):
            uid = str(ref.get("uid") or "").strip()
            name = str(ref.get("name") or "").strip()
            if uid and datasources_by_uid.get(uid):
                return str(datasources_by_uid[uid].get("name") or name or uid)
            if uid:
                return uid
            if name and datasources_by_name.get(name):
                return str(datasources_by_name[name].get("name") or name)
            if name:
                return name
        elif isinstance(ref, str):
            name = ref.strip()
            if name and datasources_by_name.get(name):
                return str(datasources_by_name[name].get("name") or name)
            if name and not is_builtin_datasource_ref(name):
                return name
    return ""


def resolve_panel_datasource_inventory_record(
    panel: dict[str, Any],
    target: dict[str, Any],
    datasources_by_uid: dict[str, dict[str, str]],
    datasources_by_name: dict[str, dict[str, str]],
) -> Optional[dict[str, str]]:
    """Resolve the backing datasource inventory record for one panel/query."""
    for ref in (target.get("datasource"), panel.get("datasource")):
        if isinstance(ref, dict):
            uid = str(ref.get("uid") or "").strip()
            name = str(ref.get("name") or "").strip()
            if uid and datasources_by_uid.get(uid):
                return datasources_by_uid[uid]
            if name and datasources_by_name.get(name):
                return datasources_by_name[name]
        elif isinstance(ref, str):
            name = ref.strip()
            if name and datasources_by_uid.get(name):
                return datasources_by_uid[name]
            if name and datasources_by_name.get(name):
                return datasources_by_name[name]
    return None


def _normalize_datasource_family_name(datasource_type: str) -> str:
    """Internal helper for normalize datasource family name."""
    lowered = str(datasource_type or "").strip().lower()
    if not lowered:
        return "unknown"
    aliases = {
        "grafana-postgresql-datasource": "postgres",
        "grafana-mysql-datasource": "mysql",
    }
    return aliases.get(lowered, lowered)


def describe_panel_datasource_type(
    panel: dict[str, Any],
    target: dict[str, Any],
    datasources_by_uid: dict[str, dict[str, str]],
    datasources_by_name: dict[str, dict[str, str]],
) -> str:
    """Resolve one best-effort datasource plugin type for a panel/query target."""
    for ref in (target.get("datasource"), panel.get("datasource")):
        if isinstance(ref, dict):
            uid = str(ref.get("uid") or "").strip()
            name = str(ref.get("name") or "").strip()
            inventory = None
            if uid:
                inventory = datasources_by_uid.get(uid)
            if inventory is None and name:
                inventory = datasources_by_name.get(name)
            if inventory is not None:
                return str(inventory.get("type") or "").strip()
            ref_type = str(ref.get("type") or "").strip()
            if ref_type:
                return ref_type
        elif isinstance(ref, str):
            name = ref.strip()
            inventory = datasources_by_uid.get(name) or datasources_by_name.get(name)
            if inventory is not None:
                return str(inventory.get("type") or "").strip()
    return ""


def build_query_report_record(
    import_dir: Path,
    dashboard: dict[str, Any],
    folder_path: str,
    folder_record: Optional[dict[str, str]],
    panel: dict[str, Any],
    target: dict[str, Any],
    dashboard_file: Path,
    datasources_by_uid: dict[str, dict[str, str]],
    datasources_by_name: dict[str, dict[str, str]],
    dashboard_tags: Optional[list[str]] = None,
    panel_context: Optional[dict[str, Any]] = None,
) -> dict[str, Any]:
    """Build one canonical per-query inspection row."""
    # Call graph: see callers/callees.
    #   Upstream callers: 141
    #   Downstream callees: 257, 280, 301, 313

    query_field, query_text = build_query_field_and_text(target)
    analysis = dispatch_query_analysis(
        panel,
        target,
        query_field,
        query_text,
        datasources_by_uid,
        datasources_by_name,
    )
    datasource_record = resolve_panel_datasource_inventory_record(
        panel,
        target,
        datasources_by_uid,
        datasources_by_name,
    ) or {}
    record = {
        "dashboardUid": str(dashboard.get("uid") or DEFAULT_UNKNOWN_UID),
        "dashboardTitle": str(dashboard.get("title") or DEFAULT_DASHBOARD_TITLE),
        "dashboardTags": list(dashboard_tags or []),
        "folderPath": str(folder_path or DEFAULT_FOLDER_TITLE),
        "folderFullPath": calculate_folder_full_path(folder_path),
        "folderLevel": calculate_folder_level(folder_path),
        "folderUid": str((folder_record or {}).get("uid") or ""),
        "parentFolderUid": str((folder_record or {}).get("parentUid") or ""),
        "panelId": str(panel.get("id") or ""),
        "panelTitle": str(panel.get("title") or ""),
        "panelType": str(panel.get("type") or ""),
        "panelTargetCount": str((panel_context or {}).get("panelTargetCount") or ""),
        "panelQueryCount": str((panel_context or {}).get("panelQueryCount") or ""),
        "panelDatasourceCount": str(
            (panel_context or {}).get("panelDatasourceCount") or ""
        ),
        "refId": str(target.get("refId") or ""),
        "datasource": describe_panel_datasource(
            panel,
            target,
            datasources_by_uid,
            datasources_by_name,
        ),
        "datasourceName": describe_panel_datasource_name(
            panel,
            target,
            datasources_by_uid,
            datasources_by_name,
        ),
        "datasourceUid": describe_panel_datasource_uid(
            panel,
            target,
            datasources_by_name,
        ),
        "datasourceOrg": str(datasource_record.get("org") or ""),
        "datasourceOrgId": str(datasource_record.get("orgId") or ""),
        "datasourceDatabase": str(datasource_record.get("database") or ""),
        "datasourceBucket": str(datasource_record.get("defaultBucket") or ""),
        "datasourceOrganization": str(datasource_record.get("organization") or ""),
        "datasourceIndexPattern": str(datasource_record.get("indexPattern") or ""),
        "datasourceType": describe_panel_datasource_type(
            panel,
            target,
            datasources_by_uid,
            datasources_by_name,
        ),
        "queryField": query_field,
        "targetHidden": str(_is_target_hidden(target)).lower(),
        "targetDisabled": str(_is_target_disabled(target)).lower(),
        "query": query_text,
        "queryVariables": extract_query_variable_names(query_text),
        "panelVariables": list((panel_context or {}).get("panelVariables") or []),
        "metrics": analysis["metrics"],
        "functions": analysis["functions"],
        "measurements": analysis["measurements"],
        "buckets": analysis["buckets"],
        "file": resolve_inspection_source_file_path(import_dir, dashboard_file),
    }
    record["datasourceFamily"] = _normalize_datasource_family_name(
        record["datasourceType"]
    )
    normalized: dict[str, Any] = {}
    for field in NORMALIZED_QUERY_REPORT_FIELDS:
        value = record.get(field)
        if isinstance(value, list):
            normalized[field] = list(value)
        else:
            normalized[field] = str(value or "")
    return normalized


def parse_report_columns(value: Optional[str]) -> Optional[list[str]]:
    """Parse one report column list into canonical inspection field ids."""
    # Call graph: see callers/callees.
    #   Upstream callers: 無
    #   Downstream callees: 無

    if value is None:
        return None
    columns: list[str] = []
    for item in value.split(","):
        column = item.strip()
        if column:
            columns.append(REPORT_COLUMN_ALIASES.get(column, column))
    if not columns:
        raise GrafanaError(
            "--report-columns requires one or more comma-separated column ids."
        )
    if "all" in columns:
        return list(SUPPORTED_REPORT_COLUMN_HEADERS.keys())
    unknown = [
        column for column in columns if column not in SUPPORTED_REPORT_COLUMN_HEADERS
    ]
    if unknown:
        raise GrafanaError(
            "Unsupported report column(s): %s. Supported values: %s."
            % (
                ", ".join(unknown),
                ", ".join(["all"] + list(SUPPORTED_REPORT_COLUMN_VALUES)),
            )
        )
    return columns


def filter_export_inspection_report_document(
    document: dict[str, Any],
    datasource_label: Optional[str] = None,
    panel_id: Optional[str] = None,
) -> dict[str, Any]:
    """Filter one flat inspection report document to narrower query rows."""
    # Call graph: see callers/callees.
    #   Upstream callers: 無
    #   Downstream callees: 無

    if not datasource_label and not panel_id:
        return document
    normalized_datasource_filter = str(datasource_label or "").strip()
    normalized_panel_id_filter = str(panel_id or "").strip()
    filtered_records = [
        dict(record)
        for record in list(document.get("queries") or [])
        if (
            (
                not normalized_datasource_filter
                or normalized_datasource_filter
                in {
                    str(record.get("datasource") or "").strip(),
                    str(record.get("datasourceUid") or "").strip(),
                    str(record.get("datasourceType") or "").strip(),
                    str(record.get("datasourceFamily") or "").strip(),
                }
            )
            and (
                not normalized_panel_id_filter
                or str(record.get("panelId") or "") == normalized_panel_id_filter
            )
        )
    ]
    return {
        "summary": {
            "dashboardCount": len(
                set(str(record.get("dashboardUid") or "") for record in filtered_records)
            ),
            "queryRecordCount": len(filtered_records),
        },
        "queries": filtered_records,
    }


def build_grouped_export_inspection_report_document(
    document: dict[str, Any]
) -> dict[str, Any]:
    """Normalize one flat inspection report into dashboard-first grouped form."""
    # Call graph: see callers/callees.
    #   Upstream callers: 無
    #   Downstream callees: 無

    query_records = list(document.get("queries") or [])
    dashboards: OrderedDict[tuple[str, str, str], dict[str, Any]] = OrderedDict()

    for record in query_records:
        dashboard_key = (
            str(record.get("folderPath") or DEFAULT_FOLDER_TITLE),
            str(record.get("dashboardTitle") or DEFAULT_DASHBOARD_TITLE),
            str(record.get("dashboardUid") or DEFAULT_UNKNOWN_UID),
        )
        dashboard_entry = dashboards.get(dashboard_key)
        if dashboard_entry is None:
            dashboard_entry = {
                "dashboardUid": dashboard_key[2],
                "dashboardTitle": dashboard_key[1],
                "folderPath": dashboard_key[0],
                "folderUid": str(record.get("folderUid") or ""),
                "parentFolderUid": str(record.get("parentFolderUid") or ""),
                "file": str(record.get("file") or ""),
                "queryCount": 0,
                "panels": OrderedDict(),
            }
            dashboards[dashboard_key] = dashboard_entry
        dashboard_entry["queryCount"] = int(dashboard_entry.get("queryCount") or 0) + 1

        panel_key = (
            str(record.get("panelId") or ""),
            str(record.get("panelTitle") or ""),
            str(record.get("panelType") or ""),
        )
        panel_entry = dashboard_entry["panels"].get(panel_key)
        if panel_entry is None:
            panel_entry = {
                "panelId": panel_key[0],
                "panelTitle": panel_key[1],
                "panelType": panel_key[2],
                "panelTargetCount": 0,
                "panelQueryCount": 0,
                "datasources": [],
                "queryCount": 0,
                "queries": [],
            }
            dashboard_entry["panels"][panel_key] = panel_entry
        panel_entry["panelTargetCount"] = max(
            int(panel_entry.get("panelTargetCount") or 0),
            int(record.get("panelTargetCount") or 0),
        )
        datasource_label = str(record.get("datasource") or "")
        if datasource_label and datasource_label not in panel_entry["datasources"]:
            panel_entry["datasources"].append(datasource_label)
        panel_entry["queryCount"] = max(
            int(panel_entry.get("queryCount") or 0),
            int(record.get("panelQueryCount") or 0),
        )
        panel_entry["panelQueryCount"] = max(
            int(panel_entry.get("panelQueryCount") or 0),
            int(record.get("panelQueryCount") or 0),
        )
        panel_entry["queries"].append(dict(record))

    dashboard_records = []
    panel_count = 0
    for dashboard_entry in dashboards.values():
        panels = []
        for panel_entry in dashboard_entry["panels"].values():
            panel_entry["datasources"].sort()
            panels.append(panel_entry)
        panel_count += len(panels)
        dashboard_records.append(
            {
                "dashboardUid": dashboard_entry["dashboardUid"],
                "dashboardTitle": dashboard_entry["dashboardTitle"],
                "folderPath": dashboard_entry["folderPath"],
                "folderUid": dashboard_entry["folderUid"],
                "parentFolderUid": dashboard_entry["parentFolderUid"],
                "file": dashboard_entry["file"],
                "panelCount": len(panels),
                "queryCount": int(dashboard_entry.get("queryCount") or 0),
                "panels": panels,
            }
        )

    return {
        "summary": {
            "dashboardCount": len(dashboard_records),
            "panelCount": panel_count,
            "queryRecordCount": len(query_records),
        },
        "dashboards": dashboard_records,
    }


from .inspection_render import (  # noqa: E402
    format_report_column_value,
    render_export_inspection_grouped_report,
    render_export_inspection_report_csv,
    render_export_inspection_report_tables,
    render_export_inspection_table_section,
    render_export_inspection_tree_tables,
)
