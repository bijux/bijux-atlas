#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path
from ._shell_common import run_k8s_test_shell

def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need helm grep
rendered="$(mktemp)"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$ROOT/ops/k8s/values/perf.yaml" >"$rendered"
grep -q "kind: HorizontalPodAutoscaler" "$rendered"
grep -q "name: bijux_http_request_latency_p95_seconds" "$rendered"
grep -q "name: bijux_inflight_heavy_queries" "$rendered"
if grep -Eq "bijux_http_request_latency_p95_second[^s]|bijux_inflight_heavy_query[^s]" "$rendered"; then
  echo "hpa metrics names contract failed: typo-like metric name detected" >&2
  exit 1
fi
echo "hpa metrics names contract passed"
        """,
        Path(__file__),
    )

if __name__ == "__main__":
    raise SystemExit(main())
