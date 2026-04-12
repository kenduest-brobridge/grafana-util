from __future__ import annotations

import json
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path

from docgen_command_docs import RenderedMarkdownDocument, render_markdown_document


INDEX_PATH = Path(__file__).resolve().parent / "contracts" / "command-reference-index.json"


@dataclass(frozen=True)
class IndexSection:
    title: str
    section_type: str
    columns: tuple[str, ...] = ()
    rows: tuple[tuple[str, ...], ...] = ()
    items: tuple[dict[str, str], ...] = ()
    lead: tuple[str, ...] = ()


@dataclass(frozen=True)
class CommandReferenceIndex:
    title: str
    intro: tuple[str, ...]
    sections: tuple[IndexSection, ...]


@lru_cache(maxsize=1)
def load_command_reference_index() -> dict:
    raw = json.loads(INDEX_PATH.read_text(encoding="utf-8"))
    if raw.get("schema") != 1:
        raise ValueError(f"Unsupported command reference index schema in {INDEX_PATH}")
    return raw


@lru_cache(maxsize=None)
def command_reference_index(locale: str) -> CommandReferenceIndex:
    locales = load_command_reference_index().get("locales", {})
    if not isinstance(locales, dict) or locale not in locales:
        raise KeyError(f"Missing command reference index locale {locale}")
    raw = locales[locale]
    if not isinstance(raw, dict):
        raise TypeError(f"command reference index locale {locale} must be an object")
    sections: list[IndexSection] = []
    for section in raw.get("sections", []):
        if not isinstance(section, dict):
            raise TypeError("command reference index section must be an object")
        items_value = section.get("items", [])
        items: list[dict[str, str]] = []
        if isinstance(items_value, list):
            for item in items_value:
                if isinstance(item, str):
                    items.append({"text": item})
                elif isinstance(item, dict):
                    items.append({str(key): str(value) for key, value in item.items()})
        rows_value = section.get("rows", [])
        rows: list[tuple[str, ...]] = []
        if isinstance(rows_value, list):
            for row in rows_value:
                if isinstance(row, list):
                    rows.append(tuple(str(cell) for cell in row))
        lead_value = section.get("lead", [])
        lead = tuple(str(item) for item in lead_value) if isinstance(lead_value, list) else ()
        columns_value = section.get("columns", [])
        columns = tuple(str(item) for item in columns_value) if isinstance(columns_value, list) else ()
        sections.append(
            IndexSection(
                title=str(section.get("title", "")),
                section_type=str(section.get("type", "")),
                columns=columns,
                rows=tuple(rows),
                items=tuple(items),
                lead=lead,
            )
        )
    return CommandReferenceIndex(
        title=str(raw.get("title", "")),
        intro=tuple(str(item) for item in raw.get("intro", [])),
        sections=tuple(sections),
    )


def _render_link_item(item: dict[str, str]) -> str:
    label = item["label"]
    target = item["target"]
    if item.get("summary"):
        return f"- [{label}]({target}): {item['summary']}"
    return f"- [{label}]({target})"


def _render_numbered_link_item(index: int, item: dict[str, str]) -> str:
    label = item["label"]
    target = item["target"]
    if item.get("summary"):
        return f"{index}. [{label}]({target})：{item['summary']}"
    return f"{index}. [{label}]({target})"


def _render_table(section: IndexSection) -> list[str]:
    lines = ["| " + " | ".join(section.columns) + " |", "| " + " | ".join(["---"] * len(section.columns)) + " |"]
    for row in section.rows:
        lines.append("| " + " | ".join(row) + " |")
    return lines


def command_reference_index_markdown(locale: str) -> str:
    page = command_reference_index(locale)
    lines = [f"# {page.title}", ""]
    for paragraph in page.intro:
        lines.append(paragraph)
        lines.append("")
    for section in page.sections:
        lines.append(f"## {section.title}")
        lines.append("")
        for paragraph in section.lead:
            lines.append(paragraph)
            lines.append("")
        if section.section_type == "bullet_links":
            lines.extend(_render_link_item(item) for item in section.items)
        elif section.section_type == "numbered_links":
            lines.extend(_render_numbered_link_item(index + 1, item) for index, item in enumerate(section.items))
        elif section.section_type == "bullet_text":
            for item in section.items:
                lines.append(f"- {item.get('text', '')}")
        elif section.section_type == "table":
            lines.extend(_render_table(section))
        else:
            raise ValueError(f"Unsupported command reference index section type: {section.section_type}")
        lines.append("")
    return "\n".join(lines).rstrip() + "\n"


def render_command_reference_index(locale: str, link_transform=None) -> RenderedMarkdownDocument:
    return render_markdown_document(command_reference_index_markdown(locale), link_transform=link_transform)
