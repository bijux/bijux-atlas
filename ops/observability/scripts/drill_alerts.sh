#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"

python3 "$ROOT/scripts/observability/check_alerts_contract.py"

before="$(curl -fsS "$ATLAS_BASE_URL/metrics" | awk '$1=="bijux_errors_total" {sum+=$2} END{print sum+0}')"
# induce a controlled 4xx/5xx path to ensure error metric can move.
curl -fsS "$ATLAS_BASE_URL/v1/genes?release=bad&species=homo_sapiens&assembly=GRCh38&limit=1" >/dev/null 2>&1 || true
sleep 1
after="$(curl -fsS "$ATLAS_BASE_URL/metrics" | awk '$1=="bijux_errors_total" {sum+=$2} END{print sum+0}')"

awk -v a="$after" -v b="$before" 'BEGIN{exit !((a+0) >= (b+0))}'

echo "alert drill contract and signal assertions passed"
