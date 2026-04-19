# ai-changes.md

Current AI change log only.

- Older detailed history moved to [`archive/ai-changes-archive-2026-03-24.md`](docs/internal/archive/ai-changes-archive-2026-03-24.md).
- Detailed 2026-03-27 entries moved to [`archive/ai-changes-archive-2026-03-27.md`](docs/internal/archive/ai-changes-archive-2026-03-27.md).
- Detailed 2026-03-28 task notes were condensed into [`archive/ai-changes-archive-2026-03-28.md`](docs/internal/archive/ai-changes-archive-2026-03-28.md).
- Detailed 2026-03-29 through 2026-03-31 entries moved to [`archive/ai-changes-archive-2026-03-31.md`](docs/internal/archive/ai-changes-archive-2026-03-31.md).
- Detailed 2026-04-01 through 2026-04-12 entries moved to [`archive/ai-changes-archive-2026-04-12.md`](docs/internal/archive/ai-changes-archive-2026-04-12.md).
- Keep this file limited to the latest active architecture and maintenance changes.
- Older entries moved to [`ai-changes-archive-2026-04-13.md`](docs/internal/archive/ai-changes-archive-2026-04-13.md).
- Older entries moved to [`ai-changes-archive-2026-04-14.md`](docs/internal/archive/ai-changes-archive-2026-04-14.md).
- Older entries moved to [`ai-changes-archive-2026-04-15.md`](docs/internal/archive/ai-changes-archive-2026-04-15.md).
- Older entries moved to [`ai-changes-archive-2026-04-16.md`](docs/internal/archive/ai-changes-archive-2026-04-16.md).
- Older entries moved to [`ai-changes-archive-2026-04-17.md`](docs/internal/archive/ai-changes-archive-2026-04-17.md).
- Older entries moved to [`ai-changes-archive-2026-04-18.md`](docs/internal/archive/ai-changes-archive-2026-04-18.md).
- Older entries moved to [`ai-changes-archive-2026-04-19.md`](docs/internal/archive/ai-changes-archive-2026-04-19.md).
- Older entries moved to [`ai-changes-archive-2026-04-20.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-04-20.md).

## 2026-04-20 - Clarify sync project-status boundary
- Summary: extracted shared sync project-status JSON helpers, reused them across staged sync and promotion domain-status producers, and moved sync domain-status tests into a dedicated Rust test module.
- Tests: preserved existing sync and promotion project-status behavior while reducing production/test mixing in `sync/project_status.rs`.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet sync_project_status --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet project_status_promotion --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet status --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet sync --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `python3 scripts/rust_maintainability_report.py`; `cargo test --manifest-path rust/Cargo.toml --quiet`.
- Impact: `rust/src/commands/sync/project_status.rs`, `rust/src/commands/sync/project_status_json.rs`, `rust/src/commands/sync/project_status_tests.rs`, `rust/src/commands/sync/project_status_promotion.rs`, `rust/src/commands/sync/mod.rs`, and AI trace docs. Public CLI behavior, generated docs, Python implementation, and output contracts are intentionally unchanged.
- Rollback/Risk: low mechanical refactor inside sync-owned status producers. Rollback would restore local JSON helpers and inline sync tests; behavior should remain unchanged because focused and full Rust tests cover the moved paths.

## 2026-04-20 - Split Rust architecture hotspots
- Summary: removed obsolete sync compatibility aliases, routed root preflight through canonical sync modules, and split large Rust command hotspots for resource, dashboard import validation, and access org workflows into focused facade-backed modules.
- Tests: preserved public Rust command surfaces and behavior while narrowing module ownership for resource CLI/catalog/runtime/rendering, dashboard import validation auth/org/dependency validation, and access org live/sync/diff workflows.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `python3 scripts/rust_maintainability_report.py`; `cargo test --manifest-path rust/Cargo.toml --quiet`.
- Impact: `rust/src/lib.rs`, `rust/src/commands/sync/root_preflight/mod.rs`, `rust/src/commands/resource/`, `rust/src/commands/dashboard/import_validation*.rs`, `rust/src/commands/access/org_workflows*.rs`, and AI trace docs. Public CLI paths, generated docs, Python implementation, and output contracts are intentionally unchanged.
- Rollback/Risk: medium mechanical architecture cleanup across several Rust command boundaries. Rollback would re-inline the facade modules and restore the removed compatibility aliases; focused and full Rust tests cover compile-time visibility and moved workflow paths.

## 2026-04-20 - Continue Rust architecture cleanup
- Summary: split dashboard artifact workflow coverage into a dedicated Rust test module, moved dashboard facade re-exports into `facade_exports.rs`, and added snapshot artifact-workspace timestamp/latest-run coverage.
- Tests: preserved behavior while narrowing test ownership and keeping the dashboard module root focused on module registration, constants, wrappers, and type definitions.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_artifact_workflow --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_cli_parser_help_workflow --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet snapshot_export_run_timestamp_uses_artifact_snapshot_root_and_records_latest_run --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet snapshot --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `python3 scripts/rust_maintainability_report.py`; `make quality-ai-workflow`.
- Impact: `rust/src/commands/dashboard/mod.rs`, `rust/src/commands/dashboard/facade_exports.rs`, dashboard artifact/parser workflow tests, `rust/src/commands/snapshot/support.rs`, snapshot export tests, and AI trace docs. Public CLI paths and output contracts are intentionally unchanged.
- Rollback/Risk: low mechanical architecture cleanup plus test coverage. Rollback would move facade re-exports and artifact tests back into their previous files and remove the snapshot artifact test.

## 2026-04-20 - Split dashboard artifact command routing
- Summary: moved dashboard artifact workspace run resolution and local input materialization out of the command runner into a focused Rust helper module while preserving existing dashboard CLI behavior.
- Tests: verified the refactor with focused dashboard parser, artifact, and dashboard command scopes plus formatter and maintainability checks.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_cli_parser_help_workflow --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_cli_parser_help_list_export --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet run_dashboard_cli --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet artifact --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `python3 scripts/rust_maintainability_report.py`; `make quality-ai-workflow`.
- Impact: `rust/src/commands/dashboard/command_runner.rs`, `rust/src/commands/dashboard/command_artifacts.rs`, dashboard module registration, and AI trace docs. Public CLI paths, flags, and output contracts are intentionally unchanged.
- Rollback/Risk: low mechanical extraction. Rollback would re-inline the artifact helpers into the command runner; risk is import/visibility drift, covered by focused Rust tests.

## 2026-04-20 - Split Rust command orchestration modules
- Summary: split the remaining oversized Rust production files for access dispatch, dashboard export, and dashboard prompt transformation into focused helper modules while preserving the existing public entrypoints.
- Tests: kept the refactor behavior-preserving with focused access, export, and prompt test filters plus formatter and maintainability checks.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cargo test --manifest-path rust/Cargo.toml --quiet prompt --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet export_ --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet run_access_cli_with_request_routes_user_export --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet access_`; `python3 scripts/rust_maintainability_report.py`.
- Impact: Rust access dispatch modules, dashboard export modules, dashboard prompt modules, and AI trace docs. No public CLI behavior or JSON contract change is intended.
- Rollback/Risk: medium mechanical refactor across command orchestration boundaries. Rollback would re-inline the helper modules; behavior should remain unchanged because the focused command tests still cover the moved paths.

## 2026-04-20 - Split Rust status producer tests
- Summary: moved dashboard project-status and datasource live project-status inline Rust tests into adjacent test modules so the production producers stay focused and below the oversized-file threshold.
- Tests: preserved private-module coverage through `#[path]` test modules and kept the existing status behavior assertions unchanged.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cargo test --manifest-path rust/Cargo.toml --quiet project_status`; `python3 scripts/rust_maintainability_report.py`.
- Impact: `rust/src/commands/dashboard/project_status.rs`, `rust/src/commands/dashboard/project_status_tests.rs`, `rust/src/commands/datasource/project_status/live.rs`, and `rust/src/commands/datasource/project_status/live_tests.rs`.
- Rollback/Risk: low mechanical split. Rollback would inline the tests again; behavior should not change because production code was not altered.

## 2026-04-20 - Complete Python artifact and plan parity
- Summary: aligned Python artifact workspace run selectors with Rust `latest`/`timestamp` semantics, added datasource `plan` plus local artifact input support for datasource list/import/diff/plan, added access plan/local workflow coverage, and expanded snapshot export/review artifact workspace handling.
- Tests: added focused parser/runtime tests for datasource plan/local artifact lanes, access local plan flows, and snapshot artifact export/review roots.
- Test Run: `cd python && PYTHONPATH=. python -m unittest -v tests.test_python_datasource_cli tests.test_python_snapshot_cli tests.test_python_profile_config`; `cd python && PYTHONPATH=. python -m unittest -v tests.test_python_access_cli`; `cd python && PYTHONPATH=. python -m unittest -v tests.test_python_unified_cli`.
- Impact: `python/grafana_utils/profile_config.py`, `python/grafana_utils/datasource/parser.py`, `python/grafana_utils/datasource/workflows.py`, `python/grafana_utils/datasource_cli.py`, `python/grafana_utils/access/parser.py`, `python/grafana_utils/access/workflows.py`, `python/grafana_utils/snapshot_cli.py`, focused Python tests, unified CLI preview, and AI trace docs.
- Rollback/Risk: medium Python CLI behavior expansion. Rollback would remove the new local artifact consumers and review-only plan surfaces; live mutation flows remain gated by existing import/add/modify/delete commands.
- Follow-up: continue with dashboard/status/resource parity depth after this focused artifact and plan slice.

## 2026-04-20 - Complete Python Rust parity surfaces
- Summary: added Python parity for Rust-public dashboard review surfaces (`dashboard plan`, `dashboard history diff`), made topology interactive mode usable through a deterministic text browser, converted status live read failures into blocked domains with all-org scoped aggregation, wired access user/team browse to live and local export-bundle flows, added profile artifact lane resolution for access browse `--local/--run/--run-id`, and added dashboard plan `--use-export-org` routed review output.
- Tests: added focused parser/runtime tests for dashboard history diff, dashboard plan, dashboard plan routed org review, topology interactive rendering, status read-failure/merge behavior, and access browse local artifact lanes.
- Test Run: `PYTHONPATH=python python -m py_compile python/grafana_utils/dashboard_authoring.py python/grafana_utils/dashboard_cli.py python/grafana_utils/dashboard_topology.py python/grafana_utils/project_status_live.py python/grafana_utils/access/parser.py python/grafana_utils/access/workflows.py python/grafana_utils/clients/access_client.py python/grafana_utils/clients/alert_client.py python/grafana_utils/profile_config.py`; `PYTHONPATH=python python -m unittest -v python/tests/test_python_dashboard_topology.py`; `PYTHONPATH=python python -m unittest -v python/tests/test_python_project_status.py`; `PYTHONPATH=python python -m unittest -v python/tests/test_python_access_cli.py`; `PYTHONPATH=python python -m unittest -v python/tests/test_python_dashboard_cli.py`; `PYTHONPATH=python python -m unittest discover -v -s python/tests -p 'test_*.py'` passed 1181 tests; `make quality-docs-surface`; `make quality-ai-workflow`.
- Impact: `python/grafana_utils/dashboard_authoring.py`, `python/grafana_utils/dashboard_cli.py`, `python/grafana_utils/dashboard_topology.py`, `python/grafana_utils/project_status_live.py`, `python/grafana_utils/access/parser.py`, `python/grafana_utils/access/workflows.py`, `python/grafana_utils/clients/access_client.py`, `python/grafana_utils/clients/alert_client.py`, `python/grafana_utils/profile_config.py`, focused Python tests, and AI trace docs.
- Rollback/Risk: medium Python CLI behavior expansion. Dashboard plan is intentionally review-only and does not mutate Grafana; access browse local artifact mode reads existing export bundles from profile artifact lanes; status live now surfaces failures that were previously hidden.
- Follow-up: none for this parity slice.

## 2026-04-19 - Broaden artifact workspace local consumers
- Summary: expanded artifact workspace local input routing so dashboard import/diff and access user/team/org/service-account import/diff can consume profile artifact runs through `--local`, `--run`, or `--run-id` without repeating input directories.
- Tests: parser/runtime behavior changed; Rust tests not run.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all`; `make man`; `make html`; `make man-check`; `make html-check`; `make quality-docs-surface`; `make quality-ai-workflow`.
- Impact: Rust dashboard/access CLI argument surfaces, dashboard/access dispatch path materialization, command docs, command-surface contract, and AI trace docs. Python implementation and README files are intentionally out of scope.
- Rollback/Risk: medium CLI behavior expansion around required local input paths. Rollback would remove the new local input flags and restore explicit `--input-dir`/`--diff-dir` only behavior.

## 2026-04-19 - Formalize artifact workspace docs
- Summary: documented artifact workspace defaults across command docs and getting-started. The docs now describe `grafana-util.yaml`, root `--config`, `GRAFANA_UTIL_CONFIG`, config-relative `artifact_root`, timestamp/latest/run-id selection, latest-run pointer behavior, and lane placement for dashboard, datasource, snapshot, and access exports.
- Tests: no runtime behavior change; docs and contract coverage only.
- Test Run: `make man`; `make html`; `make man-check`; `make html-check`; `make quality-docs-surface`; `make quality-ai-workflow`.
- Impact: `docs/user-guide/{en,zh-TW}/getting-started.md`, artifact-related command docs under `docs/commands/{en,zh-TW}/`, `scripts/contracts/command-surface.json`, and AI trace docs.
- Rollback/Risk: low docs/contract clarification. Rollback would remove operator guidance for the already implemented artifact workspace behavior.
