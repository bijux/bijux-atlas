#!/usr/bin/env sh
set -eu

BASE_URL="${ATLAS_E2E_BASE_URL:-http://127.0.0.1:18080}"
NS="${ATLAS_E2E_NAMESPACE:-atlas-e2e}"
RELEASE="${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}"
DURATION_SECS="${ATLAS_E2E_SOAK_SECS:-600}"
MAX_GROWTH_KIB="${ATLAS_E2E_SOAK_MAX_GROWTH_KIB:-131072}"

POD="$(kubectl -n "$NS" get pod -l app.kubernetes.io/instance="$RELEASE" -o jsonpath='{.items[0].metadata.name}')"
RSS0="$(kubectl -n "$NS" exec "$POD" -- awk '/VmRSS/ {print $2}' /proc/1/status)"

end=$(( $(date +%s) + DURATION_SECS ))
while [ "$(date +%s)" -lt "$end" ]; do
  curl -fsS "$BASE_URL/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=10" >/dev/null
  curl -fsS "$BASE_URL/v1/sequence/region?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-120" >/dev/null || true
  sleep 1
done

RSS1="$(kubectl -n "$NS" exec "$POD" -- awk '/VmRSS/ {print $2}' /proc/1/status)"
GROWTH=$(( RSS1 - RSS0 ))
[ "$GROWTH" -le "$MAX_GROWTH_KIB" ] || { echo "memory growth too high: ${GROWTH}KiB" >&2; exit 1; }

echo "soak passed: rss_growth_kib=$GROWTH"
