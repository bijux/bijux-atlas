#!/usr/bin/env bash
set -euo pipefail
cluster="${ATLAS_E2E_CLUSTER_NAME:-bijux-atlas-e2e}"
node="${cluster}-control-plane"
size_mb="${FILL_SIZE_MB:-512}"
mode="${1:-fill}" # fill|clean
file="/var/tmp/atlas-disk-pressure.img"

if [ "${OPS_DRY_RUN:-0}" = "1" ]; then
  echo "DRY-RUN disk-pressure mode=$mode node=$node size_mb=$size_mb"
  exit 0
fi

if [ "$mode" = "clean" ]; then
  docker exec "$node" sh -c "rm -f $file"
  echo "disk pressure file removed"
  exit 0
fi

docker exec "$node" sh -c "dd if=/dev/zero of=$file bs=1m count=$size_mb status=none"
echo "disk pressure simulated: $size_mb MiB on $node"
