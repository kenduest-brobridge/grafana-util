#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PYTHON_ACCESS_LIVE_SMOKE="${ROOT_DIR}/scripts/test-python-access-live-grafana.sh"
PYTHON_DATASOURCE_LIVE_SMOKE="${ROOT_DIR}/scripts/test-python-datasource-live-grafana.sh"

run_step() {
  local label="$1"
  shift

  printf '==> %s\n' "${label}"
  "$@"
}

main() {
  if [[ $# -ne 0 ]]; then
    printf 'ERROR: %s does not accept arguments; configure the underlying smoke scripts with environment variables.\n' "${BASH_SOURCE[0]##*/}" >&2
    exit 1
  fi

  run_step "Python access live Grafana smoke" bash "${PYTHON_ACCESS_LIVE_SMOKE}"
  run_step "Python datasource live Grafana smoke" bash "${PYTHON_DATASOURCE_LIVE_SMOKE}"

  printf 'Python live Grafana smoke tests passed.\n'
}

main "$@"
