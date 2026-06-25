# Review Contract Split Plan

**Goal:** Split `rust/src/commands/review_contract.rs` into focused submodules while keeping the existing `crate::review_contract::*` import surface stable.

**Current Problem:**
- `scripts/rust_maintainability_report.py --root rust/src` reports `rust/src/commands/review_contract.rs` as an oversized production file at 856 lines.
- The file currently mixes:
  - action/status/reason/hint constants and ranking helpers,
  - mutation action and blocked reason models,
  - TUI/browser detail-line projection helpers,
  - envelope/domain/summary rows and apply-result helpers.

**Constraints:**
- Preserve current caller imports through `crate::review_contract`.
- Do not change JSON fields, action strings, status strings, ordering, or review evidence text.
- Keep `review_contract.rs` as a thin facade plus test module.
- Do not split tests in this slice.

**Planned Files:**
- Modify: `rust/src/commands/review_contract.rs`
  - Declare submodules and re-export the existing public crate-private surface.
  - Keep only facade comments and the existing `#[cfg(test)]` test module.
- Create: `rust/src/commands/review_contract/actions.rs`
  - Own review action/status/reason/hint constants and action/domain ranking helpers.
- Create: `rust/src/commands/review_contract/model.rs`
  - Own `ReviewMutationAction`, `ReviewMutationActionInput`, `ReviewBlockedReason`, and conversion logic.
- Create: `rust/src/commands/review_contract/detail.rs`
  - Own browser/TUI review detail, narrative, impact, change, target evidence, context, next-check, diff-preview, and evidence-section helpers.
- Create: `rust/src/commands/review_contract/envelope.rs`
  - Own `ReviewMutationDomain`, `ReviewMutationSummary`, `ReviewMutationEnvelope`, `ReviewApplyResult`, summary rows, and envelope builders.

**Execution Steps:**
- [x] Capture red maintainability evidence: `scripts/rust_maintainability_report.py --root rust/src` reports `review_contract.rs` as oversized.
- [x] Move constants and ranking helpers to `review_contract/actions.rs`.
- [x] Move mutation action models to `review_contract/model.rs`.
- [x] Move detail projection helpers to `review_contract/detail.rs`.
- [x] Move envelope/apply-result helpers to `review_contract/envelope.rs`.
- [x] Replace `review_contract.rs` body with module declarations and re-exports.
- [x] Run focused review contract tests.
- [x] Run access/datasource/sync review consumers.
- [x] Run maintainability report and confirm `review_contract.rs` is no longer oversized.
- [x] Run full verification gates listed below.

**Status:** Complete. `review_contract.rs` is now a facade; behavior stays behind the same `crate::review_contract` import path.

**Verification Evidence:**
- `cargo test --manifest-path rust/Cargo.toml --quiet review_contract`: 16 passed.
- `cargo test --manifest-path rust/Cargo.toml --quiet access_plan`: 21 passed.
- `cargo test --manifest-path rust/Cargo.toml --quiet datasource_plan`: 10 passed.
- `cargo test --manifest-path rust/Cargo.toml --quiet sync`: 284 passed.
- `make lint-rust`: passed.
- `scripts/rust_maintainability_report.py --root rust/src`: no longer reports `review_contract.rs`; remaining oversized files are `common/browser/session.rs` and `datasource/browse/render.rs`.

**Verification:**
- `cargo test --manifest-path rust/Cargo.toml --quiet review_contract`
- `cargo test --manifest-path rust/Cargo.toml --quiet access_plan`
- `cargo test --manifest-path rust/Cargo.toml --quiet datasource_plan`
- `cargo test --manifest-path rust/Cargo.toml --quiet sync`
- `python scripts/rust_maintainability_report.py --root rust/src`
- `make fmt-rust-check`
- `make lint-rust`
- `make quality-architecture`
- `git diff --check`
