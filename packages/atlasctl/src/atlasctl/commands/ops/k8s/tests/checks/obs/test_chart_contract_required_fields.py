#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path
from ._shell_common import run_k8s_test_shell

def main() -> int:
    return run_k8s_test_shell("""
setup_test_traps
need helm
rendered="$(mktemp)"
rendered_defaults="$(mktemp)"
rendered_perf="$(mktemp)"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" > "$rendered"
helm template "$RELEASE" "$CHART" -n "$NS" > "$rendered_defaults"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$ROOT/ops/k8s/values/perf.yaml" > "$rendered_perf"
grep -q "readinessProbe:" "$rendered"
grep -q "livenessProbe:" "$rendered"
grep -q "resources:" "$rendered"
grep -q "requests:" "$rendered"
grep -q "limits:" "$rendered"
grep -q "kind: PodDisruptionBudget" "$rendered_defaults"
grep -q "kind: HorizontalPodAutoscaler" "$rendered_perf"
grep -q 'prometheus.io/scrape: "true"' "$rendered_perf"
grep -q "prometheus.io/path: /metrics" "$rendered_perf"
grep -q "name: bijux_http_request_latency_p95_seconds" "$rendered_perf"
grep -q "name: bijux_inflight_heavy_queries" "$rendered_perf"
rm -f "$rendered" "$rendered_defaults" "$rendered_perf"
echo "chart required fields contract passed"
    """, Path(__file__))

if __name__ == "__main__":
    raise SystemExit(main())
