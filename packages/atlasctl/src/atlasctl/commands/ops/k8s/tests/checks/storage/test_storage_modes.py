#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need helm
need kubectl

helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set cache.storageMode=emptyDir > /tmp/storage-emptydir.yaml
grep -q "emptyDir:" /tmp/storage-emptydir.yaml

helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set cache.storageMode=pvc --set cache.pvc.size=2Gi > /tmp/storage-pvc.yaml
grep -q "kind: PersistentVolumeClaim" /tmp/storage-pvc.yaml
grep -q "persistentVolumeClaim:" /tmp/storage-pvc.yaml

install_chart --set cache.storageMode=pvc --set cache.pvc.size=1Gi
wait_ready
claim="$(kubectl -n "$NS" get pvc -o jsonpath='{.items[0].metadata.name}')"
[ -n "$claim" ] || { echo "no pvc created" >&2; exit 1; }
kubectl -n "$NS" wait --for=jsonpath='{.status.phase}'=Bound pvc/"$claim" --timeout=180s >/dev/null

echo "storage mode gate passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
