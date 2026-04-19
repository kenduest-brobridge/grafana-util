import importlib
import io
import json
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_ROOT = REPO_ROOT / "python"
if str(PYTHON_ROOT) not in sys.path:
    sys.path.insert(0, str(PYTHON_ROOT))

snapshot_cli = importlib.import_module("grafana_utils.snapshot_cli")


class SnapshotCliTests(unittest.TestCase):
    def test_snapshot_export_uses_artifact_run_root_when_output_dir_is_omitted(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            config_path = root / "grafana-util.yaml"
            config_path.write_text("artifact_root: artifacts\nprofiles: {}\n", encoding="utf-8")
            scope = root / "artifacts" / "default"
            scope.mkdir(parents=True)
            (scope / "latest-run.json").write_text(
                json.dumps({"runId": "run-3"}),
                encoding="utf-8",
            )
            args = snapshot_cli.parse_args(
                ["export", "--run", "latest", "--output-format", "json"]
            )

            payload = {
                "kind": "snapshot-bundle",
                "summary": {"dashboardCount": 0},
            }
            stdout = io.StringIO()
            with (
                mock.patch.dict("os.environ", {"GRAFANA_UTIL_CONFIG": str(config_path)}),
                mock.patch.object(snapshot_cli, "_snapshot_bundle", return_value=payload),
                redirect_stdout(stdout),
            ):
                result = snapshot_cli.export_command(args)
                output_file = root / "artifacts" / "default" / "runs" / "run-3" / "snapshot.json"
                self.assertTrue(output_file.exists())
                self.assertEqual(
                    json.loads(output_file.read_text(encoding="utf-8"))["kind"],
                    "snapshot-bundle",
                )

        self.assertEqual(result, 0)

    def test_snapshot_export_timestamp_creates_new_artifact_run(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            config_path = root / "grafana-util.yaml"
            config_path.write_text("artifact_root: artifacts\nprofiles: {}\n", encoding="utf-8")
            args = snapshot_cli.parse_args(
                ["export", "--run", "timestamp", "--output-format", "json"]
            )
            payload = {
                "kind": "snapshot-bundle",
                "summary": {"dashboardCount": 0},
            }
            with (
                mock.patch.dict("os.environ", {"GRAFANA_UTIL_CONFIG": str(config_path)}),
                mock.patch.object(snapshot_cli, "_snapshot_bundle", return_value=payload),
                redirect_stdout(io.StringIO()),
            ):
                result = snapshot_cli.export_command(args)

            scope = root / "artifacts" / "default"
            pointer = json.loads((scope / "latest-run.json").read_text(encoding="utf-8"))
            output_file = scope / "runs" / pointer["runId"] / "snapshot.json"
            output_exists = output_file.exists()

        self.assertEqual(result, 0)
        self.assertTrue(output_exists)

    def test_snapshot_review_local_counts_lane_files_when_snapshot_json_is_missing(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            config_path = root / "grafana-util.yaml"
            config_path.write_text("artifact_root: artifacts\nprofiles: {}\n", encoding="utf-8")
            scope = root / "artifacts" / "default"
            scope.mkdir(parents=True)
            run_root = scope / "runs" / "run-9"
            (scope / "latest-run.json").write_text(
                json.dumps({"run_id": "run-9"}),
                encoding="utf-8",
            )
            for relative_path in (
                "dashboards/cpu-main.json",
                "dashboards/memory-main.json",
                "datasources/prom-main.json",
                "access/users/alice.json",
                "access/teams/ops.json",
                "access/teams/platform.json",
                "access/orgs/main.json",
                "access/service-accounts/deploy-bot.json",
            ):
                file_path = run_root / relative_path
                file_path.parent.mkdir(parents=True, exist_ok=True)
                file_path.write_text("{}", encoding="utf-8")

            args = snapshot_cli.parse_args(["review", "--local", "--output-format", "json"])

            stdout = io.StringIO()
            with (
                mock.patch.dict("os.environ", {"GRAFANA_UTIL_CONFIG": str(config_path)}),
                redirect_stdout(stdout),
            ):
                result = snapshot_cli.review_command(args)

        self.assertEqual(result, 0)
        document = json.loads(stdout.getvalue())
        self.assertEqual(document["kind"], "snapshot-root")
        self.assertEqual(document["items"]["dashboards"], 2)
        self.assertEqual(document["items"]["datasources"], 1)
        self.assertEqual(document["items"]["users"], 1)
        self.assertEqual(document["items"]["teams"], 2)
        self.assertEqual(document["items"]["organizations"], 1)
        self.assertEqual(document["items"]["serviceAccounts"], 1)

    def test_snapshot_review_rejects_local_with_explicit_input_dir(self):
        args = snapshot_cli.parse_args(
            ["review", "--local", "--input-dir", "./snapshot", "--output-format", "json"]
        )

        with self.assertRaises(ValueError):
            snapshot_cli.review_command(args)


if __name__ == "__main__":
    unittest.main()
