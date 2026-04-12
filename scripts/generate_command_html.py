#!/usr/bin/env python3
"""Generate the static HTML docs site from handbook and command Markdown."""

from __future__ import annotations

import argparse
from dataclasses import replace
from pathlib import Path

from docgen_command_docs import RenderedHeading
from docgen_common import check_outputs, print_written_outputs, write_outputs
from docgen_handbook import HANDBOOK_LOCALES, build_handbook_pages
from docsite_html_common import (
    COMMAND_DOC_LOCALES,
    HTML_ROOT_DIR,
    HtmlBuildConfig,
    PAGE_STYLE,
    THEME_SCRIPT,
    VersionLink,
    render_section_index,
    render_toc,
    split_display_title,
    strip_heading_decorations,
    strip_leading_h1,
    title_only,
    prefixed_output_rel,
)
from docsite_command_surface import command_audience_hint, command_handbook_stem
from docsite_html_nav import (
    command_reference_label,
    handbook_surface_label,
    render_command_map_nav,
    render_global_nav,
    render_jump_select_options,
)
from docsite_html_pages import (
    build_landing_locale_data,
    command_breadcrumb_label,
    page_shell,
    render_command_page,
    render_developer_page,
    render_footer_nav,
    render_handbook_page,
    render_landing_page,
    render_manpage_page,
    render_manpage_index_page,
)
from generate_manpages import LEGACY_ROOT_SOURCE_PAGES, generate_manpages


def generate_outputs(config=HtmlBuildConfig()):
    outputs = {prefixed_output_rel(config, "index.html"): render_landing_page(config), ".nojekyll": ""}
    developer_html = render_developer_page(config)
    if developer_html:
        outputs[prefixed_output_rel(config, "developer.html")] = developer_html

    manpage_outputs = generate_manpages(command_docs_dir=config.command_docs_root / "en", version=config.version)
    man_index_rel = prefixed_output_rel(config, "man/index.html")
    outputs[man_index_rel] = render_manpage_index_page(man_index_rel, sorted(manpage_outputs), config)
    for name, roff in sorted(manpage_outputs.items()):
        html_rel = prefixed_output_rel(config, f"man/{Path(name).stem}.html")
        outputs[html_rel] = render_manpage_page(html_rel, name, roff, config)
        if config.include_raw_manpages:
            outputs[prefixed_output_rel(config, f"man/{name}")] = roff

    for locale in COMMAND_DOC_LOCALES:
        for source in sorted((config.command_docs_root / locale).glob("*.md")):
            if source.name in LEGACY_ROOT_SOURCE_PAGES:
                continue
            output_rel = prefixed_output_rel(config, f"commands/{locale}/{source.with_suffix('.html').name}")
            outputs[output_rel] = render_command_page(locale, source, output_rel, config)

    for locale in HANDBOOK_LOCALES:
        for page in build_handbook_pages(locale, handbook_root=config.handbook_root):
            versioned_page = replace(
                page,
                output_rel=prefixed_output_rel(config, page.output_rel),
                previous_output_rel=prefixed_output_rel(config, page.previous_output_rel) if page.previous_output_rel else None,
                next_output_rel=prefixed_output_rel(config, page.next_output_rel) if page.next_output_rel else None,
                language_switch_rel=prefixed_output_rel(config, page.language_switch_rel) if page.language_switch_rel else None,
            )
            outputs[versioned_page.output_rel] = render_handbook_page(versioned_page, config)
    return outputs


def build_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--check", action="store_true")
    return parser


def main(argv=None):
    args = build_parser().parse_args(argv)
    outputs = generate_outputs()
    if args.check:
        return check_outputs(HTML_ROOT_DIR, outputs, "HTML docs", "python3 scripts/generate_command_html.py --write", prune=True)
    write_outputs(HTML_ROOT_DIR, outputs, prune=True)
    print_written_outputs(HTML_ROOT_DIR, outputs, "HTML docs", "docs/*", "docs/html/*", "docs/html/index.html")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
