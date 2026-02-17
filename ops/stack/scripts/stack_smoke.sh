#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
NS="${ATLAS_NS:-atlas-e2e}"
TIMEOUT="${ATLAS_E2E_TIMEOUT:-180s}"

"$ROOT/stack/scripts/wait_ready.sh" "$NS" "$TIMEOUT"
"$ROOT/stack/scripts/health_report.sh" "$NS" "artifacts/ops/stack/health-report.txt" >/dev/null

kubectl -n "$NS" get svc minio >/dev/null
kubectl -n "$NS" get svc prometheus >/dev/null

echo "stack-only smoke passed"
