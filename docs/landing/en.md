# Review Grafana changes before you apply them

Live inventory, export/import, diff, change preview, and safe apply in one workflow. This page keeps the most common entry points; use the jump menu for the full handbook and command index.

- [Start with the handbook](../user-guide/en/index.md)
- [Open command reference](../commands/en/index.md)
- [Troubleshooting](../user-guide/en/troubleshooting.md)

## Recommended Starts

If you are not sure which document to read first, choose one of these paths.

### First time using the CLI

Confirm `grafana-util --version`, run `grafana-util status live` as the first read-only check, then save a reusable connection profile after the connection works.

- [Getting Started](../user-guide/en/getting-started.md)
- [New User Path](../user-guide/en/role-new-user.md)
- [Profile Command Reference](../commands/en/profile.md)

### Daily operations

Check live or staged state first, then decide whether to export, compare, review, or apply a change.

- [SRE / Ops Path](../user-guide/en/role-sre-ops.md)
- [Workspace Review & Status](../user-guide/en/status-workspace.md)
- [Status Command Reference](../commands/en/status.md)

### Automation and CI

Use this path for pipelines, release automation, and repeatable validation flows. It focuses on inputs, outputs, failure handling, and stable syntax.

- [Automation / CI Path](../user-guide/en/role-automation-ci.md)
- [Technical Reference](../user-guide/en/reference.md)
- [Command Reference](../commands/en/index.md)

## Common Jobs

If you already know the work you need to finish, start here.

### Dashboard

Browse, export, summarize, review, patch, publish, and capture dashboards.

- [Dashboard Management](../user-guide/en/dashboard.md)
- [Dashboard Command Reference](../commands/en/dashboard.md)

### Data source and alerting

Manage Grafana integrations, alert rules, contact points, and governance checks.

- [Data source Management](../user-guide/en/datasource.md)
- [Alerting Governance](../user-guide/en/alert.md)
- [Alert Command Reference](../commands/en/alert.md)

### Access and credentials

Manage orgs, teams, service accounts, tokens, and permission-oriented changes.

- [Access Management](../user-guide/en/access.md)
- [Access Command Reference](../commands/en/access.md)

## Complete Reference

Use these when you need full coverage instead of a curated starting path.

### Full handbook

Use the handbook for workflow context, operating order, and recommended reading paths.

- [Operator Handbook](../user-guide/en/index.md)

### Full command reference

Use the command reference for command pages, subcommand routing, options, and examples.

- [Command Reference](../commands/en/index.md)
- [grafana-util(1)](../man/grafana-util.html)

### Source and releases

Use the repository for change history, release notes, and issue tracking.

- [GitHub repository](https://github.com/kenduest-brobridge/grafana-util)
- [GitHub releases](https://github.com/kenduest-brobridge/grafana-util/releases)
- [Issue tracker](https://github.com/kenduest-brobridge/grafana-util/issues)

## Maintainer

Maintainer docs stay in the repository docs rather than the public handbook.

- [Developer guide](../DEVELOPER.md)
