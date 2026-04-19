# datasource export

## Purpose
Export live Grafana datasource inventory as normalized JSON plus a derived
provisioning projection.

## When to use
Use this when you need a local datasource bundle for later inspection, diff, or
import.

## Key flags
- `--output-dir`: target directory for the export tree.
- `--org-id`: export one explicit Grafana org.
- `--all-orgs`: export each visible org into per-org subdirectories. Requires Basic auth.
- `--overwrite`: replace existing files.
- `--without-datasource-provisioning`: skip the provisioning variant.
- `--run`: write under an artifact workspace run when `--output-dir` is omitted. Use `timestamp` for a new timestamped run or `latest` to reuse the latest recorded run.
- `--run-id`: write under one explicit artifact workspace run id when `--output-dir` is omitted.
- `--dry-run`: preview what would be written.

## Artifact workspace output

When `--output-dir` is omitted, `datasource export` writes to:

```text
<artifact_root>/<profile-or-default>/runs/<run-id>/datasources/
```

The artifact root comes from `artifact_root` in `grafana-util.yaml`. If it is unset, the default is `.grafana-util/artifacts` next to the config file. Use the root `--config <file>` flag or `GRAFANA_UTIL_CONFIG` when the profile file is not in the current directory.

Use `--run timestamp` for a fresh export, `--run-id <name>` for a named run, and `datasource list --local --run latest` to read the latest recorded datasource lane later.

## Examples
```bash
# Export live Grafana datasource inventory as normalized JSON plus provisioning files.
grafana-util datasource export --url http://localhost:3000 --basic-user admin --basic-password admin --output-dir ./datasources --overwrite
```

```bash
# Export live Grafana datasource inventory as normalized JSON plus provisioning files.
grafana-util datasource export --url http://localhost:3000 --basic-user admin --basic-password admin --all-orgs --output-dir ./datasources --overwrite
```

```bash
# Export datasource inventory into the profile artifact workspace with a timestamped run id.
grafana-util datasource export --profile prod --run timestamp --overwrite
```

## Before / After

- **Before**: live datasource state was easy to lose because the export shape was not normalized or easy to reuse later.
- **After**: one export gives you a local bundle that is stable enough for inspection, diff, and import.

## What success looks like

- the export tree is complete enough to inspect later without Grafana
- normalized JSON and provisioning projection stay aligned with the source
  inventory
- the provisioning projection is treated as derived output, and it is only
  Grafana-ready once any secret placeholders are resolved
- the bundle is ready for diff or import without extra hand cleanup

## Failure checks

- if the export tree is missing org data, confirm the org scope and whether the credentials can see it
- if `--all-orgs` fails, use Basic auth and verify that the account can see each target org
- if the bundle looks stale, verify the export directory and whether `--overwrite` was used intentionally
- if you expected secret plaintext in the export, remember that Grafana live datasource APIs do not return it; export only captures config and `secureJsonDataPlaceholders`
- if the provisioning output still has placeholders, treat it as an intermediate
  artifact and resolve secrets before trying to apply it as Grafana

## Related commands
- [datasource list](./datasource-list.md)
- [datasource import](./datasource-import.md)
- [datasource diff](./datasource-diff.md)
