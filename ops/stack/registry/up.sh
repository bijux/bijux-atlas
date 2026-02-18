#!/usr/bin/env bash
set -euo pipefail
name="${ATLAS_KIND_REGISTRY_NAME:-kind-registry}"
port="${ATLAS_KIND_REGISTRY_PORT:-5001}"
if [ "${OPS_DRY_RUN:-0}" = "1" ]; then
  echo "DRY-RUN docker run -d -p 127.0.0.1:${port}:5000 --restart=always --name ${name} registry:2"
  exit 0
fi
if ! docker inspect -f '{{.State.Running}}' "$name" >/dev/null 2>&1; then
  docker run -d --restart=always -p "127.0.0.1:${port}:5000" --name "$name" registry:2 >/dev/null
fi
node="${ATLAS_E2E_CLUSTER_NAME:-bijux-atlas-e2e}-control-plane"
docker network connect kind "$name" >/dev/null 2>&1 || true
cat <<EOM
local registry up:
- container: $name
- host endpoint: localhost:${port}
- kind node: ${node}
EOM
