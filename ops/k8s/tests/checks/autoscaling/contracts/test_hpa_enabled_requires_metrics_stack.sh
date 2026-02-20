#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../../_lib/common.sh"
setup_test_traps
need helm grep

if helm template "$RELEASE" "$CHART" -n "$NS" -f "$ROOT/ops/k8s/values/perf.yaml" \
  --set hpa.enabled=true --set metrics.customMetrics.enabled=false >/tmp/hpa-metrics-invalid.yaml 2>/tmp/hpa-metrics-invalid.err; then
  echo "expected render failure when hpa enabled without custom metrics stack" >&2
  exit 1
fi
grep -Eq "metrics.customMetrics.enabled|hpa.enabled=true requires metrics.customMetrics.enabled=true" /tmp/hpa-metrics-invalid.err

if helm template "$RELEASE" "$CHART" -n "$NS" -f "$ROOT/ops/k8s/values/perf.yaml" \
  --set alertRules.enabled=true --set serviceMonitor.enabled=false >/tmp/hpa-obs-invalid.yaml 2>/tmp/hpa-obs-invalid.err; then
  echo "expected render failure when alert rules enabled without service monitor" >&2
  exit 1
fi
grep -Eq "alertRules.enabled=true requires serviceMonitor.enabled=true" /tmp/hpa-obs-invalid.err

echo "hpa enabled requires metrics stack contract passed"
