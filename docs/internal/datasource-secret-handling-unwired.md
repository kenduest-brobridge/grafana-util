# Datasource Secret Handling

## Purpose

This note tracks the placeholder-based datasource secret contract for
datasource imports and live mutation payloads.

## Scope

- New Python helper module:
  - `grafana_utils/datasource_secret_workbench.py`
- New unit tests:
  - `python/tests/test_python_datasource_secret_workbench.py`

## Current Behavior

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

- Datasource import contract:
  - Import records can carry `secureJsonDataPlaceholders` in the exported
    datasource JSON.
  - Live import requires `--secret-values` when a record has placeholders.
  - `--secret-values` is a JSON object that maps placeholder names to resolved
    secret values before the payload is sent to Grafana.
  - The resolved values are written into `secureJsonData` on import so the
    placeholder contract stays explicit.

## Remaining Limits

- No external secret provider support yet.
- The contract is still placeholder-based; the CLI does not resolve secrets
  from a provider or secrets manager automatically.

## Future Wire Points

- `grafana_utils/datasource/parser.py`
- `grafana_utils/datasource/workflows.py`
- `rust/src/datasource.rs`
