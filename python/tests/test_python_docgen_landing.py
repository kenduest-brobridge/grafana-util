import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "scripts" / "docgen_landing.py"
SCRIPTS_DIR = REPO_ROOT / "scripts"


def load_module():
    if str(SCRIPTS_DIR) not in sys.path:
        sys.path.insert(0, str(SCRIPTS_DIR))
    spec = importlib.util.spec_from_file_location("docgen_landing", SCRIPT_PATH)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    sys.modules.setdefault("docgen_landing", module)
    spec.loader.exec_module(module)
    return module


class DocgenLandingTests(unittest.TestCase):
    def test_load_landing_page_parses_english_source(self):
        module = load_module()

        page = module.load_landing_page("en")

        self.assertEqual(page.title, "grafana-util Docs")
        self.assertEqual(page.search.title, "Quick jump")
        self.assertEqual(len(page.sections), 4)
        self.assertEqual(page.sections[0].title, "Quick Start")
        self.assertEqual(page.sections[0].tasks[0].title, "First connection and read-only checks")
        self.assertEqual(page.sections[0].tasks[0].links[0].target, "../user-guide/en/getting-started.md")
        self.assertEqual(page.maintainer.links[0].target, "../DEVELOPER.md")

    def test_parse_landing_text_requires_search_and_maintainer_sections(self):
        module = load_module()

        with self.assertRaisesRegex(ValueError, "Search and Maintainer sections"):
            module.parse_landing_text(
                "# grafana-util Docs\n\nSummary only.\n",
                locale="en",
                source_path=REPO_ROOT / "docs" / "landing" / "broken.md",
            )

    def test_load_landing_page_falls_back_to_repo_landing_text_for_missing_version_lane_file(self):
        module = load_module()

        with tempfile.TemporaryDirectory() as tmp_dir:
            landing_root = Path(tmp_dir)
            page = module.load_landing_page("en", landing_root=landing_root)

            self.assertEqual(page.title, "grafana-util Docs")
            self.assertEqual(page.source_path, landing_root / "en.md")
            self.assertEqual(page.maintainer.links[0].target, "../DEVELOPER.md")


if __name__ == "__main__":
    unittest.main()
