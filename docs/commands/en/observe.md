# `grafana-util observe`

## Root

Purpose: read live and staged Grafana state through one canonical observe surface.

When to use: when you want readiness, overview, snapshot, or direct live reads without moving into mutation work.

Description: `observe` is the task-first read-only entrypoint. Use `live` for current-state gating, `staged` for artifact review, `overview` for project-wide summaries, `snapshot` for bundle-style review, and `resource` for direct live reads.

Examples:

```bash
# Purpose: Read current state through the live gate.
grafana-util observe live --profile prod --output-format yaml
```

```bash
# Purpose: Review staged artifacts before any mutation path.
grafana-util observe staged --desired-file ./desired.json --output-format json
```

```bash
# Purpose: Summarize staged exports across the estate.
grafana-util observe overview --dashboard-export-dir ./dashboards/raw --alert-export-dir ./alerts --output-format table
```

```bash
# Purpose: Open the live overview in the interactive workbench.
grafana-util observe overview live --url http://localhost:3000 --basic-user admin --basic-password admin --output-format interactive
```

Related commands: `grafana-util export`, `grafana-util change`, `grafana-util config profile`.

## `live`

Purpose: render a live readiness view from Grafana read surfaces.

When to use: when you need the current estate state and want to gate follow-on work.

Examples:

```bash
# Purpose: live.
grafana-util observe live --profile prod --output-format table
```

```bash
# Purpose: live.
grafana-util observe live --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --output-format json
```

## `staged`

Purpose: render a readiness gate from staged artifacts.

When to use: when you want to inspect exported files, desired state, or bundle inputs before apply.

Examples:

```bash
# Purpose: staged.
grafana-util observe staged --dashboard-export-dir ./dashboards/raw --desired-file ./desired.json --output-format table
```

```bash
# Purpose: staged.
grafana-util observe staged --dashboard-provisioning-dir ./dashboards/provisioning --alert-export-dir ./alerts --output-format interactive
```

## `overview`

Purpose: summarize staged artifacts or open the live overview.

When to use: when you want one estate-wide summary across dashboards, alerts, datasources, access, or bundle inputs.

Examples:

```bash
# Purpose: overview.
grafana-util observe overview --dashboard-export-dir ./dashboards/raw --alert-export-dir ./alerts --output-format table
```

```bash
# Purpose: overview.
grafana-util observe overview --source-bundle ./sync-source-bundle.json --target-inventory ./target-inventory.json --output-format text
```

```bash
# Purpose: overview live.
grafana-util observe overview live --profile prod --output-format yaml
```

## `snapshot`

Purpose: review or export a snapshot-style artifact bundle.

When to use: when you want a portable artifact package for inspection, review, or handoff.

Examples:

```bash
# Purpose: snapshot.
grafana-util observe snapshot --help
```

## `resource`

Purpose: inspect one live Grafana resource directly.

When to use: when you already know the resource kind and want `get`, `list`, `describe`, or `kinds` style reads.

Examples:

```bash
# Purpose: resource.
grafana-util observe resource list --help
```
