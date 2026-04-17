# dashboard prompt external export semantics

## Purpose
Define how `dashboard convert raw-to-prompt` should mirror Grafana's external UI export flow for dashboard import handoff.

## Official Reference
- Grafana dashboard sharing docs describe the UI external export flow as the dashboard JSON shape used when a dashboard may be imported with different datasource UIDs.
- Grafana's frontend `DashboardExporter.ts` is the behavior source for `Export for sharing externally`; it templateizes existing datasource references into `__inputs`, but does not synthesize new datasource variables.
- Grafana's scene exporter keeps datasource variable references in panels and targets, but can map the datasource variable's current concrete datasource through a `DS_*` input.
- The `__inputs` shape is part of the UI import/export handoff, not the Dashboard HTTP API payload contract.

## Contract
- Prompt output maps to Grafana's UI `Upload JSON` external-export shape, not to the raw dashboard API payload.
- Concrete datasource references in panels, targets, query variables, and annotations become `__inputs` entries with `${DS_*}` placeholders.
- Built-in Grafana datasource selectors are excluded from `__inputs`.
- A datasource variable with `type: datasource` keeps its own variable shape; its `query` is treated as a plugin filter, not as an import input.
- The converter must not synthesize a new datasource variable when the source dashboard did not already have one.
- A panel or target datasource reference such as `$datasource` stays variable-based. When that variable is used and has a concrete current datasource value, the variable `current` may point at the generated `${DS_*}` input so Grafana import can ask for the replacement datasource without rewriting panel references.
- Constant variables become `VAR_*` import inputs. Their `query`, `current`, and `options` point at the generated placeholder.
- Expression and built-in Grafana datasource selectors are not user-mapped datasource inputs.
- Multiple distinct concrete datasource references stay as separate prompt inputs when Grafana would ask for them separately.
- Library panel references are preserved. Local raw-to-prompt conversion does not fetch and inline missing library panel models; emit a portability warning instead of inventing `__elements`. Live export/import-handoff parity may look up the referenced model later, but that stays outside the offline converter path.

## Known Issue Pattern
- Some older prompt exports over-normalized a single datasource family into a synthetic `datasource` variable.
- That can also produce two import inputs when the source dashboard already had a datasource variable and the converter also treated the variable query as a separate input.
- The maintainer rule is to prefer the external-export contract above, not the synthetic variable shortcut.

## Later Parity
- Full library panel external-export parity requires live library panel model lookup and is intentionally left for later.
- Dashboard v2 resources use `grafana.app/export-label`, not classic `__inputs`. Keep v2 as a separate adapter boundary and do not mix it into the classic prompt lane.
- Dashboard permissions are outside prompt export semantics. Keep them as adjacent access evidence instead of trying to encode them in dashboard JSON.

## Notes
- Keep this file as the maintainer reference for prompt-lane datasource semantics.
- This spec is only about prompt export shape and placeholder mapping; it does not redefine live import, provisioning, or raw export behavior.
