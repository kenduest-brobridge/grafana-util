# ai-changes-archive-2026-04-15

## 2026-04-13 - Add shell completion command
- Summary: added `grafana-util completion bash|zsh`, implemented completion rendering through `clap_complete` from the unified Clap command tree, and routed the command through the existing CLI dispatch spine without entering Grafana runtime/auth paths.
- Tests: added parser coverage for Bash/Zsh and unsupported shell rejection, plus render coverage that completion scripts include common root commands from the unified CLI tree.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all`; `cargo test --manifest-path rust/Cargo.toml --quiet completion -- --test-threads=1`; `make man`; `make html`.
- Impact: `rust/src/cli/mod.rs`, `rust/src/cli/dispatch.rs`, new `rust/src/cli/completion.rs`, Rust CLI tests, `rust/Cargo.toml`, `rust/Cargo.lock`, README files, command docs/contracts, generated `docs/man/`, generated `docs/html/`, and AI trace docs.
- Rollback/Risk: low. Completion generation is read-only and Clap-backed; rollback removes the root `completion` command, dependency, docs, and generated completion man/html pages.
- Follow-up: none.

## 2026-04-13 - Add GitHub installer completion option
- Summary: extended `scripts/install.sh` with opt-in `INSTALL_COMPLETION=auto|bash|zsh` support and `--interactive` prompts, writing completion from the just-installed binary to the standard Bash or Zsh per-user location unless `COMPLETION_DIR` is set. Refactored the installer into named helper stages for dependency checks, platform/archive resolution, interactive choices, installation, completion, and PATH notices. Added a GitHub-free local installer smoke script and `make test-installer-local`. Updated GitHub install examples so environment overrides and `--interactive` are attached to the `sh` side of the pipe, which is the process that reads the installer.
- Tests: added installer coverage for auto-detected Zsh completion generation from a local archive fixture, interactive install-directory/completion prompts through a test terminal file, plus help/contract assertions for the new environment variables and option.
- Test Run: `sh -n scripts/install.sh`; `sh -n scripts/install.sh scripts/test-local-installer.sh`; `python3 -m unittest -v python.tests.test_python_install_script`; `make test-installer-local`; `COMPLETION_SHELL=bash make test-installer-local`; `make html`.
- Impact: `scripts/install.sh`, new `scripts/test-local-installer.sh`, `Makefile`, `python/tests/test_python_install_script.py`, `README.md`, `README.zh-TW.md`, `docs/user-guide/{en,zh-TW}/getting-started.md`, `docs/commands/{en,zh-TW}/completion.md`, `docs/internal/maintainer-quickstart.md`, generated `docs/html/`, and AI trace docs.
- Rollback/Risk: low. The installer remains binary-only by default; completion is written only when requested or confirmed in interactive mode. Rollback removes the installer env vars/option and restores manual-only completion setup.
- Follow-up: none.
