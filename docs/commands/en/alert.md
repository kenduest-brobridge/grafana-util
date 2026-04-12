# `grafana-util alert`

## Purpose

Run the alerting command surface for listing, exporting, importing, diffing, planning, applying, deleting, and authoring Grafana alert resources.

## When to use

- Export local alert bundles from Grafana.
- Import or diff alert bundles against live Grafana state.
- Build and apply a reviewed alert management plan.
- Initialize a staged desired-state tree before authoring managed alert files.
- Author staged rules, contact points, routes, and templates.
- List live alert rules, contact points, mute timings, and templates.

## Description
Open this page when the work is about Grafana alerting as a full workflow, not just one command. The `alert` namespace keeps the path shallow, and the help groups commands by task so you can move from inventory to backup and compare, desired-state authoring, and review without switching command roots.

This is the right entrypoint for SREs, platform operators, and anyone who wants to understand how alert rules, notification routing, contact points, and desired-state review fit together before diving into one exact subcommand.

## Workflow lanes

- Inventory: `list-rules`, `list-contact-points`, `list-mute-timings`, `list-templates`, `delete`
- Backup & Compare: `export`, `import`, `diff`
- Author Desired State: `init`, `add-rule`, `clone-rule`, `add-contact-point`, `set-route`, `preview-route`, `new-rule`, `new-contact-point`, `new-template`
- Review & Apply: `plan`, `apply`

Choose this page when alert work spans rules, routes, contact points, templates, or the initial desired-state scaffold and you want the workflow before the flags.

## Before / After

- **Before**: alert work is often scattered across rule editors, export scripts, and route tweaks without one grouped path from inventory to reviewed apply.
- **After**: the `alert` namespace keeps inventory, authoring, backup, planning, and apply in one place so you can inspect first, then change deliberately.

## What success looks like

- you can tell whether the task belongs to inventory, backup and compare, desired-state authoring, or review before you open a subcommand
- a plan or export can move through review without losing routing context
- the same alert flow can be repeated in CI or during incident follow-up

## Failure checks

- if an inventory command returns less than expected, confirm whether the auth scope is wide enough for the org or folder you need
- if a review or apply step behaves strangely, inspect the alert plan JSON before assuming the CLI is wrong
- if the result is going to automation, set `--output-format` explicitly so the downstream step knows the contract

## Key flags

- `--profile`, `--url`, `--token`, `--basic-user`, `--basic-password`
- `--prompt-password`, `--prompt-token`, `--timeout`, `--verify-ssl`
- the root `alert` command is a namespace; operational flags live on the leaf subcommands

## Auth notes

- Prefer `--profile` for normal alert review and apply loops.
- Use Basic auth when you need broader org visibility or admin-backed inventory.
- Token auth works best for scoped single-org reads or automation where the token permissions are already well understood.

## Examples

```bash
# Purpose: Inspect the grouped alert help before choosing a lane.
grafana-util alert --help
```

```bash
# Purpose: Initialize a staged alert desired-state tree.
grafana-util alert init --desired-dir ./alerts/desired
```

```bash
# Purpose: List live rules from a saved profile.
grafana-util alert list-rules --profile prod --json
```

```bash
# Purpose: Export alert resources for review or migration.
grafana-util alert export --url http://localhost:3000 --basic-user admin --basic-password admin --output-dir ./alerts --overwrite
```

```bash
# Purpose: Preview the route shape before mutating live alert routing.
grafana-util alert preview-route --desired-dir ./alerts/desired --label team=platform --severity critical --output-format json
```

```bash
# Purpose: Build a reviewed plan from desired alert files.
grafana-util alert plan --desired-dir ./alerts/desired --output-format json
```

## Related commands

### Inventory

- [alert list-rules](./alert-list-rules.md)
- [alert list-contact-points](./alert-list-contact-points.md)
- [alert list-mute-timings](./alert-list-mute-timings.md)
- [alert list-templates](./alert-list-templates.md)
- [alert delete](./alert-delete.md)

### Backup & Compare

- [alert export](./alert-export.md)
- [alert import](./alert-import.md)
- [alert diff](./alert-diff.md)

### Author Desired State

- [alert init](./alert.md)

- [alert add-rule](./alert-add-rule.md)
- [alert clone-rule](./alert-clone-rule.md)
- [alert add-contact-point](./alert-add-contact-point.md)
- [alert set-route](./alert-set-route.md)
- [alert preview-route](./alert-preview-route.md)
- [alert new-rule](./alert-new-rule.md)
- [alert new-contact-point](./alert-new-contact-point.md)
- [alert new-template](./alert-new-template.md)

### Review & Apply

- [alert plan](./alert-plan.md)
- [alert apply](./alert-apply.md)

### Related Surface

- [access](./access.md)
