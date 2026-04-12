# Docs Site Migration Note

This note records the current direction for a future Docusaurus migration.
It is intentionally not an implementation plan for the current release.

## Decision

Keep the project-specific documentation contract pipeline, but consider moving
the static website shell to Docusaurus in a later branch.

The current custom docs pipeline does two different jobs:

- content correctness and CLI contract checks
- static website rendering and navigation

Docusaurus should only replace the second job after a prototype proves it can
serve the same public docs structure.

## Keep

- `docs/user-guide/{en,zh-TW}/` as handbook source
- `docs/commands/{en,zh-TW}/` as command-reference source
- `docs/man/*.1` generation
- `scripts/contracts/*.json`
- `make quality-docs-surface`
- `make man` and `make man-check`
- CLI help / command surface consistency checks

These parts encode repository-specific behavior that a generic docs framework
does not replace.

## Candidate For Docusaurus

- `docs/html/` website shell
- sidebar and table-of-contents rendering
- language switching
- docs landing and version entrypoints
- responsive layout, theme, and search
- versioned site publishing

## Migration Shape

A future spike should add a separate `website/` or `docs-site/` prototype first.
That prototype should consume the existing Markdown sources and generated command
Markdown instead of introducing a second source of truth.

Do not combine the migration with broad handbook rewrites. Content-layer changes
and site-framework changes should remain separately reviewable and separately
rollbackable.

## Acceptance Criteria For A Future Switch

- English and Traditional Chinese docs build from the existing sources.
- `dev` and release-version entrypoints remain explicit.
- Command reference pages still pass contract checks before site build.
- Generated manpages remain independent from the website framework.
- GitHub Pages output is reproducible in CI.
