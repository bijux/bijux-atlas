#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need helm grep

rendered="$(mktemp)"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" >"$rendered"

if grep -Eq 'checksum/(config|secret|values)' "$rendered"; then
  echo "checksum rollout policy violated: deployment uses checksum annotations; contract requires explicit restart flow" >&2
  exit 1
fi

echo "no checksum rollout contract passed"
