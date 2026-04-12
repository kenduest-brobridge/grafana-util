from __future__ import annotations

import html
from pathlib import Path

from docgen_command_docs import command_doc_cli_path, parse_command_page, render_markdown_document
from docgen_common import relative_href
from docsite_command_index import render_command_reference_index
from docsite_command_surface import command_audience_hint, command_breadcrumb_parents, command_reference_links
from docgen_handbook import LOCALE_LABELS
from docsite_html_common import (
    html_list,
    prefixed_output_rel,
    render_template,
    render_toc,
    render_version_links,
    strip_heading_decorations,
    strip_leading_h1,
    title_only,
)
from docsite_html_links import command_handbook_context, rewrite_markdown_link
from docsite_html_nav import command_reference_label, render_global_nav, render_jump_select, render_page_locale_select
from docsite_html_page_shell import page_shell


def command_intro_labels(locale: str) -> dict[str, str]:
    if locale == "zh-TW":
        return {"what": "這頁在說什麼", "when": "什麼時候看這頁", "who": "適合誰"}
    return {"what": "What this page covers", "when": "When to open this page", "who": "Who this page is for"}
def render_command_intro(locale: str, source_path: Path) -> str:
    cli_path = command_doc_cli_path(source_path, "grafana-util " + source_path.stem.replace("-", " "))
    if cli_path is None:
        return ""
    try:
        parsed = parse_command_page(source_path, cli_path)
    except Exception:
        return ""
    labels = command_intro_labels(locale)
    blocks: list[str] = []
    if parsed.purpose:
        blocks.append(render_template("command_intro_block.html.tmpl", label=html.escape(labels["what"]), content_html=f'<p class="command-intro-text">{html.escape(parsed.purpose)}</p>'))
    when_html = ""
    if parsed.when_lines:
        items = "".join(f"<li>{html.escape(line.removeprefix('- ').strip())}</li>" for line in parsed.when_lines)
        when_html = f'<ul class="command-intro-list">{items}</ul>'
    elif parsed.when:
        when_html = f'<p class="command-intro-text">{html.escape(parsed.when)}</p>'
    if when_html:
        blocks.append(render_template("command_intro_block.html.tmpl", label=html.escape(labels["when"]), content_html=when_html))
    audience = command_audience_hint(locale, source_path)
    if audience:
        blocks.append(render_template("command_intro_block.html.tmpl", label=html.escape(labels["who"]), content_html=f'<p class="command-intro-text">{html.escape(audience)}</p>'))
    if not blocks:
        return ""
    return render_template("command_intro.html.tmpl", blocks_html="".join(blocks))


def command_language_switch_href(output_rel, locale, source_name, config):
    other = "zh-TW" if locale == "en" else "en"
    target = config.command_docs_root / other / source_name
    if not target.exists():
        return None, None
    return LOCALE_LABELS[other], relative_href(output_rel, prefixed_output_rel(config, f"commands/{other}/{Path(source_name).with_suffix('.html').as_posix()}"))


def command_breadcrumb_label(locale: str) -> str:
    return command_reference_label(locale)


def render_language_links(current_label, switch_label, switch_href):
    items = [(f"Current: {current_label}", "#")]
    if switch_label and switch_href:
        items.append((f"Switch to {switch_label}", switch_href))
    return html_list(items)


def render_command_page(locale, source_path, output_rel, config):
    if source_path.stem == "index":
        doc = render_command_reference_index(locale, link_transform=lambda target: rewrite_markdown_link(source_path, output_rel, target, config))
    else:
        doc = render_markdown_document(source_path.read_text(encoding="utf-8"), link_transform=lambda target: rewrite_markdown_link(source_path, output_rel, target, config))
    title = title_only(doc.title or source_path.stem)
    switch_label, switch_href = command_language_switch_href(output_rel, locale, source_path.name, config)
    handbook_link = command_handbook_context(locale, output_rel, source_name=source_path.name, config=config)
    related_items = command_reference_links(locale, output_rel, source_path, config)
    if handbook_link:
        related_items.append(handbook_link)
    body_html = render_command_intro(locale, source_path) + strip_heading_decorations(strip_leading_h1(doc.body_html))
    command_index_href = relative_href(output_rel, prefixed_output_rel(config, f"commands/{locale}/index.html"))
    breadcrumbs = [("Home", relative_href(output_rel, prefixed_output_rel(config, "index.html"))), (command_breadcrumb_label(locale), command_index_href)]
    for label, href in command_breadcrumb_parents(locale, output_rel, source_path, config):
        breadcrumbs.append((label, href))
    if source_path.stem != "index":
        breadcrumbs.append((title, None))
    return page_shell(page_title=title, html_lang=locale, home_href=relative_href(output_rel, prefixed_output_rel(config, "index.html")), hero_title=title, hero_summary="", breadcrumbs=breadcrumbs, body_html=body_html, toc_html=render_toc(doc.headings), related_html=html_list(related_items) if related_items else "", version_html=render_version_links(output_rel, config), locale_html=render_language_links(LOCALE_LABELS[locale], switch_label, switch_href), footer_nav_html="", footer_html="Source: " + source_path.name, jump_html=render_page_locale_select(LOCALE_LABELS[locale], switch_label, switch_href) + render_jump_select(output_rel, locale, config), nav_html=render_global_nav(output_rel, locale, config))
