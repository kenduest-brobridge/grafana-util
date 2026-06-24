# Dashboard Export History Performance

Date: 2026-06-24

## Problem

`grafana-util dashboard export --include-history` is slow on live Grafana exports.
The current Rust path fetches each dashboard payload in the main export loop, then
the history builder fetches the same current dashboard again before listing and
downloading history versions. History version bodies are then downloaded
sequentially.

## Scope

- Primary code paths:
  - `rust/src/commands/dashboard/export_scope.rs`
  - `rust/src/commands/dashboard/history_live.rs`
  - `rust/src/commands/dashboard/export.rs`
- Tests:
  - `rust/src/commands/dashboard/history_cli_rust_tests.rs`
  - `rust/src/commands/dashboard/export_focus_report_path_top_rust_tests.rs`
- Keep the history artifact JSON schema unchanged.
- Keep the existing `--include-history` flag behavior and default history limit
  unchanged.
- Do not add an async runtime or change the shared HTTP client abstraction.

## Design

1. Reuse the current dashboard payload already fetched by `dashboard export` when
   building the history artifact.
2. Add a bounded parallel history-version fetch helper for client-backed export
   paths. Use the existing Rayon dependency and the same bounded-worker style
   already used by live inspect and permission reads.
3. Preserve deterministic artifact ordering by collecting version fetch results
   in the order returned by Grafana's versions list.
4. Keep request-closure test paths serial unless a thread-safe fetcher is
   supplied.

## Acceptance Checks

- A focused test proves `dashboard export --include-history` does not re-fetch
  `/api/dashboards/uid/<uid>` after the main export fetch.
- A focused test proves parallel version fetches overlap under a controlled
  slow fetcher and preserve version order.
- Existing history export tests still pass.
- `cd rust && rtk cargo test --quiet history_cli_rust_tests` passes.
- `cd rust && rtk cargo test --quiet export_focus_report_path_top_rust_tests` passes.

## Execution Plan

1. Add red tests for current payload reuse and bounded parallel version fetching.
2. Implement a current-payload-aware history document builder.
3. Add a bounded Rayon helper for thread-safe version fetchers.
4. Route client-backed dashboard export history through the parallel helper.
5. Run focused tests, then broader dashboard export/history tests if needed.

## Verification

- Red check:
  - `cd rust && rtk cargo test --quiet dashboard_history_export_fetches_version_payloads_in_parallel_and_preserves_order`
    failed before implementation because the current-payload/version-fetcher
    helper did not exist.
- Green checks:
  - `cd rust && rtk cargo test --quiet export_dashboards_with_request_include_history_writes_scope_history_artifacts`
    passed.
  - `cd rust && rtk cargo test --quiet dashboard_history_export_fetches_version_payloads_in_parallel_and_preserves_order`
    passed.
  - `cd rust && rtk cargo test --quiet history_cli_rust_tests` passed: 11
    tests passed.
  - `cd rust && rtk cargo test --quiet export_focus_report_path_top_rust_tests`
    passed: 8 tests passed.
  - `cd rust && rtk cargo test --quiet dashboard::history` passed: 11 tests
    passed.
  - `cd rust && rtk cargo test --quiet dashboard::export` passed: 8 tests
    passed.
  - `cd rust && rtk cargo test --quiet dashboard` passed: 749 tests passed.
- Review follow-up on 2026-06-25 replaced the wall-clock parallelism assertion
  with an atomic max-active-fetch assertion to avoid CI timing flake.
  Verification after the review fix:
  - `cd rust && rtk cargo test --quiet dashboard_history_export_fetches_version_payloads_in_parallel_and_preserves_order`
    passed.
  - `cd rust && rtk cargo test --quiet history_cli_rust_tests` passed: 11
    tests passed.
  - `cd rust && rtk cargo test --quiet export_focus_report_path_top_rust_tests`
    passed: 8 tests passed.
  - `cd rust && rtk cargo test --quiet dashboard` passed: 749 tests passed.
  - `rtk git diff --check` passed.
