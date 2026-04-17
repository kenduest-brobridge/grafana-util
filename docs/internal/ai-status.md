# ai-status.md

Current AI-maintained status only.

- Older trace history moved to [`archive/ai-status-archive-2026-03-24.md`](docs/internal/archive/ai-status-archive-2026-03-24.md).
- Detailed 2026-03-27 entries moved to [`archive/ai-status-archive-2026-03-27.md`](docs/internal/archive/ai-status-archive-2026-03-27.md).
- Detailed 2026-03-28 task notes were condensed into [`archive/ai-status-archive-2026-03-28.md`](docs/internal/archive/ai-status-archive-2026-03-28.md).
- Detailed 2026-03-29 through 2026-03-31 entries moved to [`archive/ai-status-archive-2026-03-31.md`](docs/internal/archive/ai-status-archive-2026-03-31.md).
- Detailed 2026-04-01 through 2026-04-12 entries moved to [`archive/ai-status-archive-2026-04-12.md`](docs/internal/archive/ai-status-archive-2026-04-12.md).
- Keep this file short and current. Additive historical detail belongs in `docs/internal/archive/`.
- Older entries moved to [`ai-status-archive-2026-04-13.md`](docs/internal/archive/ai-status-archive-2026-04-13.md).
- Older entries moved to [`ai-status-archive-2026-04-14.md`](docs/internal/archive/ai-status-archive-2026-04-14.md).
- Older entries moved to [`ai-status-archive-2026-04-15.md`](docs/internal/archive/ai-status-archive-2026-04-15.md).
- Older entries moved to [`ai-status-archive-2026-04-16.md`](docs/internal/archive/ai-status-archive-2026-04-16.md).
- Older entries moved to [`ai-status-archive-2026-04-17.md`](docs/internal/archive/ai-status-archive-2026-04-17.md).

## 2026-04-17 - Finish classic prompt export guardrails
- State: Done
- Scope: Rust dashboard prompt/raw-to-prompt conversion, dashboard export validation, focused prompt conversion tests, maintainer prompt semantics docs, and AI trace docs. README files, Python implementation, and dashboard v2 export lane support are out of scope.
- Baseline: classic prompt export already preserves datasource variables and maps used datasource-variable current values, but constant variables are not exported as `VAR_*` inputs, dashboard v2-shaped files are not explicitly rejected by raw-to-prompt, and library panel references are not surfaced as portability warnings.
- Current Update: Added classic prompt parity for constant variables, kept expression datasource refs outside user-mapped inputs, rejected dashboard v2 resource/spec shapes in raw-to-prompt, and surfaced library panel portability warnings without adding a v2 export lane.
- Result: Focused raw-to-prompt and validation tests, full Rust tests, formatting, AI workflow, and whitespace checks pass.

## 2026-04-17 - Align dashboard prompt external export semantics
- State: Done
- Scope: Rust prompt/raw-to-prompt datasource handling and pre-resolution, focused regression tests, internal prompt-lane semantics note, raw-to-prompt command docs in English and zh-TW, and AI trace docs. README files and Python implementation are out of scope.
- Current Update: Aligned prompt conversion with Grafana UI external-export semantics: concrete datasource refs become `__inputs`, built-in Grafana datasource selectors stay excluded, datasource variables keep their own variable shape, and the converter no longer synthesizes a new datasource variable when the source dashboard did not already have one. Used datasource variables with a concrete current datasource now point `current.value` at the generated `${DS_*}` input while panel and target references stay variable-based.
- Result: The prompt builder no longer over-normalizes single datasource families or turns datasource variable plugin filters into extra import inputs, and raw-to-prompt pre-resolution now preserves `$datasource` placeholders instead of failing inference. Focused raw-to-prompt coverage and full Rust tests pass; the docs record the contract, the known bug pattern, and the later `VAR_*` parity follow-up without naming local machine paths.

## 2026-04-16 - Add dashboard export layout repair
- State: Done
- Scope: Rust dashboard convert CLI surface, local export-layout repair planner/executor, dashboard export index metadata, focused parser/runtime/export tests, command docs/generated docs, command-surface contracts, and AI trace docs. README files, Python implementation, and provisioning layout repair are out of scope.
- Baseline: older dashboard exports can have correct `raw/folders.json` metadata while `raw/` and `prompt/` files are flattened under the leaf `folderTitle`; raw dashboard JSON may also omit `meta.folderUid`, so repair needs stable artifact metadata instead of relying on the dashboard payload alone.
- Current Update: Added `dashboard convert export-layout` with dry-run planning, copy-mode repair, in-place repair with backups, raw/prompt variant selection, folder inventory lookup, updated index paths, report-style summary text/table/csv output with `--show-operations` per-dashboard details, `extraFiles` reporting for unindexed files, case-insensitive/canonical path handling, and index-level `folderUid`/`folderPath` metadata for new exports.
- Result: Focused export-layout/dashboard export index tests, full Rust tests, clippy, docs surface, generated-doc checks, AI workflow, formatting, and whitespace checks pass. Real export smoke reports move=220, same=92, blocked=0, extra=0.

## 2026-04-16 - Mirror dashboard export folder paths
- State: Done
- Scope: Rust dashboard export path assembly, focused dashboard export tests, dashboard export command docs/help, and AI trace docs. README files, Python implementation, and provisioning layout are out of scope.
- Baseline: non-flat `dashboard export` writes raw and prompt files under the dashboard leaf `folderTitle`, even when Grafana reports a nested folder path through folder inventory. This can flatten distinct `Platform / Infra` and `Apps / Infra` folders into the same `Infra/` export directory.
- Current Update: Raw and prompt export paths now use the collected folder inventory full path when available; `--flat` and provisioning layout continue to use their previous behavior.
- Result: Focused dashboard export tests, full Rust tests, clippy, generated docs, docs-surface checks, formatting, AI workflow, and whitespace checks pass.

## 2026-04-15 - Make help-flat terminal-safe
- State: Done
- Scope: Rust `--help-flat` rendering and focused help regression tests. Runtime command behavior, public docs, README files, and Python implementation are out of scope.
- Baseline: `grafana-util --help-flat` renders a padded table with long purpose text, so narrow terminals wrap rows and make the flat command inventory hard to scan.
- Current Update: Changed the flat inventory to one public command path per line and removed KIND/PURPOSE columns so terminal output stays readable; detailed purpose and flags remain available through `<COMMAND> --help`.
- Result: Focused help tests and formatting pass; manual `--help-flat` smoke output now shows one command path per line without columns or ellipses.

## 2026-04-15 - Advertise help-flat in root help
- State: Done
- Scope: Rust grouped root/domain help, focused help regression tests, and command-surface contract metadata. Runtime command behavior, README files, and Python implementation are out of scope.
- Baseline: `grafana-util --help-flat` renders the flat public command inventory, but `grafana-util --help` only advertised `--help-full`. Follow-up review also found supported `access --help-full`, `workspace --help-full`, `workspace --help-schema`, and `dashboard summary --help-full` paths were not discoverable from their adjacent grouped help, while the command-surface contract incorrectly listed unsupported `dashboard --help-full`.
- Current Update: Added the missing help hints to root/dashboard/access/workspace grouped help, extended focused help regressions, and removed unsupported `dashboard` from `help_full_supported`.
- Result: Focused help tests, docs-surface contract checks, formatting, and CLI smoke checks pass.
