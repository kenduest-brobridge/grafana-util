# Rust Maintainability Refactor Backlog

**Goal:** Track large maintainability work that is justified by current evidence, without starting broad refactors before a concrete slice has a failing guardrail, clear ownership boundary, or repeated change pressure.

**Current Baseline:**
- `make quality-architecture` passes.
- `make test-rust` passes.
- `make lint-rust` failed before the local cleanup because `export_dashboards_in_scope_with_permission_fetcher` was unused.
- `scripts/rust_maintainability_report.py` currently flags three oversized production files:
  - `rust/src/commands/review_contract.rs` at 856 lines.
  - `rust/src/common/browser/session.rs` at 856 lines.
  - `rust/src/commands/datasource/browse/render.rs` at 852 lines.
- Domain size hotspots from the same report:
  - `rust/src/commands/dashboard`: 327 files, 94k lines.
  - `rust/src/commands/access`: 109 files, 32k lines.
  - `rust/src/commands/sync`: 83 files, 24k lines.
  - `rust/src/commands/datasource`: 83 files, 24k lines.

**Non-Goals:**
- Do not rewrite the domain layout in one pass.
- Do not split files by line count alone.
- Do not move tests before production boundaries are clearer.
- Do not create new abstraction layers unless they remove concrete duplication or ownership ambiguity.

## Priority List

- [x] **P0: Close the confirmed lint blocker.**
  - Remove the unused dashboard export-scope wrapper and stale re-export.
  - Keep this as the only code change in the first slice unless verification reveals another direct gate failure.
  - Acceptance: `make lint-rust`, `make fmt-rust-check`, focused dashboard export-scope tests, and `git diff --check` pass.
  - Status: removed `export_dashboards_in_scope_with_permission_fetcher`, removed its stale re-export, and replaced low-value HTTP struct comments with useful API notes.
  - Verification: `make fmt-rust-check`, `make lint-rust`, `cargo test --manifest-path rust/Cargo.toml --quiet export_scope`, `make test-rust`, `make quality-architecture`, `make quality-ai-workflow`, `make quality-docs-surface`, and `git diff --check` pass.

- [x] **P1: Split `review_contract.rs` by contract responsibility.**
  - Current risk: shared review/action vocabulary, mutation action projection, browser/detail lines, and review envelope helpers live in one large shared file.
  - Proposed boundary:
    - `review_contract/actions.rs`: action/status/reason constants, ranks, grouping.
    - `review_contract/model.rs`: `ReviewMutationAction`, `ReviewMutationActionInput`, blocked reason model.
    - `review_contract/detail.rs`: detail-line and browser-facing projections.
    - Keep `review_contract.rs` as a thin facade and re-export boundary.
  - Acceptance: review contract tests pass, access/datasource/sync plan tests that consume review actions pass, `make quality-architecture` stays green.
  - Status: complete; `review_contract.rs` is a stable facade over `actions`, `model`, `detail`, and `envelope` submodules.

- [x] **P1: Split `common/browser/session.rs` into state, render, and terminal runtime.**
  - Current risk: shared read-only browser item model, search state, event loop, and TUI rendering are concentrated in one common file.
  - Proposed boundary:
    - `common/browser/session/model.rs`: `BrowserItem`, pane/search enums, search state.
    - `common/browser/session/search.rs`: query matching and repeat navigation.
    - `common/browser/session/render.rs`: frame/detail/footer rendering.
    - `common/browser/session/runtime.rs`: terminal setup, event loop, teardown.
    - Keep existing public module path stable through `common/browser/session.rs`.
  - Acceptance: browser session tests pass, TUI-gated compile remains green, `make quality-rust-feature-matrix` still reflects supported feature policy.
  - Status: complete; `session.rs` is a stable facade over `model`, `search`, `render`, and `runtime` submodules.

- [x] **P1: Split `datasource/browse/render.rs` by panel.**
  - Current risk: datasource browser frame, header summary, list rows, details, and overlays are in one render file.
  - Proposed boundary:
    - `datasource/browse/render_frame.rs`: frame assembly and panel layout.
    - `datasource/browse/render_list.rs`: org/datasource list row rendering.
    - `datasource/browse/render_detail.rs`: detail/review panel rendering.
    - `datasource/browse/render_overlay.rs`: delete/edit/search overlays if not already fully covered by chrome/dialog modules.
    - Keep `render.rs` as a thin facade until call sites are stable.
  - Acceptance: datasource browse render tests pass, `make quality-architecture` stops flagging this file as oversized.
  - Status: complete; `render.rs` now delegates summary, list, and detail rendering helpers to focused modules.

- [x] **P2: Dashboard domain inventory before any dashboard-wide refactor.**
  - Current risk: `dashboard` is the largest domain, but architecture guardrails pass and the biggest files are mostly tests.
  - Required inventory before code changes:
    - Map `dashboard/export_*`, `history_*`, and `browse/*` ownership and call paths.
    - Identify any file mixing workflow, contract, render, and IO.
    - Prefer one narrow slice, such as export/history helpers or browse state/render tests, over a domain-wide move.
  - Acceptance: produce a dated TODO/spec with exact files, planned moves, test list, and rollback path before editing code.
  - Status: inventory complete in `2026-06-25-domain-maintainability-inventory.md`; no dashboard-wide refactor is justified without a narrower trigger.

- [x] **P2: Access domain inventory before resource-kind refactors.**
  - Current risk: access has many resource-specific files and large user/team/service-account tests.
  - Required inventory before code changes:
    - Verify whether user/team/service-account workflows duplicate import/export/diff structure enough to justify a shared helper.
    - Do not merge resource-specific behavior into a generic abstraction unless duplicate control flow is proven.
  - Acceptance: produce a focused plan for one resource family or one shared import/export helper boundary.
  - Status: inventory complete in `2026-06-25-domain-maintainability-inventory.md`; generic access abstraction is not justified yet.

- [x] **P2: Sync domain test slicing only after production ownership is stable.**
  - Current risk: sync has large live/apply/review tests, but production files are under current architecture thresholds.
  - Required inventory before code changes:
    - Keep `sync/mod.rs` as a facade.
    - Avoid moving tests solely to lower line counts.
    - Split tests only when a production contract boundary is also becoming clearer.
  - Acceptance: focused sync tests and `make quality-sync-rust` remain green for each slice.
  - Status: inventory complete in `2026-06-25-domain-maintainability-inventory.md`; no sync test-only split is justified yet.

- [x] **P3: Keep Python legacy read-only unless a task explicitly targets parity or packaging.**
  - Current risk: Python remains large and can distract new fixes away from the supported Rust surface.
  - Acceptance: new user-facing behavior changes land in Rust first; Python edits require explicit scope.
  - Status: no Python changes were made in this Rust maintainability pass.

## Suggested Execution Order

1. Finish P0 lint cleanup.
2. Split `review_contract.rs`; it is shared and likely gives the best maintainability return.
3. Split `common/browser/session.rs`; it is common infrastructure and currently feature-gated in a way that deserves careful verification.
4. Split `datasource/browse/render.rs`; it is a concrete render-layer oversized file.
5. Re-run maintainability report and decide whether dashboard/access/sync need a new dated plan.

## Verification Matrix

- P0 lint cleanup:
  - `make fmt-rust-check`
  - `make lint-rust`
  - `cargo test --manifest-path rust/Cargo.toml --quiet export_scope`
  - `git diff --check`
- Shared review contract split:
  - `cargo test --manifest-path rust/Cargo.toml --quiet review_contract`
  - `cargo test --manifest-path rust/Cargo.toml --quiet access_plan`
  - `cargo test --manifest-path rust/Cargo.toml --quiet datasource_plan`
  - `cargo test --manifest-path rust/Cargo.toml --quiet sync`
- Shared browser session split:
  - `cargo test --manifest-path rust/Cargo.toml --quiet session`
  - `make quality-rust-feature-matrix`
  - `make lint-rust`
- Datasource browse render split:
  - `cargo test --manifest-path rust/Cargo.toml --quiet datasource_browse`
  - `make quality-architecture`
  - `make lint-rust`

## Planning Rule For Future Slices

Before implementing any P1/P2 item, create a separate dated plan under `docs/todo/` with exact file moves, expected imports/re-exports, red/green test commands, and rollback scope. Keep `docs/todo/todo.md` as the concise tracking index.
