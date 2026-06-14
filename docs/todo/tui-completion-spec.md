# TUI Completion Spec

Date: 2026-06-14

This document is the detailed backlog for completing the Rust TUI surface. Keep
root `todo.md` short and link here for task details.

## Goal

Bring every TUI and interactive terminal surface to a consistent, testable
operator experience without adding a premature domain-neutral framework.

Completion means:

- every public interactive entrypoint has a clear owner, maturity tier, docs
  location, fallback behavior, and focused test coverage;
- read-only inventory and review surfaces consume typed domain documents or
  typed review projections, not ad hoc text shaping;
- mutation review surfaces preserve secrets, blockers, target evidence, and
  domain-specific context while sharing only proven review vocabulary;
- feature-gated builds keep default `tui` and optional `browser` as the only
  supported Rust release lanes;
- generated docs and command docs describe only the behavior that exists.

## Evidence Baseline

Current inventory commands:

```bash
python3 scripts/tui_inventory_report.py
python3 scripts/tui_inventory_report.py --json
make quality-architecture
```

Observed baseline on 2026-06-14:

- `scripts/tui_inventory_report.py` matched 182 TUI/interactive files.
- Inventory categories: `browse` 54, `docs` 29, `feature-gated` 45,
  `interactive` 20, `other` 17, `shared` 2, `workbench` 15.
- `helperDrift` is empty, so the current gap is not obvious duplicated helper
  code.
- `make quality-architecture` passes, including feature-matrix policy checks.
- Default Rust features remain `["tui"]`; `browser` is opt-in.
- `--no-default-features` remains unsupported as a release surface and must not
  be advertised as supported.

## Current Surface Inventory

| Surface | Current state | Completion gap |
| --- | --- | --- |
| Shared shell/browser | `common/tui/shell.rs`, `common/browser/session.rs`, `common/browser/detail.rs` are shared primitives. | Not a complete shared review workbench. Keep it as primitives until two or more domains prove the same state/action contract. |
| Dashboard browse | Mature dedicated live/local tree browser. | Keep domain-owned. Remaining work is coverage discipline for local/live parity, destructive confirmation copy, external edit suspend/resume, and detail evidence consistency. |
| Datasource browse and local datasource list interactive | Active implementation with shared detail and Review projection reuse. | Keep strengthening local/live parity, secret redaction tests, and review evidence projection before adding more public flags. |
| Access browse, user browse, team browse | Active implementation plus specialized user/team mutation-capable flows. | Consolidated browse and specialized user/team flows need a documented ownership boundary and shared confirmation/search conventions in tests. |
| Access plan interactive | Uses shared review-contract projections for several action detail rows. | Continue moving only proven generic review rows into shared projection; keep access narrative local. |
| Dashboard summary / inspect workbench | Mature workbench for dashboard/query/governance rows. | Candidate reference for a future workbench contract, but do not force other domains into its state model. |
| Dashboard import interactive | Mature specialized import review. | Needs clearer placement in the mutation review adapter map and tests for shared diff preview coverage, secret safety, and TTY failure behavior. |
| Dashboard policy, impact, topology, dependencies | Mixed maturity specialized review modes with search/filter support. | Need one consistent per-surface checklist for grouping, search state, detail scroll, footer copy, docs, and no-TUI fallback. |
| Status overview and status TUI | Mature document-backed browser with shared search conventions. | Keep as document-backed Tier 3; do not move collection logic into TUI. Add docs parity checks when changing public examples. |
| Snapshot review | Document-backed review over snapshot artifacts. | Keep inventory-oriented. Do not force into mutation envelope. Strengthen datasource review projection reuse and docs parity. |
| Sync audit/review | Internal review surfaces with strong review and diff precedent. | Public docs should steer through `workspace` where appropriate. Shared diff helpers can be reused, but sync-specific checklist semantics stay local. |
| Datasource import `--use-export-org --output-format interactive` | Explicitly unsupported. | Decide whether routed review belongs in `datasource plan --use-export-org --output-format interactive` only, or whether import dry-run should grow a safe interactive review path. |
| Inventory tooling | `scripts/tui_inventory_report.py` exists and reports no helper drift. | It scans English command/user-guide docs and internal docs; it does not yet enforce zh-TW doc parity or CI freshness. |

## Maturity Target

Use the tiers from `docs/internal/tui-architecture-roadmap.md`:

- Tier 1 is allowed only for command-specific selectors that cannot yet share a
  domain document.
- Tier 2 is acceptable for domain workbenches with separated runtime, state, and
  render modules.
- Tier 3 is the target for read-only inventory and status surfaces.
- Tier 4 is the target for mutation review surfaces, but only through internal
  adapters over proven fields.

No new surface should be added below Tier 2 unless the command is explicitly
experimental and has a follow-up migration task.

## Requirements

### R1 - TUI Registry And Ownership

Create a maintained TUI surface registry that records:

- command entrypoint;
- owning domain module;
- tier;
- live/local/snapshot/staged input mode;
- read-only versus mutation-capable behavior;
- feature-gated fallback path;
- primary docs pages in English and zh-TW;
- focused tests and validation command.

The registry may start as Markdown in this file or a small JSON contract under
`scripts/contracts/` if command-doc validation will consume it.

Acceptance:

- every `--interactive` and `output-format interactive` public surface appears
  in the registry;
- every `cfg(feature = "tui")` public dispatch path has a corresponding
  feature-disabled fallback note;
- registry references are stable enough for future review without re-running a
  broad grep.

Unit coverage:

- extend `scripts/test_tui_inventory_report.py` or add a new contract test that
  detects missing registry entries for known public docs examples;
- keep the script read-only and deterministic.

### R2 - Docs Parity And Language Discipline

Normalize public language for interactive surfaces:

- use `--interactive` for full-screen TUI toggles;
- use `--output-format interactive` only where it is already the command's
  established output selector;
- use `--prompt` for non-full-screen confirmation or choice prompts;
- never imply `--no-default-features` is supported;
- keep `sync` internal where user workflows should say `workspace`.

Acceptance:

- English and zh-TW command docs identify the same public interactive entrypoints;
- examples do not mention interactive output where the command only supports
  prompt mode;
- no generated docs are hand-edited.

Unit coverage:

- add command-surface/doc contract checks for interactive examples when the
  registry exists;
- run `make quality-docs-surface`, `make man-check`, and `make html-check`
  after public docs or help changes.

### R3 - Feature-Gated Fallback Consistency

Every public interactive path must fail consistently without `tui`:

- return `tui_feature_required(surface)` when the TUI feature is disabled;
- preserve explicit TTY errors for default-feature builds where the TUI exists
  but stdin/stdout is not a terminal;
- avoid compiling unused TUI imports into unsupported no-default targets.

Acceptance:

- no new no-default warning surface appears in feature-matrix checks;
- fallback messages use the shared error category and the
  `requires the \`tui\` feature` wording;
- TTY-required errors stay specific to the operator action.

Unit coverage:

- focused fallback tests for any new public interactive path;
- `make quality-rust-feature-matrix` for Cargo feature or TUI gating changes;
- `make quality-architecture` for policy drift.

### R4 - Shared Review Projection, Not Shared UI Framework First

Continue sharing review evidence only through typed projections that already
exist in at least two domains:

- `ReviewMutationAction` / `ReviewMutationActionInput` for action/status/blocker
  rows;
- `ReviewDiffModel` for safe live/desired field previews;
- shared read-only browser detail helpers for facts, sections, and review lines.

Do not introduce a public `ReviewRisk`, `ReviewRequest`, or global workbench
state until the fields are proven across domains.

Acceptance:

- public JSON contracts remain unchanged unless a task explicitly changes them;
- domain-local risk/request semantics stay local;
- TUI renderers consume adapters instead of dictating plan-builder shapes.

Unit coverage:

- adapter tests assert action, status, identity, blocker, hints, raw payload, and
  ordering;
- secret-like changed fields such as `password`, `token`, `apiKey`, and
  `secureJsonData.*` are excluded from shared detail and diff previews;
- mutation review tests cover both ready and blocked rows.

### R5 - Search, Navigation, And Footer Consistency

All full-screen TUI surfaces should share the same operator semantics where the
domain does not need a conflicting key:

- `/` forward search;
- `?` reverse search where supported;
- `n` repeat search unless the surface already owns `n` for a stronger action;
- repeat search skips the current row before wrapping;
- footer copy uses compact `Esc/q` exit or close language;
- detail scroll is clamped to rendered content.

Acceptance:

- any deviation is documented in the owning module or docs;
- search prompt, idle search state, and no-match state are visible;
- headers/footers do not advertise inactive controls.

Unit coverage:

- pure state tests for search, repeat, wrap, and no-match behavior;
- render tests for footer/control copy where the surface has custom controls;
- focused tests for detail scroll clamping when a surface has a scrollable pane.

### R6 - Live/Local/Snapshot Review Evidence Parity

Where compatible data exists, local artifact and snapshot browsers should expose
the same safe facts and Review evidence as live browse surfaces.

Acceptance:

- datasource live browse, local list interactive, and snapshot review datasource
  rows share datasource identity and Review projection helpers;
- access plan/import review rows use shared review-contract detail helpers where
  fields are compatible;
- dashboard import and inspect flows expose source/provenance/ownership evidence
  without pretending every source is API-managed.

Unit coverage:

- fixture-backed tests for local and snapshot datasource details;
- access plan tests for shared action, target, context, next-check, and diff
  preview lines;
- dashboard import tests for source/provenance and safe diff preview rows.

### R7 - Mutation-Capable TUI Guardrails

Mutation-capable TUI paths must make the write boundary obvious:

- destructive confirmations separate confirm and cancel controls;
- secret values are never rendered in detail, diff, or confirmation panes;
- dry-run/review and apply phases remain distinct;
- managed/provisioned/Git Sync ownership blockers remain visible before apply.

Acceptance:

- every mutation-capable TUI surface has a blocker/confirmation test;
- local-only review panes do not imply live writes;
- TTY and feature-disabled failures happen before partial terminal state changes.

Unit coverage:

- confirmation key tests for destructive actions;
- secret-redaction tests for changed fields and raw detail rows;
- ownership/provenance blocker tests where dashboard/workspace writes are in
  scope.

### R8 - Inventory Tooling Completion

Upgrade inventory tooling only after the registry shape is agreed:

- include zh-TW docs in the scan;
- emit summary counts by public command surface, not only by file category;
- report docs/code mismatches without failing CI until the registry is stable;
- keep helper-drift detection as advisory.

Acceptance:

- report can answer "which command owns this TUI?" without manual grep;
- report skips generated HTML/man output and Cargo target output;
- JSON output remains stable enough for unit tests.

Unit coverage:

- fixture tests for category detection, zh-TW docs inclusion, and helper-drift
  false-positive avoidance;
- no dependency on a live terminal or Grafana instance.

## Constraints

- Rust-first. TUI implementation work belongs under `rust/src/`.
- Do not inspect or edit `rust/target`.
- Do not hand-edit generated `docs/man/` or `docs/html/`; regenerate when
  generated output is intentionally in scope.
- Keep root `todo.md` short; detailed TUI execution state belongs in this file.
- Preserve public CLI paths unless a task explicitly changes them.
- Preserve public JSON contracts unless a task explicitly changes them.
- Prefer shared taxonomy/rendering over per-command special cases only when the
  shared behavior is proven by at least two domains.
- Default builds include `tui`; browser support remains an opt-in feature.
- Do not claim `--no-default-features` as a supported release artifact.
- No live terminal automation is required for unit acceptance; prefer pure
  state/render tests and targeted parser/help tests.

## Execution Plan

### Phase A - Registry And Parity Audit

Items:

- [x] 1. Build the TUI surface registry from current code/docs.
- [x] 2. Extend inventory script to compare public docs and registry.
- [x] 3. Add zh-TW docs scanning.
- [x] 4. Record mismatches as advisory output first.

Validation:

```bash
python3 scripts/tui_inventory_report.py --json
python3 -m unittest -v scripts/test_tui_inventory_report.py
make quality-docs-surface
```

### Phase B - Feature Fallback And Terminology Cleanup

1. Check every public interactive dispatch path for `tui_feature_required`.
2. Align prompt/TUI wording in help and docs.
3. Add focused fallback tests for newly touched paths.

Validation:

```bash
make quality-rust-feature-matrix
make quality-architecture
cd rust && cargo test --quiet tui_feature
```

### Phase C - Review Projection Consolidation

1. Pick one already-compatible surface pair.
2. Move only shared projection logic into review-contract or review-diff helpers.
3. Keep domain narratives and domain risk semantics local.

Validation:

```bash
cd rust && cargo test --quiet review_contract
cd rust && cargo test --quiet review_diff
cd rust && cargo test --quiet access_plan_interactive
```

### Phase D - Surface-Specific Completion Passes

Work through one surface at a time:

1. dashboard browse/import/inspect;
2. datasource browse/local/snapshot;
3. access browse/plan/user/team;
4. status overview/status TUI;
5. sync audit/review;
6. snapshot review.

Each pass must include:

- current behavior inventory;
- exact missing behavior;
- focused unit tests;
- docs/help update if public behavior changes;
- `make quality-rust` before completion.

## Definition Of Done

The TUI backlog is complete when:

- registry covers every public interactive entrypoint;
- inventory script reports no unexplained code/docs mismatches;
- helper drift remains empty or every candidate has an owner and decision;
- docs parity exists for English and zh-TW command pages;
- feature matrix and architecture checks pass;
- focused tests cover each surface's search/navigation/footer/fallback behavior;
- mutation-capable surfaces cover confirmation, blockers, and secret redaction;
- `todo.md` points here instead of carrying detailed TUI state.

