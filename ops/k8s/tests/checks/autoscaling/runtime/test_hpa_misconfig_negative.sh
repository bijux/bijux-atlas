#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../../_lib/common.sh"
setup_test_traps
need helm

if helm template "$RELEASE" "$CHART" -n "$NS" -f "$ROOT/ops/k8s/values/perf.yaml" \
  --set hpa.enabled=true \
  --set metrics.customMetrics.enabled=false >/tmp/atlas-hpa-misconfig.yaml 2>/tmp/atlas-hpa-misconfig.err; then
  echo "expected helm template to fail when hpa.enabled=true and metrics.customMetrics.enabled=false" >&2
  exit 1
fi

grep -Eq "hpa.enabled=true requires metrics.customMetrics.enabled=true|metrics.customMetrics.enabled" /tmp/atlas-hpa-misconfig.err
echo "hpa misconfig negative check passed"
