# Alert Operator Handbook

This chapter is not a rewrite of `grafana-util alert --help`. It answers the operational question first: when you change Grafana Alerting, which connected objects move with the change, and should you inspect live state, author desired state, or produce a reviewed plan before touching Grafana?

Alerting work is rarely about one rule in isolation. A rule connects to a folder, rule group, labels, contact points, notification policy, templates, and mute timings. A mistake in one part may not look like a syntax failure; it may mean the right person never receives the page.

Read the `alert` subcommands as a workflow: inventory, backup, authoring, routing, review, and apply. The command reference still matters, but it should be where you confirm exact flags after you know which lane you are in.

## Who It Is For

- Operators maintaining Grafana Alerting rules, contact points, routes, templates, or mute timings.
- Reviewers who need to understand alert changes before they reach live Grafana.
- Teams moving alerting state through Git, review, or CI workflows.

## Alert Objects

`grafana-util alert` is not only about rules. It deals with several connected assets:

| Asset | Question to answer first | Common commands |
| --- | --- | --- |
| Rule | Which folder / rule group changes? Are conditions, labels, and annotations complete? | `list-rules`, `new-rule`, `add-rule`, `clone-rule` |
| Contact point | Where will the notification go? Does the receiver name match the route? | `list-contact-points`, `new-contact-point`, `add-contact-point` |
| Route / policy | Which labels match which receiver? Is a matcher too broad? | `set-route`, `preview-route` |
| Template | Does the notification content also need to move or be authored? | `list-templates`, `new-template` |
| Mute timing | When should alerts not notify? Does migration need to include the timing? | `list-mute-timings` |
| Desired state | Local files that represent the Grafana state you want. | `init`, `add-*`, `set-route`, `plan` |
| Raw bundle | A snapshot exported from live Grafana for backup, migration, and diff. | `export`, `import`, `diff` |
| Plan | The review artifact produced by comparing desired state with live Grafana. | `plan`, `apply` |

The boundaries matter. `raw/` export bundles are for moving current state. `desired/` files are for staging intent. A `plan` is the review evidence before apply. Mixing those paths is how a one-rule change accidentally becomes a route change or a delete candidate.

## Primary Goals

- Inspect live Grafana before changing it.
- Turn alert changes into local, reviewable files.
- Use plans to explain create / update / delete / noop before apply.
- Keep raw migration flows separate from desired-state authoring.

## What Success Looks Like

- You can say whether the task is inventory, backup, authoring, routing, or review/apply.
- You know whether a subcommand consumes live Grafana, a raw bundle, a desired directory, or a reviewed plan.
- Before `apply`, you have reviewed the plan and understand whether `--prune` creates delete candidates.

## Alert Workflow Map

| Job | Start here | Main input | Main output | Next step |
| --- | --- | --- | --- | --- |
| Inspect live alerting | `list-rules`, `list-contact-points`, `list-mute-timings`, `list-templates` | Grafana connection and permissions | Table, YAML, or JSON inventory | Decide whether to export or author |
| Back up or move current state | `export` | live Grafana | `raw/` bundle | `diff`, `import`, review |
| Compare a bundle with live | `diff` | `raw/` bundle + live Grafana | diff summary or JSON | Import, re-export, or stop |
| Recreate exported state | `import` | `raw/` bundle | live mutation or dry-run output | Dry-run before import |
| Author desired state | `init`, `new-*`, `add-*`, `clone-rule`, `set-route` | local desired dir | reviewable staged files | `preview-route`, `plan` |
| Preview routing | `preview-route` | desired dir + labels | matcher / receiver preview | adjust route or plan |
| Produce a change plan | `plan` | desired dir + live Grafana | plan JSON / text | human review |
| Apply reviewed changes | `apply` | reviewed plan file | live mutation result | inventory after apply |

## Inventory: Know What Is Live

If you do not know what Grafana currently contains, do not start with export, import, or apply. Start with inventory:

```bash
# Inspect rules first, including folder, rule group, and labels.
grafana-util alert list-rules --profile prod --output-format table
```

```bash
# Inspect contact points so receiver names can be matched against routes.
grafana-util alert list-contact-points --profile prod --output-format yaml
```

```bash
# Inspect templates and mute timings before migration.
grafana-util alert list-templates --profile prod --output-format table
grafana-util alert list-mute-timings --profile prod --output-format table
```

If inventory is missing resources you expected, check token permissions and org scope before you mutate anything.

`list-rules` is more than a way to print rule names. It is the first sanity check before and after alert changes. When reading its output, check rule name, folder, rule group, org scope, labels, and receiver-related fields. If you are migrating rules, also look for dashboard UID / panel ID links that must still make sense in the target environment. Use `--output-format table` for human review, and JSON / YAML when CI or a saved artifact will consume the result.

The next step depends on what inventory shows. If you only need to preserve current state, run `alert export`. If you want to derive a new rule from an existing one, use `clone-rule`. If receiver or route names do not line up, continue with `list-contact-points` and `preview-route`. If this is post-apply verification, rerun `plan` or at least preserve before/after inventory.

## Migration: Raw Bundles Are Snapshots

`alert export` writes live Grafana alerting resources into `raw/` JSON. This lane is for backup, migration, comparison, and disaster recovery. It is not the same as desired-state authoring.

```bash
# Export live alert state into a raw bundle.
grafana-util alert export --profile prod --output-dir ./alerts --overwrite
```

```bash
# Compare the raw bundle with the target Grafana estate.
grafana-util alert diff --profile prod --diff-dir ./alerts/raw --output-format json
```

```bash
# Dry-run an import before mutating live Grafana.
grafana-util alert import \
  --profile prod \
  --input-dir ./alerts/raw \
  --replace-existing \
  --dry-run --json
```

If rules are linked to dashboards or panels and the target environment uses different dashboard UIDs or panel IDs, prepare `--dashboard-uid-map` and `--panel-id-map` before diff / import / plan.

## Authoring: Desired State Is Intent

The authoring lane creates the state you want, locally, before live Grafana changes.

```bash
# Create the desired-state directory.
grafana-util alert init --desired-dir ./alerts/desired
```

Use `new-*` when you want a low-level scaffold:

```bash
# Create a rule scaffold that you will fill in later.
grafana-util alert new-rule --desired-dir ./alerts/desired --name cpu-main
```

Use `add-rule` for day-to-day authoring because it can write the rule, placement, receiver, severity, threshold, labels, and annotations together.

```bash
# Author a staged rule without touching live Grafana.
grafana-util alert add-rule \
  --desired-dir ./alerts/desired \
  --name cpu-high \
  --folder platform-alerts \
  --rule-group cpu \
  --receiver pagerduty-primary \
  --severity critical \
  --expr 'A' \
  --threshold 80 \
  --above \
  --for 5m \
  --label team=platform \
  --annotation summary='CPU high'
```

Use `clone-rule` when a new rule should start from an existing one. Use `new-contact-point` / `new-template` for low-level scaffolds, and `add-contact-point` for the higher-level contact-point authoring path.

## Routing: Preview Before Set

Routing failures are often semantic, not syntactic. Preview matcher input first, then write the managed route.

```bash
# Preview the matcher input before writing route state.
grafana-util alert preview-route \
  --desired-dir ./alerts/desired \
  --label team=platform \
  --severity critical
```

```bash
# Write the managed route after the receiver and matchers look right.
grafana-util alert set-route \
  --desired-dir ./alerts/desired \
  --receiver pagerduty-primary \
  --label team=platform \
  --severity critical
```

`set-route` replaces the tool-managed route. It is not an arbitrary merge into a hand-written policy tree. If you need to preserve a complex existing policy, export and diff the live policy first.

## Review / Apply: Plan Is The Gate

`plan` compares desired state with live Grafana. Treat it as a review artifact, not as disposable terminal output.

```bash
# Produce a plan without applying live mutations.
grafana-util alert plan \
  --profile prod \
  --desired-dir ./alerts/desired \
  --prune \
  --output-format json
```

When reviewing a plan, check at least:

- `create`: desired state exists but live Grafana is missing it.
- `update`: both sides exist but differ.
- `delete`: enabled by `--prune` when live has a resource absent from desired state.
- `noop`: both sides match.
- `blocked`: the tool decided the item should not be applied directly.

`--prune` is useful and dangerous. On a first rollout, run one plan without `--prune`, then run another with `--prune` and compare the delete candidates.

```bash
# Apply only a reviewed and saved plan.
grafana-util alert apply \
  --profile prod \
  --plan-file ./alert-plan-reviewed.json \
  --approve \
  --output-format json
```

Do not apply a just-generated plan that nobody has reviewed. In practice, save the plan as a PR artifact, change-ticket attachment, or traceable file.

## When To Use Command Reference

This chapter explains the workflow. Once you know the subcommand, open the command reference for exact flags:

- [alert](../../commands/en/alert.md)
- [alert list-rules](../../commands/en/alert-list-rules.md)
- [alert list-contact-points](../../commands/en/alert-list-contact-points.md)
- [alert list-mute-timings](../../commands/en/alert-list-mute-timings.md)
- [alert list-templates](../../commands/en/alert-list-templates.md)
- [alert export](../../commands/en/alert-export.md)
- [alert import](../../commands/en/alert-import.md)
- [alert diff](../../commands/en/alert-diff.md)
- [alert new-rule](../../commands/en/alert-new-rule.md)
- [alert add-rule](../../commands/en/alert-add-rule.md)
- [alert clone-rule](../../commands/en/alert-clone-rule.md)
- [alert new-contact-point](../../commands/en/alert-new-contact-point.md)
- [alert add-contact-point](../../commands/en/alert-add-contact-point.md)
- [alert new-template](../../commands/en/alert-new-template.md)
- [alert preview-route](../../commands/en/alert-preview-route.md)
- [alert set-route](../../commands/en/alert-set-route.md)
- [alert plan](../../commands/en/alert-plan.md)
- [alert apply](../../commands/en/alert-apply.md)
- [alert delete](../../commands/en/alert-delete.md)

## Failure Checks

- If inventory is missing resources, check profile, token permissions, org scope, and folder access.
- If export / import misses dashboard-linked rules, check dashboard UID and panel ID mappings.
- If plan has unexpected deletes, remove `--prune` and compare again.
- If route preview is empty, verify label key/value pairs and receiver names before apply.
- If apply differs from the reviewed plan, check whether live Grafana changed since the plan was produced.

---
[⬅️ Previous: Datasource Management](datasource.md) | [🏠 Home](index.md) | [➡️ Next: Access Management](access.md)
