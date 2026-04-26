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

## 2026-04-26 - Bound library-panel elements to live export
- State: Done
- Scope: Rust raw-to-prompt library-panel handling, live export prompt regression, focused tests, and TODO trace. Live export/import-handoff `__elements` support remains in scope; dashboard v2 import/export support and provisioning contract changes are out of scope.
- Baseline: Raw-to-prompt could still perform live library-panel model lookup when live datasource lookup was enabled, which blurred the boundary between local conversion and live export handoff.
- Current Update: Removed raw-to-prompt live library-panel lookup, kept live datasource lookup intact, preserved warning-only local library-panel references with empty `__elements`, and retained live export `__elements` behavior.
- Result: Focused raw-to-prompt and live export regression tests pass.

## 2026-04-26 - Type dashboard ownership evidence
- State: Done
- Scope: Rust dashboard ownership evidence helpers, sync live dashboard write guard, focused ownership tests, and TODO trace. Public JSON fields, direct write policy, Python implementation, and generated docs are out of scope.
- Baseline: `DashboardTargetOwnership` existed, but sync live write guards still rebuilt `ownership=...` evidence strings locally.
- Current Update: Added typed ownership label parsing and evidence-note helpers, routed sync live dashboard ownership evidence through the dashboard target model, and covered duplicate insertion, unknown labels, and direct-write blocking behavior.
- Result: Focused dashboard ownership, dashboard plan, and sync live tests pass.

## 2026-04-26 - Add Git Sync dashboard review layout evidence
- State: Done
- Scope: Rust dashboard local source loading, dashboard plan review metadata, workspace discovery labeling, Grafana-source dashboard fixture parity, live smoke validation, and AI trace docs. Direct live writes, dashboard v2 import/export support, and Python implementation are out of scope.
- Baseline: Repo-backed Git Sync dashboard trees could be reviewed through local dashboard paths, but plan/discovery output did not explicitly label the input as Git Sync layout and v2/source parity tests still relied on scattered inline fixtures.
- Current Update: Classified dashboard review inputs as `export` or `git-sync`, carried `inputLayout` through dashboard plan output, labeled workspace discovery Git Sync dashboard inputs, and anchored datasource-variable/library-panel/v2 boundary tests to a shared checked-in fixture bundle.
- Result: Focused dashboard/discovery tests, formatter, live Rust smoke, and full Rust tests pass.
