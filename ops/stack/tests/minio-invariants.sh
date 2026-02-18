#!/usr/bin/env bash
set -euo pipefail

check_minio_bootstrap_idempotent() {
  local root="$1"
  local ns="${2:-atlas-e2e}"
  kubectl get ns "$ns" >/dev/null 2>&1 || kubectl create ns "$ns" >/dev/null
  kubectl apply -f "$root/ops/stack/minio/minio.yaml" >/dev/null
  "$root/ops/stack/minio/bootstrap.sh"
  "$root/ops/stack/minio/bootstrap.sh"
  echo "minio bootstrap idempotent"
}

check_minio_bucket_policy() {
  local root="$1"
  local ns="${2:-atlas-e2e}"
  kubectl get ns "$ns" >/dev/null 2>&1 || kubectl create ns "$ns" >/dev/null
  kubectl apply -f "$root/ops/stack/minio/minio.yaml" >/dev/null
  "$root/ops/stack/minio/bootstrap.sh"
  kubectl -n "$ns" run minio-policy-check \
    --image=minio/mc:RELEASE.2025-01-17T23-25-50Z \
    --restart=Never --rm -i --command -- sh -ceu '
mc alias set local http://minio.'"$ns"'.svc.cluster.local:9000 minioadmin minioadmin >/dev/null
p=$(mc anonymous get local/atlas-artifacts)
echo "$p" | grep -Ei "download|readonly" >/dev/null
'
  echo "minio bucket policy enforced"
}

check_minio_reachable_from_atlas() {
  local ns="${1:-atlas-e2e}"
  local pod
  pod=$(kubectl -n "$ns" get pod -l app.kubernetes.io/name=bijux-atlas -o jsonpath='{.items[0].metadata.name}')
  kubectl -n "$ns" exec "$pod" -- sh -c "wget -qO- http://minio.$ns.svc.cluster.local:9000/minio/health/ready >/dev/null"
  echo "minio reachable from atlas pod"
}
