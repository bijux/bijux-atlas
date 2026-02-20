#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
. "$ROOT/ops/obs/tests/observability-test-lib.sh"

require_bin python3

OUT_DIR="$ROOT/artifacts/observability"
OPS_OBS_DIR="$ROOT/artifacts/ops/obs"
mkdir -p "$OUT_DIR" "$OPS_OBS_DIR"

DRILLS=()
while IFS= read -r line; do
  [ -n "$line" ] && DRILLS+=("$line")
done <<EOF
$(python3 - <<'PY'
import json
for d in json.load(open('ops/obs/drills/drills.json')).get('drills', []):
    if d.get('outage_matrix'):
        print(d['name'])
PY
)
EOF

for drill in "${DRILLS[@]}"; do
  echo "running outage drill: $drill"
  "$ROOT/ops/obs/scripts/bin/run_drill.sh" "$drill"
done

cp -f "$OPS_OBS_DIR"/metrics.prom "$OUT_DIR"/metrics.prom 2>/dev/null || true
cp -f "$OPS_OBS_DIR"/traces.snapshot.log "$OUT_DIR"/traces.snapshot.log 2>/dev/null || true
cp -f "$OPS_OBS_DIR"/traces.exemplars.log "$OUT_DIR"/traces.exemplars.log 2>/dev/null || true
test -s "$OUT_DIR/metrics.prom"
test -s "$OUT_DIR/traces.snapshot.log"
test -s "$OUT_DIR/traces.exemplars.log"

echo "observability outage matrix passed"
