#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../../_lib/common.sh"
setup_test_traps
need kubectl

"$SCRIPT_DIR/../contracts/test_metrics_pipeline_ready.sh"
wait_ready
kubectl -n "$NS" get hpa "$SERVICE_NAME" >/dev/null || {
  echo "failure_mode: hpa_not_configured" >&2
  exit 1
}

before_desired="$(kubectl -n "$NS" get hpa "$SERVICE_NAME" -o jsonpath='{.status.desiredReplicas}' || echo 1)"
"$ROOT/ops/load/scripts/run_suite.sh" spike-overload-proof.json "$ROOT/artifacts/perf/results" >/dev/null

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
  echo "failure_mode: overload_no_hpa_uptrend" >&2
  exit 1
fi
echo "overload to hpa trend contract passed"
