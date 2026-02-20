#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_init_run_id
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="$OPS_RUN_DIR"
ops_env_load
ops_entrypoint_start "k8s-restart"
ops_version_guard kubectl

NS="${ATLAS_E2E_NAMESPACE:-${ATLAS_NS:-$(ops_layer_ns_k8s)}}"
RELEASE="${ATLAS_E2E_RELEASE_NAME:-$(ops_layer_contract_get release_metadata.defaults.release_name)}"
SERVICE_NAME="${ATLAS_E2E_SERVICE_NAME:-$(ops_layer_service_atlas)}"
TIMEOUT="${ATLAS_E2E_TIMEOUT:-180s}"

./ops/run/k8s-validate-configmap-keys.sh "$NS" "$SERVICE_NAME"

echo "rolling restart deployment/${SERVICE_NAME} in namespace ${NS}"
ops_kubectl -n "$NS" rollout restart deployment/"$SERVICE_NAME" >/dev/null
ops_kubectl -n "$NS" rollout status deployment/"$SERVICE_NAME" --timeout="$TIMEOUT" >/dev/null
ops_kubectl -n "$NS" get deploy "$SERVICE_NAME" -o jsonpath='{.status.readyReplicas}' | grep -Eq '^[1-9][0-9]*$'

echo "k8s restart passed (ns=${NS} release=${RELEASE} service=${SERVICE_NAME})"
