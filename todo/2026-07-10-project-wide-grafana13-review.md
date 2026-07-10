# Project-wide Grafana 13 Review

## Status

- State: complete
- Date: 2026-07-10
- Branch reviewed: `dev`
- Review baseline: `09750544`
- Scope type: analysis and recommendations only; confirmed fixes require a
  separately authorized implementation slice.

## Goal

Re-review the complete Rust-first project for actionable backend, CLI/TUI,
browser-facing, Python compatibility, test, documentation, and operational
improvements, including bugs and Grafana 13 integration opportunities.

## Scope

### Primary implementation

- Rust core under `rust/src/`, excluding `rust/target`.
- Grafana HTTP and resource API boundaries.
- Dashboard, datasource, alert, access, sync, snapshot, status, and review
  command families.
- Shared output, auth, error, browser, and TUI infrastructure.

### Operator-facing surfaces

- CLI taxonomy, parser/dispatch behavior, help examples, and failure messages.
- TUI registry, browse/detail/review/confirmation flows, keyboard behavior,
  terminal-size handling, focus, scrolling, search, and redaction.
- Browser-enabled review surface when the optional `browser` feature is used.
- Source docs, generated man/HTML surfaces, and public command contracts.

### Secondary and cross-language surfaces

- Python code only where it remains part of packaging, install behavior,
  explicit parity, contract generation, or repository tooling.
- Shared fixtures and Rust/Python/schema contract drift.

### Grafana 13

- Verify relevant Grafana 13 functionality and API changes against current
  official Grafana documentation and release material.
- Compare those capabilities with the repository's current classic API,
  `dashboard.grafana.app` resource API, datasource, alerting, access, and
  observability surfaces.
- Separate supported public APIs from experimental, feature-flagged,
  Enterprise/Cloud-only, or UI-only functionality.

## Out of Scope

- Editing generated documentation directly.
- Inspecting Cargo outputs under `rust/target`.
- Broad Python feature expansion without an active compatibility requirement.
- Implementing findings during this review.
- Claiming support for undocumented Grafana endpoints.

## Constraints

- Treat Rust as the product implementation source of truth.
- Use CodeGraph first for structural questions and native search only for
  literal strings, fixtures, docs, or already-open files.
- Validate suspected bugs with the smallest focused test or direct fixture
  evidence before ranking them as confirmed.
- Preserve the lean default Rust feature set; do not assume `browser` belongs
  in default builds.
- Check public CLI and documentation findings against
  `scripts/contracts/command-surface.json` and
  `scripts/contracts/docs-entrypoints.json`.
- Distinguish implementation defects, missing regression coverage, product
  gaps, documentation drift, and speculative future work.

## Assumptions

- "Frontend" means the TUI, optional browser review surface, CLI presentation,
  and generated documentation site because this repository has no standalone
  application SPA in the indexed source tree.
- The current clean `dev` worktree at `09750544`, ahead of `origin/dev`, is the
  review baseline; existing commits are reviewed as current product state, not
  as a request to rewrite history.
- Grafana 13 integration recommendations must remain useful for both self-hosted
  Grafana and edition-specific deployments by labeling availability.

## Acceptance Checks

- [x] Map the current Rust command, Grafana transport, output/review, browser,
  and TUI boundaries with source evidence.
- [x] Review all public command families for error handling, destructive-action
  safety, ownership/provenance, output stability, and cross-domain consistency.
- [x] Review TUI state transitions, terminal behavior, search/scroll/focus,
  confirmation, secret redaction, fallback behavior, and test coverage.
- [x] Review remaining Python surfaces for live ownership or Rust-parity drift.
- [x] Run relevant Rust, architecture, TUI inventory, output-contract, docs
  surface, and generated-doc drift checks.
- [x] Verify Grafana 13 claims using official current sources and record API
  maturity and edition constraints.
- [x] For every confirmed bug, provide file/line evidence, impact, reproduction
  or failing contract, and a bounded fix direction.
- [x] Rank findings as P0/P1/P2/P3 and separate confirmed defects from
  improvements and Grafana 13 opportunities.
- [x] Update this dated note with commands and results, then mark the concise
  index complete only when the review report is delivered.

## Execution Plan

- [x] Inventory architecture and current feature/command surfaces.
- [x] Review Grafana transport and each backend domain boundary.
- [x] Review CLI dispatch, output, review contracts, and destructive workflows.
- [x] Review TUI/browser presentation and interaction contracts.
- [x] Check Python/fixture/packaging parity risks.
- [x] Research Grafana 13 official capabilities and compare them to the code.
- [x] Run focused and broad validation, reproducing suspected defects.
- [x] Record ranked findings, non-findings, evidence, and implementation slices.

## Evidence Log

- 2026-07-10: `git status --short --branch` showed a clean `dev` branch ahead
  of `origin/dev` by 29 commits before these tracking files were added.
- 2026-07-10: CodeGraph reported 1,038 indexed files, 20,876 nodes, and 58,856
  edges; primary source surfaces are Rust, secondary Python, and repository
  generation/contract scripts.
- 2026-07-10: File inventory confirmed no standalone SPA; frontend review scope
  is TUI, optional browser, CLI presentation, and generated docs site.
- 2026-07-10: Rust default validation passed 1,882 tests with one ignored test;
  formatting, Clippy with warnings denied, architecture, output-contract, docs
  surface, man-page drift, and HTML drift checks passed.
- 2026-07-10: The optional browser lane compiled with
  `cargo check --features browser`.
- 2026-07-10: `make test-python` passed 1,226 tests with two skips in the system
  Python environment, while the equivalent Poetry discovery run produced one
  failure and seven errors because the declared environment lacks PyYAML.
- 2026-07-10: The TUI registry checker and its 10 unit tests passed, but manual
  source-to-registry comparison found two public TUI paths that the checker
  cannot currently detect.
- 2026-07-10: Official Grafana API discovery at
  `https://play.grafana.org/apis/dashboard.grafana.app/v2` returned the served
  `dashboard.grafana.app/v2` dashboard resource and its CRUD verbs.

## Findings

### Summary

- No P0 issue was found.
- Two P1 product defects and two P2 correctness/governance defects were
  confirmed.
- The Rust-first product core is in substantially better condition than the
  compatibility and governance edges: the full default Rust suite, strict
  linting, generated documentation, architecture checks, and optional browser
  compilation all passed.
- The highest-value Grafana 13 work is to finish the public dashboard resource
  v2 lane, add shared API discovery/capability routing, and then extend resource
  history and alert-notification resources without breaking legacy instances.

### P1 - Public dashboard resource v2/all options are deterministic dead paths

**Type:** confirmed product defect and Grafana 13 compatibility blocker.

**Evidence**

- `rust/src/commands/dashboard/cli_defs_command_export.rs:27-33` publicly
  exposes `none`, `v1`, `v2`, and `all`; the help at lines 130-137 tells
  operators to use those values.
- `rust/src/grafana/api/dashboard.rs:9-23` rejects v2 with “not verified yet”.
- `rust/src/commands/dashboard/export_resource.rs:25-37`, 41-56, and 76-85
  reject the v2 lane before any export. Consequently, `all` also fails instead
  of exporting every advertised resource representation.
- The official dashboard API documentation now identifies
  `dashboard.grafana.app/v2` as the current dashboard resource API, and live
  official discovery returned that resource and its CRUD verbs.

**Impact**

Grafana 13 operators can select a documented CLI value that can never succeed.
This is especially important because dynamic dashboards are generally
available in Grafana 13 and existing dashboards migrate to the new structure
when opened.

**Bounded fix direction**

1. Implement native v2 resource export with recorded v2 fixtures and an
   authenticated live smoke lane.
2. Make `all` collect supported lanes independently and report per-lane status
   rather than failing during lane construction.
3. Keep classic import/diff/apply v2 rejection until a typed `spec.elements`
   adapter has round-trip and semantic-diff coverage; do not reinterpret v2 as
   a classic dashboard payload.

Official references:

- <https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/dashboard/>
- <https://grafana.com/docs/grafana/latest/whatsnew/whats-new-in-v13-0/>
- <https://play.grafana.org/swagger?api=dashboard.grafana.app-v2>

### P1 - The packaged Python environment silently lacks its YAML contract

**Type:** confirmed packaging/runtime defect in the maintained compatibility
surface.

**Evidence**

- `python/pyproject.toml:13-16` declares Pillow and requests but not PyYAML.
- `python/grafana_utils/yaml_compat.py:8-30` treats PyYAML as optional and
  silently replaces YAML parsing and emission with JSON.
- In the declared Poetry environment,
  `PYTHONPATH=. poetry run python -m unittest discover -s tests -q` ran 1,226
  tests and produced one failure plus seven errors. The YAML output assertion
  received JSON, while YAML config and local-artifact tests failed in the JSON
  parser.
- The same suite passed under system Python, where PyYAML 6.0.3 is installed:
  1,226 tests passed with two skips.

**Impact**

An installed package can advertise YAML output and configuration while
producing JSON or rejecting valid YAML. The green system-Python suite masks the
declared-environment failure.

**Bounded fix direction**

- Declare PyYAML as a runtime dependency and add an installed-wheel smoke test
  for one YAML config input and one `--yaml` output.
- Remove the JSON fallback for user-selected YAML behavior; fail explicitly if
  a deliberately minimal build ever omits YAML support.
- Correct `Makefile:176-177` and the matching maintainer documentation: the
  current `python -m unittest -v tests` Poetry target discovers zero tests and
  exits 5, whereas `unittest discover -s tests` runs the actual suite.

### P2 - Datasource UID auto-fallback issues a deprecated numeric-ID request

**Type:** confirmed error-routing defect, exposed more clearly by Grafana 13.

**Evidence**

- `rust/src/commands/resource/runtime.rs:107-126` always supplies
  `/api/datasources/{identity}` as the legacy fallback for a UID lookup.
- `resolve_with_api_mode` at lines 158-183 invokes that fallback after any
  primary 404. A missing UID such as `prom-main` therefore triggers the
  deprecated numeric-ID route with a nonnumeric path segment and can mask the
  useful original error.
- Tests at lines 345-365 cover numeric identity `10` but not a missing
  nonnumeric UID.
- Grafana 13 disables legacy numeric datasource-ID APIs by default; the
  `datasourceLegacyIdApi` flag is only a temporary compatibility route.

**Bounded fix direction**

Allow automatic legacy fallback only when the selector is numeric. Preserve an
explicit legacy mode for old numeric workflows and add a regression test that
a nonnumeric UID 404 performs exactly one request and preserves its error.

Official reference:
<https://grafana.com/docs/grafana/latest/upgrade-guide/upgrade-v13.0/>

### P2 - TUI registry passes while omitting public TUI commands

**Type:** confirmed governance and regression-coverage defect.

**Evidence**

- Public TUI commands exist at `grafana-util access user browse` and
  `grafana-util access team browse`; their parser definitions are in
  `rust/src/commands/access/access_user_cli.rs:136-212` and
  `rust/src/commands/access/access_team_cli.rs:107-155`, and both are documented.
- `scripts/contracts/tui-registry.json` contains neither path. It also labels
  consolidated `access browse` mutation-capable even though
  `rust/src/commands/access/access_browse.rs:392-505` only loads, filters,
  navigates, and exits.
- `scripts/tui_inventory_report.py:234-275` considers a surface code-backed if
  any TUI-like Rust file exists in the broad domain; lines 278-348 do not
  validate an exact parser entrypoint or a real test path.
- `python3 scripts/tui_inventory_report.py --registry-check --json` reported 19
  covered surfaces and no findings despite these omissions.

**Impact**

The quality gate can remain green when a public TUI route is unregistered,
misclassified, or lacks route-specific tests. That weakens change-impact
routing and makes future TUI regressions easier to miss.

**Bounded fix direction**

- Add both browse routes and correct consolidated `access browse` to read-only.
- Represent capability per mode where local and live behavior differ.
- Replace `has_tests: true` with concrete parser and test paths; validate exact
  CLI paths against the command-surface contract or generated full help.
- Add focused small-terminal modal and narrow-height scroll tests. This last
  item is a coverage improvement, not a confirmed rendering defect.

### P2 - Grafana 13 dashboard tracker state is stale

**Type:** planning/documentation drift.

`docs/todo/todo.md:14-19` still presents the v1 resource-export lane as active
and v2 as unverified. The v1 lane shipped in commit `e69053cc`, and current
official discovery serves v2. Refresh that historical tracker when the v2
implementation slice is opened so it routes work instead of preserving an
obsolete API assumption.

## Grafana 13 Integration Backlog

### 1. Dashboard v2 and shared API discovery - P1

Treat `/apis/<group>/<version>` discovery as the source of truth instead of
version-string gating. Enable native dashboard v2 export first, then introduce
a typed read-only inspect/diff adapter. Extend `status resource` from its
current UID-versus-numeric meaning toward explicit `auto`, `legacy`, `v1`, and
`v2` resource capabilities without removing legacy routes for older Grafana.

Grafana's migration guidance says the legacy `/api` family is deprecated but
not yet disabled, so this should be negotiated compatibility, not a flag-day
rewrite.

Official references:

- <https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/apis/>
- <https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/apis-migration/>

### 2. Resource history and folder resources - P1

Add a resource-history lane for stable v1/v2 resources using label and field
selectors plus `continue` pagination. Preserve classic dashboard-version
history as the compatibility default until capability discovery proves the new
lane. Add folder resource support through
`folder.grafana.app/v1/namespaces/:namespace/folders`, including continuation
tokens, rather than mapping the new API onto classic `/api/search` behavior.

Official references:

- <https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/resource-history/>
- <https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/folder/>

### 3. Alert notification resources - P1/P2

The current alert clients use `/api/v1/provisioning` for rules, contact points,
mute timings, templates, and policy trees. Add an opt-in, discovery-gated
adapter for Grafana 13's `notifications.alerting.grafana.app/v1beta1`
receivers, routing trees, template groups, time intervals, inhibition rules,
and receiver tests. Keep the legacy provisioning lane because the new surface
is beta and its schemas differ; do not silently mix alert-rule migration into
the notification-resource work.

Official reference:
<https://grafana.com/docs/grafana/latest/upgrade-guide/upgrade-v13.0/>

### 4. Git Sync 13.1 compatibility refresh - P2

The repository already models Git Sync ownership and blocks unsafe direct live
apply, so it should integrate rather than duplicate Grafana's feature. Add
fixtures for Grafana 13.1 root-level sync, synced-folder import, repository
README provenance, and verified-commit/branch expectations. Recheck existing
assumptions that local content always lives below `dashboards/git-sync/`.

Official reference:
<https://grafana.com/docs/grafana/latest/whatsnew/whats-new-in-v13-1/>

### 5. Secrets Management API - P2, gated

Consider `secret.grafana.app/v1beta1` only as an optional secure-value provider
after defining a threat model, reference semantics, redaction, and round-trip
behavior. Never turn secret resources into an export lane or artifact payload.

Official reference:
<https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/secrets_management/>

### Defer until a stable public API exists

- Grafana Advisor is useful operationally, but no stable public API was
  identified in this review; do not scrape its UI.
- Deleted-dashboard restore is generally available in Grafana 13, but no
  stable public restore endpoint was identified. A version-aware TUI message is
  reasonable; endpoint integration is not.
- Suggested dashboards, new panel styles, and Assistant are primarily Grafana
  UI experiences and do not justify CLI coupling.
- Grafana Image Renderer plugin removal does not directly break this project:
  the optional browser path compiled and uses headless Chrome rather than the
  Grafana renderer plugin.

Official dashboard-management reference:
<https://grafana.com/docs/grafana/latest/visualizations/dashboards/manage-dashboards/>

## Non-findings and Review Boundaries

- No standalone frontend application is present. The reviewed presentation
  surfaces are CLI help/output, TUI flows, optional browser capture, and
  generated documentation.
- No additional confirmed TUI crash, unsafe mutation, or secret-redaction leak
  was reproduced. Local artifact modes are guarded as read-only and existing
  key, search, scroll, confirmation, and redaction coverage is substantial.
- The isolated `make quality-docs-surface` run passed. An earlier concurrent run
  transiently reported missing `access browse` while another Cargo process was
  active; the route's direct help and the isolated rerun passed, so this is not
  recorded as a product defect. Run Cargo-backed docs checks serially in CI or
  diagnostics when investigating intermittent results.
- Python remains secondary. The review recommends repairing its declared
  installation contract, not expanding it to duplicate current Rust features.

## Verification Matrix

| Area | Command | Result |
| --- | --- | --- |
| Rust tests | `cargo test --manifest-path rust/Cargo.toml --quiet` | 1,882 passed, 1 ignored |
| Rust format | `cargo fmt --manifest-path rust/Cargo.toml --all --check` | Passed |
| Rust lint | `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings` | Passed |
| Browser feature | `cargo check --manifest-path rust/Cargo.toml --features browser` | Passed |
| Architecture | `make quality-architecture` | Passed |
| Output contracts | `make quality-output-contracts` | Passed |
| Docs surface | `make quality-docs-surface` | Passed in isolated rerun |
| TUI checker tests | `python3 -m unittest -v scripts.test_tui_inventory_report` | 10 passed |
| Generated man pages | `make man-check` | Passed |
| Generated HTML | `make html-check` | Passed |
| Python, system env | `make test-python` | 1,226 passed, 2 skipped |
| Python, Poetry env | `cd python && PYTHONPATH=. poetry run python -m unittest discover -s tests -q` | 1 failed, 7 errors; missing PyYAML confirmed |

No code fix was made in this review. The only workspace additions are this
evidence report and its concise todo index; each confirmed finding is ready to
be scheduled as a separately test-driven implementation slice.
