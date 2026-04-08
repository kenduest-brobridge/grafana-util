# Grafana 12.4 Observability as Code Assessment

Last reviewed: 2026-04-08

## Purpose

Capture what Grafana 12.4 now provides for Observability as Code, where it overlaps with `grafana-util`, and which integrations are worth building instead of competing head-on.

## Short conclusion

Grafana 12.4 materially strengthens its official Observability as Code story, but the new product surface is still narrow.

- The main official addition is Git-backed and file-backed provisioning for dashboards and folders.
- This is meaningful for dashboard Git workflows, but it is not a full Grafana GitOps replacement.
- `grafana-util` still has broader operator coverage across dashboards, alerts, access, snapshot bundles, and staged review.
- The best strategy is adapter + review + orchestration, not cloning Grafana Git Sync feature-for-feature.

## What Grafana 12.4 officially provides

Grafana now groups this work under "Observability as Code".

- Official entrypoint: Observability as Code
- Related official surfaces:
  - `grafanactl` / Grafana CLI
  - Foundation SDK
  - Git Sync
  - On-prem file provisioning
  - JSON schema v2
  - Terraform and other IaC integrations

### Observability as Code umbrella

The official docs describe this as a single path to manage Grafana resources programmatically with version control and CI/CD.

Important details:

- Grafana says Observability as Code supports versioning, automation, and scaling of Grafana configurations and workflows.
- Grafana explicitly says `grafanactl`, Git Sync, and the Foundation SDK are built on top of the new APIs.
- Grafana positions this as the replacement for a previously fragmented tool story.

Inference:

- Grafana is standardizing around new APIs plus multiple authoring and provisioning surfaces.
- This is strategic, but resource coverage is still uneven.

### Git Sync

Git Sync is the most relevant 12.4 addition for this repo.

What it does:

- Lets Grafana sync dashboards from a Git repository.
- Is bidirectional: changes can originate in Git or in the Grafana UI.
- Supports repository, branch, and path scoping.
- Can use PR-based or direct-commit style workflows depending on provider and policy.

Important limitations from the docs:

- Git Sync is still preview / experimental depending on edition.
- It only supports dashboards and folders.
- Full-instance sync is not supported.
- Some supported resources may still be incompatible until Grafana ships migration tooling.

Implication:

- Git Sync matters for dashboard repo workflows.
- It does not replace alert, access, datasource, or cross-resource review workflows.

### On-prem file provisioning

Grafana 12 also introduces a newer on-prem file provisioning story under the same Observability as Code area.

What it does:

- Watches local filesystem resources.
- Supports folders and dashboard JSON files.
- Can map multiple folders/repositories.

Important limitations from the docs:

- Experimental.
- On-prem only, not Grafana Cloud.
- Limited support and no SLA.

Implication:

- This overlaps with our dashboard `provisioning/` lane.
- It does not cover the full set of resources that `grafana-util` handles.

### grafanactl / Grafana CLI

Grafana documents `grafanactl` as a CLI on top of the new REST APIs.

What it appears to target:

- Authenticated CLI interaction with Grafana
- Multi-environment work
- Administrative operations
- CI/CD and terminal-driven workflows

Implication:

- This is the most direct "official CLI" pressure on `grafana-util`.
- But it is still API-centric. It does not obviously replace our operator-focused review, staging, conversion, history, and prompt/TUI flows.

### Foundation SDK

Foundation SDK is a typed authoring model for dashboards and related resources.

What it does:

- Uses builders and strong typing
- Supports Go and TypeScript directly, with references to additional languages
- Is explicitly positioned for CI/CD provisioning of dashboards

Implication:

- Strong competition for "dashboard generation from code"
- Much less overlap with our operator review and migration tooling
- Potential source format to ingest, validate, or bridge from

### JSON schema v2

Grafana also promotes schema v2 under the same area.

Implication:

- This is relevant for forward-compatibility and validation.
- It is not, by itself, a full GitOps workflow.

## Feature comparison

| Area | Grafana 12.4 official | `grafana-util` | Assessment |
| --- | --- | --- | --- |
| Dashboard Git-backed workflow | Yes, via Git Sync | Yes, via export/import/publish/review | High overlap |
| Dashboard file provisioning | Yes | Yes | Overlap |
| Dashboard live edit and browse | Limited in official Git Sync framing | Strong live browse/edit/review/apply flows | `grafana-util` stronger |
| Dashboard history export / offline restore review | Not equivalent | Yes | `grafana-util` stronger |
| Alert desired-state workflow | Official APIs and external IaC exist, but not the main 12.4 Git Sync surface | Yes: export/import/diff/plan/apply | `grafana-util` stronger |
| Access GitOps / identity operations | Not covered by Git Sync | Yes | `grafana-util` stronger |
| Snapshot / offline handoff artifact | Not a flagship 12.4 feature | Yes | `grafana-util` stronger |
| Cross-resource staged review | Fragmented across tools | Yes: `change inspect/check/preview` | `grafana-util` stronger |
| Typed dashboard authoring | Foundation SDK | Partial via JSON lanes and generators | Grafana stronger |
| Official schema standardization | Schema v2 | Validation lanes exist, but not schema-v2-native | Grafana stronger |

## Where the repo overlaps with Grafana 12.4

### Dashboard export / provisioning / publish

This repo already exposes multiple dashboard lanes:

- `raw`
- `prompt`
- `provisioning`
- live publish and dry-run review

Relevant code and docs:

- [README.md](../../README.md)
- [rust/src/dashboard/cli_defs_command.rs](../../rust/src/dashboard/cli_defs_command.rs)
- [docs/commands/en/dashboard-import.md](../commands/en/dashboard-import.md)

Most relevant overlap:

- `dashboard export`
- `dashboard import`
- `dashboard publish`
- dashboard provisioning-compatible output

### Dashboard live operator flows

This repo has stronger operator UX than the official Git-backed model currently exposes:

- `dashboard browse`
- `dashboard edit-live`
- `dashboard history list/export/diff/restore`

Relevant code:

- [rust/src/dashboard/cli_defs_command.rs](../../rust/src/dashboard/cli_defs_command.rs)
- [rust/src/dashboard/history.rs](../../rust/src/dashboard/history.rs)

### Staged workspace and change review

This repo already has a staged-input discovery and review model that spans multiple domains.

Relevant code:

- [rust/src/sync/guided.rs](../../rust/src/sync/guided.rs)

This is a major differentiator. Grafana's new OaC story does not obviously provide one unified cross-resource preflight and review layer.

### Alert workflows

This repo already has its own alert desired-state and replay pipeline:

- export
- import
- diff
- plan
- apply
- route preview

Relevant code:

- [rust/src/alert.rs](../../rust/src/alert.rs)

Important caveat:

- Current alert import explicitly rejects Grafana provisioning-style alert export payloads.

Relevant code:

- [rust/src/alert_support.rs](../../rust/src/alert_support.rs)

Implication:

- If users increasingly adopt official Grafana alert IaC formats, this repo will need bridge tooling instead of assuming its current raw alert contract remains the only path.

### Snapshot / offline handoff

This repo has a broader "offline artifact" story than the official Git Sync docs.

- dashboard
- datasource
- access
- metadata bundle

Relevant docs:

- [docs/commands/en/snapshot.md](../commands/en/snapshot.md)

## Impact on `grafana-util`

### Areas under the most pressure

1. Dashboard repo-backed provisioning

- Grafana now has an official answer.
- This reduces the value of a custom dashboard-only GitOps pitch.

2. Dashboard generation from code

- Foundation SDK is a credible official path.
- Teams that want typed dashboard generation may prefer Grafana-native tooling.

3. Generic CLI access to official APIs

- `grafanactl` is the clearest direct CLI overlap.

### Areas where `grafana-util` still stands out

1. Operator-grade live workflows

- browse
- edit-live
- history inspection and restore
- prompt/TUI review and confirmation

2. Cross-resource review and staged governance

- dashboards + alerts + datasources + access in one review story

3. Offline artifact workflows

- history artifacts
- snapshot bundles
- review-first handoff

4. Access and identity operations

- users
- teams
- orgs
- service accounts
- tokens

5. Alert orchestration

- desired tree
- plan/apply
- route preview

## Recommended strategy

Do not try to beat Grafana by reproducing Git Sync as a second Git Sync.

Preferred strategy:

- adapt to official layouts
- validate and review official layouts
- bridge across resources Grafana still treats separately
- preserve stronger live operator UX

## Design recommendation for this repo

To avoid design drift, this repo should treat Grafana 12.4 support as an adapter problem, not as a reason to start a second product architecture.

### Recommended product position

`grafana-util` should position itself as the operator-grade review, migration, and cross-resource control plane for Grafana, with adapters for both legacy exports and official Grafana Observability as Code layouts.

This positioning matters because it keeps the repo focused on:

- review before mutation
- cross-resource workflows
- live and offline operator tooling
- migration and compatibility bridges

It avoids turning the repo into:

- a clone of Git Sync
- a clone of `grafanactl`
- a dashboard-only GitOps tool that ignores alerts, access, and staged review

### Design principles

#### 1. One internal canonical model

Do not create parallel product lines for:

- legacy raw exports
- provisioning exports
- Git Sync repos
- schema v2
- Foundation SDK outputs

Instead, all supported inputs should normalize into the same internal domain model.

Recommended stable internal models:

- dashboard draft
- dashboard inventory item
- dashboard history artifact
- alert desired-state bundle
- access inventory bundle
- staged change bundle

This is the most important protection against long-term design sprawl.

#### 2. Stable commands, extensible formats

Prefer extending existing commands with new input or export formats.

Good direction:

- `dashboard analyze --input-format git-sync`
- `dashboard import --input-format git-sync`
- `dashboard export --export-layout git-sync`
- `dashboard validate-export --target git-sync`

Avoid:

- `dashboard git-sync analyze`
- `dashboard git-sync publish`
- new namespaces that duplicate the existing command surface

Command sprawl will make the repo harder to understand and harder to document.

#### 3. Official dashboard OaC stays a lower layer

Grafana 12.4 strengthens dashboard/folder repo workflows.

That should be treated as a source and provisioning layer, not as the repo's new top-level architecture.

`grafana-util` should continue to own the higher-value operator workflows:

- review
- diff
- preview
- publish/apply with safeguards
- live browse/edit/history
- snapshot/handoff
- cross-resource staged inspection

#### 4. Review-first remains the core contract

Every new format or integration should be tested against the same questions:

- can we inspect it?
- can we diff it?
- can we preview it?
- can we produce a durable artifact for handoff or CI?

If a new integration only adds replay and not review, it weakens the repo.

#### 5. Bridge first, rewrite later

Where Grafana's official formats differ from current repo contracts, prefer bridges.

Examples:

- Git Sync repo -> dashboard canonical model
- schema v2 -> dashboard canonical model
- Grafana alert provisioning -> alert canonical model

Bridges are safer than replacing the current workflow model immediately.

## Recommended architecture shape

### Layer 1: Source adapters

This layer should parse and normalize inputs from multiple sources:

- live API
- raw exports
- provisioning exports
- Git Sync repo layouts
- schema v2 documents
- Foundation-generated JSON artifacts

Responsibilities:

- parse
- validate
- normalize
- preserve source metadata

Non-goal:

- do not embed mutation workflow semantics here

### Layer 2: Canonical domain models

This layer should define the stable documents used by the rest of the repo.

Responsibilities:

- internal contracts for dashboards, alerts, access, history, and staged change
- compatibility boundary between source adapters and workflows

This is the layer that should stay stable even as new upstream Grafana formats appear.

### Layer 3: Operator workflows

This layer should continue to power the value-added commands:

- `review`
- `diff`
- `preview`
- `publish`
- `plan` / `apply`
- `browse`
- `edit-live`
- `history`
- `snapshot`
- `change inspect/check/preview`

These workflows should consume canonical models, not upstream raw formats directly.

## What should be built first

### Priority 1: Git Sync-aware dashboard ingestion

Recommended near-term additions:

- `dashboard analyze --input-format git-sync`
- `dashboard import --input-format git-sync`
- `dashboard validate-export --target git-sync`
- `dashboard export --export-layout git-sync`

Reason:

- This captures the most important Grafana 12.4 overlap without redesigning the whole repo.

### Priority 2: `change` workspace discovery for official repo layouts

Recommended near-term additions:

- detect Git Sync dashboard repo layouts in staged workspace discovery
- allow `change inspect` and `change preview` to operate on those repos

Reason:

- This is the clearest place where the repo can do more than Grafana itself.
- It turns official dashboard-as-code into one part of a broader operator review system.

### Priority 3: Alert bridge tooling

Recommended additions:

- `alert convert --from grafana-provisioning`
- `alert plan --input-format grafana-provisioning`
- optional Terraform-oriented bridge helpers later

Reason:

- Current alert import explicitly rejects Grafana provisioning-style alert payloads.
- Bridge tooling is the safest way to remain compatible without destabilizing the current alert lane.

### Priority 4: Schema v2 and Foundation compatibility checks

Recommended additions:

- `dashboard validate-export --target schema-v2`
- `dashboard review --input-format schema-v2`

Reason:

- This keeps the repo aligned with Grafana's official format direction without forcing an immediate rewrite.

## What should not be built first

These are likely distractions or architecture traps:

- cloning `grafanactl` command-for-command
- creating a separate `git-sync` namespace that duplicates current dashboard commands
- forcing alerts or access resources into dashboard-centric official repo layouts
- replacing current canonical contracts before adapter support exists
- removing live operator workflows in favor of repo-only workflows

## Where this repo can still be better than Grafana

### 1. Cross-resource review

Grafana's official OaC story is still resource-fragmented.

This repo can continue to unify:

- dashboards
- datasources
- alerts
- access
- promotion mapping and availability context

### 2. Live plus offline operator tooling

Grafana's Git-backed model does not replace:

- live browse
- edit-live
- history restore
- snapshot capture
- offline artifact handoff

This repo should preserve and strengthen those workflows.

### 3. Migration and compatibility tooling

Real users will operate mixed environments.

This repo can provide:

- legacy-to-official format bridges
- official-to-canonical review flows
- validation and preflight before promotion

### 4. Safer mutation UX

This repo can remain stronger in:

- preview-first changes
- interactive review
- prompt/TUI confirmations
- explicit diff and summary output

## Suggested positioning language

If the repo messaging evolves, the safer direction is:

`grafana-util` is the operator-grade review, migration, and cross-resource control plane for Grafana environments. It complements both legacy export workflows and Grafana's official Observability as Code layouts with stronger review, live operations, offline artifacts, and cross-resource orchestration.

## Immediate planning implication

If future work is opened in this area, maintainers should evaluate proposed changes against this rule:

Does the change strengthen the adapter + canonical model + operator workflow architecture, or does it introduce a second competing command family tied to one upstream format?

Only the first kind of change should be considered default-safe.

## Recommended first implementation slice

To keep follow-up work coherent, the first engineering slice should focus on foundations, not on end-user command expansion.

### First slice goals

1. Introduce a shared dashboard source-kind model
2. Use that model to reduce hardcoded `raw` versus `provisioning` branching
3. Prepare staged workspace discovery to become extensible later
4. Keep command UX stable while internal source handling becomes cleaner

### Start with these foundations

#### A. Dashboard source adapter abstraction

Define one shared internal classification for dashboard sources.

Initial practical variants should cover what the repo already uses:

- live Grafana
- raw export
- provisioning export
- dashboard history artifact

Future-safe reserved directions:

- Git Sync repo
- schema v2

This should become the common language used by import, diff, analyze, browse, vars, and staged change discovery.

#### B. Canonical dashboard source resolution

Centralize dashboard source resolution instead of repeating local `raw` versus `provisioning` decisions in multiple modules.

Short-term target:

- one helper resolves local dashboard source kind
- one helper resolves expected variant / metadata contract
- `ResolvedDashboardImportSource` carries source kind explicitly

This does not need to solve Git Sync yet. It only needs to stop current branching from spreading.

#### C. Discovery extensibility groundwork

Do not implement Git Sync discovery immediately, but prepare `change` staged-input discovery to support pluggable layout detection later.

Current hardcoded discovery works for:

- `dashboards/raw`
- `dashboards/provisioning`
- `alerts/raw`
- `datasources/provisioning`

The first slice should keep behavior stable while clarifying where a future Git Sync detector would plug in.

#### D. Validation layering

Separate three concerns:

- parse validation
- source-layout validation
- publish/apply safety validation

This is required before adding Git Sync-aware validation or schema-v2-aware validation safely.

### Modules to touch first

- [rust/src/dashboard/files.rs](../../rust/src/dashboard/files.rs)
- [rust/src/dashboard/import.rs](../../rust/src/dashboard/import.rs)
- [rust/src/dashboard/import_compare.rs](../../rust/src/dashboard/import_compare.rs)
- [rust/src/dashboard/import_validation.rs](../../rust/src/dashboard/import_validation.rs)
- [rust/src/dashboard/browse_support.rs](../../rust/src/dashboard/browse_support.rs)
- [rust/src/dashboard/vars.rs](../../rust/src/dashboard/vars.rs)
- [rust/src/sync/guided.rs](../../rust/src/sync/guided.rs)

### Explicit non-goals for the first slice

- no new public `git-sync` subcommands
- no attempt to clone `grafanactl`
- no alert bridge implementation yet
- no schema v2 parser yet
- no command-surface expansion before source handling is cleaner

### Why this slice first

If Git Sync support is added before source-kind and source-resolution cleanup, the repo will grow a second dashboard import architecture and later refactoring cost will be much higher.

## Best integration opportunities

### 1. Add Git Sync layout awareness to dashboard commands

Recommended additions:

- `dashboard analyze --input-format git-sync`
- `dashboard import --input-format git-sync`
- `dashboard validate-export --target git-sync`
- `dashboard export --git-sync-layout`

Why:

- Lets users adopt official repo layouts without losing `grafana-util` review and validation.

### 2. Teach `change` workflows to discover Git Sync repos

Recommended additions:

- `change inspect --workspace <git-sync-repo>`
- `change preview --workspace <git-sync-repo>`

Why:

- This lets `grafana-util` become the cross-resource review layer on top of Grafana's official dashboard repo model.

### 3. Add alert bridge tooling instead of changing the current alert contract

Recommended additions:

- `alert convert --from grafana-provisioning`
- `alert plan --input-format grafana-provisioning`
- `alert export --terraform-hints`

Why:

- The current repo already has a coherent alert desired-state lane.
- It is safer to bridge into that lane than to replace it.

### 4. Add Foundation SDK ingestion or validation

Recommended additions:

- `dashboard review --input-format foundation-json`
- `dashboard validate-export --target schema-v2`

Why:

- Teams adopting official dashboard authoring still need review, policy, and publish guardrails.

### 5. Position snapshot as complement to Git Sync

Recommended additions:

- `snapshot export --git-sync-seed`
- `snapshot review` enhancements that compare snapshot state to a Git Sync repo

Why:

- Git Sync does not replace incident-time offline capture.

## Suggested roadmap order

### Highest-value near-term work

1. Git Sync repo detection and dashboard analysis support
2. Git Sync-aware validation
3. `change` workspace discovery for official dashboard repo layouts

### Medium-term work

4. Alert format bridges
5. Schema v2 compatibility checks
6. Foundation SDK-derived input support

### Lower priority

7. Trying to mirror `grafanactl` command-for-command

This is low-value unless users explicitly ask for it.

## Concrete repo areas likely to change

### Dashboard

- [rust/src/dashboard/cli_defs_command.rs](../../rust/src/dashboard/cli_defs_command.rs)
- dashboard input-format enums and local loaders
- dashboard validation and analyze surfaces

### Change / staged discovery

- [rust/src/sync/guided.rs](../../rust/src/sync/guided.rs)

### Alert bridges

- [rust/src/alert.rs](../../rust/src/alert.rs)
- [rust/src/alert_support.rs](../../rust/src/alert_support.rs)

### Docs and positioning

- [README.md](../../README.md)
- user guide pages for dashboard, alert, and architecture

## Recommended positioning statement

If project messaging needs to evolve, the repo should stop sounding like a replacement for all Grafana-as-code tooling and instead sound like this:

`grafana-util` is the operator-grade review, migration, and cross-resource control plane for Grafana environments, including live workflows, offline artifacts, staged review, and integration with both legacy exports and emerging official Grafana-as-code layouts.

## Sources

Primary official sources reviewed on 2026-04-08:

- Grafana Observability as Code:
  - https://grafana.com/docs/grafana/latest/as-code/observability-as-code/
- Grafana Git Sync:
  - https://grafana.com/docs/grafana/latest/as-code/observability-as-code/git-sync/
  - https://grafana.com/docs/grafana/latest/as-code/observability-as-code/git-sync/key-concepts/
  - https://grafana.com/docs/grafana/latest/as-code/observability-as-code/git-sync/usage-limits/
  - https://grafana.com/docs/grafana/latest/as-code/observability-as-code/git-sync/git-sync-setup/
- Grafana on-prem file provisioning:
  - https://grafana.com/docs/grafana/latest/as-code/observability-as-code/provision-resources/
- Grafana Foundation SDK:
  - https://grafana.com/docs/grafana/latest/as-code/observability-as-code/foundation-sdk/
- Grafana JSON schema v2:
  - https://grafana.com/docs/grafana/latest/as-code/observability-as-code/schema-v2/
- Grafana 12.4 "what's new":
  - https://grafana.com/docs/grafana/latest/whatsnew/whats-new-in-v12-4/

## Notes on confidence

- High confidence:
  - Git Sync is limited to dashboards and folders.
  - Git Sync and on-prem file provisioning are still preview/experimental.
  - `grafanactl`, Git Sync, and Foundation SDK are part of Grafana's official OaC story.
- Medium confidence:
  - `grafanactl` will become a direct competitive CLI surface for some workflows.
  - Foundation SDK pressure will be strongest on teams doing typed dashboard generation.
- Explicit inference:
  - Grafana 12.4 does not yet provide a full-instance GitOps replacement for this repo's broader operator workflows.
