# Dashboard export Git-friendly filenames and history

## Goal

Make `dashboard export` output stable and readable for Git-managed repositories,
including classic dashboard JSON, resource API exports, and history artifacts.

## Requirements

- Use stable dashboard UID filenames for classic raw, prompt, and provisioning
  exports, for example `raw/Infra/cpu-main.json`.
- Keep dashboard title visible in JSON content and `index.json`, not as the
  canonical filename.
- Keep non-flat folder mirroring unchanged so folder moves remain visible in
  Git history.
- Keep `--flat` behavior, but write classic files as `<uid>.json` directly
  under each selected variant directory.
- Keep `resource-v1` export paths based on resource `metadata.name`; resource
  API identity is not title-derived.
- Change `dashboard export --include-history` to write one canonical file per
  history version under `history/<uid>/vN.history.json`.
- Do not write aggregate history bundles from batch export by default, avoiding
  duplicate dashboard JSON.
- Keep `dashboard history export --output <file>` unchanged as the explicit
  single-file aggregate workflow.
- Do not add a new layout or filename mode parameter; keep the CLI simple.

## Parameter Behavior

- `--include-history` writes per-version history files under
  `history/<uid>/vN.history.json`.
- `--overwrite` refreshes export artifacts and removes stale
  `history/<uid>/v*.history.json` files for dashboards being exported while
  leaving unrelated files untouched.
- `--dry-run` reports planned dashboard/history writes and stale history cleanup
  without mutating files.
- `--flat` writes classic dashboard files as `<uid>.json`.
- `--all-orgs` and `--org-id` keep existing org scoping; same dashboard UID in
  different org scopes must not be silently merged by history readers.
- `--resource-format v1` keeps resource-v1 paths based on `metadata.name`.

## Implementation Plan

1. Update classic dashboard path helpers to derive filenames from UID instead
   of title.
2. Keep `index.json` and root index path fields synchronized with actual
   emitted paths.
3. Preserve resource-v1 path behavior based on `metadata.name`.
4. Build the existing `DashboardHistoryExportDocument` once per dashboard, then
   split it into single-version documents during batch export.
5. Add a narrow stale-file cleanup helper for `history/<uid>/v*.history.json`,
   used only with `--overwrite` and not with `--dry-run`.
6. Update local history import-dir readers to merge per-version physical files
   into one logical source per export scope and dashboard UID.
7. Preserve old aggregate `.history.json` read compatibility and deduplicate
   repeated versions when aggregate and per-version artifacts coexist.
8. Update help text and command-surface expectations for UID filenames and
   per-version history artifacts.

## Acceptance Checks

- [x] Classic export writes UID paths such as `raw/Infra/cpu-main.json`,
  `prompt/Infra/cpu-main.json`, and
  `provisioning/dashboards/Infra/cpu-main.json`.
- [x] Dashboard title changes update JSON content and index metadata without
  changing file paths.
- [x] `--flat` writes `raw/cpu-main.json` and `prompt/cpu-main.json`.
- [x] Resource-v1 keeps `resource-v1/objects/<folder-path>/<metadata.name>.json`
  paths.
- [x] `--include-history` writes `history/cpu-main/v22.history.json` and no
  aggregate `history/cpu-main.history.json` bundle.
- [x] Each per-version history file contains exactly one version and preserves
  that version dashboard JSON.
- [x] `--overwrite` removes stale `v*.history.json` files only for dashboards
  being exported and preserves unrelated files.
- [x] `--dry-run` reports planned history writes and cleanup without mutation.
- [x] `history list --input-dir` and `history diff --input-dir` work with
  per-version, legacy aggregate, and mixed history artifacts.
- [x] Dashboard import/discovery ignores `history/` and `*.history.json`.

## Verification

- `cd rust && cargo test --quiet dashboard_cli_parser_help_list_export --lib`
- `cd rust && cargo test --quiet dashboard_export_contract --lib`
- `cd rust && cargo test --quiet export_focus_report_path_top --lib`
- `cd rust && cargo test --quiet history_cli --lib`
- `cd rust && cargo test --quiet dashboard_export_import_inventory --lib`
- `make quality-docs-surface`
- `cd rust && cargo test --quiet`
