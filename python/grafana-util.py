#!/usr/bin/env python3
"""Local source-tree wrapper for the packaged unified Grafana CLI."""

import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from grafana_utils.unified_cli import main


if __name__ == "__main__":
    sys.exit(main())
