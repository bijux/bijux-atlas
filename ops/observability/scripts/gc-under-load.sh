#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: run gc while lightweight load is active and assert command completes.
# stability: public
# called-by: make observability-pack-drills
set -euo pipefail
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"
(for _ in $(seq 1 10); do curl -fsS "$ATLAS_BASE_URL/v1/version" >/dev/null || true; sleep 1; done) &
bg=$!
make ops-gc-smoke
wait "$bg" || true
echo "gc under load drill passed"
