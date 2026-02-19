#!/usr/bin/env bash
set -euo pipefail
NS="${1:-atlas-e2e}"
OUT="${2:-artifacts/ops/stack/health-report.txt}"
FORMAT="${ATLAS_HEALTH_REPORT_FORMAT:-text}"
mkdir -p "$(dirname "$OUT")"
if [ "$FORMAT" = "json" ]; then
  python3 - <<PY > "$OUT"
import json
import subprocess
from datetime import datetime, timezone

ns = "${NS}"

def run(*cmd: str):
    p = subprocess.run(cmd, capture_output=True, text=True)
    return {"ok": p.returncode == 0, "code": p.returncode, "stdout": p.stdout.strip(), "stderr": p.stderr.strip()}

payload = {
    "schema_version": 1,
    "namespace": ns,
    "timestamp": datetime.now(timezone.utc).isoformat(),
    "checks": {
        "nodes": run("kubectl", "get", "nodes", "-o", "name"),
        "pods": run("kubectl", "-n", ns, "get", "pods", "-o", "name"),
        "services": run("kubectl", "-n", ns, "get", "svc", "-o", "name"),
        "storageclass": run("kubectl", "get", "storageclass", "-o", "name"),
    },
}
print(json.dumps(payload, indent=2, sort_keys=True))
PY
else
  {
    echo "namespace=$NS"
    echo "timestamp=$(date -u +%FT%TZ)"
    echo "--- nodes ---"
    kubectl get nodes -o wide || true
    echo "--- pods ---"
    kubectl -n "$NS" get pods -o wide || true
    echo "--- services ---"
    kubectl -n "$NS" get svc || true
    echo "--- storageclass ---"
    kubectl get storageclass || true
  } > "$OUT"
fi
echo "$OUT"
