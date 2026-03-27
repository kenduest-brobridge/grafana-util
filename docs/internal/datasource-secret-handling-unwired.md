# Datasource Secret Handling (Unwired)

## Purpose

This note tracks the isolated placeholder-based datasource secret scaffold
before fuller CLI, datasource, or provider wiring lands.

## Scope

- New Python helper module:
  - `grafana_utils/datasource_secret_workbench.py`
- New unit tests:
  - `python/tests/test_python_datasource_secret_workbench.py`

## Current Behavior

- Rust status:
  - Rust now has staged `${secret:...}` placeholder parsing, bundle-preflight
    blocking, and explicit placeholder resolution helpers.
  - Rust still does not wire those resolved secrets into datasource import or
    live mutation workflows.

- `collect_secret_placeholders(...)`
  - Accepts only `${secret:...}` placeholder strings inside
    `secureJsonDataPlaceholders`.
  - Rejects raw secret values or non-string objects so secret-bearing payloads
    cannot be replayed opaquely.
- `resolve_secret_placeholders(...)`
  - Resolves placeholders only from an explicit in-memory mapping.
  - Fails closed when a placeholder is missing or resolves to an empty value.
- `build_datasource_secret_plan(...)`
  - Produces one review-required plan object with resolved `secureJsonData`.
  - Keeps provider behavior explicit as `inline-placeholder-map`.

## Not Yet Wired

- No argparse or unified CLI integration yet.
- No datasource import/live-mutation workflow integration yet.
- No external secret provider support yet.
- Rust resolution is still contract-only and not yet wired into datasource
  runtime workflows.

## Future Wire Points

- `grafana_utils/datasource/parser.py`
- `grafana_utils/datasource/workflows.py`
- `rust/src/datasource.rs`
