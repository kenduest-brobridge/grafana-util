# TODO

Current maintainer backlog for the Rust-first `grafana-util` project.

Last cleaned: 2026-06-20.

## Scope

- Treat `rust/src/` as the primary implementation surface.
- Touch Python only for packaging, install behavior, explicit parity work, or
  repository tooling that is already in scope.
- Keep README changes out of this backlog unless a task explicitly targets
  public GitHub positioning.
- Prefer small grouped commits with focused validation.
- Keep this root file short. Put detailed specs under `docs/todo/`.

## Current Baseline

- Branch is `dev`.
- Release `v0.11.0` is cut at `6ab7ab03`.
- `dev` and `main` include the post-release Grafana 13 datasource API CI fix at
  `18f6f355`.
- GitHub Actions `rust-quality` and `rust-live-smoke` were green for
  `18f6f355`.
- Default Rust builds include `tui`; `browser` remains opt-in.
- `--no-default-features` is not claimed as a supported release artifact.
- Recent dashboard re-layering reduced the immediate directory-structure risk.
  Do not continue mechanical file splitting unless a fresh boundary review
  proves it is needed.
- Remaining risk is product and architecture alignment: TUI completion,
  dashboard/workspace ownership routing, operator docs, and cross-domain
  review/status consistency.

## Active Priority

### P0 - TUI Completion

Detailed requirements live in
[`docs/todo/tui-completion-spec.md`](docs/todo/tui-completion-spec.md).

Current order:

1. Datasource browse/list/snapshot: prove secret redaction and Review evidence
   parity across live, local, and snapshot details.
2. Dashboard browse/import/inspect: complete confirmation, scroll, search,
   footer, and source/provenance evidence coverage.
3. Access browse/plan/user/team: cover mutation confirmation, blockers, and
   shared review-detail rows where compatible.
4. Status overview/status TUI: tighten read-only search, detail scrolling,
   footer consistency, and docs parity.
5. Sync/workspace review: keep domain-owned; defer a shared workbench framework
   until more fields and semantics are proven.

Current status:

- [x] TUI registry exists and reports per-surface owner, tier, docs coverage,
  fallback, validation, and operator-friendliness notes.
- [x] Datasource list `--interactive` is registered and documented in English
  and zh-TW command docs.
- [x] Datasource/dashboard browse detail scrolling is clamped through
  focus-aware state helpers.
- [ ] Datasource live/local/snapshot secret-redaction tests still need a full
  pass across detail, diff, review, and confirmation panes.
- [ ] Dashboard import/inspect/browse still need confirmation, footer,
  search/repeat, and source/provenance regression coverage.
- [ ] Access, status, and workspace/sync surfaces still need blocker,
  footer/search/detail-scroll, and docs parity checks.

### P1 - Architecture Watchlist

- Dashboard/workspace ownership routing should stay evidence-led. Git
  Sync-managed and file-provisioned dashboard writes must remain blocked from
  direct live API apply unless a path explicitly proves otherwise.
- Keep shared mutation review adapters internal unless public JSON contract
  changes are explicitly planned.
- Keep `ReviewRisk` blocked until a non-dashboard mutation domain proves the
  same `severity`, `category`, and `recommendation` shape.
- Keep `ReviewRequest` blocked until at least two domains need the same request
  layer and fields.
- Keep live collection and multi-org transport outside the shared status
  producer trait; only feed stable domain-owned rows into shared status
  aggregation.
- Avoid new dashboard v2 support in classic raw/prompt/provisioning lanes until
  a dedicated adapter boundary and fixtures exist.
- Prefer product capability and operator evidence over further mechanical
  module reshaping.

### P2 - Product Surface Balance

- For every new dashboard intelligence feature, check whether datasource,
  access, alert, status, or workspace needs a minimal corresponding contract.
- Prefer shared review/status/output infrastructure before another
  dashboard-only surface.
- Keep backup/export use cases low-friction.
- Bring datasource browse closer to dashboard browse for safe operational
  review, but show only secret placeholder availability, blocker status, and
  review-required evidence.
- Add dashboard browse multi-select batch operations only after the
  selection/review model is explicit enough for export/import/delete safety.

## Completed Architecture Baseline

Keep this as a short historical checkpoint; do not expand completed checklists
back into the root TODO.

- Dashboard ownership/provenance is propagated through workspace
  source-bundle/preview/review paths.
- Workspace live apply blocks direct writes for file-provisioned and Git
  Sync-managed dashboard evidence.
- Dashboard import/plan has Git Sync ownership guardrails.
- Internal `ReviewMutationAction` adapters cover workspace, datasource plan,
  datasource import dry-run/live mutation preview, access import dry-run, alert
  plan rows, and selected apply-result evidence without changing public JSON.
- Status producer normalization exists only where domain-owned staged/live rows
  are already stable.
- Dashboard v2 input is rejected in classic lanes until a future adapter exists.
- Prompt conversion moved under `dashboard/export_prompt/`.

## Validation Gates

Run the smallest relevant test first, then broaden when the change crosses
domains or public docs.

For TUI/public docs/help changes:

- `rtk cargo test --manifest-path rust/Cargo.toml --quiet`
- `PYTHONPATH=. rtk python3 -m scripts.test_tui_inventory_report`
- `PYTHONPATH=. rtk python3 scripts/tui_inventory_report.py --registry-check`
- `rtk make quality-docs-surface`
- `rtk make quality-architecture`
- `rtk make man-check` and `rtk make html-check` when generated docs are in
  scope.

For output JSON changes:

- `rtk make quality-output-contracts`

For broad Rust refactors:

- `rtk cargo fmt --manifest-path rust/Cargo.toml --all --check`
- focused Rust tests for the touched domain
- `rtk cargo test --manifest-path rust/Cargo.toml --quiet`
- `rtk cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`

## Guardrails

- Do not inspect or edit `rust/target`.
- Do not modify README unless the task explicitly targets GitHub-facing
  positioning.
- Do not perform mechanical line-count splits without a responsibility-boundary
  review.
- Split by responsibility, not by file size alone.
- Keep the original file as a facade, routing point, or assembly point when it
  helps readability.
- Add at most 1-3 new modules per task unless splitting tests into obvious
  behavior groups.
- Do not introduce `utils`, `helpers2`, `misc`, or catch-all modules.
- Avoid shared traits or generic envelopes until at least two domains prove the
  same shape.
- Use grouped commits:
  - `bugfix:` for behavior fixes.
  - `feature:` for user-visible capability.
  - `refactor:` for behavior-preserving Rust structure changes.
  - `test:` for contract/test coverage.
  - `docs:` for maintainer docs and generated docs.
