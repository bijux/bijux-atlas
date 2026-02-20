#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
cd "$ROOT"

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  cat <<'EOF'
Usage: scripts/areas/check/no-duplicate-script-names.sh

Fails when dash/underscore duplicate script names exist.
EOF
  exit 0
fi

./scripts/bin/bijux-atlas-scripts run scripts/areas/check/check_duplicate_script_names.py
