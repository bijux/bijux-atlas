#!/usr/bin/env sh
set -eu

CLUSTER_NAME="${ATLAS_E2E_CLUSTER_NAME:-bijux-atlas-e2e}"

if command -v kind >/dev/null 2>&1; then
  if kind get clusters | grep -qx "$CLUSTER_NAME"; then
    kind delete cluster --name "$CLUSTER_NAME"
  fi
fi

echo "e2e stack is down"
