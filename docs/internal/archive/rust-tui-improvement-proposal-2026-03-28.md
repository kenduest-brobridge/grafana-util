# Rust TUI Improvement Proposal

Status: current proposal for the Rust-only TUI surfaces as of 2026-03-28.

## Working Model

- Visual thesis: calm, dense, operator-first terminal surfaces where focus and state carry the strongest contrast instead of borders or chrome.
- Content plan: shell framing, primary workspace, secondary inspector, final action/status surface.
- Interaction thesis:
  - Keep one clearly active pane at a time.
  - Let modal or viewer states take over only when they unlock a real task.
  - Make status lines answer where the operator is, what is blocked, and what action is next.

## Goal

Make the Rust TUI surfaces feel like one operator console instead of several adjacent tools with different layout grammar.

This proposal focuses on:

- shared TUI shell rules
- dashboard inspect workbench refactor priorities
- sync review layout simplification
- datasource browse consistency cleanup

## Current Assessment

The current Rust TUI surfaces are already useful, but they are converging from different directions:

- `dashboard inspect` is evolving into a full workbench with its own group, view, search, and full-detail viewer model.
- `sync review` is a staged workflow surface with checklist, diff panes, preview, and controls.
- `datasource browse` is a live browser/editor flow with list, detail, edit, delete, and search.

Each surface is individually reasonable. The main issue is that they still expose different framing and emphasis rules:

- different header heights and semantics
- different footer/control density
- different active-pane signaling
- different status placement
- different confirm/detail/modal behavior

That makes the TUI harder to learn and easier to overgrow.

## Top Risks

### 1. Dashboard Inspect Workbench Is Becoming Its Own Mini-App

The inspect workbench is now the strongest TUI subsystem and the biggest structural risk.

Relevant files:

- `rust/src/dashboard/inspect_workbench_state.rs`
- `rust/src/dashboard/inspect_workbench_render.rs`
- `rust/src/dashboard/inspect_workbench_support.rs`
- `rust/src/dashboard/inspect_workbench_content.rs`

Current strengths:

- clear three-pane model
- useful mode/view grouping
- searchable detail viewer
- meaningful summary header

Current risk:

- state owns too many concerns at once
- viewer/search/navigation logic are still close together
- more modes or actions could easily turn this into a large facade again

Recommendation:

- keep splitting by responsibility, not by file count
- treat workbench document, pane state, and modal/viewer state as separate concerns
- do not let future inspect/governance additions bypass the document model

### 2. Sync Review Gives Too Much Permanent Space To Secondary Information

Relevant files:

- `rust/src/sync/review_tui.rs`
- `rust/src/sync/review_tui_helpers.rs`

Current strengths:

- real staged workflow
- split diff view is useful
- checklist model is clear

Current risk:

- `Plan Status`, selection preview, and control help permanently take too much viewport budget
- operations and diff panes lose space to supporting information
- the primary task is not visually dominant enough

Recommendation:

- keep the main workspace dominant
- demote secondary summary/help surfaces when diff mode is active
- use the footer/status line for compact guidance before reserving full panels

### 3. Datasource Browse Mixes Inspection And Confirmation Too Abruptly

Relevant files:

- `rust/src/datasource_browse_render.rs`
- `rust/src/datasource_browse_tui.rs`

Current strengths:

- list/detail flow is easy to understand
- grouped org rows help with all-org browsing
- edit and delete are present without excessive chrome

Current risk:

- the detail pane changes role too abruptly between inspect and destructive confirmation
- confirm states do not yet feel like a consistent shell-level pattern shared with other TUIs

Recommendation:

- standardize confirm overlays and blocking states across browse/review flows
- keep detail panes for context and use modal/overlay language for irreversible actions

### 4. TUI Shell Grammar Is Not Yet Shared

Across `dashboard`, `sync`, and `datasource`, there is still no single shell pattern for:

- header title and summary lines
- active-pane emphasis
- footer controls
- status wording
- empty-state wording
- blocked-state wording
- confirm or modal overlays

Recommendation:

- define one shared shell grammar and apply it incrementally
- optimize for consistency before visual flourish

## Shared TUI Shell

This should be the default shell pattern for Rust TUI surfaces.

### Layout

- Header: 2-4 lines, only orientation and scope.
- Workspace: the dominant area for list, diff, table, or inspector.
- Footer: compact control map plus current status.
- Overlay: search, confirm, or full viewer only when necessary.

### Header Rules

- Title should name the workflow, not the implementation.
- First line should answer scope or source.
- Second line should answer counts or current state.
- Do not place long help text in the header.

Good examples:

- `Inspect Workbench`
- `Sync Review`
- `Datasource Browser`

### Workspace Rules

- one pane must be visually primary
- only one active pane should use strong highlight treatment
- side panes should support the task, not compete with it
- when a full-screen viewer opens, it should replace complexity rather than add to it

### Footer Rules

- keep one compact line for controls
- keep one line for current status or blocking guidance
- move repetitive help out of permanent panels when it can fit the footer

### Status Rules

Status should answer one of these:

- current focus
- current mode/view
- blocking condition
- next useful action

Bad status lines:

- generic loaded/success text with no next action
- repeated controls that already appear in the footer

## Refactor Priorities

### 1. Dashboard Inspect Workbench Refactor

Priority: highest

Target outcome:

- `InspectWorkbenchDocument` remains the only source for rendered content structure
- navigation state stops absorbing more content logic
- modal/viewer state becomes more explicit and isolated

Recommended slices:

1. separate shell state from viewer/search state
2. move footer/status composition into dedicated helpers
3. keep group/view semantics visible in titles and status text
4. reserve future filter/action additions for document-driven wiring only

### 2. Sync Review Layout Simplification

Priority: high

Target outcome:

- operations and diffs become the unmistakable primary workspace
- preview/help/status stop competing for fixed vertical space

Recommended slices:

1. compress `Plan Status` into header/footer status text
2. collapse preview into a lighter inspector when diff mode is active
3. keep full control documentation off-screen unless requested
4. standardize blocked/review guidance wording with promotion and preflight text

### 3. Datasource Browse Consistency Pass

Priority: medium

Target outcome:

- browse, edit, search, and delete all feel like one shell
- destructive confirmation uses the same language as other TUI overlays

Recommended slices:

1. preserve list/detail as the core layout
2. convert delete confirmation into a clearer overlay pattern
3. keep detail pane consistently informational
4. align footer/status copy with shared shell rules

## What Not To Do Yet

- do not add more colors or decorative borders to create hierarchy
- do not add more permanent panes to expose secondary information
- do not build a separate TUI component system before shell rules stabilize
- do not introduce terminal animation unless it sharpens focus or blocking state

## Success Criteria

The TUI direction is improving if:

- operators can scan the first screen and know the current task immediately
- the active pane is always obvious
- status lines consistently explain the next action or blocker
- confirm and search overlays feel reusable across workflows
- new TUI features land through shared shell rules instead of bespoke layout patterns

## Recommended Order

1. `dashboard inspect workbench` boundary cleanup
2. shared TUI shell helper conventions
3. `sync review` layout simplification
4. `datasource browse` consistency pass

## Summary

The Rust TUI does not need a visual rewrite first. It needs a stronger shared shell and stricter workspace hierarchy.

The most important next step is to keep the inspect workbench from becoming a monolith while using it to define the shell rules that `sync review` and `datasource browse` can share later.
