#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
cd "$ROOT"

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  cat <<'EOF'
Usage: scripts/areas/ci/scripts-ci.sh

CI glue for scripts checks.
Runs make scripts-check.
EOF
  exit 0
fi

export PYTHONPATH="$ROOT/packages/bijux-atlas-scripts/src${PYTHONPATH:+:$PYTHONPATH}"
exec python3 -m bijux_atlas_scripts.cli ci scripts
