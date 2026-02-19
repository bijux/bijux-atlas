#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)"
# shellcheck source=ops/_lib/common.sh
source "$ROOT/ops/_lib/common.sh"
cluster="${ATLAS_E2E_CLUSTER_NAME:-bijux-atlas-e2e}"
image="${ATLAS_E2E_LOCAL_IMAGE:-bijux-atlas:local}"
node="${cluster}-control-plane"

if [ "${OPS_DRY_RUN:-0}" = "1" ]; then
  echo "DRY-RUN validate image resolution for $image in $node"
  exit 0
fi

kind load docker-image "$image" --name "$cluster" >/dev/null
if docker exec "$node" crictl images | grep -q "${image%%:*}"; then
  echo "kind image resolution passed: $image"
else
  echo "image not present in kind node runtime: $image" >&2
  exit 1
fi
