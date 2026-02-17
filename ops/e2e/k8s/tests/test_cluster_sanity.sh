#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need kubectl

kubectl wait --for=condition=Ready nodes --all --timeout=120s >/dev/null
kubectl -n kube-system rollout status deploy/coredns --timeout=120s >/dev/null
kubectl get storageclass >/dev/null

echo "cluster sanity passed"
