# ai-changes-archive-2026-04-17

## 2026-04-15 - Split datasource tail import and inspect tests
- Summary: split datasource tail import validation/loader tests and inspect-export/local-source tests into focused sibling modules. The parent tail test module now keeps routed datasource import identity, summary, and export-org routing behavior.
- Tests: preserved existing datasource import loader, inspect-export renderer, manifest classifier, local-source help, and routed import assertions while moving them into responsibility-based modules.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cargo test --manifest-path rust/Cargo.toml --quiet datasource_`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`; `make quality-architecture`; `git diff --check`.
- Impact: `rust/src/commands/datasource/tests/tail.rs`, `rust/src/commands/datasource/tests/tail_import.rs`, `rust/src/commands/datasource/tests/tail_inspect.rs`, and AI trace docs. README files and Python implementation were intentionally left unchanged.
- Rollback/Risk: low test-only split. Rollback would move import and inspect-export tests back into the parent datasource tail module.
- Follow-up: continue remaining architecture-warning hotspots: dashboard browse support, dashboard dependency contract, datasource staged reading, datasource CLI mutation tests, sync live apply, and help-test semantic assertions.

## 2026-04-15 - Split datasource supported catalog tests
- Summary: split supported datasource catalog output tests out of `cli_mutation.rs` into `cli_mutation_supported_catalog.rs`. The parent module remains focused on command help, parser compatibility, and add-payload behavior.
- Tests: preserved existing supported catalog assertions for JSON fixture projection, profile metadata, family defaults, text, table, csv, and yaml output.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cargo test --manifest-path rust/Cargo.toml --quiet datasource_`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`; `make quality-architecture`; `git diff --check`.
- Impact: `rust/src/commands/datasource/tests/cli_mutation.rs`, `rust/src/commands/datasource/tests/cli_mutation_supported_catalog.rs`, `rust/src/commands/datasource/tests/mod.rs`, and AI trace docs. README files and Python implementation were intentionally left unchanged.
- Rollback/Risk: low test-only split. Rollback would move supported catalog assertions back into the datasource CLI mutation test module.
- Follow-up: continue remaining architecture-warning hotspots: dashboard browse support, dashboard dependency contract, datasource staged reading, sync live apply, and help-test semantic assertions.

## 2026-04-15 - Reduce dashboard help assertions
- Summary: replaced repeated direct `help.contains()` assertions in dashboard inspect/help regressions with a shared `assert_help_includes` helper for grouped semantic help checks. The expected dashboard help fragments are unchanged.
- Tests: preserved dashboard summary, validate-export, policy, dependencies, impact, variables, and full-help coverage while reducing the direct assertion pattern that architecture guardrails flag.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_cli_inspect_help`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`; `make quality-architecture`; `git diff --check`.
- Impact: `rust/src/commands/dashboard/dashboard_cli_inspect_help_rust_tests.rs` and AI trace docs. README files and Python implementation were intentionally left unchanged.
- Rollback/Risk: low test-only refactor. Rollback would restore the prior direct `help.contains()` assertions.
- Follow-up: apply the same semantic-help assertion cleanup to `sync/cli_help_rust_tests.rs`.
