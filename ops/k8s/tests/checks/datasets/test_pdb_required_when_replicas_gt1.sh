#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
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
