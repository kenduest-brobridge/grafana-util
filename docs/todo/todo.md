# Todo

## Active

- [x] [Dashboard clone-folder](2026-07-07-dashboard-clone-folder.md) -
  Add `grafana-util dashboard clone-folder` to duplicate dashboards from one
  live Grafana folder into another, with deterministic copied UIDs, optional
  target folder creation, recursive subtree support, dry-run output, and a
  `--yes` gate for live writes.
- [ ] [Grafana 13 dashboard resource export](2026-06-26-dashboard-resource-api-export.md) -
  Plan support for a family-oriented export layout: `classic/` contains
  raw/prompt/provisioning/history, while native `dashboard.grafana.app` v1
  resource API objects live under `resource-v1/objects`; keep `resource-v2/`
  postponed until a public v2 HTTP endpoint is verified, with UI-import
  compatibility kept classic-only.
- [x] [Rust maintainability refactor backlog](2026-06-25-rust-maintainability-refactor-backlog.md) -
  Track necessary larger refactor candidates from the current architecture
  audit. Active order: close confirmed lint cleanup; plan and split
  `review_contract.rs`; plan and split `common/browser/session.rs`; plan and
  split `datasource/browse/render.rs`; inventory dashboard/access/sync before
  any broader domain refactor.
- [x] [Domain maintainability inventory](2026-06-25-domain-maintainability-inventory.md) -
  Document dashboard/access/sync hotspot evidence and defer broader refactors
  until a narrower behavior or ownership trigger appears.
- [x] [Review contract split](2026-06-25-review-contract-split.md) -
  Split the shared review contract into actions, model, detail, and envelope
  modules while keeping the existing `crate::review_contract` facade stable.
- [x] [Browser session split](2026-06-25-browser-session-split.md) -
  Split the shared interactive browser session into model, search, render, and
  runtime modules while keeping `crate::interactive_browser` stable.
- [x] [Datasource browse render split](2026-06-25-datasource-browse-render-split.md) -
  Split datasource browse rendering into summary, list, and detail helper
  modules while keeping the frame renderer entrypoint stable.
- [x] [Project maintainability audit](2026-06-25-project-maintainability-audit.md) -
  Assess whether the Rust-first project structure remains maintainable, with
  emphasis on over-decomposition, architecture boundaries, Rust/Python
  ownership, CLI/docs coupling, and test coverage risk. Current audit finds
  the project still within maintainable range, with a concrete lint blocker in
  dashboard export cleanup and ongoing hotspot risk in dashboard/access/sync
  domain size.
- [x] [Doc and CLI parameter sync audit](2026-06-24-doc-cli-parameter-sync-audit.md) -
  Check whether Markdown docs, handbook/manual content, generated references,
  and Rust-supported CLI parameters are synchronized. Confirmed generated
  man/html drift plus missing `datasource list --local/--run/--run-id` source
  docs; source docs were updated and generated man/html checks now pass.
- [x] [Dashboard import and inspect test failures](2026-06-23-dashboard-import-inspect-test-failures.md) -
  Reproduce and fix the dashboard import/inspect failures seen under raw
  `make test`, without touching unrelated HTTP dotted-host work. Focused
  reruns and raw `make test` now pass; failures did not reproduce.
- [x] [HTTP dotted host URL fix](2026-06-23-http-dotted-host-url.md) - Fix Rust
  HTTP URL classification so DNS-shaped hosts such as `hostx.0.2.120` report an
  unknown-host style error instead of an invalid URL error. Review follow-up
  also covers numeric-suffix DNS names such as `grafana.prod.1`.
- [x] [Dashboard export history performance](2026-06-24-dashboard-export-history-performance.md) -
  Reuse the already-fetched current dashboard payload for `dashboard export
  --include-history` and add bounded parallel history-version fetching for
  client-backed exports. Focused history/export tests now pass.
