# Dashboard clone-folder

## Goal

Add a live dashboard workflow that duplicates dashboards from one Grafana folder
into another folder, with optional target folder creation and recursive subtree
copying.

## Requirements

- Add `grafana-util dashboard clone-folder`.
- Select the source folder by `--source-folder-uid` or `--source-path`.
- Require `--target-folder-uid` for the destination folder.
- Create the target folder only when `--create-target-folder` is set; require
  `--target-folder-title` for root target creation.
- Copy only direct child dashboards by default; include child folders and their
  dashboards only with `--recursive`.
- Generate copied dashboard UIDs deterministically with `--uid-prefix` and
  `--uid-suffix`; default suffix is `-copy`.
- Do not overwrite existing target dashboard UIDs unless `--replace-existing`
  is set.
- Support `--dry-run`, `--table`, `--json`, `--no-header`, and require `--yes`
  for live writes.

## Implementation Plan

1. Add CLI args, help text, command dispatch, and command-surface contract entry.
2. Add a focused clone-folder workflow module that builds a dry-run plan from
   live dashboard summaries and folder metadata.
3. Apply the plan by creating folders parent-first, then posting cloned
   dashboards to `/api/dashboards/db`.
4. Add parser, dry-run, recursive, folder-create, collision, and apply tests.
5. Run focused Rust tests, `make quality-docs-surface`, and a broader Rust test
   pass if the existing dirty tree permits it.

## Acceptance Checks

- [x] `clone-folder --dry-run --json` reports dashboard and folder actions without
  mutating Grafana.
- [x] Live apply refuses to run without `--yes`.
- [x] Recursive runs create child folders before cloned dashboards.
- [x] Existing target dashboard UID conflicts are blocked by default and update only
  with `--replace-existing`.
- [x] CLI help and command-surface contract stay in sync.

## Verification

- `cd rust && cargo test --quiet dashboard_clone_folder --lib`
- `cd rust && cargo test --quiet dashboard_cli_parser_help_mutation_history --lib`
- `make quality-docs-surface`
- `make man-check`
- `make html-check`
- `make quality-ai-workflow`
- `cd rust && cargo test --quiet`

## Review Follow-up

- Fixed target-folder planning so missing live folders/dashboards returned as
  HTTP 404 are treated as create candidates instead of hard errors.
- Required `--target-folder-title` when `--create-target-folder` is used.
- Avoided false folder mismatch blocks when copying into an existing target
  folder without an explicit target title or parent expectation.
