# dashboard get

## Purpose
Get one live dashboard into an API-safe local JSON draft.

## When to use
Use this when you need a local copy of one live dashboard for review, patching, cloning, or later publish work.

## Key flags
- `--dashboard-uid`: live Grafana dashboard UID to fetch.
- `--output`: write the fetched draft to this path.
- Shared live flags: `--url`, `--token`, `--basic-user`, `--basic-password`, `--profile`.

## Examples
```bash
# Fetch one live dashboard into an API-safe local JSON draft.
grafana-util dashboard get --profile prod --dashboard-uid cpu-main --output ./cpu-main.json
```

```bash
# Fetch one live dashboard into an API-safe local JSON draft.
grafana-util dashboard get --profile prod --url http://localhost:3000 --basic-user admin --basic-password admin --dashboard-uid cpu-main --output ./cpu-main.json
```

## Related commands
- [dashboard clone](./dashboard-clone.md)
- [dashboard patch](./dashboard-patch.md)
- [dashboard review](./dashboard-review.md)
