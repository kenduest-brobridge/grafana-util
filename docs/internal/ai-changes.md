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

## 2026-04-19 - Formalize artifact workspace docs
- Summary: documented artifact workspace defaults across command docs and getting-started. The docs now describe `grafana-util.yaml`, root `--config`, `GRAFANA_UTIL_CONFIG`, config-relative `artifact_root`, timestamp/latest/run-id selection, latest-run pointer behavior, and lane placement for dashboard, datasource, snapshot, and access exports.
- Tests: no runtime behavior change; docs and contract coverage only.
- Test Run: `make man`; `make html`; `make man-check`; `make html-check`; `make quality-docs-surface`; `make quality-ai-workflow`.
- Impact: `docs/user-guide/{en,zh-TW}/getting-started.md`, artifact-related command docs under `docs/commands/{en,zh-TW}/`, `scripts/contracts/command-surface.json`, and AI trace docs.
- Rollback/Risk: low docs/contract clarification. Rollback would remove operator guidance for the already implemented artifact workspace behavior.

## 2026-04-19 - Add artifact workspace run support
- Summary: added run-centric artifact workspace primitives and profile config `artifact_root`, root `--config`, timestamp/latest/run-id routing for key export flows, and local artifact resolution for selected browse/list/summary/review/plan paths.
- Tests: updated focused Rust parser/test literals for changed option shapes; no test execution.
- Test Run: not run per user instruction.
- Impact: `rust/src/common/artifact_workspace.rs`, `rust/src/commands/config/profile/config.rs`, `rust/src/cli/mod.rs`, `rust/src/cli/dispatch.rs`, Rust dashboard/snapshot/datasource/access command modules, selected Rust tests, and AI trace docs.
- Rollback/Risk: medium CLI behavior expansion guarded by explicit artifact flags or `--local`; rollback by removing artifact resolver usage and keeping existing explicit `--input-dir`/`--output-dir` paths.
- Follow-up: add generated docs/contracts and broaden artifact local support for dashboard import/diff and access import/diff if desired.

## 2026-04-18 - Fix Rust 1.95 sync review clippy failure
- Summary: fixed the GitHub Actions `rust-quality` failure by rewriting sync review TUI key handling to use guarded `match` arms instead of nested `if diff_mode` blocks that Rust 1.95 clippy reports as `collapsible_match`.
- Tests: no behavior change; preserved existing sync review TUI key behavior.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet sync --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `make quality-architecture`.
- Impact: Rust sync review TUI internals and AI trace docs. Public CLI behavior, generated docs, README files, JSON contracts, and Python implementation were intentionally left unchanged.
- Rollback/Risk: low behavior-preserving clippy compatibility refactor. Rollback would restore the Rust 1.95 CI failure.
- Follow-up: verify GitHub Actions after pushing because local stable is older than the CI toolchain.

## 2026-04-19 - Clarify contract ownership map
- Summary: clarified the maintainer contract map so runtime golden output contracts, CLI/docs routing contracts, docs-entrypoint navigation, and schema/help manifests each have a distinct source of truth. This keeps `scripts/contracts/output-contracts.json`, `scripts/contracts/command-surface.json`, `scripts/contracts/docs-entrypoints.json`, and `schemas/manifests/` in separate ownership lanes.
- Tests: no runtime behavior change.
- Test Run: `python3 scripts/check_ai_workflow.py` initially reported that maintainer/contract/architecture docs changed without a matching trace update; the trace files were then updated to satisfy the repo’s workflow guard.
- Impact: `docs/internal/contract-doc-map.md`, `docs/internal/ai-status.md`, and `docs/internal/ai-changes.md`. Runtime code, public CLI behavior, generated docs, README files, JSON contracts, and Python implementation were intentionally left unchanged.
- Rollback/Risk: low documentation-only change. Rollback would return the contract map to the less explicit ownership wording.

## 2026-04-19 - Advance status and review-governance cleanup
- Summary: routed the alert live status producer through the shared status reading model while preserving the existing project-status output shape. Cleaned stale backlog entries, documented how runtime golden contracts and schema manifests should overlap, and added an internal inventory for future mutation review-envelope work without changing public JSON contracts.
- Tests: preserved behavior for alert live project-status output and avoided public CLI or schema changes.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet alert_live_project_status`; `cargo test --manifest-path rust/Cargo.toml --quiet project_status`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `make quality-output-contracts`; `make schema-check`; `make quality-architecture`; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-ai-workflow`.
- Impact: Rust alert live status internals, maintainer contract guidance, mutation review planning docs, TODO backlog, and AI trace docs. README files, generated docs, public CLI behavior, and Python implementation were intentionally left unchanged.
- Rollback/Risk: low behavior-preserving status-model refactor plus maintainer docs. Rollback would restore direct alert live project-status construction and remove the new review-envelope planning note.

## 2026-04-18 - Split oversized Rust test surfaces
- Summary: split large Rust regression files into behavior-focused modules while preserving public behavior and existing test names. Sync bundle execution now separates source, domain artifact, and preflight cases; dashboard export/import/topology and browse workflow tests now route through small facades; snapshot tests now separate fixture, export, review, and metadata cases; access org runtime tests now separate routing, diff, import/export, and local-list cases.
- Tests: preserved existing coverage and added no public behavior changes.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet bundle_exec --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_export_import --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet routed_import --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_browse --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet snapshot --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet access_runtime_org --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet access --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `make quality-architecture`; `make quality-ai-workflow`.
- Impact: Rust sync, dashboard, snapshot, and access test module layout, TODO backlog, and AI trace docs. README files, generated user docs, public CLI behavior, JSON contracts, and Python implementation were intentionally left unchanged.
- Rollback/Risk: low behavior-preserving test refactor. Rollback would restore large test hubs and remove the new sibling test modules.
- Follow-up: continue with the remaining medium-sized test hubs only when they mix real behavior families, starting with datasource CLI mutation or payload tests.

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

## 2026-04-18 - Advance review and contract backlog
- Summary: split dashboard browse detail rendering into a dedicated helper module, introduced a shared status producer model for staged datasource and alert adapters, extracted the sync live apply phase loop into a shared client/request helper, and extended output contract validation with collection-aware path matching plus array/enum constraints.
- Tests: moved dashboard browse render tests into a dedicated module, added status model round-trip coverage, added sync live apply phase regressions, and expanded output contract checker tests for wildcard paths, array item types, minimum items, and enum values.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cargo test --manifest-path rust/Cargo.toml --quiet browse_render --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet project_status --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet phase_ --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet sync_live --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-output-contracts`; `PYTHONPATH=python python3 -m unittest -v python.tests.test_python_output_contracts`; `make quality-sync-rust`.
- Impact: Rust dashboard browse rendering modules, staged project-status adapters, sync live apply orchestration modules, output contract checker/registry/tests, internal contract mapping docs, and AI trace docs. README files and generated user docs were intentionally left unchanged.
- Rollback/Risk: low to medium internal refactor. Rollback should restore inline browse detail rendering, inline sync live apply loops, direct staged status construction, and the previous shallow output contract checker while preserving public command behavior.
- Follow-up: split the pre-existing `rust/src/commands/datasource/plan/mod.rs` architecture hard blocker before treating `make quality-architecture` as a clean gate again.

## 2026-04-18 - Advance workspace review aggregation and cleanup
- Summary: added an internal `WorkspaceReviewView` adapter so workspace preview and review TUI filtering share one action/domain/blocker normalization path while preserving the existing `grafana-utils-sync-plan` JSON shape. Split access team browse key dispatch and tests out of the input surface, reducing `team_browse_input.rs` to a small input/confirmation facade. Cleaned public dashboard summary/review wording and helper names without renaming true query analyzer internals.
- Tests: added workspace review-view contract coverage and moved access team browse input tests into a dedicated module. Updated dashboard help/parser tests and live summary help fixtures for the summary/review wording.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet team_browse --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet sync --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_cli_inspect_help --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_cli_parser_help --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `make man`; `make html`; `make quality-docs-surface`.
- Impact: Rust workspace preview/review helpers, access team browse TUI input modules, dashboard summary help/docs/tests, generated command/man/html docs, and AI trace docs. README files and Python implementation were intentionally left unchanged.
- Rollback/Risk: medium internal refactor across review/TUI surfaces. Rollback should restore the previous workspace preview contract helper, inline team browse input dispatch/tests, and previous dashboard wording while preserving already-committed domain plan contracts.
- Follow-up: connect future TUI review surfaces to `WorkspaceReviewView` directly instead of re-parsing raw JSON in each UI layer.
