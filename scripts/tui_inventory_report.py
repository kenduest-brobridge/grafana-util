#!/usr/bin/env python3
"""Print a read-only inventory of current TUI and interactive surfaces."""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import asdict, dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
REGISTRY_PATH = REPO_ROOT / "scripts" / "contracts" / "tui-registry.json"
SCAN_ROOTS = (
    Path("rust/src"),
    Path("docs/commands/en"),
    Path("docs/commands/zh-TW"),
    Path("docs/user-guide/en"),
    Path("docs/user-guide/zh-TW"),
    Path("docs/internal"),
)
SKIP_PARTS = {"target", "html", "archive", "__pycache__"}
SKIP_FILES = {
    Path("docs/commands/en/completion.md"),
    Path("docs/internal/ai-changes.md"),
    Path("docs/internal/ai-learnings.md"),
    Path("docs/internal/ai-status.md"),
}
TUI_RE = re.compile(
    r"ratatui|crossterm|tui_shell|TerminalSession|feature = \"tui\"|"
    r"--interactive|output-format interactive|browse interactively|"
    r"interactive terminal|terminal UI|Terminal UI|TUI|"
    r"互動式終端機|互動式清單視圖",
)
PUBLIC_TUI_DOC_RE = re.compile(
    r"--interactive|--output-format\s+interactive|output-format interactive|"
    r"interactive terminal|terminal UI|Terminal UI|"
    r"互動式終端機|互動式清單視圖",
)
NAMESPACE_COMMAND_DOCS = {
    "docs/commands/en/access.md",
    "docs/commands/en/dashboard.md",
    "docs/commands/en/datasource.md",
    "docs/commands/en/snapshot.md",
    "docs/commands/en/status.md",
}
HELPER_DRIFT_RE = re.compile(
    r"^\s*fn\s+"
    r"(?P<helper>detail_line|fact_line|build_info_lines|build_review_lines|"
    r"control_line|key_chip|plain|muted|plain_boxed|boxed)\s*\("
)


@dataclass(frozen=True)
class InventoryItem:
    path: str
    kind: str
    signals: tuple[str, ...]


@dataclass(frozen=True)
class HelperDriftItem:
    path: str
    helper: str
    line: int
    signal: str


def iter_scan_files() -> list[Path]:
    files: list[Path] = []
    for root in SCAN_ROOTS:
        absolute_root = REPO_ROOT / root
        if not absolute_root.exists():
            continue
        for path in absolute_root.rglob("*"):
            if not path.is_file():
                continue
            relative = path.relative_to(REPO_ROOT)
            if any(part in SKIP_PARTS for part in relative.parts):
                continue
            if relative in SKIP_FILES:
                continue
            if path.suffix == ".rs" and (
                "tests" in relative.parts
                or path.name.endswith("_rust_tests.rs")
                or path.name.endswith("_tests.rs")
            ):
                continue
            if path.suffix not in {".rs", ".md"}:
                continue
            files.append(relative)
    return sorted(files)


def classify(path: Path, text: str, signals: tuple[str, ...]) -> str:
    path_text = path.as_posix()
    if path.suffix == ".md":
        return "docs"
    if "common/tui" in path_text or "common/browser/session.rs" in path_text:
        return "shared"
    if "browse" in path_text:
        return "browse"
    if "workbench" in path_text or "review_tui" in path_text or "audit_tui" in path_text:
        return "workbench"
    if "interactive" in path_text or "--interactive" in text:
        return "interactive"
    if any("feature = \"tui\"" in signal for signal in signals):
        return "feature-gated"
    return "other"


def collect_signals(text: str) -> tuple[str, ...]:
    signals: list[str] = []
    for line in text.splitlines():
        match = TUI_RE.search(line)
        if match:
            snippet = line.strip()
            if len(snippet) > 120:
                snippet = snippet[:117].rstrip() + "..."
            signals.append(snippet)
        if len(signals) >= 3:
            break
    return tuple(signals)


def build_inventory() -> list[InventoryItem]:
    items: list[InventoryItem] = []
    for relative in iter_scan_files():
        text = (REPO_ROOT / relative).read_text(encoding="utf-8")
        signals = collect_signals(text)
        if not signals:
            continue
        items.append(
            InventoryItem(
                path=relative.as_posix(),
                kind=classify(relative, text, signals),
                signals=signals,
            )
        )
    return items


def collect_helper_drift(path: Path, text: str) -> list[HelperDriftItem]:
    path_text = path.as_posix()
    if "common/tui" in path_text or "common/browser/session.rs" in path_text:
        return []
    if "tests" in path.parts or path.name.endswith("_tests.rs"):
        return []
    items: list[HelperDriftItem] = []
    lines = text.splitlines()
    for index, line in enumerate(lines, start=1):
        match = HELPER_DRIFT_RE.search(line)
        if not match:
            continue
        body = "\n".join(lines[index : min(index + 6, len(lines))])
        body_signal = next(
            (
                body_line.strip()
                for body_line in body.splitlines()
                if "tui_shell::" in body_line or "browser_" in body_line or "format!" in body_line
            ),
            "",
        )
        if body_signal:
            signal = f"{line.strip()} -> {body_signal}"
            items.append(
                HelperDriftItem(
                    path=path_text,
                    helper=match.group("helper"),
                    line=index,
                    signal=signal,
                )
            )
    return items


def build_helper_drift() -> list[HelperDriftItem]:
    items: list[HelperDriftItem] = []
    for relative in iter_scan_files():
        if relative.suffix != ".rs":
            continue
        text = (REPO_ROOT / relative).read_text(encoding="utf-8")
        items.extend(collect_helper_drift(relative, text))
    return sorted(items, key=lambda item: (item.path, item.line, item.helper))


@dataclass(frozen=True)
class RegistryFinding:
    kind: str
    surface: str
    message: str


DEFAULT_OPERATOR_FRIENDLINESS = {
    "summary": "Registry-owned support notes; verify screen copy, navigation, fallback, and safety in the owning surface tests.",
    "search": "Supports / search or documents a surface-specific deviation.",
    "detail_scroll": "Detail panes clamp scrolling to rendered content where scrollable.",
    "footer": "Footer copy advertises only active controls and uses compact exit language.",
    "confirmation": "Mutation-capable surfaces separate confirm and cancel controls; read-only surfaces mark confirmation not applicable.",
    "secret_redaction": "Secret-like values must not render in detail, diff, or confirmation panes.",
    "blockers": "Mutation review surfaces keep blockers visible before apply; read-only surfaces mark blockers not applicable.",
}


def infer_entrypoint_kind(command: str) -> str:
    if "--output-format interactive" in command:
        return "output-format"
    if "--interactive" in command:
        return "flag"
    return "implicit"


def normalize_registry_entry(entry: dict) -> dict:
    normalized = dict(entry)
    domain = normalized.get("domain", "")
    normalized.setdefault("owner", f"rust/src/commands/{domain}")
    normalized.setdefault("entrypoint_kind", infer_entrypoint_kind(normalized["command"]))
    normalized.setdefault("validation", "python3 -m unittest -v scripts/test_tui_inventory_report.py")
    friendliness = dict(DEFAULT_OPERATOR_FRIENDLINESS)
    friendliness.update(normalized.get("operator_friendliness", {}))
    normalized["operator_friendliness"] = friendliness
    return normalized


def load_registry() -> dict:
    registry = json.loads(REGISTRY_PATH.read_text(encoding="utf-8"))
    registry["surfaces"] = [
        normalize_registry_entry(entry) for entry in registry.get("surfaces", [])
    ]
    return registry


def matching_code_paths(entry: dict, detected_rust_paths: set[str]) -> list[str]:
    domain = entry.get("domain", "")
    matching_rust = [
        p for p in detected_rust_paths if p.startswith(f"rust/src/commands/{domain}/")
    ]
    matching_infra = [
        p
        for p in detected_rust_paths
        if domain in p and p.startswith("rust/src/common/")
    ]
    return sorted(matching_rust + matching_infra)


def build_surface_summary(registry: dict, items: list[InventoryItem]) -> list[dict]:
    detected_doc_paths = {item.path for item in items if item.kind == "docs"}
    detected_rust_paths = {item.path for item in items if item.kind != "docs"}

    summary: list[dict] = []
    for entry in registry.get("surfaces", []):
        docs = [
            path
            for path in (entry.get("docs_en"), entry.get("docs_zh_TW"))
            if path is not None
        ]
        code_paths = matching_code_paths(entry, detected_rust_paths)
        summary.append(
            {
                "command": entry["command"],
                "owner": entry["owner"],
                "tier": entry["tier"],
                "entrypointKind": entry["entrypoint_kind"],
                "mode": entry.get("mode", []),
                "behavior": entry.get("behavior"),
                "docsDetected": {path: path in detected_doc_paths for path in docs},
                "codeDetected": bool(code_paths),
                "codeSignals": code_paths[:5],
                "fallback": entry.get("fallback"),
                "operatorFriendliness": entry["operator_friendliness"],
                "validation": entry["validation"],
            }
        )
    return summary


def check_registry(items: list[InventoryItem]) -> list[RegistryFinding]:
    registry = load_registry()
    findings: list[RegistryFinding] = []

    detected_doc_paths = {item.path for item in items if item.kind == "docs"}
    detected_rust_paths = {item.path for item in items if item.kind != "docs"}

    for entry in registry.get("surfaces", []):
        command = entry["command"]
        docs_en = entry.get("docs_en")
        docs_zh_tw = entry.get("docs_zh_TW")

        # Check English docs exist
        if docs_en:
            if docs_en not in detected_doc_paths:
                path_text = docs_en.replace("/en/", "/zh-TW/")
                findings.append(
                    RegistryFinding(
                        kind="missing-doc",
                        surface=command,
                        message=f"English doc `{docs_en}` not detected in TUI scan; may be missing or not contain TUI/interactive signals",
                    )
                )
        # Check zh-TW docs exist
        if docs_zh_tw:
            if docs_zh_tw not in detected_doc_paths:
                findings.append(
                    RegistryFinding(
                        kind="missing-doc-zh-TW",
                        surface=command,
                        message=f"zh-TW doc `{docs_zh_tw}` not detected in TUI scan; may be missing or not contain TUI/interactive signals",
                    )
                )

        # Check that at least one matching Rust file is detected for this surface
        domain = entry.get("domain", "")
        matching_rust = matching_code_paths(entry, detected_rust_paths)
        if not matching_rust:
            findings.append(
                RegistryFinding(
                    kind="missing-code",
                    surface=command,
                    message=f"no Rust file detected under `rust/src/commands/{domain}/` matching TUI signals",
                )
            )

    # Check for undocumented surfaces: English command docs with TUI signals not in registry
    registry_doc_paths = {
        entry.get("docs_en")
        for entry in registry.get("surfaces", [])
        if entry.get("docs_en")
    }
    for doc_path in sorted(detected_doc_paths):
        if not doc_path.startswith("docs/commands/en/"):
            continue
        if doc_path in NAMESPACE_COMMAND_DOCS:
            continue
        if doc_path in registry_doc_paths:
            continue
        doc_text = (REPO_ROOT / doc_path).read_text(encoding="utf-8")
        if not PUBLIC_TUI_DOC_RE.search(doc_text):
            continue
        findings.append(
            RegistryFinding(
                kind="undocumented-surface",
                surface=doc_path,
                message=f"command doc has TUI signals but is not claimed in registry; add a registry entry or verify TUI scope",
            )
        )

    return findings


def print_text_report(
    items: list[InventoryItem],
    helper_drift: list[HelperDriftItem],
    registry_findings: list[RegistryFinding] | None = None,
    surface_summary: list[dict] | None = None,
) -> None:
    by_kind: dict[str, list[InventoryItem]] = {}
    for item in items:
        by_kind.setdefault(item.kind, []).append(item)

    print("TUI inventory report")
    print("====================")
    print(f"Scanned roots: {', '.join(root.as_posix() for root in SCAN_ROOTS)}")
    print(f"Matched files: {len(items)}")
    print()
    for kind in sorted(by_kind):
        grouped = by_kind[kind]
        print(f"{kind} ({len(grouped)})")
        for item in grouped:
            print(f"  - {item.path}")
            print(f"    signal: {item.signals[0]}")
        print()

    print(f"helper-drift candidates ({len(helper_drift)})")
    for item in helper_drift:
        print(f"  - {item.path}:{item.line} {item.helper}")
        print(f"    signal: {item.signal}")
    print()

    if registry_findings is not None:
        print(f"registry findings ({len(registry_findings)})")
        by_kind_rf: dict[str, list[RegistryFinding]] = {}
        for finding in registry_findings:
            by_kind_rf.setdefault(finding.kind, []).append(finding)
        for kind in sorted(by_kind_rf):
            grouped = by_kind_rf[kind]
            print(f"  {kind} ({len(grouped)})")
            for finding in grouped:
                print(f"    - {finding.surface}")
                print(f"      {finding.message}")
        print()

    if surface_summary is not None:
        print(f"surface summary ({len(surface_summary)})")
        for surface in surface_summary:
            docs = surface["docsDetected"]
            detected_docs = sum(1 for detected in docs.values() if detected)
            print(f"  - {surface['command']}")
            print(f"    owner: {surface['owner']}")
            print(f"    tier: {surface['tier']} / {surface['entrypointKind']}")
            print(f"    docs: {detected_docs}/{len(docs)} detected")
            print(f"    code: {'detected' if surface['codeDetected'] else 'missing'}")
            print(f"    validation: {surface['validation']}")
        print()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit machine-readable inventory")
    parser.add_argument(
        "--registry-check",
        action="store_true",
        help="load tui-registry.json and report mismatches (advisory only)",
    )
    args = parser.parse_args()

    items = build_inventory()
    helper_drift = build_helper_drift()

    registry_findings: list[RegistryFinding] = []
    surface_summary: list[dict] = []
    if args.registry_check:
        registry = load_registry()
        registry_findings = check_registry(items)
        surface_summary = build_surface_summary(registry, items)

    if args.json:
        output: dict[str, object] = {
            "items": [asdict(item) for item in items],
            "helperDrift": [asdict(item) for item in helper_drift],
        }
        if args.registry_check:
            output["registryFindings"] = [asdict(f) for f in registry_findings]
            output["surfaceSummary"] = surface_summary
        print(json.dumps(output, indent=2, sort_keys=True))
    else:
        print_text_report(items, helper_drift, registry_findings, surface_summary)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
