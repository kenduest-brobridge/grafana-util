# ai-changes.md

Current AI change log only.

- Older detailed history moved to [`archive/ai-changes-archive-2026-03-24.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-03-24.md).
- Detailed 2026-03-27 entries moved to [`archive/ai-changes-archive-2026-03-27.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-03-27.md).
- Keep this file limited to the latest active architecture and maintenance changes.

## 2026-03-27 - Datasource secret placeholder preflight
- Summary: added `rust/src/datasource_secret.rs` for `${secret:...}` placeholder parsing and staged plan summaries, then wired `secretPlaceholderAssessment` into Rust sync bundle-preflight so missing placeholder availability becomes an explicit blocking check alongside provider and alert-artifact assessments.
- Tests: added focused datasource secret helper coverage and extended sync bundle-preflight/apply/render/promotion regressions to assert the new `secretPlaceholderBlockingCount`, staged review output, and apply rejection reason source.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all` passed; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings` passed; `cargo test --manifest-path rust/Cargo.toml --quiet sync` passed with 131 sync tests.
- Impact: `rust/src/datasource_secret.rs`, `rust/src/datasource_secret_rust_tests.rs`, `rust/src/lib.rs`, `rust/src/sync/bundle_preflight.rs`, `rust/src/sync/staged_documents_apply.rs`, `rust/src/sync/staged_documents_render.rs`, `rust/src/sync/promotion_preflight.rs`, `rust/src/sync/bundle_contract_preflight_rust_tests.rs`, `rust/src/sync/cli_apply_review_exec_apply_rust_tests.rs`, `rust/src/sync/cli_render_rust_tests.rs`, `rust/src/sync/bundle_exec_rust_tests.rs`, `rust/src/sync/promotion_preflight_rust_tests.rs`
- Rollback/Risk: this is still staged-only secret handling and does not resolve secrets; revert the new assessment if the placeholder contract or availability naming needs to change before wiring later resolution flows.

## 2026-03-27 - Sync staged/live boundary split
- Summary: split staged review/apply/preflight helper ownership out of `rust/src/sync/staged_documents.rs` into `rust/src/sync/staged_documents_apply.rs`, trimmed `rust/src/sync/staged_documents_render.rs` back to rendering and drift display, and moved live apply-intent parsing from `rust/src/sync/live_apply.rs` into `rust/src/sync/live_intent.rs`.
- Tests: existing sync CLI, staged document, and live-apply coverage were reused; no new behavior-specific tests were needed for this boundary-only refactor.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all --check` passed; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings` passed; `cargo test --manifest-path rust/Cargo.toml --quiet sync` passed with 123 sync tests.
- Impact: `rust/src/sync/cli.rs`, `rust/src/sync/live.rs`, `rust/src/sync/live_apply.rs`, `rust/src/sync/live_intent.rs`, `rust/src/sync/mod.rs`, `rust/src/sync/staged_documents.rs`, `rust/src/sync/staged_documents_apply.rs`, `rust/src/sync/staged_documents_render.rs`
- Rollback/Risk: the public sync behavior should remain stable; revert the helper splits if module visibility or staged helper reexports need to be collapsed again.
- Follow-up: none.

## 2026-03-27 - Sync explainability upgrade
- Summary: added `rust/src/sync/blocked_reasons.rs` to pull concrete blocking reasons out of staged preflight and bundle-preflight check arrays, reused it in `staged_documents_apply.rs` for apply rejection messages, and added short operator guidance lines to the sync plan/apply/bundle-preflight text renderers.
- Tests: updated focused sync render and apply regression tests to assert the new reason strings without changing CLI topology or staged JSON payload shapes.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all --check` passed; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings` passed; `cargo test --manifest-path rust/Cargo.toml --quiet sync` passed with 123 sync tests.
- Impact: `rust/src/sync/blocked_reasons.rs`, `rust/src/sync/staged_documents_apply.rs`, `rust/src/sync/staged_documents_render.rs`, `rust/src/sync/bundle_preflight.rs`, `rust/src/sync/cli_apply_review_exec_apply_rust_tests.rs`, `rust/src/sync/cli_render_rust_tests.rs`, `rust/src/sync/bundle_contract_preflight_rust_tests.rs`, `rust/src/sync/bundle_exec_rust_tests.rs`
- Rollback/Risk: the change is text-heavy and should not alter sync JSON contracts; revert the helper and focused render assertions if the extra operator guidance proves too noisy.

## 2026-03-27 - Promotion preflight skeleton
- Summary: added a first staged `sync promotion-preflight` workflow around the existing source-bundle and bundle-preflight primitives. The new document reports direct folder/datasource matches, explicit remaps from an optional mapping file, missing target mappings, and inherited bundle blockers in one reviewable contract.
- Tests: added focused promotion-preflight contract/render coverage plus CLI help/parser coverage without attempting a live promotion path yet.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all --check` passed; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings` passed; `cargo test --manifest-path rust/Cargo.toml --quiet sync` passed with 128 sync tests.
- Impact: `rust/src/sync/promotion_preflight.rs`, `rust/src/sync/cli.rs`, `rust/src/sync/mod.rs`, `rust/src/sync/promotion_preflight_rust_tests.rs`, `rust/src/sync/cli_help_rust_tests.rs`, `rust/src/sync/bundle_contract_rust_tests.rs`
- Rollback/Risk: this is intentionally a skeleton and only covers staged folder/datasource remap visibility; revert the command/module if the contract needs to be redesigned before broader promotion semantics are added.

## 2026-03-27 - Promotion mapping help example
- Summary: added a minimal `grafana-utils-sync-promotion-mapping` JSON example directly to `sync promotion-preflight --help` so the mapping file contract is discoverable from the CLI instead of only from tests and source.
- Tests: extended focused sync help coverage to assert the mapping document kind and environment metadata snippet appear in the rendered help output.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all --check` passed; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings` passed; `cargo test --manifest-path rust/Cargo.toml --quiet sync` passed with 129 sync tests.
- Impact: `rust/src/sync/mod.rs`, `rust/src/sync/cli_help_rust_tests.rs`
- Rollback/Risk: low; revert the extra help block if the long-help output becomes too noisy or if the mapping contract changes again.

## 2026-03-27 - Unified CLI help/example source split
- Summary: moved the unified root help/example blocks and help-label color table out of `rust/src/cli.rs` into a dedicated `rust/src/cli_help_examples.rs` helper so the dispatcher stays focused on rendering and routing.
- Validation: `cargo fmt --manifest-path rust/Cargo.toml --all`; `cargo test --quiet unified_help`
- Test Run: passed, with 7 unified help-focused tests.
- Impact: `rust/src/cli.rs`, `rust/src/cli_help_examples.rs`, `rust/src/lib.rs`, `rust/src/cli_rust_tests.rs`
- Rollback/Risk: the user-facing help text should stay the same; revert the helper extraction if rendered help output changes unexpectedly.

## 2026-03-27 - Dashboard dependency report human-readable output
- Summary: finished the dashboard dependency report cleanup by extracting dependency-table rendering out of `rust/src/dashboard/inspect_output.rs` into `rust/src/dashboard/inspect_dependency_render.rs`, added focused text coverage for orphan-cell normalization and dashboard dependency sections, and moved datasource family normalization into `rust/src/dashboard/inspect_family.rs` so inspect reporting no longer depends on governance internals for that shared helper.
- Validation: `cargo fmt --manifest-path rust/Cargo.toml --all --check` passed after formatting; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings` passed; `cargo test --manifest-path rust/Cargo.toml --quiet inspect_output` passed; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_inspection_dependency_contract` passed.
- Impact: `rust/src/dashboard_inspection_dependency_contract.rs`, `rust/src/dashboard/inspect_output.rs`, `rust/src/dashboard/inspect_dependency_render.rs`, `rust/src/dashboard/inspect_family.rs`, `rust/src/dashboard/inspect_governance.rs`, `rust/src/dashboard/inspect_governance_coverage.rs`, `rust/src/dashboard/inspect_query_report.rs`, `rust/src/dashboard/mod.rs`, `rust/src/lib.rs`
- Rollback/Risk: low. This is an internal ownership cleanup around already-exposed report behavior; revert the helper extraction if the inspect/governance helper split needs a different shared-module shape.

## 2026-03-27 - Current Change Summary
- Summary: archived the older detailed AI trace entries and reset the top-level AI docs to short current-only summaries.
- Validation: confirmed the new archive files exist and the current AI docs now point at both archive generations.
- Impact: `docs/internal/ai-status.md`, `docs/internal/ai-changes.md`, `docs/internal/archive/ai-status-archive-2026-03-27.md`, `docs/internal/archive/ai-changes-archive-2026-03-27.md`

## 2026-03-27 - Current Architecture Summary
- Summary: current maintainer work is centered on shrinking large Rust orchestration modules, keeping facades thin, and preserving stable CLI and JSON contracts while feature-specific test files continue to split out of umbrella suites.
- Validation: repository documentation review only.
- Impact: `docs/internal/ai-status.md`, `docs/internal/ai-changes.md`

## 2026-03-27 - Current Planned Follow-Up
- Summary: next targeted maintainer change is to continue dashboard subsystem boundary cleanup beyond the dependency report path, keep tightening crate visibility boundaries, and extend datasource secret handling from the now-wired add/modify mutation path into datasource import-side record and payload workflows before returning to narrower promotion-only refinements.
- Validation: planning note only.
- Impact: `rust/src/dashboard/`, `rust/src/datasource.rs`, `rust/src/datasource_import_export.rs`, `rust/src/lib.rs`, related dashboard and datasource tests, `docs/internal/ai-status.md`, `docs/internal/ai-changes.md`

## 2026-03-28 - Dashboard dependency boundary and datasource mutation secret wiring
- Summary: finished the current dashboard dependency-report cleanup by keeping dependency rendering in an inspect-owned helper module and moving shared datasource family normalization out of governance into `rust/src/dashboard/inspect_family.rs`. Also wired the datasource secret placeholder resolution contract into both live datasource add/modify payload builders and datasource import payloads through explicit `--secure-json-data-placeholders` and `--secret-values` JSON inputs plus import-side `secureJsonDataPlaceholders` record support.
- Tests: extended focused dashboard dependency output assertions and datasource regressions to cover dependency sections, orphan rendering normalization, placeholder resolution, fail-closed mutation input errors, import parser support, import payload resolution, and import contract acceptance of `secureJsonDataPlaceholders`.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all` passed; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings` passed; `cargo test --manifest-path rust/Cargo.toml --quiet inspect_output` passed; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_inspection_dependency_contract` passed; `cargo test --manifest-path rust/Cargo.toml --quiet datasource_rust_tests` passed; `cargo test --manifest-path rust/Cargo.toml --quiet datasource_cli_mutation_tail_rust_tests` passed; `cargo test --manifest-path rust/Cargo.toml --quiet datasource_rust_tests_tail_rust_tests` passed; `cargo test --manifest-path rust/Cargo.toml --quiet datasource_secret_rust_tests` passed.
- Impact: `rust/src/dashboard/inspect_dependency_render.rs`, `rust/src/dashboard/inspect_family.rs`, `rust/src/dashboard/inspect_output.rs`, `rust/src/dashboard/inspect_query_report.rs`, `rust/src/dashboard/inspect_governance.rs`, `rust/src/dashboard/inspect_governance_coverage.rs`, `rust/src/dashboard/mod.rs`, `rust/src/datasource.rs`, `rust/src/datasource_cli_defs.rs`, `rust/src/datasource_import_export.rs`, `rust/src/datasource_import_export_support.rs`, `rust/src/datasource_mutation_payload.rs`, `rust/src/datasource_secret.rs`, `rust/src/datasource_rust_tests.rs`, `rust/src/datasource_rust_tests_tail_rust_tests.rs`, `rust/src/datasource_cli_mutation_tail_rust_tests.rs`, `rust/src/datasource_secret_rust_tests.rs`, `rust/src/lib.rs`, `docs/internal/datasource-secret-handling-unwired.md`
- Rollback/Risk: low-to-moderate. The dashboard side is internal ownership cleanup only; the datasource side adds new explicit CLI secret-input surfaces and import-side placeholder support, but still does not extend dry-run or sync/promotion secret explainability.
