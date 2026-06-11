# ai-changes-archive-2026-06-11

## 2026-05-25 - Shared review next-check projection
- Summary: Moved access plan action next-check line projection into the shared review contract so TUI review surfaces can reuse hint/default follow-up guidance.
- Tests: cargo test --quiet review_mutation_action_next_check_lines_project_hints_and_default_guidance; cargo test --quiet access_plan_interactive_browser; cargo test --quiet review_contract; cargo test --quiet access (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py; make quality-ai-workflow; git diff --check
- Impact: Access plan interactive reviews now consume shared review-contract next-check lines while preserving existing hint, blocker, warning, create/update/delete, and no-op guidance strings. Public CLI paths, help text, generated docs, and command contracts are unchanged.
- Rollback/Risk: Low. This moves existing string projection into the shared review contract and focused access/review-contract tests cover the old output.
- Follow-up: Continue migrating compatible review panes onto shared review-contract detail and diff projection helpers.

## 2026-05-25 - Shared review diff preview projection
- Summary: Moved access plan action diff-preview line projection into the shared review contract so mutation review surfaces can reuse safe live/desired preview rows.
- Tests: cargo test --quiet review_mutation_action_diff_preview_lines_hide_secret_like_fields; cargo test --quiet access_plan_interactive_shared_diff_preview_hides_secret_like_fields; cargo test --quiet access_plan_interactive_browser; cargo test --quiet review_contract; cargo test --quiet access (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py; make quality-ai-workflow; git diff --check
- Impact: Access plan TUI keeps the same shared diff preview output and secret-like field filtering while the reusable review contract now owns the generic action changes to ReviewDiffModel preview projection. Public CLI paths, help text, generated docs, and command contracts are unchanged.
- Rollback/Risk: Low. This moves equivalent projection code into review_contract and focused access/review-contract tests cover safe field filtering plus existing TUI output.
- Follow-up: Continue moving compatible mutation review detail rows out of per-surface TUI renderers and into shared review projections.

## 2026-05-25 - Shared review change-detail projection
- Summary: Moved access plan action change-detail row projection into the shared review contract so mutation review surfaces can reuse safe Change: field bundle/live rows.
- Tests: cargo test --quiet review_mutation_action_change_detail_lines_hide_secret_like_fields; cargo test --quiet access_plan_interactive_browser; cargo test --quiet access_plan_interactive_shared_diff_preview_hides_secret_like_fields; cargo test --quiet review_contract; cargo test --quiet access (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py; make quality-ai-workflow; git diff --check
- Impact: Access plan TUI keeps the same Change: field bundle/live rows while the reusable review contract now owns safe changed-field filtering and compact value formatting for generic mutation action changes. Public CLI paths, help text, generated docs, and command contracts are unchanged.
- Rollback/Risk: Low. This moves equivalent projection code into review_contract and focused access/review-contract tests cover safe field filtering plus existing TUI output.
- Follow-up: Continue moving compatible mutation review target/context rows out of per-surface TUI renderers and into shared review projections.

## 2026-05-25 - Shared review target evidence projection
- Summary: Moved access plan live-target evidence row projection into the shared review contract so mutation review surfaces can reuse known target field rows.
- Tests: cargo test --quiet review_mutation_action_target_evidence_lines_project_known_live_target_fields; cargo test --quiet access_plan_interactive_browser; cargo test --quiet review_contract; cargo test --quiet access (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py; make quality-ai-workflow; git diff --check
- Impact: Access plan TUI keeps the same Live target: key=value rows while review_contract now owns the known target field projection for generic mutation actions. Public CLI paths, help text, generated docs, and command contracts are unchanged.
- Rollback/Risk: Low. This moves equivalent target projection code into review_contract and focused access/review-contract tests cover the old output.
- Follow-up: Continue moving compatible warning/blocker context rows out of per-surface TUI renderers and into shared review projections.
