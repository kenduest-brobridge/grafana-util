# dashboard history

## Purpose
List, restore, or export live dashboard revision history for one dashboard UID.

## When to use
Use this when you need to inspect earlier dashboard versions, recover a known-good revision, or export dashboard history into a reusable artifact for review or CI.

Restore creates a new latest revision instead of overwriting the historical version you picked. The old version stays in history.

## Key flags
- `--dashboard-uid`: the dashboard UID whose revision history you want.
- `--limit`: how many recent versions to include in list or export views.
- `--version`: the historical version number to restore.
- `--message`: revision message for the new restored revision.
- `--dry-run`: preview a restore without changing Grafana.
- `--yes`: confirm a real restore.
- `--output-format`: render list or restore output as text, table, json, or yaml.
- `--output`: write exported history artifacts to a JSON file.
- `--overwrite`: replace an existing export artifact.

## Restore semantics

- The selected historical version is copied forward as a new current revision.
- The original historical version remains in the dashboard history chain.
- `--dry-run` shows the restore intent without changing Grafana.
- A real restore requires confirmation with `--yes`.

## JSON contracts for CI

Use the built-in schema help when automation needs stable routing rules:

- `grafana-util dashboard history --help-schema`
- `grafana-util dashboard history list --help-schema`
- `grafana-util dashboard history restore --help-schema`
- `grafana-util dashboard history export --help-schema`

Routing rule:

1. inspect `kind`
2. confirm `schemaVersion`
3. only then branch on nested fields

Practical mapping:

- `dashboard history list --output-format json` -> `grafana-util-dashboard-history-list`
- `dashboard history restore --dry-run --output-format json` -> `grafana-util-dashboard-history-restore`
- `dashboard history restore --output-format json` -> the same contract shape, but live execution still creates a new latest revision
- `dashboard history export --output ./cpu-main.history.json` -> `grafana-util-dashboard-history-export`

Top-level keys worth remembering:

- list -> `kind`, `schemaVersion`, `toolVersion`, `dashboardUid`, `versionCount`, `versions`
- restore -> `kind`, `schemaVersion`, `toolVersion`, `mode`, `dashboardUid`, `currentVersion`, `restoreVersion`, `currentTitle`, `restoredTitle`, optional `targetFolderUid`, `createsNewRevision`, `message`
- export -> `kind`, `schemaVersion`, `toolVersion`, `dashboardUid`, `currentVersion`, `currentTitle`, `versionCount`, `versions`

## Examples
```bash
# Purpose: List the last 20 dashboard revisions as a table for review.
grafana-util dashboard history list --url http://localhost:3000 --basic-user admin --basic-password admin --dashboard-uid cpu-main --limit 20 --output-format table
```

```bash
# Purpose: Restore one historical dashboard revision as a new latest Grafana version.
grafana-util dashboard history restore --url http://localhost:3000 --basic-user admin --basic-password admin --dashboard-uid cpu-main --version 17 --message "Restore known good CPU dashboard after regression" --dry-run --output-format table
```

```bash
# Purpose: Export the recent dashboard history revisions into a reusable JSON artifact.
grafana-util dashboard history export --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --dashboard-uid cpu-main --limit 20 --output ./cpu-main.history.json
```

## Before / After

- **Before**: dashboard revision recovery often meant guessing which old version to trust, then manually rebuilding it or editing JSON by hand.
- **After**: one history namespace covers list, restore, and export, so operators can inspect old versions, recover a known-good revision, and hand the same artifact to review or CI.

## What success looks like

- list output shows the revision numbers and messages you expected for the target dashboard UID
- restore dry-run clearly shows the version that would become the new latest revision
- a real restore leaves the old version in history and adds a new current revision
- export writes a reusable JSON artifact that can be inspected later without Grafana

## Failure checks

- if list output is empty, confirm the dashboard UID and whether your credentials can see that dashboard
- if restore fails, verify that the target version exists and that you supplied `--yes` for a live restore
- if export writes the wrong file or seems stale, confirm the output path and whether `--overwrite` was intended

## Related commands
- [dashboard list](./dashboard-list.md)
- [dashboard inspect-live](./dashboard-inspect-live.md)
- [dashboard inspect-export](./dashboard-inspect-export.md)
- [dashboard review](./dashboard-review.md)
- [dashboard export](./dashboard-export.md)
