from __future__ import annotations

import html

from docgen_command_docs import render_markdown_document
from docgen_common import relative_href
from docgen_handbook import HANDBOOK_NAV_GROUP_LABELS, HANDBOOK_NAV_TITLES, LOCALE_LABELS, handbook_language_href
from docsite_html_common import (
    prefixed_output_rel,
    render_section_index,
    render_toc,
    render_version_links,
    strip_heading_decorations,
    strip_leading_h1,
    title_only,
)
from docsite_html_links import rewrite_markdown_link
from docsite_html_nav import handbook_surface_label, render_command_map_nav, render_global_nav, render_jump_select, render_page_locale_select
from docsite_html_page_shell import page_shell
from docsite_handbook_render import HANDBOOK_RENDER_LOCALE_STRINGS, handbook_render_policy


def render_footer_nav(prev, nxt, *, locale: str = "en"):
    cards = []
    locale_strings = HANDBOOK_RENDER_LOCALE_STRINGS[locale]
    previous_label = locale_strings["previous_chapter"]
    next_label = locale_strings["next_chapter"]
    if prev:
        cards.append(f'<a class="link-card" href="{html.escape(prev[1])}"><span>{html.escape(previous_label)}</span>{html.escape(prev[0])}</a>')
    if nxt:
        cards.append(f'<a class="link-card" href="{html.escape(nxt[1])}"><span>{html.escape(next_label)}</span>{html.escape(nxt[0])}</a>')
    return '<nav class="footer-nav">' + "".join(cards) + "</nav>" if cards else ""


def handbook_page_eyebrow(page) -> str:
    return HANDBOOK_NAV_GROUP_LABELS[page.locale][page.part_key]


def handbook_page_nav_title(title: str) -> str:
    return title_only(title)


def render_language_links(current_label, switch_label, switch_href):
    items = [(f"Current: {current_label}", "#")]
    if switch_label and switch_href:
        items.append((f"Switch to {switch_label}", switch_href))
    from docsite_html_common import html_list
    return html_list(items)


def render_developer_page(config):
    dev_path = config.source_root / "docs" / "DEVELOPER.md"
    if not dev_path.exists():
        return ""
    doc = render_markdown_document(dev_path.read_text(encoding="utf-8"), link_transform=lambda target: target)
    section_index = render_section_index(doc.headings, title="Guide Map", summary="Jump straight to the repo concern you are touching instead of scanning the whole maintainer guide.")
    body_html = section_index + strip_heading_decorations(strip_leading_h1(doc.body_html))
    return page_shell(page_title="Developer Guide", html_lang="en", home_href=relative_href("developer.html", prefixed_output_rel(config, "index.html")), hero_title="Developer Guide", hero_summary="Maintainer routing for runtime, docs, contracts, and release work.", breadcrumbs=[("Home", "index.html"), ("Developer Guide", None)], body_html=body_html, toc_html=render_toc(doc.headings), related_html="", version_html="", locale_html="", footer_nav_html="", footer_html="Source: docs/DEVELOPER.md", jump_html=render_page_locale_select("English") + render_jump_select("developer.html", "en", config), nav_html="")


def render_handbook_page(page, config):
    doc = render_markdown_document(page.source_path.read_text(encoding="utf-8"), link_transform=lambda target: rewrite_markdown_link(page.source_path, page.output_rel, target, config))
    full_title = title_only(doc.title or page.title)
    nav_title = HANDBOOK_NAV_TITLES.get(page.locale, {}).get(page.stem, full_title)
    render_policy = handbook_render_policy(page.stem)
    locale_strings = HANDBOOK_RENDER_LOCALE_STRINGS[page.locale]
    locale_href = handbook_language_href(page)
    locale_label = LOCALE_LABELS["zh-TW" if page.locale == "en" else "en"] if locale_href else None
    prev = (handbook_page_nav_title(page.previous_title), relative_href(page.output_rel, page.previous_output_rel)) if page.previous_output_rel else None
    nxt = (handbook_page_nav_title(page.next_title), relative_href(page.output_rel, page.next_output_rel)) if page.next_output_rel else None
    toc_headings = doc.headings
    section_index = render_section_index(
        toc_headings,
        title=locale_strings["section_index_title"],
        summary=locale_strings["section_index_summary"],
        levels=render_policy.section_index_levels,
        min_entries=render_policy.section_index_min_entries,
        variant=render_policy.section_index_variant,
    )
    body_html = (
        (section_index if render_policy.show_section_index else "")
        + strip_heading_decorations(strip_leading_h1(doc.body_html))
    )
    return page_shell(
        page_title=full_title,
        html_lang=page.locale,
        home_href=relative_href(page.output_rel, prefixed_output_rel(config, "index.html")),
        hero_title=nav_title,
        hero_subtitle=full_title if nav_title != full_title else "",
        hero_summary="",
        breadcrumbs=[("Home", relative_href(page.output_rel, "index.html")), (handbook_surface_label(page.locale), None), (nav_title, None)],
        body_html=body_html,
        toc_html=render_toc(toc_headings),
        related_html="",
        version_html=render_version_links(page.output_rel, config),
        locale_html=render_language_links(LOCALE_LABELS[page.locale], locale_label, locale_href),
        footer_nav_html=render_footer_nav(prev, nxt, locale=page.locale),
        footer_html="Source: " + page.source_path.name,
        jump_html=render_page_locale_select(LOCALE_LABELS[page.locale], locale_label, locale_href) + render_jump_select(page.output_rel, page.locale, config),
        nav_html=render_global_nav(page.output_rel, page.locale, config, handbook_stem=page.stem),
        hero_eyebrow=handbook_page_eyebrow(page),
    )
