# ai-changes.md

Current AI change log only.

- Older detailed history moved to [`archive/ai-changes-archive-2026-03-24.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-03-24.md).
- Detailed 2026-03-27 entries moved to [`archive/ai-changes-archive-2026-03-27.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-03-27.md).
- Detailed 2026-03-28 task notes were condensed into [`archive/ai-changes-archive-2026-03-28.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-03-28.md).
- Detailed 2026-03-29 through 2026-03-31 entries moved to [`archive/ai-changes-archive-2026-03-31.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-03-31.md).
- Detailed 2026-04-01 through 2026-04-12 entries moved to [`archive/ai-changes-archive-2026-04-12.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-04-12.md).
- Keep this file limited to the latest active architecture and maintenance changes.

## 2026-04-12 - Externalize docs entry taxonomy and add handbook command maps
- Summary: added `scripts/contracts/docs-entrypoints.json` as the shared definition file for landing quick commands, jump-select command entries, and handbook command-relationship maps; replaced the hard-coded Python metadata with a validating loader in `scripts/docgen_entrypoints.py`.
- User impact: the generated docs homepage now exposes a stable first-run path panel, jump navigation includes `version` and `config profile`, and handbook pages such as dashboard show grouped subcommand relationships in both the left nav and an in-page command map.
- Validation: `make html`; `make html-check`; `make quality-docs-surface`; `python3 -m unittest -v python.tests.test_python_docgen_entrypoints python.tests.test_python_docgen_command_docs python.tests.test_python_check_docs_surface`

## 2026-04-12 - Add docs surface contract and verifier
- Summary: introduced `scripts/contracts/command-surface.json` plus `scripts/check_docs_surface.py`, added `make quality-docs-surface`, routed AGENTS/maintainer docs to that contract, and corrected stale public docs that still taught removed roots or old alert command shapes.
- Test Run: `python3 scripts/check_docs_surface.py`; `make man`; `make html`; `make man-check`; `make html-check`; `make quality-docs-surface`; `git diff --check`.
- Follow-up: when public command paths, legacy replacements, command-doc routing, or `--help-full` support change, update the command surface contract first and keep shell fenced examples as the only verifier-owned executable doc examples.

## 2026-04-12 - Split production Rust modules and clean root artifacts
- Summary: split the sync, alert CLI, alert support, dashboard history, dashboard browse, and datasource import/export Rust surfaces into smaller owning modules, then removed the stale tracked root artifacts left in `rust/`.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cargo check --manifest-path rust/Cargo.toml --lib --quiet`; `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`; `cargo test --manifest-path rust/Cargo.toml --quiet -- --test-threads=1`; `make man-check`; `make html-check`; `make quality-ai-workflow`; `make quality-architecture`; `make quality-workspace-noise`; `git diff --check`.
- Follow-up: the Rust architecture lint now treats `sync/mod.rs` as handled instead of a known-debt warning, and the current hotspot list is narrowed to the remaining large ownership candidates.

## 2026-04-12 - Add docs architecture guardrails for manual stability
- Summary: introduced a docs-layer boundary doc that keeps handbook/manual content focused on stable intent and workflows, command docs focused on flags and syntax, generated docs derived, and trace docs concise.
- Follow-up: task briefs now carry a docs-impact matrix so agents can update the right docs layer without dragging manuals into command-reference detail.

## 2026-04-12 - Tighten AI workflow task brief and trace rules
- Summary: expanded the task brief template with owned-layer, source-of-truth, contract impact, test strategy, and generated-doc impact fields, and added an architecture consistency pass to the AI workflow note.
- Follow-up: architecture or large-file work should check the current guardrails and owning docs before editing, so future refactors stay rule-driven instead of ad hoc.

## 2026-04-12 - Split CLI dispatch and domain runtime spines
- Summary: moved parsed unified CLI routing into `cli_dispatch.rs`, kept parser topology in `cli.rs`, and removed the binary-level dashboard help bypass by routing dashboard leaf and `--help-full` output through the unified help preflight.
- Runtime Shape: dashboard and datasource command execution now live in `dashboard/command_runner.rs` and `datasource_runtime.rs`; alert, datasource, and workspace long help text moved into dedicated help text modules.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all`; `cargo test --manifest-path rust/Cargo.toml --quiet cli_rust_tests -- --test-threads=1`; `cargo test --manifest-path rust/Cargo.toml --quiet -- --test-threads=1`; `make quality-ai-workflow`; `git diff --check`.
- Follow-up: new command work should extend parser topology, dispatch decisions, help routing, and domain execution in their owning modules rather than adding one-off binary or facade branches.

## 2026-04-12 - Split CLI help into focused modules
- Summary: kept `cli_help.rs` as a small facade and moved grouped short-help specs/rendering, contextual routing, schema-help routing, and legacy command hints into focused modules under `rust/src/cli_help/`.
- Test Run: covered by the CLI help suite plus full Rust tests before the later dispatch/runtime split.
- Follow-up: if contextual routing grows again, split option-heading inference into its own module instead of moving logic back into the facade.

## 2026-04-12 - Support unique-prefix CLI subcommands
- Summary: enabled Clap inferred-subcommand behavior and canonicalized custom help preflight paths through the Clap command tree, avoiding manual abbreviation tables.
- Test Run: focused CLI regressions for inferred root help, nested dashboard help, ambiguous prefixes, colored grouped help, and parser dispatch.
- Follow-up: keep public docs on canonical full command names; add explicit aliases only when they are deliberate product decisions.
