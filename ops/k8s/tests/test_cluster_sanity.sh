#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need kubectl

kubectl wait --for=condition=Ready nodes --all --timeout=120s >/dev/null
kubectl -n kube-system rollout status deploy/coredns --timeout=120s >/dev/null
kubectl get storageclass >/dev/null
kubectl get nodes -o jsonpath='{.items[0].status.conditions[?(@.type=="Ready")].status}' | grep -q True

echo "cluster sanity passed"
