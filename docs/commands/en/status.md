# Removed root path: `grafana-util status`

## Root

Purpose: migration note for the removed staged/live status root.

When to use: when you are translating older docs or scripts to the current `observe` surface.

Description: the public staged/live status surface now lives under `grafana-util observe`. The top-level `status` root is no longer runnable. Use `observe staged`, `observe live`, `observe overview`, or `observe snapshot` instead.

Canonical replacement:

- `grafana-util status staged ...` -> `grafana-util observe staged ...`
- `grafana-util status live ...` -> `grafana-util observe live ...`

Useful next page: [observe](./observe.md)
