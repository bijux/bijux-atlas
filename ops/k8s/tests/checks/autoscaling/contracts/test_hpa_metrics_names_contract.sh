#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../../_lib/common.sh"
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
