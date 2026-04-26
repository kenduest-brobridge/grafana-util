# ai-changes-archive-2026-04-27

## 2026-04-20 - Clean up alert runtime schema keys
- Summary: grouped alert runtime plan, delete-preview, and import dry-run document keys behind local schema namespaces while preserving Grafana raw alert payload reads.
- Tests: preserved alert plan row summaries, plan execution reads, delete preview output, import dry-run summaries, and existing readable JSON fixtures.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet alert --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet runtime --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet sync --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet import --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet import_validation --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `python3 scripts/rust_maintainability_report.py`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-ai-workflow`; `git diff --check`.
- Impact: `rust/src/commands/alert/runtime_support.rs`, `todo.md`, and AI trace docs. Public CLI behavior, generated docs, Python implementation, and output contracts are intentionally unchanged.
- Rollback/Risk: low mechanical key centralization. Rollback would restore repeated raw alert runtime document keys; focused alert/runtime and full Rust tests cover the touched paths.

## 2026-04-20 - Move dashboard import into directory boundary
- Summary: moved dashboard import implementation files under `commands/dashboard/import/` while keeping `commands/dashboard/mod.rs` as the facade and leaving plan reconciliation under `commands/dashboard/plan/`.
- Tests: preserved import dry-run/apply lookup boundaries, routed import reporting, dashboard plan relationships, and browse interactive import coverage through existing regression suites.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet import --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet routed_import --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_plan --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet interactive_import --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `python3 scripts/rust_maintainability_report.py`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-ai-workflow`; `git diff --check`.
- Impact: `rust/src/commands/dashboard/mod.rs`, `rust/src/commands/dashboard/import/`, `todo.md`, and AI trace docs. Public CLI behavior, generated docs, Python implementation, and output contracts are intentionally unchanged.
- Rollback/Risk: low mechanical module move. Rollback would move the import files back to the flat dashboard directory and restore the root module declarations; focused import/routed/plan/browse and full Rust tests cover the moved paths.

## 2026-04-20 - Finish project status producer audit
- Summary: audited remaining project-status producers across sync, datasource, alert, dashboard, access, and live status fallback paths, then normalized the last dashboard live read-failure fallback through `StatusReading`.
- Tests: preserved live dashboard read failure status fields, blocker count derivation, and existing staged/live domain status evidence including health, version, discovery, and freshness paths.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet dashboard --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet access --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet status --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet project_status --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `make quality-architecture`; `make quality-ai-workflow`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `git diff --check`.
- Impact: `rust/src/grafana/api/project_status_live.rs`, `todo.md`, and AI trace docs. Public CLI behavior, generated docs, Python implementation, and output contracts are intentionally unchanged.
- Rollback/Risk: low test-support normalization. Rollback would restore the direct fallback `ProjectDomainStatus` literal; the shared model derives the same blocker and warning counts from the same data.
