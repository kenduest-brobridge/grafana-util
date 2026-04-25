"""Dashboard inspection workflow orchestration helpers."""

import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import json
import shutil
import tempfile
from pathlib import Path

from ..profile_config import resolve_input_lane_path
from .inspection_dispatch import (
    resolve_inspect_dispatch_args,
    run_inspection_dispatch,
)


def _load_json_array_file(path, error_cls, error_context):
    """Load one JSON array file for merged multi-org inspection."""
    if not path.is_file():
        return []
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except OSError as exc:
        raise error_cls("Failed to read %s %s: %s" % (error_context, path, exc)) from exc
    except json.JSONDecodeError as exc:
        raise error_cls("Invalid JSON in %s %s: %s" % (error_context, path, exc)) from exc
    if not isinstance(raw, list):
        raise error_cls("%s must be a JSON array: %s" % (error_context, path))
    return raw


def _merge_multi_org_export_root(import_dir, temp_root, deps):
    """Materialize one merged raw inspect directory from a multi-org export root."""
    inspect_raw_dir = temp_root / "inspect-export-all-orgs" / deps["RAW_EXPORT_SUBDIR"]
    merged_index = []
    merged_folders = []
    merged_datasources = []
    dashboard_count = 0

    for org_raw_dir in deps["discover_org_raw_export_dirs"](import_dir):
        org_name = org_raw_dir.parent.name
        shutil.copytree(org_raw_dir, inspect_raw_dir / org_name, dirs_exist_ok=True)
        metadata = deps["load_export_metadata"](
            org_raw_dir, expected_variant=deps["RAW_EXPORT_SUBDIR"]
        )
        index_file = str((metadata or {}).get("indexFile") or "index.json")
        index_records = _load_json_array_file(
            org_raw_dir / index_file,
            deps["GrafanaError"],
            "Dashboard export index",
        )
        for item in index_records:
            if isinstance(item, dict):
                merged = dict(item)
                merged["path"] = "%s/%s" % (org_name, str(item.get("path") or ""))
                merged_index.append(merged)
        merged_folders.extend(
            _load_json_array_file(
                org_raw_dir / deps["FOLDER_INVENTORY_FILENAME"],
                deps["GrafanaError"],
                "Dashboard folder inventory",
            )
        )
        merged_datasources.extend(
            _load_json_array_file(
                org_raw_dir / deps["DATASOURCE_INVENTORY_FILENAME"],
                deps["GrafanaError"],
                "Dashboard datasource inventory",
            )
        )
        dashboard_count += int(
            (metadata or {}).get("dashboardCount") or len(index_records)
        )

    deps["write_json_document"](
        deps["build_export_metadata"](
            variant=deps["RAW_EXPORT_SUBDIR"],
            dashboard_count=dashboard_count,
            format_name="grafana-web-import-preserve-uid",
            folders_file=deps["FOLDER_INVENTORY_FILENAME"],
            datasources_file=deps["DATASOURCE_INVENTORY_FILENAME"],
        ),
        inspect_raw_dir / deps["EXPORT_METADATA_FILENAME"],
    )
    deps["write_json_document"](merged_index, inspect_raw_dir / "index.json")
    deps["write_json_document"](
        merged_folders, inspect_raw_dir / deps["FOLDER_INVENTORY_FILENAME"]
    )
    deps["write_json_document"](
        merged_datasources, inspect_raw_dir / deps["DATASOURCE_INVENTORY_FILENAME"]
    )
    (inspect_raw_dir / ".inspect-source-root").write_text(
        str(import_dir),
        encoding="utf-8",
    )
    return inspect_raw_dir


def _bounded_concurrency(value):
    try:
        requested = int(value)
    except (TypeError, ValueError):
        requested = 8
    return max(1, min(requested, 16))


def _print_live_progress(args, index, total, uid, deps):
    if not bool(getattr(args, "progress", False)):
        return
    writer = deps.get("output_writer") or print
    writer("Fetching dashboard %s/%s: %s" % (index, total, uid))


def _fetch_live_dashboard_records(client, summaries, raw_dir, args, deps):
    records = []

    def fetch_one(index, summary):
        uid = str(summary.get("uid") or "").strip()
        if not uid:
            return None
        payload = client.fetch_dashboard(uid)
        document = deps["build_preserved_web_import_document"](payload)
        output_path = deps["build_output_path"](raw_dir, summary, flat=False)
        deps["write_dashboard"](document, output_path, overwrite=True)
        item = deps["build_dashboard_index_item"](summary, uid)
        item["raw_path"] = str(output_path)
        return index, uid, item

    tasks = [(index, summary) for index, summary in enumerate(summaries, start=1)]
    concurrency = _bounded_concurrency(getattr(args, "concurrency", 8))
    if concurrency == 1 or len(tasks) <= 1:
        for index, summary in tasks:
            result = fetch_one(index, summary)
            if result is None:
                continue
            _, uid, item = result
            _print_live_progress(args, index, len(tasks), uid, deps)
            records.append((index, item))
        return [item for _, item in records]

    with ThreadPoolExecutor(max_workers=concurrency) as executor:
        future_to_index = {
            executor.submit(fetch_one, index, summary): index
            for index, summary in tasks
        }
        for future in as_completed(future_to_index):
            result = future.result()
            if result is None:
                continue
            index, uid, item = result
            _print_live_progress(args, index, len(tasks), uid, deps)
            records.append((index, item))
    return [item for _, item in sorted(records, key=lambda record: record[0])]


def materialize_live_inspection_export(client, page_size, raw_dir, deps, args=None):
    """Write one temporary raw-export-like directory for live dashboard inspection."""
    args = args or argparse.Namespace()
    raw_dir.mkdir(parents=True, exist_ok=True)
    summaries = deps["attach_dashboard_org"](
        client, client.iter_dashboard_summaries(page_size)
    )
    org = client.fetch_current_org()
    folder_inventory = deps["collect_folder_inventory"](client, org, summaries)
    datasource_inventory = [
        deps["build_datasource_inventory_record"](item, org)
        for item in client.list_datasources()
    ]
    index_items = _fetch_live_dashboard_records(client, summaries, raw_dir, args, deps)

    raw_index = deps["build_variant_index"](
        index_items,
        "raw_path",
        "grafana-web-import-preserve-uid",
    )
    raw_metadata = deps["build_export_metadata"](
        variant=deps["RAW_EXPORT_SUBDIR"],
        dashboard_count=len(raw_index),
        format_name="grafana-web-import-preserve-uid",
        folders_file=deps["FOLDER_INVENTORY_FILENAME"],
        datasources_file=deps["DATASOURCE_INVENTORY_FILENAME"],
    )
    deps["write_json_document"](raw_index, raw_dir / "index.json")
    deps["write_json_document"](
        raw_metadata, raw_dir / deps["EXPORT_METADATA_FILENAME"]
    )
    deps["write_json_document"](
        folder_inventory, raw_dir / deps["FOLDER_INVENTORY_FILENAME"]
    )
    deps["write_json_document"](
        datasource_inventory, raw_dir / deps["DATASOURCE_INVENTORY_FILENAME"]
    )
    return raw_dir


def _build_live_inspection_clients(client, args, deps):
    if not (getattr(args, "all_orgs", False) or getattr(args, "org_id", None)):
        return [client]
    auth_header = str(getattr(client, "headers", {}).get("Authorization", ""))
    if not auth_header.startswith("Basic "):
        raise deps["GrafanaError"](
            "Dashboard org switching does not support API token auth. Use Grafana username/password login with --basic-user and --basic-password."
        )
    if getattr(args, "all_orgs", False):
        return [
            client.with_org_id(str(org.get("id")))
            for org in client.list_orgs()
            if str(org.get("id") or "").strip()
        ]
    return [client.with_org_id(str(getattr(args, "org_id")))]


def run_inspect_live(args, deps):
    """Inspect live Grafana dashboards by reusing the raw-export inspection pipeline."""
    # Call graph: see callers/callees.
    #   Upstream callers: 無
    #   Downstream callees: 13, 87

    client = deps["build_client"](args)
    with tempfile.TemporaryDirectory(prefix="grafana-utils-inspect-live-") as tmpdir:
        temp_root = Path(tmpdir)
        clients = _build_live_inspection_clients(client, args, deps)
        if getattr(args, "all_orgs", False):
            for scoped_client in clients:
                org = scoped_client.fetch_current_org()
                org_id = str(org.get("id") or org.get("orgId") or "unknown").strip()
                org_name = str(org.get("name") or "unknown").replace(" ", "_")
                materialize_live_inspection_export(
                    scoped_client,
                    page_size=int(args.page_size),
                    raw_dir=temp_root / ("org_%s_%s" % (org_id, org_name)) / deps["RAW_EXPORT_SUBDIR"],
                    deps=deps,
                    args=args,
                )
            raw_dir = _merge_multi_org_export_root(temp_root, temp_root / "merged", deps)
        else:
            raw_dir = materialize_live_inspection_export(
                clients[0],
                page_size=int(args.page_size),
                raw_dir=temp_root / deps["RAW_EXPORT_SUBDIR"],
                deps=deps,
                args=args,
            )
        inspect_args = argparse.Namespace(
            import_dir=str(raw_dir),
            input_type=None,
            input_format="raw",
            report=getattr(args, "report", None),
            output_format=getattr(args, "output_format", None),
            output_file=getattr(args, "output_file", None),
            also_stdout=bool(getattr(args, "also_stdout", False)),
            report_columns=getattr(args, "report_columns", None),
            report_filter_datasource=getattr(args, "report_filter_datasource", None),
            report_filter_panel_id=getattr(args, "report_filter_panel_id", None),
            json=bool(getattr(args, "json", False)),
            table=bool(getattr(args, "table", False)),
            no_header=bool(getattr(args, "no_header", False)),
            interactive=bool(getattr(args, "interactive", False)),
        )
        return run_inspect_export(inspect_args, deps)


def _load_dashboard_json(path, error_cls):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except OSError as exc:
        raise error_cls("Failed to read dashboard file %s: %s" % (path, exc)) from exc
    except json.JSONDecodeError as exc:
        raise error_cls("Invalid dashboard JSON in %s: %s" % (path, exc)) from exc


def _discover_plain_dashboard_files(input_dir, deps):
    ignored = {
        "index.json",
        deps["EXPORT_METADATA_FILENAME"],
        deps["FOLDER_INVENTORY_FILENAME"],
        deps["DATASOURCE_INVENTORY_FILENAME"],
    }
    return sorted(path for path in input_dir.rglob("*.json") if path.name not in ignored)


def _resolve_git_sync_dashboard_root(input_dir):
    if input_dir.name == "git-sync" and (input_dir / "raw").is_dir():
        return input_dir / "raw"
    for candidate in (
        input_dir / "dashboards" / "git-sync" / "raw",
        input_dir / "git-sync" / "raw",
        input_dir / "raw",
    ):
        if candidate.is_dir():
            return candidate
    return input_dir


def _stage_plain_dashboard_tree(input_dir, temp_root, deps):
    raw_dir = temp_root / "inspect-plain-input" / deps["RAW_EXPORT_SUBDIR"]
    raw_dir.mkdir(parents=True, exist_ok=True)
    index_items = []
    for source_file in _discover_plain_dashboard_files(input_dir, deps):
        document = _load_dashboard_json(source_file, deps["GrafanaError"])
        dashboard = document.get("dashboard") if isinstance(document, dict) else None
        if not isinstance(dashboard, dict):
            dashboard = document
            document = {"dashboard": dashboard, "meta": {}}
        if not isinstance(dashboard, dict):
            continue
        uid = str(dashboard.get("uid") or source_file.stem).strip()
        title = str(dashboard.get("title") or source_file.stem).strip()
        relative_path = source_file.relative_to(input_dir)
        target_path = raw_dir / relative_path
        deps["write_json_document"](document, target_path)
        index_items.append(
            {
                "uid": uid,
                "title": title,
                "folder": str(Path(*relative_path.parts[:-1])) if len(relative_path.parts) > 1 else "General",
                "path": str(relative_path),
                "raw_path": str(target_path),
                "format": "grafana-web-import-preserve-uid",
            }
        )
    if not index_items:
        raise deps["GrafanaError"]("No dashboard JSON files found in %s" % input_dir)
    deps["write_json_document"](index_items, raw_dir / "index.json")
    deps["write_json_document"]([], raw_dir / deps["FOLDER_INVENTORY_FILENAME"])
    deps["write_json_document"]([], raw_dir / deps["DATASOURCE_INVENTORY_FILENAME"])
    deps["write_json_document"](
        deps["build_export_metadata"](
            variant=deps["RAW_EXPORT_SUBDIR"],
            dashboard_count=len(index_items),
            format_name="grafana-web-import-preserve-uid",
            folders_file=deps["FOLDER_INVENTORY_FILENAME"],
            datasources_file=deps["DATASOURCE_INVENTORY_FILENAME"],
        ),
        raw_dir / deps["EXPORT_METADATA_FILENAME"],
    )
    (raw_dir / ".inspect-source-root").write_text(str(input_dir), encoding="utf-8")
    return raw_dir


def _resolve_inspect_import_dir(import_dir, settings, temp_root, deps):
    input_format = settings["input_format"]
    input_type = settings["input_type"]
    if input_format == "git-sync":
        return _stage_plain_dashboard_tree(
            _resolve_git_sync_dashboard_root(import_dir),
            temp_root,
            deps,
        )
    if input_format == "provisioning":
        provisioning_root = (
            import_dir / "dashboards"
            if import_dir.name == "provisioning" and (import_dir / "dashboards").is_dir()
            else import_dir
        )
        return _stage_plain_dashboard_tree(provisioning_root, temp_root, deps)
    if input_type == "raw" and (import_dir / deps["RAW_EXPORT_SUBDIR"]).is_dir():
        return import_dir / deps["RAW_EXPORT_SUBDIR"]
    if input_type is None and (import_dir / deps["RAW_EXPORT_SUBDIR"]).is_dir():
        return import_dir / deps["RAW_EXPORT_SUBDIR"]
    if input_type == "source":
        for name in ("prompt", "source"):
            candidate = import_dir / name
            if candidate.is_dir():
                return candidate
    return import_dir


def _resolve_local_artifact_import_dir(args, deps):
    """Resolve dashboard summary input from the repo-local artifact workspace."""
    if getattr(args, "import_dir", None) and getattr(args, "local", False):
        raise deps["GrafanaError"]("--input-dir and --local cannot be combined.")
    if getattr(args, "import_dir", None):
        return Path(args.import_dir)
    if not (
        getattr(args, "local", False)
        or getattr(args, "run", None)
        or getattr(args, "run_id", None)
    ):
        return None
    try:
        return resolve_input_lane_path(
            "dashboards/raw",
            run=getattr(args, "run", None),
            run_id=getattr(args, "run_id", None),
            profile_name=getattr(args, "profile", None),
            config_path=getattr(args, "config", None),
        )
    except ValueError as exc:
        raise deps["GrafanaError"](str(exc)) from exc


def run_inspect_export(args, deps):
    """Inspect one raw export directory and summarize dashboards, folders, and datasources."""
    # Call graph: see callers/callees.
    #   Upstream callers: 63
    #   Downstream callees: 無

    settings = resolve_inspect_dispatch_args(
        args,
        deps,
        deps["GrafanaError"],
    )
    import_dir = _resolve_local_artifact_import_dir(args, deps)
    if import_dir is None:
        raise deps["GrafanaError"]("Dashboard summary requires --input-dir or --local for local inspection.")
    with tempfile.TemporaryDirectory(prefix="grafana-utils-inspect-export-") as tmpdir:
        resolved_import_dir = _resolve_inspect_import_dir(
            import_dir,
            settings,
            Path(tmpdir),
            deps,
        )
        if bool(getattr(args, "interactive", False)):
            raise deps["GrafanaError"](
                "Dashboard summary interactive mode is not implemented in Python yet."
            )
        metadata = deps["load_export_metadata"](resolved_import_dir, expected_variant=None)
        if isinstance(metadata, dict) and str(metadata.get("variant") or "") == "root":
            merged_raw_dir = _merge_multi_org_export_root(
                resolved_import_dir,
                Path(tmpdir),
                deps,
            )
            return run_inspection_dispatch(merged_raw_dir, deps, settings)
        return run_inspection_dispatch(resolved_import_dir, deps, settings)
