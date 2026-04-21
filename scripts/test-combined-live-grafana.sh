#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUST_LIVE_SMOKE="${ROOT_DIR}/scripts/test-rust-live-grafana.sh"
PYTHON_LIVE_SMOKE="${ROOT_DIR}/scripts/test-python-live-grafana.sh"

run_step() {
  local label="$1"
  shift

  printf '==> %s\n' "$label"
  "$@"
}

main() {
  if [[ $# -ne 0 ]]; then
    printf 'ERROR: %s does not accept arguments; configure the underlying smoke scripts with environment variables.\n' "${BASH_SOURCE[0]##*/}" >&2
    exit 1
  fi

  run_step "Rust live Grafana smoke" bash "${RUST_LIVE_SMOKE}"
  run_step "Python live Grafana smoke" bash "${PYTHON_LIVE_SMOKE}"

  printf 'Combined live Grafana smoke tests passed.\n'
}

main "$@"
