#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: inject store latency and assert breaker/health signals.
# stability: public
# called-by: make ops-drill-toxiproxy-latency, make observability-pack-drills
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"

"$ROOT/stack/faults/toxiproxy-latency.sh" 1500 200
sleep 3
curl -fsS "$ATLAS_BASE_URL/healthz" >/dev/null || true
if ! curl -fsS "$ATLAS_BASE_URL/metrics" | grep -E '^bijux_store_breaker_open' >/dev/null; then
  echo "store breaker metric missing under injected store latency" >&2
  exit 1
fi

echo "store latency injection drill passed"
