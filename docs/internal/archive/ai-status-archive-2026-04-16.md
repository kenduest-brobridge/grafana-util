# ai-status-archive-2026-04-16

## 2026-04-15 - Reduce dashboard help assertions
- State: Done
- Scope: Rust dashboard help-test maintainability. README files and Python implementation are out of scope.
- Baseline: `make quality-architecture` warned that `dashboard_cli_inspect_help_rust_tests.rs` used many direct `help.contains()` assertions.
- Current Update: Added a small `assert_help_includes` helper and routed grouped dashboard help assertions through it while preserving the same expected help text coverage.
- Result: Dashboard help focused tests, full Rust tests, clippy, architecture guardrails, formatting, and whitespace checks pass. The dashboard help-test warning is cleared.

## 2026-04-15 - Clear remaining Rust architecture warnings
- State: Done
- Scope: Rust maintainability cleanup for sync help assertions plus large dashboard dependency, sync live-apply, datasource staged-reading, and dashboard browse-support modules. README files, Python implementation, and dashboard summary/analyze public naming are out of scope.
- Baseline: `make quality-architecture` reports five warnings: `sync/cli_help_rust_tests.rs` direct help assertions plus four production files over the 900-line warning threshold.
- Current Update: Added grouped sync help assertions and split dependency contract tests, sync request-json live-apply shim, datasource staged-reading tests, and dashboard local browse tests into focused sibling modules.
- Result: Focused tests, full Rust tests, clippy, formatting, and architecture guardrails pass. `make quality-architecture` now reports no warnings. Dashboard summary/analyze naming cleanup remains deferred.
