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

## 2026-04-20 - Add dashboard folder permission drift review
- State: Done
- Scope: Rust dashboard plan CLI, permission bundle loading, folder permission drift actions/rendering, focused tests, and AI trace docs. Dashboard import permission restore, dashboard-level ACL replay, generated docs, Python implementation, and access subject lifecycle are out of scope.
- Baseline: Dashboard export writes `raw/permissions.json`, but dashboard plan/import treat it as metadata and cannot compare exported folder ACLs against live Grafana.
- Current Update: Added `dashboard plan --include-folder-permissions`, UID-first folder permission comparison, optional path fallback, permission detail rendering, command docs, and regression coverage.
- Result: Focused dashboard plan/parser tests, docs surface, formatter, and full Rust tests pass.

## 2026-04-20 - Add contract promotion report
- State: Done
- Scope: Contract promotion report behavior, unit coverage, maintainer docs, TODO tracking, and AI trace docs. Public CLI behavior, generated docs, schema artifacts, Rust implementation, and Python package behavior are out of scope.
- Baseline: Contract ownership lanes were documented, but the promotion report still needed concrete matrix behavior for actual route shapes, runtime-only rows, informational findings, and test coverage.
- Current Update: Expanded the report to read manifest route sections and quick lookups, normalize command evidence, show runtime-only rows and categorized informational findings, and documented how maintainers read the evidence matrix.
- Result: Contract promotion report, report unit tests, output-contract checks, schema check, AI workflow, and diff checks pass.

## 2026-04-20 - Finish project status producer audit
- State: Done
- Scope: Rust project-status producer audit across sync, datasource, alert, dashboard, access, and live status fallback producers. Public CLI behavior, generated docs, Python implementation, and output contracts are out of scope.
- Baseline: Most domain producers already routed through `StatusReading`, but the remaining audit still needed to confirm whether any direct producer construction remained.
- Current Update: Audited the producer lanes in workers and normalized the remaining dashboard live read-failure fallback in test support through `StatusReading` while preserving live evidence and public status fields.
- Result: Focused dashboard/access/status/project_status tests, formatter, architecture, AI workflow, and full Rust tests pass.

## 2026-04-20 - Move dashboard import into directory boundary
- State: Done
- Scope: Rust dashboard import module layout, focused import/routed/plan/browse validation, and AI trace docs. Plan reconciliation, inspect/governance moves, public CLI behavior, generated docs, Python implementation, and output contracts are out of scope.
- Baseline: dashboard import implementation lived as many flat `commands/dashboard/import_*.rs` files, while plan-owned reconciliation already lived under `commands/dashboard/plan/`.
- Current Update: Moved the dashboard import implementation under `commands/dashboard/import/`, kept `commands/dashboard/mod.rs` as the public facade, and left plan reconciliation in the plan-owned boundary.
- Result: Focused import/routed/plan/browse tests, formatter, maintainability, AI workflow, and full Rust tests pass.
