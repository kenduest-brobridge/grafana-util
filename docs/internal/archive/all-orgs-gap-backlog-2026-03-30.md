# `--all-orgs` Gap Backlog - 2026-03-30

Purpose: capture the current high-confidence `--all-orgs` support gaps in the Rust CLI so follow-up work can be implemented in a consistent order.

This backlog separates three cases:
- flag already exists but is effectively unwired
- read/export/diff commands that should gain `--all-orgs`
- commands that should stay single-target or need a different routing model

## 1. Flag Exists But Is Effectively Unwired

### `grafana-util project-status live`
- Priority: `P0`
- Current state: `ProjectStatusLiveArgs` exposes both `--all-orgs` and `--org-id`.
- Evidence: [project_status_command.rs](/Users/kendlee/work/grafana-utils/rust/src/project_status_command.rs#L124)
- Gap: execution ignores both fields for the main live reads. The current client builder only creates one base client and never fans out per org or injects `X-Grafana-Org-Id`.
- Evidence: [project_status_command.rs](/Users/kendlee/work/grafana-utils/rust/src/project_status_command.rs#L819)
- Evidence: [project_status_command.rs](/Users/kendlee/work/grafana-utils/rust/src/project_status_command.rs#L711)
- Resulting bug: help text promises cross-org live status, but runtime still behaves as one-context current-org live read.

### `grafana-util overview live`
- Priority: `P0`
- Current state: `overview live` delegates to `project-status live`.
- Evidence: [overview.rs](/Users/kendlee/work/grafana-utils/rust/src/overview.rs#L188)
- Gap: inherits the same unwired `--all-orgs` behavior from `project-status live`.

## 2. High-Confidence Read/Export/Diff Gaps

These commands are inventory-style live reads or artifact generation paths. They match the same capability class where `dashboard` and `datasource` already support `--all-orgs`.

### Alert

#### `grafana-util alert export`
- Priority: `P1`
- Current state: `alert list-rules`, `list-contact-points`, `list-mute-timings`, and `list-templates` already support `--all-orgs`.
- Evidence: [alert_cli_defs.rs](/Users/kendlee/work/grafana-utils/rust/src/alert_cli_defs.rs#L13)
- Gap: `AlertExportArgs` has no `org_id` or `all_orgs`.
- Evidence: [alert_cli_defs.rs](/Users/kendlee/work/grafana-utils/rust/src/alert_cli_defs.rs#L132)
- Why it should support it: export is the natural multi-org inventory capture path for alerting, analogous to `dashboard export` and `datasource export`.

#### `grafana-util alert diff`
- Priority: `P1`
- Current state: `AlertDiffArgs` has no `org_id` or `all_orgs`.
- Evidence: [alert_cli_defs.rs](/Users/kendlee/work/grafana-utils/rust/src/alert_cli_defs.rs#L197)
- Why it should support it: once multi-org alert export exists, diff should be able to compare the same multi-org artifact root against live Grafana.

### Access Teams

#### `grafana-util access team list`
- Priority: `P1`
- Current state: list is scoped only through `CommonCliArgs`, which means one current org or one explicit `--org-id`.
- Evidence: [cli_defs.rs](/Users/kendlee/work/grafana-utils/rust/src/access/cli_defs.rs#L54)
- Gap: no `--all-orgs`, no org fan-out logic.
- Evidence: [team_list.rs](/Users/kendlee/work/grafana-utils/rust/src/access/team_list.rs#L92)

#### `grafana-util access team browse`
- Priority: `P2`
- Current state: browse is also single-scope only.
- Evidence: [cli_defs.rs](/Users/kendlee/work/grafana-utils/rust/src/access/cli_defs.rs#L92)
- Gap: no cross-org inventory mode even though user browse already has a cross-org path.

#### `grafana-util access team export`
- Priority: `P1`
- Current state: export walks only one scoped team surface.
- Evidence: [cli_defs.rs](/Users/kendlee/work/grafana-utils/rust/src/access/cli_defs.rs#L144)
- Evidence: [team_import_export.rs](/Users/kendlee/work/grafana-utils/rust/src/access/team_import_export.rs#L33)
- Why it should support it: export is the right place to build a combined per-org team bundle for later import/diff/review.

#### `grafana-util access team diff`
- Priority: `P1`
- Current state: diff compares one scoped live team set.
- Evidence: [cli_defs.rs](/Users/kendlee/work/grafana-utils/rust/src/access/cli_defs.rs#L227)
- Evidence: [team_list.rs](/Users/kendlee/work/grafana-utils/rust/src/access/team_list.rs#L151)
- Why it should support it: same reason as export; inventory diff should be able to operate on combined multi-org exports.

### Access Service Accounts

#### `grafana-util access service-account list`
- Priority: `P1`
- Current state: list only supports one scoped org context through `CommonCliArgs`.
- Evidence: [access_service_account_cli.rs](/Users/kendlee/work/grafana-utils/rust/src/access/access_service_account_cli.rs#L21)
- Gap: no `--all-orgs` and no fan-out path.
- Evidence: [service_account.rs](/Users/kendlee/work/grafana-utils/rust/src/access/service_account.rs#L25)

#### `grafana-util access service-account export`
- Priority: `P1`
- Current state: export is single-scope only.
- Evidence: [access_service_account_cli.rs](/Users/kendlee/work/grafana-utils/rust/src/access/access_service_account_cli.rs#L75)
- Why it should support it: export is the natural reviewable multi-org capture path for service accounts.

#### `grafana-util access service-account diff`
- Priority: `P1`
- Current state: diff is single-scope only.
- Evidence: [access_service_account_cli.rs](/Users/kendlee/work/grafana-utils/rust/src/access/access_service_account_cli.rs#L100)
- Why it should support it: same inventory parity reason as export.

## 3. Already Covered Or Covered By Another Model

These are not current gaps.

### Covered by real `--all-orgs`
- `dashboard list`
- `dashboard browse`
- `dashboard export`
- `dashboard inspect-live`
- `datasource list`
- `datasource browse`
- `datasource export`
- `alert list-rules`
- `alert list-contact-points`
- `alert list-mute-timings`
- `alert list-templates`

### Covered by a different but intentional cross-org model
- `access user list`
- `access user browse`
- `access user export`
- `access user diff`

Reason:
- user workflows already expose cross-org/global admin coverage through `--scope global`, with `--all-orgs` only used as a human-friendly alias for the list/browse read surfaces.
- Evidence: [access_user_cli.rs](/Users/kendlee/work/grafana-utils/rust/src/access/access_user_cli.rs#L12)
- Evidence: [access_user_cli.rs](/Users/kendlee/work/grafana-utils/rust/src/access/access_user_cli.rs#L245)

### Not really a multi-org fan-out problem
- `access org *`

Reason:
- org commands operate on Grafana's admin/global org registry itself, not on a per-org inventory that needs fan-out.

## 4. Lower-Confidence Candidates

These are worth discussing, but they are not as clear-cut as the inventory/export/diff gaps above.

### `grafana-util dashboard inspect-vars`
- Current state: has `--org-id` but no `--all-orgs`.
- Evidence: [cli_defs_inspect.rs](/Users/kendlee/work/grafana-utils/rust/src/dashboard/cli_defs_inspect.rs#L238)
- Why lower confidence: this is a single-dashboard targeted action, not a broad inventory/export surface.

### `grafana-util dashboard screenshot`
- Current state: has `--org-id` but no `--all-orgs`.
- Evidence: [cli_defs_inspect.rs](/Users/kendlee/work/grafana-utils/rust/src/dashboard/cli_defs_inspect.rs#L65)
- Why lower confidence: screenshot is an explicit single-target capture flow, so cross-org behavior may be better handled by `--org-id` plus target URL/UID resolution.

## 5. Suggested Implementation Order

1. Fix `project-status live` and `overview live`.
Reason: this is already a user-visible contract bug because the flag exists today.

2. Add `--all-orgs` to `alert export` and `alert diff`.
Reason: alerting already has multi-org list fan-out, so export/diff parity is the next most coherent expansion.

3. Add `--all-orgs` to `access team list/export/diff`, then `team browse`.
Reason: these are pure inventory surfaces and fit the same combined-root export model used by dashboard and datasource.

4. Add `--all-orgs` to `access service-account list/export/diff`.
Reason: same inventory pattern, but service-account semantics are slightly more operationally sensitive than teams, so it is reasonable to land after teams.

## 6. Implementation Notes

- Prefer the established `dashboard` / `datasource` pattern:
  - enumerate visible orgs with Basic auth
  - build one scoped client per org
  - aggregate rows with `orgId` and `orgName`
  - for export, write per-org subdirectories plus a combined root index/metadata document
- Keep live mutation commands single-target unless there is a reviewed routed-apply/import design.
- For commands that accept both `--org-id` and `--all-orgs`, keep them mutually exclusive.
- Add focused help/parser tests whenever a new `--all-orgs` flag is introduced.
