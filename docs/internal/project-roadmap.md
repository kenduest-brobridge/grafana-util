# Project Roadmap

Date: 2026-03-13
Source: Derived from `docs/internal/project-value-assessment.md` and current repo state.

## Purpose

This roadmap exists to keep the project moving in a coherent direction.

It is not a raw backlog dump. It is a prioritization document for deciding what to do next, what to defer, and what outcomes should define success.

## Positioning

The project should continue to position itself as:

- a Grafana migration, inspection, diff, and governance CLI

The project should not drift into:

- a generic all-in-one Grafana platform
- a full replacement for Terraform or native provisioning
- a dashboard-only exporter with loosely related extra commands

## Planning Principles

- Prefer features that improve migration safety, inspection depth, and governance value.
- Prefer work that reduces long-term complexity before adding large new surfaces.
- Keep Python and Rust behavior aligned through shared contracts and fixtures.
- Avoid roadmap items that broaden scope without strengthening the core operator value.

## Current State

Current strengths:

- Python and Rust both have strong test coverage
- CI now enforces a basic quality gate
- export / import / diff / inspect workflows already exist
- dashboard inspection capabilities are growing in useful ways

Current constraints:

- dashboard workflows are still the main complexity center
- Python and Rust dual maintenance increases coordination cost
- datasource lifecycle support is still incomplete
- access-management follow-through is now mostly about parity gaps in TLS/auth options and destructive-command live validation rather than missing core commands

## Roadmap Overview

### Phase 1: Stabilize And Simplify Core Dashboard Workflows

Target outcome:

- dashboard export/import/inspect remain the core strength of the repo, but with lower maintenance cost and clearer module boundaries

Priority items:

- continue splitting oversized Python dashboard orchestration into smaller helpers
- continue keeping Rust dashboard live/export/import/inspect flows separated by responsibility
- keep one stable inspection summary/report model across render modes
- reduce repeated live Grafana lookups during dashboard import and dry-run paths
- clean repo workflow noise so scratch outputs do not pollute normal review paths

Why this phase comes first:

- the dashboard surface is already the most valuable and the most complex
- reducing complexity now makes later feature work cheaper and safer

Definition of done for this phase:

- dashboard orchestration is no longer concentrated in one or two oversized modules
- inspection renderers share canonical intermediate models instead of ad hoc parallel paths
- import and dry-run behavior is easier to change without touching unrelated code
- common local scratch output no longer pollutes routine status/review workflows

Explicit non-goals for this phase:

- no broad new Grafana resource families yet
- no major packaging or distribution redesign

### Phase 2: Deepen Inspection And Governance Value

Target outcome:

- the project becomes materially more useful for understanding and governing Grafana state, not only moving it

Priority items:

- extend inspection into richer dependency analysis
- add datasource usage and orphan-detection report modes
- refactor query extraction behind datasource-type-specific analyzers
- improve query-family coverage for Prometheus, Loki, Flux/Influx, SQL, and future datasource families where practical
- keep report output useful for both humans and automation

Why this phase matters:

- inspection is one of the strongest differentiators in the repo
- governance features increase long-term value beyond one-time migration tasks

Definition of done for this phase:

- operators can answer datasource dependency questions without custom scripts
- inspection reports support common governance and cleanup workflows
- query analysis logic is modular enough to evolve per datasource family

Explicit non-goals for this phase:

- no attempt to parse every possible query language exhaustively
- no UI/dashboard product layer on top of inspection

### Phase 3: Add First-Class Datasource Lifecycle Support

Target outcome:

- the repo can manage datasource resources with the same seriousness already applied to dashboards and alerts

Priority items:

- add datasource `list`, `export`, `import`, and `diff` workflows
- define a stable datasource import/export contract
- strip server-managed fields consistently
- handle secure settings with explicit rules
- support clear import modes such as create-only, create-or-update, and update-existing-only
- add cross-language fixtures that lock Python/Rust normalization together

Why this phase matters:

- datasource state is central to migration and governance
- dashboards alone are not enough for credible environment portability

Definition of done for this phase:

- datasource resources can be exported, compared, and replayed through a stable contract
- Python and Rust produce compatible normalized results
- sensitive fields are handled predictably and safely

Explicit non-goals for this phase:

- no secret-management abstraction beyond the minimum needed for safe import/export behavior
- no attempt to cover every vendor-specific datasource quirk immediately

### Phase 4: Strengthen Migration Safety And Preflight Checks

Target outcome:

- imports become safer and more predictable before they mutate target Grafana environments

Priority items:

- add broader dependency preflight for datasources, plugins, and alert/contact references
- improve reporting for missing prerequisites before import starts
- keep diff and dry-run outputs trustworthy enough for review workflows
- consider bundle/package workflows that snapshot dashboards, alerts, datasources, and metadata together

Why this phase matters:

- migration safety is one of the repo's core promises
- preflight is high leverage because it prevents bad writes before they happen

Definition of done for this phase:

- operators can detect common migration blockers before mutation
- dry-run and preflight outputs are credible enough to use in team review workflows
- bundle/package support exists only if it clearly improves repeatable migration

Explicit non-goals for this phase:

- no broad policy engine
- no deployment orchestration outside the CLI's core scope

### Phase 5: Keep Access Surface Stable Pragmatically

Target outcome:

- access-management support stays usable and aligned without distracting from the project's core migration and inspection value

Priority items:

- finish shared TLS/auth option parity such as `--insecure` and `--ca-cert`
- keep per-command auth preflight explicit and tested
- extend live validation coverage for destructive access commands
- preserve `group` alias and compatibility entrypoint behavior without letting them drive new scope

Why this phase is later:

- useful, but less differentiating than migration/inspection/datasource work
- should stay in maintenance mode rather than consuming roadmap focus now that the planned command surface exists

Definition of done for this phase:

- the current access command surface remains stable and matches the established command model
- auth and TLS requirements are explicit and tested
- destructive flows have enough live validation coverage to catch drift early

Explicit non-goals for this phase:

- no ambition to become a full Grafana identity-management suite

## Cross-Cutting Work

These items should continue across phases instead of waiting for one specific milestone:

- keep CI aligned with local quality commands
- keep Python and Rust help text, contracts, and fixtures synchronized
- update maintainer docs when behavior or architecture changes materially
- resist scope creep that does not reinforce migration, inspection, diff, or governance value

## Priority Order Right Now

If only a small number of items can be advanced next, the recommended order is:

1. finish dashboard complexity reduction
2. deepen inspection and governance reporting
3. add first-class datasource lifecycle workflows
4. improve migration preflight and package safety
5. keep the access-management surface stable and low-drag

## Success Metrics

The roadmap is working if these become true:

- dashboard and alert migrations require less manual repair
- inspection reports replace one-off operator scripts for common governance questions
- datasource dependencies become understandable from CLI output alone for common cases
- Python and Rust stay aligned without frequent parity regressions
- feature growth does not cause orchestration complexity to spike again

## Bottom Line

The project should grow by going deeper on its strongest use cases, not by becoming broader for its own sake.

The best direction is:

- safer migration
- stronger inspection
- clearer governance visibility
- stable cross-language behavior
- lower orchestration complexity
