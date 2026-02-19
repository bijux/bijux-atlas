#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
. "$ROOT/ops/obs/tests/observability-test-lib.sh"

require_bin python3

DRILLS=()
while IFS= read -r line; do
  [ -n "$line" ] && DRILLS+=("$line")
done <<EOF
$(python3 - <<'PY'
import json
for d in json.load(open('ops/obs/drills/drills.json')).get('drills', []):
    print(d['name'])
PY
)
EOF

for drill in "${DRILLS[@]}"; do
  echo "running drill from manifest: $drill"
  "$ROOT/ops/obs/scripts/run_drill.sh" "$drill"
done
"$ROOT/ops/obs/scripts/summarize_drill_results.py"
test -s "$ROOT/artifacts/observability/drill-conformance-report.json"

echo "observability drill manifest run passed"
