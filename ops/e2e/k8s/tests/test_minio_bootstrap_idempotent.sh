#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need kubectl

kubectl get ns atlas-e2e >/dev/null 2>&1 || kubectl create ns atlas-e2e >/dev/null
kubectl apply -f "$ROOT/ops/stack/minio/minio.yaml" >/dev/null
"$ROOT/ops/stack/minio/bootstrap.sh"
"$ROOT/ops/stack/minio/bootstrap.sh"

echo "minio bootstrap idempotent"
