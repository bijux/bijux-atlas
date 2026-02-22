#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path
from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need kubectl

"$SCRIPT_DIR/../contracts/test_metrics_pipeline_ready.sh"
wait_ready
kubectl -n "$NS" get hpa "$SERVICE_NAME" >/dev/null
kubectl -n "$NS" get hpa "$SERVICE_NAME" -o jsonpath='{.spec.metrics[*].pods.metric.name}' | grep -q "bijux_inflight_heavy_queries" || {
  echo "failure_mode: hpa_inflight_metric_missing" >&2
  exit 1
}

before_desired="$(kubectl -n "$NS" get hpa "$SERVICE_NAME" -o jsonpath='{.status.desiredReplicas}' || echo 1)"

kubectl -n "$NS" delete pod hpa-inflight-load --ignore-not-found >/dev/null 2>&1 || true
kubectl -n "$NS" run hpa-inflight-load --image=curlimages/curl --restart=Never --command -- sh -ceu '
  for i in $(seq 1 1000); do
    curl -fsS "http://'"$SERVICE_NAME"':8080/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&name_prefix=G&limit=100" >/dev/null || true
  done
' >/dev/null

trended=0
for _ in $(seq 1 24); do
  now_desired="$(kubectl -n "$NS" get hpa "$SERVICE_NAME" -o jsonpath='{.status.desiredReplicas}' || echo "$before_desired")"
  if [ "${now_desired:-0}" -gt "${before_desired:-0}" ]; then
    trended=1
    break
  fi
  sleep 5
done
kubectl -n "$NS" delete pod hpa-inflight-load --ignore-not-found >/dev/null 2>&1 || true

if [ "$trended" -ne 1 ]; then
  echo "failure_mode: hpa_inflight_metric_no_scale_trend" >&2
  exit 1
fi
echo "hpa inflight metric scaling contract passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
