# ai-status.md

Current AI-maintained status only.

- Older trace history moved to [`archive/ai-status-archive-2026-03-24.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-status-archive-2026-03-24.md).
- Keep this file short and current. Additive historical detail belongs in `docs/internal/archive/`.

## 2026-03-26 - Task: Wire Governance-Gate Runtime Evaluation To Dynamic Policy Sources In Rust
- State: Planned
- Scope: `rust/src/dashboard/governance_gate.rs`, related dashboard governance gate tests
- Baseline: governance-gate only loads policy from a JSON file path before evaluating the resolved policy model.
- Current Update: scoped the runtime change and traced the policy-loading choke point; next step is to add JSON/YAML/built-in policy source resolution without changing the rule engine.

## 2026-03-26 - Task: Gate Sync Shared TUI Flows Behind Tui Feature
- State: Done
- Scope: `rust/src/interactive_browser.rs`, `rust/src/sync/audit_tui.rs`, `rust/src/sync/review_tui.rs`, `rust/src/sync/cli_review_tui_rust_tests.rs`
- Baseline: sync audit and review TUI entrypoints, plus the shared interactive browser helper, were compiled as unconditional TUI-backed paths and would fail at compile time when the `tui` feature was disabled.
- Current Update: feature-gated the interactive browser plus sync audit/review terminal runners and kept the default feature-on code path unchanged.
- Result: default-feature sync tests passed; the broader `--no-default-features` crate build is still blocked by unrelated unconditional dashboard TUI modules outside this scope.

## 2026-03-26 - Task: Migrate Selected Rust Dashboard Modules To Unified Error Model
- State: Blocked
- Scope: `rust/src/dashboard/inspect_live.rs`, `rust/src/dashboard/import_validation.rs`, `rust/src/dashboard/import_lookup.rs`, `rust/src/dashboard/screenshot.rs`, `rust/src/dashboard/screenshot_full_page.rs`, `rust/src/dashboard/inspect_live_rust_tests.rs`, `rust/src/dashboard/import_rust_tests.rs`, `rust/src/dashboard/screenshot_rust_tests.rs`
- Baseline: dashboard helpers still used several ad hoc `message(...)` wrappers for URL parsing, import path resolution, JSON file decoding, and a few local filesystem/build failures.
- Current Update: migrated the obvious local failure sites to the typed error helpers already present in `common.rs` and added focused regression tests for URL, JSON, and import-path failures.
- Result: code updated, but targeted Rust test runs are blocked by unrelated pre-existing compile errors in `rust/src/common.rs` and `rust/src/sync/preflight.rs`.

## 2026-03-26 - Task: Migrate Selected Rust Sync Modules To Unified Error Model
- State: Done
- Scope: `rust/src/sync/json.rs`, `rust/src/sync/apply_builder.rs`, `rust/src/sync/audit.rs`, `rust/src/sync/bundle_preflight.rs`, `rust/src/sync/live_apply.rs`, `rust/src/sync/preflight.rs`, `rust/src/sync/review_tui.rs`, `rust/src/sync/staged_documents.rs`, `rust/src/sync/summary_builder.rs`, related sync tests
- Baseline: sync builders and TUI helpers still carried several local `as_object` / `as_array` validation branches and duplicate document-shape checks with ad hoc `message(...)` errors.
- Current Update: shared JSON field helpers now cover the obvious sync document/object/array validation paths, and the sync readers/builders/TUI entrypoints use them instead of repeating shape guards inline.
- Result: the sync-side validation paths are more consistent and the unchanged sync behavior is still gated by the same document semantics.

## 2026-03-24 - Task: Extract Dashboard Import Routed Orchestration
- State: Done
- Scope: `rust/src/dashboard/import.rs`, `rust/src/dashboard/import_routed.rs`, `rust/src/dashboard/mod.rs`
- Baseline: `import.rs` still owns the export-org routing flow, including preview JSON assembly and routed import dispatch, alongside the main single-org import facade.
- Current Update: moved the routed-org orchestration into a dedicated helper module so `import.rs` reads more like a facade over single-org import behavior.
- Result: `import.rs` is smaller and the focused dashboard Rust tests still pass.

## 2026-03-24 - Split Dashboard Inspect Governance Risk Logic
- State: Done
- Scope: `rust/src/dashboard/inspect_governance.rs`, `rust/src/dashboard/inspect_governance_risk.rs`
- Baseline: governance risk scoring, audit row builders, and risk-row assembly were all inlined in `inspect_governance.rs`.
- Current Update: moved the risk logic into a dedicated submodule and kept the parent module as a stable facade for the existing inspect governance document path.
- Result: `inspect_governance.rs` is materially smaller and the focused governance inspect tests still pass.

## 2026-03-24 - Current Maintainer State
- State: Active
- Scope: Rust maintainability cleanup across `dashboard/`, `sync/`, and focused test splits.
- Current Shape:
  - `rust/src/sync/workbench.rs` is now a facade over builder-oriented helpers in `summary_builder.rs`, `bundle_builder.rs`, `plan_builder.rs`, and `apply_builder.rs`.
  - `rust/src/dashboard/import.rs` is now an orchestration layer over `import_lookup.rs`, `import_validation.rs`, `import_render.rs`, `import_compare.rs`, and `import_routed.rs`.
  - Governance rule evaluation lives in `rust/src/dashboard/governance_gate_rules.rs`, with `governance_gate.rs` reduced to command/result orchestration.
  - Large dashboard test coverage has started moving out of `rust/src/dashboard/rust_tests.rs` into feature files such as `inspect_live_rust_tests.rs`, `inspect_query_rust_tests.rs`, `inspect_governance_rust_tests.rs`, `inspect_export_rust_tests.rs`, and `screenshot_rust_tests.rs`.
- Result:
  - Remaining complexity is primarily feature density, not AI-style structural drift.
  - Current maintainability work is centered on phase boundaries, typed/stable contracts, and feature-scoped test files.

## 2026-03-24 - Extract Dashboard Screenshot Header Helpers
- State: Done
- Scope: `rust/src/dashboard/screenshot.rs`, `rust/src/dashboard/screenshot_header.rs`, `rust/src/dashboard/screenshot_full_page.rs`
- Baseline: screenshot metadata resolution, header spec construction, header image composition, and title resolution all lived inside `screenshot.rs`.
- Current Update: moved the header/metadata helpers into a dedicated submodule and wired the full-page renderer to use the new boundary.
- Result: `screenshot.rs` is now mostly orchestration, while the header/metadata and header rendering helpers live in `screenshot_header.rs`.

## 2026-03-24 - Split Sync Bundle Tests Out Of Sync CLI Suite
- State: Done
- Scope: `rust/src/sync/cli_rust_tests.rs`, `rust/src/sync/bundle_rust_tests.rs`
- Baseline: bundle-oriented sync CLI coverage lived inside the umbrella sync CLI test file alongside parser, review, apply, and audit checks.
- Current Update: Moved the bundle export and bundle-preflight CLI coverage into the existing bundle-focused test module so the sync CLI suite is smaller and the bundle feature tests live together.
- Result: `cli_rust_tests.rs` now excludes the large bundle block, while `bundle_rust_tests.rs` owns the bundle-oriented CLI cases and bundle-preflight acceptance path.
