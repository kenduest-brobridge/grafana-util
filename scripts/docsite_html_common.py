from __future__ import annotations

import html
import re
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path
from string import Template

from docgen_command_docs import RenderedHeading
from docgen_common import REPO_ROOT, VERSION, relative_href

HTML_ROOT_DIR = REPO_ROOT / "docs" / "html"
COMMAND_DOCS_ROOT = REPO_ROOT / "docs" / "commands"
HANDBOOK_ROOT = REPO_ROOT / "docs" / "user-guide"
TEMPLATE_ROOT = REPO_ROOT / "scripts" / "templates"
COMMAND_DOC_LOCALES = ("en", "zh-TW")


@dataclass(frozen=True)
class VersionLink:
    label: str
    target_rel: str


@dataclass(frozen=True)
class HtmlBuildConfig:
    source_root: Path = REPO_ROOT
    command_docs_root: Path = COMMAND_DOCS_ROOT
    handbook_root: Path = HANDBOOK_ROOT
    output_prefix: str = ""
    version: str = VERSION
    version_label: str | None = None
    version_links: tuple[VersionLink, ...] = ()
    raw_manpage_target_rel: str | None = None
    include_raw_manpages: bool = False


@lru_cache(maxsize=None)
def load_template(name: str) -> Template:
    return Template((TEMPLATE_ROOT / name).read_text(encoding="utf-8"))


@lru_cache(maxsize=None)
def load_asset(name: str) -> str:
    return (TEMPLATE_ROOT / name).read_text(encoding="utf-8")


def render_template(name: str, **values: str) -> str:
    return load_template(name).substitute(**values)


PAGE_STYLE = load_asset("docs.css") + "\n" + load_asset("prism.css")
THEME_SCRIPT = load_asset("theme_script.js")


def strip_decorative_prefix(text: str) -> str:
    if not text:
        return text
    stripped = text.strip()
    while True:
        match = re.match(r"^([^\w\s\u4e00-\u9fffA-Za-z]+)\s*(.*)$", stripped, flags=re.UNICODE)
        if not match:
            break
        prefix = match.group(1)
        if any(char.isalnum() or ("\u4e00" <= char <= "\u9fff") for char in prefix):
            break
        stripped = match.group(2).lstrip()
    return stripped or text.strip()


def strip_heading_decorations(body_html: str) -> str:
    def replace_heading(match: re.Match[str]) -> str:
        level = match.group(1)
        attrs = match.group(2)
        content = match.group(3)
        cleaned = re.sub(r"^([^\w\s\u4e00-\u9fffA-Za-z<]+)\s*", "", content, flags=re.UNICODE)
        return f"<h{level}{attrs}>{cleaned}</h{level}>"

    return re.sub(r"<h([1-6])([^>]*)>(.*?)</h\1>", replace_heading, body_html, flags=re.DOTALL)


def title_only(text: str) -> str:
    return strip_decorative_prefix(text.replace("`", "")) if text else "grafana-util docs"


def html_list(items: list[tuple[str, str]]) -> str:
    if not items:
        return '<p class="sidebar-meta-text">No related links.</p>'
    rendered: list[str] = []
    for label, href in items:
        classes = "sidebar-meta-link"
        if href == "#":
            classes += " sidebar-meta-current"
        rendered.append(f'<li><a href="{html.escape(href)}" class="{classes}">{html.escape(label)}</a></li>')
    return '<ul class="sidebar-meta-list">' + "".join(rendered) + "</ul>"


def render_breadcrumbs(items: list[tuple[str, str | None]]) -> str:
    if not items:
        return ""
    links = []
    for label, href in items:
        if href:
            links.append(f'<a href="{html.escape(href)}">{html.escape(label)}</a>')
        else:
            links.append(html.escape(label))
    return render_template("breadcrumbs.html.tmpl", items_html=" / ".join(links))


def split_display_title(title: str) -> tuple[str, str | None]:
    title = strip_decorative_prefix(title)
    match = re.fullmatch(r"(.+?)\s*\(([^()]+)\)\s*", title)
    if not match:
        return title, None
    main = match.group(1).strip()
    secondary = match.group(2).strip()
    if re.search(r"[\u4e00-\u9fff]", main) and re.search(r"[A-Za-z]", secondary):
        return main, secondary
    return title, None


def strip_leading_h1(body_html: str) -> str:
    return re.sub(r'^\s*<h1\b[^>]*>.*?</h1>\s*', "", body_html, count=1, flags=re.DOTALL)


def split_leading_symbol(label: str) -> tuple[str | None, str]:
    match = re.match(r"^\s*([^\w\s]+)\s+(.+)$", label, flags=re.UNICODE)
    if not match:
        return None, label
    return match.group(1), match.group(2).strip()


def render_toc(headings: tuple[RenderedHeading, ...]) -> str:
    entries = [(h.level, strip_decorative_prefix(h.text), f"#{h.anchor}") for h in headings if h.level in (2, 3)]
    if not entries:
        return "<p style='color:var(--muted); font-size:0.85rem;'>No subsection anchors.</p>"
    items: list[str] = []
    for level, label, href in entries:
        label_html = f'<span class="toc-label">{html.escape(label)}</span>'
        items.append(f'<li class="toc-level-{level}"><a href="{html.escape(href)}">{label_html}</a></li>')
    return '<ul class="toc-list">' + "".join(items) + "</ul>"


def render_section_index(
    headings: tuple[RenderedHeading, ...],
    *,
    title: str,
    summary: str = "",
    levels: tuple[int, ...] = (2,),
    min_entries: int = 1,
    variant: str = "grid",
) -> str:
    entries = [
        (strip_decorative_prefix(heading.text), f"#{heading.anchor}")
        for heading in headings
        if heading.level in levels
    ]
    if not entries or len(entries) < min_entries:
        return ""
    intro_html = f'<p class="section-index-summary">{html.escape(summary)}</p>' if summary else ""
    items_html = "".join(
        f'<li><a href="{html.escape(href)}">{html.escape(label)}</a></li>'
        for label, href in entries
    )
    return (
        '<section class="section-index">'
        f'<h2>{html.escape(title)}</h2>'
        f"{intro_html}"
        f'<ul class="section-index-list layout-{html.escape(variant)}">{items_html}</ul>'
        "</section>"
    )


def prefixed_output_rel(config: HtmlBuildConfig, rel: str) -> str:
    return f"{config.output_prefix.strip('/')}/{rel}" if config.output_prefix else rel


def render_version_links(output_rel: str, config: HtmlBuildConfig) -> str:
    items = []
    if config.version_label:
        items.append((f"Current: {config.version_label}", "#"))
    for link in config.version_links:
        items.append((link.label, relative_href(output_rel, link.target_rel)))
    return html_list(items) if items else '<p class="sidebar-meta-text">Current checkout</p>'
