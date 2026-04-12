from __future__ import annotations

import html

from docsite_html_common import (
    PAGE_STYLE,
    THEME_SCRIPT,
    render_breadcrumbs,
    render_template,
    split_display_title,
)
from docsite_ui_text import ui_text


def page_shell_sidebar_labels(html_lang: str) -> dict[str, str]:
    labels = ui_text(html_lang, "sidebar_labels")
    if not isinstance(labels, dict):
        raise TypeError(f"sidebar_labels must be a map for locale {html_lang}")
    return {key: str(value) for key, value in labels.items()}


def page_shell(*, page_title, html_lang, home_href, hero_title, hero_summary, breadcrumbs, body_html, toc_html, related_html, version_html, locale_html, footer_nav_html, footer_html, jump_html="", nav_html="", is_landing=False, hero_eyebrow="", hero_subtitle=""):
    header_html = ""
    if hero_title and not is_landing:
        eyebrow_html = f'<div class="hero-eyebrow">{html.escape(hero_eyebrow)}</div>' if hero_eyebrow else ""
        subtitle_html = f'<p class="hero-subtitle">{html.escape(hero_subtitle)}</p>' if hero_subtitle else ""
        summary_html = f'<p class="hero-summary">{hero_summary}</p>' if hero_summary else ""
        title_main, title_secondary = split_display_title(hero_title)
        title_html = f'<span class="hero-title-main">{html.escape(title_main)}</span>'
        if title_secondary:
            title_html += f'<span class="hero-title-secondary">{html.escape(title_secondary)}</span>'
        header_html = render_template("page_header.html.tmpl", eyebrow_html=eyebrow_html, title_html=title_html, subtitle_html=subtitle_html, summary_html=summary_html)

    sidebar_html = ""
    if not is_landing:
        sidebar_labels = page_shell_sidebar_labels(html_lang)
        sidebar_sections: list[str] = []
        if toc_html:
            sidebar_sections.append(f'<section class="sidebar-section"><h2>{sidebar_labels["toc"]}</h2>{toc_html}</section>')
        if related_html:
            sidebar_sections.append(f'<section class="sidebar-section"><h2>{sidebar_labels["related"]}</h2>{related_html}</section>')
        if version_html:
            sidebar_sections.append(f'<section class="sidebar-section"><h2>{sidebar_labels["version"]}</h2>{version_html}</section>')
        if locale_html:
            sidebar_sections.append(f'<section class="sidebar-section"><h2>{sidebar_labels["language"]}</h2>{locale_html}</section>')
        if sidebar_sections:
            sidebar_html = render_template("right_sidebar.html.tmpl", sections_html="".join(sidebar_sections))

    if is_landing:
        content = body_html
    else:
        layout_class = "layout"
        if not nav_html:
            layout_class += " layout-no-nav"
        if not sidebar_html:
            layout_class += " layout-no-sidebar"
        content = render_template(
            "article_layout.html.tmpl",
            layout_class=layout_class,
            nav_html=nav_html,
            breadcrumbs_html=render_breadcrumbs(breadcrumbs),
            header_html=header_html,
            body_html=body_html,
            footer_nav_html=footer_nav_html,
            sidebar_html=sidebar_html,
        )

    topbar_html = render_template("topbar.html.tmpl", home_href=html.escape(home_href), controls_html=jump_html)
    return render_template(
        "base.html.tmpl",
        html_lang=html.escape(html_lang),
        page_title=html.escape(page_title),
        page_style=PAGE_STYLE,
        landing_shell_class=" landing-shell" if is_landing else "",
        topbar_html=topbar_html,
        content=content,
        footer_html=footer_html,
        theme_script=THEME_SCRIPT,
    )
