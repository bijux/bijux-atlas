#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../../_lib/common.sh"
setup_test_traps
need helm

helm lint "$CHART" -f "$VALUES" >/dev/null
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" >/tmp/atlas-chart-rendered.yaml
# edge-case rendering: rollout on + offline profile
helm template "$RELEASE" "$CHART" -n "$NS" -f "$ROOT/ops/k8s/values/offline.yaml" --set rollout.enabled=true >/tmp/atlas-chart-rendered-rollout.yaml
if helm template "$RELEASE" "$CHART" -n "$NS" -f "$ROOT/ops/k8s/values/perf.yaml" --set hpa.enabled=true --set metrics.customMetrics.enabled=false >/tmp/atlas-chart-rendered-hpa-invalid.yaml 2>/tmp/atlas-chart-rendered-hpa-invalid.err; then
  echo "expected render failure when HPA is enabled without custom metrics pipeline" >&2
  exit 1
fi
grep -Eq "hpa.enabled=true requires metrics.customMetrics.enabled=true|metrics.customMetrics.enabled" /tmp/atlas-chart-rendered-hpa-invalid.err

grep -q "kind: Deployment" /tmp/atlas-chart-rendered.yaml
grep -q "kind: ConfigMap" /tmp/atlas-chart-rendered.yaml
grep -q "kind: Rollout" /tmp/atlas-chart-rendered-rollout.yaml

echo "helm template gate passed"
