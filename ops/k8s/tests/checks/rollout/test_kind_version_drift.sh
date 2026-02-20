#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)"
python3 "$ROOT/packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_tool_versions.py" kind
echo "kind version drift test passed"
