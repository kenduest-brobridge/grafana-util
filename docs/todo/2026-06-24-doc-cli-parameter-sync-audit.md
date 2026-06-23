# Doc And CLI Parameter Sync Audit

Date: 2026-06-24

## Problem

Audit whether Markdown docs, handbook/manual content, command references, and
program-supported CLI parameters stay synchronized. Identify missing coverage or
stale generated artifacts before changing public docs or CLI surfaces.

## Scope

- Public docs: `README*.md`, `docs/user-guide/`, `docs/commands/`, generated
  `docs/man/*.1`, and generated `docs/html/`.
- Contract sources: `scripts/contracts/command-surface.json` and
  `scripts/contracts/docs-entrypoints.json`.
- CLI source/help tests under `rust/src/`.
- Do not touch unrelated HTTP transport or dashboard browse cleanup changes.

## Plan

1. Run the repository docs-surface gate and capture whether it already detects
   drift.
2. Inspect the contract scripts to understand what parameter/doc surfaces are
   automatically compared.
3. Compare command-reference Markdown against generated man/html artifacts.
4. Spot-check CLI `--help-full` output against command docs for likely uncovered
   flags or examples.
5. Record any confirmed gaps and recommended next actions.

## Acceptance Checks

- `make quality-docs-surface` result is recorded.
- Generated man/html freshness is checked or a reason is recorded.
- Findings distinguish confirmed drift from residual risk.

## Verification

- `make quality-docs-surface` passed on 2026-06-24. It validated 204 Markdown
  surfaces across README, command docs, user guide, landing, and selected
  internal docs.
- `make man-check` failed on 2026-06-24. Generated manpages are out of date:
  `grafana-util-datasource-list.1`, `grafana-util-status-resource-get.1`, and
  `grafana-util-status-resource-list.1`.
- `make html-check` failed on 2026-06-24. Generated HTML docs are out of date
  for the same command/manpage surfaces: datasource list and status resource
  get/list, including the command index pages `resource.html`.
- Help-vs-Markdown spot check found `docs/commands/{en,zh-TW}/datasource-list.md`
  does not mention the supported `--local`, `--run`, and `--run-id` flags.
- Help-vs-Markdown spot check found `docs/commands/{en,zh-TW}/resource-get.md`
  and `docs/commands/{en,zh-TW}/resource-list.md` mention their command-specific
  supported flags, including `--api-mode`; the stale generated man/html artifacts
  are the gap there.
- Fix verification on 2026-06-24:
  - `make man` regenerated 87 manpages.
  - `make html` regenerated 283 HTML docs.
  - `make quality-docs-surface` passed.
  - `make man-check` passed.
  - `make html-check` passed.

## Findings

1. Confirmed generated-doc drift: man/html output is stale for datasource list
   and status resource get/list.
2. Confirmed source-doc omission: datasource list supports artifact workspace
   local reads via `--local`, `--run`, and `--run-id`, but the English and
   zh-TW command Markdown only documents `--input-dir`/`--input-format` local
   input.
3. Confirmed existing guardrail coverage: `quality-docs-surface` checks command
   examples against the Rust CLI help surface, command-doc locale mirroring,
   local links, removed public command paths, and configured `--help-full`
   support. It does not require every supported flag to appear in each command
   doc's Key flags section.
4. Residual audit risk: a broad help-vs-Markdown comparison found additional
   pages with supported flags not mentioned in source Markdown. Many are
   probably acceptable because command docs use "Key flags" rather than a full
   option reference, but high-churn surfaces such as workspace preview/test/apply
   and dashboard export/summary should be reviewed before release if the goal is
   exhaustive option parity.

## Recommended Next Slice

1. Decide whether this repo wants a stricter optional flag-parity audit for
   command docs, separate from the current example/link/locale guardrail.
