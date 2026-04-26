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
