#!/usr/bin/env bash
set -euo pipefail
NS="${ATLAS_E2E_NAMESPACE:-atlas-e2e}"
new_user="minioadmin"
new_pass="minioadmin-rotated"

kubectl -n "$NS" get secret minio-auth >/dev/null
kubectl -n "$NS" patch secret minio-auth --type merge -p \
  "{\"stringData\":{\"MINIO_ROOT_USER\":\"$new_user\",\"MINIO_ROOT_PASSWORD\":\"$new_pass\"}}" >/dev/null
kubectl -n "$NS" rollout restart deploy/minio >/dev/null
kubectl -n "$NS" rollout status deploy/minio --timeout="${OPS_WAIT_TIMEOUT:-180s}" >/dev/null

kubectl -n "$NS" delete pod minio-creds-verify --ignore-not-found >/dev/null 2>&1 || true
kubectl -n "$NS" run minio-creds-verify \
  --image=minio/mc:RELEASE.2025-01-17T23-25-50Z \
  --restart=Never \
  --rm -i --command -- /bin/sh -ceu " \
mc alias set local 'http://minio.$NS.svc.cluster.local:9000' '$new_user' '$new_pass'; \
mc admin info local >/dev/null"

echo "minio credentials rotation drill passed"
