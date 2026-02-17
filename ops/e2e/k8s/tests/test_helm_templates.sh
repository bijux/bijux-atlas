#!/usr/bin/env sh
set -eu
. "$(dirname "$0")/common.sh"
need helm

helm lint "$CHART" -f "$VALUES" >/dev/null
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" >/tmp/atlas-chart-rendered.yaml
# edge-case rendering: rollout on + offline profile
helm template "$RELEASE" "$CHART" -n "$NS" -f "$ROOT/ops/k8s/values/offline.yaml" --set rollout.enabled=true >/tmp/atlas-chart-rendered-rollout.yaml

grep -q "kind: Deployment" /tmp/atlas-chart-rendered.yaml
grep -q "kind: ConfigMap" /tmp/atlas-chart-rendered.yaml
grep -q "kind: Rollout" /tmp/atlas-chart-rendered-rollout.yaml

echo "helm template gate passed"
