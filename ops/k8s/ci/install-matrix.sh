#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
OUT="$ROOT/artifacts/ops/k8s-install-matrix.json"
mkdir -p "$(dirname "$OUT")"
cat > "$OUT" <<JSON
{
  "generated_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "profiles": ["local", "offline", "perf", "ingress", "multi-registry"],
  "tests": ["install", "networkpolicy", "hpa", "pdb", "rollout", "rollback", "secrets", "configmap", "serviceMonitor"]
}
JSON
python3 "$ROOT/scripts/areas/docs/generate_k8s_install_matrix.py" "$OUT"
