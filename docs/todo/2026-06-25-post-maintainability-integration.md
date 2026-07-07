# Post-Maintainability Integration Plan

**Goal:** Organize the current clean local `dev` branch after the maintainability pass, decide the next integration gate, and avoid starting another broad refactor without a concrete trigger.

**Current State:**
- Working tree is clean.
- Local `dev` is ahead of `origin/dev` by 26 commits.
- `docs/todo/todo.md` currently has no unchecked active dated item, but root `todo.md` still carries the active product backlog.
- The latest completed verification included Rust tests, lint, architecture checks, docs-surface checks, and whitespace checks.

## Ahead Commit Groups

### Product and TUI Capability

- `9aea6384 feature: add datasource interactive review output`
- `7fd8e91f feature: add resource API mode fallback`
- `39267a1e feature: add TUI surface registry with zh-TW scanning and registry check`
- `9678fd7e docs: register datasource list TUI`
- `d3eface6 bugfix: clamp browse TUI detail scrolling`

**Integration Risk:** Medium. These touch user-facing CLI/TUI behavior and should be covered by TUI inventory, docs surface, and focused datasource/dashboard browse tests before pushing.

### Runtime and Safety Fixes

- `446d365c bugfix: harden profile TLS and secrets`
- `268e14a3 bugfix: align access browse and snapshot review TUI feature fallback`
- `e41b1bf5 bugfix: classify numeric-suffix HTTP hosts`
- `9a52cd85 bugfix: remove stale dashboard export wrapper`

**Integration Risk:** Medium. These are good candidates for focused regression checks because they touch runtime safety, feature gating, and HTTP error classification.

### Performance

- `349901c0 feature: speed dashboard history export`

**Integration Risk:** Medium. Keep deterministic output/order checks and dashboard export/history tests in the final gate.

### Maintainability Refactors

- `cf3f94ac refactor: split status overview tui support`
- `8be6ac88 refactor: split datasource browse tui chrome`
- `eb9bee56 refactor: split datasource browse review support`
- `0a6774ee refactor: split review contract tests`
- `ef84d80e refactor: split shared browser session support`
- `8a5f038d refactor: remove stale dashboard browse helper`
- `d8d3357b refactor: split review contract modules`
- `3587514b refactor: split shared browser session`
- `dd5520d3 refactor: split datasource browse rendering`

**Integration Risk:** Low to medium. These should be behavior-preserving, but they are broad enough that feature-matrix and full Rust tests must stay in the gate.

### Docs and Tracking

- `296c2d97 docs: document resource API mode`
- `0fedc505 docs: mark Phase B complete in tui-completion-spec`
- `b8aff9f1 docs: update TUI completion backlog`
- `5dc37a58 docs: clean up completed TODO backlog`
- `19b9dfd0 docs: sync CLI command references`
- `3e02e138 docs: add completed todo index`
- `c62520a4 docs: record maintainability refactor plans`

**Integration Risk:** Low, but generated docs/man/html drift checks should be part of final validation because this branch changes command docs and generated references.

## Recommended Next Step

Do not start another code refactor first. The branch is large enough that the next action should be an integration-readiness pass:

1. Run final branch-level validation.
2. Fix only concrete failures from that validation.
3. Push or open a PR with grouped summary.
4. Start new coding only after the integration path is clear.

## Final Validation Gate

Run these before push/PR:

- `make fmt-rust-check`
- `make lint-rust`
- `make quality-architecture`
- `make quality-ai-workflow`
- `make quality-docs-surface`
- `make man-check`
- `make html-check`
- `PYTHONPATH=. python3 -m scripts.test_tui_inventory_report`
- `PYTHONPATH=. python3 scripts/tui_inventory_report.py --registry-check`
- `make quality-output-contracts`
- `make test`

If `rtk make test` is too broad for the local environment, record the exact blocker and fall back to:

- `make test-rust`
- focused datasource/dashboard/access/status tests matching changed areas
- docs/man/html checks for generated output

## Execution Results (2026-07-07)

Branch-level validation was executed against the current local `dev` worktree.
The worktree was not clean at execution start because the dashboard export
Git-friendly filename/history work was already present locally.

Concrete failures found and fixed:

- `make lint-rust` failed on an unnecessary lazy evaluation in
  `rust/src/commands/dashboard/clone_folder.rs`; changed `unwrap_or_else` to
  `unwrap_or`.
- `make man-check` and `make html-check` reported generated command docs drift
  after dashboard export help changes; regenerated `docs/man/*.1` and
  `docs/html/*`.

Validation passed after fixes:

- `make fmt-rust-check`
- `make lint-rust`
- `make quality-architecture`
- `make quality-ai-workflow`
- `make quality-docs-surface`
- `make man-check`
- `make html-check`
- `PYTHONPATH=. python3 -m scripts.test_tui_inventory_report`
- `PYTHONPATH=. python3 scripts/tui_inventory_report.py --registry-check`
- `make quality-output-contracts`
- `make test`

## Next Coding Queue After Integration

Use root `todo.md` as the product backlog source after integration. The next narrow coding slices should be:

1. **Datasource secret-redaction parity pass**
   - Scope: live/local/snapshot detail, diff, review, and confirmation panes.
   - Why first: it is the top unchecked root TODO item and directly matches recent datasource TUI/review changes.
   - Avoid: broad datasource module reshaping.

2. **Dashboard browse/import/inspect evidence coverage**
   - Scope: confirmation, footer, search/repeat, source/provenance regression tests.
   - Why second: dashboard is large, but the current trigger should be user-facing evidence coverage, not file count.
   - Avoid: dashboard-wide architecture rewrites.

3. **Access/status/workspace TUI parity**
   - Scope: blocker rows, footer/search/detail-scroll behavior, docs parity.
   - Why third: keep parity with shared browser/review contracts after datasource/dashboard evidence stabilizes.
   - Avoid: shared framework extraction until two domains prove the same semantics.

## Non-Goals

- Do not re-split files just because a directory is large.
- Do not touch Python unless a packaging/parity issue appears in validation.
- Do not update README unless the integration path explicitly needs public-facing release notes.
- Do not add a shared review/status abstraction without at least two domains proving identical fields and semantics.

## Acceptance Checks

- [x] `docs/todo/todo.md` points to this integration plan as the active next item.
- [x] Branch-level validation results are recorded before push/PR.
- [ ] Any fixes from validation are committed separately from this planning document.
- [ ] Worktree is clean before handing off, pushing, or opening a PR.
