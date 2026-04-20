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

## 2026-04-20 - Finish project status producer audit
- Summary: audited remaining project-status producers across sync, datasource, alert, dashboard, access, and live status fallback paths, then normalized the last dashboard live read-failure fallback through `StatusReading`.
- Tests: preserved live dashboard read failure status fields, blocker count derivation, and existing staged/live domain status evidence including health, version, discovery, and freshness paths.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet dashboard --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet access --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet status --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet project_status --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `make quality-architecture`; `make quality-ai-workflow`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `git diff --check`.
- Impact: `rust/src/grafana/api/project_status_live.rs`, `todo.md`, and AI trace docs. Public CLI behavior, generated docs, Python implementation, and output contracts are intentionally unchanged.
- Rollback/Risk: low test-support normalization. Rollback would restore the direct fallback `ProjectDomainStatus` literal; the shared model derives the same blocker and warning counts from the same data.

## 2026-04-20 - Move dashboard import into directory boundary
- Summary: moved dashboard import implementation files under `commands/dashboard/import/` while keeping `commands/dashboard/mod.rs` as the facade and leaving plan reconciliation under `commands/dashboard/plan/`.
- Tests: preserved import dry-run/apply lookup boundaries, routed import reporting, dashboard plan relationships, and browse interactive import coverage through existing regression suites.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet import --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet routed_import --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_plan --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet interactive_import --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `python3 scripts/rust_maintainability_report.py`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-ai-workflow`; `git diff --check`.
- Impact: `rust/src/commands/dashboard/mod.rs`, `rust/src/commands/dashboard/import/`, `todo.md`, and AI trace docs. Public CLI behavior, generated docs, Python implementation, and output contracts are intentionally unchanged.
- Rollback/Risk: low mechanical module move. Rollback would move the import files back to the flat dashboard directory and restore the root module declarations; focused import/routed/plan/browse and full Rust tests cover the moved paths.

## 2026-04-20 - Clean up alert runtime schema keys
- Summary: grouped alert runtime plan, delete-preview, and import dry-run document keys behind local schema namespaces while preserving Grafana raw alert payload reads.
- Tests: preserved alert plan row summaries, plan execution reads, delete preview output, import dry-run summaries, and existing readable JSON fixtures.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet alert --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet runtime --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet sync --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet import --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet import_validation --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `python3 scripts/rust_maintainability_report.py`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-ai-workflow`; `git diff --check`.
- Impact: `rust/src/commands/alert/runtime_support.rs`, `todo.md`, and AI trace docs. Public CLI behavior, generated docs, Python implementation, and output contracts are intentionally unchanged.
- Rollback/Risk: low mechanical key centralization. Rollback would restore repeated raw alert runtime document keys; focused alert/runtime and full Rust tests cover the touched paths.

## 2026-04-20 - Clean up dashboard import dependency schema keys
- Summary: grouped dashboard import dependency availability, desired-spec, and preflight summary keys behind local schema namespaces while preserving ordinary Grafana raw payload reads.
- Tests: preserved dependency preflight behavior for datasource/plugin availability, blocking summaries, routed import preflight, and dashboard plan relationships.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet import --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet preflight --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_plan --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `python3 scripts/rust_maintainability_report.py`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-ai-workflow`; `git diff --check`.
- Impact: `rust/src/commands/dashboard/import_validation_dependencies.rs`, `todo.md`, and AI trace docs. Alert runtime schema cleanup, import directory moves, public CLI behavior, generated docs, Python implementation, and output contracts are intentionally unchanged.
- Rollback/Risk: low mechanical key centralization. Rollback would restore repeated raw preflight strings; focused import/preflight/dashboard-plan tests cover the touched path.

## 2026-04-20 - Move dashboard authoring into directory boundary
- Summary: moved dashboard authoring implementation and direct authoring regression tests under `commands/dashboard/authoring/` while keeping `commands/dashboard/mod.rs` as the facade.
- Tests: preserved dashboard authoring helper imports, root test registration, and public dashboard CLI command paths.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet authoring --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `python3 scripts/rust_maintainability_report.py`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-ai-workflow`; `git diff --check`.
- Impact: `rust/src/commands/dashboard/mod.rs`, `rust/src/commands/dashboard/authoring/mod.rs`, `rust/src/commands/dashboard/authoring/rust_tests.rs`, `todo.md`, and AI trace docs. Import/reconcile and inspect/governance directory moves are intentionally unchanged.
- Rollback/Risk: low mechanical move. Rollback would move the two authoring files back to the flat dashboard directory and restore the root test path; focused authoring/dashboard tests cover the module path.

## 2026-04-20 - Split sync live read facets
- Summary: moved sync live read dashboard/folder, datasource, alert, and availability assembly logic into dedicated child modules while keeping `sync_live_read.rs` as the facade.
- Tests: preserved live resource-spec and availability behavior for client and request-closure paths.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet sync_live --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet fetch_live_resource_specs_with_request --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet fetch_live_availability_with_request --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet status --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `python3 scripts/rust_maintainability_report.py`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-ai-workflow`; `git diff --check`.
- Impact: `rust/src/grafana/api/sync_live_read.rs`, `rust/src/grafana/api/sync_live_read/dashboard.rs`, `rust/src/grafana/api/sync_live_read/datasource.rs`, `rust/src/grafana/api/sync_live_read/alert.rs`, `rust/src/grafana/api/sync_live_read/availability.rs`, `todo.md`, and AI trace docs. Public CLI/output contracts are intentionally unchanged.
- Rollback/Risk: low mechanical extraction. Rollback would inline the dashboard/folder, datasource, alert, and availability loops back into `sync_live_read.rs`; focused sync-live tests cover both request and client paths.

## 2026-04-20 - Clean up sync staged schema keys
- Summary: grouped sync staged render, workspace preview review, and project-status helper document keys under local namespaced constants while preserving the existing public JSON/output shape.
- Tests: preserved sync summary/plan/apply-intent rendering, workspace preview review normalization, and sync project-status aggregation behavior.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet render_sync --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet workspace_preview_review_view --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet sync_project_status --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet sync --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet status --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `python3 scripts/rust_maintainability_report.py`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-ai-workflow`; `git diff --check`.
- Impact: `rust/src/commands/sync/staged_documents_render.rs`, `rust/src/commands/sync/workspace_preview_review_view.rs`, `rust/src/commands/sync/project_status_json.rs`, `rust/src/commands/sync/project_status.rs`, `todo.md`, and AI trace docs. Public CLI behavior, generated docs, Python implementation, and output contracts are intentionally unchanged.
- Rollback/Risk: low mechanical key centralization in sync-owned document readers. Rollback would restore repeated raw strings; focused and full Rust tests cover the moved access paths.

## 2026-04-20 - Continue Rust split and schema key cleanup
- Summary: split overview parser/basic-render contract assertions, split alert runtime command args into a dedicated module, extracted project-status live HTTP test support, and grouped sync preflight summary/availability/body JSON keys under namespaced constants.
- Tests: preserved overview contract rendering/parser behavior, alert CLI parser behavior, project-status live HTTP test coverage, and sync preflight dependency checks.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet overview_contract --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet alert --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet grafana_api::project_status_live::tests --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet preflight --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet sync --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `python3 scripts/rust_maintainability_report.py`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-ai-workflow`; `git diff --check`.
- Impact: `rust/src/commands/status/overview/contract_tests.rs`, `rust/src/commands/status/overview/contract_parser_tests.rs`, `rust/src/commands/alert/cli/args.rs`, `rust/src/commands/alert/cli/args_runtime.rs`, `rust/src/grafana/api/project_status_live_tests.rs`, `rust/src/grafana/api/project_status_live_test_support.rs`, `rust/src/commands/sync/preflight.rs`, and AI trace docs. Public CLI behavior, generated docs, Python implementation, and output contracts are intentionally unchanged.
- Rollback/Risk: low mechanical split and constant centralization. Rollback would inline the moved test/args helpers and restore repeated JSON key literals; behavior is covered by focused and full Rust tests.

## 2026-04-20 - Split alert authoring CLI args
- Summary: moved alert authoring scaffold/add/clone/route argument structs into a dedicated adjacent module while keeping `args.rs` as the alert CLI facade.
- Tests: preserved alert parser and command coverage with no public CLI shape changes.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet alert --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `python3 scripts/rust_maintainability_report.py`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-ai-workflow`; `git diff --check`.
- Impact: `rust/src/commands/alert/cli/args.rs`, `rust/src/commands/alert/cli/args_authoring.rs`, and AI trace docs. Public CLI behavior, generated docs, Python implementation, and output contracts are intentionally unchanged.
- Rollback/Risk: low mechanical module split. Rollback would inline authoring args back into the facade file; behavior is unchanged.

## 2026-04-20 - Split status parser tests
- Summary: moved status CLI help, parser, and output-mode assertions into a dedicated adjacent Rust test module while leaving staged/render contract fixtures in the original test file.
- Tests: preserved parser coverage for datasource provisioning, output modes, dashboard provisioning, and conflicting dashboard inputs.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet project_status_cli --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet status --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `python3 scripts/rust_maintainability_report.py`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-ai-workflow`; `git diff --check`.
- Impact: `rust/src/commands/status/tests.rs`, `rust/src/commands/status/parser_tests.rs`, and AI trace docs. Public CLI behavior, generated docs, Python implementation, and output contracts are intentionally unchanged.
- Rollback/Risk: low mechanical test split. Rollback would move parser tests back into the original status test file; behavior is unchanged.
