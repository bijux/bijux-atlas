#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/_lib/common.sh
source "$(CDPATH= cd -- "$(dirname -- "$0")/../../_lib" && pwd)/common.sh"
# shellcheck source=ops/_lib/ports.sh
source "${REPO_ROOT}/ops/_lib/ports.sh"

PROFILE="${ATLAS_OBS_PROFILE:-kind}"
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"
if [ "${1:-}" = "--profile" ]; then
  PROFILE="${2:-}"
fi

for endpoint in \
  "${ATLAS_BASE_URL}/healthz" \
  "${ATLAS_BASE_URL}/v1/version" \
  "${ATLAS_BASE_URL}/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=3"
do
  curl -fsS "$endpoint" >/dev/null
done

"${REPO_ROOT}/ops/observability/scripts/snapshot_metrics.sh" "${REPO_ROOT}/artifacts/ops/observability"
"${REPO_ROOT}/ops/observability/scripts/snapshot_traces.sh" "${REPO_ROOT}/artifacts/ops/observability"

test -s "${REPO_ROOT}/artifacts/ops/observability/metrics.prom"
if [ "$PROFILE" != "local-compose" ]; then
  test -s "${REPO_ROOT}/artifacts/ops/observability/traces.snapshot.log"
fi

echo "observability pack smoke passed (profile=$PROFILE)"
