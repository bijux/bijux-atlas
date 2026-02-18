#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: clean uninstall atlas and verify no namespace resources remain.
# stability: public
# called-by: make ops-clean-uninstall
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
NS="${ATLAS_E2E_NAMESPACE:-${ATLAS_NS:-atlas-e2e}}"
RELEASE="${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}"
helm -n "$NS" uninstall "$RELEASE" >/dev/null 2>&1 || true
kubectl delete ns "$NS" --ignore-not-found >/dev/null 2>&1 || true
for _ in $(seq 1 60); do
  if ! kubectl get ns "$NS" >/dev/null 2>&1; then
    break
  fi
  sleep 2
done
if kubectl get ns "$NS" >/dev/null 2>&1; then
  echo "namespace still exists after uninstall: $NS" >&2
  exit 1
fi
echo "clean uninstall complete: ns=$NS release=$RELEASE"
