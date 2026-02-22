#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path
from ._shell_common import run_k8s_test_shell

def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need helm grep
if helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set replicaCount=2 --set pdb.enabled=false >/tmp/pdb-required-invalid.yaml 2>/tmp/pdb-required-invalid.err; then
  echo "expected render failure when replicaCount>1 and pdb.enabled=false" >&2
  exit 1
fi
grep -Eq "pdb.enabled must be true when replicaCount > 1" /tmp/pdb-required-invalid.err
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set replicaCount=2 --set pdb.enabled=true >/tmp/pdb-required-valid.yaml
grep -q "kind: PodDisruptionBudget" /tmp/pdb-required-valid.yaml
echo "pdb required when replicas>1 contract passed"
        """,
        Path(__file__),
    )

if __name__ == "__main__":
    raise SystemExit(main())
