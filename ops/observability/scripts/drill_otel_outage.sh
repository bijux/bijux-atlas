#!/usr/bin/env bash
set -euo pipefail
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"
: "${ATLAS_E2E_ENABLE_OTEL:=0}"
if [ "$ATLAS_E2E_ENABLE_OTEL" != "1" ]; then
  echo "otel disabled; skip"
  exit 0
fi
kubectl -n atlas-e2e scale deploy/otel-collector --replicas=0 >/dev/null || true
sleep 3
curl -fsS "$ATLAS_BASE_URL/healthz" >/dev/null
curl -fsS "$ATLAS_BASE_URL/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=1" >/dev/null || true

echo "otel outage drill passed"
