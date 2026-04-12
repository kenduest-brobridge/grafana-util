# Dashboard Operator Handbook

This guide is for operators who need to inventory dashboards, export or import them safely, inspect dependencies, capture screenshots, or review drift before a live workspace.

## Who It Is For

- SREs and platform engineers responsible for dashboard inventory, promotion, or review.
- Operators who need screenshots, dependency checks, or export trees before a workspace.
- Teams that want dashboard work to fit into Git, review, and CI flows.

## Primary Goals

- Start from live visibility instead of guessing what exists.
- Understand the export and inspect lanes before replaying files.
- Review drift and dependency shape before import, publish, or delete.

## Before / After

- Before: dashboard work usually started from ad hoc UI clicks, fragile JSON handling, or unclear dependencies.
- After: inventory, inspect, diff, and replay happen in a clearer sequence with reviewable outputs.

## What success looks like

- You know whether the current task belongs in inventory, single-dashboard authoring, export/import replay, summary, dependencies, policy, or screenshot.
- You can explain which lane you are using before mutating anything.
- You can prove the dashboard is ready to replay or publish before you touch live state.

## Failure checks

- If the export tree is incomplete, fix the source path before replaying.
- If inspect output shows missing queries or variables, stop and resolve that before import.
- If you cannot explain what a screenshot or dependencies output is proving, you are probably in the wrong workflow lane.

## Dashboard Workflow Map

Dashboard subcommands are not a flat feature list. They are organized around different inputs and outcomes:

| Job | Start here | Main input | Main output | Next step |
| --- | --- | --- | --- | --- |
| Find a dashboard | `browse`, `list`, `get` | live Grafana | UID, folder, title, JSON | clone, export, or review |
| Build a local draft | `clone`, `get`, `patch`, `serve` | live dashboard or local JSON | reviewable draft file | `review`, `publish` |
| Publish a draft | `review`, `publish`, `edit-live` | local dashboard JSON | dry-run / publish result | list / get after apply |
| Back up and replay | `export`, `import`, `diff` | live Grafana or export tree | `raw/`, `prompt/`, `provisioning/` | diff / dry-run / import |
| Analyze dependencies | `summary`, `dependencies`, `variables`, `policy` | live or local dashboard | query, datasource, variable, policy summary | fix dashboard or datasource |
| Capture evidence | `screenshot` | live dashboard URL / UID | dashboard or panel screenshot | incident, PR, or docs evidence |
| Recover history | `history list`, `history export`, `history restore` | dashboard UID and version | history list, version artifact, or new revision | review before restore |

If you only have a dashboard UID, start with `get` or `clone`. If you have an export tree, start with `summary` / `diff`, not `import`. Use `prompt/` or screenshots for human handoff; use `raw/` for replay and audit.

## Draft authoring workflow

Use the authoring lane when the work starts from one dashboard file instead of a full export tree.

- Start from a live source with `dashboard get` or `dashboard clone` when Grafana already has the closest dashboard.
- Use `dashboard serve` when you want a lightweight local preview browser for one draft file, one draft directory, or one generator script output while you edit.
- Use `dashboard review` before mutation to confirm title, UID, tags, folder UID, and any blocking validation issues.
- Use `dashboard patch` when you need to rewrite one local draft in place or write a new patched file.
- Use `dashboard edit-live` when you want to fetch one live dashboard into an external editor, get a review summary with validation blockers, and keep a safe local-draft default instead of immediately mutating Grafana.
- Use `dashboard publish` when the draft is ready to go back through the same import pipeline used by the broader replay path.

Generator-driven teams do not need to stop at an intermediate temp file for every review or publish step.

```bash
# Review one generated dashboard from standard input.
jsonnet dashboards/cpu.jsonnet | grafana-util dashboard review --input - --output-format json
```

```bash
# Publish one generated dashboard from standard input.
jsonnet dashboards/cpu.jsonnet | grafana-util dashboard publish --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --input - --replace-existing
```

If you are editing one local draft repeatedly, use `dashboard publish --watch` with a file path instead of `--input -`. Watch mode reruns publish or dry-run after each stabilized save and keeps watching even if one iteration fails validation or the API call.

```bash
# Re-run dry-run publish after each save while editing one local draft.
grafana-util dashboard publish --url http://localhost:3000 --basic-user admin --basic-password admin --input ./drafts/cpu-main.json --dry-run --watch
```

```bash
# Keep one draft open in a lightweight local preview browser while you edit.
grafana-util dashboard serve --input ./drafts/cpu-main.json --port 18080 --open-browser
```

```bash
# Fetch one live dashboard into an external editor and keep the result as a local draft by default.
grafana-util dashboard edit-live --profile prod --dashboard-uid cpu-main --output ./drafts/cpu-main.edited.json
```

`dashboard patch --input -` requires `--output` because standard input cannot be overwritten in place.
If you target Grafana's built-in General folder, `dashboard publish` normalizes that back to the default root publish path instead of sending a literal `general` folder UID.
`dashboard serve` is intentionally a lightweight preview/document-inspection surface, not a full embedded Grafana renderer.

## History and recovery

When you are trying to recover a known-good dashboard version, use the history lane instead of rebuilding JSON by hand.

- [dashboard history](../../commands/en/dashboard-history.md)
- `dashboard history list` shows the recent revisions for one dashboard UID.
- `dashboard history restore` copies one historical version forward as a new latest Grafana revision entry.
- `dashboard history export` writes a reusable revision-history artifact for review or CI.

Restore is not a destructive overwrite. The selected historical version stays in history, and the restored copy becomes the new current revision.

> **Operator-First Design**: This tool treats dashboards as version-controlled assets. The goal is to move and govern dashboard state safely, with enough visibility to decide whether a file is ready to replay, patch, or promote.

## What This Area Is For

Use the dashboard area for estate-level governance:
- **Inventory**: Understand what exists across one or many organizations.
- **Structured Export**: Move dashboards between environments with dedicated "lanes".
- **Deep Inspection**: Analyze queries and datasource dependencies offline.
- **Screenshots and visual checks**: Produce reproducible dashboard or panel captures for docs, incident notes, and debugging.
- **Drift Review**: Compare staged files against live Grafana before applying.
- **Controlled Mutation**: Import or delete dashboards with mandatory dry-runs.

---

## 🔎 Inspection and screenshot workflows

If your goal is not export or import, but understanding what a dashboard currently looks like, which dependencies it carries, and how variables resolve, start here.

- `dashboard summary`: analyze one live dashboard's structure, queries, and dependencies.
- `dashboard summary`: analyze an exported dashboard tree offline.
- `dashboard variables`: verify variables, datasource choices, and URL-scoped inputs.
- `dashboard screenshot`: generate a reproducible dashboard or panel capture with a headless browser.
- `dashboard dependencies`: trace the dashboard's upstream relationships at a glance.

Common cases:

- attaching a screenshot to an incident or runbook
- checking whether one panel resolves the intended variables and datasources
- producing docs or review captures without manual screenshots
- reviewing query/dependency structure before making changes

---

## Workflow Boundaries (The Three Lanes)

Dashboard export intentionally produces three different lanes. This is more than directory organization: it separates automated replay, human UI import, and disk-based provisioning before those workflows can be confused. When you are preserving evidence after an incident, reviewing a change before promotion, or handing a dashboard to another environment, choosing the right lane up front is cheaper than repairing a JSON file used in the wrong place.

`raw/` is the record worth keeping. It stays closest to the API response and is the best source to commit to Git when you need traceability. Start here for backup, disaster recovery, diff, audit, dry-run import, or any workflow that will push dashboards back through the CLI.

`prompt/` is the handoff copy for a person using the UI. It serves Grafana's "Upload JSON" flow, not `grafana-util dashboard import`. Use this lane when another team, another org, or a browser-based operator needs to import the dashboard manually.

`provisioning/` is a deployment projection. It lets Grafana read dashboards from the filesystem through provisioning config, which fits container images, ConfigMaps, volume mounts, and GitOps-style deployment flows. It can deploy dashboards, but it should not become the only source for daily review and replay.

| Lane | Purpose | Best Use Case |
| :--- | :--- | :--- |
| `raw/` | **Canonical Replay** | The primary source for `grafana-util dashboard import`. Reversible and API-friendly. |
| `prompt/` | **UI Import** | Compatible with the Grafana UI "Upload JSON" feature. If you only have ordinary or raw dashboard JSON, convert it first with `grafana-util dashboard convert raw-to-prompt`. |
| `provisioning/` | **File Provisioning** | When Grafana should read dashboards from disk via its internal provisioning system. |

If you add `--include-history` to `dashboard export`, the export tree also gains a `history/` subdirectory for each org scope. In `--all-orgs` mode, that becomes one history tree per exported org root.

Use `dashboard history export` when you need a standalone JSON artifact for one dashboard UID. Use `export dashboard --include-history` when you want history artifacts bundled alongside the export tree.

---

## Prompt Placeholder Notes

- `$datasource` is a dashboard variable reference.
- `${DS_*}` is an external-import placeholder created from `__inputs`.
- A prompt dashboard can legitimately contain both forms at once.
- That usually means the dashboard keeps a Grafana datasource-variable workflow while also needing external-import inputs.
- Do not assume `$datasource` automatically means mixed datasource families. In many cases it only means the dashboard is still routing panel selection through one datasource variable.

---

## Staged vs Live: The Operator Logic

- **Staged Work**: Local export trees, validation, offline inspection, and dry-run reviews.
- **Live Work**: Grafana-backed inventory, live diffs, imports, and deletions.

**The Golden Rule**: Start with `list` or `browse` to discover, `export` to a staged tree, `summary` and `diff` to verify, and only then `import` or `delete` after a matching dry-run.

---

## Reading Live Inventory

Use `dashboard browse` or `dashboard list` to get a fast picture of the estate.

```bash
# Use dashboard browse to get a fast picture of the estate.
grafana-util dashboard browse \
  --url http://localhost:3000 \
  --basic-user admin \
  --basic-password admin \
  --table
```

**Validated Output Excerpt:**
```text
UID                      NAME                                      FOLDER  FOLDER_UID      FOLDER_PATH  ORG        ORG_ID
-----------------------  ----------------------------------------  ------  --------------  -----------  ---------  ------
rYdddlPWl                Node Exporter Full for Host               Demo    ffhrmit0usjk0b  Demo         Main Org.  1
spring-jmx-node-unified  Spring JMX + Node Unified Dashboard (VM)  Demo    ffhrmit0usjk0b  Demo         Main Org.  1
```

**How to Read It:**
- **UID**: Stable identity for automation and deletion.
- **FOLDER_PATH**: Where the dashboard is organized.
- **ORG/ORG_ID**: Confirms which organization owns the object.

---

## Inventory and Get: Confirm the Target First

Dashboard work often starts from a fuzzy clue: a screen, a folder, a team name, or a screenshot left in an incident. Do not start by exporting the whole estate or publishing a local file. First identify the dashboard UID, folder, org, and title.

Use `dashboard browse` when the folder view matters, `dashboard list` when you need sortable inventory, and `dashboard get` when you need the full JSON for one UID. Once you know the target, decide whether to `clone` it into a draft, `export` it into the replay lane, or use `summary` / `dependencies` for impact review.

If `list` or `browse` misses a dashboard you expected, check profile, org scope, folder permissions, and token visibility. An empty result does not prove the dashboard does not exist.

## Choose The Export Lane

`dashboard export` does not produce just one JSON shape. It serves CLI replay, human UI import, and Grafana provisioning. Choosing the wrong lane makes review and handoff harder.

- `raw/`: for `dashboard import`, `dashboard diff`, audit, and disaster recovery. This is the best source to keep in Git.
- `prompt/`: for Grafana UI Upload JSON handoff. This is a human handoff artifact, not the primary CLI replay source.
- `provisioning/`: for Grafana file provisioning. This is a deployment projection, not a replacement for raw review.

If you are handing one dashboard to a person for manual UI import, `prompt/` is easier to use than `raw/`. If CI needs diff, dry-run, or replay, use `raw/`. If the target is GitOps or a mounted provisioning directory, use `provisioning/`.

## Review / Publish: A Draft Must Be Explainable

`dashboard review` and `dashboard publish` form a gate: first decide whether the draft is safe to send, then decide whether to write live state. Review at least the title, UID, folder UID, tags, datasource references, variables, and blocking issues. If you cannot explain where the JSON came from, which UID it will replace, and which folder it will land in, do not publish it.

Use `dashboard publish --dry-run` before PRs or manual change windows. Use `--watch` while iterating on the same local draft. After publishing, verify live state with `dashboard get` or `dashboard list` instead of trusting only the publish output.

## Analysis and Evidence

`dashboard summary`, `dependencies`, `variables`, `policy`, and `screenshot` are not side features. They answer common review questions: which datasources does this dashboard query, do variables change query behavior, does policy allow production promotion, and is there visual evidence suitable for an incident or PR?

When the review question is structural, start with `summary`, `variables`, and `dependencies`. When the review question needs a human-readable artifact, use `screenshot`. A screenshot does not replace structural review, and structural review does not replace a visual artifact when one is required.

## When To Use Command Reference

This chapter helps you choose the workflow. Once you know which command you need, use the command reference for exact flags, output formats, and complete examples:

- [dashboard command overview](../../commands/en/dashboard.md)
- [dashboard browse](../../commands/en/dashboard-browse.md)
- [dashboard get](../../commands/en/dashboard-get.md)
- [dashboard clone](../../commands/en/dashboard-clone.md)
- [dashboard list](../../commands/en/dashboard-list.md)
- [dashboard export](../../commands/en/dashboard-export.md)
- [dashboard import](../../commands/en/dashboard-import.md)
- [dashboard convert raw-to-prompt](../../commands/en/dashboard-convert-raw-to-prompt.md)
- [dashboard patch](../../commands/en/dashboard-patch.md)
- [dashboard serve](../../commands/en/dashboard-serve.md)
- [dashboard edit-live](../../commands/en/dashboard-edit-live.md)
- [dashboard review](../../commands/en/dashboard-review.md)
- [dashboard publish](../../commands/en/dashboard-publish.md)
- [dashboard delete](../../commands/en/dashboard-delete.md)
- [dashboard diff](../../commands/en/dashboard-diff.md)
- [dashboard summary](../../commands/en/dashboard-summary.md)
- [dashboard variables](../../commands/en/dashboard-variables.md)
- [dashboard history](../../commands/en/dashboard-history.md)
- [dashboard policy](../../commands/en/dashboard-policy.md)
- [dashboard dependencies](../../commands/en/dashboard-dependencies.md)
- [dashboard screenshot](../../commands/en/dashboard-screenshot.md)
- [full command index](../../commands/en/index.md)

---

## Common Commands

| Command | Full Example with Arguments |
| :--- | :--- |
| **List** | `grafana-util dashboard list --all-orgs --with-sources --table` |
| **Export** | `grafana-util export dashboard --output-dir ./dashboards --overwrite --progress` |
| **Export + History** | `grafana-util export dashboard --output-dir ./dashboards --include-history --overwrite --progress` |
| **Raw to Prompt** | `grafana-util dashboard convert raw-to-prompt --input-dir ./dashboards/raw --output-dir ./dashboards/prompt --overwrite --progress` |
| **Import** | `grafana-util dashboard import --input-dir ./dashboards/raw --replace-existing --dry-run --table` |
| **Diff** | `grafana-util dashboard diff --input-dir ./dashboards/raw --input-format raw` |
| **Summary** | `grafana-util dashboard summary --input-dir ./dashboards/raw --input-format raw --output-format dependency` |
| **Delete** | `grafana-util dashboard delete --uid <UID> --url <URL> --basic-user admin --basic-password admin` |
| **Variables** | `grafana-util dashboard variables --uid <UID> --url <URL> --table` |
| **Patch** | `grafana-util dashboard patch --input <FILE> --name "New Title" --output <FILE>` |
| **Publish** | `grafana-util dashboard publish --input <FILE> --url <URL> --basic-user admin --basic-password admin` |
| **Clone** | `grafana-util dashboard clone --source-uid <UID> --output <FILE> --url <URL>` |

---

## Operator Examples

### 1. Export Progress
Use `--progress` for a clean log during large estate exports.
```bash
# Use --progress for a clean log during large estate exports.
grafana-util export dashboard --output-dir ./dashboards --overwrite --progress
```
**Output Excerpt:**
```text
Exporting dashboard 1/7: mixed-query-smoke
Exporting dashboard 2/7: smoke-prom-only
...
Exporting dashboard 7/7: two-prom-query-smoke
```

### 2. Dry-Run Import Preview
Always confirm the destination action before mutation.
```bash
# Always confirm the destination action before mutation.
grafana-util dashboard import --input-dir ./dashboards/raw --dry-run --table
```
**Output Excerpt:**
```text
UID                    DESTINATION  ACTION  FOLDER_PATH                    FILE
---------------------  -----------  ------  -----------------------------  --------------------------------------
mixed-query-smoke      exists       update  General                        ./dashboards/raw/Mixed_Query_Dashboard.json
subfolder-chain-smoke  missing      create  Platform / Team / Apps / Prod  ./dashboards/raw/Subfolder_Chain.json
```
- **ACTION=create**: New dashboard will be added.
- **ACTION=update**: Existing live dashboard will be replaced.
- **DESTINATION=missing**: No live dashboard currently owns that UID, so the import would create a new record.
- **DESTINATION=exists**: The UID already exists in Grafana, so the import would target that live dashboard.

### 3. Provisioning-Oriented Comparison
Compare your local provisioning files against live state.
```bash
# Compare your local provisioning files against live state.
grafana-util dashboard diff --input-dir ./dashboards/provisioning --input-format provisioning
```
**Output Excerpt:**
```text
--- live/cpu-main
+++ export/cpu-main
-  "title": "CPU Overview"
+  "title": "CPU Overview v2"
```

---
[⬅️ Previous: Architecture & Design](architecture.md) | [🏠 Home](index.md) | [➡️ Next: Datasource Management](datasource.md)
