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
- Local `dev` is synchronized with `origin/dev` after the validated Rust
  maintenance commits landed.
- GitHub Actions `rust-quality` is currently green on Rust 1.95 after the
  latest clippy compatibility pass.
- Default Rust build and `--features browser` are supported release surfaces.
- `--no-default-features` is explicitly not claimed as a supported release surface yet.
- Dashboard `summary` / `dependencies` naming and review-source model are now clearer.
- Output contracts have root and nested-path validation through `requiredFields`, `requiredPaths`, `pathTypes`, and golden fixtures.
- Oversized Rust test facades and test-only `pub(crate)` visibility have been
  reduced. Do not re-open those unless a new mixed-responsibility hotspot appears.
- Remaining risk is mostly maintainability: the remaining status producers,
  TUI input/render modules, live apply paths, output contract depth, and
  overlapping contract systems.

## Active Execution Queue

Run the next development passes in this order unless a CI failure or user report
changes priority.

- [ ] Finish the remaining project status producer audit and only normalize the last direct producers that still need the shared internal status reading model. Keep live Grafana evidence such as health and version where available.
- [ ] Add the first concrete contract promotion checker report after the contract ownership lanes are documented. Start informational only.
- [ ] Keep the mutation review envelope adapter work later and only introduce a shared adapter once two or more domains prove the same review shape.
- [ ] Keep dashboard v2 as a future adapter boundary. Continue rejecting v2-shaped input in the classic prompt lane and keep prompt export parity guarded with fixtures and tests.

## Rust Architecture Follow-up Checklist

Use this checklist for the next Rust maintenance passes. Keep each checked item
as a focused commit group with narrow validation.

### P1 - Schema Boundary Cleanup

- [x] Define the boundary between tool-owned artifact schema keys and Grafana raw API response keys in the touched module before refactoring.
- [x] Centralize sync staged/review/apply document keys in namespaced modules or typed helpers instead of repeating `"kind"`, `"summary"`, `"resourceCount"`, and `"blockingCount"` in production parsing/render paths.
- [x] Clean up `rust/src/commands/sync/staged_documents_render.rs` raw tool schema literals.
- [x] Clean up `rust/src/commands/sync/workspace_preview_review_view.rs` plan and review document key access.
- [x] Clean up `rust/src/commands/sync/project_status_json.rs` summary helper key access if it still reads tool-owned document sections directly.
- [ ] Clean up `rust/src/commands/alert/runtime_support.rs` alert plan, delete-preview, and import dry-run document keys where they are tool-owned.
- [x] Clean up `rust/src/commands/dashboard/import_validation_dependencies.rs` summary/blocking document keys without globalizing ordinary Grafana keys such as `uid`, `name`, or `folderUid`.
- [ ] Keep test fixture `json!` documents readable; do not force every fixture key through constants unless it removes real duplication.
- [ ] Run focused validation for `sync`, `alert`, `dashboard import_validation`, and full Rust tests.

### P1 - Split `grafana/api/sync_live_read.rs`

- [x] Keep the existing public API/facade stable before moving code.
- [x] Split dashboard live-read collection into a dedicated child module.
- [x] Split datasource live-read collection into a dedicated child module.
- [x] Split folder live-read collection into a dedicated child module if it has enough independent behavior.
- [x] Split alert live-read collection into a dedicated child module.
- [x] Keep availability aggregation in one clear module after the read facets are separated.
- [x] Avoid changing live apply or request behavior during the split.
- [x] Run `cargo test --manifest-path rust/Cargo.toml --quiet sync_live --lib`.
- [x] Run `cargo test --manifest-path rust/Cargo.toml --quiet status --lib`.
- [x] Run full Rust tests.

### P2 - Dashboard Directory Re-layering

- [x] Propose the first directory boundary before moving files; prefer one stable domain at a time.
- [x] Move dashboard authoring into `commands/dashboard/authoring/` only if the facade remains easy to follow.
- [ ] Move dashboard import/reconcile files into `commands/dashboard/import/` after schema cleanup reduces key drift.
- [ ] Move inspect/governance/report files into a clearer inspect/governance sub-tree only when imports remain manageable.
- [ ] Keep `commands/dashboard/mod.rs` as the public facade and avoid changing CLI command paths.
- [x] Use `git mv` for tracked file moves.
- [x] Run focused dashboard suites and parser/help tests.
- [ ] Run docs surface checks if public help or command routing changes.
- [x] Run full Rust tests.

### P2 - Split `common/mod.rs`

- [ ] Split only after the sync/API/dashboard cleanup above settles to avoid noisy import churn.
- [ ] Extract shared error/result definitions into `common/error.rs`.
- [ ] Extract auth/header resolution into `common/auth.rs`.
- [ ] Extract JSON render/color handling into `common/json_output.rs`.
- [ ] Extract file output helpers into `common/io.rs`.
- [ ] Extract string/path normalization helpers into `common/normalize.rs` if call sites stay readable.
- [ ] Extract shared diff document helpers into `common/diff_document.rs`.
- [ ] Keep `common/mod.rs` as the facade and preserve existing imports where practical.
- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet common --lib`.
- [ ] Run CLI help/parser tests.
- [ ] Run full Rust tests.

Completed cleanup now closed:

- Push baseline on `dev` completed and is already validated.
- Status producers routed through `StatusReading` for access and dashboard.
- Access user browse mutation dispatch was split into a dedicated mutation
  module.
- Access user browse reload behavior was split into a dedicated reload module.
- Dashboard browse footer and action rendering were split from the main render
  path.
- Sync live apply datasource target lookup was split into a shared helper.
- Sync live apply alert dispatch was split into a dedicated helper.
- Dashboard browse live detail loading was split from browse document support.
- Raw-to-prompt resolver responsibilities were split into prompt path and
  datasource resolution modules.
- Raw-to-prompt live library-panel prompt export parity is covered by a mock
  Grafana regression test.
- Raw-to-prompt clippy test module ordering was fixed.
- Output contract checker collection and enum constraint checks are in place.
- Docs diff classifier is in place.
- Feature matrix full probe is in place.
- Access team browse reload and confirmation boundaries are split.
- Dashboard browse tree rows are split.
- Sync live apply response normalization and error classification are split.
- Oversized Rust test suites split into smaller facades.
- Dashboard test helper re-exports narrowed from crate-wide visibility.
- Dashboard and snapshot test-support helpers narrowed to local module trees.
- Output contract checker now validates collection shape and enum constraints.

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

- [ ] What responsibility is being separated?
- [ ] Which file remains the facade after the split?
- [ ] Can a reviewer understand the workflow without opening every new file?
- [ ] Is the new module name domain-specific and stable?
- [ ] Does the split reduce mixed responsibility, or only reduce line count?
- [ ] Are fixtures/setup duplicated after the split?

Reject the split if the answer is only "the file is large." Large files are
acceptable when they own one clear responsibility and are easier to read in one
place.

## P0 - Dashboard Prompt External Export

### Align Prompt Export With Grafana UI Semantics

Status: classic prompt parity is covered for datasource variables, placeholder
references, selected current datasource mapping, and live library-panel model
export. Keep this item open for regression hardening and any future dashboard v2
adapter work.

Problem:

Grafana's official source has two external dashboard export paths. The classic
exporter and the scene exporter agree that prompt output must not synthesize
datasource variables or treat a datasource variable `query` as an import input.
The newer scene exporter also preserves `$datasource` panel references while
mapping a datasource variable's current concrete datasource through a `DS_*`
input when that variable is used by panel or target datasource references.

Official source areas to keep using as behavior references:

- `/Users/kendlee/tmp/grafana/public/app/features/dashboard/components/DashExportModal/DashboardExporter.ts`
- `/Users/kendlee/tmp/grafana/public/app/features/dashboard-scene/scene/export/exporters.ts`
- `/Users/kendlee/tmp/grafana/public/app/features/manage-dashboards/import/utils/inputs.ts`
- `/Users/kendlee/tmp/grafana/pkg/services/dashboardimport/utils/dash_template_evaluator.go`
- `/Users/kendlee/tmp/grafana/pkg/services/dashboardimport/service/service.go`

Action:

- [ ] Keep concrete datasource references mapped to `__inputs` and `${DS_*}`.
- [ ] Keep datasource variable definitions as variables; do not convert the variable `query` into a datasource input.
- [ ] Preserve panel and target datasource references such as `$datasource`.
- [ ] When a used datasource variable has a concrete current value and datasource type, add the corresponding `DS_*` input and set the variable `current.value` to `${DS_*}`.
- [ ] Keep constant variables mapped through `VAR_*` inputs.
- [ ] Keep expression datasource import handling (`__expr__`) out of user-mapped datasource inputs.
- [ ] Reject dashboard v2 resource/spec input in raw-to-prompt until a dedicated adapter exists.
- [ ] Keep library panel `__elements` live-model export covered by regression tests and add import input validation only when the import lane consumes those elements directly.

### Dashboard Source-Alignment Follow-ups

Keep these follow-ups separated from the classic prompt contract so the next
changes stay reviewable and do not blur lane boundaries.

- [ ] Keep live library-panel `__elements` lookup limited to the live export / import-handoff path. Keep local raw-to-prompt conversion warning-only when a referenced library panel model is missing.
- [ ] Keep prompt/export fixture parity anchored to Grafana source testdata for datasource variables, selected current datasource handling, library panels, and the classic-vs-v2 rejection boundary.
- [ ] Add dashboard import/publish preflight evidence for provisioned or managed dashboards before any live write. Surface ownership and provenance as target evidence instead of waiting for Grafana API failures.
- [ ] Keep dashboard v2 as a separate future adapter boundary. Continue rejecting v2-shaped input in the classic prompt lane rather than mixing it into `raw/`, `prompt/`, or provisioning behavior.
- [ ] Treat provisioning as a derived projection that can be compared later against Grafana file provisioning. Do not rebase the dashboard contract on provisioning as if it were the source of truth.
- [ ] Keep dashboard permissions adjacent to access evidence and access workflows, not as dashboard JSON fields or as an extension of the prompt export shape.
- [ ] Split large dashboard modules by responsibility, not by line count alone. Favor focused export planning, prompt conversion, live preflight, and provisioning projection boundaries over arbitrary file carving.

Validation:

- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet raw_to_prompt`.
- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet`.
- [ ] Run `cargo fmt --manifest-path rust/Cargo.toml --all --check`.

## P1 - TUI Boundary Cleanup

### Split Access User Browse Input

Problem:

`rust/src/commands/access/user_browse_input.rs` is still a dense TUI input surface. Mutation dispatch and reload behavior are now split; key dispatch, selection state, and error handling should be split only if a stable responsibility boundary remains.

Action:

- [ ] Extract only the next stable focused boundary if it remains mixed. Candidate boundary: key handling.
- [ ] Keep public behavior unchanged.
- [ ] Do not create all candidate modules in one pass unless each one removes a clearly mixed responsibility.

Validation:

- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet access`.
- [ ] Run `cargo fmt --manifest-path rust/Cargo.toml --all --check`.

### Continue Dashboard Browse Render Split

Problem:

Dashboard browse/render is still large and UI-sensitive even after the row split.

Hotspots:

- `rust/src/commands/dashboard/browse_support.rs`
- `rust/src/commands/dashboard/browse_render.rs`

Action:

- [ ] Keep detail-pane rendering, footer/action rendering, and live detail loading split.
- [ ] Separate live-tree rendering from local-export-tree rendering where practical.
- [ ] Keep the main render path readable from the current parent module; do not turn one render file into many single-widget files.

Validation:

- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_browse`.
- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_cli`.

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

- [ ] Introduce a shared data shape before introducing a trait. Candidate names: `StatusProducer`, `StatusReading`, `StatusWarning`, `StatusBlockedReason`, `StatusRecordCount`.
- [ ] Keep `status overview` as a consumer/reporting surface, not an orchestration owner.
- [ ] Move domain-specific discovery and warnings into domain producers.
- [ ] Delay a shared trait until at least dashboard, datasource, and access prove the same producer interface.

Validation:

- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet status`.
- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet project_status`.
- [ ] Run `make quality-architecture`.

## P1 - HTTP Transport Efficiency

### Improve Live Grafana Request Throughput

Problem:

Rust HTTP handling is reliable and centralized, but current live/export/status
paths are conservative for large Grafana instances. `JsonHttpClient` reuses one
reqwest blocking client, which is good, but successful responses are fully read
and converted to `String` before JSON parsing, response compression is disabled
with `Accept-Encoding: identity`, HTTP/2 is disabled with `http1_only()`, and
large inventory flows fetch dashboard details, templates, permissions, and
all-org scopes serially.

Relevant areas:

- `rust/src/grafana/http.rs`
- `rust/src/grafana/api/dashboard.rs`
- `rust/src/grafana/api/sync_live_read.rs`
- `rust/src/commands/dashboard/export_support.rs`
- `rust/src/commands/status/live.rs`
- `rust/src/commands/status/live_multi_org.rs`

Action:

- [ ] In `request_json`, avoid converting successful response bodies to `String`; keep error response text for diagnostics only.
- [ ] Re-evaluate `Accept-Encoding: identity`; prefer reqwest-managed gzip, brotli, and deflate unless a Grafana compatibility case proves this unsafe.
- [ ] Re-evaluate `.http1_only()` and allow HTTP/2 when the server/proxy supports it.
- [ ] Add bounded concurrency for dashboard detail fetches after `/api/search`.
- [ ] Add bounded concurrency for alert template detail fetches.
- [ ] Add bounded concurrency for dashboard/folder permission export fetches.
- [ ] Keep write/apply paths serial unless dependency ordering and Grafana API safety are explicitly modeled.
- [ ] In all-org status/list paths, avoid rebuilding scoped clients or re-reading the same live inputs more than needed; prefer one scoped read pass per org/domain boundary.
- [ ] Reduce `serde_json::Value` cloning only at proven hot spots; keep flexible JSON handling where the API shape varies by Grafana version.

Validation:

- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet http`.
- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet sync_live`.
- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet dashboard`.
- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet status`.
- [ ] Run live smoke against a fixed local Grafana container before changing concurrency defaults.

## P2 - Live Apply Safety

### Split Sync Live Apply By Phase

Problem:

`rust/src/grafana/api/sync_live_apply.rs` is a high-risk live mutation path and remains large.

Action:

- [ ] Split request builders into a phase-specific module.
- [ ] Split dependency ordering into a phase-specific module.
- [ ] Split apply execution into a phase-specific module.
- [ ] Keep API behavior unchanged.
- [ ] Add focused tests around ordering and the next split boundary if missing.
- [ ] Start with one phase boundary, then reassess. Do not split every phase in a single pass if the parent control flow becomes harder to follow.

Validation:

- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet sync_live`.
- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet apply`.
- [ ] Run `make quality-sync-rust`.

### Standardize Mutation Review Envelopes

Problem:

Dashboard, datasource, access, alert, and workspace mutation flows each have review/dry-run/apply concepts, but envelopes are still domain-shaped.

Action:

- [ ] Introduce a shared `ReviewAction` concept.
- [ ] Introduce a shared `ReviewRisk` concept.
- [ ] Introduce a shared `ReviewRequest` concept.
- [ ] Introduce a shared `ReviewApplyResult` concept.
- [ ] Introduce a shared `ReviewBlockedReason` concept.
- [ ] Keep domain-specific payloads behind a shared review wrapper.
- [ ] Avoid changing public JSON contracts until a migration path is defined.
- [ ] Start with one internal model or adapter. Do not force all domains to adopt the envelope in the first commit.

Validation:

- [ ] Run domain-focused tests first.
- [ ] Run full `cargo test --manifest-path rust/Cargo.toml --quiet` after shared envelope changes.
- [ ] Run `make quality-output-contracts` if JSON output changes.

## P2 - Contract Depth And Schema Governance

### Reconcile Output Contracts And Schema Manifests

Problem:

There are two contract systems:

- `scripts/contracts/output-contracts.json`
- `schemas/manifests` plus `scripts/generate_schema_artifacts.py`

Action:

- [ ] Define output contract ownership as runtime golden JSON artifacts and regression gates.
- [ ] Define schema manifest ownership as published schema/help contract.
- [ ] Promote only stable public artifacts from output contracts into schema manifests.
- [ ] Document promotion criteria in `docs/internal/contract-doc-map.md`.

Validation:

- [ ] Run `make quality-output-contracts`.
- [ ] Run `make schema-check`.
- [ ] Run `make quality-docs-surface`.

## P2 - Dashboard Review Model Completion

### Wire Review Source Model Into Remaining Dashboard Paths

Problem:

`review_source.rs` now models export-tree, saved-artifact, and live review inputs for topology/impact/policy. Some dashboard summary/help/internal names still use inspection/analysis vocabulary where the concept is really review or summary.

Action:

- [ ] Audit dashboard modules for stale user-facing `analysis` wording.
- [ ] Keep true query analyzer internals as analyzer names.
- [ ] Route any remaining policy/topology/impact source resolution through `review_source`.
- [ ] Add tests around saved-artifact vs live/export source selection.
- [ ] Do not rename internal analyzer modules that really parse query language or query family behavior.

Validation:

- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet topology`.
- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet governance_gate`.
- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_cli_inspect_help`.

## P3 - Docs And Generated Surface Discipline

### Keep Public Command Wording Consistent

Problem:

The project has intentionally moved away from stale `dashboard analyze` naming. Future command docs and help text can drift back unless wording stays guarded.

Action:

- [ ] Keep removed public paths in `scripts/contracts/command-surface.json`.
- [ ] Keep docs checks rejecting removed public paths outside archive/trace contexts.
- [ ] Prefer `dashboard summary` for live dashboard review.
- [ ] Prefer `dashboard dependencies` for local/export dependency review.
- [ ] Use `query analyzer` only for true internal analyzer code.

Validation:

- [ ] Run `make quality-docs-surface`.
- [ ] Run `make quality-ai-workflow`.
- [ ] Run targeted `rg` search for removed public paths.

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
  - [ ] Use `refactor:` for behavior-preserving Rust splits.
  - [ ] Use `test:` for contract/test coverage.
  - [ ] Use `docs:` for maintainer docs and generated docs.
  - [ ] Use `bugfix:` only for real behavior fixes.
- For public CLI/help/docs changes, run:
  - [ ] Run `make quality-docs-surface`.
  - [ ] Run `make man-check`.
  - [ ] Run `make html-check`.
- For output JSON changes, run:
  - [ ] Run `make quality-output-contracts`.
- For broad Rust refactors, run:
  - [ ] Run `cargo fmt --manifest-path rust/Cargo.toml --all --check`.
  - [ ] Run focused Rust tests.
  - [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet`.
  - [ ] Run `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`.
