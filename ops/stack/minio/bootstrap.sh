#!/usr/bin/env sh
set -eu

NS="${NS:-atlas-e2e}"
BUCKET="${MINIO_BUCKET:-atlas-artifacts}"
USER="${MINIO_ROOT_USER:-minioadmin}"
PASS="${MINIO_ROOT_PASSWORD:-minioadmin}"
ENDPOINT="${MINIO_ENDPOINT:-http://minio.${NS}.svc.cluster.local:9000}"

kubectl -n "$NS" delete pod minio-bootstrap --ignore-not-found >/dev/null 2>&1 || true

# Namespace bootstrap in fresh kind clusters can race default serviceaccount creation.
for _ in 1 2 3 4 5 6 7 8 9 10; do
  if kubectl -n "$NS" get serviceaccount default >/dev/null 2>&1; then
    break
  fi
  sleep 1
done
kubectl -n "$NS" get serviceaccount default >/dev/null 2>&1

kubectl -n "$NS" wait --for=condition=available deploy/minio --timeout="${MINIO_WAIT_TIMEOUT:-180s}" >/dev/null

kubectl -n "$NS" run minio-bootstrap \
  --image=minio/mc:RELEASE.2025-01-17T23-25-50Z \
  --restart=Never \
  --rm -i --command -- /bin/sh -ceu "
mc alias set local '$ENDPOINT' '$USER' '$PASS'
mc mb --ignore-existing local/$BUCKET
mc anonymous set download local/$BUCKET
mc ilm export local/$BUCKET >/dev/null 2>&1 || true
"
