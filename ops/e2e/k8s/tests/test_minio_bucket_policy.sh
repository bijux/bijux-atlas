#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need kubectl

kubectl get ns atlas-e2e >/dev/null 2>&1 || kubectl create ns atlas-e2e >/dev/null
kubectl apply -f "$ROOT/ops/stack/minio/minio.yaml" >/dev/null
"$ROOT/ops/stack/minio/bootstrap.sh"

kubectl -n atlas-e2e run minio-policy-check \
  --image=minio/mc:RELEASE.2025-01-17T23-25-50Z \
  --restart=Never --rm -i --command -- sh -ceu '
mc alias set local http://minio.atlas-e2e.svc.cluster.local:9000 minioadmin minioadmin >/dev/null
p=$(mc anonymous get local/atlas-artifacts)
echo "$p" | grep -Ei "download|readonly" >/dev/null
'

echo "minio bucket policy enforced"
