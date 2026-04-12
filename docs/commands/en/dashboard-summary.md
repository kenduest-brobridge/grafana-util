# dashboard summary

## Purpose
Analyze live Grafana dashboards through the canonical `dashboard summary` command.

## When to use
Use this when you need the same analysis views as the local export-tree flow, but sourced from live Grafana instead of a local export tree. Prefer `dashboard summary --url ...` in new docs and scripts.

## Key flags
- `--page-size`: dashboard search page size.
- `--concurrency`: maximum parallel fetch workers.
- `--org-id`: analyze one explicit Grafana org.
- `--all-orgs`: analyze across visible orgs.
- `--output-format`, `--output-file`, `--interactive`, `--no-header`: output controls.
- `--report-columns`: trim table, csv, or tree-table query output to the selected fields. Use `all` for the full query-column set.
- `--list-columns`: print the supported `--report-columns` values and exit.
- `--progress`: show fetch progress.

## Examples
```bash
# Analyze live Grafana dashboards through the canonical dashboard summary command.
grafana-util dashboard summary --url http://localhost:3000 --basic-user admin --basic-password admin --output-format governance
```

```bash
# Analyze live Grafana dashboards through the canonical dashboard summary command.
grafana-util dashboard summary --url http://localhost:3000 --basic-user admin --basic-password admin --interactive
```

## Related commands
- [dashboard dependencies](./dashboard-dependencies.md)
- [dashboard variables](./dashboard-variables.md)
- [dashboard policy](./dashboard-policy.md)
