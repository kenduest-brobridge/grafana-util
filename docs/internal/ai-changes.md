# ai-changes.md

Current AI change log only.

- Older detailed history moved to [`archive/ai-changes-archive-2026-03-24.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-03-24.md).
- Detailed 2026-03-27 entries moved to [`archive/ai-changes-archive-2026-03-27.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-03-27.md).
- Detailed 2026-03-28 task notes were condensed into [`archive/ai-changes-archive-2026-03-28.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-03-28.md).
- Detailed 2026-03-29 through 2026-03-31 entries moved to [`archive/ai-changes-archive-2026-03-31.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-03-31.md).
- Detailed 2026-04-01 through 2026-04-12 entries moved to [`archive/ai-changes-archive-2026-04-12.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-04-12.md).
- Keep this file limited to the latest active architecture and maintenance changes.
- Older entries moved to [`ai-changes-archive-2026-04-13.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-04-13.md).

## 2026-04-13 - Split Rust snapshot/import/live-status hotspots
- Summary: split `snapshot.rs` into focused CLI definition, lane-loading, count/warning, and review-document modules; changed snapshot review output assembly from one large `json!` object to module-local `Serialize` structs for the stable document contract; integrated worker splits for dashboard import lookup, dashboard inspect CLI definitions, and access live-status helpers.
- Tests: preserved existing behavior coverage and added no new public output changes.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all`; `cd rust && cargo test --quiet snapshot_rust_tests --no-run`; `cd rust && cargo test --quiet dashboard_import --no-run`; `cd rust && cargo test --quiet access_live_project_status --no-run`; `cd rust && cargo test --quiet`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `python3 scripts/rust_maintainability_report.py --root rust/src`.
- Impact: `rust/src/snapshot.rs`, new `rust/src/snapshot_cli_defs.rs`, `rust/src/snapshot_review_counts.rs`, `rust/src/snapshot_review_document.rs`, `rust/src/snapshot_review_lanes.rs`, `rust/src/dashboard/import_lookup*.rs`, `rust/src/dashboard/cli_defs_inspect*.rs`, and `rust/src/access/live_project_status*.rs`.
- Rollback/Risk: behavior-preserving module-boundary refactor; rollback would collapse helper modules back into their former large files. The snapshot review document is now constrained by internal serde structs plus existing tests, but there is still no external JSON Schema file.
- Follow-up: keep using the maintainability report to target remaining non-test hotspots, especially datasource project status/live status, `snapshot_support.rs`, dashboard browse/export/import-apply/project-status/topology, and sync preflight modules.

## 2026-04-13 - Reject credentials in Grafana base URLs
- Summary: added a Rust connection-resolution guard that rejects username/password userinfo in Grafana base URLs supplied through `--url`, `GRAFANA_URL`, or profile `url`, and points operators to explicit Basic auth flags, Basic auth environment variables, or profile credentials instead.
- Tests: added focused profile-config regressions for `GRAFANA_URL` and profile URLs containing credentials, including a check that the secret value is not echoed in the error.
- Test Run: `rustfmt --check rust/src/profile_config.rs` passed; `git diff --check -- rust/src/profile_config.rs docs/internal/ai-status.md docs/internal/ai-changes.md` passed; `cargo test --manifest-path rust/Cargo.toml --quiet profile_config::tests::resolve_connection_settings_rejects_credentials -- --test-threads=1` did not compile because the current worktree has unrelated pre-existing Rust errors in `snapshot_review_common.rs`, `snapshot_support.rs`, `access/live_project_status.rs`, and `dashboard/import_lookup*.rs`.
- Impact: `rust/src/profile_config.rs`, `docs/internal/ai-status.md`, and `docs/internal/ai-changes.md`.
- Rollback/Risk: low CLI behavior change; users who relied on `http://user:pass@host` must move credentials to `--basic-user` plus `--basic-password` / `--prompt-password`, `GRAFANA_USERNAME` / `GRAFANA_PASSWORD`, or a profile secret.
- Follow-up: consider documenting the rejected URL-userinfo form in troubleshooting if operators hit it often.

## 2026-04-13 - Split Rust facade and CLI-args hotspots
- Summary: split several Rust maintainability hotspots into focused modules while keeping command paths, flags, output contracts, and public runner behavior unchanged; added a read-only maintainability reporter for oversized Rust files and re-export-heavy module roots.
- Tests: preserved existing Rust coverage and added Python coverage for the new reporter.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `cd rust && cargo test --quiet --lib`; `python -m unittest -q python.tests.test_python_rust_maintainability_report`; `python3 scripts/rust_maintainability_report.py --root rust/src --source-line-limit 2000 --test-line-limit 3000 --reexport-line-limit 100`.
- Impact: `rust/src/dashboard/command_runner.rs`, new `rust/src/dashboard/run_{list,inspect}.rs`, `rust/src/access/mod.rs`, new `rust/src/access/{dispatch,auth_materialize}.rs`, `rust/src/datasource.rs`, new datasource helper modules, split `rust/src/sync/cli_args*.rs`, `scripts/rust_maintainability_report.py`, Python reporter tests, `docs/overview-rust.md`, and AI trace docs.
- Rollback/Risk: behavior-preserving module-boundary refactor; rollback would collapse helper modules back into the original large facades. Future feature work should extend the new focused modules instead of re-growing root facades.
- Follow-up: run the maintainability reporter periodically and decide later whether to wire it into a quality target with project-specific thresholds.

## 2026-04-13 - Fix docs portal, landing, and manpage HTML navigation
- Summary: fixed the GitHub Pages root version portal so handbook, command reference, and manpage shortcuts deep-link to the selected output under latest/dev lanes instead of all pointing at the same lane index; removed latest minor duplication from version choices; shortened the local landing page into recommended starts, common jobs, and complete reference; widened the landing layout for common 1366 and 1920 width viewports; moved root manpage router copy into a JSON contract; shortened root manpage subcommand listings; changed generated manpage HTML definition lists from a wide two-column grid to stacked readable entries; rendered root subcommand manpages as collapsible linked groups; linked manpage references in index and SEE ALSO sections; rendered paragraph-style CLI examples as code blocks in HTML; replaced the manpage page left nav with a grouped full manpage index plus documentation lane links instead of the handbook tree; clarified maintainer docs so the published root `index.html` is traceable to the portal contract/generator rather than local `docs/html/index.html`.
- Tests: added focused portal rendering coverage for deep output links, removed old same-lane shortcut labels, checked that the latest minor lane is not repeated as a separate version option, updated HTML generator tests for the current landing and handbook navigation behavior, covered roff `.SS` subsection rendering, collapsible root subcommand groups, manpage cross-links, SEE ALSO links, paragraph-style CLI example recovery, and the dedicated manpage lane nav in generated manpage HTML.
- Test Run: `make man`; `make html`; `python3 -m unittest -v python.tests.test_python_build_pages_site python.tests.test_python_generate_command_html`; `python3 -m unittest -v python.tests.test_python_generate_command_html python.tests.test_python_generate_manpages`; `make html-check`; `make man-check`; `python3 scripts/build_pages_site.py --output-dir /tmp/grafana-util-pages-test --include-dev`; `rg -n "latest/handbook/zh-TW/index.html|latest/commands/zh-TW/index.html|dev/handbook/zh-TW/index.html|dev/commands/zh-TW/index.html|latest/man/index.html|href=\"#outputs\"|先開啟任一版本線|Open a docs lane first|v0.10/index.html" /tmp/grafana-util-pages-test/index.html`; Playwright screenshots at 1366x768 and 1920x1080 with local Chrome; `git diff --check`.
- Impact: `docs/landing/{en,zh-TW}.md`, `scripts/templates/docs.css`, generated `docs/html/`, generated `docs/man/`, `scripts/docsite_html_roff.py`, `scripts/docsite_html_manpage_pages.py`, `scripts/generate_manpages.py`, `scripts/contracts/manpage-router.json`, `scripts/docsite_version_portal.py`, `scripts/build_pages_site.py`, `scripts/contracts/versioned-docs-portal.json`, `python/tests/test_python_build_pages_site.py`, `python/tests/test_python_generate_command_html.py`, `python/tests/test_python_generate_manpages.py`, `docs/internal/generated-docs-playbook.md`, `docs/internal/generated-docs-architecture.md`, and AI trace docs.
- Rollback/Risk: docs navigation and generated HTML/manpage presentation only; lane HTML generation remains on the shared docs generator. Rollback would restore ambiguous portal links, duplicated latest minor version choices, the longer local landing page, table-like manpage command lists, long root subcommand summaries, and non-clickable manpage references.
- Follow-up: none.

## 2026-04-13 - Improve public docs voice and hygiene
- Summary: refreshed handbook and command-reference wording so docs explain user workflows first, added explicit workflow maps plus task-first guidance sections for alert, dashboard, datasource, access, and status/workspace subcommand families, documented that handbooks should not duplicate one page per leaf command, removed generated-looking `Purpose` / `用途` example-comment labels, removed decorative handbook heading emoji, kept command maps out of handbook bodies, renamed the sidebar command map to command shortcuts, removed handbook chapter-count chrome, and tightened zh-TW product terminology.
- Tests: regenerated command HTML and manpages, checked docs surface, and ran whitespace validation.
- Test Run: `make html`; `make man`; `make html-check`; `make man-check`; `make quality-docs-surface`; `git diff --check`.
- Impact: `README.zh-TW.md`, `docs/commands/{en,zh-TW}/`, `docs/user-guide/{en,zh-TW}/`, generated `docs/html/`, `docs/man/`, `docs/internal/zh-tw-style-guide.md`, `docs/internal/generated-docs-playbook.md`, and AI trace docs.
- Rollback/Risk: docs-only wording and generated-output refresh; broad command-doc comment cleanup touches many files, so review should focus on example captions and generated HTML/man parity.
- Follow-up: consider a later targeted pass for command pages whose examples still share similar captions after the global label cleanup.

## 2026-04-12 - Split Rust architecture hotspots and test modules
- Summary: split unified help routing, snapshot review, access rendering, alert CLI runtime/output, and the largest Rust test suites into focused helper modules with thin aggregators.
- Tests: no behavior changes; preserved existing coverage and re-ran focused Rust targets for CLI, dashboard help, access CLI, overview, alert, and snapshot review.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet cli_rust_tests -- --test-threads=1`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_cli_parser_help_rust_tests -- --test-threads=1`; `cargo test --manifest-path rust/Cargo.toml --quiet access_cli_rust_tests -- --test-threads=1`; `cargo test --manifest-path rust/Cargo.toml --quiet overview_rust_tests -- --test-threads=1`; `cargo test --manifest-path rust/Cargo.toml --quiet alert_rust_tests -- --test-threads=1`; `cargo test --manifest-path rust/Cargo.toml --quiet snapshot_rust_tests -- --test-threads=1`; `make quality-architecture`; `make quality-ai-workflow`; `git diff --check`.
- Impact: `rust/src/cli_help.rs`, `rust/src/cli_help/routing.rs`, `rust/src/cli_help/contextual.rs`, `rust/src/cli_help/flat.rs`, `rust/src/snapshot_review*.rs`, `rust/src/access/render*.rs`, `rust/src/alert*.rs`, `rust/src/*_rust_tests.rs`, `rust/src/access/*_rust_tests.rs`, and dashboard/overview test children.
- Rollback/Risk: behavior-preserving module-boundary refactor; rollback would collapse the helper modules back into the original large files. Remaining architecture warnings are pre-existing hotspots outside this pass.
- Follow-up: consider a later pass for remaining warnings in access live status/tests, dashboard import/browse/inspect surfaces, datasource status/import-export, `snapshot.rs`, and the remaining brittle help tests in dashboard inspect and sync.

## 2026-04-12 - Split snapshot review shaping and browser behavior
- Summary: split snapshot review into shared validation, text rendering, tabular/output shaping, and browser-specific helper modules, keeping the public snapshot review entrypoints unchanged while making `snapshot_review.rs` a thin module hub.
- Tests: relied on the existing snapshot review Rust coverage for behavior; reran the focused snapshot review Rust target after the split.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all`; `cargo test --manifest-path rust/Cargo.toml --quiet snapshot_rust_tests -- --test-threads=1`.
- Impact: `rust/src/snapshot_review.rs`, `rust/src/snapshot_review_common.rs`, `rust/src/snapshot_review_render.rs`, `rust/src/snapshot_review_browser.rs`, `rust/src/snapshot_review_output.rs`.
- Rollback/Risk: low. The refactor is behavior-preserving and only changes module boundaries, but the full crate still has unrelated `access` / `alert` compile failures in the current worktree, so broader verification remains blocked until those existing edits are resolved.
- Follow-up: none.

## 2026-04-12 - Split unified CLI help routing helpers
- Summary: split unified CLI help routing into a thinner orchestration layer plus focused `contextual` and `flat` helper modules, keeping the existing public help entrypoints and inferred-subcommand behavior unchanged.
- Tests: re-ran focused unified help, dashboard help parser, and dashboard inspect/help-full Rust suites after the module split.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet cli_rust_tests`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_cli_parser_help_rust_tests`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_cli_inspect_help_rust_tests`.
- Impact: `rust/src/cli_help.rs`, `rust/src/cli_help/routing.rs`, `rust/src/cli_help/contextual.rs`, `rust/src/cli_help/flat.rs`, and AI trace docs.
- Rollback/Risk: low to moderate. The refactor is behavior-preserving and covered by focused help tests, but future help work should extend the focused helper modules instead of re-growing `routing.rs` into another mixed-responsibility file.
- Follow-up: none.

## 2026-04-12 - Add AI trace maintenance tool
- Summary: added `scripts/ai_trace.py` with structured `add`, `compact`, and `check-size` commands for maintaining AI trace files, and wired trace size enforcement into `scripts/check_ai_workflow.py`.
- Tests: added Python unittest coverage for trace insertion, compact/archive append behavior, size-limit checks, and workflow-gate integration.
- Test Run: `python3 -m unittest -v python.tests.test_python_ai_trace python.tests.test_python_check_ai_workflow`; `python3 scripts/ai_trace.py check-size`; `make quality-ai-workflow`; `git diff --check`.
- Impact: `scripts/ai_trace.py`, `scripts/check_ai_workflow.py`, `python/tests/test_python_ai_trace.py`, `python/tests/test_python_check_ai_workflow.py`, `docs/internal/ai-status.md`, `docs/internal/ai-changes.md`, and current AI trace archives after compaction.
- Rollback/Risk: internal maintainer tooling only; rollback removes the helper and the size check, but manual trace maintenance would again be required.
- Follow-up: none.

## 2026-04-12 - Add flat CLI help inventory
- Summary: added root `grafana-util --help-flat` output that expands the visible public Clap command tree into a grep-friendly table with command path, group/command kind, and operator-facing purpose text.
- Tests: added CLI help coverage for root pre-parse routing, colorized output, public command inclusion, hidden command exclusion, and rejection of leaked internal `Struct definition` / `Arguments for` wording.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all`; `cargo test --manifest-path rust/Cargo.toml --quiet cli_rust_tests -- --test-threads=1`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`; `git diff --check`; `python3 -m json.tool scripts/contracts/command-surface.json`.
- Validation: manually checked `cargo run --manifest-path rust/Cargo.toml --quiet --bin grafana-util -- --help-flat` and confirmed all public roots render with purpose text and access leaf commands use operator-facing descriptions.
- Impact: `rust/src/cli.rs`, `rust/src/cli_help.rs`, `rust/src/cli_help/routing.rs`, access CLI command definitions, CLI help tests, command-surface contract/checker, command-reference index docs, and maintainer workflow docs that reference root help inventory support.
- Rollback/Risk: root pre-parse now reserves `--help-flat`; command purposes depend on command-level Clap `about` metadata, so new commands should provide product-facing `about` text instead of relying on Args struct comments.
- Follow-up: none.
