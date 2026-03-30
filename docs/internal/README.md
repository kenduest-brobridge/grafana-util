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
- `architecture-review-2026-03-27.md`
  - project-level architecture review and structural risk scan
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
- Keep core architecture docs in the root only when maintainers should still
  read them before editing code.
