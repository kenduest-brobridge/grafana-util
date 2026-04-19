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

## 2026-04-18 - Fix Rust 1.95 sync review clippy failure
- State: Done
- Scope: Rust sync review TUI key handling, CI failure analysis, focused sync tests, full Rust test, clippy, formatting, architecture gate, and AI trace docs. Public CLI behavior, generated docs, README files, JSON contracts, and Python implementation are out of scope.
- Baseline: GitHub Actions `rust-quality` passed cargo tests on commit `8a6b7d6b`, then failed under Rust 1.95 clippy because nested `if diff_mode` checks inside `review_tui` key handling triggered the new `collapsible_match` lint.
- Current Update: Collapsed the nested diff-mode checks into guarded `match key.code` arms while preserving the same checklist and diff-view key behavior.
- Result: Focused sync tests, full Rust tests, local clippy, formatting, and architecture checks pass. CI must rerun on pushed commits to verify the Rust 1.95 lint gate.

## 2026-04-19 - Clarify contract ownership map
- State: Done
- Scope: `docs/internal/contract-doc-map.md`, contract registry routing notes, and trace docs. Runtime JSON output, schema manifests, public CLI behavior, generated docs, README files, and Python implementation are out of scope.
- Current Update: Clarified the boundary between runtime golden output contracts, CLI/docs routing contracts, docs-entrypoint navigation, and schema/help manifests so the maintainer map now names the source of truth for each layer explicitly.
- Result: The contract map now distinguishes `command-surface.json`, `docs-entrypoints.json`, `output-contracts.json`, and `schemas/manifests/` as separate ownership surfaces.

## 2026-04-19 - Advance status and review-governance cleanup
- State: Done
- Scope: Rust alert live project-status normalization, TODO backlog cleanup, contract promotion guidance, mutation review-envelope inventory, focused tests, formatting, architecture checks, and AI trace docs. Public CLI behavior, generated docs, README files, and Python implementation are out of scope.
- Current Update: Routed the alert live status producer through the shared status reading model, removed stale completed work from the active backlog, documented runtime-vs-schema promotion rules, and captured an internal review-envelope inventory before any public JSON changes.
- Result: Focused alert/status tests, contract/schema checks, full Rust tests, clippy, formatting, architecture, and AI workflow checks pass locally.

## 2026-04-18 - Split oversized Rust test surfaces
- State: Done
- Scope: Rust test-surface maintainability for sync bundle execution, dashboard export/import/topology, dashboard browse workflow, snapshot, access org runtime, TODO backlog, focused tests, full Rust test, clippy, architecture gate, and AI trace docs. README files, generated user docs, public CLI behavior, JSON contracts, and Python implementation are out of scope.
- Baseline: Several Rust regression files mixed unrelated behavior suites in 900+ line modules, which made review and worker assignment harder even after production architecture warnings were clean.
- Current Update: Split the largest test hubs into behavior-named sibling modules while keeping their original files as routing facades and shared fixture homes where appropriate. Restored the existing access user runtime module include and kept dashboard browse's test-only document builder explicit for clippy.
- Result: Focused sync/dashboard/snapshot/access tests pass, full Rust tests pass, clippy and formatting pass, and `make quality-architecture` remains clean.
