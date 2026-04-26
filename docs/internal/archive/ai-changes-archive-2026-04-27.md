# ai-changes-archive-2026-04-27

## 2026-04-20 - Clean up alert runtime schema keys
- Summary: grouped alert runtime plan, delete-preview, and import dry-run document keys behind local schema namespaces while preserving Grafana raw alert payload reads.
- Tests: preserved alert plan row summaries, plan execution reads, delete preview output, import dry-run summaries, and existing readable JSON fixtures.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet alert --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet runtime --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet sync --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet import --lib`; `cargo test --manifest-path rust/Cargo.toml --quiet import_validation --lib`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `python3 scripts/rust_maintainability_report.py`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-ai-workflow`; `git diff --check`.
- Impact: `rust/src/commands/alert/runtime_support.rs`, `todo.md`, and AI trace docs. Public CLI behavior, generated docs, Python implementation, and output contracts are intentionally unchanged.
- Rollback/Risk: low mechanical key centralization. Rollback would restore repeated raw alert runtime document keys; focused alert/runtime and full Rust tests cover the touched paths.
