# Datasource Browse Render Split Plan

**Goal:** Split `rust/src/commands/datasource/browse/render.rs` into focused render helpers while keeping `render_datasource_browser_frame` as the stable entrypoint.

**Current Problem:**
- `scripts/rust_maintainability_report.py --root rust/src` reports `rust/src/commands/datasource/browse/render.rs` as an oversized production file at 852 lines.
- The file mixes frame assembly, header summary lines, list row rendering, detail/delete text, detail panels, review panels, focusable list rendering, utility helpers, and tests.

**Constraints:**
- Keep `render_datasource_browser_frame(frame, state)` as the only external entrypoint.
- Do not change TUI layout, labels, footer controls, delete confirmation text, review evidence text, or tests.
- Keep existing render tests in `render.rs` for this slice to avoid moving test modules and behavior in the same change.

**Planned Files:**
- Modify: `rust/src/commands/datasource/browse/render.rs`
  - Keep frame assembly, module declarations, imports, and tests.
- Create: `rust/src/commands/datasource/browse/render_summary.rs`
  - Own `summary_lines` and `blank_dash`.
- Create: `rust/src/commands/datasource/browse/render_list.rs`
  - Own datasource/org list row rendering and tree branch glyphs.
- Create: `rust/src/commands/datasource/browse/render_detail.rs`
  - Own delete/detail text, detail panel rendering, review panel lines, and focusable panel helper.

**Execution Steps:**
- [x] Capture red maintainability evidence: maintainability report flags `datasource/browse/render.rs`.
- [x] Move summary helpers to `render_summary.rs`.
- [x] Move list helpers to `render_list.rs`.
- [x] Move detail/review helpers to `render_detail.rs`.
- [x] Keep tests and entrypoint in `render.rs`.
- [x] Run focused datasource browse/render tests.
- [x] Confirm maintainability report has no oversized production files.
- [x] Run lint, architecture, and diff checks.

**Status:** Complete. `render.rs` keeps the frame renderer and tests; summary, list, and detail helpers now live in focused modules.

**Verification Evidence:**
- `cargo test --manifest-path rust/Cargo.toml --quiet datasource_browse`: 45 passed.
- `cargo test --manifest-path rust/Cargo.toml --quiet datasource_browse_render`: 14 passed.
- Review fix: source guard tests now scan `render.rs`, `render_detail.rs`, `render_list.rs`, and `render_summary.rs` so wrapper-drift checks still cover helpers after the split.
- Review fix verification: `cargo test --manifest-path rust/Cargo.toml --quiet datasource_browse_render`: 14 passed.
- `make lint-rust`: passed.
- `make quality-architecture`: passed.
- `scripts/rust_maintainability_report.py --root rust/src`: no oversized production files reported.

**Verification:**
- `cargo test --manifest-path rust/Cargo.toml --quiet datasource_browse`
- `cargo test --manifest-path rust/Cargo.toml --quiet datasource_browse_render`
- `python scripts/rust_maintainability_report.py --root rust/src`
- `make fmt-rust-check`
- `make lint-rust`
- `make quality-architecture`
- `git diff --check`
