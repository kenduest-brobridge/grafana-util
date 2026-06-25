# Project Maintainability Audit Plan

**Goal:** Assess whether the project remains within a maintainable range, with attention to over-decomposition, architecture boundaries, Rust/Python ownership, CLI/docs coupling, and test reliability.

**Scope:**
- Primary surface: `rust/src/`, `rust/Cargo.toml`, and Rust tests.
- Secondary surface: `python/grafana_utils/`, `python/tests/`, and packaging contracts only where they still affect architecture or parity.
- Documentation and generated surfaces: `README.md`, `docs/`, `scripts/contracts/`, generated man/html freshness expectations, and TODO hygiene.
- Excluded: generated build output such as `rust/target`.

**Questions:**
- Are modules split along useful domain boundaries, or has decomposition become hard to navigate?
- Are CLI, runtime, docs, and test responsibilities duplicated in ways that create drift?
- Is the Rust-first migration state clear, or are Rust and Python ownership boundaries ambiguous?
- Are tests and contracts strong enough to protect user-facing behavior?
- Are there architecture risks that should be fixed before adding more features?

**Constraints and Assumptions:**
- This is an audit-only pass unless a separate implementation request follows.
- Use CodeGraph for structural questions and native search/read commands for literal text and file inventory.
- Prefer evidence from current repository state over previous memory; previous memory only guides areas worth checking.
- Keep findings actionable and severity-ranked.

**Execution Plan:**
- [x] Inventory repository structure, major Rust modules, tests, docs contracts, and Python legacy surface.
- [x] Use CodeGraph to inspect structural boundaries around CLI dispatch, command modules, HTTP/client code, and dashboard/access surfaces.
- [x] Review file-size and test distribution for maintainability pressure, especially signs of over-splitting or duplicated ownership.
- [x] Run narrow non-mutating validation commands where useful for confidence.
- [x] Summarize findings with severity, evidence, architectural interpretation, and recommended next actions.

**Evidence Collected:**
- CodeGraph index healthy: 1,024 indexed files, 20,710 nodes, 59,390 edges.
- Rust source inventory: 798 Rust files under `rust/src`; dashboard is the largest domain with 327 files and about 94k lines.
- Test distribution: 212 Rust test-ish files, about 26.6% of Rust files.
- Guardrails passed: `make quality-architecture`, `make quality-workspace-noise`, `make quality-docs-surface`, `make quality-ai-workflow`, `make fmt-rust-check`.
- Rust tests passed: `make test-rust` ran 1,825 tests successfully.
- Lint did not pass: `make lint-rust` fails because `rust/src/commands/dashboard/export_scope.rs` has an unused `export_dashboards_in_scope_with_permission_fetcher` function under `-D warnings`.
- Follow-up fix: the unused dashboard export-scope wrapper and stale re-export were removed, and low-value HTTP struct comments were replaced with behavior-oriented API notes.
- Post-fix verification passed: `make fmt-rust-check`, `make lint-rust`, `cargo test --manifest-path rust/Cargo.toml --quiet export_scope`, `make test-rust`, `make quality-architecture`, `make quality-ai-workflow`, `make quality-docs-surface`, and `git diff --check`.

**Acceptance Checks:**
- [x] The final answer clearly states whether the project is still in a normal maintainability range.
- [x] Findings include file references and distinguish confirmed risks from weaker observations.
- [x] The answer covers over-decomposition, architecture boundaries, Rust/Python split, docs/CLI drift risk, and tests.
- [x] `docs/todo/todo.md` is updated with the audit status.
