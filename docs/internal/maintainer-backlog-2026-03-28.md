# Maintainer Backlog

Date: 2026-03-28
Scope: Rust runtime only
Audience: Maintainers
Status: Active follow-up backlog derived from the current architecture review, roadmap, and AI trace docs

## Purpose

This file turns the current architecture and roadmap direction into a short
working backlog.

It is intentionally narrower than `project-roadmap.md` and more action-oriented
than `architecture-review-2026-03-27.md`.

## Now

### 1. Finish dashboard dependency report output

Why now:

- This is the only item explicitly marked `In Progress` in current maintainer
  trace docs.
- It closes an already-open loop instead of starting another broad refactor.

Scope:

- finish typed dependency usage and orphan reporting
- complete the human-readable non-JSON renderer
- add the focused dashboard inspect tests that are still pending

Target files:

- `rust/src/dashboard_inspection_dependency_contract.rs`
- `rust/src/dashboard/inspect_output.rs`
- focused dashboard inspect tests

### 2. Start dashboard subsystem boundary cleanup

Why now:

- `dashboard` is now the clearest primary complexity center in the architecture
  review.
- More feature work will keep landing here unless ownership boundaries are made
  more explicit.

Scope:

- separate inspect pipeline ownership from governance evaluation
- keep interactive workbench logic from bleeding into unrelated paths
- continue shrinking orchestration facades instead of only splitting helper
  files

Target areas:

- `rust/src/dashboard/`
- related dashboard tests

### 3. Preserve current sync and promotion contract discipline

Why now:

- `sync` and promotion improved materially, but the wins are recent and easy to
  regress.
- Follow-on work should not collapse staged/live and review/apply boundaries
  back into broad facade modules.

Scope:

- keep staged document ownership explicit
- keep promotion contract additions attached to promotion modules
- avoid adding new behavior directly into `sync` orchestration facades unless
  ownership is already clear

Target areas:

- `rust/src/sync/`

## Next

### 1. Wire fuller datasource secret handling

Why next:

- secret handling is now the clearest remaining adoption gap
- the staged placeholder preflight contract is in place, but the workflow is
  still incomplete

Scope:

- formalize operator input for placeholder secret mappings
- wire datasource import and mutation paths to the staged secret contract
- make secret-missing and secret-loss cases explicit through later workflow
  stages, not only bundle-preflight

Target areas:

- `rust/src/datasource.rs`
- `rust/src/datasource_secret.rs`
- sync/apply integration points

### 2. Tighten public vs internal crate boundaries

Why next:

- `lib.rs` still exports a wider surface than the maintainers likely want to
  support long-term
- this is easier to tighten before more modules accumulate compatibility
  exposure

Scope:

- review public modules and compatibility re-exports
- reduce public exposure for internal-only helper paths where possible
- keep contract modules explicit and implementation helpers less visible

Target files:

- `rust/src/lib.rs`
- any modules currently exported only for convenience

### 3. Deepen inspection and governance outputs

Why next:

- inspection remains one of the strongest differentiators in the roadmap
- better dependency and governance reporting adds operator value without large
  product-scope drift

Scope:

- deepen datasource usage and orphan reporting
- add stronger governance and quality signals
- reuse the canonical inspection report model instead of adding parallel
  ad hoc outputs

Target areas:

- dashboard inspect and governance modules

## Later

### 1. Extend promotion from preflight into review/apply handoff

Why later:

- promotion is no longer missing, but it is still intentionally a skeleton
- current docs place dashboard boundaries and datasource secret wiring ahead of
  deeper promotion refinement

Scope:

- promotion review artifact
- resolved remap inventory
- warning vs blocking separation
- controlled handoff from promotion review into eventual apply

### 2. Expand promotion remap and prerequisite coverage

Why later:

- current promotion checks focus on folders and datasource references
- broader remap logic should build on the existing contract instead of widening
  too early

Scope:

- plugin prerequisites
- alert and contact-point prerequisites
- additional environment-specific rewrite visibility

### 3. Keep advanced analysis and packaging exploratory

Why later:

- these ideas are additive and not on the current correctness path
- they should only grow on top of the existing Rust analysis core

Scope:

- optional AI-assisted or rule-assisted analysis
- optional local packaging surfaces such as browser or WASM reuse

## Order Of Execution

If only a few slices move next, the recommended order is:

1. finish dashboard dependency report output
2. start dashboard subsystem boundary cleanup
3. wire fuller datasource secret handling
4. tighten crate boundaries
5. continue promotion review/apply work later

## Non-Goals For This Backlog

- no Python parity work
- no attempt to replace Terraform or Grafana provisioning
- no SaaS or controller-style expansion
- no broadening of `sync` resource scope before trust and review surfaces are
  stronger
