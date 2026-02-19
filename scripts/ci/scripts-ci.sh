#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  cat <<'EOF'
Usage: scripts/ci/scripts-ci.sh

CI glue for scripts checks.
Runs make scripts-check.
EOF
  exit 0
fi

exec make -s scripts-check
