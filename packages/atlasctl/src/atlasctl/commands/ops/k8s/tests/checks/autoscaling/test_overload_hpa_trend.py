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
kubectl -n "$NS" get hpa "$SERVICE_NAME" >/dev/null || {
  echo "failure_mode: hpa_not_configured" >&2
  exit 1
}

before_desired="$(kubectl -n "$NS" get hpa "$SERVICE_NAME" -o jsonpath='{.status.desiredReplicas}' || echo 1)"
(cd "$ROOT" && ./bin/atlasctl ops load --report text run --suite spike-overload-proof.json --out artifacts/perf/results) >/dev/null

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
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
