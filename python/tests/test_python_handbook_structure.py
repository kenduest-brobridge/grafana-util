import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


class HandbookStructureTests(unittest.TestCase):
    def test_zh_tw_handbook_pages_include_audience_and_goal_sections(self):
        handbook_dir = REPO_ROOT / "docs" / "user-guide" / "zh-TW"
        exempt = {"index.md", "what-is-grafana-util.md"}

        for path in sorted(handbook_dir.glob("*.md")):
            if path.name in exempt:
                continue
            text = path.read_text(encoding="utf-8")
            self.assertIn("## 適用對象", text, path.name)
            self.assertIn("## 主要目標", text, path.name)

    def test_en_handbook_pages_include_audience_and_goal_sections(self):
        handbook_dir = REPO_ROOT / "docs" / "user-guide" / "en"
        exempt = {"index.md", "what-is-grafana-util.md"}

        for path in sorted(handbook_dir.glob("*.md")):
            if path.name in exempt:
                continue
            text = path.read_text(encoding="utf-8")
            self.assertIn("## Who It Is For", text, path.name)
            self.assertIn("## Primary Goals", text, path.name)


if __name__ == "__main__":
    unittest.main()
