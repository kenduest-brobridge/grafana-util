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
