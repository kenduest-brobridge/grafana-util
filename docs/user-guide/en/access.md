# Access Management (Identity & Access)

Use this chapter when identity and access are the task: org boundaries, users, teams, service accounts, and the tokens that let automation act safely.
Inventory reads can come from live Grafana or from a local export bundle; the workflow is the same either way.

Access work is easy to underestimate because it often looks like adding one user or deleting one token. In practice, each user, team, org, and service account defines who can see data, who can change assets, and which automation can touch Grafana while nobody is watching. This chapter starts with scope so the command is not chosen in isolation.

Read it by asking whether the task affects a person, a team, an org boundary, or a machine credential. Then decide whether you need live inventory, a local snapshot, a diff, or a real mutation. Do not start from "I have a token"; start from "who or what will this permission change affect?"

## Who It Is For

- Administrators managing org, user, team, or service-account lifecycle work.
- Operators exporting or replaying identity state across environments.
- Teams rotating service-account tokens or auditing access drift.

## Primary Goals

- Clarify which access surface matches the task before running mutations.
- Keep org, user, team, and service-account inventory reviewable from live or local sources.
- Treat token rotation and access replay as controlled workflows instead of one-off edits.

## Before / After

- Before: identity and access work was spread across several command groups with different naming styles.
- After: org, user, team, and service-account workflows share one guide, one glossary, and clearer entry points.

## What success looks like

- You can tell whether the task is org management, user management, team management, or service-account handling.
- You can name the exact command family before you start mutating anything.
- You know when you need a review step before applying a workspace.

## Failure checks

- If an identity workspace would affect more orgs than you intended, stop and verify the scope first.
- If you are unsure whether a token or service account can perform the task, check the command page before mutation.
- If the wording still feels split across team/org/user surfaces, return to the glossary and the command reference.

## Access Workflow Map

Classify access subcommands by who or what they affect, not by which token you happen to have:

| Job | Start here | Main input | Main output | Next step |
| --- | --- | --- | --- | --- |
| Manage orgs | `access org list/export/import/diff` | Live Grafana or org bundle | Org inventory / dry-run / diff | Review before import |
| Manage users | `access user list/add/modify/delete/export/import/diff` | Login, org role, bundle | User inventory or mutation result | Verify with list / diff |
| Manage teams | `access team list/export/import/diff` | Org scope, team bundle | Team and member inventory | Import after dry-run |
| Manage service accounts | `access service-account list/add/modify/delete/export/import/diff` | Org scope, service-account bundle | Automation identity inventory | Manage tokens or diff |
| Manage tokens | `access service-account token add/delete/list` | Service account id/name | One-time token secret or delete result | Store the secret immediately / rotate the old token |
| Compare drift | `diff` subcommands | Local bundle + live Grafana | Change differences | Fix the bundle or import |

The key decision is scope. A global user, an org role, team membership, service-account permissions, and token lifecycle are different layers. Do not infer that a token able to call one API should be used for every access workflow.

## org Management

Use `access org` when you need Basic-auth-backed inventory, export, or replay for orgs, especially when you need to verify which orgs exist before a cross-org workspace. The same `list` command can also read a saved bundle with `--input-dir`.

### 1. List, Export, and Replay orgs
```bash
# Start by listing the orgs visible in live Grafana.
grafana-util access org list --table
```

```bash
# Review a saved org bundle without touching live Grafana.
grafana-util access org list --input-dir ./access-orgs --table
```

```bash
# Export org inventory into a replayable local bundle.
grafana-util export access org --output-dir ./access-orgs
```

```bash
# Dry-run the import before creating or updating orgs.
grafana-util access org import --input-dir ./access-orgs --dry-run
```
**Expected Output:**
```text
ID   NAME        IS_MAIN   QUOTA
1    Main Org    true      -
5    SRE Team    false     10

Exported organization inventory -> access-orgs/orgs.json
Exported organization metadata   -> access-orgs/export-metadata.json

PREFLIGHT IMPORT:
  - would create 0 org(s)
  - would update 1 org(s)
```
Use the list output to confirm the main org, then export/import when you need a repeatable org snapshot.

---

## User and team management

Use `access user` and `access team` for membership changes, snapshots, and drift checks when you need to reconcile who can see or edit what. Their `list` and `browse` commands both read local bundles through `--input-dir`.

Separate the user and team responsibilities first. User workflows decide whether one login exists, whether it is disabled, and which org role it has. Team workflows manage collaboration groups and membership. If the question is "can this person sign in or hold this role?", start with `access user`. If the question is "can this group see or manage a set of assets?", start with `access team`.

Use `list` for live or bundled inventory, `export` for reviewable snapshots, `diff` to detect live drift from a bundle, and `import` only after dry-run. When using `add`, `modify`, or `delete` for a user, confirm whether the scope is global user state, org role, or Grafana admin state before mutating.

### 1. Add, Modify, and Diff Users
```bash
# Add a new user with global admin role
grafana-util access user add --login dev-user --role Admin --prompt-password

# Update an existing user's organization role
grafana-util access user modify --login dev-user --org-id 5 --role Editor

# Compare a saved user snapshot against live Grafana
grafana-util access user diff --diff-dir ./access-users --scope global
```

If you want the same user inventory from a saved bundle, use:

```bash
# Inspect the same user inventory from a local bundle
grafana-util access user list --input-dir ./access-users
```
**Expected Output:**
```text
Created user dev-user -> id=12 orgRole=Editor grafanaAdmin=true

No user differences across 12 user(s).
```
Use `--prompt-password` when you do not want a password in shell history. `--scope global` requires Basic auth.

### 2. Discover and Sync Teams
```bash
# List the teams currently present in one org.
grafana-util access team list --org-id 1 --table
```

```bash
# Inspect the same team inventory from a local bundle.
grafana-util access team list --input-dir ./access-teams --table
```

```bash
# Export teams and preserve membership state.
grafana-util export access team --output-dir ./access-teams --with-members
```

```bash
# Preview team replay actions before replacing existing state.
grafana-util access team import --input-dir ./access-teams --replace-existing --dry-run --table
```
**Expected Output:**
```text
ID   NAME           MEMBERS   EMAIL
10   Platform SRE   5         sre@company.com

Exported team inventory -> access-teams/teams.json
Exported team metadata   -> access-teams/export-metadata.json

LOGIN       ROLE    ACTION   STATUS
dev-admin   Admin   update   existing
ops-user    Viewer  create   missing
```
Use `--with-members` when the export must preserve membership state, and use `--dry-run --table` before a destructive import.

---

## service account management

Service accounts are the foundation of repeatable automation, CI jobs, and scoped integrations.
Their inventory can be reviewed from live Grafana or from a local export bundle before you touch tokens.

A service account is a machine identity, not a safer shortcut for a human user. It should map to one automation purpose, such as CI deployment, nightly audit, or an incident bot. `access service-account` manages the identity; `access service-account token` manages API tokens for that identity. Review those layers separately.

Token secrets are usually shown only once. After creating a token, store the secret in the right secret store before rotating the old token. Do not treat terminal logs as secret storage, and do not assume a token is still usable just because the service account appears in inventory.

### 1. List and Export Service Accounts
```bash
# Use JSON when service-account inventory will feed a script.
grafana-util access service-account list --json
```

```bash
# Review service-account inventory from a saved bundle.
grafana-util access service-account list --input-dir ./access-sa --output-format text
```

```bash
# Export service-account inventory for review or replay.
grafana-util export access service-account --output-dir ./access-sa
```
**Expected Output:**
```text
[
  {
    "id": "15",
    "name": "deploy-bot",
    "role": "Editor",
    "disabled": false,
    "tokens": "1",
    "orgId": "1"
  }
]

Listed 1 service account(s) at http://127.0.0.1:3000

Exported service account inventory -> access-sa/service-accounts.json
Exported service account tokens    -> access-sa/tokens.json
```
`export access service-account` writes both the inventory and the token bundle. Treat `tokens.json` as sensitive.

### 2. Create and Delete Tokens
```bash
# Add a new token to a service account by name
grafana-util access service-account token add --name deploy-bot --token-name nightly --seconds-to-live 3600

# Add a new token by numeric id and capture the one-time secret
grafana-util access service-account token add --service-account-id 15 --token-name ci-deployment-token --json

# Delete an old token after verification
grafana-util access service-account token delete --service-account-id 15 --token-name nightly --yes --json
```
**Expected Output:**
```text
Created service-account token nightly -> serviceAccountId=15

{
  "serviceAccountId": "15",
  "name": "ci-deployment-token",
  "secondsToLive": "3600",
  "key": "eyJ..."
}

{
  "serviceAccountId": "15",
  "tokenId": "42",
  "name": "nightly",
  "message": "Service-account token deleted."
}
```
Use `--json` when you need the one-time `key` field. Plain text is better for logs, not for credential capture.

---

## Drift Detection (Diff)

Compare your local identity snapshots against the live Grafana server.

```bash
# Compare your local identity snapshots against the live Grafana server.
grafana-util access user diff --input-dir ./access-users
```

```bash
# Compare your local identity snapshots against the live Grafana server.
grafana-util access team diff --diff-dir ./access-teams
```

```bash
# Compare your local identity snapshots against the live Grafana server.
grafana-util access service-account diff --diff-dir ./access-sa
```
**Expected Output:**
```text
--- Live Users
+++ Snapshot Users
-  "login": "old-user"
+  "login": "new-user"

No team differences across 4 team(s).
No service account differences across 2 service account(s).
```
Use diff output to decide whether a snapshot is safe to import or whether live Grafana has already drifted.

## When To Use Command Reference

This chapter helps you choose the access scope. Once you know whether the task affects orgs, users, teams, service accounts, or tokens, use the command reference for exact flags, output formats, and complete examples:

- [access command overview](../../commands/en/access.md)
- [access user](../../commands/en/access-user.md)
- [access org](../../commands/en/access-org.md)
- [access team](../../commands/en/access-team.md)
- [access service-account](../../commands/en/access-service-account.md)
- [access service-account token](../../commands/en/access-service-account-token.md)
- [full command index](../../commands/en/index.md)

---
[⬅️ Previous: Alerting Governance](alert.md) | [🏠 Home](index.md) | [➡️ Next: Workspace Review & Status](status-workspace.md)
