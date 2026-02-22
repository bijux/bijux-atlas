#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need kubectl

install_chart
wait_ready

CM_NAME="${SERVICE_NAME}-config"
kubectl -n "$NS" patch configmap "$CM_NAME" --type merge -p '{"data":{"ATLAS_UNKNOWN_KEY_SHOULD_FAIL":"1"}}' >/dev/null
if ./bin/atlasctl ops k8s --report text validate-configmap-keys "$NS" "$SERVICE_NAME"; then
  echo "unknown configmap key guard failed: validator accepted unexpected key" >&2
  exit 1
fi
kubectl -n "$NS" patch configmap "$CM_NAME" --type json -p='[{"op":"remove","path":"/data/ATLAS_UNKNOWN_KEY_SHOULD_FAIL"}]' >/dev/null
./bin/atlasctl ops k8s --report text validate-configmap-keys "$NS" "$SERVICE_NAME"

echo "configmap unknown key guard passed"
