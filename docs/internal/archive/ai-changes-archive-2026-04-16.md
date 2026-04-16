# ai-changes-archive-2026-04-16

## 2026-04-15 - Split access runtime user tests
- Summary: split user-focused runtime coverage out of `access_runtime_org_rust_tests.rs` into `access_runtime_user_rust_tests.rs`. The new module owns user diff routing, user diff count behavior, global user export/import/diff coverage, org user export/diff-with-teams coverage, and local user list input-dir routing.
- Tests: preserved existing access runtime assertions and moved tests without changing behavior.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cargo test --manifest-path rust/Cargo.toml --quiet access`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`; `make quality-architecture`; `git diff --check`.
- Impact: `rust/src/commands/access/access_runtime_org_rust_tests.rs`, `rust/src/commands/access/access_runtime_user_rust_tests.rs`, and AI trace docs. README files and Python implementation were intentionally left unchanged.
- Rollback/Risk: low test-only split. Rollback would move the user runtime tests back into the org runtime test module.
- Follow-up: continue remaining architecture-warning hotspots: dashboard browse support, dashboard dependency contract, datasource staged reading, datasource CLI mutation/tail tests, snapshot tests, sync live apply, and help-test semantic assertions.

## 2026-04-15 - Split snapshot review tests
- Summary: split snapshot staged-scope resolver tests and snapshot review wrapper/warning tests out of `snapshot/tests.rs` into focused sibling modules. The parent test module keeps shared fixture builders and broader snapshot export/review coverage.
- Tests: preserved existing snapshot assertions while moving staged export scope resolver coverage and review warning/wrapper coverage to dedicated modules.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cargo test --manifest-path rust/Cargo.toml --quiet snapshot`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`; `make quality-architecture`; `git diff --check`.
- Impact: `rust/src/commands/snapshot/tests.rs`, `rust/src/commands/snapshot/tests_staged_scopes.rs`, `rust/src/commands/snapshot/tests_review_warnings.rs`, and AI trace docs. README files and Python implementation were intentionally left unchanged.
- Rollback/Risk: low test-only split. Rollback would move the staged-scope and review warning tests back into the parent snapshot test module.
- Follow-up: continue remaining architecture-warning hotspots: dashboard browse support, dashboard dependency contract, datasource staged reading, datasource CLI mutation/tail tests, sync live apply, and help-test semantic assertions.
