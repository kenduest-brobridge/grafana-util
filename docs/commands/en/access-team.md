# `grafana-util access team`

## Purpose

List or browse live and local Grafana teams, create, modify, export, import, diff, or delete Grafana teams.

## When to use

- Inspect team inventory and team memberships.
- Inspect teams from a live Grafana server or from a local export bundle.
- Create or update team membership and admin assignments.
- Export or import team bundles.
- Remove a team by id or exact name.

## Before / After

- **Before**: team membership changes often happen through UI side menus or scattered scripts.
- **After**: one namespace handles inventory, membership updates, export/import, and deletion with the same auth model.

## What success looks like

- team membership changes stay tied to an exact team id or name
- admin assignments are visible before you add or remove members
- exported bundles can be reused in another environment without re-creating the team by hand

## Failure checks

- if list, add, modify, or delete fails, confirm the team exists in the selected org and the auth scope is correct
- if membership looks incomplete, recheck the exact member names and whether `--with-members` was set
- if import reports a blocked membership update, check whether the target team is provisioned or whether each member identity resolves to a live org user with an email address
- if an import behaves unexpectedly, verify the workspace package and the target environment before retrying

## Import notes

- Team import accepts exported member identities, but Grafana's bulk membership API applies members by email. The importer resolves login, email, or user id against live org users before apply.
- `--dry-run --output-format table` or `--dry-run --json` reports blocked provisioned-team updates before live writes.

## Key flags

- `list`: `--input-dir`, `--query`, `--name`, `--with-members`, `--output-columns`, `--list-columns`, `--page`, `--per-page`, `--table`, `--csv`, `--json`, `--yaml`, `--output-format`
- `browse`: `--input-dir`, `--query`, `--name`, `--with-members`, `--page`, `--per-page`
- `add`: `--name`, `--email`, `--member`, `--admin`, `--json`
- `modify`: `--team-id`, `--name`, `--add-member`, `--remove-member`, `--add-admin`, `--remove-admin`, `--json`
- `export` and `diff`: `--output-dir` or `--diff-dir`, `--run`, `--run-id`, `--overwrite`, `--dry-run`, `--with-members`
- `import`: `--input-dir`, `--replace-existing`, `--dry-run`, `--table`, `--json`, `--output-format`, `--yes`
- `delete`: `--team-id`, `--name`, `--prompt`, `--yes`, `--json`

## Examples

When `--output-dir` is omitted, `access team export` writes to the profile artifact workspace under `access/teams/`. Use `--run timestamp` for a fresh timestamped run or `--run-id <name>` for a deterministic run. Later local reads can use `access team list --local --run latest`.

```bash
# Inspect team membership before adding or removing people.
grafana-util access team list --url http://localhost:3000 --basic-user admin --basic-password admin --output-format text
```

```bash
# Review a saved team bundle before replaying it.
grafana-util access team list --input-dir ./access-teams --output-format table
```

```bash
# Browse one saved team bundle interactively without touching Grafana.
grafana-util access team browse --input-dir ./access-teams --name platform-team
```

```bash
# Export teams into the profile artifact workspace with a timestamped run id.
grafana-util access team export --profile prod --run timestamp --with-members --overwrite
```

```bash
# Create a team with explicit member and admin assignments.
grafana-util access team add --url http://localhost:3000 --basic-user admin --basic-password admin --name platform-team --email platform@example.com --member alice --admin alice --json
```

```bash
# Import a team bundle before switching environments.
grafana-util access team import --url http://localhost:3000 --basic-user admin --basic-password admin --input-dir ./access-teams --replace-existing --yes
```

```bash
# Prompt for one team, confirm it, and then delete it.
grafana-util access team delete --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --prompt
```

## Related commands

- [access](./access.md)
- [access user](./access-user.md)
- [access org](./access-org.md)
- [access service-account](./access-service-account.md)
