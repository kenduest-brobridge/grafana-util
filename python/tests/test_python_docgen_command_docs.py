import importlib.util
import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "scripts" / "docgen_command_docs.py"
SCRIPTS_DIR = REPO_ROOT / "scripts"


def load_module():
    if str(SCRIPTS_DIR) not in sys.path:
        sys.path.insert(0, str(SCRIPTS_DIR))
    spec = importlib.util.spec_from_file_location("docgen_command_docs", SCRIPT_PATH)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    sys.modules.setdefault("docgen_command_docs", module)
    spec.loader.exec_module(module)
    return module


class DocgenCommandDocsTests(unittest.TestCase):
    def test_parse_command_page_supports_zh_tw_section_labels(self):
        module = load_module()
        source = REPO_ROOT / "docs" / "commands" / "zh-TW" / "dashboard-screenshot.md"

        parsed = module.parse_command_page(source, "grafana-util dashboard screenshot")

        self.assertIn("事件處理紀錄", parsed.when)
        self.assertIn("PDF", parsed.purpose)
        self.assertTrue(parsed.key_flags)
        self.assertTrue(parsed.examples)

    def test_render_markdown_document_renders_ordered_lists(self):
        module = load_module()

        rendered = module.render_markdown_document(
            "# Title\n\n1. First step\n2. Second step\n\n- Bullet\n"
        )

        self.assertIn("<ol><li>First step</li><li>Second step</li></ol>", rendered.body_html)
        self.assertIn("<ul><li>Bullet</li></ul>", rendered.body_html)

    def test_render_markdown_document_keeps_multiline_shell_fence_in_one_code_block(self):
        module = load_module()

        rendered = module.render_markdown_document(
            "# Title\n\n```bash\ngrafana-util dashboard list \\\n  --url http://localhost:3000 \\\n  --basic-user admin \\\n  --basic-password admin \\\n  --table\n```\n"
        )

        self.assertIn("grafana-util dashboard list \\", rendered.body_html)
        self.assertIn("--basic-user admin \\", rendered.body_html)
        self.assertEqual(rendered.body_html.count("<pre><code>"), 1)


if __name__ == "__main__":
    unittest.main()
