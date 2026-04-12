from __future__ import annotations

import html
from pathlib import Path

from docgen_common import relative_href
from docgen_entrypoints import JUMP_COMMAND_ENTRIES
from docgen_handbook import existing_handbook_files
from docsite_html_common import prefixed_output_rel
from docsite_html_nav_command import command_reference_label
from docsite_html_nav_handbook import handbook_surface_label, handbook_nav_titles
from docsite_ui_text import ui_text


def render_jump_select_options(output_rel: str, locale: str, config) -> str:
    handbook_label = handbook_surface_label(locale)
    commands_label = command_reference_label(locale)
    prompt_label = ui_text(locale, "jump_prompt_label")
    titles = handbook_nav_titles(locale, config)
    sections = [f'<option value="" selected>{html.escape(prompt_label)}</option>', f'<optgroup label="{handbook_label}">']
    for name in existing_handbook_files(locale, handbook_root=config.handbook_root):
        stem = Path(name).stem
        label = titles.get(stem, stem.replace("-", " ").title() if stem != "index" else ("Overview" if locale == "en" else "概觀"))
        target = relative_href(output_rel, prefixed_output_rel(config, f"handbook/{locale}/{stem}.html"))
        sections.append(f'<option value="{html.escape(target)}">{html.escape(label)}</option>')
    sections.append(f'</optgroup><optgroup label="{commands_label}">')
    for entry in JUMP_COMMAND_ENTRIES[locale]:
        target = relative_href(output_rel, prefixed_output_rel(config, entry.target))
        sections.append(f'<option value="{html.escape(target)}">{html.escape(entry.label)}</option>')
    sections.append("</optgroup>")
    return "".join(sections)


def render_jump_select(output_rel: str, locale: str, config) -> str:
    return f'<select id="jump-select" aria-label="Jump">{render_jump_select_options(output_rel, locale, config)}</select>'


def render_landing_locale_select(current_locale: str = "en") -> str:
    selected_auto = ' selected' if current_locale == "auto" else ""
    selected_en = ' selected' if current_locale == "en" else ""
    selected_zh = ' selected' if current_locale == "zh-TW" else ""
    return (
        '<select id="locale-select" aria-label="Language">'
        f'<option value="auto"{selected_auto}>Auto</option>'
        f'<option value="en"{selected_en}>English</option>'
        f'<option value="zh-TW"{selected_zh}>繁體中文</option>'
        "</select>"
    )


def render_page_locale_select(current_label: str, switch_label: str | None = None, switch_href: str | None = None) -> str:
    options = [f'<option value="" selected>{html.escape(ui_text("en", "page_locale_current_prefix"))}{html.escape(current_label)}</option>']
    if switch_label and switch_href:
        options.append(f'<option value="{html.escape(switch_href)}">{html.escape(ui_text("en", "page_locale_switch_prefix"))}{html.escape(switch_label)}</option>')
    return f'<select id="page-locale-select" aria-label="Language switch">{"".join(options)}</select>'
