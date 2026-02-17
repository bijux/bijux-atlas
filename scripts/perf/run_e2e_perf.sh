#!/usr/bin/env sh
set -eu

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ART="$ROOT/artifacts/e2e/k6"
SCENARIOS="$ROOT/e2e/k6/scenarios"
BASE_URL="${BASE_URL:-http://127.0.0.1:18080}"
PR_MODE="${PR_MODE:-0}"

mkdir -p "$ART"

run_suite() {
  name="$1"
  suite="$2"
  if [ "$PR_MODE" = "1" ] && [ "$name" != "mixed" ] && [ "$name" != "cheap_only_survival" ]; then
    return 0
  fi
  OUT_DIR="$ART" "$ROOT/scripts/perf/run_suite.sh" "$suite" "$ART" >/dev/null
  src="$ART/${suite%.js}.summary.json"
  dst="$ART/$name.summary.json"
  [ -f "$src" ] && cp "$src" "$dst"
}

for spec in "$SCENARIOS"/*.json; do
  name="$(basename "$spec" .json)"
  suite="$(python3 - <<PY
import json
print(json.load(open('$spec')).get('suite',''))
PY
)"
  [ -n "$suite" ] || continue
  BASE_URL="$BASE_URL" run_suite "$name" "$suite"
done

# cold start result
if [ "$PR_MODE" != "1" ]; then
  OUT_DIR="$ART" "$ROOT/scripts/perf/cold_start_benchmark.sh" >/dev/null
  if [ -f "$ART/result.json" ]; then
    cp "$ART/result.json" "$ART/cold_start.result.json"
  fi
fi

# scrape metrics once for histogram/metric contract checks
authless_metrics="$(curl -fsS "$BASE_URL/metrics" || true)"
printf "%s\n" "$authless_metrics" > "$ART/metrics.prom"

"$ROOT/scripts/perf/score_k6.py"

echo "e2e perf complete: $ART"
