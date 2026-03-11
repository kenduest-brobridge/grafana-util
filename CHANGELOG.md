# Changelog

## 2026-03-11

- `2972101` Add Grafana dry-run and diff workflows
  - Added versioned export metadata for dashboard export roots and variants.
  - Added dashboard `diff` as an explicit CLI subcommand.
  - Added alerting `--diff-dir` to compare local exports with live Grafana state.
  - Added non-mutating `--dry-run` import behavior to both Python CLIs.

- `56ac3d2` Reorganize docs and split Python Rust tests
  - Renamed the Python unittest files so their implementation type is obvious from the filename.
  - Standardized the Python test names as `test_python_dashboard_cli.py`, `test_python_alert_cli.py`, and `test_python_packaging.py`.
  - Moved Rust inline unit tests out of production modules and into dedicated `*_rust_tests.rs` files.
  - Updated module wiring so Rust test files are still compiled and discovered from their parent modules.
  - Refreshed maintainer docs and repo guidance so targeted test commands point at the new file names.
  - Preserved Python `unittest` discovery and Rust `cargo test` behavior after the layout change.

- `4991900` Add build Makefile for Python and Rust
  - Added a root `Makefile` to give the repository one shared build and test entrypoint.
  - Introduced `build-python`, `build-rust`, `build`, `test-python`, `test-rust`, `test`, and `help` targets.
  - Standardized where Python and Rust artifacts land so operators know where to find wheels and release binaries.
  - Updated README, Traditional Chinese README, maintainer docs, and repo instructions to document the new commands.
  - Extended `.gitignore` so Python build outputs created by the Makefile do not pollute the worktree.
  - Reduced the need to remember separate `pip` and `cargo` command sequences for common tasks.

- `54f44ba` Rename dashboard export variant flags
  - Renamed the dashboard raw export suppression flag from the shorter form to `--without-dashboard-raw`.
  - Renamed the prompt export suppression flag to `--without-dashboard-prompt` for the same reason.
  - Updated the Python dashboard CLI, the Rust dashboard CLI, and the matching test coverage together.
  - Refreshed operator docs so command examples no longer use the old short flag names.
  - Made the option names self-describing when read out of context in scripts or CI jobs.
  - Kept both implementations aligned so Python and Rust users see the same public flag surface.

- `257150f` Port Grafana API flows into Rust
  - Replaced the earlier Rust scaffolding and stubs with real Grafana HTTP request flows.
  - Added a shared Rust HTTP client layer and connected it to the dashboard and alerting modules.
  - Implemented dashboard raw export/import behavior in Rust, including live API fetching and write paths.
  - Implemented prompt export datasource rewrite behavior so Rust can generate Grafana-style import placeholders.
  - Implemented alerting export/import behavior for rules, contact points, mute timings, notification policies, and templates.
  - Added Rust-side linked-dashboard handling so alert rules can carry and repair dashboard linkage metadata.
  - Brought the Rust implementation much closer to Python parity instead of remaining helper-only code.

- `c4bcc7b` Package Grafana utilities for install
  - Added Python packaging metadata so the repository can be installed cleanly on other systems.
  - Created installable console scripts for `grafana-utils` and `grafana-alert-utils`.
  - Moved the real Python implementation into the `grafana_utils/` package instead of keeping logic only in top-level scripts.
  - Kept `cmd/` wrappers as thin source-tree launchers so repo-local execution still works during development.
  - Updated install, usage, and maintainer docs to explain package install paths and command names.
  - Made it possible to install into a global environment or a user-local Python environment with the normal `pip` flow.

- `79f9b7e` Make Grafana HTTP transport replaceable
  - Added a swappable transport abstraction for Grafana HTTP requests.
  - Enabled both `requests` and `httpx` transport implementations behind the same interface.
  - Kept client code focused on Grafana API behavior instead of library-specific HTTP details.
  - Added tests to verify transport selection and injection.

- `b4065b2` Refactor Grafana CLI readability
  - Split large Python flows into smaller helpers with clearer names.
  - Reduced nesting in import/export paths and separated orchestration from detail logic.
  - Made datasource rewrite and linked-dashboard repair flows easier to scan.
  - Preserved behavior while improving code readability for maintainers.

- `89299ba` Move Grafana CLIs into cmd
  - Moved the repo-run Python entrypoints into `cmd/`.
  - Updated unit tests to load the new command paths.
  - Refreshed documentation so repo usage examples point at `cmd/`.
  - Kept the implementation separate from the thin command wrappers.

- `20e1fff` Split Chinese README guide
  - Separated the English and Traditional Chinese READMEs into distinct files.
  - Added top-level navigation links between the two README variants.
  - Restored the English README as the main English-facing document.
  - Improved language-specific onboarding from the repository homepage.

- `464514d` Add Chinese README summary
  - Added a Traditional Chinese summary near the top of the project docs.
  - Clarified the repository purpose for Chinese readers at a glance.
  - Explained the difference between the dashboard and alerting tools in Chinese.
  - Improved README accessibility without changing code behavior.

- `0594a6d` Clarify README purpose
  - Reworked the README opening to explain what the repository is for.
  - Brought backup, migration, and re-import use cases to the top.
  - Clarified the split between dashboard and alerting workflows.
  - Reduced the need to infer project purpose from option sections alone.

- `e921e06` add LICENSE
  - Restored the repository `LICENSE` file from the main branch.
  - Reintroduced an explicit license file into the working tree.
  - Ensured the repository includes the expected legal metadata.

## 2026-03-10

- `06a71a9` Reorganize project docs and tests
  - Reworked documentation structure to separate operator-facing and maintainer-facing content.
  - Cleaned up test organization so it better matched the evolving project layout.
  - Refreshed internal tracking files to reflect the reorganized structure.
  - Improved navigation across the main project documents.

- `1e563b6` Clarify documentation policy
  - Defined where public usage docs should live versus internal notes.
  - Clarified the intended roles of README, DEVELOPER, and internal trace files.
  - Reduced ambiguity about where future project guidance should be added.
  - Established a clearer documentation maintenance boundary.

- `2ffd80a` Extend Grafana alert utility mapping and templates
  - Improved alert-rule remapping behavior for linked dashboards and panels.
  - Extended template handling so update paths preserve Grafana expectations.
  - Expanded alert import logic for more realistic migration scenarios.
  - Added coverage around the new alert mapping behavior.

- `ff80bb4` Add developer maintenance notes
  - Added `DEVELOPER.md` to capture internal architecture and maintenance guidance.
  - Documented important API usage and implementation tradeoffs for maintainers.
  - Created a stable place for operational notes that do not belong in the public README.
  - Improved long-term maintainability of the project documentation.

- `6b0bac9` Extend Grafana alerting backup compatibility
  - Broadened alerting backup and restore support across more resource shapes.
  - Improved compatibility between exported tool documents and import expectations.
  - Reduced failure cases when moving alerting resources between Grafana instances.
  - Strengthened the backup/restore story for alerting workflows.

- `b45ba4c` Add explicit dashboard subcommands and export-dir flag
  - Introduced explicit dashboard `export` and `import` subcommands.
  - Switched the dashboard export option name to `--export-dir`.
  - Reduced argument ambiguity by separating export-only and import-only flags.
  - Updated tests and docs to match the new command layout.

- `0a87701` Ignore local Grafana artifacts
  - Added ignore rules for local files and generated Grafana artifacts.
  - Reduced accidental commits of temporary or environment-specific output.
  - Improved cleanliness of the git working tree during local testing.

- `2fcf3ac` Refresh alerting utility documentation and status
  - Updated alerting usage examples and project status notes.
  - Synchronized internal change tracking with the current alerting capabilities.
  - Improved discoverability of supported alerting resource behavior.
  - Kept project docs aligned with the evolving alert utility.

- `dc2ee24` Make Grafana utilities RHEL 8 compatible and default to localhost
  - Reworked Python annotations so the code stays parseable on Python 3.6 syntax.
  - Locked the CLIs to a localhost Grafana default URL instead of a hardcoded remote target.
  - Added tests that validate Python 3.6 grammar compatibility.
  - Updated documentation to note RHEL 8 support expectations.

- `6d79b85` Refine alert utility change log
  - Improved the recorded history for recent alert utility work.
  - Made internal status notes clearer and more traceable.
  - Tightened the wording around what changed in alerting support.
  - Helped future maintenance by preserving better internal context.

- `c411ffd` Add Grafana alert rule utility
  - Added the standalone Python alerting CLI for Grafana resources.
  - Implemented export/import support for core alerting provisioning resources.
  - Added linked-dashboard metadata handling for alert-rule portability.
  - Added unit tests and documentation for the new alerting tool.

- `6637d4b` Document dashboard datasource export flow
  - Documented how prompt exports rewrite datasource references.
  - Explained the role of Grafana `__inputs` placeholders in prompt exports.
  - Made the dashboard export behavior easier to understand for operators.
  - Captured the datasource rewrite flow for future maintenance.

- `0a9db4c` Add Grafana dashboard export/import utility
  - Added the initial Python dashboard export/import CLI.
  - Established dual export variants under `raw/` and `prompt/`.
  - Added unit tests for export, import, auth, and datasource placeholder behavior.
  - Documented the basic dashboard backup and restore workflow.
