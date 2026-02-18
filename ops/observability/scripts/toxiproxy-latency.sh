#!/usr/bin/env sh
set -eu
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
NS="${ATLAS_NS:-atlas-e2e}"
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"

"$ROOT/stack/faults/toxiproxy-latency.sh" 1500 200
sleep 3
curl -fsS "$ATLAS_BASE_URL/healthz" >/dev/null || true
if ! curl -fsS "$ATLAS_BASE_URL/metrics" | grep -E '^bijux_store_breaker_open' >/dev/null; then
  echo "store breaker metric missing under toxiproxy latency" >&2
  exit 1
fi

echo "toxiproxy latency drill passed"
