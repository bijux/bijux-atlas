#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need kubectl

install_chart
wait_ready
pod=$(kubectl -n "$ATLAS_E2E_NAMESPACE" get pod -l app.kubernetes.io/name=bijux-atlas -o jsonpath='{.items[0].metadata.name}')
kubectl -n "$ATLAS_E2E_NAMESPACE" exec "$pod" -- sh -c 'wget -qO- http://minio.atlas-e2e.svc.cluster.local:9000/minio/health/ready >/dev/null'

echo "minio reachable from atlas pod"
