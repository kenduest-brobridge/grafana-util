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

## 2026-04-19 - Add artifact workspace run support
- State: Done
- Scope: Rust profile config, artifact resolver, dashboard/snapshot/datasource/access export and local artifact routing, CLI config flag, focused parser/test literal updates, and AI trace docs. Generated docs, README files, Python implementation, and live Grafana behavior beyond explicit artifact/local flags are out of scope.
- Baseline: Export commands had per-domain default directories and profile config resolved connection settings from `grafana-util.yaml` or `GRAFANA_UTIL_CONFIG`; local browse/summary/review commands required explicit input directories.
- Current Update: Added config-relative `artifact_root`, run-centric artifact resolver, root `--config`, artifact `--run`/`--run-id`, and selected `--local` consumers for dashboard, snapshot, datasource, and access lanes.
- Result: Implementation completed without running validation by request. Known limitations: dashboard import/diff required-input flows and access import/diff still prefer explicit directories; snapshot review uses the default artifact scope until profile-aware review args are added.

## 2026-04-19 - Formalize artifact workspace docs
- State: Done
- Scope: public command docs, getting-started handbook, command-surface contract, generated docs, and AI trace docs for artifact workspace export defaults. Runtime code, Python implementation, README files, and live Grafana behavior are out of scope.
- Baseline: artifact workspace support existed in Rust, but operator docs and the public command-surface contract did not yet spell out config-relative `artifact_root`, timestamp/latest/run-id behavior, or lane placement.
- Current Update: Documented `grafana-util.yaml`, root `--config`, `artifact_root`, run layout, latest pointer, and dashboard/snapshot/datasource/access artifact lanes in English and zh-TW docs.
- Result: Generated docs, docs-surface, and AI workflow checks pass.

## 2026-04-19 - Broaden artifact workspace local consumers
- State: Done
- Scope: Rust dashboard/access import and diff artifact input routing, command docs, command-surface contract, generated docs, and AI trace docs. Python implementation, README files, and live Grafana behavior beyond resolving local artifact input paths are out of scope.
- Baseline: Dashboard import/diff and access import/diff required explicit input or diff directories even after export/list/browse flows could resolve profile artifact runs.
- Current Update: Added `--local`, `--run`, and `--run-id` artifact input resolution for dashboard import/diff and access user/team/org/service-account import/diff.
- Result: Rust formatting, generated docs, docs-surface, and AI workflow checks pass. Rust tests were not run.

## 2026-04-20 - Complete Python Rust parity surfaces
- State: Done
- Scope: Python dashboard history diff/plan, dashboard topology interactive rendering, status live all-org/read-failure handling, access user/team browse entrypoints, artifact-workspace local browse resolution, focused Python tests, and AI trace docs. Rust implementation and generated docs are out of scope except as source-of-truth references.
- Baseline: Python lacked Rust-public `dashboard history diff`, `dashboard plan`, and access browse entrypoints; topology interactive mode returned an unsupported error; status live silently swallowed several live read failures and called a missing dashboard client method.
- Current Update: Added Python command wiring and runtime documents for dashboard plan/history diff, a deterministic topology interactive text browser, scoped live status all-org aggregation with blocked read-failure domains, access browse list/local-bundle flows, profile artifact lane resolution for access browse `--local/--run/--run-id`, dashboard plan `--use-export-org` routed review, and focused tests.
- Result: Focused Python syntax/unit tests, full Python discovery, docs-surface, and AI workflow checks pass.

## 2026-04-20 - Complete Python artifact and plan parity
- State: Done
- Scope: Python artifact workspace resolver, datasource local/plan flows, access local/plan flows, snapshot local review, focused Python tests, and AI trace docs. Existing Rust worktree changes are out of scope and must not be modified.
- Baseline: Python artifact run selectors still accepted legacy `previous`, datasource lacked Rust-public `plan`, access lacked root `plan` and local import/diff/list coverage, and snapshot review could not resolve artifact workspace runs.
- Current Update: Normalized Python artifact selectors to `latest`/`timestamp`/`run-id`, added datasource plan/local coverage, access plan/local coverage, and snapshot artifact review/export coverage.
- Result: Python parity surfaces now match the Rust artifact-workspace direction for focused datasource, access, and snapshot flows.

## 2026-04-20 - Split dashboard artifact command routing
- State: Done
- Scope: Rust dashboard command artifact workspace routing, local artifact input materialization, focused Rust tests, and AI trace docs. Public CLI behavior, command docs, generated docs, Python implementation, and output contracts are out of scope.
- Baseline: `dashboard command_runner.rs` still owned both top-level command dispatch and artifact workspace run/local-input resolution, keeping export orchestration coupled to artifact path materialization.
- Current Update: Extracted dashboard artifact run selection, output lane routing, latest-run recording, and local input materialization into `command_artifacts.rs`; `command_runner.rs` now delegates those artifact concerns while keeping command execution routing.
- Result: Focused dashboard parser/artifact command tests, full Rust tests, maintainability report, and AI workflow checks pass.

## 2026-04-20 - Continue Rust architecture cleanup
- State: Done
- Scope: Rust dashboard test organization, dashboard facade re-export boundary, snapshot artifact-workspace coverage, focused Rust tests, and AI trace docs. Public CLI behavior, command docs, generated docs, Python implementation, and output contracts are out of scope.
- Baseline: Dashboard artifact workspace tests lived inside a broader parser/help workflow test file, `dashboard/mod.rs` mixed module registration with a long public/crate-private re-export block, and snapshot export had no direct latest-run pointer coverage for artifact-workspace timestamp runs.
- Current Update: Split dashboard artifact workflow tests into a dedicated test module, moved dashboard facade re-exports into `facade_exports.rs`, and added a narrow snapshot artifact export latest-run coverage test.
- Result: Focused dashboard artifact/parser tests, dashboard scope tests, snapshot scope tests, full Rust tests, formatter check, maintainability report, and AI workflow checks pass.
