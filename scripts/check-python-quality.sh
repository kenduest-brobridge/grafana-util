#!/usr/bin/env bash

set -u

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PYTHON_BIN="${PYTHON:-python3}"
STATUS=0
PYTHON_QUALITY_PATHS=()

log() {
  printf '%s\n' "$*"
}

warn() {
  printf 'warning: %s\n' "$*" >&2
}

run_step() {
  local label="$1"
  shift

  log "==> ${label}"
  if ! "$@"; then
    STATUS=1
  fi
}

run_optional_cmd() {
  local tool_name="$1"
  local label="$2"
  shift 2

  if ! command -v "$tool_name" >/dev/null 2>&1; then
    warn "skipping ${label}; ${tool_name} is not installed"
    return 0
  fi

  run_step "$label" "$@"
}

run_optional_python_module() {
  local module_name="$1"
  local label="$2"
  shift 2

  if ! "$PYTHON_BIN" -c "import ${module_name}" >/dev/null 2>&1; then
    warn "skipping ${label}; python module ${module_name} is not installed"
    return 0
  fi

  run_step "$label" "$PYTHON_BIN" -m "$module_name" "$@"
}

collect_python_quality_paths() {
  local path

  for path in grafana_utils tests python; do
    if [ -e "$path" ]; then
      PYTHON_QUALITY_PATHS+=("$path")
    fi
  done
}

cd "$ROOT_DIR" || exit 1
collect_python_quality_paths

if [ "${#PYTHON_QUALITY_PATHS[@]}" -eq 0 ]; then
  warn "no Python quality paths were found"
  exit 1
fi

run_step "python bytecode compile check" \
  "$PYTHON_BIN" -m compileall -q "${PYTHON_QUALITY_PATHS[@]}"

run_step "python unittest suite" \
  "$PYTHON_BIN" -m unittest -v

run_optional_python_module ruff "ruff lint" \
  check "${PYTHON_QUALITY_PATHS[@]}"

run_optional_python_module mypy "mypy type check" \
  "${PYTHON_QUALITY_PATHS[@]}"

run_optional_python_module black "black format check" \
  --check "${PYTHON_QUALITY_PATHS[@]}"

if [ "$STATUS" -ne 0 ]; then
  warn "python quality checks finished with failures"
fi

exit "$STATUS"
