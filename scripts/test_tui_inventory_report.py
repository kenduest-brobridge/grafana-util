from __future__ import annotations

import json
import unittest
from pathlib import Path

from scripts.tui_inventory_report import (
    REGISTRY_PATH,
    SCAN_ROOTS,
    collect_helper_drift,
    check_registry,
    load_registry,
    build_inventory,
    build_surface_summary,
)


class TuiInventoryReportTest(unittest.TestCase):
    def test_collect_helper_drift_reports_local_tui_delegate_helpers(self) -> None:
        text = """
use crate::tui_shell;

fn control_line(items: &[(&str, Color, &str)]) -> Line<'static> {
    tui_shell::fixed_body_control_line(items, 14)
}
"""

        drift = collect_helper_drift(Path("rust/src/commands/datasource/browse/render.rs"), text)

        self.assertEqual(len(drift), 1)
        self.assertEqual(drift[0].helper, "control_line")
        self.assertEqual(drift[0].line, 4)
        self.assertIn("tui_shell", drift[0].signal)

    def test_collect_helper_drift_ignores_shared_tui_shell_helpers(self) -> None:
        text = """
pub(crate) fn control_line(items: &[(&str, Color, &str)]) -> Line<'static> {
    build_control_line(items, &[])
}
"""

        drift = collect_helper_drift(Path("rust/src/common/tui/shell.rs"), text)

        self.assertEqual(drift, [])

    def test_load_registry_returns_valid_structure(self) -> None:
        registry = load_registry()
        self.assertIn("schema", registry)
        self.assertIn("surfaces", registry)
        self.assertIsInstance(registry["surfaces"], list)
        for entry in registry["surfaces"]:
            self.assertIn("command", entry)
            self.assertIn("domain", entry)
            self.assertIn("tier", entry)
            self.assertIn("feature_gate", entry)
            self.assertIn("has_tests", entry)

    def test_check_registry_finds_no_critical_errors_in_live_data(self) -> None:
        items = build_inventory()
        findings = check_registry(items)
        self.assertEqual([], findings)

    def test_scan_roots_include_zh_tw(self) -> None:
        root_strs = [root.as_posix() for root in SCAN_ROOTS]
        self.assertIn("docs/commands/zh-TW", root_strs)
        self.assertIn("docs/user-guide/zh-TW", root_strs)

    def test_registry_json_is_valid_json(self) -> None:
        text = REGISTRY_PATH.read_text(encoding="utf-8")
        data = json.loads(text)
        self.assertIsInstance(data, dict)
        self.assertGreater(len(data["surfaces"]), 0)

    def test_registry_surfaces_have_required_fields(self) -> None:
        registry = load_registry()
        for entry in registry["surfaces"]:
            with self.subTest(command=entry["command"]):
                self.assertIsInstance(entry["command"], str)
                self.assertIsInstance(entry["owner"], str)
                self.assertIn(entry["entrypoint_kind"], {"flag", "output-format", "implicit"})
                self.assertIn(entry["tier"], {2, 3, 4})
                self.assertIsInstance(entry["validation"], str)
                self.assertIsInstance(entry["has_tests"], bool)
                self.assertIn(entry["feature_gate"], {"tui", "browser"})

    def test_registry_surfaces_record_operator_friendliness(self) -> None:
        registry = load_registry()
        for entry in registry["surfaces"]:
            with self.subTest(command=entry["command"]):
                friendliness = entry["operator_friendliness"]
                self.assertIsInstance(friendliness["summary"], str)
                self.assertIsInstance(friendliness["search"], str)
                self.assertIsInstance(friendliness["detail_scroll"], str)
                self.assertIsInstance(friendliness["footer"], str)
                self.assertIsInstance(friendliness["confirmation"], str)
                self.assertIsInstance(friendliness["secret_redaction"], str)
                self.assertIsInstance(friendliness["blockers"], str)

    def test_surface_summary_reports_registry_coverage(self) -> None:
        registry = load_registry()
        items = build_inventory()
        summary = build_surface_summary(registry, items)

        commands = {entry["command"] for entry in registry["surfaces"]}
        self.assertEqual(commands, {entry["command"] for entry in summary})
        for entry in summary:
            with self.subTest(command=entry["command"]):
                self.assertIn("owner", entry)
                self.assertIn("docsDetected", entry)
                self.assertIn("codeDetected", entry)
                self.assertIn("operatorFriendliness", entry)
                self.assertIn("validation", entry)

    def test_zh_tw_docs_are_scanned(self) -> None:
        items = build_inventory()
        zh_tw_docs = [i.path for i in items if i.path.startswith("docs/commands/zh-TW/")]
        self.assertGreater(len(zh_tw_docs), 0, "expected zh-TW command docs to appear in scan")


if __name__ == "__main__":
    unittest.main()
