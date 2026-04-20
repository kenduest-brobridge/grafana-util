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
