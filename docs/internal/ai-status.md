# ai-status.md

Current AI-maintained status only.

- Older trace history moved to [`archive/ai-status-archive-2026-03-24.md`](docs/internal/archive/ai-status-archive-2026-03-24.md).
- Detailed 2026-03-27 entries moved to [`archive/ai-status-archive-2026-03-27.md`](docs/internal/archive/ai-status-archive-2026-03-27.md).
- Detailed 2026-03-28 task notes were condensed into [`archive/ai-status-archive-2026-03-28.md`](docs/internal/archive/ai-status-archive-2026-03-28.md).
- Detailed 2026-03-29 through 2026-03-31 entries moved to [`archive/ai-status-archive-2026-03-31.md`](docs/internal/archive/ai-status-archive-2026-03-31.md).
- Detailed 2026-04-01 through 2026-04-12 entries moved to [`archive/ai-status-archive-2026-04-12.md`](docs/internal/archive/ai-status-archive-2026-04-12.md).
- Keep this file short and current. Additive historical detail belongs in `docs/internal/archive/`.
- Older entries moved to [`ai-status-archive-2026-04-13.md`](docs/internal/archive/ai-status-archive-2026-04-13.md).
- Older entries moved to [`ai-status-archive-2026-04-14.md`](docs/internal/archive/ai-status-archive-2026-04-14.md).
- Older entries moved to [`ai-status-archive-2026-04-15.md`](docs/internal/archive/ai-status-archive-2026-04-15.md).
- Older entries moved to [`ai-status-archive-2026-04-16.md`](docs/internal/archive/ai-status-archive-2026-04-16.md).
- Older entries moved to [`ai-status-archive-2026-04-17.md`](docs/internal/archive/ai-status-archive-2026-04-17.md).
- Older entries moved to [`ai-status-archive-2026-04-18.md`](docs/internal/archive/ai-status-archive-2026-04-18.md).
- Older entries moved to [`ai-status-archive-2026-04-19.md`](docs/internal/archive/ai-status-archive-2026-04-19.md).
- Older entries moved to [`ai-status-archive-2026-04-20.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-status-archive-2026-04-20.md).
- Older entries moved to [`ai-status-archive-2026-04-26.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-status-archive-2026-04-26.md).
- Older entries moved to [`ai-status-archive-2026-04-27.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-status-archive-2026-04-27.md).

## 2026-04-26 - Prove provisioning remains derived dashboard projection
- State: Done
- Scope: Rust dashboard compare/import regression tests, sync bundle dashboard source guard, focused tests, and TODO trace. Public JSON, generated docs, and dashboard v2 support are out of scope.
- Baseline: Provisioning was already a file-backed export lane, but TODO still needed regression evidence that it is not the canonical dashboard contract.
- Current Update: Added compare tests proving raw export wrappers and provisioning projections normalize to the same classic dashboard payload, and added a sync bundle guard that rejects explicit dual dashboard raw/provisioning inputs.
- Result: Focused compare, source-loader, sync bundle, and import dry-run tests pass.

## 2026-04-27 - Refresh dashboard directory re-layering inventory
- State: Done
- Scope: Maintainer-only dashboard directory inventory, hotspot summary, future move candidates, validation guidance, and TODO trace. Code moves, public CLI/docs, generated docs, and runtime behavior are out of scope.
- Baseline: The dashboard directory re-layering TODO required a fresh inventory before any later move, and recent ownership/Git Sync/permission boundary work changed the best next candidates.
- Current Update: Added `docs/internal/dashboard-directory-relayering-inventory.md` with mixed-responsibility files, stable boundaries, and candidate future moves for prompt-lane transform, export-org source discovery, and status live collector namespace cleanup.
- Result: The inventory checkpoint is complete; actual `git mv` work remains gated behind one-boundary-per-commit guardrails.

## 2026-04-27 - Move dashboard prompt transform boundary
- State: Done
- Scope: Rust dashboard prompt transform module layout, facade re-exports, focused prompt/export tests, full Rust validation, and TODO trace. Public CLI/docs, generated docs, Python implementation, and behavior changes are out of scope.
- Baseline: The dashboard re-layering inventory identified root-level `prompt*.rs` files as a shared prompt-lane transform boundary used by live export and offline raw-to-prompt.
- Current Update: Moved the prompt transform and helper files under `rust/src/commands/dashboard/export_prompt/`, kept `commands/dashboard/mod.rs` as the public facade, and rewired direct consumers plus test support to the new module.
- Result: Focused raw-to-prompt, export prompt, inventory, library-panel, and export-diff tests pass; full Rust validation is run for the commit.

## 2026-04-27 - Guard dashboard permissions as adjacent evidence
- State: Done
- Scope: Rust dashboard permission-artifact rejection, dashboard/raw-to-prompt/review regressions, sync/access workspace boundary tests, and TODO trace. Permission restore/apply behavior, public JSON changes, generated docs, and Python implementation are out of scope.
- Baseline: Directory-based dashboard flows skipped `permissions.json`, but single-object dashboard flows could still treat dashboard permission artifacts as dashboard JSON.
- Current Update: Rejected dashboard permission bundle/export artifacts in the shared dashboard object extractor, wired the inventory regression module into the Rust suite, and added sync/access tests proving permission bundles stay out of dashboard source and access-bundle collection.
- Result: Focused dashboard, raw-to-prompt, sync bundle, and access plan tests pass.

## 2026-04-27 - Guard Git Sync dashboard live apply boundaries
- State: Done
- Scope: Rust dashboard browse local-mode routing, sync apply-intent/live-apply regressions, focused tests, and TODO trace. Public JSON, generated docs, Python implementation, and Git repository/PR automation are out of scope.
- Baseline: Sync live apply already blocked file-provisioned and Git Sync-owned dashboards, but workspace-backed dashboard browse trees did not share the same local-mode detection as explicit `--input-dir` local browse trees.
- Current Update: Centralized dashboard browse local-source detection so `--workspace` Git Sync review trees use read-only local mode, and added sync regressions proving Git Sync dashboard ownership survives apply-intent handoff and blocks live transport.
- Result: Focused browse, sync apply-intent, live-apply, and reusable-output tests pass.

## 2026-04-26 - Add dashboard v2 adapter boundary regressions
- State: Done
- Scope: Rust dashboard diff/import source-wrapper regression tests, focused validation, and TODO trace. Public JSON, generated docs, classic dashboard behavior, and actual v2 adapter support are out of scope.
- Baseline: Classic raw/provisioning import and plan lanes already rejected dashboard v2 resources, but adapter-facing diff and root-export normalization paths still lacked dedicated regression coverage.
- Current Update: Added diff-lane tests proving raw and provisioning compare entrypoints reject dashboard v2 input before any remote compare request runs, and added import source-wrapper tests proving root export normalization into temp raw/provisioning variants still rejects v2 payloads.
- Result: Focused export-diff and import-loaded-source tests pass, and the remaining v2 adapter-boundary TODO is now satisfied.
