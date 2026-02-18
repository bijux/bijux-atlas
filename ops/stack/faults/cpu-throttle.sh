#!/usr/bin/env bash
set -euo pipefail
ns="${ATLAS_NS:-atlas-e2e}"
if [ "${OPS_DRY_RUN:-0}" = "1" ]; then
  echo "DRY-RUN cpu throttle via stress pod in namespace $ns"
  exit 0
fi
kubectl -n "$ns" run cpu-throttle --restart=Never --rm -i --image=alpine:3.20 --limits='cpu=100m,memory=128Mi' --requests='cpu=100m,memory=64Mi' -- sh -ceu '
  apk add --no-cache stress-ng >/dev/null
  stress-ng --cpu 1 --cpu-load 95 --timeout 45s
'
echo "cpu throttle simulation completed"
