import importlib.util
import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "scripts" / "generate_command_html.py"
SCRIPTS_DIR = REPO_ROOT / "scripts"


def load_module():
    if str(SCRIPTS_DIR) not in sys.path:
        sys.path.insert(0, str(SCRIPTS_DIR))
    spec = importlib.util.spec_from_file_location("generate_command_html", SCRIPT_PATH)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    sys.modules.setdefault("generate_command_html", module)
    spec.loader.exec_module(module)
    return module


class GenerateCommandHtmlTests(unittest.TestCase):
    def test_generated_html_matches_checked_in_outputs(self):
        module = load_module()

        generated = module.generate_outputs()
        html_root = REPO_ROOT / "docs" / "html"
        checked_in = {
            path.relative_to(html_root).as_posix(): path.read_text(encoding="utf-8")
            for path in sorted(html_root.rglob("*.html"))
        }
        nojekyll_path = html_root / ".nojekyll"
        if nojekyll_path.exists():
            checked_in[".nojekyll"] = nojekyll_path.read_text(encoding="utf-8")

        self.assertEqual(set(generated), set(checked_in))
        self.assertEqual(generated, checked_in)

    def test_generate_outputs_supports_versioned_lane(self):
        module = load_module()

        config = module.HtmlBuildConfig(
            source_root=REPO_ROOT,
            command_docs_root=REPO_ROOT / "docs" / "commands",
            handbook_root=REPO_ROOT / "docs" / "user-guide",
            output_prefix="v9.9",
            version="9.9.0",
            version_label="v9.9",
            version_links=(
                module.VersionLink("Version portal", "index.html"),
                module.VersionLink("Latest release", "latest/index.html"),
            ),
            raw_manpage_target_rel="v9.9/man/grafana-util.1",
            include_raw_manpages=True,
        )

        generated = module.generate_outputs(config)

        self.assertIn("v9.9/index.html", generated)
        self.assertIn("v9.9/man/index.html", generated)
        self.assertIn("v9.9/man/grafana-util.1", generated)
        self.assertIn("Current: v9.9", generated["v9.9/commands/en/dashboard.html"])
        self.assertIn("../index.html", generated["v9.9/commands/en/dashboard.html"])

    def test_render_landing_page_uses_structured_sections(self):
        module = load_module()

        config = module.HtmlBuildConfig(
            source_root=REPO_ROOT,
            command_docs_root=REPO_ROOT / "docs" / "commands",
            handbook_root=REPO_ROOT / "docs" / "user-guide",
            version="9.9.0",
            version_label="v9.9",
            version_links=(module.VersionLink("Latest", "latest/index.html"),),
        )

        rendered = module.render_landing_page(config)

        self.assertIn('class="landing-hero"', rendered)
        self.assertIn('id="landing-search-form"', rendered)
        self.assertIn('id="locale-select"', rendered)
        self.assertIn('id="landing-i18n"', rendered)
        self.assertIn("Quick Start", rendered)
        self.assertIn("Common Tasks", rendered)
        self.assertIn("Reference", rendered)
        self.assertIn("Export dashboards", rendered)
        self.assertIn("快速開始", rendered)
        self.assertIn("繁體中文", rendered)
        self.assertIn("Developer guide", rendered)
        self.assertIn("latest/index.html", rendered)

    def test_render_manpage_page_renders_structured_html(self):
        module = load_module()

        config = module.HtmlBuildConfig(version="9.9.0")
        roff = "\n".join(
            (
                '.TH TEST 1 "2026-04-04" "grafana-util 9.9.0" "User Commands"',
                ".SH NAME",
                r"grafana\-util\-test \- sample command",
                ".SH DESCRIPTION",
                r"Run \fBgrafana-util test\fR with readable HTML output.",
                r".IP \(bu 2",
                "First bullet",
                ".TP",
                ".B --flag",
                "Flag description",
                ".SH EXAMPLES",
                ".EX",
                "grafana-util test --flag",
                ".EE",
            )
        )

        rendered = module.render_manpage_page("man/test.html", "grafana-util-test.1", roff, config)

        self.assertIn('<div class="manpage-rendered">', rendered)
        self.assertIn("<h2>DESCRIPTION</h2>", rendered)
        self.assertIn("<strong>grafana-util test</strong>", rendered)
        self.assertIn('<ul class="man-bullets">', rendered)
        self.assertIn('<dl class="man-definitions">', rendered)
        self.assertIn('<pre class="man-example"><code>grafana-util test --flag</code></pre>', rendered)
        self.assertNotIn('<pre class="manpage">', rendered)

    def test_strip_leading_h1_removes_duplicate_document_title(self):
        module = load_module()

        stripped = module.strip_leading_h1('<h1 id="title">Title</h1>\n<p>Body</p>')

        self.assertEqual(stripped, "<p>Body</p>")

    def test_split_display_title_separates_english_subtitle_for_mixed_heading(self):
        module = load_module()

        main, secondary = module.split_display_title("📖 維運導引手冊 (Operator Handbook)")

        self.assertEqual(main, "維運導引手冊")
        self.assertEqual(secondary, "Operator Handbook")

    def test_strip_heading_decorations_removes_leading_emoji_from_heading_html(self):
        module = load_module()

        rendered = module.strip_heading_decorations('<h2 id="quick-start">⚡ 30 秒快速上手 (Quick Start)</h2><p>Body</p>')

        self.assertIn(">30 秒快速上手 (Quick Start)<", rendered)
        self.assertNotIn("⚡", rendered)

    def test_render_toc_adds_hierarchy_classes_and_strips_emoji_labels(self):
        module = load_module()

        headings = (
            module.RenderedHeading(level=2, text="⚡ 30 秒快速上手 (Quick Start)", anchor="quick-start"),
            module.RenderedHeading(level=3, text="安裝 (全域 Binary)", anchor="install"),
        )

        rendered = module.render_toc(headings)

        self.assertIn('class="toc-level-2"', rendered)
        self.assertIn('class="toc-level-3"', rendered)
        self.assertIn('30 秒快速上手 (Quick Start)', rendered)
        self.assertIn('href="#install"', rendered)
        self.assertNotIn("⚡", rendered)

    def test_render_global_nav_uses_localized_handbook_titles(self):
        module = load_module()

        config = module.HtmlBuildConfig(
            source_root=REPO_ROOT,
            command_docs_root=REPO_ROOT / "docs" / "commands",
            handbook_root=REPO_ROOT / "docs" / "user-guide",
            version="9.9.0",
        )

        rendered = module.render_global_nav("handbook/zh-TW/index.html", "zh-TW", config)

        self.assertIn("開始", rendered)
        self.assertIn("角色路徑", rendered)
        self.assertIn("核心操作", rendered)
        self.assertIn("實戰與參考", rendered)
        self.assertIn("開始使用", rendered)
        self.assertIn("新手快速入門", rendered)
        self.assertIn("Dashboard 維運人員手冊", rendered)
        self.assertIn('class="nav-group-block"', rendered)
        self.assertIn('class="nav-group-title"', rendered)
        self.assertNotIn(">Getting Started<", rendered)
        self.assertNotIn(">Role New User<", rendered)

    def test_render_global_nav_renders_single_command_reference_entry(self):
        module = load_module()

        config = module.HtmlBuildConfig(
            source_root=REPO_ROOT,
            command_docs_root=REPO_ROOT / "docs" / "commands",
            handbook_root=REPO_ROOT / "docs" / "user-guide",
            version="9.9.0",
        )

        rendered = module.render_global_nav("commands/zh-TW/datasource.html", "zh-TW", config)

        self.assertIn(">開啟指令參考<", rendered)
        self.assertIn('class="nav-command-entry active"', rendered)
        self.assertNotIn(">Datasource<", rendered)
        self.assertNotIn(">Dashboard<", rendered)
        self.assertNotIn("Grafana-util-datasource", rendered)


if __name__ == "__main__":
    unittest.main()
