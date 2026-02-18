#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: verify pack endpoints and atlas readiness via a single health command.
# stability: public
# called-by: make ops-observability-pack-health
set -euo pipefail
PROFILE="${ATLAS_OBS_PROFILE:-kind}"
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"
./ops/obs/scripts/verify_pack.sh --profile "$PROFILE"
curl -fsS "$ATLAS_BASE_URL/readyz" >/dev/null
echo "pack health ok (profile=$PROFILE)"
