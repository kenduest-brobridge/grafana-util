# Agent Guide

This file is the short repo-level guide for coding agents. Keep it concise. Put detailed maintainer notes in [docs/DEVELOPER.md](docs/DEVELOPER.md).

## Core Rules

- Treat `dev` as the preview branch and `main` as the release branch.
- On `dev`, keep versions in dev form:
  - Python in `pyproject.toml`: `X.Y.Z.devN`
  - Rust in `rust/Cargo.toml`: `X.Y.Z-dev.N`
- On `main`, keep both package versions in plain release form `X.Y.Z`.
- Create formal release tags as `vX.Y.Z` from `main` only, and keep the tag aligned with both package versions.
- Update Python and Rust package metadata together whenever the version changes.
- Prefer the unified CLI shape in docs and examples:
  - `grafana-util dashboard ...`
  - `grafana-util alert ...`
  - `grafana-util datasource ...`
  - `grafana-util access ...`
- Use `apply_patch` for manual file edits.
- Keep commit messages in this format:
  - first line with a type prefix such as `feature:`, `bugfix:`, `docs:`, `test:`, or `refactor:`
  - blank line
  - 2-4 flat `- ...` detail bullets

## Working Defaults

- Keep external operator-facing usage in `README.md`, `README.zh-TW.md`, `docs/user-guide.md`, and `docs/user-guide-TW.md`.
- Keep internal rationale and maintenance workflow in [docs/DEVELOPER.md](docs/DEVELOPER.md).
- Update tests for user-visible CLI behavior changes, especially parser or help-text changes.
- Use Poetry-first Python commands for development and testing when possible.

## Release Flow

1. Work on `dev` and push preview versions there.
2. Promote `dev` into `main` when the release candidate is ready.
3. Change both package versions on `main` to `X.Y.Z`.
4. Push `main`.
5. Tag that exact commit as `vX.Y.Z` and push the tag.

## Further Reading

- Maintainer workflow and detailed release notes: [docs/DEVELOPER.md](docs/DEVELOPER.md)
- Operator usage: [README.md](README.md), [README.zh-TW.md](README.zh-TW.md)
- Full command guides: [docs/user-guide.md](docs/user-guide.md), [docs/user-guide-TW.md](docs/user-guide-TW.md)
