#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need helm
need kubectl

# Template-level checks
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set cache.storageMode=emptyDir > /tmp/storage-emptydir.yaml
grep -q "emptyDir:" /tmp/storage-emptydir.yaml

helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set cache.storageMode=pvc --set cache.pvc.size=2Gi > /tmp/storage-pvc.yaml
grep -q "kind: PersistentVolumeClaim" /tmp/storage-pvc.yaml
grep -q "persistentVolumeClaim:" /tmp/storage-pvc.yaml

# Real PV behavior check: pvc mode should create and bind a claim on kind's default SC
install_chart --set cache.storageMode=pvc --set cache.pvc.size=1Gi
wait_ready
claim="$(kubectl -n "$NS" get pvc -o jsonpath='{.items[0].metadata.name}')"
[ -n "$claim" ] || { echo "no pvc created" >&2; exit 1; }
kubectl -n "$NS" wait --for=jsonpath='{.status.phase}'=Bound pvc/"$claim" --timeout=180s >/dev/null

echo "storage mode gate passed"
