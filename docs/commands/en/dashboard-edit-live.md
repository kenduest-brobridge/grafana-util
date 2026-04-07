# dashboard edit-live

## Purpose
Fetch one live dashboard into an external editor, then preview, save, or apply the edited result.

## When to use
Use this when Grafana already has the closest source dashboard and you want to edit that payload directly, while keeping a clear choice between preview-only, saved draft, and direct live apply flows.

## Key flags
- `--dashboard-uid`: live Grafana dashboard UID to edit.
- `--output`: local draft path to write after editing. When omitted, `edit-live` stays ephemeral and does not create a local draft file.
- `--apply-live`: write the edited payload back to Grafana instead of writing a local draft.
- `--publish-dry-run`: when `--output` is set, save the local draft and immediately run the equivalent `dashboard publish --dry-run` preview.
- `--yes`: required with `--apply-live` because it mutates live Grafana.
- `--message`: revision message used when `--apply-live` writes back to Grafana.
- `--profile`, `--url`, `--token`, `--basic-user`, `--basic-password`: live Grafana connection settings.

## Notes
- The command opens `$VISUAL`, then `$EDITOR`, then falls back to `vi`.
- With neither `--output` nor `--apply-live`, the command keeps the edit session ephemeral and automatically runs a live `publish --dry-run` preview instead of writing a draft file.
- With `--output`, the command writes the edited draft to disk and stops there unless you also add `--publish-dry-run`.
- After editing, the command prints a review summary that includes blocking validation issues and the suggested next action.
- `--apply-live` only proceeds when the edited draft keeps `dashboard.id` null, preserves the source dashboard UID, and has no blocking review issues.
- The edited payload must still contain `dashboard.uid`.

## Examples
```bash
# Purpose: Edit one live dashboard and preview the live publish without writing a draft file.
grafana-util dashboard edit-live --profile prod --dashboard-uid cpu-main
```

```bash
# Purpose: Edit one live dashboard and keep the result as a local draft.
grafana-util dashboard edit-live --url http://localhost:3000 --basic-user admin --basic-password admin --dashboard-uid cpu-main --output ./drafts/cpu-main.edited.json
```

```bash
# Purpose: Edit one live dashboard, save the draft, and immediately preview the publish.
grafana-util dashboard edit-live --profile prod --dashboard-uid cpu-main --output ./drafts/cpu-main.edited.json --publish-dry-run
```

```bash
# Purpose: Edit one live dashboard and explicitly write the result back to Grafana.
grafana-util dashboard edit-live --profile prod --dashboard-uid cpu-main --apply-live --yes --message 'Hotfix CPU dashboard'
```

## Related commands
- [dashboard fetch-live](./dashboard-fetch-live.md)
- [dashboard clone-live](./dashboard-clone-live.md)
- [dashboard publish](./dashboard-publish.md)
