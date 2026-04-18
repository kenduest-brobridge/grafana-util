import importlib
import io
import json
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_ROOT = REPO_ROOT / "python"
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(PYTHON_ROOT))

dashboard_cli = importlib.import_module("grafana_utils.dashboard_cli")


class DashboardLatestSurfacesTests(unittest.TestCase):
    def test_dashboard_list_vars_canonical_command_is_normalized(self):
        args = dashboard_cli.parse_args(
            ["variables", "--dashboard-uid", "cpu-main", "--output-format", "json"]
        )

        self.assertEqual(args.command, "variables")
        self.assertEqual(args.dashboard_uid, "cpu-main")
        self.assertEqual(args.output_format, "json")

    def test_dashboard_raw_to_prompt_converts_single_raw_dashboard_file(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            input_file = root / "cpu-main.json"
            output_dir = root / "prompt"
            dashboard_cli.write_json_document(
                {
                    "dashboard": {
                        "id": None,
                        "uid": "cpu-main",
                        "title": "CPU Main",
                        "panels": [],
                    },
                    "meta": {"folderTitle": "Infra"},
                },
                input_file,
            )
            args = dashboard_cli.parse_args(
                [
                    "raw-to-prompt",
                    "--input-file",
                    str(input_file),
                    "--output-dir",
                    str(output_dir),
                    "--overwrite",
                ]
            )

            stdout = io.StringIO()
            with redirect_stdout(stdout):
                result = dashboard_cli.raw_to_prompt_command(args)

            self.assertEqual(result, 0)
            output_file = output_dir / input_file.name
            self.assertTrue(output_file.exists())
            document = json.loads(output_file.read_text(encoding="utf-8"))
            self.assertEqual(document["uid"], "cpu-main")
            self.assertEqual(document["title"], "CPU Main")
            self.assertIn("raw-to-prompt completed", stdout.getvalue())

    def test_dashboard_impact_command_reports_blast_radius(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            governance_file = root / "governance.json"
            queries_file = root / "queries.json"
            output_file = root / "impact.json"
            dashboard_cli.write_json_document(
                {
                    "datasourceInventory": [
                        {
                            "uid": "prom-main",
                            "name": "Prometheus Main",
                            "type": "prometheus",
                            "orgId": "1",
                            "referenceCount": 1,
                        }
                    ]
                },
                governance_file,
            )
            dashboard_cli.write_json_document(
                {
                    "queries": [
                        {
                            "dashboardUid": "cpu-main",
                            "dashboardTitle": "CPU Main",
                            "folderPath": "General",
                            "panelId": "7",
                            "panelTitle": "CPU Usage",
                            "panelType": "timeseries",
                            "datasource": "prom-main",
                            "datasourceUid": "prom-main",
                        }
                    ]
                },
                queries_file,
            )
            args = dashboard_cli.parse_args(
                [
                    "impact",
                    "--governance",
                    str(governance_file),
                    "--queries",
                    str(queries_file),
                    "--datasource-uid",
                    "prom-main",
                    "--output-format",
                    "json",
                    "--output-file",
                    str(output_file),
                ]
            )

            stdout = io.StringIO()
            with redirect_stdout(stdout):
                result = dashboard_cli.impact_command(args)

            self.assertEqual(result, 0)
            payload = json.loads(stdout.getvalue())
            self.assertEqual(payload["datasourceUid"], "prom-main")
            self.assertEqual(payload["dashboardCount"], 1)
            self.assertEqual(payload["panelCount"], 1)
            self.assertEqual(payload["summary"]["datasourceCount"], 1)
            self.assertTrue(output_file.exists())
            self.assertEqual(
                json.loads(output_file.read_text(encoding="utf-8")),
                payload,
            )


if __name__ == "__main__":
    unittest.main()
