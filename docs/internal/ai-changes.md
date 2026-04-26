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
- Older entries moved to [`ai-changes-archive-2026-04-26.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-04-26.md).

## 2026-04-26 - Type dashboard ownership evidence
- Summary: extended the existing `DashboardTargetOwnership` model with typed label parsing and evidence-note helpers, then reused it in the sync live dashboard write guard instead of rebuilding `ownership=...` strings locally.
- Tests: added ownership evidence helper coverage for duplicate preservation, known ownership insertion, unknown labels, and direct-write blocking; preserved dashboard plan and sync live owned-dashboard guard behavior.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet target_ownership_evidence`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_plan_blocks_git_sync_managed`; `cargo test --manifest-path rust/Cargo.toml --quiet sync_live_client_rejects_owned_dashboard_before_transport`; `cargo test --manifest-path rust/Cargo.toml --quiet execute_live_apply_with_request_rejects_owned_dashboard_before_transport`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`.
- Impact: `rust/src/commands/dashboard/import/target.rs`, `rust/src/grafana/api/sync_live_apply_dashboard.rs`, `todo.md`, and AI trace docs. Public JSON fields, direct live write policy, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low internal-model change. Rollback would restore local string construction in sync live apply; focused ownership and sync live guard tests cover the behavior.

## 2026-04-26 - Add Git Sync dashboard review layout evidence
- Summary: classified local dashboard review inputs as plain export or repo-backed Git Sync layout, surfaced `inputLayout` in dashboard plan JSON/text output, labeled Git Sync dashboard inputs in workspace discovery provenance, and reused one Grafana-source parity fixture bundle for datasource variable, library-panel, and v2 rejection tests.
- Tests: added Git Sync layout coverage for source loading, browse source labels, dashboard plan output, workspace discovery labels, and fixture-backed raw-to-prompt/validate/import v2 boundary behavior.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet raw_to_prompt_`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_plan`; `cargo test --manifest-path rust/Cargo.toml --quiet source_loader_contract`; `cargo test --manifest-path rust/Cargo.toml --quiet workspace_root_browse`; `cargo test --manifest-path rust/Cargo.toml --quiet discovery_model`; `cargo test --manifest-path rust/Cargo.toml --quiet workspace_discovery`; `cargo test --manifest-path rust/Cargo.toml --quiet validate_dashboard`; `cargo test --manifest-path rust/Cargo.toml --quiet import_loaded_source`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_cli_parser_help_workflow`; `make test-rust-live`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cargo test --manifest-path rust/Cargo.toml --quiet`.
- Impact: `rust/src/commands/dashboard/source_loader*`, dashboard browse/plan rendering, workspace discovery labels, `tests/fixtures/dashboard_grafana_source_parity_cases.json`, focused dashboard tests, `todo.md`, and AI trace docs. Direct live apply/write semantics, dashboard v2 support, Python implementation, and generated docs are intentionally unchanged.
- Rollback/Risk: moderate review-surface change because dashboard plan JSON now includes `inputLayout` and text output includes an input-layout line. Rollback would remove layout classification and keep existing local export behavior; focused plan/discovery tests cover the new review labels.

## 2026-04-20 - Add dashboard folder permission drift review
- Summary: added a read-only folder permission drift lane to `dashboard plan`, including `--include-folder-permissions`, UID-first matching, optional `uid-then-path` fallback, folder permission action rows, permission detail rendering, and synced English/zh-TW command docs.
- Tests: added parser/help coverage, permission drift action coverage for same/update/extra/missing/path-fallback cases, and an input-collection regression that loads `raw/permissions.json` and fetches live folder permissions.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_plan --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_cli_parser_help_workflow --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `make quality-docs-surface`; `cargo test --manifest-path rust/Cargo.toml --quiet`.
- Impact: `rust/src/commands/dashboard/cli_defs_command_plan.rs`, `rust/src/commands/dashboard/plan/`, `rust/src/commands/dashboard/plan_types.rs`, `rust/src/commands/dashboard/dashboard_runtime.rs`, `docs/commands/en/dashboard-plan.md`, `docs/commands/zh-TW/dashboard-plan.md`, and AI trace docs. Import-time permission restore, dashboard ACL diff, Python implementation, and generated docs are intentionally unchanged.
- Rollback/Risk: moderate review-surface change. Rollback would remove the optional flag and permission action lane while keeping existing dashboard plan behavior; the feature is opt-in and read-only, but plan summary counts include folder permission rows when enabled.
- Follow-up: add dashboard permission diff or import-time folder permission restore only after subject-resolution and ACL apply policy are finalized.

## 2026-04-20 - Add contract promotion report
- Summary: made `scripts/contract_promotion_report.py` a concrete informational evidence matrix for runtime golden, schema/help manifest, public route, docs entrypoint, generated docs, and artifact workspace lanes.
- Tests: added report unit coverage for actual manifest route shapes, runtime-only rows, generated-doc detection, artifact workspace evidence, deterministic ordering, categorized findings, and informational default exits.
- Test Run: `python3 -m unittest -v scripts.test_contract_promotion_report`; `python3 scripts/contract_promotion_report.py`; `python3 scripts/contract_promotion_report.py --strict-structure`; `make contract-promotion-report`; `make quality-output-contracts`; `make schema-check`; `make quality-ai-workflow`; `git diff --check`.
- Impact: `scripts/contract_promotion_report.py`, `scripts/test_contract_promotion_report.py`, `docs/internal/contract-doc-map.md`, `todo.md`, and AI trace docs. Public CLI behavior, generated docs, schema artifacts, Rust implementation, and Python package behavior are intentionally unchanged.
- Rollback/Risk: low script/reporting change. Rollback would restore the previous overlap report with less route and runtime-only evidence; the report remains informational and does not gate findings.

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
