# ai-status-archive-2026-04-20

## 2026-04-19 - Advance status and review-governance cleanup
- State: Done
- Scope: Rust alert live project-status normalization, TODO backlog cleanup, contract promotion guidance, mutation review-envelope inventory, focused tests, formatting, architecture checks, and AI trace docs. Public CLI behavior, generated docs, README files, and Python implementation are out of scope.
- Current Update: Routed the alert live status producer through the shared status reading model, removed stale completed work from the active backlog, documented runtime-vs-schema promotion rules, and captured an internal review-envelope inventory before any public JSON changes.
- Result: Focused alert/status tests, contract/schema checks, full Rust tests, clippy, formatting, architecture, and AI workflow checks pass locally.

## 2026-04-19 - Clarify contract ownership map
- State: Done
- Scope: `docs/internal/contract-doc-map.md`, contract registry routing notes, and trace docs. Runtime JSON output, schema manifests, public CLI behavior, generated docs, README files, and Python implementation are out of scope.
- Current Update: Clarified the boundary between runtime golden output contracts, CLI/docs routing contracts, docs-entrypoint navigation, and schema/help manifests so the maintainer map now names the source of truth for each layer explicitly.
- Result: The contract map now distinguishes `command-surface.json`, `docs-entrypoints.json`, `output-contracts.json`, and `schemas/manifests/` as separate ownership surfaces.

## 2026-04-18 - Fix Rust 1.95 sync review clippy failure
- State: Done
- Scope: Rust sync review TUI key handling, CI failure analysis, focused sync tests, full Rust test, clippy, formatting, architecture gate, and AI trace docs. Public CLI behavior, generated docs, README files, JSON contracts, and Python implementation are out of scope.
- Baseline: GitHub Actions `rust-quality` passed cargo tests on commit `8a6b7d6b`, then failed under Rust 1.95 clippy because nested `if diff_mode` checks inside `review_tui` key handling triggered the new `collapsible_match` lint.
- Current Update: Collapsed the nested diff-mode checks into guarded `match key.code` arms while preserving the same checklist and diff-view key behavior.
- Result: Focused sync tests, full Rust tests, local clippy, formatting, and architecture checks pass. CI must rerun on pushed commits to verify the Rust 1.95 lint gate.
