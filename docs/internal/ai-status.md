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

## 2026-04-20 - Split dashboard artifact command routing
- State: Done
- Scope: Rust dashboard command artifact workspace routing, local artifact input materialization, focused Rust tests, and AI trace docs. Public CLI behavior, command docs, generated docs, Python implementation, and output contracts are out of scope.
- Baseline: `dashboard command_runner.rs` still owned both top-level command dispatch and artifact workspace run/local-input resolution, keeping export orchestration coupled to artifact path materialization.
- Current Update: Extracted dashboard artifact run selection, output lane routing, latest-run recording, and local input materialization into `command_artifacts.rs`; `command_runner.rs` now delegates those artifact concerns while keeping command execution routing.
- Result: Focused dashboard parser/artifact command tests, full Rust tests, maintainability report, and AI workflow checks pass.

## 2026-04-20 - Complete Python artifact and plan parity
- State: Done
- Scope: Python artifact workspace resolver, datasource local/plan flows, access local/plan flows, snapshot local review, focused Python tests, and AI trace docs. Existing Rust worktree changes are out of scope and must not be modified.
- Baseline: Python artifact run selectors still accepted legacy `previous`, datasource lacked Rust-public `plan`, access lacked root `plan` and local import/diff/list coverage, and snapshot review could not resolve artifact workspace runs.
- Current Update: Normalized Python artifact selectors to `latest`/`timestamp`/`run-id`, added datasource plan/local coverage, access plan/local coverage, and snapshot artifact review/export coverage.
- Result: Python parity surfaces now match the Rust artifact-workspace direction for focused datasource, access, and snapshot flows.

## 2026-04-20 - Complete Python Rust parity surfaces
- State: Done
- Scope: Python dashboard history diff/plan, dashboard topology interactive rendering, status live all-org/read-failure handling, access user/team browse entrypoints, artifact-workspace local browse resolution, focused Python tests, and AI trace docs. Rust implementation and generated docs are out of scope except as source-of-truth references.
- Baseline: Python lacked Rust-public `dashboard history diff`, `dashboard plan`, and access browse entrypoints; topology interactive mode returned an unsupported error; status live silently swallowed several live read failures and called a missing dashboard client method.
- Current Update: Added Python command wiring and runtime documents for dashboard plan/history diff, a deterministic topology interactive text browser, scoped live status all-org aggregation with blocked read-failure domains, access browse list/local-bundle flows, profile artifact lane resolution for access browse `--local/--run/--run-id`, dashboard plan `--use-export-org` routed review, and focused tests.
- Result: Focused Python syntax/unit tests, full Python discovery, docs-surface, and AI workflow checks pass.

## 2026-04-19 - Broaden artifact workspace local consumers
- State: Done
- Scope: Rust dashboard/access import and diff artifact input routing, command docs, command-surface contract, generated docs, and AI trace docs. Python implementation, README files, and live Grafana behavior beyond resolving local artifact input paths are out of scope.
- Baseline: Dashboard import/diff and access import/diff required explicit input or diff directories even after export/list/browse flows could resolve profile artifact runs.
- Current Update: Added `--local`, `--run`, and `--run-id` artifact input resolution for dashboard import/diff and access user/team/org/service-account import/diff.
- Result: Rust formatting, generated docs, docs-surface, and AI workflow checks pass. Rust tests were not run.

## 2026-04-19 - Formalize artifact workspace docs
- State: Done
- Scope: public command docs, getting-started handbook, command-surface contract, generated docs, and AI trace docs for artifact workspace export defaults. Runtime code, Python implementation, README files, and live Grafana behavior are out of scope.
- Baseline: artifact workspace support existed in Rust, but operator docs and the public command-surface contract did not yet spell out config-relative `artifact_root`, timestamp/latest/run-id behavior, or lane placement.
- Current Update: Documented `grafana-util.yaml`, root `--config`, `artifact_root`, run layout, latest pointer, and dashboard/snapshot/datasource/access artifact lanes in English and zh-TW docs.
- Result: Generated docs, docs-surface, and AI workflow checks pass.

## 2026-04-19 - Add artifact workspace run support
- State: Done
- Scope: Rust profile config, artifact resolver, dashboard/snapshot/datasource/access export and local artifact routing, CLI config flag, focused parser/test literal updates, and AI trace docs. Generated docs, README files, Python implementation, and live Grafana behavior beyond explicit artifact/local flags are out of scope.
- Baseline: Export commands had per-domain default directories and profile config resolved connection settings from `grafana-util.yaml` or `GRAFANA_UTIL_CONFIG`; local browse/summary/review commands required explicit input directories.
- Current Update: Added config-relative `artifact_root`, run-centric artifact resolver, root `--config`, artifact `--run`/`--run-id`, and selected `--local` consumers for dashboard, snapshot, datasource, and access lanes.
- Result: Implementation completed without running validation by request. Known limitations: dashboard import/diff required-input flows and access import/diff still prefer explicit directories; snapshot review uses the default artifact scope until profile-aware review args are added.
