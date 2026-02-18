#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
ACTION="${1:-}"
shift || true

case "$ACTION" in
  block-minio)
    exec "$ROOT/stack/faults/block-minio.sh" "${1:-on}"
    ;;
  toxiproxy-latency)
    exec "$ROOT/stack/faults/toxiproxy-latency.sh" "${1:-250}" "${2:-25}"
    ;;
  throttle-network)
    exec "$ROOT/stack/faults/throttle-network.sh" "${1:-256}"
    ;;
  cpu-throttle)
    exec "$ROOT/stack/faults/cpu-throttle.sh"
    ;;
  fill-node-disk)
    exec "$ROOT/stack/faults/fill-node-disk.sh" "${1:-fill}"
    ;;
  *)
    echo "usage: $0 {block-minio|toxiproxy-latency|throttle-network|cpu-throttle|fill-node-disk} [args...]" >&2
    exit 2
    ;;
esac
