# Developer Notes

This document is for maintainers. Keep `README.md` GitHub-facing and operator-oriented; keep implementation detail, release ritual, and maintenance notes here.

## Documentation Contract

- Keep `README.md` and `README.zh-TW.md` focused on the current Rust `grafana-util` operator surface.
- Keep `docs/user-guide.md` and `docs/user-guide-TW.md` aligned on command names, option naming, and examples.
- Prefer `--output-format` guidance over legacy `--table` / `--csv` / `--json` wording when the command supports both.
- Remove stale references to retired entrypoints whenever command docs are touched.

## Repository Scope

- `rust/src/cli.rs`: unified Rust entrypoint for namespaced command dispatch and `--help-full`.
- `rust/src/dashboard/`: dashboard export, import, diff, inspect, prompt-export, and screenshot workflows.
- `rust/src/datasource.rs`: datasource list, export, import, diff, add, modify, and delete workflows.
- `rust/src/alert.rs`: alerting export, import, diff, and shared alert document helpers.
- `rust/src/alert_list.rs`: alert list rendering and list command orchestration.
- `rust/src/access/`: access org, user, team, and service-account workflows plus shared renderers and request helpers.
- `rust/src/sync/`: staged sync bundle, preflight, review, and apply flows.
- `rust/src/*_rust_tests.rs`: Rust regression and contract coverage.
- `Makefile`: maintainer shortcuts for build, test, lint, and version bump flows.
- `.github/workflows/ci.yml`: CI entrypoint that should stay aligned with local quality gates.
- `scripts/check-rust-quality.sh`: centralized Rust test / fmt / clippy gate used locally and in CI.
- `scripts/set-version.sh`: shared version bump helper for `VERSION`, `rust/Cargo.toml`, and `rust/Cargo.lock`.
- `docs/overview-rust.md`: deeper Rust architecture walkthrough.

## Version Workflow

- `dev` is the preview branch; `main` is the release branch.
- `VERSION` is the checked-in maintainer version source.
- Use `make print-version` to inspect the current checked-in version state.
- Use `make sync-version` after editing `VERSION` manually.
- Use `make set-release-version VERSION=X.Y.Z` when preparing `main` for release.
- Use `make set-dev-version VERSION=X.Y.Z DEV_ITERATION=N` when moving `dev` to the next preview cycle.
- Preferred release ritual:
  - work on `dev`
  - merge `dev` into `main`
  - run `make set-release-version VERSION=X.Y.Z` on `main`
  - run `make test`
  - create tag `vX.Y.Z`
  - merge `main` back into `dev`
  - run `make set-dev-version VERSION=X.Y.$((Z+1)) DEV_ITERATION=1` or the intended next preview
- Treat the post-release `main -> dev` sync as required so CI, docs, scripts, and version metadata do not drift.

## CLI Boundaries

- `grafana-util` is the maintained operator entrypoint.
- Keep namespaced commands stable: `grafana-util dashboard ...`, `grafana-util datasource ...`, `grafana-util alert ...`, `grafana-util access ...`, `grafana-util sync ...`.
- `dashboard list-data-sources` remains compatibility-only; new docs and examples should prefer `datasource list`.
- `inspect-export`, `inspect-live`, and `--help-full` are part of the supported Rust operator surface and should be documented consistently.

## Quality Gates

- `make quality-rust` is the baseline Rust quality gate.
- `scripts/check-rust-quality.sh` must stay the single source of truth for local and CI Rust validation.
- `cargo clippy --all-targets -- -D warnings` is release-blocking in CI; treat new warnings as failures.
- When docs claim an example was validated, prefer commands exercised against the local Docker Grafana smoke paths.

## Build Notes

- `make build-rust` builds release artifacts and cross-build outputs used for published binaries.
- `make build-rust-native` is the local-host-only release build.
- `make build-rust-macos-arm64` and `make build-rust-linux-amd64` are the platform-specific release helpers.
- Keep release artifact names and docs aligned with the current Rust-only distribution model.

## Maintenance Rules

- Keep README and user guides Rust-only unless a maintained non-Rust distribution is intentionally reintroduced.
- If a workflow change affects operator behavior, update both user guides in the same change.
- If a release or versioning rule changes, update this file in the same change.
- Historical notes in `docs/internal/` are archival; do not treat them as current operator docs.
