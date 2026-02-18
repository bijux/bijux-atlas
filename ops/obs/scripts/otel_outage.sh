#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: run otel outage drill and assert atlas serve path remains available.
# stability: public
# called-by: make observability-pack-drills
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
"$ROOT/ops/obs/scripts/otel-outage.sh"
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"
curl -fsS "$ATLAS_BASE_URL/v1/version" >/dev/null
echo "otel outage drill preserved serve path"
