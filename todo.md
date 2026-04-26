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
- Recent Rust re-layering has reduced the immediate dashboard directory
  structure risk across browse, inspect workbench, governance gate, live status,
  and topology. Do not continue fine-grained file splitting unless a fresh
  responsibility-boundary review proves it is needed.
- Remaining risk is now mostly product and architecture alignment: broader
  dashboard source ownership/Git Sync routing, operator docs, and cross-domain
  balance.

## First Priority - Architecture Deficit Audit

This is the current first-priority backlog. Treat it as the ordering lens
before taking new cleanup work. The next phase should favor product/architecture
capability over more mechanical module reshaping.

Observed gaps:

- [x] Dashboard remains the heaviest product domain, but the next risk is not
  file size. The main risk is source ownership: API-managed dashboards,
  file-provisioned dashboards, and Git Sync-managed dashboards must route to the
  correct review/apply path. The first read/review route now propagates
  ownership/provenance through workspace source-bundle, preview, and review
  output; workspace live-apply now blocks direct writes for file-provisioned and
  Git Sync-managed dashboard evidence.
- [ ] Grafana 13 Git Sync ownership is now guarded in dashboard import/plan
  paths. Remaining Git Sync work is broader dashboard/workspace source routing,
  export layout, and operator docs, not the direct-write safety guard.
- [ ] Crate-root and domain facade routing should stay stable. Avoid moving
  shared surfaces to crate root unless they are already proven across domains
  and documented as shared architecture.
- [ ] TUI/browser feature surfaces are broad. Default `tui` and optional
  `browser` builds are supported release lanes, so any TUI/browser-adjacent
  change must validate the feature matrix, not just default tests.
- [x] Live read throughput has bounded fan-out for dashboard details, alert
  templates, dashboard/folder permission export reads, and a shared
  dashboard/datasource all-org read pass. Live status diagnostics now preserve
  useful read failures; remaining transport risk is proven hot spots only.
- [x] Mutation review envelopes remain domain-shaped. A shared internal adapter
  should be introduced only after workspace plus one concrete domain prove the
  same action/status/reason/risk shape. Workspace and datasource plan now share
  an internal `ReviewMutationAction` projection without adding a public
  datasource `review` field.
- [ ] Production assumptions need opportunistic cleanup. Most `unwrap`,
  `expect`, and `panic` occurrences are tests or hard-coded regex assertions,
  but touched live/operator paths should prefer `Result` errors over panic.

First-priority handling order:

- [x] First complete a dashboard source-ownership matrix across import, plan,
  export/layout, workspace, live inventory, and docs.
- [x] Then implement the smallest missing source-ownership route, preferably a
  read/review path before a write/apply path.
- [x] Then introduce one internal mutation-review adapter over an already-stable
  domain output without changing public JSON.
- [x] Defer dashboard v2 and broad shared-status rewrites until source
  ownership and review adapters are stable. Dashboard v2 now has a central
  classic-lane rejection guardrail; broad shared-status rewrites remain deferred.

## Active Execution Queue

Run the next development passes in this order unless a CI failure or user report
changes priority.

- [x] P0: Build the dashboard source-ownership matrix. Record current behavior
  and missing ownership evidence for import, plan, export layout,
  workspace/sync, live inventory, and operator docs.
- [x] P0: Extend one missing source-ownership read/review path. Prefer
  workspace source-bundle/preview ownership evidence before direct live writes.
- [x] P1: Add an internal mutation-review adapter for workspace plus one domain
  plan/review output. Start with datasource plan internal projection and do not
  change public JSON contracts in the first pass.
- [x] P1: Clean up workspace/status wording drift while preserving schema and
  compatibility names where they describe existing wire shapes.
- [x] P1: Improve live status diagnostics before changing read concurrency.
  Preserve deterministic output ordering and keep write/apply/import serial.
- [x] P2: Revisit dashboard v2 as a separate adapter boundary. Continue
  rejecting v2-shaped input in the classic prompt lane until fixtures and tests
  prove a clean migration path.
- [ ] P1: Normalize the status producer model only where a domain-owned signal
  already exists and can feed shared `status` aggregation without moving live
  collection into the shared trait.
- [ ] P2: Perform more dashboard directory re-layering only after the
  pre-split checklist proves a mixed-responsibility hotspot remains.

## Next Architecture Checklists

Use these checklists for workers. Each item should be a focused commit group
with narrow validation and a final full Rust test run when code changes.

### P0 - Dashboard Source Ownership Matrix

- [ ] Inventory existing ownership evidence in `dashboard/import/target.rs` and
  identify every caller that consumes `ownership=...` evidence.
- [ ] Check dashboard import/apply direct-write behavior for API-managed,
  file-provisioned, Git Sync-managed, and unknown-managed targets.
- [ ] Check dashboard plan behavior for the same ownership classes.
- [ ] Check export/layout conversion behavior for Git Sync tree input and
  whether the output can be reviewed without pretending it is an API export.
- [ ] Check workspace/sync dashboard apply paths for missing ownership
  evidence before live writes.
- [ ] Check live inventory/review outputs for ownership/provenance visibility.
- [ ] Check operator docs/help for clear routing: Git Sync targets go through
  repository/PR workflow; API-managed targets may use direct API apply.
- [x] Produce a short implementation order with one read/review gap first and
  one write/apply gap later.
- [x] First implementation: propagate dashboard ownership/provenance from
  dashboard export indexes into workspace source-bundle/preview specs and
  review output.
- [x] Later implementation: add dashboard ownership preflight to workspace live
  apply before POST/DELETE, reusing dashboard import/plan ownership semantics.

### P0 - Source Ownership Implementation

- [ ] Add or extend typed ownership evidence instead of passing ad hoc strings
  where a stable model already exists.
- [x] Preserve export index `ownership` and `provenance` when normalizing
  dashboard bundle items for workspace/source-bundle review.
- [x] Surface ownership/provenance in workspace preview/review before changing
  live write behavior.
- [x] Keep direct write blocked by default for file-provisioned and Git
  Sync-managed dashboards.
- [x] Keep managed-unknown as warning unless a path proves it must be blocked.
- [x] Preserve existing API-managed import/apply behavior.
- [x] Add tests for all ownership classes before changing write behavior.
- [ ] If output JSON changes, update contracts/fixtures and run
  `make quality-output-contracts`.

### P1 - Workspace / Status / Overview Boundary

- [x] Audit current public/user-facing references to `workspace`, `status`,
  `status overview`, `sync`, and `project-status`.
- [x] Clean up user-facing drift: avoid `project status` and `staged sync`
  wording in normal help/docs unless the text is explicitly schema or
  compatibility-related.
- [x] Keep schema/contract references to `project-status`,
  `grafana-util-project-status`, and `grafana-utils-sync-*` when they describe
  existing wire shapes.
- [x] Keep `workspace` as the public staged change workflow surface.
- [x] Keep `sync` as internal runtime/JSON compatibility naming unless a
  deliberate contract migration is planned.
- [x] Keep shared staged/live aggregation under `status`, not `overview`.
- [x] Keep `overview` as human-first projection and handoff surface.
- [x] When replacing stale docs/help, use public terms only unless the text is
  explicitly maintainer-only or compatibility-related.

### P1 - Internal Mutation Review Adapter

- [x] Pick one existing normalized shape as the seed, likely
  `ReviewMutationAction`.
- [x] Map workspace review actions into the adapter without changing public JSON.
- [x] Map datasource plan actions into an internal `ReviewMutationAction`
  projection without adding a public `review` field to datasource plan JSON.
- [x] Keep `raw` payload available for domain-specific evidence.
- [ ] Add `risk` only after real risk evidence exists in at least two domains.
- [ ] Avoid introducing public `ReviewRequest` until two mutation-review domains
  prove the same request fields.
- [ ] Add adapter tests that assert action, status, reason, identity, ordering,
  and blocked-reason behavior.

### P1 - Status Producer Model

- [ ] Keep domain-owned collection outside the shared producer trait.
- [ ] Keep multi-org live transport outside the shared producer trait.
- [ ] Feed domain-owned signals into shared `status` aggregation only after the
  domain has a stable staged or live status row.
- [ ] Avoid moving overview-specific projection into `status`.
- [ ] Add focused tests for any new domain producer before changing overview.

### P1 - Live Status Diagnostics And Read-Only Throughput

- [x] Identify the exact hot spot before changing concurrency or transport
  behavior.
- [x] Correct outdated assumptions before implementation: `JsonHttpClient`
  already uses `response.bytes()` plus `serde_json::from_slice`, compression is
  enabled through reqwest features, and no explicit `Accept-Encoding: identity`
  or `http1_only()` setting is present.
- [x] First implementation: preserve the first useful underlying HTTP/API error
  when live status all-org or dashboard/datasource read-pass paths fall back to
  `live-read-failed`.
- [x] Defer new all-org concurrency until diagnostics are clear and live smoke
  can confirm rate-limit behavior.
- [x] Preserve deterministic output ordering after concurrent reads.
- [x] Keep write/apply/import requests serial.
- [x] Use a conservative default concurrency constant and document why it is
  safe.
- [x] Add partial-failure tests that keep the first useful diagnostic visible.
- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet http`.
- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet sync_live`.
- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet dashboard`.
- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet status`.
- [ ] Run live smoke against a fixed local Grafana container before changing
  default concurrency.

### P2 - Dashboard v2 Adapter Boundary

- [x] Add focused tests proving v2 rejection coverage for raw import,
  provisioning import, dashboard plan raw/source, and dependency preflight.
- [x] Centralize v2 detection so raw-to-prompt, validate/import, plan, and
  provisioning lanes share one rejection rule.
- [x] Keep dashboard v2 rejected in classic prompt/raw/provisioning lanes until
  a dedicated adapter exists.
- [ ] Anchor fixtures to Grafana source testdata for datasource variables,
  selected current datasource handling, library panels, and v2 rejection.
- [ ] Keep provisioning as a derived projection, not the source-of-truth
  dashboard contract.
- [ ] Keep live library-panel `__elements` lookup limited to live export /
  import-handoff paths.
- [ ] Add adapter tests before allowing any v2-shaped import/export path.

### P2 - Dashboard Directory Re-layering

- [ ] Do not split files only because they are large.
- [ ] Use a fresh inventory before any later move.
- [ ] Choose exactly one mixed-responsibility boundary per commit.
- [ ] Use `git mv` for tracked moves.
- [ ] Keep `commands/dashboard/mod.rs` as the public facade.
- [ ] Keep public CLI/help/docs unchanged unless the task explicitly targets
  those surfaces.
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
- [x] Detect and surface dashboard ownership/provenance in live inventory and
  preflight evidence: API-managed, file-provisioned, or Git Sync-managed. Live
  inventory/review output now exposes provenance for non-write paths; keep Git
  Sync targets read-only for direct live dashboard writes by default.
- [ ] Add Git Sync-friendly layout support in dashboard export/convert,
  workspace scan/preview, and dashboard plan so repo trees can be reviewed
  without pretending they are ordinary live API targets.
- [x] Update dashboard import/apply docs and command guidance so Git Sync
  folders route changes to Git while normal unmanaged folders can still use API
  import/apply.
- [ ] Keep datasource, alert, access, and status workflows as direct product
  differentiators; Grafana Git Sync mainly changes dashboard JSON ownership, not
  datasource/access/alert lifecycle management.
- [ ] Keep live library-panel `__elements` lookup limited to the live export / import-handoff path. Keep local raw-to-prompt conversion warning-only when a referenced library panel model is missing.
- [ ] Keep prompt/export fixture parity anchored to Grafana source testdata for datasource variables, selected current datasource handling, library panels, and the classic-vs-v2 rejection boundary.
- [x] Extend the implemented dashboard import/plan ownership evidence into any
  remaining publish or workspace paths that still lack provenance before live
  writes.
- [x] Keep dashboard v2 as a separate future adapter boundary. Continue rejecting v2-shaped input in the classic prompt lane rather than mixing it into `raw/`, `prompt/`, or provisioning behavior.
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

Rust HTTP handling is reliable and centralized, and live/export/status paths now
have bounded read-only fan-out for the known repeated-read hot spots. Keep later
transport changes evidence-led: `JsonHttpClient` already parses from response
bytes, compression is provided by reqwest features, and no explicit
`Accept-Encoding: identity` or `http1_only()` setting is present.

Relevant areas:

- `rust/src/grafana/http.rs`
- `rust/src/grafana/api/dashboard.rs`
- `rust/src/grafana/api/sync_live_read.rs`
- `rust/src/commands/dashboard/export_support.rs`
- `rust/src/commands/status/live.rs`
- `rust/src/commands/status/live_multi_org.rs`

Action:

- [x] Keep write/apply paths serial unless dependency ordering and Grafana API safety are explicitly modeled.
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
- [x] Start with one internal model or adapter. Do not force all domains to adopt the envelope in the first commit.

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
