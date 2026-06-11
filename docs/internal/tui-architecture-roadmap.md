# TUI Architecture Roadmap

Phase 0 owns the inventory and planning layer plus low-risk, focused browse
maturity cleanup where the behavior is already covered by narrow tests. The
follow-on consistency pass aligned shared search/status/control language across
several existing Rust TUI surfaces without changing public CLI paths.

## Current Inventory

| Surface | Entrypoint | Current tier | Notes |
| --- | --- | --- | --- |
| Dashboard browse | `grafana-util dashboard browse` | Mature | Dedicated live/local tree browser under `rust/src/commands/dashboard/browse/`; keep current ownership disjoint. |
| Access browse | `grafana-util access browse`, `access user browse`, `access team browse` | Active implementation | Consolidated access browse now has in-session filtering, selection summaries, and shared-shell header/footer language. User/team specialized browsers remain separate richer flows; repeat-search now skips the selected row and wraps across matching rows. |
| Datasource browse | `grafana-util datasource browse`, local `datasource list --interactive` | Active implementation | Live/local datasource browser under `rust/src/commands/datasource/browse/`; it surfaces row, kind, search context, repeat-search wrap, and a first-class Review pane for secret/provider/read-only plus compatible local plan/diff/import evidence carried in datasource details. Local `datasource list --interactive` reuses that Review projection in its item details for already-loaded artifacts. |
| Status overview | `grafana-util status overview live --output-format interactive` and staged/live status interactive output | Mature document browser | Uses the overview/status document model, then projects into TUI. It now supports current-view `/`, `?`, and `n` item search with explicit search status in the shell. |
| Dashboard summary / inspect workbench | `grafana-util dashboard summary --interactive` | Mature review workbench | Query, dashboard, and governance rows share the inspect workbench. |
| Dashboard import review | `grafana-util dashboard import --interactive` | Mature but specialized | Client-backed selector and focused review flow are import-specific. Keep changes evidence-led. |
| Dashboard policy / dependencies / impact / topology | `--interactive` review modes | Mixed maturity | Useful precedent for shared review navigation, but several surfaces still have domain-specific runners. |
| Snapshot review | `grafana-util snapshot review --interactive` | Document-backed review | Browser-style review over snapshot artifacts. |
| Sync audit/review | `grafana-util sync ... --interactive` internal flows | Internal review surface | Keeps shared review ideas, but public docs should continue steering users through `workspace` where applicable. |
| Shared TUI shell | `rust/src/common/tui/` and `rust/src/common/browser/session.rs` | Shared primitive | Owns visual shell/session primitives and read-only browser search. Shared read-only browser repeat-search skips the current row and wraps within the active filter. It is not yet a complete domain-neutral workbench framework. |

## Maturity Tiers

- Tier 0: terminal prompts, such as `--prompt`, that are not full-screen TUI.
- Tier 1: one-off interactive selectors that are tied to one command and one data shape.
- Tier 2: domain workbenches that separate data collection from render/input state.
- Tier 3: document-backed browsers where text/JSON/TUI consume the same domain document.
- Tier 4: shared review framework where actions, risks, hints, and navigation are reusable across domains.

The target shape is Tier 3 for inventory/read surfaces and Tier 4 for review or
mutation previews. New work should avoid adding another Tier 1 surface unless
the command is explicitly experimental.

## Architecture Debt

- Public flag language is still mixed: `--interactive` is used for full-screen
  TUI, while some older human-operated flows were migrated to `--prompt`.
- Shared visual shell helpers and shared read-only browser search exist, but many
  workbenches still own local state, footer controls, filtering, and focus
  models.
- Feature gating is still coarse. Default builds include `tui`; `--no-default-features`
  must keep clear errors for interactive paths, but TUI modules are not fully
  isolated behind a separate artifact lane.
- Several review surfaces duplicate similar action/status/risk projection logic
  instead of consuming one normalized review model.
- Docs mention interactive surfaces across command pages and handbooks, but there
  has not been a lightweight inventory report to keep architecture discussions
  grounded in current files.

## Approved Phase Plan

1. Phase 0: document the inventory, tiers, debt, and next phases; add a
   read-only report script if it fits repo conventions; and allow focused
   access/datasource browse maturity cleanup when covered by narrow tests.
2. Phase 1: define shared TUI ownership rules for document-backed browsers,
   review workbenches, prompt-vs-TUI naming, and `tui` feature fallback messages.
3. Phase 2: extract common review workbench contracts only where two or more
   domains already prove the same shape. Start from read-only projections before
   mutation actions.
4. Phase 3: migrate one low-risk Tier 1 or Tier 2 surface to the shared contract
   with focused parser/render tests and no public CLI drift.
5. Phase 4: update command docs and generated artifacts only after behavior or
   public CLI wording changes. Run the command-surface and generated-doc checks
   for those later phases.

## Phase 0 Report Script

Run the current inventory helper manually when planning TUI work:

```bash
python3 scripts/tui_inventory_report.py
```

The script is intentionally non-blocking and is not wired into CI. It scans
`rust/src`, English command docs, English user guide docs, and `docs/internal`
while skipping generated HTML and Cargo build output. It also reports
`helper-drift candidates` for local TUI/detail/review helper functions that
still look like possible shared-helper follow-up work.

## Recent Follow-Up

- Status overview TUI now follows the smaller facade/runtime/tests shape used by
  the newer status TUI module: `tui.rs` owns state and search behavior,
  `tui_runtime.rs` owns the terminal event loop, and `tui_tests.rs` owns the
  focused behavior tests. This removed `status/overview/tui.rs` from the
  architecture warning-threshold list without changing public CLI behavior.
- `access user browse` and `access team browse` repeat-search behavior now skips
  the selected row and wraps in line with datasource browse and status overview,
  so `n` can continue within the current result set after reaching the first or
  last matching row.
- `datasource browse` now uses the same boundary rule, so repeated search does
  not reselect the first or last matching row before wrapping.
- `datasource browse` now separates secret placeholder, blocker, and
  review-required evidence into a dedicated Review pane, leaving general
  datasource metadata in Facts and keeping resolved credential values hidden.
- `datasource browse` Review lines now also recognize compatible review
  evidence fields already carried in datasource details, including action,
  status, match basis, destination, source file, target UID/version/read-only,
  blocked reason, changed fields, review hints, and secret-value requirements.
  Secret-like changed-field paths such as `secureJsonData.*` stay hidden.
- Local `datasource list --interactive` now appends the same safe Review
  evidence to datasource item details when local inventory/provisioning records
  already carry compatible plan/diff/import fields. This reuses the browse
  projection without adding local input flags to the live browse command.
- The shared read-only browser now applies the same repeat-search boundary rule
  as datasource/access/status browsers: `n` skips the selected row first, then
  wraps forward or backward within the active filter.
- Dashboard inspect workbench footer controls now use the same compact `Esc/q`
  exit label as the other Rust TUI browse/review surfaces.
- Dashboard inspect workbench full-detail viewer status and footer controls now
  use compact `Enter/Esc/q` close language instead of split close hints.
- Dashboard inspect workbench repeat-search now also skips the selected row
  before wrapping backward at the first matching row.
- Dashboard browse repeat-search now follows the same backward boundary rule,
  so it does not reselect the first matching tree node before wrapping.
- Snapshot review datasource rows now preserve safe local review evidence and
  reuse the same datasource Review projection in browser details.
- Sync review TUI operation previews now reuse the datasource safe changed-field
  rule, so secret-like paths such as `secureJsonData.*` stay out of the preview.
- Access user/team and datasource browse destructive confirmation dialogs now
  use compact `Confirm: y` and `Cancel: n/Esc/q` body copy.
- Access user/team, dashboard browse, datasource browse, dashboard inspect
  workbench, and status overview search prompts now share the compact
  `Enter search`, `Esc cancel`, and `n repeat` hint language.
- The shared read-only browser now also uses the compact pending search summary
  and footer copy, so downstream artifact browsers do not regress to
  `Enter apply` wording or advertise repeat while the prompt is accepting text.
- Shared read-only browser detail titles now use the filtered visible position
  and total, matching the list numbering and footer selection count.
- Dashboard policy/governance gate TUI now supports current-group finding
  search/filtering and surfaces idle, active, and prompt search state in the
  footer.
- Dashboard impact TUI now supports current-group item search/filtering and
  surfaces idle, active, and prompt search state in the footer.
- Dashboard topology/dependencies TUI now supports the same current-group node
  search/filtering pattern and footer search state copy.
- Sync audit TUI now supports current-group drift row search/filtering and the
  same idle, active, and prompt search state footer copy.
- Sync review checklist mode now supports current-view operation
  search/filtering with the same footer search state copy while keeping `n` as
  the existing select-none command.
- Access plan interactive review now consumes shared review-contract action
  detail lines for generic action, identity, status, blocker, and detail
  evidence while keeping its access-specific narrative and next-check guidance.
- Dashboard TUI feature fallback now uses the shared `tui_feature_required`
  error helper so dashboard interactive paths report the same TUI error
  category and `requires the `tui` feature` wording as the other Rust TUI
  surfaces.
- Dashboard import review footer copy now advertises the existing page jump,
  bounds jump, and `Esc/q`/`Ctrl-C` cancel controls instead of only `q`.
- Dashboard browse delete review summary now splits `Confirm: y` from
  `Cancel: n/Esc/q`, matching the footer/input behavior used by the other
  destructive TUI confirmations.
- Status TUI detail scrolling now clamps to the selected detail content so
  repeated scroll keys cannot move the detail pane beyond its rendered lines.
- Status TUI footer copy now advertises the existing `Home/End` jump behavior
  alongside movement, detail scroll, and `Esc/q` exit controls.
- Status TUI now supports `/`, `?`, and `n` search over domain and action lists
  with the same compact search prompt hint language used by the browse surfaces.
- Status TUI header rows now use the shared shell key-chip/plain span helpers
  instead of carrying local duplicates for the same visual primitives.
- Status overview section details now use the shared browser detail fact helper
  for `Label: value` strings instead of carrying a local formatter.
- Datasource browse review empty states now use a shared browser `REVIEW`
  message helper, leaving the datasource renderer with only panel-specific
  review selection logic.
- Dashboard browse fact rendering now uses a dashboard-specific detail builder
  name for its local action-row filtering and live-details badge behavior,
  clearing the remaining helper-drift inventory candidate.
- Dashboard browse header context now matches datasource browse by surfacing the
  selected row position, selected node kind, and current search state.
- Dashboard browse overlay modes such as search now surface as explicit header
  modes instead of continuing to read as plain browse/local-browse.
- Datasource browse edit dialog now wraps Tab/Shift+Tab field navigation and
  advertises both next and previous field controls in the dialog header.
- Rust TUI no-default isolation now keeps access/dashboard/datasource/status,
  sync-audit, and snapshot interactive helpers behind the `tui` feature during
  test builds, while preserving the shared feature-disabled fallback test.
- The sync review side-by-side diff rendering helpers now live with the shared
  review diff model, so future compatible dashboard, alert, datasource, access,
  or workspace review surfaces can reuse the same title, scroll, wrap, clip,
  and TUI list-item projection instead of copying sync-local helpers.
- Dashboard import interactive reviews now build a shared `ReviewDiffModel`
  for compatible live-vs-local title, folder UID, tag, and panel changed-field
  evidence, then render a compact shared live/desired preview in the review
  pane while keeping the existing summary, structural, and raw diff lines.
- Access plan interactive reviews now build compact shared live/desired diff
  previews from safe bundle/live change rows, using the same `ReviewDiffModel`
  vocabulary while filtering secret-like access change fields out of both the
  generic review detail string and TUI preview output.
- Access plan now gets that compact live/desired diff preview through the
  shared review-contract action projection, so compatible mutation review TUIs
  can reuse the same safe changed-field preview path.
- Access plan change-detail rows now also come from the shared review-contract
  action projection, keeping `Change: field bundle=... live=...` rows and
  secret-like field filtering reusable across mutation review panes.
- Access plan live-target evidence rows now use shared review-contract action
  projection for the known target fields rendered as `Live target: key=value`.
- Access plan warning/blocker context rows now use shared review-contract action
  projection for blocked reasons, safe warning changed fields, and blocked live
  target flags.
- Access plan narrative and impact rows now use shared review-contract action
  projection, leaving the access TUI renderer to assemble sections instead of
  owning generic action/status/changed-field guidance text.
- Access plan interactive reviews now also use shared review-contract next-check
  projection for action hints and default follow-up guidance, reducing another
  per-surface review pane shaping rule.
- Review changed-field safety now uses the shared review diff predicate across
  sync diff models, datasource/snapshot review evidence, and access plan review
  details so secret-like paths such as password, token, API key, and
  secureJsonData fields are filtered before TUI review panes render field names
  or values.
- Datasource local list and snapshot datasource review browser rows now call
  the datasource Review projection directly from datasource details instead of
  constructing dummy browse items, reducing repeated local artifact browser
  shaping while preserving the live datasource browse review pane.
- Datasource local list and snapshot datasource review browser rows now also
  share datasource identity detail projection, so `Name`, `UID`, `Type`, `Org`,
  `URL`, `Access`, and `Default` facts stay consistent before review evidence.
- Access plan, datasource local, and snapshot datasource review browser rows now
  use the shared review-contract helper for appending `Review evidence:`
  sections, keeping the heading/empty-section rule consistent across these TUI
  browser details.
- The shared read-only browser now owns a reusable detail-section helper for
  `Heading: none` versus `Heading:` plus body lines; dashboard topology inbound
  and outbound edge summaries use it instead of repeating that section shaping
  locally.
- Dashboard inspect workbench detail rows now use the shared read-only browser
  fact formatter for `Label: value` rows, reducing another local detail
  projection helper while preserving the inspect workbench item output.
- Dashboard inspect workbench full-detail rows now use a shared read-only
  browser aligned fact formatter for the `Kind`, `Title`, and `Summary`
  metadata rows, removing the remaining local full-detail fact helper.
- Datasource browse detail rows now reuse shared read-only browser fact
  formatters, including trimmed fallback handling for blank UID/name/type/URL
  style metadata, while preserving existing datasource detail output.
- Datasource browse detail rendering now uses the shared read-only browser
  styled info-line projection for `Label: value` rows instead of carrying a
  local copy of the same 18-column label/value styling.
- Dashboard browse detail rendering now uses the same shared styled info-line
  projection while keeping its dashboard-specific action-line filters and
  `LIVE DETAILS` badge behavior.
- Dashboard browse and datasource browse muted labels now use shared
  `tui_shell::muted` spans instead of carrying identical local helpers.
- Dashboard browse and datasource browse boxed helper text now use shared
  `tui_shell::boxed` spans instead of carrying local plain-boxed helpers.
- Datasource browse footer controls now use a shared fixed-body-width shell
  control-line helper instead of a local footer-row renderer.
- Datasource browse review-pane rows now use a shared read-only browser review
  info-line projection for 24-column `Label: value` evidence and
  blocker/required highlighting.
- Access user/team browse detail rows now share the read-only browser detail
  info-line helper for 18-column `Label: value` rows with blank-value fallback,
  removing duplicated detail-line renderers from both specialized browsers.
- Access user/team browse action rows now call shared shell key-chip/plain span
  helpers directly instead of carrying local delegate wrappers.
- Access user/team browse facts now call the shared browser detail info-line
  helper directly for `Label: value` rows instead of keeping local
  `detail_line` delegate wrappers.
- Dashboard inspect workbench full-detail viewer wrapping now uses a shared
  read-only browser labeled-detail wrapper for aligned metadata rows, keeping
  logical row mapping local to the viewer.
- Dashboard inspect workbench shell controls now call shared `tui_shell`
  key-chip/plain/control-line helpers directly instead of keeping local
  delegate wrappers in render helpers.
- No-default TUI warning noise is narrower: feature-disabled builds no longer
  compile unused imports or large dead-code surfaces for access browse, access
  plan summary lines, shared read-only browser internals, snapshot browser
  shaping imports, review diff helpers, governance finding item shaping, or
  datasource browse support. Remaining warnings are smaller ownership/fallback
  cleanups.
- Dashboard no-default TUI cfg noise is narrower again: topology/impact
  interactive test browser branches are now gated on the `tui` feature, and
  dashboard import/inspect test-only re-exports used by TUI-gated tests no
  longer compile into no-default all-target builds.
- The remaining no-default TUI helper/alias warning surface is cleared:
  access plan review projections, shared review-contract detail projection,
  dashboard browse source helpers, report-column test wrappers, TUI-only
  dashboard fixtures, and snapshot root-command helpers no longer compile into
  no-default targets that cannot use them.
- The change stayed in state/tests. Public CLI/docs and generated docs remain
  unchanged because the user-facing command surface did not change.

## Completion Audit

The 2026-05-25 completion pass found no remaining helper-drift candidates in
`scripts/tui_inventory_report.py --json`; its `helperDrift` array is empty.
The earlier follow-up items are now covered by current code:

- Local artifact browser review/detail projection is shared where the data
  shape is compatible: datasource local list and snapshot datasource rows reuse
  the datasource detail and Review projections, and access plan rows reuse the
  shared review-contract action evidence, narrative, impact, target, context,
  next-check, and diff-preview projections.
- Compatible diff review surfaces consume the shared review diff model path:
  sync review builds operation diff models through `review_diff`, dashboard
  import review builds `ReviewDiffModel` for compatible live/local evidence,
  and access plan review gets compact shared diff-preview rows through the
  review contract.
- Remaining domain TUI modules still own domain-specific input loops, focus
  state, and renderer layout by design. The roadmap does not require replacing
  those with a complete domain-neutral workbench framework in this pass.

Completion evidence for this pass should include the focused TUI regressions,
`cargo test --quiet access` outside the sandbox when local mock-server binding
is denied, `RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features
--all-targets`, `cargo fmt --check`, `python3 scripts/tui_inventory_report.py
--json`, `make quality-ai-workflow`, and `git diff --check`.
