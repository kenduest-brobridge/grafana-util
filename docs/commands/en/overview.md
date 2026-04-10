# Removed root path: `grafana-util overview`

## Root

Purpose: migration note for the removed project-wide overview root.

When to use: when you are translating older docs or scripts to the current `observe` surface.

Description: the public overview surface now lives under `grafana-util observe`. The top-level `overview` root is no longer runnable. Use `observe overview` or `observe live` instead.

Canonical replacement:

- `grafana-util overview ...` -> `grafana-util observe overview ...`
- `grafana-util overview live ...` -> `grafana-util observe overview live ...`

Useful next page: [observe](./observe.md)
