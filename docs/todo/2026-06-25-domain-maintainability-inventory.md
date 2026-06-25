# Domain Maintainability Inventory

**Goal:** Record the dashboard/access/sync maintainability inventory required before any broader domain refactor.

**Current Gate State:**
- `scripts/rust_maintainability_report.py --root rust/src` reports no oversized production files after the P1 splits.
- `make quality-architecture` passes.
- Dashboard, access, and sync remain large domains, but their current risk is domain/test volume rather than a direct architecture gate failure.

## Dashboard

**Inventory Evidence:**
- `rust/src/commands/dashboard`: 327 files, 94k lines.
- Largest current hotspots are test-heavy:
  - `raw_to_prompt/rust_tests.rs`: 1,156 lines.
  - `topology_impact_rust_tests.rs`: 966 lines.
  - `dashboard_plan_rust_tests.rs`: 940 lines.
  - `dashboard_list_render_rust_tests.rs`: 916 lines.
  - `export_focus_report_path_top_rust_tests.rs`: 910 lines.
- Largest production file in the sampled hotspots is `export_layout.rs` at 783 lines, under the current production warning limit.

**Decision:**
- Do not start a dashboard-wide refactor from this inventory alone.
- Next dashboard refactor must be tied to a concrete feature/change pressure area, preferably one of:
  - export/history helper ownership,
  - browse state/render test consolidation,
  - topology/impact test fixture structure.

**Next Slice Trigger:**
- Create a separate dated plan before editing dashboard files.
- The plan must include exact files, current caller paths, expected test filters, and rollback scope.

## Access

**Inventory Evidence:**
- `rust/src/commands/access`: 109 files, 32k lines.
- Largest hotspots:
  - `access_plan_tests.rs`: 1,048 lines.
  - `access_service_account_org_rust_tests.rs`: 943 lines.
  - `access_runtime_user_rust_tests.rs`: 939 lines.
  - `user_mutation.rs`: 785 lines.
  - `user_workflows_import_export_import.rs`: 761 lines.
  - `user_browse_render.rs`: 744 lines.

**Decision:**
- Do not introduce a generic resource abstraction yet.
- Access duplication should be proven across user/team/service-account flows before extracting shared helpers.
- Resource-specific behavior remains valuable because Grafana user/team/service-account constraints differ.

**Next Slice Trigger:**
- If future work touches user import/export or service-account/org behavior, first compare the corresponding team/service-account path and extract only repeated control flow with identical semantics.

## Sync

**Inventory Evidence:**
- `rust/src/commands/sync`: 83 files, 24k lines.
- Largest hotspots:
  - `live_rust_tests.rs`: 968 lines.
  - `cli_apply_review_exec_apply_rust_tests.rs`: 731 lines.
  - `rust_tests.rs`: 729 lines.
  - `task_first.rs`: 699 lines.
  - `audit_tui.rs`: 632 lines.
  - `review_tui.rs`: 629 lines.
- Production files are below current architecture thresholds.

**Decision:**
- Do not split sync tests solely by line count.
- Keep `sync/mod.rs` as a facade.
- Split tests only when a production contract boundary is clarified by a behavior change or repeated review friction.

**Next Slice Trigger:**
- If changing live apply, staged document lineage, or review/audit TUI behavior, write a narrow plan that pairs production boundary changes with matching test movement.

## Python Legacy

**Decision:**
- No Python changes were needed for this Rust maintainability pass.
- Keep Python read-only unless a future task explicitly targets Python parity, packaging, or compatibility smoke behavior.
