# ai-changes.md

Current AI change log only.

- Older detailed history moved to [`archive/ai-changes-archive-2026-03-24.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-03-24.md).
- Keep this file limited to the latest active architecture and maintenance changes.

## 2026-03-26 - Gate Sync Shared TUI Flows Behind Tui Feature
- Summary: wrapped the shared interactive browser runner plus sync audit/review terminal entrypoints in `feature = "tui"` gates, added disabled-feature stubs that return a typed `tui` error, and kept the default feature-on behavior unchanged.
- Tests: kept the existing sync audit/review helper coverage and added disabled-feature regression tests alongside the gated entrypoints for the browser, audit, and review runners.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet --lib cli_review_tui_rust_tests` (pass); `cargo test --manifest-path rust/Cargo.toml --quiet --lib cli_audit_preflight_rust_tests` (pass); `cargo test --manifest-path rust/Cargo.toml --quiet --no-default-features --lib cli_review_tui_rust_tests` (blocked)
- Reason: the no-default-features compile is still blocked by unrelated unconditional dashboard TUI modules outside this task scope.
- Validation: `cargo fmt --manifest-path rust/Cargo.toml --all`; focused default-feature sync test targets passed.
- Impact: `rust/src/interactive_browser.rs`, `rust/src/sync/audit_tui.rs`, `rust/src/sync/review_tui.rs`, `rust/src/sync/cli_review_tui_rust_tests.rs`, `docs/internal/ai-status.md`, `docs/internal/ai-changes.md`
- Rollback/Risk: low to medium. The runtime fallback only affects feature-disabled builds; rollback is to restore the previous unconditional terminal UI wiring if the feature gate needs to be removed.
- Follow-up: gate the remaining dashboard TUI modules if full `--no-default-features` crate compilation becomes a requirement.

## 2026-03-26 - Migrate Selected Rust Dashboard Modules To Unified Error Model
- Summary: migrated dashboard URL parsing to the typed `Url` helper, turned import-path validation into the typed `Validation` helper, and kept local JSON decoding on the typed `Json` path for live inspect/import validation. Also switched the full-page screenshot manifest write to the typed JSON serializer path.
- Tests: added focused regression coverage for invalid dashboard URLs, invalid import-relative dashboard paths, and invalid dashboard export index JSON.
- Test Run: `cargo test --quiet --lib screenshot_rust_tests` (blocked)
- Reason: the dirty worktree already contains unrelated compile errors in `rust/src/common.rs` and `rust/src/sync/preflight.rs`, so the targeted dashboard test run could not reach the modified code paths.
- Validation: code review plus targeted regression tests were added; the compilation blocker is outside the owned dashboard slice.
- Impact: `rust/src/dashboard/inspect_live.rs`, `rust/src/dashboard/import_validation.rs`, `rust/src/dashboard/import_lookup.rs`, `rust/src/dashboard/screenshot.rs`, `rust/src/dashboard/screenshot_full_page.rs`, `rust/src/dashboard/inspect_live_rust_tests.rs`, `rust/src/dashboard/import_rust_tests.rs`, `rust/src/dashboard/screenshot_rust_tests.rs`
- Rollback/Risk: medium. The behavioral change is limited to error categorization and wording for local failures; command semantics and 404 handling are unchanged.
- Follow-up: rerun the targeted dashboard tests once the unrelated `common.rs` / `sync/preflight.rs` compile breakage is resolved.

## 2026-03-26 - Migrate Selected Rust Sync Modules To Unified Error Model
- Summary: added shared JSON object/array/field helpers in `rust/src/sync/json.rs` and migrated the sync apply/audit/preflight/review/staged-document readers to use them for obvious document-shape and validation failures.
- Tests: touched sync regression coverage was left in place; no new assertions were required for the helper migration.
- Test Run: `cargo test --quiet --lib load_apply_intent_operations_requires_operations_array` (blocked by pre-existing unrelated compile errors in `src/common.rs` after the sync code compiled far enough to reach the shared library path).
- Validation: `git diff --check` passed on the touched files; `rustfmt --edition 2021` was run on the edited Rust files.
- Impact: `rust/src/sync/json.rs`, `rust/src/sync/apply_builder.rs`, `rust/src/sync/audit.rs`, `rust/src/sync/bundle_preflight.rs`, `rust/src/sync/live_apply.rs`, `rust/src/sync/preflight.rs`, `rust/src/sync/review_tui.rs`, `rust/src/sync/staged_documents.rs`, `rust/src/sync/summary_builder.rs`, `docs/internal/ai-status.md`, `docs/internal/ai-changes.md`
- Rollback/Risk: low. The refactor only centralizes local validation checks; rollback is to restore the previous inline shape guards if a caller depends on the new helper wording.
- Follow-up: resolve the unrelated `src/common.rs` `as_dyn_error` compile failure before relying on `cargo test` for repo-wide validation.

## 2026-03-24 - Extract Dashboard Import Routed Orchestration
- Summary: moved the export-org routed import flow, including routed preview JSON assembly and routed dispatch, into `rust/src/dashboard/import_routed.rs` while keeping `rust/src/dashboard/import.rs` focused on the single-org import facade and shared import loop.
- Tests: reused the existing dashboard import coverage; no new assertions were needed for the refactor.
- Test Run: `cargo check --quiet --lib` (pass); `cargo test --quiet --lib dashboard_rust_tests` (pass)
- Validation: confirmed the library checks and the focused dashboard Rust test target pass after the module split.
- Impact: `rust/src/dashboard/import.rs`, `rust/src/dashboard/import_routed.rs`, `rust/src/dashboard/mod.rs`, `docs/internal/ai-status.md`, `docs/internal/ai-changes.md`
- Rollback/Risk: low. This is an internal module split with no JSON contract or CLI behavior change; rollback is to inline the routed helper module again if needed.
- Follow-up: none.

## 2026-03-24 - Extract Dashboard Screenshot Header Helpers
- Summary: moved dashboard screenshot metadata resolution, header spec construction, header image composition, and manifest title resolution into `rust/src/dashboard/screenshot_header.rs`, leaving `rust/src/dashboard/screenshot.rs` focused on orchestration and browser capture flow.
- Tests: reused the existing screenshot coverage; no new assertions were needed for the refactor.
- Test Run: `cargo test --quiet --lib screenshot_rust_tests` (pass); `cargo check --quiet --lib` (pass)
- Validation: confirmed the focused screenshot test target and the broader lib check both pass after the module split.
- Impact: `rust/src/dashboard/screenshot.rs`, `rust/src/dashboard/screenshot_header.rs`, `rust/src/dashboard/screenshot_full_page.rs`, `docs/internal/ai-status.md`, `docs/internal/ai-changes.md`
- Rollback/Risk: low. This is an internal module split with no CLI or JSON/output contract change; rollback is to inline the helper module back into `screenshot.rs` if needed.
- Follow-up: none.

## 2026-03-24 - Split Dashboard Inspect Governance Risk Logic
- Summary: extracted the governance risk scoring, audit row builders, and governance risk row assembly into `rust/src/dashboard/inspect_governance_risk.rs` while keeping `rust/src/dashboard/inspect_governance.rs` as a stable facade.
- Tests: no new behavior tests were needed; existing governance inspect coverage was reused.
- Test Run: `cargo test --quiet --lib inspect_governance_rust_tests` (pass)
- Validation: confirmed the focused governance inspect test file still passes after the split.
- Impact: `rust/src/dashboard/inspect_governance.rs`, `rust/src/dashboard/inspect_governance_risk.rs`
- Rollback/Risk: low. The change is a refactor with no contract change; rollback is to inline the helper module again if needed.
- Follow-up: none.

## 2026-03-24 - Split Sync Bundle Tests Out Of Sync CLI Suite
- Summary: Moved the bundle export and bundle-preflight CLI coverage out of `rust/src/sync/cli_rust_tests.rs` and into the existing feature-specific bundle test module so the sync CLI test file is smaller and the bundle behavior stays grouped with the bundle preflight contract tests.
- Tests: Updated the moved bundle CLI cases and bundle-preflight acceptance coverage in the new module. `rustfmt --check rust/src/sync/bundle_rust_tests.rs rust/src/sync/cli_rust_tests.rs` passed.
- Test Run: `cargo test --quiet --lib sync_bundle_rust_tests` and `cargo test --quiet --lib sync_cli_rust_tests` both failed in unrelated dashboard compile paths (`src/dashboard/inspect_governance.rs` / missing `inspect_governance_risk` module and symbols).
- Reason: The repository currently has pre-existing dashboard compile failures that prevent a clean cargo test run for the sync test targets.
- Validation: Confirmed the touched sync test files are rustfmt-clean after the split.
- Impact: `rust/src/sync/cli_rust_tests.rs`, `rust/src/sync/bundle_rust_tests.rs`, `docs/internal/ai-status.md`, `docs/internal/ai-changes.md`
- Rollback/Risk: Low. The change only relocates tests between modules and keeps test names and assertions intact.
- Follow-up: Fix the unrelated dashboard compile breakage before relying on cargo test output for sync test validation.

## 2026-03-24 - Archive Historical AI Change Log
- Summary: Archived the oversized historical AI trace files into `docs/internal/archive/` and reset the top-level `ai-status.md` / `ai-changes.md` files to current-only summaries.
- Validation: Confirmed the archive files exist at the new paths and the current files now point to them.
- Impact: `docs/internal/ai-status.md`, `docs/internal/ai-changes.md`, `docs/internal/archive/ai-status-archive-2026-03-24.md`, `docs/internal/archive/ai-changes-archive-2026-03-24.md`
- Rollback/Risk: Low. This is a documentation-only archive move; rollback is just restoring the files to the previous paths if a flat history file is preferred.

## 2026-03-24 - Split Rust Sync Workbench Into Builder Modules
- Summary: Split the Rust sync workbench implementation into `summary_builder`, `bundle_builder`, `plan_builder`, and `apply_builder` modules while leaving `rust/src/sync/workbench.rs` as the stable facade for existing public and crate-visible entrypoints.
- Validation: `cargo check --quiet --lib`; focused sync builder tests passed during the refactor.
- Impact: `rust/src/sync/workbench.rs`, `rust/src/sync/summary_builder.rs`, `rust/src/sync/bundle_builder.rs`, `rust/src/sync/plan_builder.rs`, `rust/src/sync/apply_builder.rs`, `rust/src/sync/mod.rs`

## 2026-03-24 - Phase Split Dashboard Import Flow
- Summary: Split dashboard import responsibilities into lookup, validation, render, and compare helpers so `rust/src/dashboard/import.rs` reads as orchestration instead of a mixed implementation file.
- Validation: `cargo check --quiet --lib`; `cargo test --quiet --lib dashboard_rust_tests`; `cargo test --quiet --lib inspect_export_rust_tests`
- Impact: `rust/src/dashboard/import.rs`, `rust/src/dashboard/import_lookup.rs`, `rust/src/dashboard/import_validation.rs`, `rust/src/dashboard/import_render.rs`, `rust/src/dashboard/import_compare.rs`, `rust/src/dashboard/mod.rs`

## 2026-03-24 - Extract Governance Rule Engine
- Summary: Moved dashboard governance policy parsing and rule evaluation into `rust/src/dashboard/governance_gate_rules.rs`, leaving `governance_gate.rs` focused on command/result orchestration.
- Validation: `cargo check --quiet --lib`; focused governance tests passed during the refactor.
- Impact: `rust/src/dashboard/governance_gate.rs`, `rust/src/dashboard/governance_gate_rules.rs`

## 2026-03-24 - Continue Dashboard Test Splits
- Summary: Moved focused dashboard coverage out of the umbrella test file into dedicated modules, including live inspect, query/governance, inspect-export parser/help, and screenshot slices.
- Validation: `cargo test --quiet --lib dashboard_rust_tests`; `cargo test --quiet --lib inspect_export_rust_tests`
- Impact: `rust/src/dashboard/rust_tests.rs`, `rust/src/dashboard/inspect_live_rust_tests.rs`, `rust/src/dashboard/inspect_query_rust_tests.rs`, `rust/src/dashboard/inspect_governance_rust_tests.rs`, `rust/src/dashboard/inspect_export_rust_tests.rs`, `rust/src/dashboard/screenshot_rust_tests.rs`
