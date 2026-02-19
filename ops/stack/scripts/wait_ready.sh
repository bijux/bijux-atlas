#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
source "$ROOT/_lib/common.sh"
NS="${1:-$(ops_layer_ns_stack)}"
TIMEOUT="${2:-180s}"

kubectl wait --for=condition=Ready nodes --all --timeout="$TIMEOUT" >/dev/null
kubectl -n kube-system rollout status deploy/coredns --timeout="$TIMEOUT" >/dev/null
kubectl -n "$NS" wait --for=condition=available deploy/"$(ops_layer_contract_get services.minio.service_name)" --timeout="$TIMEOUT" >/dev/null
kubectl -n "$NS" wait --for=condition=available deploy/"$(ops_layer_contract_get services.prometheus.service_name)" --timeout="$TIMEOUT" >/dev/null

echo "stack ready: ns=$NS"
