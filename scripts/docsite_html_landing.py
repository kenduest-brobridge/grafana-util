from __future__ import annotations

import html
import json
from pathlib import Path

from docgen_common import relative_href
from docgen_entrypoints import QUICK_COMMANDS
from docgen_landing import LANDING_LOCALES, LANDING_UI_LABELS, LandingLink, LandingSection, LandingTask, load_landing_page
from docsite_html_common import prefixed_output_rel, render_template
from docsite_html_nav import render_jump_select, render_jump_select_options, render_landing_locale_select
from docsite_html_page_shell import page_shell


def landing_panel_html(title: str, summary: str, links: list[tuple[str, str]]) -> str:
    links_html = "".join(f'<li><a href="{html.escape(href)}">{html.escape(label)}</a></li>' for label, href in links)
    return render_template("landing_panel.html.tmpl", title=html.escape(title), summary=html.escape(summary), links_html=links_html)


def quick_commands_panel_html(locale: str, landing_rel: str, config) -> str:
    title = "First-Run Path" if locale == "en" else "第一次執行路線"
    summary = "Use this as the shortest safe starting path, then continue into the handbook or command reference for the full workflow." if locale == "en" else "把這組命令當作最短的安全起手路線；需要完整流程時，再往手冊或指令參考繼續。"
    links = [(f"{item.label}: {item.command}", relative_href(landing_rel, prefixed_output_rel(config, item.target))) for item in QUICK_COMMANDS[locale]]
    return landing_panel_html(title, summary, links)


def landing_link_is_available(source_path: Path, target: str, config) -> bool:
    if target.startswith(("http", "mailto:", "#")):
        return True
    bare = target.split("#", 1)[0]
    resolved = (source_path.parent / bare).resolve()
    try:
        relative = resolved.relative_to(config.source_root / "docs")
    except Exception:
        return True
    relative_path = Path(relative)
    if relative_path.parts[:1] == ("user-guide",) and relative_path.suffix == ".md":
        return config.handbook_root.joinpath(*relative_path.parts[1:]).exists()
    if relative_path.parts[:1] == ("commands",) and relative_path.suffix == ".md":
        return config.command_docs_root.joinpath(*relative_path.parts[1:]).exists()
    return True


def render_landing_links(source_path: Path, output_rel: str, links: tuple[LandingLink, ...], config, rewrite_markdown_link) -> list[tuple[str, str]]:
    rendered: list[tuple[str, str]] = []
    for link in links:
        if not landing_link_is_available(source_path, link.target, config):
            continue
        rendered.append((link.label, rewrite_markdown_link(source_path, output_rel, link.target, config)))
    return rendered


def render_landing_task(source_path: Path, output_rel: str, task: LandingTask, config, rewrite_markdown_link) -> str:
    rendered_links = render_landing_links(source_path, output_rel, task.links, config, rewrite_markdown_link)
    links_html = "".join(f'<li><a href="{html.escape(href)}">{html.escape(label)}</a></li>' for label, href in rendered_links)
    return render_template("landing_task.html.tmpl", title=html.escape(task.title), summary=html.escape(task.summary), links_html=links_html)


def render_landing_section(source_path: Path, output_rel: str, section: LandingSection, config, rewrite_markdown_link) -> str:
    inline_html = ""
    inline_links = render_landing_links(source_path, output_rel, section.links, config, rewrite_markdown_link)
    if inline_links:
        inline_html = '<ul class="landing-inline-links">' + "".join(f'<li><a href="{html.escape(href)}">{html.escape(label)}</a></li>' for label, href in inline_links) + "</ul>"
    return render_template("landing_section.html.tmpl", title=html.escape(section.title), summary=html.escape(section.summary), inline_html=inline_html, tasks_html="".join(render_landing_task(source_path, output_rel, task, config, rewrite_markdown_link) for task in section.tasks))


def build_landing_locale_data(config, rewrite_markdown_link) -> dict[str, dict[str, str]]:
    landing_rel = prefixed_output_rel(config, "index.html")
    landing_root = config.source_root / "docs" / "landing"
    version_links = [(config.version_label or config.version, "#")]
    version_links.extend((link.label, relative_href(landing_rel, link.target_rel)) for link in config.version_links)
    if config.include_raw_manpages or config.raw_manpage_target_rel:
        version_links.append(("Manpages", relative_href(landing_rel, prefixed_output_rel(config, "man/index.html"))))
    landing_data: dict[str, dict[str, str]] = {}
    for locale in LANDING_LOCALES:
        page = load_landing_page(locale, landing_root=landing_root)
        ui_labels = LANDING_UI_LABELS[locale]
        maintainer_links = render_landing_links(page.source_path, landing_rel, page.maintainer.links, config, rewrite_markdown_link)
        meta_html = "".join((
            quick_commands_panel_html(locale, landing_rel, config),
            landing_panel_html(page.maintainer.title, page.maintainer.summary, maintainer_links),
            landing_panel_html("Version" if locale == "en" else "版本", "Version switching is secondary here. Pick a language first, then jump release context if you need it." if locale == "en" else "版本切換是次要操作。先選語言，再視需要跳去特定版本內容。", version_links),
        ))
        landing_data[locale] = {
            "lang": locale,
            "eyebrow": ui_labels["eyebrow"],
            "hero_title": page.title,
            "hero_summary": page.summary,
            "search_heading": page.search.title,
            "search_copy": page.search.summary,
            "search_placeholder": ui_labels["search_placeholder"],
            "search_button": ui_labels["search_button"],
            "sections_html": "".join(render_landing_section(page.source_path, landing_rel, section, config, rewrite_markdown_link) for section in page.sections),
            "meta_html": meta_html,
            "jump_options_html": render_jump_select_options(landing_rel, locale, config),
        }
    return landing_data


def render_landing_page(config, rewrite_markdown_link):
    landing_data = build_landing_locale_data(config, rewrite_markdown_link)
    copy = landing_data["en"]
    body = f"""
<div class="landing-page">
  <section class="landing-hero">
    <div class="landing-hero-inner">
      <div id="landing-eyebrow" class="landing-eyebrow">{html.escape(copy["eyebrow"])}</div>
      <h1 id="landing-title" class="landing-title">{html.escape(copy["hero_title"])}</h1>
      <p id="landing-summary" class="landing-summary">{html.escape(copy["hero_summary"])}</p>
    </div>
    <section class="landing-search-panel">
      <h2 id="landing-search-heading">{html.escape(copy["search_heading"])}</h2>
      <p id="landing-search-copy">{html.escape(copy["search_copy"])}</p>
      <form id="landing-search-form" class="landing-search-form">
        <input id="landing-search" class="landing-search-input" type="search" placeholder="{html.escape(copy['search_placeholder'])}" aria-label="{html.escape(copy['search_placeholder'])}" />
        <button id="landing-search-button" class="landing-search-button" type="submit">{html.escape(copy["search_button"])}</button>
      </form>
    </section>
  </section>
  <div id="landing-sections" class="landing-sections">{copy["sections_html"]}</div>
  <div id="landing-meta" class="landing-meta">{copy["meta_html"]}</div>
  <script id="landing-i18n" type="application/json">{json.dumps(landing_data, ensure_ascii=False)}</script>
</div>
"""
    return page_shell(page_title="Grafana Documentation", html_lang="en", home_href=relative_href(prefixed_output_rel(config, "index.html"), prefixed_output_rel(config, "index.html")), hero_title="", hero_summary="", breadcrumbs=[], body_html=body, toc_html="", related_html="", version_html="", locale_html="", footer_nav_html="", footer_html="Generated by scripts/generate_command_html.py", jump_html=render_landing_locale_select("auto") + render_jump_select(prefixed_output_rel(config, "index.html"), "en", config), nav_html="", is_landing=True)
