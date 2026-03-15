#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
output_path="${repo_root}/docs/context/FILE_INDEX.json"

mkdir -p "$(dirname "${output_path}")"

if ! command -v rg >/dev/null 2>&1; then
  echo "error: rg is required to build docs/context/FILE_INDEX.json" >&2
  exit 1
fi

python3 - <<'PY' "$repo_root" "$output_path"
import json
import os
import subprocess
import sys
from datetime import datetime, timezone

repo_root = sys.argv[1]
output_path = sys.argv[2]

seed = {
    "AGENTS.md": {
        "purpose": "Repository rules and agent workflow constraints.",
        "depends_on": ["README.md", "docs/DEVELOPER.md"],
        "owned_tests": [],
        "tags": ["policy", "agents", "docs"],
    },
    "Makefile": {
        "purpose": "Build/test task runner for Python and Rust.",
        "depends_on": ["pyproject.toml", "rust/Cargo.toml"],
        "owned_tests": ["tests/test_python_packaging.py"],
        "tags": ["build", "test", "tooling"],
    },
    "README.md": {
        "purpose": "Operator-facing usage and quickstart documentation.",
        "depends_on": ["grafana_utils/unified_cli.py"],
        "owned_tests": ["tests/test_python_packaging.py"],
        "tags": ["docs", "user", "entrypoint"],
    },
    "docs/DEVELOPER.md": {
        "purpose": "Maintainer-focused implementation notes and tradeoffs.",
        "depends_on": ["grafana_utils/unified_cli.py", "rust/src/lib.rs"],
        "owned_tests": [],
        "tags": ["docs", "internal", "developer"],
    },
    "docs/context/PROJECT_MAP.md": {
        "purpose": "AI-first module map and read order.",
        "depends_on": ["AGENTS.md", "docs/context/CHANGE_IMPACT.md", "docs/context/FILE_INDEX.json"],
        "owned_tests": [],
        "tags": ["context", "ai", "docs"],
    },
    "docs/context/CHANGE_IMPACT.md": {
        "purpose": "Change-impact matrix for targeted follow-up edits and tests.",
        "depends_on": ["docs/context/PROJECT_MAP.md"],
        "owned_tests": [],
        "tags": ["context", "ai", "docs"],
    },
    "grafana_utils/__main__.py": {
        "purpose": "Python module entrypoint forwarding to unified CLI.",
        "depends_on": ["grafana_utils/unified_cli.py"],
        "owned_tests": ["tests/test_python_unified_cli.py"],
        "tags": ["python", "entrypoint"],
    },
    "grafana_utils/unified_cli.py": {
        "purpose": "Python unified command dispatcher.",
        "depends_on": [
            "grafana_utils/dashboard_cli.py",
            "grafana_utils/alert_cli.py",
            "grafana_utils/access_cli.py",
            "grafana_utils/datasource_cli.py",
            "grafana_utils/sync_cli.py",
        ],
        "owned_tests": ["tests/test_python_unified_cli.py"],
        "tags": ["python", "cli", "dispatcher"],
    },
    "grafana_utils/dashboard_cli.py": {
        "purpose": "Dashboard command parsing and orchestration.",
        "depends_on": ["grafana_utils/clients/dashboard_client.py", "grafana_utils/http_transport.py"],
        "owned_tests": ["tests/test_python_dashboard_cli.py", "tests/test_python_dashboard_integration_flow.py"],
        "tags": ["python", "dashboard", "cli"],
    },
    "grafana_utils/alert_cli.py": {
        "purpose": "Alert command parsing and orchestration.",
        "depends_on": ["grafana_utils/clients/alert_client.py", "grafana_utils/http_transport.py"],
        "owned_tests": ["tests/test_python_alert_cli.py"],
        "tags": ["python", "alert", "cli"],
    },
    "grafana_utils/access_cli.py": {
        "purpose": "Access command parsing and orchestration.",
        "depends_on": ["grafana_utils/access/parser.py", "grafana_utils/access/workflows.py", "grafana_utils/http_transport.py"],
        "owned_tests": ["tests/test_python_access_cli.py"],
        "tags": ["python", "access", "cli"],
    },
    "grafana_utils/access/parser.py": {
        "purpose": "Access argparse definitions.",
        "depends_on": ["grafana_utils/http_transport.py", "grafana_utils/access/pending_cli_staging.py"],
        "owned_tests": ["tests/test_python_access_cli.py", "tests/test_python_access_pending_cli_staging.py"],
        "tags": ["python", "access", "parser"],
    },
    "grafana_utils/access/workflows.py": {
        "purpose": "Access workflows for users/orgs/teams/service accounts.",
        "depends_on": ["grafana_utils/clients/access_client.py", "grafana_utils/access/models.py"],
        "owned_tests": ["tests/test_python_access_cli.py"],
        "tags": ["python", "access", "workflow"],
    },
    "grafana_utils/datasource_cli.py": {
        "purpose": "Datasource command parsing and workflow dispatch.",
        "depends_on": ["grafana_utils/datasource/parser.py", "grafana_utils/datasource/workflows.py"],
        "owned_tests": ["tests/test_python_datasource_cli.py"],
        "tags": ["python", "datasource", "cli"],
    },
    "grafana_utils/sync_cli.py": {
        "purpose": "Sync planning/review/preflight/apply orchestration.",
        "depends_on": ["grafana_utils/gitops_sync.py", "grafana_utils/sync_preflight_workbench.py"],
        "owned_tests": ["tests/test_python_sync_cli.py", "tests/test_python_gitops_sync.py"],
        "tags": ["python", "sync", "cli"],
    },
    "grafana_utils/http_transport.py": {
        "purpose": "Shared HTTP transport abstraction used by all clients.",
        "depends_on": [
            "grafana_utils/clients/dashboard_client.py",
            "grafana_utils/clients/alert_client.py",
            "grafana_utils/clients/access_client.py",
            "grafana_utils/clients/datasource_client.py",
        ],
        "owned_tests": ["tests/test_python_datasource_client.py"],
        "tags": ["python", "http", "shared"],
    },
    "pyproject.toml": {
        "purpose": "Python package metadata and console-script entrypoints.",
        "depends_on": ["grafana_utils/unified_cli.py", "grafana_utils/__main__.py"],
        "owned_tests": ["tests/test_python_packaging.py"],
        "tags": ["packaging", "python"],
    },
    "rust/Cargo.toml": {
        "purpose": "Rust package metadata and release version.",
        "depends_on": ["rust/src/lib.rs", "rust/src/bin/grafana-util.rs"],
        "owned_tests": ["rust/src/common_rust_tests.rs"],
        "tags": ["packaging", "rust"],
    },
    "rust/src/bin/grafana-util.rs": {
        "purpose": "Rust binary entrypoint.",
        "depends_on": ["rust/src/cli.rs", "rust/src/lib.rs"],
        "owned_tests": ["rust/src/cli_rust_tests.rs"],
        "tags": ["rust", "entrypoint", "cli"],
    },
    "rust/src/lib.rs": {
        "purpose": "Rust crate module export root.",
        "depends_on": ["rust/src/cli.rs", "rust/src/dashboard.rs", "rust/src/alert.rs", "rust/src/access.rs"],
        "owned_tests": ["rust/src/common_rust_tests.rs"],
        "tags": ["rust", "core", "module"],
    },
    "rust/src/cli.rs": {
        "purpose": "Rust unified CLI dispatch definitions.",
        "depends_on": ["rust/src/dashboard_cli_defs.rs", "rust/src/alert_cli_defs.rs", "rust/src/access_cli_defs.rs"],
        "owned_tests": ["rust/src/cli_rust_tests.rs"],
        "tags": ["rust", "cli", "dispatcher"],
    },
    "rust/src/access_org.rs": {
        "purpose": "Rust organization list/add/modify/delete/export/import workflows.",
        "depends_on": ["rust/src/access.rs", "rust/src/access_cli_defs.rs", "rust/src/access_render.rs"],
        "owned_tests": ["rust/src/access_rust_tests.rs"],
        "tags": ["rust", "access", "org"],
    },
    "tests/test_python_unified_cli.py": {
        "purpose": "Python regression tests for unified CLI behavior.",
        "depends_on": ["grafana_utils/unified_cli.py", "grafana_utils/__main__.py"],
        "owned_tests": ["tests/test_python_unified_cli.py"],
        "tags": ["tests", "python", "unified"],
    },
    "tests/test_python_dashboard_cli.py": {
        "purpose": "Python regression tests for dashboard CLI behavior.",
        "depends_on": ["grafana_utils/dashboard_cli.py", "grafana_utils/dashboards"],
        "owned_tests": ["tests/test_python_dashboard_cli.py"],
        "tags": ["tests", "python", "dashboard"],
    },
    "tests/test_python_alert_cli.py": {
        "purpose": "Python regression tests for alert CLI behavior.",
        "depends_on": ["grafana_utils/alert_cli.py", "grafana_utils/alerts/provisioning.py"],
        "owned_tests": ["tests/test_python_alert_cli.py"],
        "tags": ["tests", "python", "alert"],
    },
    "tests/test_python_access_cli.py": {
        "purpose": "Python regression tests for access CLI behavior.",
        "depends_on": ["grafana_utils/access_cli.py", "grafana_utils/access/parser.py", "grafana_utils/access/workflows.py"],
        "owned_tests": ["tests/test_python_access_cli.py"],
        "tags": ["tests", "python", "access"],
    },
    "tests/test_python_datasource_cli.py": {
        "purpose": "Python regression tests for datasource CLI behavior.",
        "depends_on": ["grafana_utils/datasource_cli.py", "grafana_utils/datasource/parser.py", "grafana_utils/datasource/workflows.py"],
        "owned_tests": ["tests/test_python_datasource_cli.py"],
        "tags": ["tests", "python", "datasource"],
    },
    "tests/test_python_sync_cli.py": {
        "purpose": "Python regression tests for sync CLI behavior.",
        "depends_on": ["grafana_utils/sync_cli.py", "grafana_utils/gitops_sync.py"],
        "owned_tests": ["tests/test_python_sync_cli.py"],
        "tags": ["tests", "python", "sync"],
    },
}


def run_rg(args):
    result = subprocess.run(["rg"] + args, cwd=repo_root, capture_output=True, text=True, check=False)
    if result.returncode not in (0, 1):
        raise RuntimeError(result.stderr.strip() or "rg failed")
    return [line.strip() for line in result.stdout.splitlines() if line.strip()]


def discover_public_apis(rel_path):
    if rel_path.endswith(".py"):
        lines = run_rg(["-o", r"^(def|class)\s+[A-Za-z_][A-Za-z0-9_]*", rel_path])
        names = sorted({line.split()[1] for line in lines if len(line.split()) >= 2})
        return names
    if rel_path.endswith(".rs"):
        lines = run_rg(["-o", r"^\s*pub\s+(fn|struct|enum|trait|mod)\s+[A-Za-z_][A-Za-z0-9_]*", rel_path])
        names = sorted({line.split()[2] for line in lines if len(line.split()) >= 3})
        return names
    return []


entries = []
for rel_path in sorted(seed.keys()):
    abs_path = os.path.join(repo_root, rel_path)
    if not os.path.exists(abs_path):
        continue
    cfg = seed[rel_path]
    entries.append(
        {
            "path": rel_path,
            "purpose": cfg["purpose"],
            "public_apis": discover_public_apis(rel_path),
            "depends_on": cfg["depends_on"],
            "owned_tests": cfg["owned_tests"],
            "tags": cfg["tags"],
        }
    )

payload = {
    "metadata": {
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "generator": "scripts/update_context_index.sh",
        "schema_version": 1,
        "entry_count": len(entries),
    },
    "entries": entries,
}

with open(output_path, "w", encoding="utf-8") as handle:
    json.dump(payload, handle, indent=2, ensure_ascii=True)
    handle.write("\n")
PY

echo "Updated ${output_path}"
