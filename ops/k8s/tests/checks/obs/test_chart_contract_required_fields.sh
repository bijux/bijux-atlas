#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need helm
rendered="$(mktemp)"
rendered_defaults="$(mktemp)"
rendered_perf="$(mktemp)"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" > "$rendered"
helm template "$RELEASE" "$CHART" -n "$NS" > "$rendered_defaults"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$ROOT/ops/k8s/values/perf.yaml" > "$rendered_perf"

# Deployment must define readiness/liveness probes and resources (requests+limits).
grep -q "readinessProbe:" "$rendered"
grep -q "livenessProbe:" "$rendered"
grep -q "resources:" "$rendered"
grep -q "requests:" "$rendered"
grep -q "limits:" "$rendered"

# PDB is required by chart contract (defaults must include it).
grep -q "kind: PodDisruptionBudget" "$rendered_defaults"

# HPA perf profile must carry metrics annotations and custom metrics wiring.
grep -q "kind: HorizontalPodAutoscaler" "$rendered_perf"
grep -q "prometheus.io/scrape: \"true\"" "$rendered_perf"
grep -q "prometheus.io/path: /metrics" "$rendered_perf"
grep -q "name: bijux_http_request_latency_p95_seconds" "$rendered_perf"
grep -q "name: bijux_inflight_heavy_queries" "$rendered_perf"

rm -f "$rendered" "$rendered_defaults" "$rendered_perf"
echo "chart required fields contract passed"
