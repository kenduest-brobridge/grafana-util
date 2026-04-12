from __future__ import annotations

import html

from docgen_command_docs import RenderedHeading
from docgen_common import relative_href
from docgen_entrypoints import HANDBOOK_COMMAND_MAPS
from docsite_html_common import prefixed_output_rel, render_template
from docsite_html_nav_handbook import handbook_group_for_stem, handbook_nav_groups
from docsite_ui_text import ui_text


def command_reference_label(locale: str) -> str:
    return ui_text(locale, "command_reference_label")


def command_tokens(command: str) -> tuple[str, ...]:
    parts = command.split()
    if parts[:1] == ["grafana-util"]:
        parts = parts[1:]
    return tuple(parts)


def command_label(tokens: tuple[str, ...]) -> str:
    return " ".join(tokens).strip()


def shared_command_prefix(token_sets: tuple[tuple[str, ...], ...]) -> tuple[str, ...]:
    if not token_sets:
        return ()
    if len(token_sets) == 1:
        tokens = token_sets[0]
        return tokens[:-1] if len(tokens) >= 2 else ()
    shortest = min(len(tokens) for tokens in token_sets)
    prefix: list[str] = []
    for index in range(shortest):
        candidate = token_sets[0][index]
        if all(tokens[index] == candidate for tokens in token_sets[1:]):
            prefix.append(candidate)
            continue
        break
    return tuple(prefix)


def render_command_map_links(output_rel: str, locale: str, links, config) -> str:
    token_sets = tuple(command_tokens(link.command) for link in links)
    prefix = shared_command_prefix(token_sets)
    if prefix:
        root_target = next((link.target for link, tokens in zip(links, token_sets) if tokens == prefix), None)
        root_label = command_label(prefix)
        if root_target is not None:
            root_href = relative_href(output_rel, prefixed_output_rel(config, f"commands/{locale}/{root_target}"))
            root_html = f'<a href="{html.escape(root_href)}" class="nav-command-root-link">{html.escape(root_label)}</a>'
        else:
            root_html = f'<span class="nav-command-root-link nav-command-root-static">{html.escape(root_label)}</span>'
        child_items: list[str] = []
        for link, tokens in zip(links, token_sets):
            if tokens == prefix:
                continue
            tail = command_label(tokens[len(prefix):]) or command_label(tokens)
            href = relative_href(output_rel, prefixed_output_rel(config, f"commands/{locale}/{link.target}"))
            child_items.append(f'<li class="nav-command-branch-item"><a href="{html.escape(href)}" class="nav-command-leaf-link">{html.escape(tail)}</a></li>')
        children_html = f'<ul class="nav-command-branch-list">{"".join(child_items)}</ul>' if child_items else ""
        return '<div class="nav-command-branch">' f'<div class="nav-command-root">{root_html}</div>' f"{children_html}" "</div>"

    items_html = []
    for link, tokens in zip(links, token_sets):
        label = command_label(tokens)
        href = relative_href(output_rel, prefixed_output_rel(config, f"commands/{locale}/{link.target}"))
        items_html.append(f'<li class="nav-command-link"><a href="{html.escape(href)}" class="nav-command-leaf-link">{html.escape(label)}</a></li>')
    return f'<ul class="nav-command-tree">{"".join(items_html)}</ul>'


def render_command_map_nav(output_rel: str, locale: str, handbook_stem: str | None, config, *, compact: bool = False) -> str:
    if handbook_stem is None:
        return ""
    groups = HANDBOOK_COMMAND_MAPS.get(handbook_stem)
    if not groups:
        return ""
    section_title = ui_text(locale, "command_relationships_title")
    items_html: list[str] = []
    for index, group in enumerate(groups):
        collapsed = compact and index > 0
        expanded_attr = "false" if collapsed else "true"
        collapsed_class = " collapsed" if collapsed else ""
        items_html.append(
            '<div class="nav-group nav-command-group'
            f'{collapsed_class}">'
            f'<button class="nav-group-header" type="button" aria-expanded="{expanded_attr}">'
            f'<span class="nav-group-title-text">{html.escape(group.title_for(locale))}</span>'
            '<span class="nav-group-caret" aria-hidden="true">▾</span>'
            '</button>'
            f'<div class="nav-sub-list nav-command-sub-list">{render_command_map_links(output_rel, locale, group.links, config)}</div>'
            '</div>'
        )
    if compact:
        return (
            '<section class="nav-section nav-command-section collapsed">'
            f'<button class="nav-group-header nav-section-toggle" type="button" aria-expanded="false">'
            f'<span class="nav-group-title-text">{html.escape(section_title)}</span>'
            '<span class="nav-group-caret" aria-hidden="true">▾</span>'
            '</button>'
            f'<div class="nav-section-body">{"".join(items_html)}</div>'
            '</section>'
        )
    return (
        '<section class="handbook-command-map">'
        f'<h2 id="command-relationships">{html.escape(section_title)}</h2>'
        f'<div class="handbook-command-map-panel">{"".join(items_html)}</div>'
        "</section>"
    )


def render_command_map_heading(locale: str, handbook_stem: str | None) -> tuple[RenderedHeading, ...]:
    if handbook_stem is None or handbook_stem not in HANDBOOK_COMMAND_MAPS:
        return ()
    return (
        RenderedHeading(
            level=2,
            text="Command Relationships" if locale == "en" else "指令關係",
            anchor="command-relationships",
        ),
    )


def render_global_nav(output_rel: str, locale: str, config, *, handbook_stem: str | None = None) -> str:
    if not output_rel or "index.html" in output_rel and "/" not in output_rel.replace("index.html", ""):
        return ""
    handbook_label = "Guide Map" if locale == "en" else "手冊導覽"
    commands_label = command_reference_label(locale)
    sections = []
    active_group = handbook_group_for_stem(handbook_stem) if handbook_stem else None
    group_html: list[str] = []
    for group_key, group_label, items in handbook_nav_groups(locale, config):
        collapsed = active_group is not None and group_key != active_group
        expanded_attr = "false" if collapsed else "true"
        collapsed_class = " collapsed" if collapsed else ""
        group_items: list[str] = []
        for _, label, target in items:
            href = relative_href(output_rel, target)
            active = " active" if output_rel == target else ""
            group_items.append(f'<li class="nav-tree-node"><a href="{html.escape(href)}" class="nav-item{active}">{html.escape(label)}</a></li>')
        group_html.append(
            '<div class="nav-group'
            f'{collapsed_class}">'
            f'<button class="nav-group-header" type="button" aria-expanded="{expanded_attr}">'
            f'<span class="nav-group-title-text">{html.escape(group_label)}</span>'
            '<span class="nav-group-caret" aria-hidden="true">▾</span>'
            '</button>'
            f'<ul class="nav-sub-list">{"".join(group_items)}</ul>'
            '</div>'
        )
    sections.append(f'<section class="nav-section"><h2>{handbook_label}</h2>{"".join(group_html)}</section>')
    command_map_html = render_command_map_nav(output_rel, locale, handbook_stem, config, compact=True)
    if command_map_html:
        sections.append(command_map_html)
    command_target = prefixed_output_rel(config, f"commands/{locale}/index.html")
    command_href = relative_href(output_rel, command_target)
    command_active = " active" if output_rel == command_target or f"commands/{locale}/" in output_rel else ""
    command_label_text = ui_text(locale, "open_command_reference_label")
    sections.append(f'<section class="nav-section"><h2>{commands_label}</h2><a href="{html.escape(command_href)}" class="nav-command-entry{command_active}">{html.escape(command_label_text)}</a></section>')
    return render_template("nav_sidebar.html.tmpl", sections_html="".join(sections))
