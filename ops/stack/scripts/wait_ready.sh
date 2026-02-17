#!/usr/bin/env bash
set -euo pipefail
NS="${1:-atlas-e2e}"
TIMEOUT="${2:-180s}"

kubectl wait --for=condition=Ready nodes --all --timeout="$TIMEOUT" >/dev/null
kubectl -n kube-system rollout status deploy/coredns --timeout="$TIMEOUT" >/dev/null
kubectl -n "$NS" wait --for=condition=available deploy/minio --timeout="$TIMEOUT" >/dev/null
kubectl -n "$NS" wait --for=condition=available deploy/prometheus --timeout="$TIMEOUT" >/dev/null

echo "stack ready: ns=$NS"
