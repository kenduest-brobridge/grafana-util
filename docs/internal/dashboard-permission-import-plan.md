# Dashboard Permission Import Plan

Reviewed: 2026-03-21

Status summary:

- Done today:
  - Python and Rust `dashboard export` already write `raw/permissions.json`.
  - Raw export metadata already records `permissionsFile`.
  - Dashboard import, inspect, and sync discovery already treat `permissions.json`
    as metadata instead of a dashboard document.
- Not implemented today:
  - No Python or Rust import path restores dashboard or folder ACLs.
  - No CLI flag such as `--restore-permissions` exists yet.
- Recommendation:
  - Keep this as a future design note, not an active implementation plan, until
    dashboard import/inspection priorities move back to permission replay.

## Goal

Add an opt-in dashboard permission restore workflow that can replay exported
dashboard and folder ACLs back into Grafana without trusting stale numeric IDs
from the source environment.

The intended target remains shared behavior across Python and Rust, even if a
future implementation lands in Rust first.

## Current Baseline

- `dashboard export` writes `raw/permissions.json` in both Python and Rust.
- Permission export rows already capture enough information for backup and
  manual review:
  - resource kind and UID
  - org and orgId
  - subject type
  - normalized `subjectKey`, `subjectId`, and `subjectName`
  - normalized permission level and `permissionName`
  - `inherited`
- `dashboard import` currently restores dashboard content, folder placement, and
  raw inventory only.
- `sync bundle` and dashboard import discovery now explicitly ignore
  `permissions.json` as a dashboard content document.

## Gap Between Current Export And Future Restore

The export side is already useful as backup metadata, but it is still not a
complete restore contract.

Current export rows do not consistently preserve richer user identity fields
such as:

- `subjectLogin`
- `subjectEmail`

Those fields are likely needed if permission replay is ever added across
Grafana environments.

## Core Design Rule

Permission restore should resolve destination subjects from stable identity
fields, not by replaying exported numeric IDs directly.

Do not trust source-environment IDs such as:

- `userId`
- `teamId`
- `serviceAccountId`

Those values are useful as export snapshots, but they are not safe import keys
across environments.

## Subject Resolution Strategy

Import should resolve permission subjects in this order:

- `user`
  - prefer `login`
  - optionally fall back to `email`
- `team`
  - prefer team `name`
- `service-account`
  - prefer service-account `name`
- `role`
  - use role name directly

## Recommended Future Export Contract Extension

If restore work resumes, permission rows should preserve richer user identity
fields when Grafana returns them.

Recommended fields per permission row:

- `subjectType`
- `subjectKey`
- `subjectId`
- `subjectName`
- `subjectLogin`
- `subjectEmail`
- `permission`
- `permissionName`
- `inherited`

Expected meaning by subject type:

- `user`
  - `subjectLogin` is the primary restore key
  - `subjectEmail` is a fallback when available
- `team`
  - `subjectName` is the restore key
- `service-account`
  - `subjectName` is the restore key
- `role`
  - `subjectName` or role string is the restore key

## Import Scope

Permission restore should remain opt-in.

Recommended CLI shape:

- `--restore-permissions`
- `--permissions-mode strict|skip-missing|report-only`
- `--permissions-subject-source live|access-export|auto`

Recommended first-version default behavior:

- no permission replay unless `--restore-permissions` is set
- default mode under restore is `strict`

## Import Flow

Suggested execution order:

1. run the normal dashboard import workflow
2. load `raw/permissions.json`
3. resolve each exported resource to the destination dashboard or folder
4. resolve each permission subject against the destination Grafana
5. build the destination ACL payload
6. apply the ACL through the Grafana permissions APIs

This keeps content restore and ACL restore explicit, and avoids partially
restoring permissions before the destination dashboard or folder exists.

## Subject Sources

Three source modes are worth supporting conceptually:

- `live`
  - resolve users, teams, and service accounts directly from destination Grafana
- `access-export`
  - use exported access-management data as the main subject inventory
- `auto`
  - try access export first, then fall back to live lookup

Recommended first version:

- implement `live` first
- add `auto` later when access export/import integration is ready

## Relationship To Access Export/Import

Dashboard permission restore depends on destination subjects already existing.

That makes `access export/import` the natural companion workflow for:

- users
- teams
- service accounts
- org-level identity inventory

Recommended ownership split:

- `dashboard export/import`
  - owns dashboard and folder ACL definitions
- `access export/import`
  - owns subject existence and subject lifecycle

Do not make dashboard import auto-create missing users or teams in the first
version.

## Dry-Run Requirements

Permission restore must be visible in dry-run output.

Dry-run should report:

- which dashboard or folder resources would receive ACL updates
- which subjects matched successfully
- which subjects are missing
- which permissions would be skipped or would fail

Suggested summary fields:

- `resourceCount`
- `permissionCount`
- `matchedSubjectCount`
- `missingSubjectCount`
- `appliedPermissionCount`
- `skippedPermissionCount`

## Failure Policy

Recommended semantics:

- `strict`
  - any unresolved subject fails the whole permission-restore step
- `skip-missing`
  - unresolved subjects are skipped, remaining permissions are applied
- `report-only`
  - evaluate and report only, never mutate Grafana

Recommended first version:

- implement `strict`
- implement `skip-missing`
- treat `report-only` as optional follow-up if dry-run JSON or table output is
  already strong enough

## Resource Matching Rules

Permission replay needs deterministic destination resource resolution.

Suggested rules:

- dashboard permissions map by imported dashboard UID
- folder permissions map by imported or exported folder UID after folder-ensure
  logic resolves destination folders

If the destination resource cannot be proven, fail closed.

## Recommended First Implementation Slice

If this work becomes active again, keep the first slice small:

- Rust `dashboard import`
- `--restore-permissions`
- `--permissions-mode strict|skip-missing`
- destination subject resolution from live Grafana only
- support `user`, `team`, and `role`
- keep `service-account` support as a follow-up if live lookup cost is higher
- full dry-run reporting for permission replay

## Follow-Up Candidates

- extend permission export rows with `subjectLogin` and `subjectEmail`
- add service-account restore
- add `--permissions-subject-source access-export|auto`
- integrate with routed multi-org dashboard import
- add Python runtime parity once the Rust contract stabilizes
- document an end-to-end restore sequence:
  - access import first
  - dashboard import second
  - permission restore last

## Guardrails

- never apply exported numeric subject IDs directly across environments
- never auto-create missing users or teams during dashboard import in the first version
- keep permission replay opt-in
- keep dry-run output explicit enough that operators can review subject matches before mutation
