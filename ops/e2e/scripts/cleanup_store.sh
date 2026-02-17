#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
STORE_ROOT="${ATLAS_E2E_STORE_ROOT:-$ROOT/artifacts/e2e-store}"
OUT_ROOT="${ATLAS_E2E_OUTPUT_ROOT:-$ROOT/artifacts/e2e-datasets}"
NS="${ATLAS_E2E_NAMESPACE:-atlas-e2e}"
BUCKET="${MINIO_BUCKET:-atlas-artifacts}"

rm -rf "$STORE_ROOT" "$OUT_ROOT"
mkdir -p "$STORE_ROOT" "$OUT_ROOT"

# reset bucket content deterministically (best effort)
if kubectl -n "$NS" get svc minio >/dev/null 2>&1; then
  kubectl -n "$NS" run minio-clean \
    --image=minio/mc:RELEASE.2025-01-17T23-25-50Z \
    --restart=Never --rm -i --command -- /bin/sh -ceu "
mc alias set local http://minio.$NS.svc.cluster.local:9000 minioadmin minioadmin
mc rm -r --force local/$BUCKET || true
mc mb --ignore-existing local/$BUCKET
"
fi

echo "store cleaned"
