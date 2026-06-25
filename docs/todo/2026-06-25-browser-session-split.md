# Browser Session Split Plan

**Goal:** Split `rust/src/common/browser/session.rs` into focused submodules while preserving the shared interactive browser behavior and existing facade path.

**Current Problem:**
- `scripts/rust_maintainability_report.py --root rust/src` reports `rust/src/common/browser/session.rs` as an oversized production file at 856 lines.
- The file mixes:
  - browser item and search state models,
  - search matching and repeat navigation,
  - terminal session setup/teardown,
  - TUI frame rendering,
  - keyboard event loop.

**Constraints:**
- Preserve `crate::interactive_browser::BrowserItem` and `run_interactive_browser` call paths.
- Keep shared detail helper re-exports stable.
- Keep no-TUI fallback behavior unchanged.
- Do not change key bindings, footer text, search wrapping, filter behavior, or detail scrolling.

**Planned Files:**
- Modify: `rust/src/common/browser/session.rs`
  - Keep facade, shared detail helper re-exports, no-TUI fallback, and tests.
- Create: `rust/src/common/browser/session/model.rs`
  - Own `BrowserItem`, `BrowserPane`, `SearchDirection`, `SearchPromptState`, `SearchState`, and `BrowserSearchController`.
- Create: `rust/src/common/browser/session/search.rs`
  - Own query matching, repeat search, search state construction, and search symbols.
- Create: `rust/src/common/browser/session/render.rs`
  - Own visual helpers and frame rendering for the TUI browser.
- Create: `rust/src/common/browser/session/runtime.rs`
  - Own terminal session setup/teardown and the event loop.

**Execution Steps:**
- [x] Capture red maintainability evidence: maintainability report flags `common/browser/session.rs`.
- [x] Move browser/search model types to `model.rs`.
- [x] Move search helper functions to `search.rs`.
- [x] Move frame rendering helpers to `render.rs`.
- [x] Move terminal session and event loop to `runtime.rs`.
- [x] Keep `session.rs` as facade and no-TUI fallback.
- [x] Run focused session tests.
- [x] Run feature matrix, lint, and architecture checks.
- [x] Confirm maintainability report no longer flags `common/browser/session.rs`.

**Status:** Complete. `session.rs` is now a facade; model/search/render/runtime behavior moved under `common/browser/session/`.

**Verification Evidence:**
- `cargo test --manifest-path rust/Cargo.toml --quiet browser`: 45 passed.
- `cargo test --manifest-path rust/Cargo.toml --quiet search`: 76 passed.
- Review fix: `cargo test --manifest-path rust/Cargo.toml --quiet --no-default-features session` first reproduced an unresolved `render` import from a test-only facade re-export; the facade and `detail_title` test were narrowed to `feature = "tui"`.
- Review fix verification: `cargo test --manifest-path rust/Cargo.toml --quiet --no-default-features session` compiles cleanly with 0 matched tests.
- `make lint-rust`: passed.
- `make quality-rust-feature-matrix`: passed.
- `scripts/rust_maintainability_report.py --root rust/src`: no longer reports `common/browser/session.rs`; remaining oversized file is `datasource/browse/render.rs`.

**Verification:**
- `cargo test --manifest-path rust/Cargo.toml --quiet session`
- `make quality-rust-feature-matrix`
- `make lint-rust`
- `make quality-architecture`
- `python scripts/rust_maintainability_report.py --root rust/src`
- `git diff --check`
