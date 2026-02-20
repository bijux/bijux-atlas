#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../../_lib/common.sh"
setup_test_traps
need curl

install_chart
wait_ready
with_port_forward 18080
"$ROOT/ops/obs/scripts/bin/run_drill.sh" store-outage-under-load

curl -fsS "$BASE_URL/healthz" >/dev/null || { echo "failure_mode: store_outage_healthz_unavailable" >&2; exit 1; }
cached_code="$(curl -s -o /tmp/atlas-store-outage-cached.json -w '%{http_code}' \
  "$BASE_URL/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38" || true)"
uncached_code="$(curl -s -o /tmp/atlas-store-outage-uncached.json -w '%{http_code}' \
  "$BASE_URL/v1/genes/count?release=999&species=homo_sapiens&assembly=GRCh38" || true)"
case "$cached_code" in
  200|304) ;;
  *)
    echo "failure_mode: store_outage_cached_path_failed" >&2
    exit 1
    ;;
esac
case "$uncached_code" in
  400|422|429|503) ;;
  *)
    echo "failure_mode: store_outage_uncached_path_not_shed" >&2
    exit 1
    ;;
esac

echo "store outage under load cached-only protection contract passed"
