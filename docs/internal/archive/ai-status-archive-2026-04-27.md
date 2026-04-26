# ai-status-archive-2026-04-27

## 2026-04-20 - Add dashboard folder permission drift review
- State: Done
- Scope: Rust dashboard plan CLI, permission bundle loading, folder permission drift actions/rendering, focused tests, and AI trace docs. Dashboard import permission restore, dashboard-level ACL replay, generated docs, Python implementation, and access subject lifecycle are out of scope.
- Baseline: Dashboard export writes `raw/permissions.json`, but dashboard plan/import treat it as metadata and cannot compare exported folder ACLs against live Grafana.
- Current Update: Added `dashboard plan --include-folder-permissions`, UID-first folder permission comparison, optional path fallback, permission detail rendering, command docs, and regression coverage.
- Result: Focused dashboard plan/parser tests, docs surface, formatter, and full Rust tests pass.

## 2026-04-26 - Add Git Sync dashboard review layout evidence
- State: Done
- Scope: Rust dashboard local source loading, dashboard plan review metadata, workspace discovery labeling, Grafana-source dashboard fixture parity, live smoke validation, and AI trace docs. Direct live writes, dashboard v2 import/export support, and Python implementation are out of scope.
- Baseline: Repo-backed Git Sync dashboard trees could be reviewed through local dashboard paths, but plan/discovery output did not explicitly label the input as Git Sync layout and v2/source parity tests still relied on scattered inline fixtures.
- Current Update: Classified dashboard review inputs as `export` or `git-sync`, carried `inputLayout` through dashboard plan output, labeled workspace discovery Git Sync dashboard inputs, and anchored datasource-variable/library-panel/v2 boundary tests to a shared checked-in fixture bundle.
- Result: Focused dashboard/discovery tests, formatter, live Rust smoke, and full Rust tests pass.

## 2026-04-26 - Type dashboard ownership evidence
- State: Done
- Scope: Rust dashboard ownership evidence helpers, sync live dashboard write guard, focused ownership tests, and TODO trace. Public JSON fields, direct write policy, Python implementation, and generated docs are out of scope.
- Baseline: `DashboardTargetOwnership` existed, but sync live write guards still rebuilt `ownership=...` evidence strings locally.
- Current Update: Added typed ownership label parsing and evidence-note helpers, routed sync live dashboard ownership evidence through the dashboard target model, and covered duplicate insertion, unknown labels, and direct-write blocking behavior.
- Result: Focused dashboard ownership, dashboard plan, and sync live tests pass.

## 2026-04-26 - Bound library-panel elements to live export
- State: Done
- Scope: Rust raw-to-prompt library-panel handling, live export prompt regression, focused tests, and TODO trace. Live export/import-handoff `__elements` support remains in scope; dashboard v2 import/export support and provisioning contract changes are out of scope.
- Baseline: Raw-to-prompt could still perform live library-panel model lookup when live datasource lookup was enabled, which blurred the boundary between local conversion and live export handoff.
- Current Update: Removed raw-to-prompt live library-panel lookup, kept live datasource lookup intact, preserved warning-only local library-panel references with empty `__elements`, and retained live export `__elements` behavior.
- Result: Focused raw-to-prompt and live export regression tests pass.
