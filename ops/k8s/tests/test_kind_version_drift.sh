#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../../.." && pwd)"
python3 "$ROOT/scripts/layout/check_tool_versions.py" kind
echo "kind version drift test passed"
