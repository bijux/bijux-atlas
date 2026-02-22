#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
# shellcheck source=/dev/null
. "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/assets/lib/ops_common.sh"

NS="${ATLAS_E2E_NAMESPACE:-$(ops_layer_ns_stack)}"
OUT="${1:-$ROOT/artifacts/evidence/contracts/live-snapshot.services.json}"
mkdir -p "$(dirname "$OUT")"

if ! kubectl cluster-info >/dev/null 2>&1; then
  echo "live snapshot validation skipped: cluster not available"
  exit 0
fi
if ! kubectl get ns "$NS" >/dev/null 2>&1; then
  echo "live snapshot validation skipped: namespace $NS not found"
  exit 0
fi

deploy_out="$(dirname "$OUT")/live-snapshot.deployments.json"
triage_out="$(dirname "$OUT")/layer-drift-triage.json"

kubectl -n "$NS" get svc -o json > "$OUT"
kubectl -n "$NS" get deploy -o json > "$deploy_out"
python3 "$ROOT/ops/stack/tests/check_live_layer_snapshot.py" \
  "$OUT" \
  "$deploy_out" \
  "$ROOT/ops/_meta/layer-contract.json" \
  "$triage_out"
