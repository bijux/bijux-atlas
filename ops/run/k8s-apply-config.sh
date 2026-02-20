#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_init_run_id
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="$OPS_RUN_DIR"
ops_env_load
ops_entrypoint_start "k8s-apply-config"
ops_version_guard kubectl helm

NS="${ATLAS_E2E_NAMESPACE:-${ATLAS_NS:-$(ops_layer_ns_k8s)}}"
SERVICE_NAME="${ATLAS_E2E_SERVICE_NAME:-$(ops_layer_service_atlas)}"
CM_NAME="${SERVICE_NAME}-config"
VALUES_FILE="${ATLAS_E2E_VALUES_FILE:-${ATLAS_VALUES_FILE:-$ROOT/ops/k8s/values/local.yaml}}"
PROFILE="${PROFILE:-local}"

old_hash="$(kubectl -n "$NS" get configmap "$CM_NAME" -o jsonpath='{.metadata.resourceVersion}' 2>/dev/null || true)"

make -s ops-values-validate
PROFILE="$PROFILE" make -s ops-deploy

new_hash="$(kubectl -n "$NS" get configmap "$CM_NAME" -o jsonpath='{.metadata.resourceVersion}' 2>/dev/null || true)"
if [ -n "$old_hash" ] && [ -n "$new_hash" ] && [ "$old_hash" != "$new_hash" ]; then
  echo "configmap changed after deploy (${CM_NAME}); restarting workloads"
  ./ops/run/k8s-restart.sh
else
  echo "configmap unchanged after deploy (${CM_NAME}); restart skipped"
fi

./ops/run/k8s-validate-configmap-keys.sh "$NS" "$SERVICE_NAME"
echo "k8s apply-config passed (values=${VALUES_FILE})"
