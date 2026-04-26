# `grafana-util workspace`

## Root

Purpose: scan, test, preview, package, and apply one local Grafana workspace.

When to use: when you already have a local repo root or staged package and want to understand it, validate it, preview the impact, package it for handoff, or apply a reviewed result.

Description: `workspace` is the user-facing local package lane. Start with `scan` to discover inputs, use `test` to check whether they are structurally safe, use `preview` to see what would change, and use `apply` only after review. Use `ci` for lower-level contract checks and handoff documents.

Git Sync and file-provisioned dashboards are source-owned. `workspace scan`, `test`, and `preview` can inspect those trees, but live dashboard writes must go through the Git repository/PR or provisioning workflow rather than `workspace apply --execute-live`.

First-run path:

1. `workspace scan`
2. `workspace test`
3. `workspace preview`
4. `workspace apply`

Key inputs: an optional workspace path, `--desired-file`, `--dashboard-export-dir`, `--dashboard-provisioning-dir`, `--alert-export-dir`, `--target-inventory`, `--availability-file`, `--mapping-file`, `--fetch-live`, `--live-file`, `--preview-file`, `--approve`, `--execute-live`, and `--output-format`.

Examples:

```bash
# Run this example command.
grafana-util workspace scan ./grafana-oac-repo
# Run this example command.
grafana-util workspace test ./grafana-oac-repo --fetch-live --output-format json
# Run this example command.
grafana-util workspace preview ./grafana-oac-repo --fetch-live --profile prod
# Run this example command.
grafana-util workspace package ./grafana-oac-repo --output-file ./workspace-package.json
# Run this example command.
grafana-util workspace apply --preview-file ./workspace-preview.json --approve --execute-live --profile prod
```

Related commands: `grafana-util status`, `grafana-util export`, `grafana-util config profile`.

## `scan`

Purpose: discover what is in one local workspace or staged package.

## `test`

Purpose: validate whether the local workspace is structurally safe to continue.

## `preview`

Purpose: show what would change from the current workspace inputs.

## `apply`

Purpose: turn a reviewed preview into staged or live apply output. Do not use live apply to overwrite Git Sync-managed or file-provisioned dashboards; update the owning source instead.

## `package`

Purpose: package dashboards, alerts, datasources, and metadata into one local handoff artifact.

## `ci`

Purpose: expose lower-level contract checks for CI and automation.
