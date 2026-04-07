#!/usr/bin/env python3
"""Generate JSON Schema and schema-help artifacts from repo-owned manifests."""

from __future__ import annotations

import argparse
import json
from copy import deepcopy
from pathlib import Path
from typing import Any

from docgen_common import REPO_ROOT, check_outputs, print_written_outputs, write_outputs


MANIFESTS_DIR = REPO_ROOT / "schemas" / "manifests"
JSONSCHEMA_DIR = REPO_ROOT / "schemas" / "jsonschema"
HELP_DIR = REPO_ROOT / "schemas" / "help"


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def load_manifest_collections() -> tuple[dict[str, dict[str, Any]], list[dict[str, Any]]]:
    contracts: dict[str, dict[str, Any]] = {}
    routes: list[dict[str, Any]] = []
    for family_dir in sorted(path for path in MANIFESTS_DIR.iterdir() if path.is_dir()):
        family = family_dir.name
        contract_path = family_dir / "contracts.json"
        route_path = family_dir / "routes.json"
        if not contract_path.exists() or not route_path.exists():
            continue
        contract_entries = load_json(contract_path)
        route_entries = load_json(route_path)
        for contract in contract_entries:
            contract_id = contract["contractId"]
            if contract_id in contracts:
                raise SystemExit(f"duplicate contract id: {contract_id}")
            contract["family"] = family
            contracts[contract_id] = contract
        for route in route_entries:
            route["family"] = family
            routes.append(route)
    if not contracts:
        raise SystemExit("no schema manifests were found")
    return contracts, routes


def ensure_golden_reference_exists(reference: str) -> None:
    path_text, _, fragment = reference.partition("#")
    path = REPO_ROOT / path_text
    if not path.exists():
        raise SystemExit(f"golden sample does not exist: {reference}")
    if not fragment:
        return
    payload = load_json(path)
    if isinstance(payload, list):
        for item in payload:
            if isinstance(item, dict) and (
                item.get("contractId") == fragment
                or item.get("domain") == fragment
                or item.get("id") == fragment
            ):
                return
    elif isinstance(payload, dict):
        if fragment in payload:
            return
    raise SystemExit(f"golden sample fragment not found: {reference}")


def validate_manifests(contracts: dict[str, dict[str, Any]], routes: list[dict[str, Any]]) -> None:
    for contract in contracts.values():
        ensure_golden_reference_exists(contract["goldenSample"])
        for section in contract.get("nestedSections", []):
            if not section["path"]:
                raise SystemExit(f"empty nested section path in {contract['contractId']}")
    for route in routes:
        if not route.get("contractSections"):
            raise SystemExit(f"route has no contractSections: {route['routeId']}")
        for section in route["contractSections"]:
            contract_id = section["contractId"] if isinstance(section, dict) else section
            if contract_id not in contracts:
                raise SystemExit(
                    f"route {route['routeId']} references unknown contract {contract_id}"
                )


def split_path(path: str) -> list[tuple[str, bool]]:
    parts: list[tuple[str, bool]] = []
    for chunk in path.split("."):
        is_array = chunk.endswith("[]")
        name = chunk[:-2] if is_array else chunk
        parts.append((name, is_array))
    return parts


def ensure_container_schema(root: dict[str, Any], path: str) -> dict[str, Any]:
    current = root
    parts = split_path(path)
    for index, (name, is_array) in enumerate(parts):
        is_last = index == len(parts) - 1
        current.setdefault("type", "object")
        current.setdefault("properties", {})
        current.setdefault("required", [])
        current.setdefault("additionalProperties", False)
        if name not in current["properties"]:
            current["properties"][name] = {"type": "array" if is_array else "object"}
        node = current["properties"][name]
        if name not in current["required"]:
            current["required"].append(name)
        if is_array:
            node.setdefault("type", "array")
            node.setdefault("items", {"type": "object"})
            current = node["items"]
        else:
            current = node
        if not is_last:
            current.setdefault("type", "object")
    current.setdefault("type", "object")
    current.setdefault("properties", {})
    current.setdefault("required", [])
    current.setdefault("additionalProperties", False)
    return current


def set_field_schema(root: dict[str, Any], path: str, schema: dict[str, Any]) -> None:
    parts = split_path(path)
    if not parts:
        return
    parent_path = ".".join(
        f"{name}{'[]' if is_array else ''}" for name, is_array in parts[:-1]
    )
    parent = root if not parent_path else ensure_container_schema(root, parent_path)
    parent.setdefault("type", "object")
    parent.setdefault("properties", {})
    parent.setdefault("required", [])
    parent.setdefault("additionalProperties", False)
    field_name, _ = parts[-1]
    parent["properties"][field_name] = deepcopy(schema)
    if field_name not in parent["required"]:
        parent["required"].append(field_name)


def build_json_schema(contract: dict[str, Any]) -> dict[str, Any]:
    schema = {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": f"https://schemas.grafana-util.local/{contract['family']}/{contract['contractId']}.schema.json",
        "title": contract["title"],
        "type": "object",
        "required": list(contract["topLevelKeys"]),
        "additionalProperties": False,
        "properties": {},
    }
    for section in contract.get("nestedSections", []):
        ensure_container_schema(schema, section["path"])
        container = ensure_container_schema(schema, section["path"])
        for key in section["keys"]:
            container["properties"].setdefault(key, {})
            if key not in container["required"]:
                container["required"].append(key)
    for key, field_schema in contract.get("fieldSchemas", {}).items():
        set_field_schema(schema, key, field_schema)
    return schema


def format_key_list(keys: list[str]) -> str:
    return ", ".join(keys)


def render_contract_section(
    contract: dict[str, Any],
    *,
    heading_label: str | None = None,
    description: str | None = None,
    show_nested: bool = False,
    shape_label: str | None = None,
) -> list[str]:
    lines: list[str] = []
    if heading_label:
        if contract.get("kind"):
            lines.append(f"- {heading_label} -> {contract['kind']}")
        else:
            lines.append(f"- {heading_label} -> {shape_label or contract['title']}")
    elif contract.get("kind"):
        lines.extend(["kind:", f"  {contract['kind']}"])
    elif shape_label:
        lines.extend(["shape:", f"  {shape_label}"])
    if description:
        if heading_label:
            lines.append(f"  {description}")
        else:
            lines.extend(["", f"Notes:\n- {description}"])
    top_level = format_key_list(contract["topLevelKeys"])
    if heading_label:
        lines.append(f"  top-level keys: {top_level}")
    else:
        lines.extend(["", "Top-level keys:", *[f"- {key}" for key in contract["topLevelKeys"]]])
        if show_nested:
            for section in contract.get("nestedSections", []):
                label = section.get("label", section["path"])
                lines.extend([f"- {label}"])
                lines.extend([f"  - {key}" for key in section["keys"]])
    return lines


def render_help_text(route: dict[str, Any], contracts: dict[str, dict[str, Any]]) -> str:
    lines = [route["title"]]
    if route.get("intro"):
        lines.extend(["", *route["intro"]])
    commands = route.get("commands", [])
    if commands:
        lines.extend(["", "Commands:" if len(commands) > 1 else "Command:"])
        lines.extend([f"  {command}" for command in commands])
    if route.get("generalRule"):
        lines.extend(["", "General rule:"])
        lines.extend([f"- {item}" for item in route["generalRule"]])
    if route.get("contractSectionHeading"):
        lines.extend(["", f"{route['contractSectionHeading']}:"])
    sections = route["contractSections"]
    single_contract = len(sections) == 1 and not route.get("contractSectionHeading")
    for section in sections:
        entry = section if isinstance(section, dict) else {"contractId": section}
        contract = contracts[entry["contractId"]]
        lines.extend(
            [""] + render_contract_section(
                contract,
                heading_label=None if single_contract else entry.get("label"),
                description=entry.get("description"),
                show_nested=entry.get("showNested", single_contract),
                shape_label=entry.get("shapeLabel"),
            )
        )
    if route.get("quickLookups"):
        lines.extend(["", "Quick lookups:"])
        lines.extend([f"- {item}" for item in route["quickLookups"]])
    if route.get("notes"):
        lines.extend(["", "Notes:"])
        lines.extend([f"- {item}" for item in route["notes"]])
    return "\n".join(lines).rstrip() + "\n"


def generate_outputs() -> tuple[dict[str, str], dict[str, str]]:
    contracts, routes = load_manifest_collections()
    validate_manifests(contracts, routes)
    schema_outputs: dict[str, str] = {}
    help_outputs: dict[str, str] = {}
    for contract in contracts.values():
        schema_rel = f"{contract['family']}/{contract['contractId']}.schema.json"
        schema_outputs[schema_rel] = (
            json.dumps(build_json_schema(contract), indent=2, sort_keys=False) + "\n"
        )
    for route in routes:
        help_rel = f"{route['family']}/{route['routeId']}.txt"
        help_outputs[help_rel] = render_help_text(route, contracts)
    return schema_outputs, help_outputs


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()

    schema_outputs, help_outputs = generate_outputs()
    if args.check:
        status = 0
        status |= check_outputs(
            JSONSCHEMA_DIR,
            schema_outputs,
            "schema artifacts",
            "make schema",
        )
        status |= check_outputs(
            HELP_DIR,
            help_outputs,
            "schema-help artifacts",
            "make schema",
        )
        return status

    if args.write or not args.check:
        write_outputs(JSONSCHEMA_DIR, schema_outputs)
        write_outputs(HELP_DIR, help_outputs)
        print_written_outputs(
            JSONSCHEMA_DIR,
            schema_outputs,
            "JSON Schema artifacts",
            "schemas/manifests/**/{contracts,routes}.json",
            "schemas/jsonschema/**/*.schema.json",
            "schemas/jsonschema/",
        )
        print_written_outputs(
            HELP_DIR,
            help_outputs,
            "schema-help artifacts",
            "schemas/manifests/**/{contracts,routes}.json",
            "schemas/help/**/*.txt",
            "schemas/help/",
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
