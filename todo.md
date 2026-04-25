# TODO

Current maintainer backlog for the Rust-first `grafana-util` project.

Scope rules:

- Treat `rust/src/` as the primary implementation surface.
- Ignore Python implementation unless packaging, install behavior, or explicit parity work requires it.
- Keep README changes out of this backlog unless a task explicitly targets public GitHub positioning.
- Prefer small grouped commits with focused validation.
- Use the conservative boundary policy below before starting any split.

## Current Baseline

- Branch is `dev`; keep new work grouped into focused Rust/test commits.
- Release `v0.11.0` is cut at `6ab7ab03`. `dev` and `main` now include the
  post-release Grafana 13 datasource API CI fix at `18f6f355`.
- GitHub Actions `rust-quality` and `rust-live-smoke` are green for
  `18f6f355`.
- Local validation for the Grafana 13 datasource fix passed with
  `make quality-rust` and `make test-rust-live` against
  `grafana/grafana:13.0.1`.
- Default Rust build and `--features browser` are supported release surfaces.
- `--no-default-features` is explicitly not claimed as a supported release surface yet.
- Dashboard `summary` / `dependencies` naming and review-source model are now clearer.
- Output contracts have root and nested-path validation through `requiredFields`, `requiredPaths`, `pathTypes`, and golden fixtures.
- Oversized Rust test facades and test-only `pub(crate)` visibility have been
  reduced. Do not re-open those unless a new mixed-responsibility hotspot appears.
- Remaining risk is mostly maintainability: the remaining status producers,
  TUI input/render modules, live apply paths, read-throughput hotspots, and
  overlapping contract systems.

## First Priority - Rust Deficit Audit

This is the current first-priority Rust backlog. Treat it as the ordering lens
before taking new architecture or cleanup work.

Observed gaps:

- [ ] Dashboard remains the heaviest domain and the highest maintenance risk.
  The issue is not line count alone; dashboard owns export/import, inspect,
  governance, topology, live review, TUI, screenshot, and source-alignment
  behavior in one broad surface.
- [ ] Grafana 13 Git Sync ownership is now guarded in dashboard import/plan
  paths. Remaining Git Sync work is broader dashboard/workspace source routing,
  export layout, and operator docs, not the direct-write safety guard.
- [ ] Crate-root internal module routing is heavy. `rust/src/lib.rs` documents
  the facade boundaries, but many crate-private modules are still mounted from
  the root with `#[path = ...]`, so new shared surfaces should be added
  conservatively and kept domain-local where possible.
- [ ] TUI/browser feature surfaces are broad. Default `tui` and optional
  `browser` builds are supported release lanes, so any TUI/browser-adjacent
  change must validate the feature matrix, not just default tests.
- [ ] Live read throughput has bounded fan-out for dashboard details, alert
  templates, dashboard/folder permission export reads, and a shared
  dashboard/datasource all-org read pass. Remaining transport risk is proven
  hot spots only.
- [ ] Mutation review envelopes remain domain-shaped. A shared adapter should
  wait until dashboard/datasource/access/alert/workspace prove the same shape.
- [ ] Production assumptions need opportunistic cleanup. Most `unwrap`,
  `expect`, and `panic` occurrences are tests or hard-coded regex assertions,
  but touched live/operator paths should prefer `Result` errors over panic.

First-priority handling order:

- [ ] Continue dashboard inspect/governance/report splits across one stable
  responsibility boundary at a time. The topology impact builder and
  inspect-summary projection/document/row builders are already separated.
- [ ] Keep mutation review envelope, dashboard v2 adapter, and broader shared
  status producer adoption deferred until the earlier boundaries prove stable.

## Active Execution Queue

Run the next development passes in this order unless a CI failure or user report
changes priority.

- [ ] Continue dashboard inspect/governance/report code splits only where a
  stable responsibility boundary is obvious. Report model, query-report
  collection, query analyzer, inspect governance report internals,
  inspect-summary projection, topology impact, governance gate rules/TUI, and
  governance gate runner/output support are done; keep
  `commands/dashboard/mod.rs` as the facade for later moves.
- [ ] Keep the mutation review envelope adapter work later and only introduce a shared adapter once two or more domains prove the same review shape.
- [ ] Keep dashboard v2 as a future adapter boundary. Continue rejecting v2-shaped input in the classic prompt lane and keep prompt export parity guarded with fixtures and tests.

Detailed execution items:

- Dashboard inspect/governance/report re-layering:
  - [ ] Use a fresh inventory of remaining inspect, governance, report,
    topology, and policy modules before each later move; topology impact and
    inspect-summary projection are already split.
  - [ ] Choose exactly one next boundary; remaining candidates should be based
    on a fresh inventory because the obvious governance gate runner/output
    boundary is already separated from the facade.
  - [ ] Use `git mv` for tracked moves and keep `commands/dashboard/mod.rs`
    as the facade.
  - [ ] Keep public CLI/help unchanged; if help changes accidentally, back out
    wording changes or update docs/contracts in the same commit.
- Read-only HTTP throughput:
  - [ ] Preserve deterministic output ordering after concurrent fetches.
  - [ ] Keep write/apply/import requests serial.
  - [ ] Gate any default concurrency value behind a conservative constant and
    live smoke it against the fixed Grafana container.
  - [ ] Add tests for partial read failures so the first useful diagnostic is
    still visible.
## Rust Architecture Follow-up Checklist

Use this checklist for the next Rust maintenance passes. Keep each checked item
as a focused commit group with narrow validation.

### P2 - Dashboard Directory Re-layering

- [ ] Continue moving inspect/governance/report files into clearer
  inspect/governance boundaries only when imports remain manageable.
- [ ] Keep `commands/dashboard/mod.rs` as the public facade and avoid changing
  CLI command paths.
- [ ] Use `git mv` for tracked file moves.
- [ ] Run focused dashboard suites and parser/help tests.
- [ ] Run full Rust tests after the move.

## Split Policy - Conservative Boundaries

Use this policy before implementing any TODO item in this file.

The goal is not to make every file small. The goal is to make each module
own one stable responsibility without turning the codebase into a maze of tiny
files.

Rules:

- Split by responsibility, not by line count alone.
- Keep the original file as a facade, routing point, or assembly point when that helps readability.
- Add at most 1-3 new modules per task unless splitting a test suite into obvious behavior groups.
- Do not extract a module unless its name describes a stable concept in the domain.
- Do not introduce `utils`, `helpers2`, `misc`, or similar catch-all modules.
- Prefer behavior-preserving moves before abstraction changes.
- Keep control flow readable from the parent file after the split.
- Avoid shared traits or generic envelopes until at least two or three domains have proven the same shape.

Pre-split checklist:

- What responsibility is being separated?
- Which file remains the facade after the split?
- Can a reviewer understand the workflow without opening every new file?
- Is the new module name domain-specific and stable?
- Does the split reduce mixed responsibility, or only reduce line count?
- Are fixtures/setup duplicated after the split?

Reject the split if the answer is only "the file is large." Large files are
acceptable when they own one clear responsibility and are easier to read in one
place.

## P0 - Dashboard Source-Alignment Follow-ups

Keep these follow-ups separated from the classic prompt contract so the next
changes stay reviewable and do not blur lane boundaries.

- [ ] Add first-class Grafana Git Sync awareness to dashboard/workspace flows.
  Git Sync-managed dashboard folders should be treated as Git-owned targets:
  dashboard JSON deployment should go through the Git repository / PR path, not
  direct dashboard API import or workspace apply.
- [ ] Detect and surface dashboard ownership/provenance in live inventory and
  preflight evidence: API-managed, file-provisioned, or Git Sync-managed. Mark
  Git Sync targets as read-only for direct live dashboard writes by default.
- [ ] Add Git Sync-friendly layout support in dashboard export/convert,
  workspace scan/preview, and dashboard plan so repo trees can be reviewed
  without pretending they are ordinary live API targets.
- [ ] Update dashboard import/apply docs and command guidance so Git Sync
  folders route changes to Git while normal unmanaged folders can still use API
  import/apply.
- [ ] Keep datasource, alert, access, and status workflows as direct product
  differentiators; Grafana Git Sync mainly changes dashboard JSON ownership, not
  datasource/access/alert lifecycle management.
- [ ] Keep live library-panel `__elements` lookup limited to the live export / import-handoff path. Keep local raw-to-prompt conversion warning-only when a referenced library panel model is missing.
- [ ] Keep prompt/export fixture parity anchored to Grafana source testdata for datasource variables, selected current datasource handling, library panels, and the classic-vs-v2 rejection boundary.
- [ ] Extend the implemented dashboard import/plan ownership evidence into any
  remaining publish or workspace paths that still lack provenance before live
  writes.
- [ ] Keep dashboard v2 as a separate future adapter boundary. Continue rejecting v2-shaped input in the classic prompt lane rather than mixing it into `raw/`, `prompt/`, or provisioning behavior.
- [ ] Treat provisioning as a derived projection that can be compared later against Grafana file provisioning. Do not rebase the dashboard contract on provisioning as if it were the source of truth.
- [ ] Keep dashboard permissions adjacent to access evidence and access workflows, not as dashboard JSON fields or as an extension of the prompt export shape.
- [ ] Split large dashboard modules by responsibility, not by line count alone. Favor focused export planning, prompt conversion, live preflight, and provisioning projection boundaries over arbitrary file carving.

## P1 - Status Producer Model

### Normalize Project Status Producers

Problem:

Status/project-status logic exists across dashboard, datasource, access, alert, sync, and `status overview`, but the producer contract is not fully unified.

Relevant areas:

- `rust/src/commands/dashboard/project_status.rs`
- `rust/src/commands/datasource/project_status/live.rs`
- `rust/src/commands/datasource/project_status/staged.rs`
- `rust/src/commands/access/project_status.rs`
- `rust/src/commands/status/live.rs`
- `rust/src/commands/status/overview/`

Action:

- [ ] Keep live producer collection and multi-org transport outside the shared
  trait; dashboard/datasource live status now share the producer adapter after
  their domain inputs are collected.

## P1 - HTTP Transport Efficiency

### Improve Live Grafana Request Throughput

Problem:

Rust HTTP handling is reliable and centralized, but current live/export/status
paths are conservative for large Grafana instances. `JsonHttpClient` reuses one
reqwest blocking client, which is good, but successful responses are fully read
and converted to `String` before JSON parsing, response compression is disabled
with `Accept-Encoding: identity`, HTTP/2 is disabled with `http1_only()`, and
dashboard/detail fetches, alert template details, dashboard/folder permission
export reads, and the dashboard/datasource all-org status pass already avoid
the known repeated reads.

Relevant areas:

- `rust/src/grafana/http.rs`
- `rust/src/grafana/api/dashboard.rs`
- `rust/src/grafana/api/sync_live_read.rs`
- `rust/src/commands/dashboard/export_support.rs`
- `rust/src/commands/status/live.rs`
- `rust/src/commands/status/live_multi_org.rs`

Action:

- [ ] Keep write/apply paths serial unless dependency ordering and Grafana API safety are explicitly modeled.
- [ ] Reduce `serde_json::Value` cloning only at proven hot spots; dashboard
  live-read detail normalization now moves the fetched dashboard body instead
  of deep-cloning it, while flexible JSON handling remains for version-varying
  Grafana API shapes.

Validation:

- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet http`.
- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet sync_live`.
- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet dashboard`.
- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet status`.
- [ ] Run live smoke against a fixed local Grafana container before changing concurrency defaults.

## P2 - Live Apply Safety

### Standardize Mutation Review Envelopes

Problem:

Dashboard, datasource, access, alert, and workspace mutation flows each have review/dry-run/apply concepts, but envelopes are still domain-shaped.

Current baseline:

- Shared internal `ReviewAction`, `ReviewBlockedReason`, and
  `ReviewApplyResult` adapters exist without changing public JSON contracts.

Action:

- [ ] Introduce a shared `ReviewRisk` concept.
- [ ] Introduce a shared `ReviewRequest` concept.
- [ ] Keep domain-specific payloads behind a shared review wrapper.
- [ ] Avoid changing public JSON contracts until a migration path is defined.
- [ ] Start with one internal model or adapter. Do not force all domains to adopt the envelope in the first commit.

Current blocker:

- Dashboard/access now prove a shared action/status/blocked-reason shape, and
  alert/sync live apply now prove the common apply-result evidence shape.
  `ReviewRisk` and `ReviewRequest` still need cautious evidence handling.
  Current risk records are still only dashboard-governance shaped
  (`GovernanceRiskSpec` and `GovernanceRiskRow`). Current request structs do
  not prove a shared review-request shape: dashboard source loading,
  datasource import request planning, and dashboard import lookup request
  closure wrappers represent different layers and should stay domain-local
  until a second mutation-review domain emits the same evidence fields.

Validation:

- [ ] Run domain-focused tests first.
- [ ] Run full `cargo test --manifest-path rust/Cargo.toml --quiet` after shared envelope changes.
- [ ] Run `make quality-output-contracts` if JSON output changes.

## P3 - Product Surface Balance

### Keep Domain Maturity Balanced

Problem:

Dashboard tooling remains deeper than some other domains. That is useful, but the tool should not become dashboard-only in practice.

Action:

- [ ] For every new dashboard intelligence feature, check whether access, datasource, alert, or workspace needs a corresponding minimal contract.
- [ ] Prefer shared review/status/output infrastructure before adding another dashboard-only surface.
- [ ] Keep simple backup/export use cases low-friction.

Validation:

- [ ] Run `make quality-architecture`.
- [ ] Run `make quality-docs-surface`.
- [ ] Run domain-focused Rust tests.

## General Guardrails

- Do not inspect or edit `rust/target`.
- Do not modify README unless the task explicitly targets GitHub-facing positioning.
- Do not touch Python implementation for these tasks.
- Do not perform mechanical line-count splits without the pre-split checklist.
- Prefer fewer, stronger modules over many tiny modules.
- Use grouped commits:
  - Use `refactor:` for behavior-preserving Rust splits.
  - Use `test:` for contract/test coverage.
  - Use `docs:` for maintainer docs and generated docs.
  - Use `bugfix:` only for real behavior fixes.
- For public CLI/help/docs changes, run:
  - Run `make quality-docs-surface`.
  - Run `make man-check`.
  - Run `make html-check`.
- For output JSON changes, run:
  - Run `make quality-output-contracts`.
- For broad Rust refactors, run:
  - Run `cargo fmt --manifest-path rust/Cargo.toml --all --check`.
  - Run focused Rust tests.
  - Run `cargo test --manifest-path rust/Cargo.toml --quiet`.
  - Run `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`.
