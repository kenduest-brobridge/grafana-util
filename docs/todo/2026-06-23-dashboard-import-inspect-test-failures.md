# Dashboard Import And Inspect Test Failures

Date: 2026-06-23

## Problem

`make test` reports failures in dashboard import dry-run routing and inspect export
summary/input handling tests. The visible failing tests are clustered around:

- routed import dry-run JSON reporting
- routed import create-missing-org dry-run/live alignment
- inspect export summary rendering
- inspect export input-type validation for dashboard roots

## Scope

- Primary code path: `rust/src/commands/dashboard/`.
- Test surface: focused failing Rust tests under dashboard import and inspect.
- Keep unrelated HTTP dotted-host changes untouched.
- Do not change generated docs or command contracts unless failure output proves
  the public behavior changed.

## Investigation Plan

1. Re-run the pasted failing tests without `rtk` and capture full assertion text.
2. Identify the shared recent change or data-flow break behind the failure cluster.
3. Add or adjust the narrowest regression coverage only if an expected behavior is
   not already covered by the failing tests.
4. Make the smallest code change that restores the expected contracts.
5. Re-run focused tests first, then raw `make test`.

## Acceptance Checks

- Each pasted failing test passes when run directly.
- Raw `make test` passes.
- Existing unrelated worktree changes remain intact.

## Verification

- `cargo test dashboard::import_rust_tests::import_routed_reporting_rust_tests -- --nocapture`
  passed on 2026-06-23.
- `cargo test dashboard::import_rust_tests::import_routed_scope_rust_tests::import_routed_scope_auth_rust_tests::routed_import_create_missing_orgs_dry_run_and_live_created_scope_stay_aligned -- --nocapture`
  passed on 2026-06-23.
- `cargo test dashboard::inspect::inspect_output::summary::tests::render_export_inspection_summary_output_honors_csv_and_yaml_modes -- --nocapture`
  passed on 2026-06-23.
- `cargo test dashboard::inspect_export_rust_tests::analyze_export_dir_requires_input_type_for_dashboard_root_with_raw_and_prompt_variants -- --nocapture`
  passed on 2026-06-23.
- `cargo test dashboard::import_rust_tests -- --format terse` passed on
  2026-06-23.
- `cargo test dashboard::inspect -- --format terse` passed on 2026-06-23.
- `cargo test dashboard -- --format terse` passed on 2026-06-23.
- `make test` passed on 2026-06-23: Rust lib reported `1822 passed; 0
  failed; 1 ignored`, integration tests reported `7 passed` and `30 passed`,
  and doctests reported `0 failed`.

## Outcome

The pasted dashboard import and inspect failures did not reproduce after raw,
non-RTK focused and full-suite runs. No dashboard production code was changed
for this failure cluster.
