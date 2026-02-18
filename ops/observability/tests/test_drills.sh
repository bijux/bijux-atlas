#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
. "$ROOT/ops/observability/tests/observability-test-lib.sh"

require_bin python3

mapfile -t DRILLS < <(python3 - <<'PY'
import json
for d in json.load(open('ops/observability/drills/drills.json')).get('drills',[]):
    print(d['name'])
PY
)

for drill in "${DRILLS[@]}"; do
  echo "running drill from manifest: $drill"
  "$ROOT/ops/observability/scripts/run_drill.sh" "$drill"
done
"$ROOT/ops/observability/scripts/summarize_drill_results.py"
test -s "$ROOT/artifacts/observability/drill-conformance-report.json"

echo "observability drill manifest run passed"
