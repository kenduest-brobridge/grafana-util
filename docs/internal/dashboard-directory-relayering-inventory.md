# Dashboard Directory Re-layering Inventory

Last refreshed: 2026-04-27

This inventory records evidence for later dashboard re-layering. It is not a
move plan by itself. Do not split dashboard files only because they are large;
pick one stable responsibility boundary per commit and keep
`commands/dashboard/mod.rs` as the public facade.

## Current Shape

- `rust/src/commands/dashboard/` currently has 327 Rust files and about 92k
  lines.
- Largest files are mostly tests, plus a few implementation hotspots:
  `raw_to_prompt/rust_tests.rs`, `dashboard_plan_rust_tests.rs`,
  `dashboard_list_render_rust_tests.rs`, `export_layout.rs`,
  `import/lookup_folder.rs`, `plan/input.rs`, `authoring/mod.rs`,
  `review_source.rs`, `topology.rs`, `live.rs`, `plan/reconcile.rs`, and
  `live_project_status.rs`.
- Several boundaries are already good enough to preserve: `browse/`,
  `export_prompt/`, `import/`, `plan/`, `raw_to_prompt/`, `governance_gate/`,
  `governance_gate_rules/`, `inspect_workbench/`, `topology/`, and
  `source_loader.rs`.

## Mixed Responsibilities

### Export And Prompt

- `export_scope.rs` mixes org scoping, directory creation, datasource and
  folder inventory, permission export, per-dashboard live fetch/history,
  ownership provenance, raw writes, prompt writes, provisioning writes, and
  metadata emission.
- `export_prompt/` owns shared prompt-lane transformation: datasource slot
  rewrite, variable rewrite, panel type collection, library-panel element
  shaping, and final prompt document output.
- `export_layout.rs` still combines layout CLI workflow, planning, variant
  discovery, metadata probing, folder resolution heuristics, and repair
  decisions. `export_layout_apply.rs` and `export_layout_render.rs` are already
  good lower-level boundaries.
- `raw_to_prompt/resolution.rs` is smaller, but it still combines file loading,
  v2 rejection, portability warnings, placeholder-path preservation,
  datasource rewrite orchestration, synthetic catalog construction, and prompt
  generation.

### Import, Plan, And Review Source

- `plan/input.rs` mixes export-org scope discovery, metadata parsing,
  workspace/root normalization, auth/client setup, file loading, and live-state
  collection.
- `import/dry_run.rs` mixes source resolution, export-org validation, folder
  inventory checks, schema validation, action lookup, folder-path matching,
  target ownership review, and output-record assembly.
- `review_source.rs` bridges source selection, live export side effects,
  temporary directory ownership, saved-artifact loading, and governance/query
  artifact building.
- `dashboard_plan_rust_tests.rs` is a broad integration bucket covering
  reconciliation, render contract shape, ownership blocking, folder-permission
  drift, export-org routing, and live request behavior.

### Browse, Live, Inspect, Status, And Topology

- `browse/` is already decomposed well and should be preserved.
- `live.rs` mixes low-level Grafana reads, folder create/delete helpers, folder
  inventory reconstruction, and datasource inventory shaping.
- `inspect_live.rs` mixes live export argument shaping, fetch strategy
  selection, temp-dir staging, inventory writing, TUI bootstrap, and offline
  analyzer handoff.
- `topology.rs` still concentrates review artifact resolution, optional alert
  contract loading, interactive/non-interactive dispatch, rendering selection,
  and output writing; its `topology/{build,render,browser,types}.rs` internals
  are already a good split.
- `rust/src/grafana/api/project_status_live.rs` is workflow-level status
  collection but currently lives in the generic Grafana API namespace.

## Candidate Next Moves

### 1. Shared Prompt-Lane Transform Boundary - Done 2026-04-27

Moved shared prompt transformation out of root-level `prompt*.rs` into
`dashboard/export_prompt/`.

Evidence:

- Live export calls `build_external_export_document_with_library_panels`.
- Offline `raw_to_prompt` calls the same prompt conversion core without the
  live library-panel path.
- Library-panel export normalization is part of prompt-lane output, not a
  command-specific concern.

Suggested validation:

- Completed focused validation included `raw_to_prompt`,
  `build_external_export_document`,
  `dashboard_export_import_inventory_rust_tests`,
  `collect_library_panel_exports_with_request_records_failures_as_warnings`,
  and `export_diff_rust_tests`.
- `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_export_contract_rust_tests`
- `cargo test --manifest-path rust/Cargo.toml --quiet export_focus_report_path_top_rust_tests`
- `cargo test --manifest-path rust/Cargo.toml --quiet export_layout_tests`
- `cargo test --manifest-path rust/Cargo.toml --quiet raw_to_prompt`
- `cargo test --manifest-path rust/Cargo.toml --quiet`

### 2. Export-Org Source Discovery Boundary

Move import-only export-org scope discovery into a shared dashboard-level module,
for example `dashboard/export_org_scope.rs`.

Evidence:

- `import/validation_org_scope.rs` owns export-org parsing and source-org
  discovery.
- `plan/input.rs` duplicates the same concern for plan input collection.
- Both paths scan `org_*` directories, normalize variant roots, read metadata,
  and derive source org identity.

Suggested validation:

- `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_plan_`
- `cargo test --manifest-path rust/Cargo.toml --quiet import_loaded_source_`
- `cargo test --manifest-path rust/Cargo.toml --quiet source_loader_tests`
- `cargo test --manifest-path rust/Cargo.toml --quiet resolve_dashboard_review_`
- `cargo test --manifest-path rust/Cargo.toml --quiet`

### 3. Status Live Collector Namespace Move

Move `rust/src/grafana/api/project_status_live.rs` under the status runtime or
support layer, for example `rust/src/commands/status/project_status_live.rs`.

Evidence:

- The file describes itself as workflow-level status collection, not a generic
  SDK surface.
- The status command layer already treats it as support for live status.
- This is a namespace correction more than a dashboard split, so it should not
  be mixed with dashboard file carving.

Suggested validation:

- `cargo test --manifest-path rust/Cargo.toml --quiet status`
- `cargo test --manifest-path rust/Cargo.toml --quiet project_status_live`
- `cargo test --manifest-path rust/Cargo.toml --quiet`

## Not Recommended Yet

- Do not split `dashboard/live.rs` only by line count; first decide whether the
  boundary is live read helpers, folder inventory reconstruction, datasource
  inventory shaping, or live mutation support.
- Do not split `dashboard_plan_rust_tests.rs` as a standalone cleanup unless a
  production module boundary changes at the same time or the test split clearly
  maps to existing modules.
- Do not move `source_loader.rs`; it is already a useful shared facade for local
  source normalization.
