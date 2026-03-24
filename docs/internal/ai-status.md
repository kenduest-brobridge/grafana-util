# ai-status.md

Current AI-maintained status only.

- Older trace history moved to [`archive/ai-status-archive-2026-03-24.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-status-archive-2026-03-24.md).
- Keep this file short and current. Additive historical detail belongs in `docs/internal/archive/`.

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
  - `rust/src/dashboard/import.rs` is now an orchestration layer over `import_lookup.rs`, `import_validation.rs`, `import_render.rs`, and `import_compare.rs`.
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
