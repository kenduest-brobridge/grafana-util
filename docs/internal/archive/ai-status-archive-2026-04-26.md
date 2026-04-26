# ai-status-archive-2026-04-26

## 2026-04-20 - Clean up dashboard import dependency schema keys
- State: Done
- Scope: Rust dashboard import dependency preflight schema-key cleanup, focused import/preflight/dashboard-plan validation, and AI trace docs. Alert runtime schema cleanup, import directory moves, public CLI behavior, generated docs, Python implementation, and output contracts are out of scope.
- Baseline: `commands/dashboard/import_validation_dependencies.rs` mixed tool-owned preflight keys such as `datasourceUids`, `pluginIds`, `sourcePath`, and `summary.blockingCount` with ordinary Grafana payload field reads.
- Current Update: Grouped dashboard dependency availability, resource-spec, and preflight summary keys under local schema namespaces while leaving Grafana raw payload fields direct.
- Result: Focused import, preflight, dashboard-plan, formatter, maintainability, AI workflow, and full Rust tests pass.

## 2026-04-20 - Clean up alert runtime schema keys
- State: Done
- Scope: Rust alert runtime plan, delete-preview, and import dry-run schema-key cleanup, focused sync/alert/import validation, and AI trace docs. Alert diff shared document internals, public CLI behavior, generated docs, Python implementation, and output contracts are out of scope.
- Baseline: `commands/alert/runtime_support.rs` still repeated tool-owned plan row, plan document, delete-preview, and import dry-run keys directly in production render/read paths.
- Current Update: Grouped alert runtime document, row, and summary keys under local schema namespaces while keeping Grafana raw alert payload fields direct.
- Result: Focused sync, alert, runtime, dashboard import, formatter, maintainability, AI workflow, and full Rust tests pass.

## 2026-04-20 - Move dashboard import into directory boundary
- State: Done
- Scope: Rust dashboard import module layout, focused import/routed/plan/browse validation, and AI trace docs. Plan reconciliation, inspect/governance moves, public CLI behavior, generated docs, Python implementation, and output contracts are out of scope.
- Baseline: dashboard import implementation lived as many flat `commands/dashboard/import_*.rs` files, while plan-owned reconciliation already lived under `commands/dashboard/plan/`.
- Current Update: Moved the dashboard import implementation under `commands/dashboard/import/`, kept `commands/dashboard/mod.rs` as the public facade, and left plan reconciliation in the plan-owned boundary.
- Result: Focused import/routed/plan/browse tests, formatter, maintainability, AI workflow, and full Rust tests pass.

## 2026-04-20 - Finish project status producer audit
- State: Done
- Scope: Rust project-status producer audit across sync, datasource, alert, dashboard, access, and live status fallback producers. Public CLI behavior, generated docs, Python implementation, and output contracts are out of scope.
- Baseline: Most domain producers already routed through `StatusReading`, but the remaining audit still needed to confirm whether any direct producer construction remained.
- Current Update: Audited the producer lanes in workers and normalized the remaining dashboard live read-failure fallback in test support through `StatusReading` while preserving live evidence and public status fields.
- Result: Focused dashboard/access/status/project_status tests, formatter, architecture, AI workflow, and full Rust tests pass.
