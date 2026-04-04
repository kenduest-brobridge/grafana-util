#!/usr/bin/env python3
"""Generate the static HTML docs site from handbook and command Markdown."""

from __future__ import annotations

import argparse
import html
import json
import re
from dataclasses import dataclass, replace
from pathlib import Path

from docgen_command_docs import RenderedHeading, render_markdown_document
from docgen_common import REPO_ROOT, VERSION, check_outputs, print_written_outputs, relative_href, write_outputs
from docgen_handbook import (
    HANDBOOK_LOCALES,
    HANDBOOK_NAV_GROUP_LABELS,
    HANDBOOK_NAV_GROUPS,
    HANDBOOK_ORDER,
    LOCALE_LABELS,
    build_handbook_pages,
    handbook_language_href,
)
from docgen_landing import LANDING_LOCALES, LANDING_UI_LABELS, LandingLink, LandingSection, LandingTask, load_landing_page
from generate_manpages import NAMESPACE_SPECS, generate_manpages

HTML_ROOT_DIR = REPO_ROOT / "docs" / "html"
COMMAND_DOCS_ROOT = REPO_ROOT / "docs" / "commands"
HANDBOOK_ROOT = REPO_ROOT / "docs" / "user-guide"
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

HANDBOOK_CONTEXT_BY_COMMAND = {
    "index": "index",
    "dashboard": "dashboard",
    "datasource": "datasource",
    "alert": "alert",
    "access": "access",
    "change": "change-overview-status",
    "status": "change-overview-status",
    "overview": "change-overview-status",
    "snapshot": "change-overview-status",
    "profile": "getting-started",
}

PAGE_STYLE = """
:root {
  color-scheme: light dark;
  --font-scale: 1;
  --font-sans: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Segoe UI Variable Text", "Segoe UI", "Helvetica Neue", Arial, sans-serif;
  --font-display: -apple-system, BlinkMacSystemFont, "SF Pro Display", "Segoe UI Variable Display", "Segoe UI", "Helvetica Neue", Arial, sans-serif;
  --font-zh: "PingFang TC", "Noto Sans TC", "Microsoft JhengHei", sans-serif;
  --font-mono: ui-monospace, "JetBrains Mono", SFMono-Regular, Menlo, Consolas, monospace;
  
  --bg: #ffffff;
  --panel: #ffffff;
  --panel-strong: #f9fbfe;
  --text: #1c2740;
  --muted: #30435f;
  --heading: #0b1633;
  --accent: #0d84d8;
  --accent-soft: #ecf7ff;
  --border: #ccd8e5;
  --code-bg: #edf3f9;
  --pre-bg: #0f172a;
  --pre-text: #f8fafc;
  --shadow: 0 8px 22px rgba(15, 23, 42, 0.05);
  --input-bg: #ffffff;
  --input-border: #ccd8e5;
  --input-text: #1c2740;
  --cta-bg: #0d84d8;
  --cta-border: #0d84d8;
  --cta-text: #ffffff;
}

@media (prefers-color-scheme: dark) {
  :root {
    --bg: #0b0f14;
    --panel: #11161c;
    --panel-strong: #151b22;
    --text: #e6edf3;
    --muted: #9da7b3;
    --heading: #f3f4f6;
    --accent: #aab4bf;
    --accent-soft: rgba(170, 180, 191, 0.16);
    --border: #202833;
    --code-bg: #1b222b;
    --pre-bg: #0a0d10;
    --pre-text: #f3f4f6;
    --shadow: 0 14px 32px rgba(0, 0, 0, 0.34);
    --input-bg: #11161c;
    --input-border: #28313b;
    --input-text: #f3f4f6;
    --cta-bg: #24303b;
    --cta-border: #334250;
    --cta-text: #f3f4f6;
  }
}

html[data-theme="light"] {
  --bg: #ffffff;
  --panel: #ffffff;
  --panel-strong: #f9fbfe;
  --text: #1c2740;
  --muted: #30435f;
  --heading: #0b1633;
  --accent: #0d84d8;
  --accent-soft: #ecf7ff;
  --border: #ccd8e5;
  --code-bg: #edf3f9;
  --pre-bg: #0f172a;
  --pre-text: #f8fafc;
  --shadow: 0 8px 22px rgba(15, 23, 42, 0.05);
  --input-bg: #ffffff;
  --input-border: #ccd8e5;
  --input-text: #1c2740;
  --cta-bg: #0d84d8;
  --cta-border: #0d84d8;
  --cta-text: #ffffff;
}

html[data-theme="dark"] {
  --bg: #0b0f14;
  --panel: #11161c;
  --panel-strong: #151b22;
  --text: #e6edf3;
  --muted: #9da7b3;
  --heading: #f3f4f6;
  --accent: #aab4bf;
  --accent-soft: rgba(170, 180, 191, 0.16);
  --border: #202833;
  --code-bg: #1b222b;
  --pre-bg: #0a0d10;
  --pre-text: #f3f4f6;
  --shadow: 0 14px 32px rgba(0, 0, 0, 0.34);
  --input-bg: #11161c;
  --input-border: #28313b;
  --input-text: #f3f4f6;
  --cta-bg: #24303b;
  --cta-border: #334250;
  --cta-text: #f3f4f6;
}

html { font-size: calc(16px * var(--font-scale)); scroll-behavior: smooth; }
* { box-sizing: border-box; }
body { margin: 0; font-family: var(--font-sans); font-weight: 400; color: var(--text); background: var(--bg); text-rendering: optimizeLegibility; -webkit-font-smoothing: antialiased; }
html[lang="zh-TW"] body { font-family: var(--font-zh); letter-spacing: 0.025em; }
a { color: var(--accent); text-decoration: none; font-weight: 500; transition: color 0.15s; }
a:hover { color: var(--heading); }

/* Typography */
h1, h2, h3, h4 { font-family: var(--font-display); color: var(--heading); font-weight: 650; line-height: 1.2; margin: 0; }
p, li { line-height: 1.72; margin-bottom: 1rem; }
code { font: 0.88em var(--font-mono); background: var(--code-bg); padding: 0.2em 0.4em; border-radius: 4px; }

/* Pre & Code Blocks */
pre { position: relative; padding: 20px; border-radius: 8px; background: var(--pre-bg); color: var(--pre-text); margin: 1.5rem 0; border: 1px solid var(--border); overflow-x: auto; }
pre code { background: transparent !important; color: inherit !important; padding: 0 !important; font-size: 0.9rem; line-height: 1.6; white-space: pre; }
pre.wrapped code { white-space: pre-wrap !important; word-break: break-all; }
.code-controls { position: absolute; top: 8px; right: 8px; display: flex; gap: 6px; opacity: 0; transition: opacity 0.2s; z-index: 10; }
pre:hover .code-controls { opacity: 1; }
.control-btn { padding: 4px 8px; border-radius: 4px; border: 1px solid rgba(255,255,255,0.1); background: rgba(255,255,255,0.1); color: #fff; font: 500 11px var(--font-mono); cursor: pointer; transition: 0.2s; }
.control-btn:hover { background: rgba(255,255,255,0.2); }
.control-btn.active { background: var(--accent); border-color: var(--accent); }

/* Layout Elements */
.site { max-width: 1820px; margin: 0 auto; padding: 20px 18px; min-height: 100vh; display: flex; flex-direction: column; }
.site.landing-shell { max-width: 1640px; padding-left: 36px; padding-right: 36px; }
.site.landing-shell .topbar { margin-bottom: 10px; }
.site-content { flex-grow: 1; }
.topbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; flex-wrap: wrap; gap: 16px; border-bottom: 1px solid var(--border); padding-bottom: 14px; }
.brand { font: 760 1.08rem var(--font-mono); text-transform: uppercase; letter-spacing: 0.045em; color: var(--heading) !important; text-decoration: none !important; }
.themebar { display: flex; gap: 8px; align-items: center; }
.themebar select {
  border: 1px solid var(--border);
  background: var(--panel);
  color: var(--text);
  padding: 4px 10px;
  font-size: 0.8rem; font-weight: 500; font-family: var(--font-sans);
  border-radius: 4px; height: 28px;
  cursor: pointer; transition: all 0.2s;
}
.themebar select:hover, .themebar select:focus { border-color: var(--accent); outline: none; }

/* Content Grid */
.layout { display: grid; grid-template-columns: 232px minmax(0, 1fr) 244px; gap: 20px; align-items: start; }
@media (max-width: 1280px) { .layout { grid-template-columns: 210px minmax(0, 1fr); gap: 18px; } .sidebar { display: none !important; } }
@media (max-width: 768px) { .layout { grid-template-columns: 1fr; } .nav-sidebar { display: none !important; } }

/* Left Nav Sidebar */
.nav-sidebar { position: sticky; top: 12px; padding: 2px 0 0; max-height: calc(100vh - 24px); overflow-y: auto; font-family: var(--font-sans); letter-spacing: -0.01em; }
html[lang="zh-TW"] .nav-sidebar { font-family: var(--font-zh); letter-spacing: 0; }
.nav-sidebar li { margin-bottom: 0; }
.nav-sidebar h2 { font-size: 0.74rem; font-weight: 700; text-transform: uppercase; color: var(--muted); margin: 0 0 8px; letter-spacing: 0.04em; }
.nav-section + .nav-section { margin-top: 18px; }
.nav-group-block + .nav-group-block { margin-top: 14px; }
.nav-group-title { margin: 0 0 6px; font-size: 0.76rem; font-weight: 700; color: var(--muted); letter-spacing: 0.02em; }
.nav-tree,
.nav-sub-list { list-style: none; margin: 0; padding: 0; position: relative; }
.nav-tree::before,
.nav-sub-list::before { display: none; }
.nav-tree-node { position: relative; margin: 0; }
.nav-tree-node + .nav-tree-node { margin-top: 2px; }
.nav-tree-node::before { display: none; }
.nav-group { margin: 0; position: relative; }
.nav-group + .nav-group { margin-top: 4px; }
.nav-group-header { display: flex; justify-content: space-between; align-items: center; gap: 6px; padding: 1px 0; cursor: pointer; border-radius: 0; font-weight: 620; color: var(--heading); transition: color 0.15s; font-size: 0.92rem; }
.nav-group-header:hover { color: var(--heading); }
.nav-group-link { flex: 1; display: block; padding: 5px 8px; border-radius: 8px; color: var(--heading); text-decoration: none !important; font-weight: 610; line-height: 1.42; }
.nav-group-link:hover { background: var(--panel-strong); }
.nav-item { display: block; padding: 5px 8px; border-radius: 8px; color: var(--text); text-decoration: none !important; transition: 0.15s; font-weight: 550; font-size: 0.9rem; line-height: 1.44; }
.nav-group-link,
.nav-item,
.nav-command-entry { white-space: nowrap; }
.nav-item:hover { background: var(--panel-strong); color: var(--heading); }
.nav-item.active,
.nav-group-link.active { background: color-mix(in srgb, var(--accent-soft) 82%, white); color: var(--heading); font-weight: 620; box-shadow: inset 2px 0 0 var(--accent); }
.nav-sub-list { margin-top: 4px; margin-left: 10px; }
.nav-sub-list .nav-item { color: var(--muted); font-weight: 520; }
.nav-sub-list .nav-item.active { color: var(--heading); font-weight: 610; }
.nav-command-entry { display: block; padding: 5px 8px; border-radius: 8px; color: var(--muted); text-decoration: none !important; font-size: 0.88rem; font-weight: 560; line-height: 1.4; transition: 0.15s; }
.nav-command-entry:hover { color: var(--heading); background: var(--panel-strong); }
.nav-command-entry.active { color: var(--heading); font-weight: 620; background: color-mix(in srgb, var(--accent-soft) 60%, white); box-shadow: inset 2px 0 0 var(--accent); }

/* Main Article */
.article { padding: 10px 0 24px; }
.article p,
.article li { color: var(--text); font-weight: 460; }
.article ul li::marker,
.article ol li::marker { color: var(--heading); }
.hero-eyebrow { display: inline-block; padding: 2px 8px; border-radius: 999px; background: var(--accent-soft); color: var(--accent); font: 700 10px var(--font-mono); margin-bottom: 8px; text-transform: uppercase; }
.article h1 { font-size: 2.15rem; margin-bottom: 12px; letter-spacing: -0.018em; font-weight: 680; }
.article h2 { margin-top: 40px; border-top: 1px solid var(--border); padding-top: 24px; font-size: 1.5rem; margin-bottom: 12px; letter-spacing: -0.012em; font-weight: 620; }
.article h3 { margin-top: 24px; font-size: 1.2rem; margin-bottom: 12px; letter-spacing: -0.01em; font-weight: 610; }
.hero-title-main { display: block; }
.hero-title-secondary { display: block; margin-top: 6px; font-size: 0.68em; line-height: 1.2; color: var(--muted); font-weight: 560; letter-spacing: -0.012em; }
.hero-summary { margin: 0 0 18px; max-width: 68ch; color: var(--text); font-size: 1rem; line-height: 1.72; font-weight: 450; }
.article blockquote { margin: 1.5rem 0; padding: 12px 16px; border-left: 3px solid var(--accent); background: var(--accent-soft); border-radius: 4px; }
.article blockquote p { margin: 0; font-size: 0.95rem; color: var(--heading); }

/* Right TOC Sidebar */
.sidebar { position: sticky; top: 12px; padding: 2px 0 0; max-height: calc(100vh - 24px); overflow-y: auto; font-family: var(--font-sans); letter-spacing: -0.01em; }
html[lang="zh-TW"] .sidebar { font-family: var(--font-zh); letter-spacing: 0; }
.sidebar li { margin-bottom: 0; }
.sidebar h2 { font-size: 0.74rem; font-weight: 700; text-transform: uppercase; color: var(--muted); margin-bottom: 8px; letter-spacing: 0.05em; }
.sidebar-section + .sidebar-section { margin-top: 22px; }
.toc-list { list-style: none; padding: 0; margin: 0; border-left: 1px solid var(--border); }
.toc-list li + li { margin-top: 2px; }
.toc-list li.toc-level-2 a { font-size: 0.84rem; font-weight: 620; color: var(--text); }
.toc-list li.toc-level-3 a { font-size: 0.78rem; font-weight: 500; color: var(--muted); padding-left: 18px; }
.toc-list a { display: block; padding: 4px 10px; border-left: 2px solid transparent; margin-left: -1px; color: var(--muted); font-size: 0.82rem; transition: 0.15s; font-weight: 520; text-decoration: none !important; white-space: nowrap; }
.toc-list a:hover { color: var(--heading); border-left-color: var(--border); }
.toc-list a.active { color: var(--heading); border-left-color: var(--accent); font-weight: 680; }
.sidebar-meta-list { list-style: none; margin: 0; padding: 0; display: grid; gap: 6px; }
.sidebar-meta-link,
.sidebar-meta-text { display: block; font-size: 0.84rem; line-height: 1.4; font-weight: 520; }
.sidebar-meta-link { color: var(--text); text-decoration: none !important; }
.sidebar-meta-link,
.sidebar-meta-text { white-space: nowrap; }
.sidebar-meta-link:hover { color: var(--heading); }
.sidebar-meta-text { color: var(--muted); }
.sidebar-meta-current { color: var(--heading); font-weight: 620; }
.toc-icon { display: inline-block; opacity: 0.58; transform: scale(0.9); transform-origin: left center; margin-right: 0.32rem; }

.footer-nav { display: flex; justify-content: space-between; gap: 16px; margin-top: 48px; padding-top: 24px; border-top: 1px solid var(--border); }
.link-card { flex: 1; padding: 12px 16px; border: 1px solid var(--border); border-radius: 6px; background: var(--panel-strong); transition: 0.2s; text-decoration: none !important; color: var(--text); }
.link-card:hover { border-color: var(--accent); }
.link-card span { display: block; font-size: 0.7rem; text-transform: uppercase; color: var(--muted); margin-bottom: 4px; font-weight: 700; }

.site-footer { margin-top: 40px; padding: 24px 0; border-top: 1px solid var(--border); color: var(--muted); font-size: 0.85rem; text-align: center; }

/* Landing */
.landing-page {
  width: 100%;
  padding: 12px 0 20px;
}
.landing-hero {
  display: grid;
  grid-template-columns: minmax(0, 0.92fr) minmax(360px, 0.72fr);
  gap: 40px;
  align-items: start;
  padding: 18px 0 30px;
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--panel) 98%, white), color-mix(in srgb, var(--panel) 96%, var(--accent-soft)));
}
.landing-hero-inner {
  max-width: 760px;
}
.landing-title {
  margin: 0 0 18px;
  font-size: clamp(2.35rem, 4.3vw, 3.6rem);
  line-height: 0.96;
  letter-spacing: -0.05em;
}
.landing-summary {
  max-width: 72ch;
  margin: 0;
  font-size: 1.02rem;
  line-height: 1.72;
  color: var(--text);
}
.landing-search-panel {
  padding-top: 12px;
}
.landing-search-panel h2 {
  margin: 0 0 8px;
  font-size: 1.15rem;
  letter-spacing: -0.02em;
}
.landing-search-panel p {
  margin: 0 0 16px;
  font-size: 0.95rem;
  line-height: 1.7;
  color: var(--muted);
}
.landing-search-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
  align-items: stretch;
  max-width: 100%;
}
.landing-search-input {
  flex: 1;
  min-height: 48px;
  padding: 0 16px;
  border: 1px solid var(--input-border);
  border-radius: 14px;
  background: var(--input-bg);
  color: var(--input-text);
  font: 500 1rem var(--font-sans);
}
.landing-search-input::placeholder {
  color: var(--muted);
}
.landing-search-input:focus {
  outline: none;
  border-color: var(--accent);
}
.landing-search-button {
  min-height: 46px;
  padding: 0 18px;
  border: 1px solid var(--cta-border);
  border-radius: 14px;
  background: var(--cta-bg);
  color: var(--cta-text);
  font: 700 0.95rem var(--font-sans);
  cursor: pointer;
  align-self: flex-start;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
  transition: background 0.15s, border-color 0.15s, color 0.15s, transform 0.15s;
}
.landing-search-button:hover {
  background: color-mix(in srgb, var(--cta-bg) 84%, white);
  border-color: color-mix(in srgb, var(--cta-border) 78%, white);
}
.landing-search-button:focus-visible {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-soft);
}
.landing-search-button:active {
  transform: translateY(1px);
}
.landing-sections {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 36px;
  margin-top: 30px;
  padding-top: 26px;
  border-top: 1px solid var(--border);
}
.landing-section h2 {
  margin: 0 0 10px;
  font-size: 1.2rem;
  letter-spacing: -0.02em;
}
.landing-section p {
  margin: 0 0 18px;
  font-size: 0.96rem;
  line-height: 1.72;
  color: var(--muted);
}
.landing-task-list {
  margin: 0;
  padding: 0;
  list-style: none;
  display: grid;
  gap: 0;
}
.landing-meta {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 28px;
  align-items: start;
  margin-top: 36px;
  padding-top: 18px;
  border-top: 1px solid var(--border);
}
.landing-panel {
  padding: 0;
}
.landing-panel h2 {
  margin: 0 0 8px;
  font-size: 0.92rem;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}
.landing-panel p {
  margin: 0 0 14px;
  font-size: 0.98rem;
  line-height: 1.7;
  color: var(--muted);
}
html[data-theme="dark"] .landing-hero {
  background:
    radial-gradient(circle at 14% 10%, rgba(255, 255, 255, 0.04), transparent 24%),
    radial-gradient(circle at 82% 14%, rgba(170, 180, 191, 0.11), transparent 28%),
    linear-gradient(180deg, #0b0f14 0%, #0d1218 58%, #10161d 100%);
  border-bottom: 1px solid var(--border);
}
html[data-theme="dark"] .landing-search-panel {
  padding: 18px 20px 20px;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 18px;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.03);
}
html[data-theme="dark"] .landing-search-input::placeholder {
  color: color-mix(in srgb, var(--muted) 82%, white);
}
html[data-theme="dark"] .landing-sections {
  gap: 24px;
}
html[data-theme="dark"] .landing-section {
  padding: 18px 20px 20px;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 18px;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.02);
}
html[data-theme="dark"] .landing-task {
  border-top-color: color-mix(in srgb, var(--border) 76%, white);
}
html[data-theme="dark"] .landing-meta {
  border-top-color: var(--border);
}
.landing-link-list {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  margin: 0;
  padding: 0;
  list-style: none;
}
.landing-link-list a {
  display: inline-flex;
  align-items: center;
  min-height: 30px;
  padding: 0;
  background: transparent;
  color: var(--text);
}
.landing-task {
  padding: 16px 0 18px;
  border-top: 1px solid var(--border);
}
.landing-task:first-child {
  border-top: 0;
  padding-top: 0;
}
.landing-task-title {
  margin: 0 0 6px;
  font-size: 1rem;
  font-weight: 700;
  color: var(--heading);
}
.landing-task-copy {
  margin: 0 0 10px;
  font-size: 0.92rem;
  line-height: 1.65;
  color: var(--text);
}
.landing-task-links {
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
  margin: 0;
  padding: 0;
  list-style: none;
}
.landing-task-links a {
  color: var(--accent);
  font-size: 0.9rem;
  font-weight: 700;
}
.landing-inline-links {
  display: flex;
  flex-wrap: wrap;
  gap: 12px 18px;
  margin: 0 0 18px;
  padding: 0;
  list-style: none;
}
.landing-inline-links a {
  color: var(--text);
  font-size: 0.92rem;
  font-weight: 650;
}

/* Manpage */
.manpage-rendered { display: grid; gap: 18px; }
.manpage-rendered > p { margin: 0; }
.manpage-rendered .man-section h2 { margin: 0 0 10px; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
.manpage-rendered .man-section + .man-section { margin-top: 6px; }
.manpage-rendered .man-bullets { margin: 0; padding-left: 1.2rem; }
.manpage-rendered .man-bullets li + li { margin-top: 8px; }
.manpage-rendered .man-definitions { display: grid; grid-template-columns: minmax(0, 220px) minmax(0, 1fr); gap: 10px 18px; margin: 0; }
.manpage-rendered .man-definitions dt { font-weight: 700; color: var(--heading); }
.manpage-rendered .man-definitions dd { margin: 0; }
.manpage-rendered pre.man-example { margin: 0; }

@media (max-width: 860px) {
  .landing-sections,
  .landing-meta {
    grid-template-columns: 1fr;
  }
  .landing-page {
    width: 100%;
    margin: 0;
  }
  .landing-hero {
    grid-template-columns: 1fr;
    gap: 20px;
    padding: 20px 0 24px;
  }
  .landing-hero-inner {
    max-width: 100%;
  }
  .landing-summary {
    font-size: 1.05rem;
    white-space: normal;
  }
  .landing-search-form {
    align-items: stretch;
  }
  .landing-search-button {
    width: 100%;
    align-self: stretch;
  }
  .landing-panel {
    padding: 0;
  }
  .landing-meta {
    grid-template-columns: 1fr;
    gap: 18px;
  }
  .site.landing-shell {
    padding-left: 20px;
    padding-right: 20px;
  }
}
"""

PRISM_STYLE = """
code[class*="language-"],pre[class*="language-"]{color:#ccc;background:none;font-family:var(--font-mono);font-size:1em;text-align:left;white-space:pre;word-spacing:normal;word-break:normal;word-wrap:normal;line-height:1.5;-moz-tab-size:4;-o-tab-size:4;tab-size:4;-webkit-hyphens:none;-moz-hyphens:none;-ms-hyphens:none;hyphens:none}
.token.comment{color:#999}.token.punctuation{color:#ccc}.token.keyword{color:#cc99cd}.token.string{color:#7ec699}.token.function{color:#f08d49}.token.operator{color:#67cdcc}
"""

THEME_SCRIPT = """
<script>
(() => {
  const root = document.documentElement;
  const storageKey = "grafana-util-docs-theme";
  const select = document.getElementById("theme-select");
  const saved = localStorage.getItem(storageKey) || "auto";
  const applyTheme = (val) => { if(val==="auto") root.removeAttribute("data-theme"); else root.setAttribute("data-theme", val); };
  applyTheme(saved);
  if(select) { select.value = saved; select.onchange = (e) => { localStorage.setItem(storageKey, e.target.value); applyTheme(e.target.value); }; }

  const fontKey = "grafana-util-docs-font";
  const fontSelect = document.getElementById("font-select");
  const savedFont = localStorage.getItem(fontKey) || "1";
  const applyFont = (val) => { root.style.setProperty("--font-scale", val); };
  applyFont(savedFont);
  if(fontSelect) { fontSelect.value = savedFont; fontSelect.onchange = (e) => { localStorage.setItem(fontKey, e.target.value); applyFont(e.target.value); }; }

  const wrapKey = "grafana-util-docs-wrap";
  let isWrapped = localStorage.getItem(wrapKey) === "true";
  const updateWrap = (block, btn, wrapped) => {
    if(!btn) return;
    if(wrapped) { block.classList.add("wrapped"); btn.classList.add("active"); btn.innerText="Wrap: ON"; }
    else { block.classList.remove("wrapped"); btn.classList.remove("active"); btn.innerText="Wrap: OFF"; }
  };
  document.querySelectorAll("pre").forEach(block => {
    const controls = document.createElement("div"); controls.className = "code-controls";
    const wrapBtn = document.createElement("button"); wrapBtn.className = "control-btn";
    updateWrap(block, wrapBtn, isWrapped);
    wrapBtn.onclick = () => { isWrapped = !isWrapped; localStorage.setItem(wrapKey, isWrapped); document.querySelectorAll("pre").forEach(b => updateWrap(b, b.querySelector(".control-btn"), isWrapped)); };
    const copyBtn = document.createElement("button"); copyBtn.className = "control-btn"; copyBtn.innerText = "Copy";
    copyBtn.onclick = () => {
      const raw = block.querySelector("code").innerText;
      const filtered = raw.split("\\n").filter(l => l.trim() && !l.trim().startsWith("#")).join("\\n") || raw;
      navigator.clipboard.writeText(filtered).then(() => { copyBtn.innerText="Copied!"; setTimeout(()=>copyBtn.innerText="Copy", 2000); });
    };
    controls.append(wrapBtn, copyBtn); block.append(controls);
  });

  document.querySelectorAll(".nav-group-header").forEach(header => {
    header.onclick = () => { header.closest(".nav-group").classList.toggle("collapsed"); };
  });

  const jumpSelect = document.getElementById("jump-select");
  if(jumpSelect) jumpSelect.onchange = (e) => { if(e.target.value) window.location.href = e.target.value; };

  const landingI18n = document.getElementById("landing-i18n");
  const localeSelect = document.getElementById("locale-select");
  const landingSearchForm = document.getElementById("landing-search-form");
  const landingSearchInput = document.getElementById("landing-search");
  const landingSearchButton = document.getElementById("landing-search-button");
  const landingTitle = document.getElementById("landing-title");
  const landingSummary = document.getElementById("landing-summary");
  const landingSearchHeading = document.getElementById("landing-search-heading");
  const landingSearchCopy = document.getElementById("landing-search-copy");
  const landingSections = document.getElementById("landing-sections");
  const landingMeta = document.getElementById("landing-meta");
  if(landingI18n) {
    const landingData = JSON.parse(landingI18n.textContent);
    const landingLocaleKey = "grafana-util-docs-locale";
    const pickLandingLocale = () => {
      const saved = localStorage.getItem(landingLocaleKey);
      if(saved && landingData[saved]) return saved;
      const browserLocales = [...(navigator.languages || []), navigator.language || ""];
      const preferredZh = browserLocales.some((locale) => locale && locale.toLowerCase().startsWith("zh"));
      return preferredZh && landingData["zh-TW"] ? "zh-TW" : "en";
    };
    const applyLandingLocale = (locale) => {
      const copy = landingData[locale] || landingData.en;
      if(!copy) return;
      document.documentElement.lang = copy.lang || locale;
      if(localeSelect) localeSelect.value = locale;
      if(landingTitle) landingTitle.textContent = copy.hero_title;
      if(landingSummary) landingSummary.textContent = copy.hero_summary;
      if(landingSearchHeading) landingSearchHeading.textContent = copy.search_heading;
      if(landingSearchCopy) landingSearchCopy.textContent = copy.search_copy;
      if(landingSearchInput) {
        landingSearchInput.placeholder = copy.search_placeholder;
        landingSearchInput.setAttribute("aria-label", copy.search_placeholder);
      }
      if(landingSearchButton) landingSearchButton.textContent = copy.search_button;
      if(landingSections) landingSections.innerHTML = copy.sections_html;
      if(landingMeta) landingMeta.innerHTML = copy.meta_html;
      if(jumpSelect) jumpSelect.innerHTML = copy.jump_options_html;
      localStorage.setItem(landingLocaleKey, locale);
    };
    applyLandingLocale(pickLandingLocale());
    if(localeSelect) {
      localeSelect.addEventListener("change", (event) => {
        applyLandingLocale(event.target.value);
      });
    }
  }

  if(landingSearchForm && landingSearchInput && jumpSelect) {
    landingSearchForm.addEventListener("submit", (event) => {
      event.preventDefault();
      const query = landingSearchInput.value.trim().toLowerCase();
      if(!query) return;
      const options = [...jumpSelect.querySelectorAll("option")].filter((option) => option.value);
      const exact = options.find((option) => option.textContent.toLowerCase().includes(query));
      const fuzzy = options.find((option) => option.value.toLowerCase().includes(query.replace(/\\s+/g, "-")));
      const target = exact || fuzzy;
      if(target) window.location.href = target.value;
    });
  }

  const observer = new IntersectionObserver(entries => {
    entries.forEach(entry => {
      const id = entry.target.id; if(!id) return;
      const link = document.querySelector(`.sidebar a[href="#${id}"]`);
      if(link && entry.isIntersecting) { document.querySelectorAll(".sidebar a").forEach(l => l.classList.remove("active")); link.classList.add("active"); }
    });
  }, { rootMargin: "-20px 0px -80% 0px" });
  document.querySelectorAll(".article h2[id], .article h3[id]").forEach(el => observer.observe(el));
})();
</script>
"""

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
        cleaned = strip_decorative_prefix(html.unescape(content))
        return f"<h{level}{attrs}>{html.escape(cleaned)}</h{level}>"
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
    if not items: return ""
    links = []
    for l, h in items:
        if h: links.append(f'<a href="{html.escape(h)}" style="color:inherit;text-decoration:none;">{html.escape(l)}</a>')
        else: links.append(html.escape(l))
    return '<div style="margin-bottom:12px; font-size:0.84rem; color:color-mix(in srgb, var(--muted) 78%, white); font-weight:460;">' + " / ".join(links) + "</div>"

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
        items.append(
            f'<li class="toc-level-{level}"><a href="{html.escape(href)}">{label_html}</a></li>'
        )
    return '<ul class="toc-list">' + "".join(items) + "</ul>"

def prefixed_output_rel(config: HtmlBuildConfig, rel: str) -> str:
    return f"{config.output_prefix.strip('/')}/{rel}" if config.output_prefix else rel

def render_version_links(output_rel: str, config: HtmlBuildConfig) -> str:
    items = []
    if config.version_label: items.append((f"Current: {config.version_label}", "#"))
    for link in config.version_links: items.append((link.label, relative_href(output_rel, link.target_rel)))
    return html_list(items) if items else '<p class="sidebar-meta-text">Current checkout</p>'

def handbook_nav_titles(locale: str, config: HtmlBuildConfig) -> dict[str, str]:
    pages = build_handbook_pages(locale, handbook_root=config.handbook_root)
    titles: dict[str, str] = {}
    for page in pages:
        stem = Path(page.output_rel).stem
        titles[stem] = format_handbook_nav_label(page.title, locale, stem)
    return titles


def handbook_nav_groups(locale: str, config: HtmlBuildConfig) -> list[tuple[str, list[tuple[str, str, str]]]]:
    titles = handbook_nav_titles(locale, config)
    grouped: list[tuple[str, list[tuple[str, str, str]]]] = []
    labels = HANDBOOK_NAV_GROUP_LABELS[locale]
    for group_key, filenames in HANDBOOK_NAV_GROUPS:
        items: list[tuple[str, str, str]] = []
        for filename in filenames:
            stem = Path(filename).stem
            target = prefixed_output_rel(config, f"handbook/{locale}/{stem}.html")
            items.append((stem, titles[stem], target))
        grouped.append((labels[group_key], items))
    return grouped

def format_handbook_nav_label(title: str, locale: str, stem: str) -> str:
    if stem == "index":
        return "Overview" if locale == "en" else "概觀"
    clean = re.sub(r"^[^\w\u4e00-\u9fffA-Za-z]+", "", title).strip()
    main, secondary = split_display_title(clean)
    if locale == "zh-TW" and secondary:
        return main
    return clean or title

def command_namespace_label(spec) -> str:
    root = Path(spec.root_doc).stem.replace("-", " ")
    return " ".join(part.capitalize() for part in root.split())

def render_jump_select_options(output_rel: str, locale: str, config: HtmlBuildConfig) -> str:
    handbook_label = "Handbook" if locale == "en" else "使用手冊"
    commands_label = "Commands" if locale == "en" else "指令參考"
    prompt_label = "Jump to..." if locale == "en" else "快速跳轉..."
    handbook_titles = handbook_nav_titles(locale, config)
    sections = [f'<option value="" selected>{html.escape(prompt_label)}</option>', f'<optgroup label="{handbook_label}">']
    for name in HANDBOOK_ORDER:
        stem = Path(name).stem
        label = handbook_titles.get(stem, (stem.replace("-", " ").title()) if stem != "index" else ("Overview" if locale == "en" else "概觀"))
        sections.append(f'<option value="{html.escape(relative_href(output_rel, prefixed_output_rel(config, f"handbook/{locale}/{stem}.html")))}">{html.escape(label)}</option>')
    sections.append(f'</optgroup><optgroup label="{commands_label}">')
    for spec in NAMESPACE_SPECS:
        sections.append(
            f'<option value="{html.escape(relative_href(output_rel, prefixed_output_rel(config, f"commands/{locale}/{spec.root_doc[:-3]}.html")))}">{html.escape(command_namespace_label(spec))}</option>'
        )
    sections.append('</optgroup>')
    return "".join(sections)

def render_jump_select(output_rel: str, locale: str, config: HtmlBuildConfig) -> str:
    return f'<select id="jump-select" aria-label="Jump">{render_jump_select_options(output_rel, locale, config)}</select>'

def render_landing_locale_select(current_locale: str = "en") -> str:
    selected_en = ' selected' if current_locale == "en" else ""
    selected_zh = ' selected' if current_locale == "zh-TW" else ""
    return (
        '<select id="locale-select" aria-label="Language">'
        f'<option value="en"{selected_en}>English</option>'
        f'<option value="zh-TW"{selected_zh}>繁體中文</option>'
        "</select>"
    )

def render_global_nav(output_rel: str, locale: str, config: HtmlBuildConfig) -> str:
    if not output_rel or "index.html" in output_rel and "/" not in output_rel.replace("index.html", ""): return ""
    handbook_label = "Handbook" if locale == "en" else "使用手冊"
    commands_label = "Command Reference" if locale == "en" else "指令參考"
    sections = []
    group_html: list[str] = []
    for group_label, items in handbook_nav_groups(locale, config):
        group_items: list[str] = []
        for stem, label, target in items:
            href = relative_href(output_rel, target)
            is_active = output_rel == target
            active = " active" if is_active else ""
            group_items.append(f'<li class="nav-tree-node"><a href="{html.escape(href)}" class="nav-item{active}">{html.escape(label)}</a></li>')
        group_html.append(f'<div class="nav-group-block"><div class="nav-group-title">{html.escape(group_label)}</div><ul class="nav-tree">{"".join(group_items)}</ul></div>')
    sections.append(f'<section class="nav-section"><h2>{handbook_label}</h2>{"".join(group_html)}</section>')
    command_target = prefixed_output_rel(config, f"commands/{locale}/index.html")
    command_href = relative_href(output_rel, command_target)
    command_active = " active" if output_rel == command_target or f"commands/{locale}/" in output_rel else ""
    command_label = "Open command docs" if locale == "en" else "開啟指令參考"
    sections.append(
        f'<section class="nav-section"><h2>{commands_label}</h2><a href="{html.escape(command_href)}" class="nav-command-entry{command_active}">{html.escape(command_label)}</a></section>'
    )
    return f'<aside class="nav-sidebar">{"".join(sections)}</aside>'

def page_shell(*, page_title, html_lang, home_href, hero_title, hero_summary, breadcrumbs, body_html, toc_html, related_html, version_html, locale_html, footer_nav_html, footer_html, jump_html="", nav_html="", is_landing=False):
    header_html = ""
    if hero_title and not is_landing:
        eyebrow_html = f'<div class="hero-eyebrow">Docs</div>'
        summary_html = f'<p class="hero-summary">{hero_summary}</p>' if hero_summary else ""
        title_main, title_secondary = split_display_title(hero_title)
        title_html = f'<span class="hero-title-main">{html.escape(title_main)}</span>'
        if title_secondary:
            title_html += f'<span class="hero-title-secondary">{html.escape(title_secondary)}</span>'
        header_html = f'{eyebrow_html}<h1>{title_html}</h1>{summary_html}'
    
    sidebar_html = ""
    if toc_html and not is_landing:
        sidebar_html = f'<aside class="sidebar"><section class="sidebar-section"><h2>On This Page</h2>{toc_html}</section><section class="sidebar-section"><h2>Related</h2>{related_html}</section><section class="sidebar-section"><h2>Version</h2>{version_html}</section><section class="sidebar-section"><h2>Language</h2>{locale_html}</section></aside>'
    
    content = ""
    if is_landing:
        content = body_html
    else:
        layout_class = "layout" if nav_html and sidebar_html else "layout"
        content = f"""
        <div class="{layout_class}">
          {nav_html}
          <article class="article">
            {render_breadcrumbs(breadcrumbs)}
            {header_html}
            {body_html}
            {footer_nav_html}
          </article>
          {sidebar_html}
        </div>
        """
    
    shell = f"""<!DOCTYPE html>
<html lang="{html.escape(html_lang)}">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{html.escape(page_title)} · grafana-util docs</title>
  <style>STYLE_PLACEHOLDER</style>
  <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/themes/prism-tomorrow.min.css" media="print" onload="this.media='all'">
</head>
<body>
  <div class="site{' landing-shell' if is_landing else ''}">
    <div class="site-content">
      <div class="topbar">
        <a class="brand" href="{html.escape(home_href)}">grafana-util docs</a>
        <div class="themebar">
          {jump_html}
          <select id="font-select" aria-label="Font Size">
            <option value="0.9">Font: Compact</option>
            <option value="1">Font: Default</option>
            <option value="1.1">Font: Large</option>
            <option value="1.2">Font: Extra Large</option>
          </select>
          <select id="theme-select" aria-label="Theme">
            <option value="auto">Theme: Auto</option>
            <option value="light">Theme: Light</option>
            <option value="dark">Theme: Dark</option>
          </select>
        </div>
      </div>
      {content}
    </div>
    <footer class="site-footer">{footer_html}</footer>
  </div>
  <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-core.min.js" async></script>
  <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/plugins/autoloader/prism-autoloader.min.js" async></script>
  {THEME_SCRIPT}
</body>
</html>
"""
    return shell.replace("STYLE_PLACEHOLDER", PAGE_STYLE + PRISM_STYLE)

def command_intro_text(locale): return ""
def handbook_intro_text(locale): return ""

def render_footer_nav(prev, nxt):
    cards = []
    if prev: cards.append(f'<a class="link-card" href="{html.escape(prev[1])}"><span>Previous</span>{html.escape(prev[0])}</a>')
    if nxt: cards.append(f'<a class="link-card" href="{html.escape(nxt[1])}"><span>Next</span>{html.escape(nxt[0])}</a>')
    return '<nav class="footer-nav">' + "".join(cards) + "</nav>" if cards else ""

def render_language_links(cur, sw_l, sw_h):
    items = [(f"Current: {cur}", "#")]
    if sw_l and sw_h: items.append((f"Switch to {sw_l}", sw_h))
    return html_list(items)

def command_reference_root_for_stem(stem, config):
    if stem == "grafana-util": return prefixed_output_rel(config, "commands/en/index.html")
    for spec in NAMESPACE_SPECS:
        if spec.stem == stem: return prefixed_output_rel(config, f"commands/en/{spec.root_doc[:-3]}.html")
    return None

def render_manpage_index_page(output_rel, names, config):
    body = (
        "<p>This lane mirrors the checked-in generated manpages as browser-readable HTML for local browsing and GitHub Pages.</p>"
        + "<ul>"
        + "".join(
            f'<li><a href="{html.escape(relative_href(output_rel, prefixed_output_rel(config, f"man/{Path(n).stem}.html")))}">{html.escape(n)}</a></li>'
            for n in names
        )
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
        toc_html="<p>Open a generated manpage mirror or jump back to the command-reference lane.</p>",
        related_html=html_list(related),
        version_html=render_version_links(output_rel, config),
        locale_html="<p>English only. This lane currently generates from English command docs.</p>",
        footer_nav_html="",
        footer_html='Generated from <code>docs/man/*.1</code> via <code>scripts/generate_manpages.py</code>.',
        jump_html=render_jump_select(output_rel, "en", config),
        nav_html=render_global_nav(output_rel, "en", config),
    )


ROFF_FONT_TOKEN_RE = re.compile(r"\\f[BRI]|\\fR")


def normalize_roff_text(text: str) -> str:
    return text.replace(r"\-", "-").replace(r"\(bu", "•")


def render_roff_inline(text: str) -> str:
    pieces = []
    stack = []
    cursor = 0
    for match in ROFF_FONT_TOKEN_RE.finditer(text):
        if match.start() > cursor:
            pieces.append(html.escape(normalize_roff_text(text[cursor:match.start()])))
        token = match.group(0)
        if token == r"\fB":
            pieces.append("<strong>")
            stack.append("strong")
        elif token == r"\fI":
            pieces.append("<em>")
            stack.append("em")
        elif token == r"\fR" and stack:
            pieces.append(f"</{stack.pop()}>")
        cursor = match.end()
    if cursor < len(text):
        pieces.append(html.escape(normalize_roff_text(text[cursor:])))
    while stack:
        pieces.append(f"</{stack.pop()}>")
    return "".join(pieces)


def render_roff_macro_text(line: str) -> str:
    if line.startswith(".B "):
        return f"<strong>{render_roff_inline(line[3:])}</strong>"
    if line.startswith(".I "):
        return f"<em>{render_roff_inline(line[3:])}</em>"
    return render_roff_inline(line)


def render_roff_manpage_html(roff_text_body: str) -> str:
    body_parts = []
    section_parts = []
    paragraph_lines = []
    bullet_items = []
    definition_items = []
    definition_term = None
    definition_desc = []
    code_lines = []
    current_heading = None
    in_code_block = False
    pending_bullet = False
    expecting_definition_term = False

    def flush_paragraph():
        nonlocal paragraph_lines
        if paragraph_lines:
            section_parts.append(
                "<p>"
                + " ".join(
                    render_roff_macro_text(line) if line.startswith((".B ", ".I ")) else render_roff_inline(line)
                    for line in paragraph_lines
                )
                + "</p>"
            )
            paragraph_lines = []

    def flush_bullets():
        nonlocal bullet_items
        if bullet_items:
            section_parts.append('<ul class="man-bullets">' + "".join(f"<li>{item}</li>" for item in bullet_items) + "</ul>")
            bullet_items = []

    def flush_definition():
        nonlocal definition_term, definition_desc
        if definition_term is not None:
            definition_items.append((definition_term, " ".join(render_roff_inline(line) for line in definition_desc).strip()))
            definition_term = None
            definition_desc = []

    def flush_definitions():
        nonlocal definition_items
        flush_definition()
        if definition_items:
            section_parts.append(
                '<dl class="man-definitions">'
                + "".join(f"<dt>{term}</dt><dd>{desc}</dd>" for term, desc in definition_items)
                + "</dl>"
            )
            definition_items = []

    def flush_code():
        nonlocal code_lines
        if code_lines:
            section_parts.append(f'<pre class="man-example"><code>{html.escape(chr(10).join(code_lines))}</code></pre>')
            code_lines = []

    def flush_section_content():
        flush_paragraph()
        flush_bullets()
        flush_definitions()
        flush_code()

    def emit_section():
        nonlocal section_parts
        flush_section_content()
        if current_heading is not None:
            body_parts.append(f'<section class="man-section"><h2>{html.escape(current_heading)}</h2>{"".join(section_parts)}</section>')
            section_parts = []

    for raw_line in roff_text_body.splitlines():
        line = raw_line.rstrip()
        if in_code_block:
            if line == ".EE":
                in_code_block = False
                flush_code()
            else:
                code_lines.append(line)
            continue
        if pending_bullet:
            bullet_items.append(render_roff_inline(line))
            pending_bullet = False
            continue
        if expecting_definition_term:
            definition_term = render_roff_macro_text(line)
            definition_desc = []
            expecting_definition_term = False
            continue
        if line.startswith('.\\"') or line.startswith(".TH"):
            continue
        if line.startswith(".SH "):
            emit_section()
            current_heading = normalize_roff_text(line[4:])
            continue
        if line == ".PP":
            flush_paragraph()
            flush_bullets()
            flush_definitions()
            continue
        if line.startswith(".IP "):
            flush_paragraph()
            flush_definitions()
            pending_bullet = True
            continue
        if line == ".TP":
            flush_paragraph()
            flush_bullets()
            flush_definition()
            expecting_definition_term = True
            continue
        if line == ".EX":
            flush_paragraph()
            flush_bullets()
            flush_definitions()
            in_code_block = True
            code_lines = []
            continue
        if definition_term is not None:
            definition_desc.append(line)
        else:
            paragraph_lines.append(line)

    emit_section()
    if not body_parts and section_parts:
        body_parts.extend(section_parts)
    return '<div class="manpage-rendered">' + "".join(body_parts) + "</div>"

def render_manpage_page(output_rel, name, roff, config):
    stem = Path(name).stem
    command_root = command_reference_root_for_stem(stem, config)
    related = [("Manpage index", relative_href(output_rel, prefixed_output_rel(config, "man/index.html")))]
    if config.raw_manpage_target_rel:
        related.append(("Raw roff source", relative_href(output_rel, prefixed_output_rel(config, f"man/{name}"))))
    if command_root:
        related.append(("Matching command reference", relative_href(output_rel, command_root)))
    body = (
        "<p>This page renders the generated roff manpage into readable HTML for browser use. "
        "For richer per-subcommand detail, prefer the command-reference pages.</p>"
        f"{render_roff_manpage_html(roff)}"
    )
    return page_shell(
        page_title=name,
        html_lang="en",
        home_href=relative_href(output_rel, prefixed_output_rel(config, "index.html")),
        hero_title=name,
        hero_summary="Browser-readable rendering of a generated roff manpage.",
        breadcrumbs=[
            ("Home", relative_href(output_rel, prefixed_output_rel(config, "index.html"))),
            ("Manpages", relative_href(output_rel, prefixed_output_rel(config, "man/index.html"))),
            (name, None),
        ],
        body_html=body,
        toc_html="<p>This page renders the generated manpage into readable HTML sections.</p>",
        related_html=html_list(related),
        version_html=render_version_links(output_rel, config),
        locale_html="<p>English only. The manpage lane currently generates from English command docs.</p>",
        footer_nav_html="",
        footer_html=f'Source: <code>docs/man/{html.escape(name)}</code><br>Generated by <code>scripts/generate_command_html.py</code>.',
        jump_html=render_jump_select(output_rel, "en", config),
        nav_html=render_global_nav(output_rel, "en", config),
    )

def landing_panel_html(title: str, summary: str, links: list[tuple[str, str]]) -> str:
    return f"""
    <section class="landing-panel">
      <h2>{html.escape(title)}</h2>
      <p>{html.escape(summary)}</p>
      <ul class="landing-link-list">
        {''.join(f'<li><a href="{html.escape(href)}">{html.escape(label)}</a></li>' for label, href in links)}
      </ul>
    </section>
    """

def render_landing_links(source_path: Path, output_rel: str, links: tuple[LandingLink, ...], config: HtmlBuildConfig) -> list[tuple[str, str]]:
    return [(link.label, rewrite_markdown_link(source_path, output_rel, link.target, config)) for link in links]

def render_landing_task(source_path: Path, output_rel: str, task: LandingTask, config: HtmlBuildConfig) -> str:
    rendered_links = render_landing_links(source_path, output_rel, task.links, config)
    return f"""
        <li class="landing-task">
          <h3 class="landing-task-title">{html.escape(task.title)}</h3>
          <p class="landing-task-copy">{html.escape(task.summary)}</p>
          <ul class="landing-task-links">
            {''.join(f'<li><a href="{html.escape(href)}">{html.escape(label)}</a></li>' for label, href in rendered_links)}
          </ul>
        </li>
    """

def render_landing_section(source_path: Path, output_rel: str, section: LandingSection, config: HtmlBuildConfig) -> str:
    inline_html = ""
    inline_links = render_landing_links(source_path, output_rel, section.links, config)
    if inline_links:
        inline_html = (
            '<ul class="landing-inline-links">'
            + "".join(f'<li><a href="{html.escape(href)}">{html.escape(label)}</a></li>' for label, href in inline_links)
            + "</ul>"
        )
    return f"""
    <section class="landing-section">
      <h2>{html.escape(section.title)}</h2>
      <p>{html.escape(section.summary)}</p>
      {inline_html}
      <ul class="landing-task-list">
        {''.join(render_landing_task(source_path, output_rel, task, config) for task in section.tasks)}
      </ul>
    </section>
    """

def build_landing_locale_data(config: HtmlBuildConfig) -> dict[str, dict[str, str]]:
    landing_rel = prefixed_output_rel(config, "index.html")
    version_links = [(config.version_label or config.version, "#")]
    version_links.extend((link.label, relative_href(landing_rel, link.target_rel)) for link in config.version_links)
    if config.include_raw_manpages or config.raw_manpage_target_rel:
        version_links.append(("Manpages", relative_href(landing_rel, prefixed_output_rel(config, "man/index.html"))))

    landing_data: dict[str, dict[str, str]] = {}
    for locale in LANDING_LOCALES:
        page = load_landing_page(locale)
        ui_labels = LANDING_UI_LABELS[locale]
        maintainer_links = render_landing_links(page.source_path, landing_rel, page.maintainer.links, config)
        meta_html = "".join(
            (
                landing_panel_html(page.maintainer.title, page.maintainer.summary, maintainer_links),
                landing_panel_html("Version" if locale == "en" else "版本", "Version switching is secondary here. Pick a language first, then jump release context if you need it." if locale == "en" else "版本切換是次要操作。先選語言，再視需要跳去特定版本內容。", version_links),
            )
        )
        landing_data[locale] = {
            "lang": locale,
            "hero_title": page.title,
            "hero_summary": page.summary,
            "search_heading": page.search.title,
            "search_copy": page.search.summary,
            "search_placeholder": ui_labels["search_placeholder"],
            "search_button": ui_labels["search_button"],
            "sections_html": "".join(render_landing_section(page.source_path, landing_rel, section, config) for section in page.sections),
            "meta_html": meta_html,
            "jump_options_html": render_jump_select_options(landing_rel, locale, config),
        }
    return landing_data

def render_landing_page(config):
    landing_data = build_landing_locale_data(config)
    default_locale = "en"
    copy = landing_data[default_locale]
    body = f"""
<div class="landing-page">
  <section class="landing-hero">
    <div class="landing-hero-inner">
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
    return page_shell(
        page_title="grafana-util Docs",
        html_lang="en",
        home_href=relative_href(prefixed_output_rel(config, "index.html"), prefixed_output_rel(config, "index.html")),
        hero_title="",
        hero_summary="",
        breadcrumbs=[],
        body_html=body,
        toc_html="",
        related_html="",
        version_html="",
        locale_html="",
        footer_nav_html="",
        footer_html="Generated by scripts/generate_command_html.py",
        jump_html=render_landing_locale_select(default_locale) + render_jump_select(prefixed_output_rel(config, "index.html"), default_locale, config),
        nav_html="",
        is_landing=True,
    )

def render_developer_page(config):
    dev_path = config.source_root / "DEVELOPER.md"
    if not dev_path.exists(): return ""
    doc = render_markdown_document(dev_path.read_text(encoding="utf-8"), link_transform=lambda t: t)
    return page_shell(page_title="Developer Guide", html_lang="en", home_href=relative_href("developer.html", prefixed_output_rel(config, "index.html")), hero_title="Developer Guide", hero_summary="Maintainer and developer documentation.", breadcrumbs=[("Home", "index.html"), ("Developer Guide", None)], body_html=strip_heading_decorations(strip_leading_h1(doc.body_html)), toc_html=render_toc(doc.headings), related_html="", version_html="", locale_html="", footer_nav_html="", footer_html="Source: DEVELOPER.md", jump_html=render_jump_select("developer.html", "en", config), nav_html="")

def command_language_switch_href(output_rel, locale, source_name, config):
    other = "zh-TW" if locale == "en" else "en"
    target = config.command_docs_root / other / source_name
    if not target.exists(): return None, None
    return LOCALE_LABELS[other], relative_href(output_rel, prefixed_output_rel(config, f"commands/{other}/{Path(source_name).with_suffix('.html').as_posix()}"))

def command_handbook_context(locale, output_rel, source_name, config):
    stem = Path(source_name).stem
    root = stem.split("-", 1)[0]
    h_stem = HANDBOOK_CONTEXT_BY_COMMAND.get(stem) or HANDBOOK_CONTEXT_BY_COMMAND.get(root)
    if not h_stem: return None
    return ("Matching handbook chapter", relative_href(output_rel, prefixed_output_rel(config, f"handbook/{locale}/{h_stem}.html")))

def rewrite_markdown_link(source_path, output_rel, target, config):
    if target.startswith(("http", "mailto:", "#")): return target
    bare, frag = (target.split("#", 1) + [""])[:2]
    res = (source_path.parent / bare).resolve()
    if res == (config.source_root / "DEVELOPER.md").resolve():
        rel = "developer.html"
    else:
        try:
            rel = res.relative_to(config.source_root / "docs").as_posix()
        except Exception:
            return target
    if rel.startswith("commands/") and rel.endswith(".md"): rel = rel[:-3] + ".html"
    elif rel.startswith("user-guide/") and rel.endswith(".md"): rel = rel.replace("user-guide/", "handbook/", 1)[:-3] + ".html"
    return f"{relative_href(output_rel, prefixed_output_rel(config, rel))}#{frag}" if frag else relative_href(output_rel, prefixed_output_rel(config, rel))

def render_handbook_page(page, config):
    doc = render_markdown_document(page.source_path.read_text(encoding="utf-8"), link_transform=lambda t: rewrite_markdown_link(page.source_path, page.output_rel, t, config))
    title = title_only(doc.title or page.title)
    locale_href = handbook_language_href(page)
    locale_label = LOCALE_LABELS["zh-TW" if page.locale=="en" else "en"] if locale_href else None
    prev = (title_only(page.previous_title), relative_href(page.output_rel, page.previous_output_rel)) if page.previous_output_rel else None
    nxt = (title_only(page.next_title), relative_href(page.output_rel, page.next_output_rel)) if page.next_output_rel else None
    return page_shell(page_title=title, html_lang=page.locale, home_href=relative_href(page.output_rel, prefixed_output_rel(config, "index.html")), hero_title=title, hero_summary=handbook_intro_text(page.locale), breadcrumbs=[("Home", relative_href(page.output_rel, "index.html")), ("Handbook", None), (title, None)], body_html=strip_heading_decorations(strip_leading_h1(doc.body_html)), toc_html=render_toc(doc.headings), related_html="", version_html=render_version_links(page.output_rel, config), locale_html=render_language_links(LOCALE_LABELS[page.locale], locale_label, locale_href), footer_nav_html=render_footer_nav(prev, nxt), footer_html="Source: " + page.source_path.name, jump_html=render_jump_select(page.output_rel, page.locale, config), nav_html=render_global_nav(page.output_rel, page.locale, config))

def render_command_page(locale, source_path, output_rel, config):
    doc = render_markdown_document(source_path.read_text(encoding="utf-8"), link_transform=lambda t: rewrite_markdown_link(source_path, output_rel, t, config))
    title = title_only(doc.title or source_path.stem)
    sw_l, sw_h = command_language_switch_href(output_rel, locale, source_path.name, config)
    h_link = command_handbook_context(locale, output_rel, source_name=source_path.name, config=config)
    return page_shell(page_title=title, html_lang=locale, home_href=relative_href(output_rel, prefixed_output_rel(config, "index.html")), hero_title=title, hero_summary=command_intro_text(locale), breadcrumbs=[("Home", relative_href(output_rel, prefixed_output_rel(config, "index.html"))), ("Commands", None), (title, None)], body_html=strip_heading_decorations(strip_leading_h1(doc.body_html)), toc_html=render_toc(doc.headings), related_html=html_list([h_link]) if h_link else "", version_html=render_version_links(output_rel, config), locale_html=render_language_links(LOCALE_LABELS[locale], sw_l, sw_h), footer_nav_html="", footer_html="Source: " + source_path.name, jump_html=render_jump_select(output_rel, locale, config), nav_html=render_global_nav(output_rel, locale, config))

def generate_outputs(config=HtmlBuildConfig()):
    outputs = {prefixed_output_rel(config, "index.html"): render_landing_page(config), ".nojekyll": ""}
    dev_html = render_developer_page(config)
    if dev_html: outputs[prefixed_output_rel(config, "developer.html")] = dev_html
    manpage_outputs = generate_manpages(command_docs_dir=config.command_docs_root / "en", version=config.version)
    outputs[prefixed_output_rel(config, "man/index.html")] = render_manpage_index_page(prefixed_output_rel(config, "man/index.html"), sorted(manpage_outputs), config)
    for name, roff in sorted(manpage_outputs.items()):
        outputs[prefixed_output_rel(config, f"man/{Path(name).stem}.html")] = render_manpage_page(prefixed_output_rel(config, f"man/{Path(name).stem}.html"), name, roff, config)
        if config.include_raw_manpages:
            outputs[prefixed_output_rel(config, f"man/{name}")] = roff
    for loc in COMMAND_DOC_LOCALES:
        for src in sorted((config.command_docs_root / loc).glob("*.md")):
            out_rel = prefixed_output_rel(config, f"commands/{loc}/{src.with_suffix('.html').name}")
            outputs[out_rel] = render_command_page(loc, src, out_rel, config)
    for loc in HANDBOOK_LOCALES:
        for page in build_handbook_pages(loc, handbook_root=config.handbook_root):
            page = replace(page, output_rel=prefixed_output_rel(config, page.output_rel), previous_output_rel=prefixed_output_rel(config, page.previous_output_rel) if page.previous_output_rel else None, next_output_rel=prefixed_output_rel(config, page.next_output_rel) if page.next_output_rel else None, language_switch_rel=prefixed_output_rel(config, page.language_switch_rel) if page.language_switch_rel else None)
            outputs[page.output_rel] = render_handbook_page(page, config)
    return outputs

def build_parser():
    parser = argparse.ArgumentParser(); parser.add_argument("--write", action="store_true"); parser.add_argument("--check", action="store_true"); return parser

def main(argv=None):
    args = build_parser().parse_args(argv)
    outputs = generate_outputs()
    if args.check: return check_outputs(HTML_ROOT_DIR, outputs, "HTML docs", "python3 scripts/generate_command_html.py --write")
    write_outputs(HTML_ROOT_DIR, outputs)
    print_written_outputs(HTML_ROOT_DIR, outputs, "HTML docs", "docs/*", "docs/html/*", "docs/html/index.html")
    return 0

if __name__ == "__main__": raise SystemExit(main())
