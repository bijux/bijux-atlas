#!/usr/bin/env bash
set -euo pipefail
cluster="${ATLAS_E2E_CLUSTER_NAME:-bijux-atlas-e2e}"
if ! kind get clusters | grep -qx "$cluster"; then
  echo "kind cluster not present: $cluster"
  exit 0
fi
if [ "${OPS_DRY_RUN:-0}" = "1" ]; then
  echo "DRY-RUN kind delete cluster --name $cluster"
  exit 0
fi
kind delete cluster --name "$cluster"
echo "kind cluster deleted: $cluster"
