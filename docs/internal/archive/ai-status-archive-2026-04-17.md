# ai-status-archive-2026-04-17

## 2026-04-15 - Fix stale dashboard command references
- State: Done
- Scope: Public docs and maintainer guidance that still present removed dashboard command names or stale inspect-artifact wording. Python implementation is out of scope.
- Baseline: `grafana-util dashboard analyze` and `grafana-util dashboard inspect-export` are rejected by the CLI, but README and maintainer docs still mention them as command paths.
- Current Update: Replaced README dependency examples with `dashboard summary`, updated policy/about artifact wording to dashboard summary JSON artifacts, refreshed generated man/html docs, and clarified maintainer docs that `inspect` is an internal artifact flow rather than a public dashboard command.
- Result: CLI smoke checks confirm `dashboard analyze` and `dashboard inspect-export` are rejected while `dashboard summary` is accepted. Docs, generated-doc, Rust help, AI workflow, and whitespace checks pass.

## 2026-04-15 - Add Grafana instance metadata to status live
- State: Done
- Scope: Rust `status live` document assembly, focused status-live tests, operator docs, generated docs, and AI trace docs. Python implementation is out of scope.
- Baseline: `status live` returns project/domain readiness fields but does not read `/api/health`, so JSON/YAML output cannot show Grafana instance fields such as version, commit, or database health.
- Current Update: Added an additive `discovery.instance` section populated from `GET /api/health`; failed health reads are recorded as non-blocking discovery metadata instead of changing domain readiness.
- Result: Focused status tests, full Rust tests, clippy, docs surface, generated-doc checks, AI workflow, and whitespace checks pass. Source command/reference docs and generated man/html docs now describe the live instance metadata shape.
