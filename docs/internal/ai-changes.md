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
- Older entries moved to [`ai-changes-archive-2026-04-27.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-04-27.md).

## 2026-04-26 - Prove provisioning remains derived dashboard projection
- Summary: added regression coverage that provisioning dashboard files normalize to the same classic dashboard compare payload as raw export wrappers, and that sync bundle rejects explicit dual raw/provisioning dashboard inputs instead of treating provisioning as an alternate source of truth.
- Tests: covered raw wrapper vs provisioning compare normalization, provisioning classic payload shape, direct provisioning source loading, import dry-run provisioning roots, and sync bundle dual dashboard source rejection.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet compare_local_document_`; `cargo test --manifest-path rust/Cargo.toml --quiet source_loader_contract_resolves_direct_provisioning_root`; `cargo test --manifest-path rust/Cargo.toml --quiet collect_import_dry_run_report_accepts_provisioning_root_variant_metadata`; `cargo test --manifest-path rust/Cargo.toml --quiet run_sync_cli_bundle_rejects_dual_dashboard_sources`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`.
- Impact: `rust/src/commands/dashboard/import/compare.rs`, `rust/src/commands/sync/bundle_exec_sources_rust_tests.rs`, `todo.md`, and AI trace docs. Public JSON, generated docs, Python implementation, and dashboard v2 support are intentionally unchanged.
- Rollback/Risk: low test-only boundary hardening. Rollback would remove the regression coverage while leaving existing provisioning behavior unchanged.

## 2026-04-27 - Guard dashboard permissions as adjacent evidence
- Summary: rejected dashboard permission bundle/export artifacts from shared dashboard JSON extraction and added workspace/access regressions so permission bundles remain adjacent evidence rather than dashboard JSON or prompt export input.
- Tests: covered preserved-web-import and import-payload rejection for permission bundles, raw-to-prompt single-file rejection, review artifact resolution with raw `permissions.json`, sync workspace bundle auto-discovery ignoring dashboard permission bundles, and access `resource=all` ignoring dashboard workspace JSON.
- Test Run: `cargo test --manifest-path rust/Cargo.toml build_preserved_web_import_document_rejects_permission_bundle --quiet`; `cargo test --manifest-path rust/Cargo.toml raw_to_prompt_single_file_rejects_permission_bundle --quiet`; `cargo test --manifest-path rust/Cargo.toml run_sync_cli_bundle_workspace_auto_discovery_ignores_dashboard_permissions_bundle --quiet`; `cargo test --manifest-path rust/Cargo.toml all_plan_ignores_dashboard_workspace_json_when_collecting_access_bundles --quiet`.
- Impact: `rust/src/commands/dashboard/files.rs`, dashboard regression tests, `rust/src/commands/sync/bundle_exec_sources_rust_tests.rs`, `rust/src/commands/access/access_plan_tests.rs`, `todo.md`, and AI trace docs. Permission restore/apply behavior, public JSON contracts, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low targeted boundary fix. Rollback would allow single-object dashboard paths to reinterpret permission artifacts as dashboards and remove cross-domain guard coverage.

## 2026-04-27 - Guard Git Sync dashboard live apply boundaries
- Summary: treated workspace-backed dashboard browse sources as local review trees and added sync apply handoff regressions so Git Sync-managed dashboards remain Git-owned targets instead of becoming direct live API writes.
- Tests: covered workspace browse local-source detection, apply-intent ownership/provenance preservation, Git Sync dashboard live-apply rejection before transport, and reusable command-output live-apply rejection.
- Test Run: `cargo test --manifest-path rust/Cargo.toml workspace_roots_are_treated_as_local_browse_sources --quiet`; `cargo test --manifest-path rust/Cargo.toml build_sync_apply_intent_document_preserves_dashboard_ownership_provenance --quiet`; `cargo test --manifest-path rust/Cargo.toml execute_live_apply_with_request_blocks_git_sync_dashboard_from_apply_intent_handoff --quiet`; `cargo test --manifest-path rust/Cargo.toml execute_sync_command_rejects_live_apply_reusable_output --quiet`.
- Impact: `rust/src/commands/dashboard/browse/mod.rs`, `rust/src/commands/dashboard/browse/tui.rs`, `rust/src/commands/sync/rust_tests.rs`, `rust/src/commands/sync/live_rust_tests.rs`, `rust/src/commands/sync/cli_rust_tests.rs`, `todo.md`, and AI trace docs. Public JSON, generated docs, Python implementation, and actual Git PR automation are intentionally unchanged.
- Rollback/Risk: low behavior-boundary fix. Rollback would make workspace-backed local browse trees depend on the older `input_dir`-only local-mode check and would remove the sync handoff regressions.

## 2026-04-26 - Add dashboard v2 adapter boundary regressions
- Summary: added adapter-facing regression coverage so classic dashboard diff and root-export source-wrapper paths reject Grafana dashboard v2 resources before any remote compare or normalized temp import source can blur the future adapter boundary.
- Tests: covered raw diff rejection, provisioning diff rejection, root raw export normalization rejection, and root provisioning export normalization rejection using the shared Grafana-source `v2-elements` fixture.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet diff_dashboards_with_client_rejects_raw_dashboard_v2_resource_before_remote_compare`; `cargo test --manifest-path rust/Cargo.toml --quiet diff_dashboards_with_client_rejects_provisioning_dashboard_v2_resource_before_remote_compare`; `cargo test --manifest-path rust/Cargo.toml --quiet import_loaded_source_rust_tests`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`.
- Impact: `rust/src/commands/dashboard/export_diff_rust_tests.rs`, `rust/src/commands/dashboard/import_loaded_source_rust_tests.rs`, `todo.md`, and AI trace docs. Public CLI/docs, generated artifacts, Python implementation, and actual dashboard v2 support are intentionally unchanged.
- Rollback/Risk: low test-only boundary hardening. Rollback would remove the regression coverage and reopen a gap around future adapter entrypoints without changing current classic-lane behavior.

## 2026-04-26 - Bound library-panel elements to live export
- Summary: removed raw-to-prompt live library-panel model lookup so `dashboard convert raw-to-prompt` keeps library-panel references warning-only with empty `__elements`, while live export/import-handoff remains the only path that fetches live library-panel models into prompt `__elements`.
- Tests: updated raw-to-prompt live-lookup tests to prove datasource live lookup still works without library-panel inlining, preserved warning-only missing-model behavior, and kept the live export prompt `__elements` regression passing.
- Test Run: `cargo test --manifest-path rust/Cargo.toml raw_to_prompt --quiet`; `cargo test --manifest-path rust/Cargo.toml export_dashboards_with_request_includes_live_library_panel_elements_in_prompt_variant --quiet`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`.
- Impact: `rust/src/commands/dashboard/raw_to_prompt/resolution.rs`, `rust/src/commands/dashboard/raw_to_prompt/prompt_paths.rs`, `rust/src/commands/dashboard/raw_to_prompt/rust_tests.rs`, `todo.md`, and AI trace docs. Live export `__elements`, dashboard v2 rejection, Python implementation, and generated docs are intentionally unchanged.
- Rollback/Risk: low behavior-boundary change. Rollback would reintroduce live library-panel fetches into raw-to-prompt; focused raw-to-prompt and live export tests cover the intended split.

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
