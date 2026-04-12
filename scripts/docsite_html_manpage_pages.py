from __future__ import annotations

import html
from pathlib import Path

from docgen_common import relative_href
from docsite_html_common import html_list, prefixed_output_rel, render_template, render_version_links
from docsite_html_nav import render_jump_select
from docsite_html_page_shell import page_shell
from docsite_html_roff import render_roff_manpage_html
from generate_manpages import NAMESPACE_SPECS


def command_reference_root_for_stem(stem, config):
    if stem == "grafana-util":
        return prefixed_output_rel(config, "commands/en/index.html")
    for spec in NAMESPACE_SPECS:
        if spec.stem == stem:
            return prefixed_output_rel(config, f"commands/en/{spec.root_doc[:-3]}.html")
    return None


def manpage_label(name):
    if not name.endswith(".1"):
        return name
    return f"{name[:-2]}(1)"


def manpage_group(name):
    stem = Path(name).stem
    if stem == "grafana-util":
        return ("overview", "Overview")
    prefix = "grafana-util-"
    if not stem.startswith(prefix):
        return ("other", "Other")
    family = stem.removeprefix(prefix).split("-", 1)[0]
    return (family, family)


def manpage_group_item_label(name, group_key):
    stem = Path(name).stem
    if group_key == "overview":
        return manpage_label(name)
    root_stem = f"grafana-util-{group_key}"
    child_prefix = root_stem + "-"
    if stem == root_stem:
        return f"{group_key}(1)"
    if stem.startswith(child_prefix):
        return f"{stem.removeprefix(child_prefix).replace('-', ' ')}(1)"
    return manpage_label(name)


def render_manpage_index_nav(output_rel, config, names, *, current_name=None):
    if not names:
        return ""
    grouped: dict[str, tuple[str, list[str]]] = {}
    for name in names:
        key, label = manpage_group(name)
        grouped.setdefault(key, (label, []))[1].append(name)
    group_order = ["overview", "access", "alert", "config", "dashboard", "datasource", "status", "workspace", "export", "version", "other"]
    ordered_keys = [key for key in group_order if key in grouped] + sorted(key for key in grouped if key not in group_order)
    groups_html: list[str] = []
    for key in ordered_keys:
        label, group_names = grouped[key]
        collapsed = bool(current_name) and current_name not in group_names
        expanded_attr = "false" if collapsed else "true"
        collapsed_class = " collapsed" if collapsed else ""
        items: list[str] = []
        for name in group_names:
            if name == current_name:
                continue
            href = relative_href(output_rel, prefixed_output_rel(config, f"man/{Path(name).stem}.html"))
            items.append(f'<li class="nav-tree-node"><a href="{html.escape(href)}" class="nav-item nav-manpage-item">{html.escape(manpage_group_item_label(name, key))}</a></li>')
        if current_name in group_names and not items:
            collapsed_class += " nav-manpage-group-current-only"
        groups_html.append(
            '<div class="nav-group nav-manpage-group'
            f'{collapsed_class}">'
            f'<button class="nav-group-header" type="button" aria-expanded="{expanded_attr}">'
            f'<span class="nav-group-title-text">{html.escape(label)} <span class="nav-manpage-count">{len(group_names)}</span></span>'
            '<span class="nav-group-caret" aria-hidden="true">▸</span>'
            '</button>'
            f'<ul class="nav-sub-list nav-manpage-list">{"".join(items)}</ul>'
            '</div>'
        )
    return "".join(groups_html)


def render_manpage_nav(output_rel, config, *, names=(), current_name=None, command_root=None):
    command_index = prefixed_output_rel(config, "commands/en/index.html")
    handbook_index = prefixed_output_rel(config, "handbook/en/index.html")
    site_home = prefixed_output_rel(config, "index.html")

    def nav_link(label, target, *, active=False):
        href = relative_href(output_rel, target)
        active_class = " active" if active else ""
        return f'<a href="{html.escape(href)}" class="nav-command-entry{active_class}">{html.escape(label)}</a>'

    docs_items = [
        nav_link("Documentation home", site_home),
        nav_link("Command reference", command_index),
        nav_link("English handbook", handbook_index),
    ]
    if command_root and command_root != command_index:
        docs_items.append(nav_link("Matching command page", command_root))

    current_html = ""
    if current_name:
        current_html = f'<span class="nav-command-entry nav-manpage-current active">{html.escape(manpage_label(current_name))}</span>'

    sections = [
        '<section class="nav-section"><h2>Manual Pages</h2>' + current_html + render_manpage_index_nav(output_rel, config, names, current_name=current_name) + "</section>",
        '<section class="nav-section"><h2>Documentation</h2>' + "".join(docs_items) + "</section>",
    ]
    return render_template("nav_sidebar.html.tmpl", sections_html="".join(sections))


def render_manpage_index_page(output_rel, names, config):
    body = (
        "<p>This lane mirrors the checked-in generated manpages as browser-readable HTML for local browsing and GitHub Pages.</p>"
        + "<ul>"
        + "".join(f'<li><a href="{html.escape(relative_href(output_rel, prefixed_output_rel(config, f"man/{Path(name).stem}.html")))}">{html.escape(name)}</a></li>' for name in names)
        + "</ul>"
    )
    related = [
        ("English handbook", relative_href(output_rel, prefixed_output_rel(config, "handbook/en/index.html"))),
        ("English command reference", relative_href(output_rel, prefixed_output_rel(config, "commands/en/index.html"))),
    ]
    if config.raw_manpage_target_rel:
        related.append(("Top-level roff manpage", relative_href(output_rel, config.raw_manpage_target_rel)))
    return page_shell(
        page_title="Manpages",
        html_lang="en",
        home_href=relative_href(output_rel, prefixed_output_rel(config, "index.html")),
        hero_title="Generated Manpages",
        hero_summary="Browser-readable HTML mirrors of the generated roff pages.",
        breadcrumbs=[("Home", relative_href(output_rel, prefixed_output_rel(config, "index.html"))), ("Manpages", None)],
        body_html=body,
        toc_html="<p>Open a browser-readable manpage or jump back to the command-reference lane.</p>",
        related_html=html_list(related),
        version_html=render_version_links(output_rel, config),
        locale_html="",
        footer_nav_html="",
        footer_html='Generated from <code>docs/man/*.1</code> via <code>scripts/generate_manpages.py</code>.',
        jump_html=render_jump_select(output_rel, "en", config),
        nav_html=render_manpage_nav(output_rel, config, names=names),
    )


def render_manpage_page(output_rel, name, roff, config, *, manpage_names=()):
    stem = Path(name).stem
    display_name = manpage_label(name)
    command_root = command_reference_root_for_stem(stem, config)
    related = [("Manpage index", relative_href(output_rel, prefixed_output_rel(config, "man/index.html")))]
    if config.raw_manpage_target_rel:
        related.append(("Raw roff source", relative_href(output_rel, prefixed_output_rel(config, f"man/{name}"))))
    if command_root:
        related.append(("Matching command reference", relative_href(output_rel, command_root)))

    def manpage_href(target_stem):
        return relative_href(output_rel, prefixed_output_rel(config, f"man/{target_stem}.html"))

    body = render_template("manpage_intro.html.tmpl", summary_html="This page keeps the terminal manpage format available in the browser. Use the command reference for fuller workflow notes and examples.", body_html=render_roff_manpage_html(roff, manpage_href=manpage_href))
    return page_shell(
        page_title=display_name,
        html_lang="en",
        home_href=relative_href(output_rel, prefixed_output_rel(config, "index.html")),
        hero_title=display_name,
        hero_summary="CLI manual page.",
        breadcrumbs=[("Home", relative_href(output_rel, prefixed_output_rel(config, "index.html"))), ("Manpages", relative_href(output_rel, prefixed_output_rel(config, "man/index.html"))), (display_name, None)],
        body_html=body,
        toc_html='<ul class="sidebar-meta-list"><li><a href="#synopsis" class="sidebar-meta-link">Syntax</a></li><li><a href="#description" class="sidebar-meta-link">Description</a></li><li><a href="#see-also" class="sidebar-meta-link">Related manpages</a></li></ul>',
        related_html=html_list(related),
        version_html=render_version_links(output_rel, config),
        locale_html="",
        footer_nav_html="",
        footer_html=f'Source: <code>docs/man/{html.escape(name)}</code><br>Generated by <code>scripts/generate_command_html.py</code>.',
        jump_html=render_jump_select(output_rel, "en", config),
        nav_html=render_manpage_nav(output_rel, config, names=manpage_names, current_name=name, command_root=command_root),
    )
