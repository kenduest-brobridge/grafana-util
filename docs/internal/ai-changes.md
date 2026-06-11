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
- Older entries moved to [`ai-changes-archive-2026-04-28.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-04-28.md).
- Older entries moved to [`ai-changes-archive-2026-05-02.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-05-02.md).
- Older entries moved to [`ai-changes-archive-2026-05-14.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-05-14.md).
- Older entries moved to [`ai-changes-archive-2026-05-16.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-05-16.md).
- Older entries moved to [`ai-changes-archive-2026-05-25.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-05-25.md).
- Older entries moved to [`ai-changes-archive-2026-05-28.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-05-28.md).
- Older entries moved to [`ai-changes-archive-2026-06-11.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-06-11.md).

## 2026-06-11 - Datasource browse support review split
- Summary: Moved datasource browse review-evidence projection and safe changed-field filtering into a dedicated support review module while keeping support.rs focused on browse documents, item rows, and detail facts.
- Tests: cargo test --quiet datasource_browse --features tui; make quality-rust; make quality-architecture; cargo check --quiet --no-default-features --all-targets; python3 scripts/tui_inventory_report.py --json; make quality-ai-workflow
- Impact: `datasource/browse/support.rs` is now below the architecture warning threshold, reducing the warning list from three files to two without changing public CLI paths, help text, generated docs, or interactive behavior.
- Rollback/Risk: Low; this is a structural move of existing review projection code covered by focused datasource browse detail/review tests and downstream datasource inspect/snapshot re-export checks.
- Follow-up: Continue reducing the remaining shared warning-threshold files: review_contract and shared browser session.

## 2026-06-11 - Datasource browse TUI chrome split
- Summary: Moved datasource browse footer controls and search prompt rendering into a dedicated TUI chrome module while keeping the main renderer focused on frame, list, and detail layout.
- Tests: cargo test --quiet datasource_browse --features tui; make quality-rust; make quality-architecture; cargo check --quiet --no-default-features --all-targets; python3 scripts/tui_inventory_report.py --json; make quality-ai-workflow
- Impact: `datasource/browse/render.rs` is now below the architecture warning threshold, reducing the warning list from four files to three without changing public CLI paths, help text, generated docs, or interactive behavior.
- Rollback/Risk: Low; this is a structural move of existing footer/search rendering covered by focused datasource browse render tests.
- Follow-up: Continue reducing the remaining TUI/shared warning-threshold files: review_contract and shared browser session.

## 2026-06-11 - Status overview TUI support split
- Summary: Split status overview TUI runtime and focused tests out of the state/search module while preserving existing interactive behavior.
- Tests: cargo test --quiet overview_tui --features tui; make quality-rust; make quality-architecture; cargo check --quiet --no-default-features --all-targets; python3 scripts/tui_inventory_report.py --json; make quality-ai-workflow
- Impact: `status/overview/tui.rs` is now under the architecture warning threshold, leaving status overview with the same facade/runtime/tests shape as the newer status TUI module. Public CLI paths, help text, generated docs, and command contracts are unchanged.
- Rollback/Risk: Low; this is a structural split of existing code with focused TUI tests covering search, Project Home handoff, initial item focus, and render output.
- Follow-up: Continue reducing the remaining TUI/shared warning-threshold files: datasource browse render/support and shared browser session.

## 2026-05-28 - Datasource interactive review output
- Summary: Added shared read-only mutation review browser projection and wired datasource plan/import dry-run interactive output.
- Tests: cargo test --quiet datasource_plan_review_envelope_builds_user_friendly_browser_items; cargo test --quiet datasource_plan_parser_accepts_interactive_output; cargo test --quiet datasource_import_dry_run_parser_accepts_interactive_output; cargo test --quiet datasource_import_rejects_use_export_org_interactive_review_output; cargo test --quiet datasource_plan; cargo test --quiet datasource_import_dry_run_review; cargo test --quiet cli_mutation; cargo test --quiet datasource (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; make quality-docs-surface; make man-check; make html-check; make quality-ai-workflow; git diff --check
- Impact: Operators can inspect datasource plan and import dry-run actions interactively with blockers, warnings, review hints, and safe diffs while public JSON remains unchanged.
- Rollback/Risk: Low to medium; public help/docs gain an interactive output value, runtime path is read-only and uses existing review envelopes.
- Follow-up: Consider moving access plan TUI onto the shared review browser builder after comparing existing access-specific summary/resource rows.

## 2026-05-25 - TUI empty selection key handling
- Summary: Kept datasource/access browse edit and delete keys inside the TUI when no row is selected, surfacing status messages instead of propagating selected-row errors.
- Tests: cargo test --quiet edit_key_on_empty_document_stays_in_browser; cargo test --quiet delete_key_on_empty_document_stays_in_browser; cargo test --quiet empty_user_browse_edit_and_delete_keys_stay_in_browser; cargo test --quiet empty_team_browse_edit_and_delete_keys_stay_in_browser; cargo test --quiet datasource_browse_input; cargo test --quiet user_browse_input; cargo test --quiet team_browse_input; cargo test --quiet datasource_browse; cargo test --quiet access::user_browse; cargo test --quiet access::team_browse; cargo test --quiet browse; cargo fmt --check; git diff --check
- Impact: Datasource browse, access user browse, and access team browse now treat empty edit/delete key presses as in-browser no-selection states.
- Rollback/Risk: Low; changes are limited to empty-selection branches and regression tests cover the key paths.
- Follow-up: none

## 2026-05-25 - Status overview starts on items
- Summary: Changed status overview interactive mode to start with the Items pane focused so Up/Down moves rows immediately after launch instead of requiring Tab first.
- Tests: cargo test --quiet overview_tui_starts_on_items_so_arrow_keys_move_rows_immediately; cargo test --quiet project_home_is_available_and_hands_off_to_first_blocked_section; cargo test --quiet interactive_render_starts_on_project_home_surface; cargo test --quiet status_overview; cargo test --quiet status_tui; RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; git diff --check
- Impact: Operators entering status overview interactive mode can navigate the item list with arrow keys immediately. Project Home remains available via h and its handoff behavior is preserved. Public CLI paths, help text, generated docs, and command contracts are unchanged.
- Rollback/Risk: Low. The change only adjusts initial TUI focus and focused tests cover immediate row movement plus Project Home access.
- Follow-up: If operators prefer a visual home summary on launch, consider a visible non-focused home panel while keeping keyboard focus on Items.

## 2026-05-25 - TUI completion audit
- Summary: Replaced the open-ended TUI follow-up section with a completion audit that maps current evidence to the finished shared review/detail/diff projection work and records why domain-specific input loops remain local.
- Tests: cargo test --quiet; cargo test --quiet user_browse; cargo test --quiet team_browse; cargo test --quiet datasource_browse; cargo test --quiet status_tui; cargo test --quiet review_contract; cargo test --quiet access (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py --json; make quality-ai-workflow; git diff --check
- Impact: The roadmap now has an evidence-backed completion audit instead of stale continue-follow-up items. Public CLI paths, help text, generated docs, Rust runtime behavior, and command contracts are unchanged.
- Rollback/Risk: Low. Documentation-only audit update based on current inventory/search/test evidence.
- Follow-up: Treat future TUI work as new scoped feature work unless a fresh inventory or user report identifies a concrete regression or duplication.

## 2026-05-25 - Shared review narrative and impact projection
- Summary: Moved access plan narrative and impact row projection into the shared review contract so mutation review surfaces can reuse action/status/changed-field guidance text.
- Tests: cargo test --quiet review_mutation_action_narrative_and_impact_lines_project_action_guidance; cargo test --quiet access_plan_interactive_browser; cargo test --quiet review_contract; cargo test --quiet access (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py; make quality-ai-workflow; git diff --check
- Impact: Access plan TUI keeps the same Narrative and Why this matters rows while review_contract now owns the generic action narrative and changed-field impact projection. Public CLI paths, help text, generated docs, and command contracts are unchanged.
- Rollback/Risk: Low. This moves equivalent projection code into review_contract, removes the now-unused access review action alias, and focused access/review-contract tests cover the old output.
- Follow-up: Continue auditing remaining compatible local artifact browser review/detail projections before declaring the broader TUI design work complete.

## 2026-05-25 - Shared review context projection
- Summary: Moved access plan warning and blocker context row projection into the shared review contract so mutation review surfaces can reuse blocked reasons, safe warning changed fields, and blocked target flag evidence.
- Tests: cargo test --quiet review_mutation_action_context_lines_project_warning_and_blocker_evidence; cargo test --quiet access_plan_interactive_browser; cargo test --quiet review_contract; cargo test --quiet access (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py; make quality-ai-workflow; git diff --check
- Impact: Access plan TUI keeps the same Blocked context, Warning context, and Blocked evidence rows while review_contract now owns the generic warning/blocker context projection for mutation actions. Public CLI paths, help text, generated docs, and command contracts are unchanged.
- Rollback/Risk: Low. This moves equivalent context projection code into review_contract and focused access/review-contract tests cover the old output.
- Follow-up: Continue auditing the remaining access-specific narrative and impact rows before deciding whether they belong in shared review projections.

## 2026-05-25 - Shared review target evidence projection
- Summary: Moved access plan live-target evidence row projection into the shared review contract so mutation review surfaces can reuse known target field rows.
- Tests: cargo test --quiet review_mutation_action_target_evidence_lines_project_known_live_target_fields; cargo test --quiet access_plan_interactive_browser; cargo test --quiet review_contract; cargo test --quiet access (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py; make quality-ai-workflow; git diff --check
- Impact: Access plan TUI keeps the same Live target: key=value rows while review_contract now owns the known target field projection for generic mutation actions. Public CLI paths, help text, generated docs, and command contracts are unchanged.
- Rollback/Risk: Low. This moves equivalent target projection code into review_contract and focused access/review-contract tests cover the old output.
- Follow-up: Continue moving compatible warning/blocker context rows out of per-surface TUI renderers and into shared review projections.
