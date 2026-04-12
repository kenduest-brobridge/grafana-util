# `grafana-util version`

## Purpose
Print the current `grafana-util` version.

## When to use
Use this when you need to confirm the installed binary or capture machine-readable version details for automation.

## Key flags
- `--json`: render version details as JSON for external tooling

## Examples
```bash
# Print the human-readable version.
grafana-util version
```

```bash
# Print machine-readable version details.
grafana-util version --json
```

## Related commands
- [config](./config.md)
- [status](./status.md)
