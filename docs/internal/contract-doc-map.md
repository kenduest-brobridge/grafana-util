# Contract Documentation Map

Current guide for where contract information belongs.

Use three layers:

- Summary:
  short maintainer-facing policy in `docs/DEVELOPER.md`
- Spec:
  detailed current requirements in dedicated `docs/internal/*` contract docs
- Trace:
  concise status/change history in `docs/internal/ai-status.md` and
  `docs/internal/ai-changes.md`

## Current Contract Specs

- Repo-level export-root policy:
  [`export-root-output-layering-policy.md`](docs/internal/export-root-output-layering-policy.md)
- Dashboard export-root contract:
  [`dashboard-export-root-contract.md`](docs/internal/dashboard-export-root-contract.md)
- Datasource masked-recovery contract:
  [`datasource-masked-recovery-contract.md`](docs/internal/datasource-masked-recovery-contract.md)
- Alert/access boundary policy:
  [`alert-access-contract-policy.md`](docs/internal/alert-access-contract-policy.md)
- CLI/docs surface contract:
  [`scripts/contracts/command-surface.json`](scripts/contracts/command-surface.json)
- Docs-entrypoint contract:
  [`scripts/contracts/docs-entrypoints.json`](scripts/contracts/docs-entrypoints.json)
- JSON output contract registry:
  [`scripts/contracts/output-contracts.json`](scripts/contracts/output-contracts.json)
- Schema/help manifest source:
  [`schemas/manifests/`](schemas/manifests/)
  - family-owned `contracts.json` and `routes.json`
  - generated schema artifacts under `schemas/jsonschema/`
  - generated schema-help artifacts under `schemas/help/`
- Generated docs navigation projections:
  [`scripts/contracts/command-reference-index.json`](scripts/contracts/command-reference-index.json)
  and [`scripts/contracts/handbook-nav.json`](scripts/contracts/handbook-nav.json)

## Runtime Output Contract Inventory

Runtime output contracts are executable regression gates for machine-readable
JSON produced by commands or persisted artifact workflows. They are owned by
`scripts/contracts/output-contracts.json` and the matching checked-in fixtures
under `scripts/contracts/output-fixtures/`.

Current runtime output-contract artifacts:

| Contract name | Runtime `kind` | Golden fixture | Ownership note |
| --- | --- | --- | --- |
| `sync-plan` | `grafana-utils-sync-plan` | `sync-plan.json` | Workspace preview JSON output shape |
| `sync-preflight` | `grafana-utils-sync-preflight` | `sync-preflight.json` | Workspace preflight/test JSON output shape |
| `sync-source-bundle` | `grafana-utils-sync-source-bundle` | `sync-source-bundle.json` | Workspace source-bundle artifact shape |
| `datasource-export-index` | `grafana-utils-datasource-export-index` | `datasource-export-index.json` | Datasource masked-recovery export index shape |
| `dashboard-summary-governance` | `grafana-utils-dashboard-summary-governance` | `dashboard-summary-governance.json` | Dashboard summary governance JSON shape |
| `dashboard-dependencies-topology` | `grafana-utils-dashboard-dependencies-topology` | `dashboard-dependencies-topology.json` | Dashboard dependency topology JSON shape |
| `dashboard-impact` | `grafana-utils-dashboard-impact` | `dashboard-impact.json` | Dashboard impact JSON shape |
| `dashboard-policy-gate` | `grafana-utils-dashboard-policy-gate` | `dashboard-policy-gate.json` | Dashboard policy gate JSON shape |

These files are runtime contracts because the registry validates concrete
output fixtures for root fields, fixed values, nested paths, path types,
collection item types, collection minimums, enum values, and forbidden fields.
They do not automatically become published schema/help contracts just because
they have stable golden coverage.

## Public Schema And Help Contract Inventory

Public schema/help contracts are authored in `schemas/manifests/**` and
projected into generated artifacts. The manifest layer owns the published
contract description; generated files are checkable outputs, not policy sources.

Authoring sources:

- `schemas/manifests/change/contracts.json`
- `schemas/manifests/change/routes.json`
- `schemas/manifests/status/contracts.json`
- `schemas/manifests/status/routes.json`
- `schemas/manifests/dashboard-history/contracts.json`
- `schemas/manifests/dashboard-history/routes.json`
- `schemas/manifests/diff/contracts.json`
- `schemas/manifests/diff/routes.json`

Generated public artifacts:

- JSON Schemas under `schemas/jsonschema/change/`
- JSON Schemas under `schemas/jsonschema/status/`
- JSON Schemas under `schemas/jsonschema/dashboard-history/`
- JSON Schemas under `schemas/jsonschema/diff/`
- schema-help files under `schemas/help/change/`
- schema-help files under `schemas/help/status/`
- schema-help files under `schemas/help/dashboard-history/`
- schema-help files under `schemas/help/diff/`

Current manifest families:

| Family | Contract IDs | Generated artifact role |
| --- | --- | --- |
| `change` | `sync-summary`, `sync-plan`, `sync-plan-reviewed`, `sync-apply-intent`, `sync-apply-live-result`, `sync-audit`, `sync-preflight`, `alert-sync-plan`, `sync-bundle-preflight`, `sync-promotion-preflight`, `sync-source-bundle` | Workspace/change JSON schema and `--help-schema` text |
| `status` | `project-status` | Project status JSON schema and status schema help |
| `dashboard-history` | `dashboard-history-list`, `dashboard-history-inventory`, `dashboard-history-restore`, `dashboard-history-export`, `dashboard-history-diff` | Dashboard history JSON schemas and help text |
| `diff` | `grafana-util-dashboard-diff`, `grafana-util-alert-diff`, `grafana-util-datasource-diff` | Cross-domain diff JSON schemas and help text |

Treat `schemas/jsonschema/**` and `schemas/help/**` as generated projections.
When they drift, update the manifest source or generator and regenerate; do not
hand-edit the generated artifact as the ownership fix.

## Ownership Rules

- Command-surface contracts own public CLI routing.
  - Treat `scripts/contracts/command-surface.json` as the source of truth for
    public command paths, legacy replacements, removed public paths, docs
    routing, and `--help-full` / `--help-flat` support.
  - Keep `scripts/contracts/command-surface.json` current when public paths or
    docs-routing behavior change.

- Docs-entrypoint contracts own navigation shortcuts.
  - Treat `scripts/contracts/docs-entrypoints.json` as the source of truth for
    landing quick commands, jump-select entries, and handbook sidebar command
    shortcuts.
  - Treat `scripts/contracts/command-reference-index.json` and
    `scripts/contracts/handbook-nav.json` as generated navigation projections,
    not as primary authoring surfaces.

- Output contracts own runtime golden regression gates.
  - Treat `scripts/contracts/output-contracts.json` as the registry for
    machine-readable runtime JSON output contracts.
  - Use it to define which fields, nested paths, array shapes, enum values, and
    forbidden fields must stay stable in golden regression fixtures.
  - When a runtime output shape changes, update the contract registry and the
    matching runtime golden fixtures together so the checker is verifying live
    behavior, not a stale expectation.

- Schema manifests own published schema/help contracts.
  - Treat `schemas/manifests/**/contracts.json` and `schemas/manifests/**/routes.json`
    as the source of truth for published schema/help surfaces.
  - The generated `schemas/jsonschema/` and `schemas/help/` trees are published
    artifacts derived from those manifests, not the place to author policy.
  - Any new `--help-schema` or schema-oriented command surface should be
    represented in the manifest layer first, then projected into generated
    schema/help output.

- Stable public artifacts need a promotion gate.
  - Promote an output-contract artifact into schema manifests only when it is
    intentionally documented as a public `--help-schema` or schema-help surface,
    not merely because a fixture exists.
  - Before promotion, confirm the output shape has a stable `kind` /
    `schemaVersion` discriminator, documented top-level fields, documented
    nested sections that scripts may consume, and a compatibility expectation
    for additive versus breaking changes.
  - The promotion patch should add or update manifest coverage in
    `schemas/manifests/**/contracts.json`, route coverage in the matching
    `routes.json`, generated schema/help artifacts through `make schema`, and
    runtime golden coverage in `scripts/contracts/output-contracts.json` when
    behavior-sensitive output fields need regression checks.
  - Public command paths should also have command-surface and docs-entrypoint
    evidence when the schema/help surface is discoverable from CLI or docs
    navigation.
  - If the artifact is still under active shape churn, keep it in the runtime
    golden layer only and do not describe it as a stable public schema/help
    contract yet.
  - Use `make contract-promotion-report` as an informational evidence matrix
    across runtime golden, schema/help manifest, public CLI route,
    docs-entrypoint, generated-docs, and artifact-workspace lanes. Structural
    gaps are report findings by default; only explicit strict report mode should
    turn them into a failing gate.

## Contract Promotion Report

`make contract-promotion-report` is a maintainer audit, not a default quality
gate. Use it before promoting a JSON output or schema/help surface from
implementation detail to stable public contract.

Read the evidence matrix as lane ownership:

- `runtime_golden` points to the golden JSON fixture covered by
  `scripts/contracts/output-contracts.json`; this proves live command output is
  regression-checked.
- `schema/help_manifest` points to the family manifest under
  `schemas/manifests/`; this proves the published schema/help contract is
  authored in the manifest layer.
- `public_cli_route` shows manifest routes that resolve to public command
  paths from `scripts/contracts/command-surface.json`; this proves the contract
  is attached to a public CLI surface.
- `docs_entrypoint` shows whether the public command is present in
  `scripts/contracts/docs-entrypoints.json`; this proves landing-page or
  handbook navigation has an owned entrypoint.
- `generated_docs` shows generated command-reference files derived from the
  docs pipeline; this proves public command docs exist, but those files are not
  authoring sources.
- `artifact_workspace_lane` shows whether the command participates in an
  artifact workspace export or local-consumer lane from `command-surface.json`;
  this proves the contract is tied to an owned workspace artifact flow.

Missing cells are informational until a maintainer intentionally opts into the
strict report mode. Treat them as promotion evidence gaps, not immediate build
failures.

## Current Contract Lane Overlap

The output-contract registry and schema manifests intentionally do not have a
one-to-one contract-id mapping yet.

- Runtime output contracts currently cover deep golden regression checks for
  selected machine outputs, including dashboard summary/governance, dashboard
  topology/impact/policy, datasource export index, and sync plan/preflight/source
  bundle artifacts.
- Schema manifests currently publish schema/help contracts for status, change
  workflow, dashboard history, and diff families.
- Overlap should be promoted by artifact family, not by renaming IDs. For
  example, `grafana-utils-sync-plan` in the runtime output registry and
  `sync-plan` in the change schema manifest both describe the workspace preview
  family, but each layer owns a different validation purpose.

Promotion rule:

- Add or update runtime output-contract coverage when a JSON command output has
  behavior-sensitive fields that need golden regression checks.
- Add or update schema manifest coverage when the same output is documented as a
  published `--help-schema` or schema/help surface.
- Treat an output as a stable public schema/help surface only after both lanes
  have coverage, or after the maintainer doc records why one lane does not
  apply.

## Maintainer Rules

- Keep `docs/DEVELOPER.md` short enough to orient maintainers quickly.
- Put stable field lists, promotion gates, compatibility rules, and detailed
  contract requirements in the dedicated spec docs.
- Keep `ai-status.md` and `ai-changes.md` current and trace-oriented; do not
  restate full specs there.
- Archive older trace entries once they stop helping with current navigation.
- Keep `scripts/contracts/command-surface.json` current when public command paths, legacy
  replacements, docs routing, removed public path guards, or `--help-full` /
  `--help-flat` support change.
- Keep `scripts/contracts/docs-entrypoints.json` current when landing quick
  commands, jump-select entries, or handbook sidebar shortcuts change.
- Treat `scripts/contracts/command-reference-index.json` and
  `scripts/contracts/handbook-nav.json` as generated projections of the docs
  entrypoint and handbook routing contracts.
- Keep `scripts/contracts/output-contracts.json` current when adding or changing
  machine-readable JSON outputs; use root `requiredFields` for envelope checks and
  `requiredPaths` / `pathTypes` / `arrayItemTypes` / `minimumItems` / `enumValues`
  for small nested and collection shape guarantees in golden fixtures.
