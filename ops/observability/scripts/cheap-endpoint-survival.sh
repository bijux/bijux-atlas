#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: prove cheap endpoint remains available under overload admission control.
# stability: public
# called-by: make observability-pack-drills
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"
"$ROOT/ops/observability/scripts/overload-admission-control.sh"
curl -fsS "$ATLAS_BASE_URL/v1/version" >/dev/null
curl -fsS "$ATLAS_BASE_URL/metrics" | grep -q '^bijux_cheap_queries_served_while_overloaded_total'
echo "cheap endpoint survival drill passed"
