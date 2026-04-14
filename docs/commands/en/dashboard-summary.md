# dashboard summary

## Purpose
Summarize live Grafana dashboards through the canonical `dashboard summary` command.

## When to use
Use this when you need the same summary and governance views as the local export-tree flow, but sourced from live Grafana instead of a local export tree. Prefer `dashboard summary --url ...` in new docs and scripts.

## Before / After

- **Before**: live dashboard review starts by exporting files or clicking through the UI before you know which dashboards, variables, or datasources matter.
- **After**: `dashboard summary` reads live Grafana directly and produces reviewable summary, governance, dependency, or query outputs before you decide the next command.

## Key flags
- `--page-size`: dashboard search page size.
- `--concurrency`: maximum parallel fetch workers.
- `--org-id`: summarize one explicit Grafana org.
- `--all-orgs`: summarize across visible orgs.
- `--output-format`, `--output-file`, `--interactive`, `--no-header`: output controls.
- `--report-columns`: trim table, csv, or tree-table query output to the selected fields. Use `all` for the full query-column set.
- `--list-columns`: print the supported `--report-columns` values and exit.
- `--progress`: show fetch progress.

## Examples
```bash
# Summarize live Grafana dashboards through the canonical dashboard summary command.
grafana-util dashboard summary --url http://localhost:3000 --basic-user admin --basic-password admin --output-format governance
```

```bash
# Summarize live Grafana dashboards through the canonical dashboard summary command.
grafana-util dashboard summary --url http://localhost:3000 --basic-user admin --basic-password admin --interactive
```

## Related commands
- [dashboard dependencies](./dashboard-dependencies.md)
- [dashboard variables](./dashboard-variables.md)
- [dashboard policy](./dashboard-policy.md)

## What success looks like

- you can explain the live dashboard estate without exporting it first
- governance or query output is stable enough to hand to policy, impact, CI, or another operator
- the command choice is clear: live sources use `dashboard summary`; local export trees use `dashboard dependencies`

## Failure checks

- if the result looks smaller than the UI, check org scope, credentials, and folder permissions first
- if a downstream command cannot consume the output, confirm whether you emitted `governance-json`, `queries-json`, or a human-readable table
- if the review is meant to be offline or reproducible without Grafana access, export first and switch to `dashboard dependencies`
