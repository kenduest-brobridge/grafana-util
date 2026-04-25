from __future__ import annotations

import io
import json
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
import sys
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import scripts.contract_promotion_report as contract_promotion_report
from scripts.contract_promotion_report import (
    ManifestContract,
    ManifestRoute,
    RuntimeContract,
    build_evidence_matrix,
    build_promotion_report,
    load_manifest_contracts,
    load_runtime_contracts,
    render_text_report,
)


class ContractPromotionReportTest(unittest.TestCase):
    def test_groups_manifest_and_runtime_families(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            manifests_dir = root / "schemas" / "manifests"
            change_dir = manifests_dir / "change"
            status_dir = manifests_dir / "status"
            change_dir.mkdir(parents=True)
            status_dir.mkdir(parents=True)

            change_dir.joinpath("contracts.json").write_text(
                json.dumps(
                    [
                        {
                            "contractId": "sync-plan",
                            "title": "Sync plan",
                            "goldenSample": "tests/fixtures/machine_schema_golden_cases.json#sync-plan",
                        },
                        {
                            "contractId": "sync-preflight",
                            "title": "Sync preflight",
                            "goldenSample": "tests/fixtures/machine_schema_golden_cases.json#sync-preflight",
                        },
                    ]
                ),
                encoding="utf-8",
            )
            status_dir.joinpath("contracts.json").write_text(
                json.dumps(
                    [
                        {
                            "contractId": "project-status",
                            "title": "Project status",
                            "goldenSample": "tests/fixtures/machine_schema_golden_cases.json#project-status",
                        }
                    ]
                ),
                encoding="utf-8",
            )

            registry_path = root / "scripts" / "contracts"
            registry_path.mkdir(parents=True)
            registry_path.joinpath("output-contracts.json").write_text(
                json.dumps(
                    {
                        "schema": 1,
                        "contracts": [
                            {
                                "name": "sync-plan",
                                "kind": "grafana-utils-sync-plan",
                                "schemaVersion": 1,
                                "fixture": "output-fixtures/sync-plan.json",
                                "requiredFields": ["kind"],
                            },
                            {
                                "name": "dashboard-summary-governance",
                                "kind": "grafana-utils-dashboard-summary-governance",
                                "schemaVersion": 1,
                                "fixture": "output-fixtures/dashboard-summary-governance.json",
                                "requiredFields": ["kind"],
                            },
                        ],
                    }
                ),
                encoding="utf-8",
            )

            manifest_contracts = load_manifest_contracts(manifests_dir)
            runtime_contracts = load_runtime_contracts(registry_path / "output-contracts.json")
            report = build_promotion_report(manifest_contracts, runtime_contracts)
            text = render_text_report(report, manifest_contracts, runtime_contracts)

            self.assertIn("manifest families: 2", text)
            self.assertIn("runtime families: 2", text)
            self.assertIn("change: total=2 shared=1 only=1 runtime=sync", text)
            self.assertIn("status: total=1 shared=0 only=1 runtime=-", text)
            self.assertIn("sync: total=1 shared=1 only=0 manifest=change", text)
            self.assertIn("dashboard: total=1 shared=0 only=1 manifest=-", text)
            self.assertEqual(report.shared_contract_ids, ("sync-plan",))
            self.assertEqual(report.manifest_only_contract_ids, ("project-status", "sync-preflight"))
            self.assertEqual(report.runtime_only_contract_ids, ("dashboard-summary-governance",))

    def test_evidence_matrix_reports_routes_docs_artifacts_and_findings(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            command_surface_path = root / "scripts" / "contracts" / "command-surface.json"
            docs_entrypoints_path = root / "scripts" / "contracts" / "docs-entrypoints.json"
            generated_docs_dir = root / "docs" / "commands"
            command_surface_path.parent.mkdir(parents=True)
            (generated_docs_dir / "en").mkdir(parents=True)
            (generated_docs_dir / "zh-TW").mkdir(parents=True)
            (generated_docs_dir / "en" / "dashboard-export.md").write_text(
                "# dashboard export\n",
                encoding="utf-8",
            )
            command_surface_path.write_text(
                json.dumps(
                    {
                        "doc_pages": {
                            "alert": "docs/commands/en/alert.md",
                            "dashboard": "docs/commands/en/dashboard.md",
                        },
                        "artifact_workspace": {
                            "export_commands": ["dashboard export"],
                            "lanes": {"alert": "alert lane"},
                        },
                    }
                ),
                encoding="utf-8",
            )
            docs_entrypoints_path.write_text(
                json.dumps({"quick": {"command": "grafana-util dashboard export"}}),
                encoding="utf-8",
            )

            manifest_contracts = [
                ManifestContract(
                    family="beta",
                    contract_id="schema-gap",
                    golden_sample="tests/fixtures/schema-gap.json",
                    title="Schema gap",
                    kind=None,
                    schema_version=None,
                    aliases=(),
                ),
                ManifestContract(
                    family="alpha",
                    contract_id="runtime-backed",
                    golden_sample="tests/fixtures/runtime-backed.json",
                    title="Runtime backed",
                    kind="grafana-utils-runtime-backed",
                    schema_version="1",
                    aliases=(),
                ),
                ManifestContract(
                    family="alpha",
                    contract_id="public-missing-doc",
                    golden_sample="tests/fixtures/public-missing-doc.json",
                    title="Public route missing generated doc",
                    kind=None,
                    schema_version=None,
                    aliases=(),
                ),
            ]
            runtime_contracts = [
                RuntimeContract(
                    family="dashboard",
                    contract_id="runtime-backed-output",
                    fixture="output-fixtures/runtime-backed.json",
                    kind="grafana-utils-runtime-backed",
                    schema_version="1",
                    aliases=(),
                )
            ]
            routes = [
                ManifestRoute(
                    family="alpha",
                    route_id="alert-sync",
                    commands=("grafana-util alert sync --help-schema",),
                    contract_ids=("public-missing-doc",),
                ),
                ManifestRoute(
                    family="alpha",
                    route_id="dashboard-export",
                    commands=("grafana-util dashboard export --help-schema",),
                    contract_ids=("runtime-backed",),
                ),
            ]

            with patch.object(contract_promotion_report, "REPO_ROOT", root), patch.object(
                contract_promotion_report,
                "GENERATED_DOCS_DIR",
                generated_docs_dir,
            ):
                rows, issues = build_evidence_matrix(
                    manifest_contracts,
                    runtime_contracts,
                    routes,
                    command_surface_path=command_surface_path,
                    docs_entrypoints_path=docs_entrypoints_path,
                )

            self.assertEqual(
                [row.contract_id for row in rows],
                ["public-missing-doc", "runtime-backed", "schema-gap"],
            )

            by_contract = {row.contract_id: row for row in rows}
            self.assertEqual(by_contract["runtime-backed"].match_method, "kind")
            self.assertEqual(
                by_contract["runtime-backed"].runtime_golden,
                "output-fixtures/runtime-backed.json",
            )
            self.assertEqual(
                by_contract["runtime-backed"].public_cli_route,
                "dashboard export",
            )
            self.assertEqual(by_contract["runtime-backed"].docs_entrypoint, "dashboard export")
            self.assertEqual(
                by_contract["runtime-backed"].generated_docs,
                "docs/commands/en/dashboard-export.md",
            )
            self.assertEqual(by_contract["runtime-backed"].artifact_workspace_lane, "dashboard export")

            self.assertEqual(by_contract["public-missing-doc"].runtime_golden, "-")
            self.assertEqual(
                by_contract["public-missing-doc"].public_cli_route,
                "alert sync",
            )
            self.assertEqual(by_contract["public-missing-doc"].generated_docs, "-")
            self.assertEqual(by_contract["public-missing-doc"].artifact_workspace_lane, "alert sync")

            self.assertEqual(by_contract["schema-gap"].match_method, "unmatched")
            self.assertEqual(by_contract["schema-gap"].public_cli_route, "-")
            self.assertEqual(by_contract["schema-gap"].schema_help_manifest, "beta/contracts.json")

            self.assertEqual(
                [(issue.category, issue.contract_id) for issue in issues],
                [
                    ("missing-runtime-golden", "public-missing-doc"),
                    ("missing-generated-doc", "public-missing-doc"),
                    ("missing-runtime-golden", "schema-gap"),
                    ("missing-schema-route", "schema-gap"),
                ],
            )
            issue_messages = "\n".join(issue.message for issue in issues)
            self.assertIn("no matching runtime golden", issue_messages)
            self.assertIn("no generated command-doc", issue_messages)
            self.assertIn("not referenced by a help route", issue_messages)

    def test_main_is_informational_by_default_with_findings(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            manifests_dir = root / "schemas" / "manifests"
            registry_path = root / "scripts" / "contracts" / "output-contracts.json"
            manifest_family_dir = manifests_dir / "change"
            manifest_family_dir.mkdir(parents=True)
            registry_path.parent.mkdir(parents=True)

            manifest_family_dir.joinpath("contracts.json").write_text(
                json.dumps(
                    [
                        {
                            "contractId": "schema-only",
                            "title": "Schema only",
                            "goldenSample": "tests/fixtures/schema-only.json",
                        }
                    ]
                ),
                encoding="utf-8",
            )
            registry_path.write_text(
                json.dumps(
                    {
                        "schema": 1,
                        "contracts": [
                            {
                                "name": "runtime-only",
                                "fixture": "output-fixtures/runtime-only.json",
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )

            stdout = io.StringIO()
            with redirect_stdout(stdout):
                exit_code = contract_promotion_report.main(
                    [
                        "--manifests-dir",
                        str(manifests_dir),
                        "--registry",
                        str(registry_path),
                    ]
                )
            text = stdout.getvalue()

            self.assertEqual(exit_code, 0)
            self.assertIn("runtime-only contract ids: 1", text)
            self.assertIn("schema-only [unmatched]", text)
            self.assertIn("runtime-only [runtime-only]", text)
            self.assertIn("informational only; findings do not change the exit status.", text)
            self.assertIn("missing-runtime-golden: schema-only", text)
            self.assertIn("missing-schema-route: schema-only", text)
            self.assertIn("missing-schema-manifest: runtime-only", text)


if __name__ == "__main__":
    unittest.main()
