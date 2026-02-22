#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path
from ._shell_common import run_k8s_test_shell

def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need kubectl; need python3
wait_ready
kubectl -n "$NS" get pdb "$SERVICE_NAME" >/dev/null
MIN_AVAIL="$(kubectl -n "$NS" get pdb "$SERVICE_NAME" -o jsonpath='{.spec.minAvailable}')"
[ -n "$MIN_AVAIL" ] || { echo "pdb minAvailable missing" >&2; exit 1; }
POD="$(pod_name)"
[ -n "$POD" ] || { echo "no pod found for pdb test" >&2; exit 1; }
kubectl -n "$NS" create -f - >/dev/null <<YAML
apiVersion: policy/v1
kind: Eviction
metadata:
  name: ${POD}
  namespace: ${NS}
YAML
sleep 5
READY_REPLICAS="$(kubectl -n "$NS" get deploy "$SERVICE_NAME" -o jsonpath='{.status.readyReplicas}')"
python3 - "$MIN_AVAIL" "$READY_REPLICAS" <<'PY'
import sys
min_avail, ready = sys.argv[1], int(sys.argv[2] or "0")
try:
    required = int(min_avail)
except ValueError:
    required = 1
if ready < required:
    raise SystemExit(f"pdb violation: ready={ready} required>={required}")
PY
echo "pdb gate passed"
        """,
        Path(__file__),
    )

if __name__ == "__main__":
    raise SystemExit(main())
