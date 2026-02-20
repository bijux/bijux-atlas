#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../../_lib/common.sh"
setup_test_traps
need curl

wait_ready
with_port_forward 18080
"$ROOT/ops/load/scripts/run_suite.sh" spike-overload-proof.json "$ROOT/artifacts/perf/results" >/dev/null

code="$(curl -s -o /tmp/atlas-overload-body.json -w '%{http_code}' \
  "$BASE_URL/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-999999999&limit=500" || true)"
case "$code" in
  200|422|429|503) ;;
  *)
    echo "failure_mode: overload_unexpected_status" >&2
    exit 1
    ;;
esac

if [ "$code" = "429" ] || [ "$code" = "503" ]; then
  grep -Eq '"code"\s*:\s*"(RateLimited|QueryRejectedByPolicy|NotReady)"' /tmp/atlas-overload-body.json || {
    echo "failure_mode: overload_missing_policy_code" >&2
    exit 1
  }
fi
echo "overload graceful degradation contract passed"
