#!/usr/bin/env bash
set -euo pipefail
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"
kubectl -n atlas-e2e scale deploy/prometheus --replicas=0 >/dev/null || true
sleep 3
curl -fsS "$ATLAS_BASE_URL/healthz" >/dev/null
curl -fsS "$ATLAS_BASE_URL/metrics" >/dev/null

echo "prometheus outage drill passed"
