from __future__ import annotations

from pathlib import Path

from docgen_common import relative_href
from docsite_command_surface import command_handbook_stem
from docsite_html_common import prefixed_output_rel


def command_handbook_context(locale, output_rel, source_name, config):
    handbook_stem = command_handbook_stem(Path(source_name))
    if not handbook_stem:
        return None
    label = "Matching handbook chapter" if locale == "en" else "對應手冊章節"
    return (label, relative_href(output_rel, prefixed_output_rel(config, f"handbook/{locale}/{handbook_stem}.html")))


def rewrite_markdown_link(source_path, output_rel, target, config):
    if target.startswith(("http", "mailto:", "#")):
        return target
    bare, frag = (target.split("#", 1) + [""])[:2]
    resolved = (source_path.parent / bare).resolve()
    if resolved == (config.source_root / "docs" / "DEVELOPER.md").resolve():
        rel = "developer.html"
    else:
        try:
            rel = resolved.relative_to(config.source_root / "docs").as_posix()
        except Exception:
            return target
    if rel.startswith("commands/") and rel.endswith(".md"):
        rel = rel[:-3] + ".html"
    elif rel.startswith("user-guide/") and rel.endswith(".md"):
        rel = rel.replace("user-guide/", "handbook/", 1)[:-3] + ".html"
    href = relative_href(output_rel, prefixed_output_rel(config, rel))
    return f"{href}#{frag}" if frag else href
