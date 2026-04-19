# `grafana-util access user`

## Purpose

List or browse live and local Grafana users, create, modify, export, import, diff, or delete Grafana users.

## When to use

- Inspect users in the current org or in global admin scope.
- Inspect users from a live Grafana server or from a local export bundle.
- Create or update users with login, email, role, and admin settings.
- Export and import user inventory bundles.
- Remove users from the org membership or from the global registry.

## Before / After

- **Before**: user lifecycle work is often split across org settings, admin screens, and one-off export or import scripts.
- **After**: one namespace covers inventory, create/update, export/import, and targeted removal with the same auth model.

## What success looks like

- created or modified users have the expected login, email, and role
- inventory and bundles can be diffed before deletion or migration
- membership scope stays explicit, so you do not accidentally manage the wrong org or global registry
- machine-readable `list` / export output includes structured `origin` and `lastActive` user metadata for automation and audits

## Failure checks

- if list, add, or delete looks empty or wrong, confirm the selected profile or auth token has the right org or admin scope
- if create or modify fails, recheck login or email uniqueness and whether the selected scope is org or global
- if import reports a blocked update, check whether the target user is external, externally synced, or managed by a provisioned identity source
- if an import does not behave as expected, verify the bundle source and the target scope before retrying

## Import notes

- User import treats `id`, `userId`, `uid`, `authLabels`, and external/provisioned flags as target evidence, not portable identity.
- `--dry-run --output-format table` or `--dry-run --json` reports blocked profile, org role, or Grafana admin updates before live writes when Grafana marks the target user as externally managed.
- `modify` uses the same Grafana-source guardrails before any write: external or provisioned users block profile/password changes, externally synced users block org-role changes, and externally synced Grafana-admin state blocks admin changes.

## Key flags

- `list`: `--input-dir`, `--scope`, `--all-orgs`, `--query`, `--login`, `--email`, `--org-role`, `--grafana-admin`, `--with-teams`, `--output-columns`, `--list-columns`, `--page`, `--per-page`, `--table`, `--csv`, `--json`, `--yaml`, `--output-format`
- `browse`: `--input-dir`, `--scope`, `--all-orgs`, `--current-org`, `--query`, `--login`, `--email`, `--org-role`, `--grafana-admin`, `--page`, `--per-page`
- `add`: `--login`, `--email`, `--name`, `--password` or `--password-file` or `--prompt-user-password`, `--org-role`, `--grafana-admin`, `--json`
- `modify`: `--user-id`, `--login`, `--email`, `--set-login`, `--set-email`, `--set-name`, `--set-password` or `--set-password-file` or `--prompt-set-password`, `--set-org-role`, `--set-grafana-admin`, `--json`
- `export` and `diff`: `--output-dir` or `--diff-dir`, `--run`, `--run-id`, `--overwrite`, `--dry-run`, `--scope`, `--with-teams`
- `import`: `--input-dir`, `--scope`, `--replace-existing`, `--dry-run`, `--table`, `--json`, `--output-format`, `--yes`
- `delete`: `--user-id`, `--login`, `--email`, optional `--scope`, `--prompt`, `--yes`, `--json`

## Examples

When `--output-dir` is omitted, `access user export` writes to the profile artifact workspace under `access/users/`. Use `--run timestamp` for a fresh timestamped run or `--run-id <name>` for a deterministic run. Later local reads can use `access user list --local --run latest`.

```bash
# Inspect users in one org before changing membership or roles.
grafana-util access user list --url http://localhost:3000 --basic-user admin --basic-password admin --scope org --output-format text
```

```bash
# Review a saved user bundle without touching Grafana.
grafana-util access user list --input-dir ./access-users --output-format table
```

```bash
# Browse one saved user bundle interactively without touching Grafana.
grafana-util access user browse --input-dir ./access-users --login alice
```

```bash
# Export users into the profile artifact workspace with a timestamped run id.
grafana-util access user export --profile prod --run timestamp --scope org --overwrite
```

```bash
# Create one user with explicit auth and org scope.
grafana-util access user add --url http://localhost:3000 --basic-user admin --basic-password admin --login alice --email alice@example.com --name Alice --password secret
```

```bash
# Delete one account after checking the current org users.
grafana-util access user delete --url http://localhost:3000 --basic-user admin --basic-password admin --login temp-user --scope global --yes --json
```

```bash
# Prompt for one user, confirm the target, and then delete it.
grafana-util access user delete --url http://localhost:3000 --basic-user admin --basic-password admin --scope global --prompt
```

```bash
# Prompt for the delete scope first, then select one user and confirm the delete.
grafana-util access user delete --url http://localhost:3000 --basic-user admin --basic-password admin --prompt
```

## Related commands

- [access](./access.md)
- [access org](./access-org.md)
- [access team](./access-team.md)
- [access service-account](./access-service-account.md)
