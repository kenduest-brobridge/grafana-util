# ai-status-archive-2026-04-13

## 2026-04-12 - Infer unique long option prefixes
- State: Done
- Scope: `rust/src/cli.rs`, `rust/src/access/cli_defs.rs`, CLI parser tests, and AI trace docs.
- Baseline: unique-prefix matching worked for subcommands, but long options such as `--all-o` only produced a suggestion for `--all-orgs` instead of resolving the unique match.
- Current Update: enabled Clap unique long-argument inference on the unified root parser and access parser, with tests for inferred unique prefixes and rejected ambiguous prefixes.
- Result: `grafana-util access user list --all-o --tab` now parses as `--all-orgs --table`; ambiguous or invalid long prefixes still stay on Clap's error path.
