# dashboard review

## Purpose
Review one local dashboard JSON file without touching Grafana.

## When to use
Use this when you want a local read-only check of a dashboard draft before publishing or importing it.

## Key flags
- `--input`: dashboard JSON file to review. Use `-` to read one wrapped or bare dashboard JSON document from standard input.
- `--output-format`: choose `text`, `table`, `csv`, `json`, or `yaml`.
- `--json`, `--table`, `--csv`, `--yaml`: direct output selectors.

## Examples
```bash
# Review one local dashboard JSON file without touching Grafana.
grafana-util dashboard review --input ./drafts/cpu-main.json
```

```bash
# Review one local dashboard JSON file without touching Grafana.
grafana-util dashboard review --input ./drafts/cpu-main.json --output-format yaml
```

```bash
# Review one generated dashboard from standard input.
jsonnet dashboards/cpu.jsonnet | grafana-util dashboard review --input - --output-format json
```

## Related commands
- [dashboard patch](./dashboard-patch.md)
- [dashboard publish](./dashboard-publish.md)
- [dashboard dependencies](./dashboard-dependencies.md)
