# `grafana-util status snapshot`

## Root

Purpose: export and review Grafana snapshot inventory bundles.

When to use: when you want a local snapshot root that captures dashboard and datasource inventory for later inspection.

Description: open this page when you need an offline snapshot of Grafana inventory that can be reviewed later without talking to the server again. This snapshot surface is useful for handoff, backup, incident review, or any workflow where you want one local artifact before moving into deeper analysis. Snapshot exports stage dashboard and datasource lanes under one root and also write `snapshot-metadata.json` so later tooling can discover those lanes without guessing paths.

## Before / After

- **Before**: snapshot-style review usually means re-querying Grafana or opening a pile of dashboards and datasources one by one.
- **After**: export first, then review the local bundle as a repeatable artifact without touching the live server again.

## What success looks like

- you can hand off a snapshot root and another operator can inspect it without asking for live access
- export output is a durable artifact instead of a temporary UI session
- the snapshot root includes lane metadata that later analysis can resolve without rescanning the whole tree
- review output is clear enough to feed into a follow-up analysis or incident note

## Failure checks

- if a snapshot export looks empty, verify the auth profile or live connection before assuming the source system is blank
- if review output looks surprising, confirm that you are pointing at the intended snapshot directory
- if automation reads the output, keep the chosen `--output-format` explicit so the downstream parser knows the contract

Key flags: the root command is a namespace; the operational flags live on `export` and `review`. The shared root flag is `--color`.

Examples:

```bash
# Export a local snapshot bundle from live Grafana.
grafana-util status snapshot export --url http://localhost:3000 --basic-user admin --basic-password admin --output-dir ./snapshot
```

```bash
# Review the exported snapshot bundle as JSON.
grafana-util status snapshot review --input-dir ./snapshot --output-format json
```

Related commands: `grafana-util status overview`, `grafana-util status staged`, `grafana-util workspace package`.

## `export`

Purpose: export dashboard and datasource inventory into a local snapshot bundle.

When to use: when you need a local snapshot root that can be reviewed without Grafana access.

What gets written:

- `snapshot/dashboards/`
- `snapshot/datasources/`
- `snapshot/snapshot-metadata.json`

Key flags: `--output-dir`, `--overwrite`, `--prompt`, `--run`, `--run-id`, plus the shared Grafana connection and auth flags. `--prompt` opens a terminal multi-select prompt so you can choose which lanes to export before the snapshot starts. The datasource lane exports config and any `secureJsonDataPlaceholders`, but it does not export datasource plaintext secrets because Grafana live APIs do not return them.

When `--output-dir` is omitted, snapshot export writes the snapshot root under the artifact workspace run:

```text
<artifact_root>/<profile-or-default>/runs/<run-id>/
```

That means the snapshot lanes remain directly under the run root, for example `dashboards/`, `datasources/`, and `access/users/` when those lanes are exported. The artifact root comes from `artifact_root` in `grafana-util.yaml`, or defaults to `.grafana-util/artifacts` next to the config file.

Examples:

```bash
# export.
grafana-util status snapshot export --url http://localhost:3000 --basic-user admin --basic-password admin --output-dir ./snapshot
```

```bash
# export.
grafana-util status snapshot export --profile prod --output-dir ./snapshot --overwrite
```

```bash
# choose snapshot lanes from a terminal prompt before export.
grafana-util status snapshot export --profile prod --prompt --output-dir ./snapshot
```

```bash
# export a snapshot into the profile artifact workspace with a timestamped run id.
grafana-util status snapshot export --profile prod --run timestamp --overwrite
```

Related commands: `snapshot review`, `workspace package`, `status overview`.

## `review`

Purpose: review a local snapshot inventory without touching Grafana.

When to use: when you want to inspect an exported snapshot root as table, csv, text, json, yaml, or interactive output.

Review summary focuses on the exported dashboard and datasource lanes present in the snapshot root.

Key flags: `--input-dir`, `--interactive`, `--output-format`, `--run`, `--run-id`.

When `--input-dir` is omitted, `--run latest` reads the latest recorded artifact workspace run and `--run-id <name>` reads one explicit run.

Examples:

```bash
# review an exported snapshot root as table, csv, text, json, yaml, or interactive output.
grafana-util status snapshot review --input-dir ./snapshot --output-format table
```

```bash
# review an exported snapshot root as table, csv, text, json, yaml, or interactive output.
grafana-util status snapshot review --input-dir ./snapshot --interactive
```

```bash
# review the latest artifact workspace snapshot run without naming its directory.
grafana-util status snapshot review --run latest --output-format table
```

Related commands: `snapshot export`, `status overview`, `status staged`.
