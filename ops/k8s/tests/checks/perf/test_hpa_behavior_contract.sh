#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need helm grep

rendered="$(mktemp)"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$ROOT/ops/k8s/values/perf.yaml" >"$rendered"
grep -q "kind: HorizontalPodAutoscaler" "$rendered"
grep -q "behavior:" "$rendered"
grep -q "scaleUp:" "$rendered"
grep -q "scaleDown:" "$rendered"
grep -q "policies:" "$rendered"

echo "hpa behavior contract passed"
