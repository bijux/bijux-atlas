#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need helm

helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set cache.storageMode=emptyDir > /tmp/storage-emptydir.yaml
grep -q "emptyDir:" /tmp/storage-emptydir.yaml

helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set cache.storageMode=pvc --set cache.pvc.size=2Gi > /tmp/storage-pvc.yaml
grep -q "kind: PersistentVolumeClaim" /tmp/storage-pvc.yaml
grep -q "persistentVolumeClaim:" /tmp/storage-pvc.yaml

echo "storage mode gate passed"
