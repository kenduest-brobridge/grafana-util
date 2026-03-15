# TODO Archive

This file preserves completed and superseded items that previously lived in `TODO.md`.

`TODO.md` should stay focused on active backlog and current constraints.

## Completed Access And CLI Shape Work

- unified primary CLI is now `grafana-utils`
- Python source-tree wrapper is now `python/grafana-utils.py`
- Python `grafana-access-utils` shim was removed
- Python and Rust both support access-management commands through `grafana-utils access ...`
- implemented access `user list`
- implemented access `user add`
- implemented access `user modify`
- implemented access `user delete`
- implemented access `team list`
- implemented access `team add`
- implemented access `team modify`
- implemented access `team delete`
- implemented access `service-account list`
- implemented access `service-account add`
- implemented access `service-account delete`
- implemented access `service-account token add`
- implemented access `service-account token delete`
- implemented access `group` alias for `team`
- added unit tests and Docker-backed live validation for the implemented access workflows
- dashboard CLI also includes `list-data-sources` in both Python and Rust, but that is outside the remaining access-management scope once tracked in `TODO.md`

### Access Command Shape

```text
grafana-utils access user list
grafana-utils access user add
grafana-utils access user modify
grafana-utils access user delete

grafana-utils access team list
grafana-utils access team add
grafana-utils access team modify
grafana-utils access team delete

grafana-utils access group list
grafana-utils access group add
grafana-utils access group modify
grafana-utils access group delete

grafana-utils access service-account list
grafana-utils access service-account add
grafana-utils access service-account delete
grafana-utils access service-account token add
grafana-utils access service-account token delete
```

### Access Notes

- `group` remains a compatibility alias for `team`
- Rust may still keep `grafana-access-utils` as a compatibility binary, but the primary command model is `grafana-utils access ...`
- Python should not reintroduce a separate `grafana-access-utils` wrapper or console script

## Completed Internal Refactors

- Rust dashboard internals were split into:
  - `dashboard_cli_defs.rs`
  - `dashboard_list.rs`
  - `dashboard_export.rs`
  - `dashboard_prompt.rs`
- Rust dashboard export metadata and index documents now use typed internal structs without changing JSON output shape
- Rust `access.rs` internals were split into:
  - `access_cli_defs.rs`
  - `access_user.rs`
  - `access_team.rs`
  - `access_service_account.rs`
  - `access_render.rs`
- Rust alert internals were split further across:
  - `alert_cli_defs.rs`
  - `alert_client.rs`
  - `alert_list.rs`
  - `alert.rs` orchestration helpers
