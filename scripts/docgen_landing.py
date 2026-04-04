"""Structured metadata for the generated docs landing page.

The landing page is authored in Markdown, but the homepage itself is rendered
through the HTML generator. This module keeps the landing-content contract and
parsing logic separate from the page shell and styling.
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path

from docgen_command_docs import clean_markdown
from docgen_common import REPO_ROOT


def get_landing_root(repo_root: Path = REPO_ROOT) -> Path:
    return repo_root / "docs" / "landing"


LANDING_ROOT = get_landing_root()
LANDING_LOCALES = ("en", "zh-TW")
LANDING_UI_LABELS = {
    "en": {
        "search_placeholder": "Search tasks, commands, or handbook pages",
        "search_button": "Search",
    },
    "zh-TW": {
        "search_placeholder": "搜尋任務、指令或手冊頁面",
        "search_button": "搜尋",
    },
}


@dataclass(frozen=True)
class LandingLink:
    label: str
    target: str


@dataclass(frozen=True)
class LandingTask:
    title: str
    summary: str
    links: tuple[LandingLink, ...]


@dataclass(frozen=True)
class LandingSection:
    title: str
    summary: str
    links: tuple[LandingLink, ...]
    tasks: tuple[LandingTask, ...]


@dataclass(frozen=True)
class LandingPage:
    locale: str
    source_path: Path
    title: str
    summary: str
    search: LandingSection
    sections: tuple[LandingSection, ...]
    maintainer: LandingSection


@dataclass
class _TaskBuilder:
    title: str
    lines: list[str]


@dataclass
class _SectionBuilder:
    title: str
    lines: list[str]
    tasks: list[_TaskBuilder]


LINK_LINE_RE = re.compile(r"^- \[([^\]]+)\]\(([^)]+)\)\s*$")


def load_landing_page(locale: str, landing_root: Path = LANDING_ROOT) -> LandingPage:
    if locale not in LANDING_LOCALES:
        raise ValueError(f"Unsupported landing locale: {locale}")
    return parse_landing_page(landing_root / f"{locale}.md", locale)


def parse_landing_page(path: Path, locale: str) -> LandingPage:
    text = path.read_text(encoding="utf-8")
    return parse_landing_text(text, locale=locale, source_path=path)


def parse_landing_text(text: str, *, locale: str, source_path: Path) -> LandingPage:
    title = ""
    hero_lines: list[str] = []
    sections: list[_SectionBuilder] = []
    current_section: _SectionBuilder | None = None
    current_task: _TaskBuilder | None = None

    def flush_task() -> None:
        nonlocal current_task
        if current_task is not None and current_section is not None:
            current_section.tasks.append(current_task)
            current_task = None

    def flush_section() -> None:
        nonlocal current_section
        flush_task()
        if current_section is not None:
            sections.append(current_section)
            current_section = None

    for raw_line in text.splitlines():
        if raw_line.startswith("# "):
            title = clean_markdown(raw_line[2:].strip())
            continue
        if raw_line.startswith("## "):
            flush_section()
            current_section = _SectionBuilder(title=clean_markdown(raw_line[3:].strip()), lines=[], tasks=[])
            continue
        if raw_line.startswith("### "):
            if current_section is None:
                raise ValueError(f"Landing task heading appeared before a section in {source_path}")
            flush_task()
            current_task = _TaskBuilder(title=clean_markdown(raw_line[4:].strip()), lines=[])
            continue
        if current_task is not None:
            current_task.lines.append(raw_line)
        elif current_section is not None:
            current_section.lines.append(raw_line)
        else:
            hero_lines.append(raw_line)

    flush_section()

    if not title:
        raise ValueError(f"Landing page is missing a top-level title: {source_path}")
    if len(sections) < 2:
        raise ValueError(f"Landing page must have at least Search and Maintainer sections: {source_path}")

    hero_summary = _extract_first_paragraph(hero_lines)
    if not hero_summary:
        raise ValueError(f"Landing page is missing hero summary text: {source_path}")

    parsed_sections = tuple(_build_section(section, source_path) for section in sections)
    search = parsed_sections[0]
    maintainer = parsed_sections[-1]
    body_sections = parsed_sections[1:-1]
    if not body_sections:
        raise ValueError(f"Landing page must define at least one body section between Search and Maintainer: {source_path}")

    if search.tasks:
        raise ValueError(f"Search section cannot contain tasks: {source_path}")
    if maintainer.tasks:
        raise ValueError(f"Maintainer section cannot contain tasks: {source_path}")

    return LandingPage(
        locale=locale,
        source_path=source_path,
        title=title,
        summary=hero_summary,
        search=search,
        sections=body_sections,
        maintainer=maintainer,
    )


def _build_section(section: _SectionBuilder, source_path: Path) -> LandingSection:
    summary = _extract_first_paragraph(section.lines)
    links = _extract_links(section.lines, source_path)
    tasks = tuple(_build_task(task, source_path) for task in section.tasks)
    if not summary:
        raise ValueError(f"Landing section '{section.title}' is missing summary text: {source_path}")
    return LandingSection(title=section.title, summary=summary, links=links, tasks=tasks)


def _build_task(task: _TaskBuilder, source_path: Path) -> LandingTask:
    summary = _extract_first_paragraph(task.lines)
    links = _extract_links(task.lines, source_path)
    if not summary:
        raise ValueError(f"Landing task '{task.title}' is missing summary text: {source_path}")
    if not links:
        raise ValueError(f"Landing task '{task.title}' must define at least one Markdown link bullet: {source_path}")
    return LandingTask(title=task.title, summary=summary, links=links)


def _extract_first_paragraph(lines: list[str]) -> str:
    paragraph: list[str] = []
    for raw_line in lines:
        stripped = raw_line.strip()
        if not stripped:
            if paragraph:
                break
            continue
        if stripped.startswith("- "):
            if paragraph:
                break
            continue
        paragraph.append(clean_markdown(stripped))
    return " ".join(paragraph)


def _extract_links(lines: list[str], source_path: Path) -> tuple[LandingLink, ...]:
    links: list[LandingLink] = []
    for raw_line in lines:
        stripped = raw_line.strip()
        if not stripped.startswith("- "):
            continue
        match = LINK_LINE_RE.fullmatch(stripped)
        if match is None:
            raise ValueError(f"Landing link bullets must use standard Markdown link syntax: {source_path} -> {stripped}")
        links.append(LandingLink(label=clean_markdown(match.group(1).strip()), target=match.group(2).strip()))
    return tuple(links)
