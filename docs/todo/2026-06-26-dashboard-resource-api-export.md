# Grafana 13 Dashboard Resource Export Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add dashboard export support for Grafana dashboard resource API formats alongside the existing classic raw/prompt/provisioning export lanes.

**Architecture:** Keep classic dashboard export unchanged by default. Add a family-oriented artifact layout for exports that include Grafana dashboard resource API objects: `classic/` contains the existing raw/prompt/provisioning/history lanes, while `resource-v1/` and `resource-v2/` contain native API objects. Do not nest raw/prompt/provisioning under v1/v2 because those classifications belong to the classic dashboard JSON workflow, not the resource API workflow.

**Tech Stack:** Rust CLI, `serde_json::Value` compatibility models, existing `JsonHttpClient`, existing dashboard export metadata/index helpers, focused Rust unit tests, command docs generated through the existing docs pipeline.

---

## Current Reality

- Classic dashboard export lists dashboards through `GET /api/search` and fetches payloads through `GET /api/dashboards/uid/{uid}`.
- Export currently writes classic `raw/`, `prompt/`, and `provisioning/` lanes.
- Dashboard resource-shaped JSON where `apiVersion` starts with `dashboard.grafana.app/` is intentionally rejected by classic import, diff, plan, and prompt conversion paths.
- Existing fixtures already cover `dashboard.grafana.app/v2` and a pre-v2 resource wrapper as unsupported classic inputs.
- The `resource` command has an `auto` / `legacy` API mode, but dashboard resource listing/get still points at classic `/api/search` and `/api/dashboards/uid/{uid}`.
- Grafana's UI import flow accepts uploaded dashboard JSON, pasted Grafana.com dashboard URL/ID, or pasted dashboard JSON text. It is not a native dashboard resource v1/v2 YAML import surface.
- Grafana's new dashboard HTTP API is a separate `/apis` surface. Official docs describe `GET /apis/dashboard.grafana.app/v1/namespaces/:namespace/dashboards/:uid` and list through `GET /apis/dashboard.grafana.app/v1/namespaces/:namespace/dashboards`, with `kind`, `apiVersion`, `metadata`, and `spec` envelopes.
- Grafana docs describe `/api` as legacy and `/apis` as the new standardized API structure with consistent versioning.

## Requirements

- Preserve current default `grafana-util dashboard export` behavior and output layout.
- Add explicit dashboard resource export lanes for Grafana dashboard resource API formats.
- Support both v1-family and v2-family dashboard resource envelopes when Grafana exposes them.
- Keep native resource envelopes intact in resource lanes; do not convert v2 resources into classic `raw/`, `prompt/`, or `provisioning/`.
- Keep classic lane rejection for v1/v2 resource wrappers until resource import/diff/plan is separately designed.
- Make API-version selection explicit in CLI and metadata so operators know which Grafana API produced an artifact.
- Keep all-org export behavior consistent: each org scope may contain classic lanes plus resource lanes when requested.
- Use family-oriented export layout for resource-enabled exports so tools and humans can distinguish classic dashboard JSON from resource API objects before opening files.
- Treat JSON as the canonical resource artifact format for this slice. YAML may be added later as a projection, but it must not be presented as compatible with Grafana's UI import page.
- Update command docs, command-surface contract, and generated docs if the CLI surface changes.

## Non-Goals

- Do not implement dashboard resource import/apply in this slice.
- Do not change default classic export format.
- Do not silently coerce v2 `spec.elements` into classic dashboard `panels`.
- Do not add YAML resource output in the first implementation unless a verified Grafana workflow consumes it directly.
- Do not label resource v1/v2 artifacts as `prompt` or `provisioning`.
- Do not weaken Git Sync / file provisioning ownership guardrails.
- Do not make resource API support depend on the optional `browser` feature.

## Open Verification Before Coding

- Confirm the Grafana 13 dashboard resource API endpoint shapes from primary sources or a live Grafana 13 instance before implementing endpoint constants.
- Confirm whether Grafana exposes both v1 and v2 simultaneously, or whether v1 is represented by an alpha/beta version such as `dashboard.grafana.app/v1alpha1`.
- Confirm the resource identity field used by the API path: UID, metadata name, namespace, or another slug.
- Confirm whether org scoping uses the existing `orgId` query/header behavior or a resource namespace mapping.
- Confirm whether `dashboard.grafana.app/v2` has a public `/apis/.../v2/...` HTTP endpoint in the target Grafana 13 build, or whether v2 is an internal schema/conversion package while the public HTTP API remains v1.

## Format Meaning

Classic dashboard JSON and dashboard resource envelopes solve different jobs:

- Classic `raw/` is for API replay, diff, review, and backup using the legacy dashboard shape.
- Classic `prompt/` is for Grafana UI import, where a person uploads or pastes dashboard JSON and resolves datasource inputs.
- Classic `provisioning/` is for Grafana file provisioning, with dashboard JSON plus provider YAML.
- Resource v1/v2 objects are for the new `/apis` resource API shape: `kind`, `apiVersion`, `metadata`, `spec`, and optional `status`. They are closer to Kubernetes-style API resources than to the UI import payload.
- YAML is only a serialization possibility for resource-shaped objects. It is not useful for Grafana UI import unless Grafana documents a YAML import route. The initial implementation should export resource objects as JSON.

## Verified API Contract

Verified on 2026-06-27 against Grafana 13.1 `latest` documentation:

- API overview source:
  `https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/apis.md`
- Dashboard HTTP API source:
  `https://grafana.com/docs/grafana/latest/developers/http_api/dashboard.md`
- UI import source:
  `https://grafana.com/docs/grafana/latest/dashboards/build-dashboards/import-dashboards.md`

Confirmed endpoint table:

| Operation | Endpoint | Status for this slice |
| --- | --- | --- |
| List dashboards | `GET /apis/dashboard.grafana.app/v1/namespaces/:namespace/dashboards` | Public v1 endpoint; implement first. |
| Get dashboard | `GET /apis/dashboard.grafana.app/v1/namespaces/:namespace/dashboards/:uid` | Public v1 endpoint; implement first. |
| Get dashboard DTO | `GET /apis/dashboard.grafana.app/v1/namespaces/:namespace/dashboards/:uid/dto` | Documented, but not needed for native resource export unless a later task needs access metadata. |
| Create/update/delete dashboard | `POST`, `PUT`, `DELETE` under `/apis/dashboard.grafana.app/v1/namespaces/:namespace/dashboards` | Out of scope because this slice is export-only. |
| Public v2 dashboard HTTP endpoint | No `dashboard.grafana.app/v2` HTTP endpoint found in the Grafana 13.1 dashboard HTTP API docs. | Keep `resource-v2/` planned but disabled/unsupported until a primary source or live Grafana 13 transcript verifies a public v2 endpoint. |

Contract decisions:

- Grafana 13 uses `/apis` for the newer Kubernetes-style API surface; `/api` is legacy but still present.
- Dashboard resource paths follow `/apis/<group>/<version>/namespaces/<namespace>/<resource>[/<name>]`.
- The public dashboard resource group/version to implement first is `dashboard.grafana.app/v1`.
- List responses in the documented examples use `apiVersion: dashboard.grafana.app/v1alpha1` for the list wrapper/items even though the HTTP endpoint path is `/v1`; export code should preserve the returned envelope instead of rewriting `apiVersion`.
- Resource identity for the path is the dashboard response `metadata.name`. The docs explicitly distinguish this from `metadata.uid`, which is an internal identifier.
- Namespace mapping follows the API overview: OSS/on-prem default org is `default`, other orgs are `org-<org_id>`, and Grafana Cloud uses `stacks-<stack_id>`. The first implementation should default to `default` for single-org exports and map all-org export scopes to `org-<org_id>` unless a user-supplied namespace is added later.
- Folder identity is carried as metadata annotation `grafana.app/folder` when present.
- UI import remains classic-dashboard oriented for this slice: the documented UI accepts a dashboard JSON file, a Grafana.com dashboard URL/ID, or pasted dashboard JSON text. Do not document resource JSON/YAML as UI-import compatible.
- Existing `dashboard.grafana.app/v2` fixtures remain useful as classic-lane rejection fixtures, but v2 export is postponed until a public v2 HTTP endpoint is verified.

## Proposed CLI Surface

Keep the existing classic flags and add one explicit selector:

```text
grafana-util dashboard export \
  --profile prod \
  --output-dir ./dashboards \
  --resource-format v2 \
  --overwrite
```

Allowed values:

- `none`: default; write only existing classic lanes.
- `v1`: write `resource-v1/` in addition to any enabled classic lanes.
- `v2`: write `resource-v2/` in addition to any enabled classic lanes.
- `all`: write both resource lanes when Grafana supports both.

Resource lanes should be independent from `--without-raw`, `--without-prompt`, and `--without-provisioning`. A command with all classic lanes disabled plus one resource format selected is valid:

```text
grafana-util dashboard export \
  --profile prod \
  --without-raw \
  --without-prompt \
  --without-provisioning \
  --resource-format v2 \
  --output-dir ./dashboards
```

Add an explicit layout selector only if implementation needs to preserve both old and new directory shapes:

```text
--artifact-layout legacy|family
```

Rules:

- `legacy`: current default when `--resource-format none`; writes `raw/`, `prompt/`, and `provisioning/` directly under the output root.
- `family`: required when any resource format is requested; writes `classic/`, `resource-v1/`, and/or `resource-v2/`.
- If the user requests `--resource-format v1|v2|all` with `--artifact-layout legacy`, fail early with a clear error because legacy root layout cannot separate classic and resource formats safely.

## Artifact Layout

Single-org export:

```text
dashboards/
  classic/
    raw/
    prompt/
    provisioning/
    history/
    index.json
    export-metadata.json
  resource-v1/
    objects/
      <folder-path>/<metadata.name>.json
    index.json
    export-metadata.json
  resource-v2/
    objects/
      <folder-path>/<metadata.name>.json
    index.json
    export-metadata.json
  index.json
  export-metadata.json
```

All-org export:

```text
dashboards/
  org_1_Main_Org/
    classic/
      raw/
      prompt/
      provisioning/
    resource-v2/
      objects/
  org_2_Ops/
    classic/
      raw/
      prompt/
      provisioning/
    resource-v2/
      objects/
  index.json
  export-metadata.json
```

Resource object files should be named from `metadata.name`, not `metadata.uid`, because Grafana's dashboard HTTP API says the path `:uid` corresponds to the dashboard response `metadata.name`, not the Kubernetes-style `metadata.uid` field.

Resource metadata:

```json
{
  "kind": "grafana-utils-dashboard-export-index",
  "schemaVersion": 1,
  "variant": "resource-v2",
  "format": "grafana-dashboard-resource-v2",
  "source": "live",
  "resourceApiVersion": "dashboard.grafana.app/v2",
  "serialization": "json",
  "uiImportCompatible": false
}
```

## File Map

- Modify `rust/src/commands/dashboard/cli_defs.rs`: add the export `--resource-format` value enum and default.
- Modify `rust/src/commands/dashboard/export.rs`: pass resource export settings through export orchestration.
- Modify `rust/src/commands/dashboard/export_scope.rs`: create resource output directories, fetch resource documents, write resource lane indexes, and include resource paths in root indexes.
- Create `rust/src/commands/dashboard/export_resource.rs`: resource-format enum helpers, endpoint resolution, resource document fetch/list helpers, artifact naming, and resource metadata builders.
- Modify `rust/src/commands/dashboard/export_paths.rs`: support `classic/` family layout plus `resource-v1/objects` and `resource-v2/objects` path builders.
- Modify `rust/src/grafana/api/dashboard.rs`: add dashboard resource API methods only after endpoint verification.
- Modify `rust/src/commands/resource/cli_defs.rs`: align dashboard `resource describe` endpoint text once dashboard resource APIs are supported.
- Modify `rust/src/commands/resource/runtime.rs`: route dashboard `ResourceApiMode::Auto` to resource API first with classic fallback.
- Modify `rust/src/commands/dashboard/models.rs`: extend dashboard export index item or add a resource lane index item type.
- Modify `tests/fixtures/dashboard_grafana_source_parity_cases.json`: add v1/v2 native resource export examples from verified Grafana 13 payloads.
- Add tests under `rust/src/commands/dashboard/*resource*_rust_tests.rs` for CLI parsing, endpoint routing, artifact layout, metadata, and classic rejection preservation.
- Modify `scripts/contracts/command-surface.json` if `dashboard export --help-full` changes.
- Modify `docs/commands/en/dashboard-export.md` and `docs/commands/zh-TW/dashboard-export.md`.
- Modify `docs/user-guide/en/dashboard.md` and `docs/user-guide/zh-TW/dashboard.md` if the handbook needs the classic-vs-resource distinction.
- Regenerate `docs/man/*.1` and `docs/html/` with `make man` and `make html` when docs are updated.

## Execution Plan

### Task 1: Verify Grafana 13 Resource API Contract

**Files:**
- Modify: `docs/todo/2026-06-26-dashboard-resource-api-export.md`
- Create or update if needed: `tests/fixtures/dashboard_grafana_source_parity_cases.json`

- [x] Capture primary-source endpoint evidence for dashboard resource list/get paths, supported versions, namespace behavior, and identity field.
- [x] Record the confirmed endpoint table in this plan under a new `Verified API Contract` section.
- [x] Add one compact v1-family resource fixture and one compact v2 resource fixture with real envelope fields.
- [x] Record that Grafana UI import is classic dashboard JSON only for this slice, while `/apis` resource objects are API-management artifacts.
- [x] Record whether v2 is public HTTP API, internal schema, or both in the target Grafana 13 build.
- [x] Keep existing unsupported classic fixture cases in place.

Acceptance:

- Endpoint paths are backed by primary docs or a live Grafana 13 response transcript.
- Fixtures include `apiVersion`, `kind`, `metadata`, and `spec`.
- The plan explicitly states whether `resource-v2/` is enabled, experimental, or postponed based on verified endpoint availability.
- No production Rust code is changed before the endpoint table is verified.

Verification evidence:

- `jq empty tests/fixtures/dashboard_grafana_source_parity_cases.json` passed on 2026-06-27.
- `cd rust && cargo test --quiet dashboard_v2_resource` passed on 2026-06-27 with 13 tests.
- `cd rust && cargo test --quiet v2_resource` passed on 2026-06-27 with 13 tests.
- `cd rust && cargo test --quiet validate_dashboard_export_surfaces_dashboard_v2_resource_as_warning` passed on 2026-06-27 with 1 test.

### Task 2: Add CLI Surface and Parser Tests

**Files:**
- Modify: `rust/src/commands/dashboard/cli_defs.rs`
- Modify: `rust/src/cli/tests/parser_surface_rust_tests.rs`
- Modify: `scripts/contracts/command-surface.json`

- [x] Add `DashboardResourceFormat` with `none`, `v1`, `v2`, and `all`.
- [x] Add `DashboardArtifactLayout` with `legacy` and `family` only if the code cannot infer the layout from resource format safely. Not added in this slice; layout remains inferred from `resource_format`.
- [x] Add `resource_format: DashboardResourceFormat` to dashboard export args with default `none`.
- [x] If `--artifact-layout` is added, default it to `legacy` for classic-only exports and require `family` for resource-enabled exports. Not applicable because `--artifact-layout` was not added.
- [x] Add parser tests for default export, `--resource-format v2`, and resource-only export with all classic lanes disabled.
- [x] Add validation tests that reject resource-enabled legacy layout. No `--artifact-layout legacy|family` surface was added; family layout is inferred when `--resource-format v1` is selected.
- [x] Update the command-surface contract so docs validation knows the new flag.

Focused validation:

```bash
cd rust && cargo test --quiet parser_surface_rust_tests dashboard_export
```

Acceptance:

- Existing export invocations parse identically.
- `--resource-format none` is equivalent to omitting the flag.
- Resource-enabled exports cannot write ambiguous mixed root directories; until family-layout resource lanes exist, `--resource-format v1|v2|all` fails before any export requests or writes.
- Invalid resource format values fail through clap.

Verification evidence:

- `cd rust && cargo test --quiet export_dashboards_rejects_resource_format_before_resource_lanes_exist` passed on 2026-06-27 with 1 test before Task 4 wiring; superseded by the v1 resource-lane export test below.
- `cd rust && cargo test --quiet parse_dashboard_export_` passed on 2026-06-27 with 4 tests.
- `cd rust && cargo test --quiet parser_surface_rust_tests` passed on 2026-06-27 with 23 tests.
- `cd rust && cargo test --quiet dashboard_export` passed on 2026-06-27 with 63 tests.
- `cd rust && cargo fmt --all --check` passed on 2026-06-27.
- `python3 -m json.tool scripts/contracts/command-surface.json >/dev/null` passed on 2026-06-27.
- `make quality-docs-surface` passed on 2026-06-27.

### Task 3: Add Dashboard Resource API Client Boundary

**Files:**
- Modify: `rust/src/grafana/api/dashboard.rs`
- Modify: `rust/src/grafana/api/tests.rs`
- Create: `rust/src/commands/dashboard/export_resource.rs`

- [x] Add a small `DashboardResourceApiVersion` enum or reuse the CLI enum internally without leaking clap types into API code.
- [x] Add list/get helpers for verified resource endpoints.
- [x] Keep resource helpers returning raw `serde_json::Value` or `Map<String, Value>` until the v1/v2 schema stabilizes enough for typed structs.
- [x] Add request-path tests for v1 list/get using `/apis/dashboard.grafana.app/v1/namespaces/:namespace/dashboards` and `/apis/dashboard.grafana.app/v1/namespaces/:namespace/dashboards/:name`.
- [x] Add v2 request-path tests only after public v2 HTTP endpoints are verified; otherwise add tests that v2 export reports unsupported in the target build.
- [x] Add graceful 404 behavior for unsupported resource versions so `--resource-format all` can skip absent versions with a clear warning.

Focused validation:

```bash
cd rust && cargo test --quiet grafana::api::tests dashboard_resource
```

Acceptance:

- Classic `fetch_dashboard` remains unchanged.
- Resource endpoint methods do not alter existing classic list/get tests.
- Unsupported resource API versions return actionable errors or skip warnings according to the chosen CLI mode.

Verification evidence:

- `cd rust && cargo test --quiet dashboard_resource_client_` passed on 2026-06-27 with 6 tests.
- `cd rust && cargo test --quiet dashboard_resource` passed on 2026-06-27 with 6 tests.
- `cd rust && cargo test --quiet grafana_api::tests` passed on 2026-06-27 with 22 tests.
- `cd rust && cargo test --quiet export_dashboards_rejects_resource_format_before_resource_lanes_exist` passed on 2026-06-27 with 1 test.
- `cd rust && cargo fmt --all --check` passed on 2026-06-27.

### Task 4: Write Resource Export Lanes

**Files:**
- Modify: `rust/src/commands/dashboard/export.rs`
- Modify: `rust/src/commands/dashboard/export_scope.rs`
- Modify: `rust/src/commands/dashboard/models.rs`
- Modify: `rust/src/commands/dashboard/export_root_bundle.rs`
- Modify: `rust/src/commands/dashboard/export_paths.rs`
- Test: `rust/src/commands/dashboard/dashboard_resource_export_rust_tests.rs`

- [x] Add `classic`, `resource-v1`, `resource-v2`, and `objects` export directory constants.
- [x] Add resource path builders that reuse folder paths when the resource payload exposes folder identity and otherwise fall back to flat UID/name paths.
- [x] Fetch v1 resource documents after classic summaries are known, matching by `metadata.name` / classic UID according to the verified contract.
- [x] Write v1 resource documents unchanged except for stable pretty JSON formatting.
- [x] Write v1 lane-specific `index.json` and `export-metadata.json`.
- [x] Include v1 resource lane pointers in the root `index.json` without changing existing raw/prompt/provisioning fields.
- [x] Move classic raw/prompt/provisioning under `classic/` only in family layout; leave the legacy layout untouched for classic-only default exports.
- [x] Do not write YAML resource files in this task.
- [ ] Add tests for classic-only default, v2 lane, resource-only lane, and all-org resource lane. Resource-only v1 is covered; classic-only compatibility remains covered by existing export tests; v2 and all-org resource lane tests remain pending.

Focused validation:

```bash
cd rust && cargo test --quiet dashboard_resource_export
```

Acceptance:

- Default export byte layout remains unchanged in existing tests.
- `--resource-format v1` writes `resource-v1/objects/` with native JSON envelopes.
- `--resource-format v2` writes `resource-v2/objects/` only when public v2 endpoints are verified.
- `--resource-format all` writes each supported lane and reports missing unsupported lanes clearly.
- Resource lanes are not consumed by classic import/diff paths.

Verification evidence:

- `cd rust && cargo test --quiet export_dashboards_writes_resource_v1_lane_without_classic_variants` passed on 2026-06-27 with 1 test.
- `cd rust && cargo test --quiet dashboard_resource` passed on 2026-06-27 with 6 tests.
- `cd rust && cargo test --quiet resource_format` passed on 2026-06-27 with 3 tests.
- `cd rust && cargo test --quiet` passed on 2026-06-27 with 1832 passed, 1 ignored Rust lib tests plus 7 and 30 binary/integration tests.

### Task 5: Keep Classic Rejection Boundaries Explicit

**Files:**
- Modify: `rust/src/commands/dashboard/import_loaded_source_rust_tests.rs`
- Modify: `rust/src/commands/dashboard/export_diff_rust_tests.rs`
- Modify: `rust/src/commands/dashboard/dashboard_plan_rust_tests.rs`
- Modify: `rust/src/commands/dashboard/raw_to_prompt/rust_tests.rs`
- Modify only if needed: `rust/src/commands/dashboard/files.rs`

- [ ] Add regression cases proving `resource-v1/` and `resource-v2/` artifacts are not accidentally accepted as classic raw/provisioning input.
- [ ] Keep the existing error message direction: export classic dashboard JSON before using classic import/plan/prompt paths.
- [ ] If a new source-loader layout detects `resource-v*`, make the error name the resource lane explicitly.
- [ ] Add source-loader tests that `classic/raw` is accepted in family layout and `resource-v*/objects` is rejected by classic import/diff/plan.

Focused validation:

```bash
cd rust && cargo test --quiet dashboard_v2 dashboard_resource
```

Acceptance:

- Classic import/diff/plan still reject dashboard resource envelopes.
- Resource export support does not imply resource import support.
- Error messages tell the operator which workflow is unsupported.

### Task 6: Align `resource` Command Dashboard Mode

**Files:**
- Modify: `rust/src/commands/resource/cli_defs.rs`
- Modify: `rust/src/commands/resource/runtime.rs`
- Modify: `rust/src/commands/resource/catalog.rs`
- Test: `rust/src/commands/resource/*`

- [ ] Update dashboard resource descriptions and endpoint reporting.
- [ ] Route `ResourceApiMode::Auto` to the verified dashboard resource API first when supported.
- [ ] Keep `ResourceApiMode::Legacy` on `/api/search` and `/api/dashboards/uid/{uid}`.
- [ ] Make `ResourceApiMode::Legacy` wording say "classic dashboard API" so operators do not confuse it with the resource family export layout.
- [ ] Add tests for auto success, auto 404 fallback, and forced legacy.

Focused validation:

```bash
cd rust && cargo test --quiet resource
```

Acceptance:

- `grafana-util resource describe dashboards` reports the resource endpoint and fallback clearly.
- Existing datasource/alert/org resource behavior is unchanged.
- Dashboard resource get/list can inspect v1/v2 native envelopes without using dashboard export.

### Task 7: Update Operator Docs and Generated References

**Files:**
- Modify: `docs/commands/en/dashboard-export.md`
- Modify: `docs/commands/zh-TW/dashboard-export.md`
- Modify: `docs/commands/en/resource.md`
- Modify: `docs/commands/zh-TW/resource.md`
- Regenerate: `docs/man/*.1`
- Regenerate: `docs/html/`

- [ ] Document classic lanes separately from dashboard resource lanes.
- [ ] Add examples for v2 export and resource-only export.
- [ ] Explain that Grafana UI import accepts dashboard JSON and is served by classic `prompt/`, not resource v1/v2 artifacts.
- [ ] Explain that resource v1/v2 artifacts are for `/apis` resource API compatibility and future API-managed workflows.
- [ ] Avoid YAML examples unless a verified Grafana workflow consumes them.
- [ ] State that resource import/apply is not implemented in this slice.
- [ ] Regenerate man/html instead of hand-editing generated output.

Focused validation:

```bash
make quality-docs-surface
make man-check
make html-check
```

Acceptance:

- English and zh-TW command docs match.
- Help-full contract and generated docs are fresh.
- Docs do not imply resource import support.
- Docs do not imply YAML support in the Grafana UI import page.

### Task 8: Run Broader Validation and Commit

**Files:**
- All touched files from Tasks 1-7.

- [ ] Run focused tests from each task first.
- [ ] Run broader Rust tests:

```bash
cd rust && cargo test --quiet dashboard
cd rust && cargo test --quiet resource
cd rust && cargo fmt --all --check
```

- [ ] Run docs gates:

```bash
make quality-docs-surface
make man-check
make html-check
```

- [ ] Run final whitespace check:

```bash
git diff --check
```

- [ ] Commit with a grouped feature message:

```text
feature: add dashboard resource export lanes

- add explicit dashboard resource export format selection
- write native v1/v2 resource artifacts separately from classic lanes
- keep classic import and diff rejection boundaries intact
- update command docs and generated references
```

Acceptance:

- Focused tests pass.
- Broader dashboard/resource tests pass.
- Docs gates pass.
- Commit contains only this feature and its tests/docs.

## Risk Notes

- The highest risk is guessing Grafana 13 endpoint details. Do not code endpoint constants until Task 1 records verified paths.
- Resource v1 and v2 may not have enough structural overlap for a single typed model; use raw JSON envelopes first.
- Folder identity may differ between classic summaries and resource metadata. If mapping is ambiguous, write resource artifacts flat and record folder mapping as metadata instead of inventing hierarchy.
- v2 may be a schema/conversion version without a public HTTP endpoint in a given Grafana release. In that case, keep `resource-v2/` planned but disabled with a clear unsupported message.
- YAML may be valid as a Go/Kubernetes-style serialization for typed resources, but that does not make it Grafana UI-import compatible. Keep JSON canonical until a consumer is verified.
- Import/apply semantics are materially riskier than export; keep them out of this slice.

## Acceptance Checklist

- [ ] Default dashboard export remains classic-only and backwards compatible.
- [ ] Resource-enabled exports use family layout: `classic/`, `resource-v1/`, and/or `resource-v2/`.
- [ ] `--resource-format v1`, `v2`, and `all` are documented and tested.
- [ ] Native resource envelopes are written as JSON to `resource-v1/objects/` and/or `resource-v2/objects/`.
- [ ] Existing classic lanes still reject dashboard resource envelopes.
- [ ] `resource describe dashboards` reflects the new dashboard resource API support.
- [ ] Docs explain that UI import belongs to classic dashboard JSON, not resource YAML.
- [ ] Command docs, generated man pages, and generated HTML are synchronized.
