#!/usr/bin/env sh
set -eu

BASE_URL="${ATLAS_E2E_BASE_URL:-http://127.0.0.1:18080}"
NS="${ATLAS_E2E_NAMESPACE:-atlas-e2e}"
RELEASE="${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}"
LOCAL_PORT="${ATLAS_E2E_LOCAL_PORT:-18080}"
CURL="curl --connect-timeout 2 --max-time 5 -fsS"

if ! $CURL "$BASE_URL/healthz" >/dev/null 2>&1; then
  if ! kubectl config current-context >/dev/null 2>&1; then
    echo "metrics runtime check skipped: kubectl context is not configured"
    exit 0
  fi
  POD="$(kubectl -n "$NS" get pods -l app.kubernetes.io/instance="$RELEASE" --field-selector=status.phase=Running -o name 2>/dev/null | tail -n1 | cut -d/ -f2)"
  if [ -z "$POD" ]; then
    echo "metrics runtime check skipped: no running atlas pod found in namespace '$NS'"
    exit 0
  fi
  kubectl -n "$NS" port-forward "pod/$POD" "$LOCAL_PORT:8080" >/tmp/atlas-metrics-port-forward.log 2>&1 &
  PF_PID=$!
  trap 'kill "$PF_PID" >/dev/null 2>&1 || true' EXIT INT TERM
  BASE_URL="http://127.0.0.1:$LOCAL_PORT"
  for _ in 1 2 3 4 5 6 7 8 9 10; do
    if $CURL "$BASE_URL/healthz" >/dev/null 2>&1; then
      break
    fi
    sleep 1
  done
fi

METRICS="$($CURL "$BASE_URL/metrics")"
DATASETS="$($CURL "$BASE_URL/v1/datasets" || true)"
HAS_DATASETS=0
if echo "$DATASETS" | grep -q '"dataset"'; then
  HAS_DATASETS=1
fi

required="
bijux_http_requests_total
bijux_http_request_latency_p95_seconds
bijux_overload_shedding_active
bijux_store_breaker_open
bijux_errors_total
"
if [ "$HAS_DATASETS" = "1" ]; then
required="$required
bijux_dataset_hits
bijux_dataset_misses
bijux_store_download_p95_seconds
"
fi

for m in $required; do
  echo "$METRICS" | grep -q "^$m" || { echo "missing metric: $m" >&2; exit 1; }
done

echo "metrics verified"
