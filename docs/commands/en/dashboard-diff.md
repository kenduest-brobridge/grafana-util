# dashboard diff

## Purpose
Compare local dashboard files against live Grafana dashboards.

## When to use
Use this when you want to see what would change before importing or publishing a dashboard bundle.

## Key flags
- `--input-dir`: compare this export directory against Grafana.
- `--input-format`: choose `raw` or `provisioning`.
- `--import-folder-uid`: override the destination folder UID for the comparison.
- `--context-lines`: unified diff context.
- `--output-format`: choose `text` or `json`.

## Examples
```bash
# Compare local dashboard files against live Grafana dashboards.
grafana-util dashboard diff --url http://localhost:3000 --basic-user admin --basic-password admin --input-dir ./dashboards/raw
```

```bash
# Compare local dashboard files against live Grafana dashboards.
grafana-util dashboard diff --url http://localhost:3000 --basic-user admin --basic-password admin --org-id 2 --input-dir ./dashboards/raw --output-format json
```

## Related commands
- [dashboard export](./dashboard-export.md)
- [dashboard import](./dashboard-import.md)
- [dashboard dependencies](./dashboard-dependencies.md)
- [Shared diff JSON contract](../../user-guide/en/diff-json-contract.md)

CLI schema lookup:

- `grafana-util dashboard diff --help-schema`
- [shared diff JSON contract](../../user-guide/en/diff-json-contract.md)
