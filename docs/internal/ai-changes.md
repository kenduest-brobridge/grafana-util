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

## 2026-04-20 - Split project status live API tests
- Summary: moved Grafana project-status live API tests into a dedicated adjacent Rust test module, leaving production read/freshness helpers in `project_status_live.rs`.
- Tests: preserved org, dashboard, datasource, version-history, alert-surface, and freshness coverage while reducing production/test mixing.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet grafana_api::project_status_live::tests --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet project_status_live_org_id_scopes_live_reads --lib -- --test-threads=1`; `cargo test --manifest-path rust/Cargo.toml --quiet project_status_live_all_orgs_fans_out_across_visible_orgs --lib -- --test-threads=1`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cargo test --manifest-path rust/Cargo.toml --quiet sync --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet status --lib`; `python3 scripts/rust_maintainability_report.py`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-ai-workflow`; `git diff --check`.
- Impact: `rust/src/grafana/api/project_status_live.rs`, `rust/src/grafana/api/project_status_live_tests.rs`, and AI trace docs. Public CLI behavior, generated docs, Python implementation, and output contracts are intentionally unchanged.
- Rollback/Risk: low mechanical test split. Rollback would inline the tests again; production behavior is unchanged.

## 2026-04-20 - Align sync live availability keys
- Summary: moved sync live availability map keys into a shared namespaced module and reused them from both the live read path and availability merge allow-list.
- Tests: preserved Grafana API live availability behavior and sync/status status aggregation behavior while removing repeated raw availability key strings.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet availability --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cargo test --manifest-path rust/Cargo.toml --quiet sync --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet status --lib`; `python3 scripts/rust_maintainability_report.py`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-ai-workflow`; `git diff --check`.
- Impact: `rust/src/grafana/api/sync_live.rs`, `rust/src/grafana/api/sync_live_read.rs`, and AI trace docs. Public CLI behavior, generated docs, Python implementation, and output contracts are intentionally unchanged.
- Rollback/Risk: low mechanical refactor around availability document keys. Rollback would restore raw strings in read and merge paths; behavior should remain unchanged because the same key literals are centralized.

## 2026-04-20 - Align live sync status helpers
- Summary: reused the shared sync project-status JSON summary helper in the live sync status producer and grouped live sync summary keys and signal source strings under namespaced constants.
- Tests: moved live sync status assertions into a dedicated Rust test module and preserved none, blocker, partial, additive source, no-resource, and generic bundle fallback behavior.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet live_sync_domain_status --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cargo test --manifest-path rust/Cargo.toml --quiet sync --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet status --lib`; `python3 scripts/rust_maintainability_report.py`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-ai-workflow`; `git diff --check`.
- Impact: `rust/src/commands/sync/live_project_status_sync.rs`, `rust/src/commands/sync/live_project_status_sync_tests.rs`, `rust/src/commands/sync/mod.rs`, and AI trace docs. Public CLI behavior, generated docs, Python implementation, and output contracts are intentionally unchanged.
- Rollback/Risk: low mechanical refactor inside live sync status producer. Rollback would restore the local summary helper, inline tests, and raw schema/source strings; behavior should remain unchanged because focused sync/status tests cover the moved path.

## 2026-04-20 - Align staged promotion status helpers
- Summary: grouped staged promotion status JSON field names and signal source strings under namespaced constants, keeping the producer document-driven while removing flat schema string clutter.
- Tests: moved staged promotion status assertions into a dedicated Rust test module and preserved blocker, partial, handoff, continuation, nested blocking, and remap-complexity behavior.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet project_status_promotion --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cargo test --manifest-path rust/Cargo.toml --quiet sync --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet status --lib`; `python3 scripts/rust_maintainability_report.py`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-ai-workflow`; `git diff --check`.
- Impact: `rust/src/commands/sync/project_status_promotion.rs`, `rust/src/commands/sync/project_status_promotion_tests.rs`, `rust/src/commands/sync/mod.rs`, and AI trace docs. Public CLI behavior, generated docs, Python implementation, and output contracts are intentionally unchanged.
- Rollback/Risk: low mechanical refactor inside staged promotion status producer. Rollback would inline the tests and restore raw schema/source strings; behavior should remain unchanged because focused promotion and sync/status tests cover the moved path.

## 2026-04-20 - Align live promotion status helpers
- Summary: aligned the live promotion status producer with shared sync project-status JSON helpers, grouped live promotion schema keys as namespaced constants, and moved live promotion status tests into a dedicated Rust test module.
- Tests: preserved live promotion readiness, blocker, handoff, continuation, mapping, and availability behavior while removing inline tests from the production producer.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet live_promotion_project_status --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet project_status_promotion --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cargo test --manifest-path rust/Cargo.toml --quiet sync --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet status --lib`; `python3 scripts/rust_maintainability_report.py`; `cargo test --manifest-path rust/Cargo.toml --quiet`.
- Impact: `rust/src/commands/sync/live_project_status_promotion.rs`, `rust/src/commands/sync/live_project_status_promotion_tests.rs`, `rust/src/commands/sync/project_status_json.rs`, `rust/src/commands/sync/mod.rs`, and AI trace docs. Public CLI behavior, generated docs, Python implementation, and output contracts are intentionally unchanged.
- Rollback/Risk: low mechanical refactor inside live promotion status producer. Rollback would restore local JSON helpers and inline tests; behavior should remain unchanged because focused promotion tests cover the moved path.

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
