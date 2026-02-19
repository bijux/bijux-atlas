#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  cat <<'EOF'
Usage: scripts/check/no-direct-path-usage.sh

Fails on direct script execution in CI workflows.
EOF
  exit 0
fi
"$ROOT/scripts/layout/check_no_direct_script_runs.sh"
