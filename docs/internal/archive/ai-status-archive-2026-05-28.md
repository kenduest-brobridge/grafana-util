# ai-status-archive-2026-05-28

## 2026-05-25 - Shared review target evidence projection
- State: Done
- Scope: rust/src/commands/review_contract.rs; rust/src/commands/access/access_plan_tui.rs; docs/internal/tui-architecture-roadmap.md
- Current Update: Moved access plan live-target evidence row projection into the shared review contract so mutation review surfaces can reuse known target field rows.
- Result: Access plan TUI keeps the same Live target: key=value rows while review_contract now owns the known target field projection for generic mutation actions. Public CLI paths, help text, generated docs, and command contracts are unchanged.
