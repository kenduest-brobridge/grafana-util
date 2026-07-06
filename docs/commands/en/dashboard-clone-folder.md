# dashboard clone-folder

## Purpose
Duplicate dashboards from one live Grafana folder into another live folder.

## When to use
Use this when you need staging, review, or promotion copies of every dashboard in a folder without changing the source dashboards.

## Key flags
- `--source-folder-uid`: source Grafana folder UID.
- `--source-path`: source Grafana folder path, such as `Platform / Infra`.
- `--target-folder-uid`: destination Grafana folder UID.
- `--target-folder-title`: title to use when creating the target folder.
- `--create-target-folder`: create missing destination folders.
- `--recursive`: include child folders and their dashboards.
- `--uid-prefix` / `--uid-suffix`: deterministic cloned dashboard UID rule.
- `--replace-existing`: update existing target dashboard UIDs instead of blocking.
- `--dry-run`: preview without changing Grafana.
- `--yes`: acknowledge live writes.

## Examples
```bash
# Preview direct dashboard copies into an existing target folder.
grafana-util dashboard clone-folder --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --source-folder-uid infra --target-folder-uid staging-infra --dry-run --table
```

```bash
# Create the target folder and copy a folder subtree.
grafana-util dashboard clone-folder --url http://localhost:3000 --basic-user admin --basic-password admin --source-path 'Platform / Infra' --target-folder-uid staging-infra --target-folder-title 'Staging Infra' --create-target-folder --recursive --uid-suffix '-staging' --yes
```

## Related commands
- [dashboard clone](./dashboard-clone.md)
- [dashboard publish](./dashboard-publish.md)
- [dashboard delete](./dashboard-delete.md)
