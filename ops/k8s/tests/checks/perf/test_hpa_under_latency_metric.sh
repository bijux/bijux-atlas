#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need kubectl

"$SCRIPT_DIR/test_metrics_pipeline_ready.sh"

wait_ready
kubectl -n "$NS" get hpa "$SERVICE_NAME" >/dev/null
kubectl -n "$NS" get hpa "$SERVICE_NAME" -o jsonpath='{.spec.metrics[*].pods.metric.name}' | grep -q "bijux_http_request_latency_p95_seconds" || {
  echo "failure_mode: hpa_latency_metric_missing" >&2
  exit 1
}

before_desired="$(kubectl -n "$NS" get hpa "$SERVICE_NAME" -o jsonpath='{.status.desiredReplicas}' || echo 1)"
"$ROOT/ops/load/scripts/run_suite.sh" hpa-validation-short.json "$ROOT/artifacts/perf/results" >/dev/null

trended=0
for _ in $(seq 1 24); do
  now_desired="$(kubectl -n "$NS" get hpa "$SERVICE_NAME" -o jsonpath='{.status.desiredReplicas}' || echo "$before_desired")"
  if [ "${now_desired:-0}" -gt "${before_desired:-0}" ]; then
    trended=1
    break
  fi
  sleep 5
done

if [ "$trended" -ne 1 ]; then
  echo "failure_mode: hpa_latency_metric_no_scale_trend" >&2
  exit 1
fi
echo "hpa latency metric scaling contract passed"
