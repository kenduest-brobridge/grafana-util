# dashboard import

## Purpose
Import dashboard JSON files through the Grafana API.

## When to use
Use this when you have a local export tree for dashboards that are owned by Grafana's API state and need to push dashboards back into Grafana, either live or as a dry run. This command consumes `raw/` or `provisioning/` inputs; it does not consume the Grafana UI `prompt/` lane.

Do not use this command to bypass source-owned dashboards. If the target dashboard is managed by Grafana Git Sync or file provisioning, change the repository, PR, or provisioning source and let that workflow deploy it instead of forcing a direct dashboard API import.

## Before / After
- **Before**: import is a blind replay step, and you find folder, org, or schema problems only after the live call.
- **After**: import becomes a controlled replay step that can be previewed first with `--dry-run`, then executed with clearer intent.

## Key flags
- `--input-dir`: source directory for raw or combined export input.
- `--input-format`: choose `raw` or `provisioning`.
- `--local`, `--run`, `--run-id`: read the source dashboard lane from the artifact workspace instead of passing `--input-dir`.
- `--org-id`, `--use-export-org`, `--only-org-id`, `--create-missing-orgs`: control cross-org routing.
- `--import-folder-uid`: force a destination folder UID.
- `--ensure-folders`, `--replace-existing`, `--update-existing-only`: control import behavior.
- `--require-matching-folder-path`, `--require-matching-export-org`, `--strict-schema`, `--target-schema-version`: safety checks.
- `--import-message`: revision message stored in Grafana.
- `--interactive`, `--dry-run`, `--table`, `--json`, `--output-format`, `--output-columns`, `--list-columns`, `--no-header`, `--progress`, `--verbose`: preview and reporting controls. Use `--output-columns all` for the full dry-run table.

## What success looks like
- dry-run shows the expected create/update actions before you touch the live server
- dry-run also surfaces live dashboard target evidence when Grafana already owns the UID, including provisioned or managed-state warnings
- the destination org and folder routing are explicit enough to review
- the chosen input lane matches the replay goal: `raw` or `provisioning`, not `prompt`
- Git Sync or file-provisioned targets are treated as source-owned and routed back to their repository or provisioning workflow

## Failure checks
- if folder or org placement looks wrong, verify the routing flags before re-running live import
- if the replay looks too destructive, stop at `--dry-run` and inspect the export tree first
- if Grafana reports a provisioned or Git Sync-managed dashboard at the target UID, do not retry with a direct import; update the owning source and redeploy through that lane
- if the schema check blocks replay, confirm whether the source tree needs normalization before import

## Examples
```bash
# Import dashboard JSON files through the Grafana API.
grafana-util dashboard import --url http://localhost:3000 --basic-user admin --basic-password admin --input-dir ./dashboards/raw --replace-existing
```

```bash
# Import dashboard JSON files through the Grafana API.
grafana-util dashboard import --url http://localhost:3000 --basic-user admin --basic-password admin --input-dir ./dashboards/raw --dry-run --table
```

```bash
# Dry-run import from the latest profile artifact workspace dashboard run.
grafana-util dashboard import --profile prod --local --dry-run --table
```

## Related commands
- [dashboard export](./dashboard-export.md)
- [dashboard convert raw-to-prompt](./dashboard-convert-raw-to-prompt.md)
- [dashboard diff](./dashboard-diff.md)
- [dashboard publish](./dashboard-publish.md)
