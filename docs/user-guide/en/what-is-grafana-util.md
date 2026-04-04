# What grafana-util is for

`grafana-util` is not just a thin wrapper around the Grafana HTTP API, and it is not only a backup/export tool. Its job is to connect the day-to-day operator workflows around inventory, inspection, review, migration, and replay into one consistent CLI.

If you regularly hit problems like these, this is the tool solving them:

- you need a fast picture of what exists across a Grafana estate, not just one UI screen at a time
- you want to move dashboards, alerts, or data sources without relying only on manual UI clicks
- you want to know what a change will do before you apply it
- you want to keep exports in Git, CI/CD, or review workflows without dumping secrets into plain files
- you want repeatable operator workflows instead of rebuilding command lines from scratch every time

---

## What it is

`grafana-util` is best understood as a Grafana operations workflow tool, not a single-purpose CLI.

It breaks the work into a few clear surfaces:

- **Inventory and observation**: start with `status` and `overview`
- **Asset operations**: use `dashboard`, `datasource`, `alert`, and `access`
- **Change review**: use `change` to summarize, preflight, plan, and review before apply
- **Connection and credentials**: use `profile` to keep URLs, auth defaults, and secret sources repeatable

The goal is not to memorize every command first. The goal is to know what kind of work you are doing.

---

## Feature overview table

| Area | Main command | What you use it for |
| :--- | :--- | :--- |
| Readiness and health checks | `status` | Check whether live or staged state is healthy enough to move forward |
| Estate-wide overview | `overview` | Get a fast picture of the Grafana estate and decide where to drill in next |
| Dashboard operations | `dashboard` | Export, import, diff, inspect, screenshot, and topology analysis |
| Data source operations | `datasource` | Inventory, export, import, diff, mutation, and recovery for data sources |
| Alert governance | `alert` | Alert rules, notification routing, contact points, and plan/apply workflows |
| Identity and access | `access` | Manage orgs, users, teams, service accounts, and tokens |
| Change review | `change` | Summarize, preflight, plan, and review changes before apply |
| Connection and credentials | `profile` | Keep URLs, auth defaults, and secret sources repeatable |

If all you need is “where do I start?”, use this table first, then move into the matching handbook chapter.

---

## Where it helps most

### 1. Daily operations and checks

You want quick answers to questions like:

- what dashboards, alerts, and data sources exist right now?
- does live state look healthy?
- where does the estate already look like it is drifting?

This usually starts with `status live` or `overview live`.

### 2. Export, migration, and replay

You want to move dashboards or data sources between environments, or keep a replayable export tree. That usually means you need more than just one export command:

- export into the right lane
- inspect, diff, or dry-run first
- then decide whether to import or replay

### 3. Review before mutation

You do not want to apply changes blind. You want answers to:

- what will this actually change?
- is the staged input complete?
- are routes, secrets, dependencies, and permissions in a sane state?

That is where `change summary`, `change preflight`, and `alert plan` matter.

### 4. Automation and CI/CD

You want Grafana operations to fit into scripts, pipelines, or scheduled jobs instead of depending on manual UI work.

That usually means:

- using `--profile` or env-backed auth cleanly
- choosing stable output formats
- keeping gates and review steps before mutation

---

## What it is not trying to replace

There are cases where `grafana-util` is not the first thing you need:

- you are making one tiny change in the Grafana UI
- you only need to inspect one value on one screen
- you do not need review, export, migration, replay, or automation

If the work does not need a repeatable or reviewable trail, the Grafana UI may still be the faster path.

---

## A good way to start

If this is your first time, do not start by reading every command page. A better sequence is:

1. understand the supported connection and auth patterns
2. run one safe read-only command
3. choose the path that fits you: new user, SRE/operator, or automation
4. open command docs only when you need exact syntax and flags

If you are starting now, read these next:

- [Getting Started](getting-started.md)
- [New User Path](role-new-user.md)
- [Command Docs](../../commands/en/index.md)

---
[⬅️ Back to handbook home](index.md) | [➡️ Next: Getting Started](getting-started.md)
