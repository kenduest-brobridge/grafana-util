# ai-status-archive-2026-06-11

## 2026-05-25 - Shared review context projection
- State: Done
- Scope: rust/src/commands/review_contract.rs; rust/src/commands/access/access_plan_tui.rs; docs/internal/tui-architecture-roadmap.md
- Current Update: Moved access plan warning and blocker context row projection into the shared review contract so mutation review surfaces can reuse blocked reasons, safe warning changed fields, and blocked target flag evidence.
- Result: Access plan TUI keeps the same Blocked context, Warning context, and Blocked evidence rows while review_contract now owns the generic warning/blocker context projection for mutation actions. Public CLI paths, help text, generated docs, and command contracts are unchanged.

## 2026-05-25 - Shared review narrative and impact projection
- State: Done
- Scope: rust/src/commands/review_contract.rs; rust/src/commands/access/access_plan_tui.rs; rust/src/commands/access/access_plan_types.rs; docs/internal/tui-architecture-roadmap.md
- Current Update: Moved access plan narrative and impact row projection into the shared review contract so mutation review surfaces can reuse action/status/changed-field guidance text.
- Result: Access plan TUI keeps the same Narrative and Why this matters rows while review_contract now owns the generic action narrative and changed-field impact projection. Public CLI paths, help text, generated docs, and command contracts are unchanged.

## 2026-05-25 - TUI completion audit
- State: Done
- Scope: docs/internal/tui-architecture-roadmap.md
- Current Update: Replaced the open-ended TUI follow-up section with a completion audit that maps current evidence to the finished shared review/detail/diff projection work and records why domain-specific input loops remain local.
- Result: The roadmap now has an evidence-backed completion audit instead of stale continue-follow-up items. Public CLI paths, help text, generated docs, Rust runtime behavior, and command contracts are unchanged.

## 2026-05-25 - Status overview starts on items
- State: Done
- Scope: rust/src/commands/status/overview/tui.rs
- Current Update: Changed status overview interactive mode to start with the Items pane focused so Up/Down moves rows immediately after launch instead of requiring Tab first.
- Result: Operators entering status overview interactive mode can navigate the item list with arrow keys immediately. Project Home remains available via h and its handoff behavior is preserved. Public CLI paths, help text, generated docs, and command contracts are unchanged.

## 2026-05-25 - TUI empty selection key handling
- State: Done
- Scope: rust/src/commands/datasource/browse/input.rs; rust/src/commands/access/user_browse_dispatch.rs; rust/src/commands/access/user_browse_input.rs; rust/src/commands/access/team_browse_dispatch.rs; rust/src/commands/access/team_browse_input_tests.rs
- Current Update: Kept datasource/access browse edit and delete keys inside the TUI when no row is selected, surfacing status messages instead of propagating selected-row errors.
- Result: Datasource browse, access user browse, and access team browse now treat empty edit/delete key presses as in-browser no-selection states.
