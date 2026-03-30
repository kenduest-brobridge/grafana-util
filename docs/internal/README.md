# Internal Docs Index

`docs/internal/` now keeps only the maintainer docs that still act as current
entrypoints or stable architecture maps. Older plans, unwired scaffolds,
backlogs, market analysis, and progress snapshots have been moved into
`docs/internal/archive/`.

## Keep In The Root

- `ai-status.md`
  - current change trace and active maintainer notes
- `ai-changes.md`
  - current summarized change log for meaningful behavior or architecture work
- `overview-architecture.md`
  - source-of-truth maintainer map for `grafana-util overview`
- `project-status-architecture.md`
  - project-wide status-model architecture above any single command surface

## Internal Examples

- `examples/datasource_live_mutation_api_example.py`
- `examples/datasource_live_mutation_safe_api_example.py`

## Archive Policy

- Move any unwired plan, dated execution note, backlog, proposal, or historical
  implementation scaffold into `archive/` unless it is still the current source
  of truth.
- Move dated architecture reviews and generated reference snapshots into
  `archive/` as well; keep only current maintainer entrypoints in the root.
- Keep core architecture docs in the root only when maintainers should still
  read them before editing code.
- Prefer consolidating small one-off maintainer references into
  `docs/DEVELOPER.md`, `docs/overview-rust.md`, or `docs/overview-python.md`
  instead of creating new standalone index pages.
