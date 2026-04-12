# dashboard export

## Purpose
Export dashboards to `raw/`, `prompt/`, and `provisioning/` files, with optional `history/` artifacts.

## When to use
Use this when you need a local export tree for later import, review, diff, or file-provisioning workflows. Add `--include-history` when you also need dashboard revision-history artifacts under each exported org scope. The `prompt/` lane is for Grafana UI import, not dashboard API import; use `dashboard convert raw-to-prompt` when you need to convert ordinary or raw dashboard JSON into prompt JSON.

## Before / After
- **Before**: export is a one-off action, and you only discover later whether the tree is reviewable or reusable.
- **After**: export becomes the first artifact in a repeatable workflow that can feed inspect, diff, dry-run import, and Git review.

## Key flags
- `--output-dir`: target directory for the export tree.
- `--org-id`: export one explicit Grafana org.
- `--all-orgs`: export each visible org into per-org subdirectories. Prefer Basic auth.
- `--flat`: write files directly into each export variant directory.
- `--overwrite`: replace existing export files.
- `--without-raw`, `--without-prompt`, `--without-provisioning`: skip a variant.
- `--include-history`: write dashboard revision-history artifacts under a `history/` subdirectory for each exported org scope.
- `--provider-name`, `--provider-org-id`, `--provider-path`: customize the generated provisioning provider file.
- `--provider-disable-deletion`, `--provider-allow-ui-updates`, `--provider-update-interval-seconds`: tune provisioning behavior.
- `--dry-run`: preview what would be written.

## Notes
- Writes `raw/`, `prompt/`, and `provisioning/` by default.
- Use Basic auth with `--all-orgs`.
- Use `--flat` for files directly under each variant directory.
- Use `--include-history` to add `history/` under each exported org scope.
- The provider file is `provisioning/provisioning/dashboards.yaml`.
- Keep `raw/` for API import or diff, `prompt/` for UI import, and `provisioning/` for file provisioning.

## Export lane differences

`dashboard export` writes three dashboard representations by default. That is not just extra output; it separates three different Grafana entrypoints before the next person has to guess whether one JSON file is safe to import, review, or mount into provisioning.

Think of `raw/` as the field negative. It is closest to what the API returned and is the lane you keep when the export needs to be traceable, reviewable, repeatable, and fit for Git history. Start here for backup, diff, review, dry-run import, and `dashboard import`.

Think of `prompt/` as the handoff copy for a person using the Grafana UI. It is shaped for the Import dashboard flow in the browser, not for automated API replay. If you only have ordinary dashboard JSON or a `raw/` file, convert it with `dashboard convert raw-to-prompt` before handing it to a UI user.

Think of `provisioning/` as the deployment projection Grafana reads from disk. It includes dashboard JSON plus provider YAML, and it belongs in Grafana provisioning paths for startup or provisioning reload. It is not an interactive import path, and it should not replace `raw/` as the main review or replay source.

| Lane | Contents | Best next step | Notes |
| :--- | :--- | :--- | :--- |
| `raw/` | Dashboard JSON read from the Grafana API with the context needed for API import. | `dashboard import`, `dashboard diff`, `dashboard review`, `dashboard dependencies`, `workspace scan/test/preview`. | This is the best format for CLI replay, diff, review, and Git history. If you are not sure which lane to keep, keep `raw/`. |
| `prompt/` | Dashboard JSON shaped for Grafana UI import prompts. | Human handoff, Grafana UI Import dashboard flow, or converting ordinary dashboard JSON into a UI-importable shape. | Do not use this as the input for API replay through `dashboard import`. Use `dashboard convert raw-to-prompt` when you only have ordinary or raw dashboard JSON. |
| `provisioning/` | File provisioning tree with dashboard JSON plus provider YAML. | Mount into Grafana provisioning paths or keep as a GitOps-style file-provisioning source. | The provider YAML defaults to `provisioning/provisioning/dashboards.yaml`. This lane is for Grafana provisioning reload/startup, not interactive API import. |

Common choices:

- For backup, review, diff, or dry-run import, start from `raw/`.
- For a human UI import handoff, deliver `prompt/`.
- For Grafana file provisioning, use `provisioning/` and confirm the provider settings match the deployment path.
- If you only need one output shape, use `--without-raw`, `--without-prompt`, or `--without-provisioning` to reduce noise.

## What success looks like
- a `raw/` tree exists for API replay and deeper inspection
- a `prompt/` tree exists when you need a cleaner handoff for UI-style import
- a `history/` tree exists under each exported org scope when you pass `--include-history`
- the export tree is stable enough to commit, diff, or inspect before mutation

## Failure checks
- if dashboards are missing, check org scope before suspecting the exporter
- if multi-org output looks partial, check whether the credential can really see every org
- if expected history artifacts are missing, confirm that you passed `--include-history` and are checking the right org scope
- if the next step is import, confirm whether you should continue from `raw/` or `prompt/`

## Examples
```bash
# Export dashboards to `raw/`, `prompt/`, and `provisioning/` files.
grafana-util dashboard export --profile prod --output-dir ./dashboards --overwrite
```

```bash
# Export dashboards to `raw/`, `prompt/`, and `provisioning/` files.
grafana-util dashboard export --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --output-dir ./dashboards --overwrite
```

```bash
# Export dashboards to `raw/`, `prompt/`, and `provisioning/` files.
grafana-util dashboard export --url http://localhost:3000 --basic-user admin --basic-password admin --all-orgs --output-dir ./dashboards --overwrite
```

```bash
# Export dashboards plus per-org revision-history artifacts into a reusable tree.
grafana-util dashboard export --url http://localhost:3000 --basic-user admin --basic-password admin --all-orgs --include-history --output-dir ./dashboards --overwrite
```

## Related commands
- [dashboard dependencies](./dashboard-dependencies.md)
- [dashboard import](./dashboard-import.md)
- [dashboard diff](./dashboard-diff.md)
- [dashboard convert raw-to-prompt](./dashboard-convert-raw-to-prompt.md)
- [dashboard history](./dashboard-history.md)
