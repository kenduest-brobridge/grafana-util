# ai-changes-archive-2026-06-11

## 2026-05-25 - Shared review next-check projection
- Summary: Moved access plan action next-check line projection into the shared review contract so TUI review surfaces can reuse hint/default follow-up guidance.
- Tests: cargo test --quiet review_mutation_action_next_check_lines_project_hints_and_default_guidance; cargo test --quiet access_plan_interactive_browser; cargo test --quiet review_contract; cargo test --quiet access (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py; make quality-ai-workflow; git diff --check
- Impact: Access plan interactive reviews now consume shared review-contract next-check lines while preserving existing hint, blocker, warning, create/update/delete, and no-op guidance strings. Public CLI paths, help text, generated docs, and command contracts are unchanged.
- Rollback/Risk: Low. This moves existing string projection into the shared review contract and focused access/review-contract tests cover the old output.
- Follow-up: Continue migrating compatible review panes onto shared review-contract detail and diff projection helpers.
