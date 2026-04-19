#!/usr/bin/env python3
"""Report overlap and gaps between schema manifests and runtime output contracts."""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Optional, Union


REPO_ROOT = Path(__file__).resolve().parents[1]
MANIFESTS_DIR = REPO_ROOT / "schemas" / "manifests"
RUNTIME_REGISTRY_PATH = REPO_ROOT / "scripts" / "contracts" / "output-contracts.json"
COMMAND_SURFACE_PATH = REPO_ROOT / "scripts" / "contracts" / "command-surface.json"
DOCS_ENTRYPOINTS_PATH = REPO_ROOT / "scripts" / "contracts" / "docs-entrypoints.json"
GENERATED_DOCS_DIR = REPO_ROOT / "docs" / "commands"


@dataclass(frozen=True)
class ManifestContract:
    family: str
    contract_id: str
    golden_sample: str
    title: str
    kind: Optional[str]
    schema_version: Optional[str]
    aliases: tuple[str, ...]


@dataclass(frozen=True)
class RuntimeContract:
    family: str
    contract_id: str
    fixture: str
    kind: Optional[str]
    schema_version: Optional[str]
    aliases: tuple[str, ...]


@dataclass(frozen=True)
class FamilyReport:
    family: str
    contract_count: int
    shared: tuple[str, ...]
    left_only: tuple[str, ...]
    right_families: tuple[str, ...]


@dataclass(frozen=True)
class PromotionReport:
    manifest_families: tuple[FamilyReport, ...]
    runtime_families: tuple[FamilyReport, ...]
    shared_contract_ids: tuple[str, ...]
    manifest_only_contract_ids: tuple[str, ...]
    runtime_only_contract_ids: tuple[str, ...]
    evidence_matrix: tuple["EvidenceRow", ...]
    structural_issues: tuple["StructuralIssue", ...]


@dataclass(frozen=True)
class ManifestRoute:
    family: str
    route_id: str
    commands: tuple[str, ...]
    contract_ids: tuple[str, ...]


@dataclass(frozen=True)
class EvidenceRow:
    contract_id: str
    match_method: str
    runtime_golden: str
    schema_help_manifest: str
    public_cli_route: str
    docs_entrypoint: str
    generated_docs: str
    artifact_workspace_lane: str


@dataclass(frozen=True)
class StructuralIssue:
    severity: str
    contract_id: str
    message: str


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def _optional_str(value: Any) -> Optional[str]:
    if value is None or value == "":
        return None
    return str(value)


def _aliases(entry: dict[str, Any]) -> tuple[str, ...]:
    raw_aliases = entry.get("aliases")
    if raw_aliases is None:
        return ()
    if not isinstance(raw_aliases, list):
        raise TypeError("aliases must be a list when present")
    return tuple(str(alias) for alias in raw_aliases if str(alias))


def match_keys(contract_id: str, kind: Optional[str], schema_version: Optional[str], aliases: tuple[str, ...]) -> tuple[str, ...]:
    keys: list[str] = []
    if kind and schema_version:
        keys.append(f"kind:{kind}@{schema_version}")
    keys.append(f"id:{contract_id}")
    for alias in aliases:
        keys.append(f"alias:{alias}")
    return tuple(keys)


def load_manifest_contracts(manifests_dir: Path = MANIFESTS_DIR) -> list[ManifestContract]:
    contracts: list[ManifestContract] = []
    seen_contract_ids: set[str] = set()
    for family_dir in sorted(path for path in manifests_dir.iterdir() if path.is_dir()):
        contract_path = family_dir / "contracts.json"
        if not contract_path.is_file():
            continue
        payload = load_json(contract_path)
        if not isinstance(payload, list):
            raise TypeError(f"{contract_path} must contain a list")
        for entry in payload:
            if not isinstance(entry, dict):
                raise TypeError(f"{contract_path} contains a non-object contract entry")
            contract_id = str(entry.get("contractId") or "")
            golden_sample = str(entry.get("goldenSample") or "")
            title = str(entry.get("title") or "")
            if not contract_id:
                raise TypeError(f"{contract_path} contains a contract without contractId")
            if not golden_sample:
                raise TypeError(f"{contract_path} contains contract {contract_id!r} without goldenSample")
            if not title:
                raise TypeError(f"{contract_path} contains contract {contract_id!r} without title")
            if contract_id in seen_contract_ids:
                raise TypeError(f"{contract_path} duplicates contractId {contract_id!r}")
            seen_contract_ids.add(contract_id)
            contracts.append(
                ManifestContract(
                    family=family_dir.name,
                    contract_id=contract_id,
                    golden_sample=golden_sample,
                    title=title,
                    kind=_optional_str(entry.get("kind")),
                    schema_version=_optional_str(entry.get("schemaVersion")),
                    aliases=_aliases(entry),
                )
            )
    if not contracts:
        raise SystemExit("no schema manifests were found")
    return contracts


def runtime_family(name: str) -> str:
    return name.split("-", 1)[0] if name else ""


def load_runtime_contracts(registry_path: Path = RUNTIME_REGISTRY_PATH) -> list[RuntimeContract]:
    payload = load_json(registry_path)
    contracts = payload.get("contracts")
    if not isinstance(contracts, list):
        raise TypeError(f"{registry_path} contracts must be a list")
    records: list[RuntimeContract] = []
    seen_contract_ids: set[str] = set()
    for entry in contracts:
        if not isinstance(entry, dict):
            raise TypeError(f"{registry_path} contains a non-object contract entry")
        name = str(entry.get("name") or "")
        fixture = str(entry.get("fixture") or "")
        if not name:
            raise TypeError(f"{registry_path} contains a contract without name")
        if not fixture:
            raise TypeError(f"{registry_path} contains contract {name!r} without fixture")
        if name in seen_contract_ids:
            raise TypeError(f"{registry_path} duplicates contract name {name!r}")
        seen_contract_ids.add(name)
        records.append(
            RuntimeContract(
                family=runtime_family(name),
                contract_id=name,
                fixture=fixture,
                kind=str(entry.get("kind") or "") or None,
                schema_version=_optional_str(entry.get("schemaVersion")),
                aliases=_aliases(entry),
            )
        )
    if not records:
        raise SystemExit("no runtime output contracts were found")
    return records


Record = Union[ManifestContract, RuntimeContract]


def load_manifest_routes(manifests_dir: Path = MANIFESTS_DIR) -> list[ManifestRoute]:
    routes: list[ManifestRoute] = []
    for family_dir in sorted(path for path in manifests_dir.iterdir() if path.is_dir()):
        routes_path = family_dir / "routes.json"
        if not routes_path.is_file():
            continue
        payload = load_json(routes_path)
        if not isinstance(payload, list):
            raise TypeError(f"{routes_path} must contain a list")
        for entry in payload:
            if not isinstance(entry, dict):
                raise TypeError(f"{routes_path} contains a non-object route entry")
            route_id = str(entry.get("routeId") or "")
            commands = entry.get("commands") or []
            contract_refs = entry.get("contractRefs") or []
            if not isinstance(commands, list):
                raise TypeError(f"{routes_path} route {route_id!r} commands must be a list")
            if not isinstance(contract_refs, list):
                raise TypeError(f"{routes_path} route {route_id!r} contractRefs must be a list")
            contract_ids: list[str] = []
            for contract_ref in contract_refs:
                if not isinstance(contract_ref, dict):
                    raise TypeError(f"{routes_path} route {route_id!r} has a non-object contractRef")
                contract_id = str(contract_ref.get("contractId") or "")
                if contract_id:
                    contract_ids.append(contract_id)
            routes.append(
                ManifestRoute(
                    family=family_dir.name,
                    route_id=route_id,
                    commands=tuple(str(command) for command in commands if str(command)),
                    contract_ids=tuple(contract_ids),
                )
            )
    return routes


def _group_by_family(records: list[Record]) -> dict[str, list[Any]]:
    grouped: dict[str, list[Any]] = {}
    for record in records:
        grouped.setdefault(record.family, []).append(record)
    return grouped


def _index_runtime_contracts(runtime_contracts: list[RuntimeContract]) -> dict[str, tuple[RuntimeContract, str]]:
    indexed: dict[str, tuple[RuntimeContract, str]] = {}
    for record in runtime_contracts:
        for key in match_keys(record.contract_id, record.kind, record.schema_version, record.aliases):
            indexed.setdefault(key, (record, key.split(":", 1)[0]))
    return indexed


def _match_manifest_to_runtime(
    manifest_contracts: list[ManifestContract],
    runtime_contracts: list[RuntimeContract],
) -> dict[str, tuple[RuntimeContract, str]]:
    runtime_index = _index_runtime_contracts(runtime_contracts)
    matches: dict[str, tuple[RuntimeContract, str]] = {}
    for record in manifest_contracts:
        for key in match_keys(record.contract_id, record.kind, record.schema_version, record.aliases):
            runtime_match = runtime_index.get(key)
            if runtime_match:
                matches[record.contract_id] = runtime_match
                break
    return matches


def _commands_by_contract(routes: list[ManifestRoute]) -> dict[str, set[str]]:
    commands_by_contract: dict[str, set[str]] = {}
    for route in routes:
        for contract_id in route.contract_ids:
            commands_by_contract.setdefault(contract_id, set()).update(route.commands)
    return commands_by_contract


def _all_doc_entrypoint_commands(payload: Any) -> set[str]:
    commands: set[str] = set()

    def visit(value: Any) -> None:
        if isinstance(value, dict):
            command = value.get("command")
            if isinstance(command, str) and command:
                commands.add(command)
            for nested in value.values():
                visit(nested)
        elif isinstance(value, list):
            for nested in value:
                visit(nested)

    visit(payload)
    return commands


def _strip_schema_help(command: str) -> str:
    return command.replace(" --help-schema", "").replace(" --help-full", "").strip()


def _command_doc_candidates(command: str) -> tuple[Path, ...]:
    command_path = _strip_schema_help(command)
    if not command_path.startswith("grafana-util "):
        return ()
    doc_name = command_path.removeprefix("grafana-util ").replace(" ", "-") + ".md"
    return (GENERATED_DOCS_DIR / "en" / doc_name, GENERATED_DOCS_DIR / "zh-TW" / doc_name)


def _evidence_value(values: set[str], max_items: int = 3) -> str:
    if not values:
        return "-"
    ordered = sorted(values)
    shown = ", ".join(ordered[:max_items])
    if len(ordered) > max_items:
        shown = f"{shown}, +{len(ordered) - max_items} more"
    return shown


def build_evidence_matrix(
    manifest_contracts: list[ManifestContract],
    runtime_contracts: list[RuntimeContract],
    routes: list[ManifestRoute],
    command_surface_path: Path = COMMAND_SURFACE_PATH,
    docs_entrypoints_path: Path = DOCS_ENTRYPOINTS_PATH,
) -> tuple[tuple[EvidenceRow, ...], tuple[StructuralIssue, ...]]:
    runtime_matches = _match_manifest_to_runtime(manifest_contracts, runtime_contracts)
    commands_by_contract = _commands_by_contract(routes)
    command_surface = load_json(command_surface_path) if command_surface_path.is_file() else {}
    docs_entrypoint_commands = (
        _all_doc_entrypoint_commands(load_json(docs_entrypoints_path))
        if docs_entrypoints_path.is_file()
        else set()
    )

    doc_pages = command_surface.get("doc_pages") if isinstance(command_surface, dict) else {}
    public_route_roots = set(doc_pages) if isinstance(doc_pages, dict) else set()
    artifact_workspace = command_surface.get("artifact_workspace") if isinstance(command_surface, dict) else {}
    workspace_commands: set[str] = set()
    workspace_lanes: dict[str, str] = {}
    if isinstance(artifact_workspace, dict):
        for key in ("export_commands", "local_consumer_commands"):
            values = artifact_workspace.get(key) or []
            if isinstance(values, list):
                workspace_commands.update(str(command) for command in values if str(command))
        lanes = artifact_workspace.get("lanes") or {}
        if isinstance(lanes, dict):
            workspace_lanes = {str(key).replace("_", " "): str(value) for key, value in lanes.items()}

    rows: list[EvidenceRow] = []
    issues: list[StructuralIssue] = []
    for contract in sorted(manifest_contracts, key=lambda item: (item.family, item.contract_id)):
        runtime_match = runtime_matches.get(contract.contract_id)
        runtime = runtime_match[0] if runtime_match else None
        match_method = runtime_match[1] if runtime_match else "unmatched"
        route_commands = commands_by_contract.get(contract.contract_id, set())
        route_command_roots = {_strip_schema_help(command).removeprefix("grafana-util ") for command in route_commands}
        public_routes = {
            command
            for command in route_commands
            if any(
                route == _strip_schema_help(command).removeprefix("grafana-util ")
                or _strip_schema_help(command).removeprefix("grafana-util ").startswith(f"{route} ")
                for route in public_route_roots
            )
        }
        docs_entrypoints = {
            command
            for command in route_command_roots
            if f"grafana-util {command}" in docs_entrypoint_commands
        }
        generated_docs = {
            str(path.relative_to(REPO_ROOT))
            for command in route_commands
            for path in _command_doc_candidates(command)
            if path.is_file()
        }
        workspace_hits = {
            command
            for command in route_command_roots
            if command in workspace_commands
            or any(command == lane or command.startswith(f"{lane} ") for lane in workspace_lanes)
        }
        rows.append(
            EvidenceRow(
                contract_id=contract.contract_id,
                match_method=match_method,
                runtime_golden=runtime.fixture if runtime else "-",
                schema_help_manifest=f"{contract.family}/contracts.json",
                public_cli_route=_evidence_value(public_routes),
                docs_entrypoint=_evidence_value(docs_entrypoints),
                generated_docs=_evidence_value(generated_docs),
                artifact_workspace_lane=_evidence_value(workspace_hits),
            )
        )
        if not runtime:
            issues.append(
                StructuralIssue(
                    severity="info",
                    contract_id=contract.contract_id,
                    message="no matching runtime golden contract found",
                )
            )
        if not route_commands:
            issues.append(
                StructuralIssue(
                    severity="info",
                    contract_id=contract.contract_id,
                    message="schema manifest contract is not referenced by a manifest route",
                )
            )
        if public_routes and not generated_docs:
            issues.append(
                StructuralIssue(
                    severity="info",
                    contract_id=contract.contract_id,
                    message="public manifest route has no generated command-doc file candidate",
                )
            )
    return tuple(rows), tuple(issues)


def build_promotion_report(
    manifest_contracts: list[ManifestContract],
    runtime_contracts: list[RuntimeContract],
    routes: Optional[list[ManifestRoute]] = None,
) -> PromotionReport:
    manifest_by_id: dict[str, ManifestContract] = {}
    runtime_by_id: dict[str, RuntimeContract] = {}
    runtime_matches = _match_manifest_to_runtime(manifest_contracts, runtime_contracts)

    for record in manifest_contracts:
        if record.contract_id:
            manifest_by_id[record.contract_id] = record
    for record in runtime_contracts:
        if record.contract_id:
            runtime_by_id[record.contract_id] = record

    matched_runtime_ids = {runtime.contract_id for runtime, _method in runtime_matches.values()}
    shared_ids = tuple(sorted(runtime_matches))
    manifest_only_ids = tuple(sorted(set(manifest_by_id) - set(runtime_matches)))
    runtime_only_ids = tuple(sorted(set(runtime_by_id) - matched_runtime_ids))

    manifest_family_groups = _group_by_family(manifest_contracts)
    runtime_family_groups = _group_by_family(runtime_contracts)

    shared_by_manifest_family: dict[str, list[str]] = {}
    runtime_families_by_manifest_family: dict[str, set[str]] = {}
    shared_by_runtime_family: dict[str, list[str]] = {}
    manifest_families_by_runtime_family: dict[str, set[str]] = {}

    for contract_id in shared_ids:
        manifest_record = manifest_by_id[contract_id]
        runtime_record = runtime_matches[contract_id][0]
        shared_by_manifest_family.setdefault(manifest_record.family, []).append(contract_id)
        shared_by_runtime_family.setdefault(runtime_record.family, []).append(runtime_record.contract_id)
        runtime_families_by_manifest_family.setdefault(manifest_record.family, set()).add(
            runtime_record.family
        )
        manifest_families_by_runtime_family.setdefault(runtime_record.family, set()).add(
            manifest_record.family
        )

    manifest_reports: list[FamilyReport] = []
    for family in sorted(manifest_family_groups):
        contract_ids = sorted(record.contract_id for record in manifest_family_groups[family])
        shared = tuple(sorted(shared_by_manifest_family.get(family, [])))
        left_only = tuple(contract_id for contract_id in contract_ids if contract_id not in shared)
        manifest_reports.append(
            FamilyReport(
                family=family,
                contract_count=len(contract_ids),
                shared=shared,
                left_only=left_only,
                right_families=tuple(sorted(runtime_families_by_manifest_family.get(family, set()))),
            )
        )

    runtime_reports: list[FamilyReport] = []
    for family in sorted(runtime_family_groups):
        contract_ids = sorted(record.contract_id for record in runtime_family_groups[family])
        shared = tuple(sorted(shared_by_runtime_family.get(family, [])))
        left_only = tuple(contract_id for contract_id in contract_ids if contract_id not in shared)
        runtime_reports.append(
            FamilyReport(
                family=family,
                contract_count=len(contract_ids),
                shared=shared,
                left_only=left_only,
                right_families=tuple(sorted(manifest_families_by_runtime_family.get(family, set()))),
            )
        )

    evidence_matrix, structural_issues = build_evidence_matrix(
        manifest_contracts,
        runtime_contracts,
        routes or [],
    )

    return PromotionReport(
        manifest_families=tuple(manifest_reports),
        runtime_families=tuple(runtime_reports),
        shared_contract_ids=shared_ids,
        manifest_only_contract_ids=manifest_only_ids,
        runtime_only_contract_ids=runtime_only_ids,
        evidence_matrix=evidence_matrix,
        structural_issues=structural_issues,
    )


def render_text_report(
    report: PromotionReport,
    manifest_contracts: list[ManifestContract],
    runtime_contracts: list[RuntimeContract],
    verbose: bool = False,
    show_structure: bool = False,
) -> str:
    manifest_by_id = {record.contract_id: record for record in manifest_contracts}
    runtime_by_id = {record.contract_id: record for record in runtime_contracts}

    lines: list[str] = [
        "contract promotion report",
        f"  manifest families: {len(report.manifest_families)}",
        f"  runtime families: {len(report.runtime_families)}",
        f"  shared contract ids: {len(report.shared_contract_ids)}",
        f"  manifest-only contract ids: {len(report.manifest_only_contract_ids)}",
        f"  runtime-only contract ids: {len(report.runtime_only_contract_ids)}",
        "",
        "manifest families:",
    ]

    for family_report in report.manifest_families:
        overlap = ", ".join(family_report.right_families) if family_report.right_families else "-"
        lines.append(
            f"  {family_report.family}: total={family_report.contract_count} shared={len(family_report.shared)} "
            f"only={len(family_report.left_only)} runtime={overlap}"
        )
        if verbose:
            if family_report.shared:
                lines.append("    shared:")
                for contract_id in family_report.shared:
                    lines.append(
                        f"      - {contract_id} ({manifest_by_id[contract_id].golden_sample})"
                    )
            if family_report.left_only:
                lines.append("    manifest-only:")
                for contract_id in family_report.left_only:
                    lines.append(
                        f"      - {contract_id} ({manifest_by_id[contract_id].golden_sample})"
                    )

    lines.extend(["", "runtime families:"])
    for family_report in report.runtime_families:
        overlap = ", ".join(family_report.right_families) if family_report.right_families else "-"
        lines.append(
            f"  {family_report.family}: total={family_report.contract_count} shared={len(family_report.shared)} "
            f"only={len(family_report.left_only)} manifest={overlap}"
        )
        if verbose:
            if family_report.shared:
                lines.append("    shared:")
                for contract_id in family_report.shared:
                    lines.append(f"      - {contract_id} ({runtime_by_id[contract_id].fixture})")
            if family_report.left_only:
                lines.append("    runtime-only:")
                for contract_id in family_report.left_only:
                    lines.append(f"      - {contract_id} ({runtime_by_id[contract_id].fixture})")

    lines.extend(
        [
            "",
            "evidence matrix:",
            "  columns: runtime_golden | schema/help_manifest | public_cli_route | docs_entrypoint | generated_docs | artifact_workspace_lane",
        ]
    )
    for row in report.evidence_matrix:
        lines.append(
            f"  {row.contract_id} [{row.match_method}]: "
            f"{row.runtime_golden} | {row.schema_help_manifest} | {row.public_cli_route} | "
            f"{row.docs_entrypoint} | {row.generated_docs} | {row.artifact_workspace_lane}"
        )

    if show_structure or report.structural_issues:
        lines.extend(["", "structural checks:"])
        if report.structural_issues:
            lines.append("  informational by default; pass --strict-structure to fail on these findings.")
            for issue in report.structural_issues:
                lines.append(f"  - {issue.severity}: {issue.contract_id}: {issue.message}")
        else:
            lines.append("  no structural findings")

    return "\n".join(lines)


def main(argv: Optional[list[str]] = None) -> int:
    parser = argparse.ArgumentParser(
        description="Report overlap and gaps between schema manifests and runtime output contracts."
    )
    parser.add_argument(
        "--manifests-dir",
        default=str(MANIFESTS_DIR),
        help="Path to schemas/manifests.",
    )
    parser.add_argument(
        "--registry",
        default=str(RUNTIME_REGISTRY_PATH),
        help="Path to scripts/contracts/output-contracts.json.",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Print contract-level references under each family.",
    )
    parser.add_argument(
        "--strict-structure",
        action="store_true",
        help="Exit non-zero when informational structural checks find missing matrix evidence.",
    )
    parser.add_argument(
        "--show-structure",
        action="store_true",
        help="Print structural check findings even when --strict-structure is not used.",
    )
    args = parser.parse_args(argv)

    manifests_dir = Path(args.manifests_dir)
    registry_path = Path(args.registry)

    manifest_contracts = load_manifest_contracts(manifests_dir)
    runtime_contracts = load_runtime_contracts(registry_path)
    routes = load_manifest_routes(manifests_dir)
    report = build_promotion_report(manifest_contracts, runtime_contracts, routes=routes)
    print(
        render_text_report(
            report,
            manifest_contracts,
            runtime_contracts,
            verbose=args.verbose,
            show_structure=args.show_structure or args.strict_structure,
        )
    )
    if args.strict_structure and report.structural_issues:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
