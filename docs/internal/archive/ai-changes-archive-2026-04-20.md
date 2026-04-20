# ai-changes-archive-2026-04-20

## 2026-04-18 - Advance review and contract backlog
- Summary: split dashboard browse detail rendering into a dedicated helper module, introduced a shared status producer model for staged datasource and alert adapters, extracted the sync live apply phase loop into a shared client/request helper, and extended output contract validation with collection-aware path matching plus array/enum constraints.
- Tests: moved dashboard browse render tests into a dedicated module, added status model round-trip coverage, added sync live apply phase regressions, and expanded output contract checker tests for wildcard paths, array item types, minimum items, and enum values.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cargo test --manifest-path rust/Cargo.toml --quiet browse_render --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet project_status --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet phase_ --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet sync_live --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-output-contracts`; `PYTHONPATH=python python3 -m unittest -v python.tests.test_python_output_contracts`; `make quality-sync-rust`.
- Impact: Rust dashboard browse rendering modules, staged project-status adapters, sync live apply orchestration modules, output contract checker/registry/tests, internal contract mapping docs, and AI trace docs. README files and generated user docs were intentionally left unchanged.
- Rollback/Risk: low to medium internal refactor. Rollback should restore inline browse detail rendering, inline sync live apply loops, direct staged status construction, and the previous shallow output contract checker while preserving public command behavior.
- Follow-up: split the pre-existing `rust/src/commands/datasource/plan/mod.rs` architecture hard blocker before treating `make quality-architecture` as a clean gate again.

## 2026-04-18 - Clear Rust architecture warnings
- Summary: cleared the remaining Rust architecture warning-threshold files by splitting production modules and tests along real responsibility boundaries. Dashboard plan now has input/reconcile/render helpers; dashboard import/apply has backend/prepare/live/render helpers; dashboard export has provisioning/root-bundle helpers; export-layout has apply/render/tests helpers; status live has discovery/domains/multi-org/tests helpers; datasource CLI format parsing moved out of the clap definition file; alert runtime tests split by scenario group.
- Tests: preserved existing behavior coverage with focused dashboard plan/import/export/export-layout, status, datasource CLI, and alert tests. Full Rust tests and clippy also pass.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_plan --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet import_dashboards --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet export_dashboards --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet export_layout --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet status --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet datasource_cli --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet alert --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`; `make quality-architecture`; `make quality-rust` passed outside the sandbox after the sandboxed run failed local mock-server tests with `Operation not permitted`.
- Impact: Rust dashboard plan/export/export-layout/import-apply internals, status live internals, datasource CLI format helpers, alert runtime test layout, focused test stability fixes for temp paths, and AI trace docs. README files, generated user docs, and Python implementation were intentionally left unchanged.
- Rollback/Risk: medium internal refactor across several Rust surfaces. Rollback would restore the previous large files and reintroduce architecture warnings; public CLI behavior and JSON contracts should remain unchanged by this split.
- Follow-up: keep new work inside the responsibility-specific helper modules instead of growing command entrypoints again.

## 2026-04-18 - Fix datasource plan architecture gate
- Summary: split the oversized datasource plan module into explicit model, builder, render, and test modules. Added a shared review/action contract vocabulary and replaced scattered action/status comparisons in datasource, access, dashboard, sync preview, sync apply, and review TUI-adjacent paths.
- Tests: moved existing datasource plan regressions into a dedicated test module and preserved coverage for create/update/same/extra, prune delete candidates, read-only blockers, and stable JSON action IDs. Re-ran focused access/dashboard/sync tests to cover the shared vocabulary replacements.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cargo test --manifest-path rust/Cargo.toml --quiet datasource_plan --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet access_plan --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_plan --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet workspace_preview_review_view --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet sync_live --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`; `make quality-architecture`.
- Impact: Rust datasource plan module layout, shared review contract vocabulary, access/dashboard/datasource plan comparisons, sync preview/apply action filtering, and live apply guards. README files, generated user docs, and Python implementation were intentionally left unchanged.
- Rollback/Risk: low internal refactor. Rollback would restore the single large datasource plan module and reintroduce the architecture hard failure; public datasource plan JSON and CLI behavior should remain unchanged.
- Follow-up: apply the same contract-constant pattern when splitting the remaining warning-threshold plan/export surfaces.

## 2026-04-18 - Split oversized Rust test surfaces
- Summary: split large Rust regression files into behavior-focused modules while preserving public behavior and existing test names. Sync bundle execution now separates source, domain artifact, and preflight cases; dashboard export/import/topology and browse workflow tests now route through small facades; snapshot tests now separate fixture, export, review, and metadata cases; access org runtime tests now separate routing, diff, import/export, and local-list cases.
- Tests: preserved existing coverage and added no public behavior changes.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet bundle_exec --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_export_import --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet routed_import --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_browse --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet snapshot --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet access_runtime_org --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet access --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `make quality-architecture`; `make quality-ai-workflow`.
- Impact: Rust sync, dashboard, snapshot, and access test module layout, TODO backlog, and AI trace docs. README files, generated user docs, public CLI behavior, JSON contracts, and Python implementation were intentionally left unchanged.
- Rollback/Risk: low behavior-preserving test refactor. Rollback would restore large test hubs and remove the new sibling test modules.
- Follow-up: continue with the remaining medium-sized test hubs only when they mix real behavior families, starting with datasource CLI mutation or payload tests.

## 2026-04-19 - Advance status and review-governance cleanup
- Summary: routed the alert live status producer through the shared status reading model while preserving the existing project-status output shape. Cleaned stale backlog entries, documented how runtime golden contracts and schema manifests should overlap, and added an internal inventory for future mutation review-envelope work without changing public JSON contracts.
- Tests: preserved behavior for alert live project-status output and avoided public CLI or schema changes.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet alert_live_project_status`; `cargo test --manifest-path rust/Cargo.toml --quiet project_status`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `make quality-output-contracts`; `make schema-check`; `make quality-architecture`; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-ai-workflow`.
- Impact: Rust alert live status internals, maintainer contract guidance, mutation review planning docs, TODO backlog, and AI trace docs. README files, generated docs, public CLI behavior, and Python implementation were intentionally left unchanged.
- Rollback/Risk: low behavior-preserving status-model refactor plus maintainer docs. Rollback would restore direct alert live project-status construction and remove the new review-envelope planning note.

## 2026-04-19 - Clarify contract ownership map
- Summary: clarified the maintainer contract map so runtime golden output contracts, CLI/docs routing contracts, docs-entrypoint navigation, and schema/help manifests each have a distinct source of truth. This keeps `scripts/contracts/output-contracts.json`, `scripts/contracts/command-surface.json`, `scripts/contracts/docs-entrypoints.json`, and `schemas/manifests/` in separate ownership lanes.
- Tests: no runtime behavior change.
- Test Run: `python3 scripts/check_ai_workflow.py` initially reported that maintainer/contract/architecture docs changed without a matching trace update; the trace files were then updated to satisfy the repo’s workflow guard.
- Impact: `docs/internal/contract-doc-map.md`, `docs/internal/ai-status.md`, and `docs/internal/ai-changes.md`. Runtime code, public CLI behavior, generated docs, README files, JSON contracts, and Python implementation were intentionally left unchanged.
- Rollback/Risk: low documentation-only change. Rollback would return the contract map to the less explicit ownership wording.

## 2026-04-18 - Fix Rust 1.95 sync review clippy failure
- Summary: fixed the GitHub Actions `rust-quality` failure by rewriting sync review TUI key handling to use guarded `match` arms instead of nested `if diff_mode` blocks that Rust 1.95 clippy reports as `collapsible_match`.
- Tests: no behavior change; preserved existing sync review TUI key behavior.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet sync --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `make quality-architecture`.
- Impact: Rust sync review TUI internals and AI trace docs. Public CLI behavior, generated docs, README files, JSON contracts, and Python implementation were intentionally left unchanged.
- Rollback/Risk: low behavior-preserving clippy compatibility refactor. Rollback would restore the Rust 1.95 CI failure.
- Follow-up: verify GitHub Actions after pushing because local stable is older than the CI toolchain.

## 2026-04-19 - Add artifact workspace run support
- Summary: added run-centric artifact workspace primitives and profile config `artifact_root`, root `--config`, timestamp/latest/run-id routing for key export flows, and local artifact resolution for selected browse/list/summary/review/plan paths.
- Tests: updated focused Rust parser/test literals for changed option shapes; no test execution.
- Test Run: not run per user instruction.
- Impact: `rust/src/common/artifact_workspace.rs`, `rust/src/commands/config/profile/config.rs`, `rust/src/cli/mod.rs`, `rust/src/cli/dispatch.rs`, Rust dashboard/snapshot/datasource/access command modules, selected Rust tests, and AI trace docs.
- Rollback/Risk: medium CLI behavior expansion guarded by explicit artifact flags or `--local`; rollback by removing artifact resolver usage and keeping existing explicit `--input-dir`/`--output-dir` paths.
- Follow-up: add generated docs/contracts and broaden artifact local support for dashboard import/diff and access import/diff if desired.

## 2026-04-19 - Formalize artifact workspace docs
- Summary: documented artifact workspace defaults across command docs and getting-started. The docs now describe `grafana-util.yaml`, root `--config`, `GRAFANA_UTIL_CONFIG`, config-relative `artifact_root`, timestamp/latest/run-id selection, latest-run pointer behavior, and lane placement for dashboard, datasource, snapshot, and access exports.
- Tests: no runtime behavior change; docs and contract coverage only.
- Test Run: `make man`; `make html`; `make man-check`; `make html-check`; `make quality-docs-surface`; `make quality-ai-workflow`.
- Impact: `docs/user-guide/{en,zh-TW}/getting-started.md`, artifact-related command docs under `docs/commands/{en,zh-TW}/`, `scripts/contracts/command-surface.json`, and AI trace docs.
- Rollback/Risk: low docs/contract clarification. Rollback would remove operator guidance for the already implemented artifact workspace behavior.

## 2026-04-19 - Broaden artifact workspace local consumers
- Summary: expanded artifact workspace local input routing so dashboard import/diff and access user/team/org/service-account import/diff can consume profile artifact runs through `--local`, `--run`, or `--run-id` without repeating input directories.
- Tests: parser/runtime behavior changed; Rust tests not run.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all`; `make man`; `make html`; `make man-check`; `make html-check`; `make quality-docs-surface`; `make quality-ai-workflow`.
- Impact: Rust dashboard/access CLI argument surfaces, dashboard/access dispatch path materialization, command docs, command-surface contract, and AI trace docs. Python implementation and README files are intentionally out of scope.
- Rollback/Risk: medium CLI behavior expansion around required local input paths. Rollback would remove the new local input flags and restore explicit `--input-dir`/`--diff-dir` only behavior.

## 2026-04-20 - Complete Python Rust parity surfaces
- Summary: added Python parity for Rust-public dashboard review surfaces (`dashboard plan`, `dashboard history diff`), made topology interactive mode usable through a deterministic text browser, converted status live read failures into blocked domains with all-org scoped aggregation, wired access user/team browse to live and local export-bundle flows, added profile artifact lane resolution for access browse `--local/--run/--run-id`, and added dashboard plan `--use-export-org` routed review output.
- Tests: added focused parser/runtime tests for dashboard history diff, dashboard plan, dashboard plan routed org review, topology interactive rendering, status read-failure/merge behavior, and access browse local artifact lanes.
- Test Run: `PYTHONPATH=python python -m py_compile python/grafana_utils/dashboard_authoring.py python/grafana_utils/dashboard_cli.py python/grafana_utils/dashboard_topology.py python/grafana_utils/project_status_live.py python/grafana_utils/access/parser.py python/grafana_utils/access/workflows.py python/grafana_utils/clients/access_client.py python/grafana_utils/clients/alert_client.py python/grafana_utils/profile_config.py`; `PYTHONPATH=python python -m unittest -v python/tests/test_python_dashboard_topology.py`; `PYTHONPATH=python python -m unittest -v python/tests/test_python_project_status.py`; `PYTHONPATH=python python -m unittest -v python/tests/test_python_access_cli.py`; `PYTHONPATH=python python -m unittest -v python/tests/test_python_dashboard_cli.py`; `PYTHONPATH=python python -m unittest discover -v -s python/tests -p 'test_*.py'` passed 1181 tests; `make quality-docs-surface`; `make quality-ai-workflow`.
- Impact: `python/grafana_utils/dashboard_authoring.py`, `python/grafana_utils/dashboard_cli.py`, `python/grafana_utils/dashboard_topology.py`, `python/grafana_utils/project_status_live.py`, `python/grafana_utils/access/parser.py`, `python/grafana_utils/access/workflows.py`, `python/grafana_utils/clients/access_client.py`, `python/grafana_utils/clients/alert_client.py`, `python/grafana_utils/profile_config.py`, focused Python tests, and AI trace docs.
- Rollback/Risk: medium Python CLI behavior expansion. Dashboard plan is intentionally review-only and does not mutate Grafana; access browse local artifact mode reads existing export bundles from profile artifact lanes; status live now surfaces failures that were previously hidden.
- Follow-up: none for this parity slice.

## 2026-04-20 - Complete Python artifact and plan parity
- Summary: aligned Python artifact workspace run selectors with Rust `latest`/`timestamp` semantics, added datasource `plan` plus local artifact input support for datasource list/import/diff/plan, added access plan/local workflow coverage, and expanded snapshot export/review artifact workspace handling.
- Tests: added focused parser/runtime tests for datasource plan/local artifact lanes, access local plan flows, and snapshot artifact export/review roots.
- Test Run: `cd python && PYTHONPATH=. python -m unittest -v tests.test_python_datasource_cli tests.test_python_snapshot_cli tests.test_python_profile_config`; `cd python && PYTHONPATH=. python -m unittest -v tests.test_python_access_cli`; `cd python && PYTHONPATH=. python -m unittest -v tests.test_python_unified_cli`.
- Impact: `python/grafana_utils/profile_config.py`, `python/grafana_utils/datasource/parser.py`, `python/grafana_utils/datasource/workflows.py`, `python/grafana_utils/datasource_cli.py`, `python/grafana_utils/access/parser.py`, `python/grafana_utils/access/workflows.py`, `python/grafana_utils/snapshot_cli.py`, focused Python tests, unified CLI preview, and AI trace docs.
- Rollback/Risk: medium Python CLI behavior expansion. Rollback would remove the new local artifact consumers and review-only plan surfaces; live mutation flows remain gated by existing import/add/modify/delete commands.
- Follow-up: continue with dashboard/status/resource parity depth after this focused artifact and plan slice.

## 2026-04-20 - Split Rust status producer tests
- Summary: moved dashboard project-status and datasource live project-status inline Rust tests into adjacent test modules so the production producers stay focused and below the oversized-file threshold.
- Tests: preserved private-module coverage through `#[path]` test modules and kept the existing status behavior assertions unchanged.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cargo test --manifest-path rust/Cargo.toml --quiet project_status`; `python3 scripts/rust_maintainability_report.py`.
- Impact: `rust/src/commands/dashboard/project_status.rs`, `rust/src/commands/dashboard/project_status_tests.rs`, `rust/src/commands/datasource/project_status/live.rs`, and `rust/src/commands/datasource/project_status/live_tests.rs`.
- Rollback/Risk: low mechanical split. Rollback would inline the tests again; behavior should not change because production code was not altered.
