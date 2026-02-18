#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need curl

: "${ATLAS_E2E_ENABLE_REDIS:=0}"
if [ "$ATLAS_E2E_ENABLE_REDIS" != "1" ]; then
  echo "redis disabled; skip"
  exit 0
fi

install_chart
wait_ready
with_port_forward 18080
curl -fsS "$BASE_URL/healthz" >/dev/null
curl -fsS "$BASE_URL/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=1" >/dev/null || true
if ! curl -fsS "$BASE_URL/metrics" | grep -E '^bijux_redis_' >/dev/null; then
  echo "expected redis metrics not found" >&2
  exit 1
fi

echo "redis backend metrics present"
