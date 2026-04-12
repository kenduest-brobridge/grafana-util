from __future__ import annotations

import json
from functools import lru_cache
from pathlib import Path

from docgen_command_docs import command_doc_cli_path
from docgen_common import relative_href
from docsite_html_common import HtmlBuildConfig, prefixed_output_rel
from docsite_ui_text import ui_text

SURFACE_PATH = Path(__file__).resolve().parent / "contracts" / "command-surface.json"
TAXONOMY_PATH = Path(__file__).resolve().parent / "contracts" / "command-taxonomy.json"


def normalize_cli_path(cli_path: str) -> str:
    stripped = cli_path.strip()
    if stripped.startswith("grafana-util "):
        stripped = stripped[len("grafana-util ") :]
    return stripped.strip()


def command_display_label(cli_path: str) -> str:
    return normalize_cli_path(cli_path)


@lru_cache(maxsize=1)
def load_command_surface() -> dict:
    return json.loads(SURFACE_PATH.read_text(encoding="utf-8"))


@lru_cache(maxsize=1)
def load_command_taxonomy() -> dict:
    raw = json.loads(TAXONOMY_PATH.read_text(encoding="utf-8"))
    if raw.get("schema") != 1:
        raise ValueError(f"Unsupported command taxonomy schema in {TAXONOMY_PATH}")
    return raw


@lru_cache(maxsize=1)
def command_doc_pages() -> dict[str, str]:
    raw = load_command_surface().get("doc_pages", {})
    if not isinstance(raw, dict):
        return {}
    return {normalize_cli_path(cli_path): filename for cli_path, filename in raw.items()}


@lru_cache(maxsize=1)
def handbook_context_by_page() -> dict[str, str]:
    raw = load_command_taxonomy().get("handbook_context_by_page", {})
    if not isinstance(raw, dict):
        return {}
    return {str(key): str(value) for key, value in raw.items()}


@lru_cache(maxsize=1)
def audience_hints_by_root() -> dict[str, dict[str, str]]:
    raw = load_command_taxonomy().get("audience_hints_by_root", {})
    if not isinstance(raw, dict):
        return {}
    hints: dict[str, dict[str, str]] = {}
    for root, values in raw.items():
        if not isinstance(values, dict):
            continue
        hints[str(root)] = {str(locale): str(text) for locale, text in values.items()}
    return hints


def resolve_command_cli_path(source_path: Path) -> str | None:
    fallback = f"grafana-util {source_path.stem.replace('-', ' ')}"
    cli_path = command_doc_cli_path(source_path, fallback)
    if cli_path is None:
        return None
    return normalize_cli_path(cli_path)


def command_root_key(source_path: Path) -> str:
    return source_path.stem.split("-", 1)[0]


def command_handbook_stem(source_path: Path) -> str | None:
    page_key = source_path.stem
    handbook_map = handbook_context_by_page()
    return handbook_map.get(page_key) or handbook_map.get(command_root_key(source_path))


def command_audience_hint(locale: str, source_path: Path) -> str:
    hints = audience_hints_by_root().get(command_root_key(source_path))
    if not hints:
        return ""
    return hints.get(locale, "")


def parent_command_path(cli_path: str, pages: dict[str, str]) -> str | None:
    tokens = cli_path.split()
    for size in range(len(tokens) - 1, 0, -1):
        candidate = " ".join(tokens[:size])
        if candidate in pages:
            return candidate
    return None


def child_command_paths(cli_path: str, pages: dict[str, str]) -> tuple[str, ...]:
    prefix = f"{cli_path} "
    children = []
    for candidate in sorted(pages):
        if not candidate.startswith(prefix):
            continue
        tail = candidate[len(prefix) :]
        if " " in tail:
            continue
        children.append(candidate)
    return tuple(children)


def sibling_command_paths(cli_path: str, pages: dict[str, str]) -> tuple[str, ...]:
    parent = parent_command_path(cli_path, pages)
    if parent is None:
        return ()
    return tuple(candidate for candidate in child_command_paths(parent, pages) if candidate != cli_path)


def command_doc_href(output_rel: str, locale: str, cli_path: str, config: HtmlBuildConfig) -> str | None:
    target = command_doc_pages().get(normalize_cli_path(cli_path))
    if not target:
        return None
    return relative_href(output_rel, prefixed_output_rel(config, f"commands/{locale}/{target[:-3]}.html"))


def command_breadcrumb_parents(locale: str, output_rel: str, source_path: Path, config: HtmlBuildConfig) -> list[tuple[str, str]]:
    pages = command_doc_pages()
    cli_path = resolve_command_cli_path(source_path)
    if cli_path is None:
        return []
    breadcrumbs: list[tuple[str, str]] = []
    seen: set[str] = set()
    parent = parent_command_path(cli_path, pages)
    while parent is not None and parent not in seen:
        seen.add(parent)
        href = command_doc_href(output_rel, locale, parent, config)
        if href:
            breadcrumbs.append((command_display_label(parent), href))
        parent = parent_command_path(parent, pages)
    breadcrumbs.reverse()
    return breadcrumbs


def command_reference_links(locale: str, output_rel: str, source_path: Path, config: HtmlBuildConfig) -> list[tuple[str, str]]:
    pages = command_doc_pages()
    cli_path = resolve_command_cli_path(source_path)
    if cli_path is None:
        return []

    locale_labels = {
        "parent": ui_text(locale, "parent_command_label"),
        "child": ui_text(locale, "child_command_label"),
        "sibling": ui_text(locale, "sibling_command_label"),
    }

    links: list[tuple[str, str]] = []
    parent = parent_command_path(cli_path, pages)
    if parent is not None:
        href = command_doc_href(output_rel, locale, parent, config)
        if href:
            links.append((f'{locale_labels["parent"]}: {command_display_label(parent)}', href))

    for child in child_command_paths(cli_path, pages):
        href = command_doc_href(output_rel, locale, child, config)
        if href:
            links.append((f'{locale_labels["child"]}: {command_display_label(child)}', href))

    for sibling in sibling_command_paths(cli_path, pages):
        href = command_doc_href(output_rel, locale, sibling, config)
        if href:
            links.append((f'{locale_labels["sibling"]}: {command_display_label(sibling)}', href))
    return links
