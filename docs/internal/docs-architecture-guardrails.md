# Docs Architecture Guardrails

Use this page when you are changing handbook, manual, command-reference, or
trace docs and want the docs layer to stay stable as code changes.

## Docs Layers

Keep each docs layer doing one job:

- `public handbook/manual`
  - owns stable workflow, user intent, decision paths, and recommended
    journeys
  - does not duplicate large flag inventories or CLI topology
- `command docs`
  - owns command names, flags, examples, and per-command reference detail
  - does not become a second handbook
- `command surface contract`
  - owns machine-readable public command paths, legacy replacements,
    command-doc routing, and `--help-full` / `--help-flat` support
  - lives in `scripts/contracts/command-surface.json`
- `generated docs`
  - owns derived man pages and HTML output
  - does not replace the source Markdown
- `internal docs`
  - owns maintainer contracts, architecture notes, routing, and validation
  - does not duplicate operator-facing workflow prose
- `trace docs`
  - owns concise current status and condensed change history
  - does not become a long-form design spec

## Rules

- keep manual docs focused on intent, workflow, and decisions
- keep command docs focused on flags, commands, and exact examples
- do not use unstable examples as the main manual path
- do not put CLI topology or namespace wiring in the manual
- do not repeat long flag tables in both manual and command docs
- when CLI behavior changes, update command reference and source snippets first
- when public command paths, legacy replacements, or `--help-full` / `--help-flat` support
  change, update `scripts/contracts/command-surface.json` and run
  `make quality-docs-surface`
- when a public command path is retired, record the removed public path list in
  `scripts/contracts/command-surface.json` so docs checks can block stale
  references in public docs and evidence tests
- public live analysis stays `grafana-util dashboard summary`; local/export
  analysis stays `grafana-util dashboard dependencies`
- historical `analyze` wording is acceptable only in archive or trace docs
- keep manual updates limited to user journeys, decision tables, and stable
  narrative examples
- prefer semantic examples and task-based routing over large copy-heavy
  rewrites
- keep derived docs derived; update the source Markdown first and regenerate
  output after

## Good Update Order

When a CLI or workflow change lands, update in this order:

1. source code and focused tests
2. command-reference source snippets
3. command-surface contract, if command paths or help support changed
4. handbook/manual user journeys or decision tables
5. generated docs, if the source layer changed
6. internal routing or trace docs, if the maintainer contract changed

## Anti-Patterns

Treat these as signs the docs boundary is drifting:

- handbook pages that restate every flag from command docs
- manual pages that explain parser topology or namespace layout
- examples that rely on unstable edge cases or implementation quirks
- generated pages edited as the primary source
- internal docs that repeat operator walkthroughs instead of maintainer rules
- trace docs that grow into design notes or implementation plans
- command examples that bypass the command-surface contract or mention removed
  roots without a replacement
- public docs or docs evidence tests that reintroduce removed public dashboard
  analysis paths instead of the current `summary` / `dependencies` split

## Maintenance Rule

If a code change forces the manual to change too often, tighten the doc split:

- move exact command syntax back to command docs
- keep the manual at the level of stable intent and workflow
- add a decision table when users need help choosing a command path
- add semantic examples when users need a concrete first success path
