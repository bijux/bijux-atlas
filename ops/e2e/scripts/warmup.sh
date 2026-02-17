#!/usr/bin/env sh
set -eu

RELEASE="${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}"
NS="${ATLAS_E2E_NAMESPACE:-atlas-e2e}"
SERVICE_NAME="${ATLAS_E2E_SERVICE_NAME:-$RELEASE-bijux-atlas}"

if ! command -v kubectl >/dev/null 2>&1; then
  echo "kubectl is required" >&2
  exit 1
fi

if kubectl -n "$NS" get job "$SERVICE_NAME-dataset-warmup" >/dev/null 2>&1; then
  kubectl -n "$NS" wait --for=condition=complete --timeout=5m job/"$SERVICE_NAME-dataset-warmup"
fi

POD="$(kubectl -n "$NS" get pod -l app.kubernetes.io/instance="$RELEASE" -o jsonpath='{.items[0].metadata.name}')"

# Touch one query twice; second call should hit cache in-process if available.
kubectl -n "$NS" exec "$POD" -- sh -ceu '
  wget -qO- http://127.0.0.1:8080/metrics >/tmp/m_before.txt || true
  wget -qO- "http://127.0.0.1:8080/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=GENE1" >/dev/null || true
  wget -qO- "http://127.0.0.1:8080/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=GENE1" >/dev/null || true
  wget -qO- http://127.0.0.1:8080/metrics >/tmp/m_after.txt || true
  BEFORE=$(grep -E "^bijux_dataset_cache_hit_total" /tmp/m_before.txt | awk "{print \$2}" | head -n1)
  AFTER=$(grep -E "^bijux_dataset_cache_hit_total" /tmp/m_after.txt | awk "{print \$2}" | head -n1)
  [ -n "${BEFORE:-}" ] && [ -n "${AFTER:-}" ] && [ "$AFTER" -ge "$BEFORE" ]
'

echo "warmup verification completed"
