#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need helm

for values in "$ROOT"/ops/k8s/values/*.yaml; do
  helm lint "$CHART" -f "$values" >/dev/null
  helm template "$RELEASE" "$CHART" -n "$NS" -f "$values" >/dev/null
done

echo "values profiles validity contract passed"
