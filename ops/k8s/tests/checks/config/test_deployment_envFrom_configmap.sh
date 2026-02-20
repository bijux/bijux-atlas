#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need helm grep

rendered="$(mktemp)"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" >"$rendered"
grep -q "kind: Deployment" "$rendered"
grep -q "envFrom:" "$rendered"
grep -q "configMapRef:" "$rendered"
grep -q "name: ${SERVICE_NAME}-config" "$rendered"

echo "deployment envFrom configmap contract passed"
