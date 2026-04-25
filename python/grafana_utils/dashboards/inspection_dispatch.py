"""Shared inspect output validation and rendering helpers."""

import csv
import json
import io
import sys
from pathlib import Path
from typing import Any

from .inspection_dependency_models import build_dependency_rows_from_query_report


INSPECT_OUTPUT_FORMAT_TO_MODE = {
    "text": (None, False, False),
    "table": (None, False, True),
    "csv": (None, False, False),
    "json": (None, True, False),
    "yaml": (None, False, False),
    "report-table": ("table", False, False),
    "report-csv": ("csv", False, False),
    "report-json": ("json", False, False),
    "report-tree": ("tree", False, False),
    "report-tree-table": ("tree-table", False, False),
    "report-dependency": ("dependency", False, False),
    "report-dependency-json": ("dependency-json", False, False),
    "dependency": ("dependency", False, False),
    "dependency-json": ("dependency-json", False, False),
    "governance": ("governance", False, False),
    "governance-json": ("governance-json", False, False),
    "queries-json": ("queries-json", False, False),
}

REPORT_FORMATS_WITH_COLUMN_SUPPORT = ("table", "csv", "tree-table")
GOVERNANCE_REPORT_FORMATS = ("governance", "governance-json")
SUMMARY_OUTPUT_FORMATS_WITH_COLUMNS = ("table", "csv", "tree-table", "queries-json")


def resolve_inspect_output_mode(args, grafana_error):
    """Normalize legacy inspect output flags and the newer --output-format selector."""
    output_format = getattr(args, "output_format", None)
    report_format = getattr(args, "report", None)
    json_output = bool(getattr(args, "json", False))
    table_output = bool(getattr(args, "table", False))

    if output_format and (report_format or json_output or table_output):
        raise grafana_error(
            "--output-format cannot be combined with --json, --table, or --report."
        )
    if output_format:
        return INSPECT_OUTPUT_FORMAT_TO_MODE[output_format]
    return report_format, json_output, table_output


def resolve_inspect_dispatch_args(args, deps, grafana_error):
    """Validate inspect output arguments and normalize dispatch settings."""
    # Call graph: see callers/callees.
    #   Upstream callers: 無
    #   Downstream callees: 30

    report_format, json_output, table_output = resolve_inspect_output_mode(
        args, grafana_error
    )
    if report_format and (table_output or json_output):
        raise grafana_error("--report cannot be combined with --table or --json.")
    if table_output and json_output:
        raise grafana_error(
            "--table and --json are mutually exclusive for summary."
        )

    report_columns = deps["parse_report_columns"](
        getattr(args, "report_columns", None)
    )
    report_filter_datasource = getattr(args, "report_filter_datasource", None)
    report_filter_panel_id = getattr(args, "report_filter_panel_id", None)
    no_header = bool(getattr(args, "no_header", False))
    requested_output_format = getattr(args, "output_format", None)
    output_format = (
        requested_output_format
        if requested_output_format
        else "table" if table_output else "json" if json_output else "text"
    )
    input_type = getattr(args, "input_type", None)
    input_format = getattr(args, "input_format", "raw")
    if input_type not in (None, "raw", "source"):
        raise grafana_error("--input-type must be raw or source.")
    if input_format not in ("raw", "provisioning", "git-sync"):
        raise grafana_error("--input-format must be raw, provisioning, or git-sync.")

    if report_columns is not None and report_format is None:
        raise grafana_error(
            "--report-columns is only supported together with table, csv, tree-table, or queries-json output."
        )
    if report_filter_datasource and report_format is None:
        raise grafana_error(
            "--report-filter-datasource is only supported together with table, csv, tree-table, dependency, dependency-json, governance, governance-json, or queries-json output."
        )
    if report_filter_panel_id and report_format is None:
        raise grafana_error(
            "--report-filter-panel-id is only supported together with table, csv, tree-table, dependency, dependency-json, governance, governance-json, or queries-json output."
        )
    if report_columns is not None and report_format in GOVERNANCE_REPORT_FORMATS:
        raise grafana_error(
            "--report-columns is not supported with governance output."
        )
    if report_columns is not None and report_format not in REPORT_FORMATS_WITH_COLUMN_SUPPORT:
        raise grafana_error(
            "--report-columns is only supported with table, csv, or tree-table output."
        )
    if no_header and not (
        table_output
        or report_format in REPORT_FORMATS_WITH_COLUMN_SUPPORT
        or output_format in SUMMARY_OUTPUT_FORMATS_WITH_COLUMNS
    ):
        raise grafana_error(
            "--no-header is only supported with --table, table, csv, or tree-table report output."
        )

    return {
        "json_output": json_output,
        "output_format": output_format,
        "no_header": no_header,
        "report_columns": report_columns,
        "report_filter_datasource": report_filter_datasource,
        "report_filter_panel_id": report_filter_panel_id,
        "report_format": report_format,
        "table_output": table_output,
        "output_file": getattr(args, "output_file", None),
        "also_stdout": bool(getattr(args, "also_stdout", False)),
        "input_type": input_type,
        "input_format": input_format,
    }


def _write_output_to_file(output, output_file):
    if not output_file:
        return
    output_path = Path(output_file)
    if output_path.parent:
        output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(output, encoding="utf-8")


def build_filtered_report_document(import_dir, deps, settings):
    """Build one filtered flat query report document for inspect output."""
    return deps["filter_export_inspection_report_document"](
        deps["build_export_inspection_report_document"](import_dir),
        datasource_label=settings["report_filter_datasource"],
        panel_id=settings["report_filter_panel_id"],
    )


def _render_report_output(import_dir, deps, settings):
    """Build inspect report output without printing."""
    report_format = settings["report_format"]
    report_columns = settings["report_columns"]
    include_header = not settings["no_header"]

    if report_format == "json":
        return (
            json.dumps(
                build_filtered_report_document(import_dir, deps, settings),
                indent=2,
                sort_keys=False,
                ensure_ascii=False,
            )
            + "\n"
        )

    if report_format == "queries-json":
        return (
            json.dumps(
                build_filtered_report_document(import_dir, deps, settings),
                indent=2,
                sort_keys=False,
                ensure_ascii=False,
            )
            + "\n"
        )

    if report_format == "table":
        lines = deps["render_export_inspection_report_tables"](
            build_filtered_report_document(import_dir, deps, settings),
            import_dir,
            include_header=include_header,
            selected_columns=report_columns,
        )
        return "\n".join(lines) + "\n"

    if report_format == "csv":
        return deps["render_export_inspection_report_csv"](
            build_filtered_report_document(import_dir, deps, settings),
            selected_columns=report_columns,
            include_header=include_header,
        )

    if report_format in ("tree", "tree-table"):
        grouped_document = deps["build_grouped_export_inspection_report_document"](
            build_filtered_report_document(import_dir, deps, settings)
        )
        if report_format == "tree":
            lines = deps["render_export_inspection_grouped_report"](
                grouped_document,
                import_dir,
            )
            return "\n".join(lines) + "\n"
        lines = deps["render_export_inspection_tree_tables"](
            grouped_document,
            import_dir,
            include_header=include_header,
            selected_columns=report_columns,
        )
        return "\n".join(lines) + "\n"

    if report_format in ("dependency", "dependency-json"):
        report_document = build_filtered_report_document(import_dir, deps, settings)
        inventory = deps["load_datasource_inventory"](
            import_dir,
            deps["load_export_metadata"](import_dir, deps["RAW_EXPORT_SUBDIR"]),
        )
        dependency_report = build_dependency_rows_from_query_report(
            report_document["queries"],
            inventory,
        ).to_dict()
        payload = {"kind": "grafana-utils-dashboard-dependency-contract"}
        payload.update(dependency_report)
        payload.update(dependency_report.get("summary", {}))
        return (
            json.dumps(payload, indent=2, sort_keys=False, ensure_ascii=False)
            + "\n"
        )

    report_document = build_filtered_report_document(import_dir, deps, settings)
    governance_document = deps["build_export_inspection_governance_document"](
        deps["build_export_inspection_document"](import_dir),
        report_document,
    )
    if report_format == "governance-json":
        return (
            json.dumps(
                governance_document,
                indent=2,
                sort_keys=False,
                ensure_ascii=False,
            )
            + "\n"
        )
    return "\n".join(
        deps["render_export_inspection_governance_tables"](
            governance_document,
            import_dir,
        )
    ) + "\n"


def _build_summary_output_rows(document: dict[str, Any]) -> list[list[str]]:
    """Build canonical summary rows as [name, value] tuples."""

    summary = document.get("summary") or {}
    summary_rows = [
        ["export_org", str(summary.get("exportOrg") or summary.get("export_org") or "")],
        ["export_org_id", str(summary.get("exportOrgId") or summary.get("export_org_id") or "")],
        ["dashboard_count", str(int(summary.get("dashboardCount") or summary.get("dashboard_count") or 0))],
        ["folder_count", str(int(summary.get("folderCount") or summary.get("folder_count") or 0))],
        ["panel_count", str(int(summary.get("panelCount") or summary.get("panel_count") or 0))],
        ["query_count", str(int(summary.get("queryCount") or summary.get("query_count") or 0))],
        ["datasource_inventory_count", str(int(summary.get("datasourceInventoryCount") or summary.get("datasource_inventory_count") or 0))],
        ["orphaned_datasource_count", str(int(summary.get("orphanedDatasourceCount") or summary.get("orphaned_datasource_count") or 0))],
        [
            "mixed_datasource_dashboard_count",
            str(int(summary.get("mixedDatasourceDashboardCount") or summary.get("mixed_datasource_dashboard_count") or 0)),
        ],
    ]
    return summary_rows


def _render_simple_yaml(value: Any, indent: int = 0) -> str:
    prefix = " " * indent
    if isinstance(value, dict):
        lines = []
        for key, item in value.items():
            if isinstance(item, (dict, list)):
                lines.append("%s%s:" % (prefix, key))
                lines.append(_render_simple_yaml(item, indent + 2))
            else:
                lines.append("%s%s: %s" % (prefix, key, _format_yaml_scalar(item)))
        return "\n".join(lines)
    if isinstance(value, list):
        if not value:
            return "%s[]" % prefix
        lines = []
        for item in value:
            if isinstance(item, (dict, list)):
                lines.append("%s-" % prefix)
                lines.append(_render_simple_yaml(item, indent + 2))
            else:
                lines.append("%s- %s" % (prefix, _format_yaml_scalar(item)))
        return "\n".join(lines)
    return "%s%s" % (prefix, _format_yaml_scalar(value))


def _format_yaml_scalar(value: Any) -> str:
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float)):
        return str(value)
    text = str(value)
    if text == "":
        return '""'
    return json.dumps(text, ensure_ascii=False)


def _render_summary_output(import_dir, deps, settings):
    """Build inspect summary output without printing."""
    document = deps["build_export_inspection_document"](import_dir)
    if settings["json_output"]:
        return (
            json.dumps(document, indent=2, sort_keys=False, ensure_ascii=False)
            + "\n"
        )
    if settings["output_format"] == "csv":
        output = io.StringIO()
        writer = csv.writer(output)
        writer.writerow(["NAME", "VALUE"])
        writer.writerows(_build_summary_output_rows(document))
        return output.getvalue()
    if settings["output_format"] == "yaml":
        return _render_simple_yaml(document) + "\n"
    if settings["table_output"]:
        return "\n".join(
            deps["render_export_inspection_tables"](
                document,
                import_dir,
                include_header=not settings["no_header"],
            )
        ) + "\n"
    return "\n".join(deps["render_export_inspection_summary"](document, import_dir)) + "\n"


def run_inspection_dispatch(import_dir, deps, settings):
    """Render inspect output using the normalized dispatch settings."""
    # Call graph: see callers/callees.
    #   Upstream callers: 無
    #   Downstream callees: 112, 215

    if settings["report_format"] is not None:
        output = _render_report_output(import_dir, deps, settings)
    else:
        output = _render_summary_output(import_dir, deps, settings)
    _write_output_to_file(output, settings["output_file"])
    if output and (not settings["output_file"] or settings["also_stdout"]):
        sys.stdout.write(output)
    return 0
