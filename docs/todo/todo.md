# Todo

## Active

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
