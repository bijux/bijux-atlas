#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need helm grep

rendered="$(mktemp)"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$ROOT/ops/k8s/values/perf.yaml" --set hpa.enabled=true >"$rendered"
grep -q "kind: HorizontalPodAutoscaler" "$rendered"
grep -q "resources:" "$rendered"
grep -q "requests:" "$rendered"
grep -q "limits:" "$rendered"

echo "hpa enabled resources contract passed"
