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

## 2026-04-20 - Clean up alert runtime schema keys
- State: Done
- Scope: Rust alert runtime plan, delete-preview, and import dry-run schema-key cleanup, focused sync/alert/import validation, and AI trace docs. Alert diff shared document internals, public CLI behavior, generated docs, Python implementation, and output contracts are out of scope.
- Baseline: `commands/alert/runtime_support.rs` still repeated tool-owned plan row, plan document, delete-preview, and import dry-run keys directly in production render/read paths.
- Current Update: Grouped alert runtime document, row, and summary keys under local schema namespaces while keeping Grafana raw alert payload fields direct.
- Result: Focused sync, alert, runtime, dashboard import, formatter, maintainability, AI workflow, and full Rust tests pass.

## 2026-04-20 - Clean up dashboard import dependency schema keys
- State: Done
- Scope: Rust dashboard import dependency preflight schema-key cleanup, focused import/preflight/dashboard-plan validation, and AI trace docs. Alert runtime schema cleanup, import directory moves, public CLI behavior, generated docs, Python implementation, and output contracts are out of scope.
- Baseline: `commands/dashboard/import_validation_dependencies.rs` mixed tool-owned preflight keys such as `datasourceUids`, `pluginIds`, `sourcePath`, and `summary.blockingCount` with ordinary Grafana payload field reads.
- Current Update: Grouped dashboard dependency availability, resource-spec, and preflight summary keys under local schema namespaces while leaving Grafana raw payload fields direct.
- Result: Focused import, preflight, dashboard-plan, formatter, maintainability, AI workflow, and full Rust tests pass.

## 2026-04-20 - Move dashboard authoring into directory boundary
- State: Done
- Scope: Rust dashboard authoring module layout, focused authoring/dashboard validation, and AI trace docs. Import/reconcile directory moves, inspect/governance moves, public CLI behavior, generated docs, Python implementation, and output contracts are out of scope.
- Baseline: dashboard authoring implementation and root authoring regression tests lived as flat files in `commands/dashboard/`.
- Current Update: Moved dashboard authoring implementation and direct authoring regression tests under `commands/dashboard/authoring/` while keeping `commands/dashboard/mod.rs` as the public facade.
- Result: Focused authoring/dashboard tests, formatter, maintainability, AI workflow, and full Rust tests pass.
